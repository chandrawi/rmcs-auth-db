use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::types::chrono::{DateTime, Utc};
use sea_query::{MysqlQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::auth_token::{Token, TokenSchema};
use crate::utility;

enum TokenSelector {
    Access(u32),
    Refresh(String),
    User(u32)
}

async fn select_token(pool: &Pool<MySql>, 
    selector: TokenSelector
) -> Result<Vec<TokenSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            Token::AccessId,
            Token::RefreshId,
            Token::UserId,
            Token::Expire,
            Token::Ip
        ])
        .from(Token::Table)
        .to_owned();

    match selector {
        TokenSelector::Access(value) => {
            stmt = stmt.and_where(Expr::col(Token::AccessId).eq(value)).to_owned();
        },
        TokenSelector::Refresh(value) => {
            stmt = stmt.and_where(Expr::col(Token::RefreshId).eq(value)).to_owned();
        },
        TokenSelector::User(value) => {
            stmt = stmt.and_where(Expr::col(Token::UserId).eq(value)).to_owned();
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let row = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            TokenSchema {
                access_id: row.get(0),
                refresh_id: row.get(1),
                user_id: row.get(2),
                expire: row.get(3),
                ip: row.get(4)
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(row)
}

pub(crate) async fn select_token_by_access(pool: &Pool<MySql>,
    access_id: u32
) -> Result<TokenSchema, Error>
{
    match select_token(pool, TokenSelector::Access(access_id)).await?.into_iter().next() {
        Some(value) => Ok(value),
        None => Err(Error::RowNotFound)
    }
}

pub(crate) async fn select_token_by_refresh(pool: &Pool<MySql>,
    refresh_id: &str
) -> Result<TokenSchema, Error>
{
    match select_token(pool, TokenSelector::Refresh(String::from(refresh_id))).await?.into_iter().next() {
        Some(value) => Ok(value),
        None => Err(Error::RowNotFound)
    }
}

pub(crate) async fn select_token_by_user(pool: &Pool<MySql>,
    user_id: u32
) -> Result<Vec<TokenSchema>, Error>
{
    select_token(pool, TokenSelector::User(user_id)).await
}

pub(crate) async fn insert_token(pool: &Pool<MySql>, 
    access_id: Option<u32>,
    user_id: u32, 
    expire: DateTime<Utc>, 
    ip: &[u8]
) -> Result<(u32, String), Error> 
{
    let refresh_id = utility::generate_random_base64(32);

    let access_id = if let Some(value) = access_id {
        value
    } else {
        let sql = Query::select()
            .expr(Func::max(Expr::col(Token::AccessId)))
            .from(Token::Table)
            .to_string(MysqlQueryBuilder);
        let id: u32 = sqlx::query(&sql)
            .map(|row: MySqlRow| row.try_get(0))
            .fetch_one(pool)
            .await
            .unwrap_or(Ok(0))
            .unwrap_or(0);
        if id < u32::MAX { id + 1 } else { 1 }
    };

    let (sql, values) = Query::insert()
        .into_table(Token::Table)
        .columns([
            Token::RefreshId,
            Token::AccessId,
            Token::UserId,
            Token::Expire,
            Token::Ip
        ])
        .values([
            refresh_id.clone().into(),
            access_id.into(),
            user_id.into(),
            expire.into(),
            ip.to_vec().into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok((access_id, refresh_id))
}

pub(crate) async fn update_token(pool: &Pool<MySql>, 
    access_id: Option<u32>,
    refresh_id: Option<&str>,
    expire: Option<DateTime<Utc>>, 
    ip: Option<&[u8]>
) -> Result<String, Error> 
{
    let new_refresh_id = utility::generate_random_base64(32);

    let mut stmt = Query::update()
        .table(Token::Table)
        .to_owned();
    if let Some(value) = expire {
        stmt = stmt.value(Token::Expire, value).to_owned();
    }
    if let Some(value) = ip {
        stmt = stmt.value(Token::Ip, value).to_owned();
    }

    if let Some(value) = refresh_id {
        if let Some(v) = access_id {
            stmt = stmt.value(Token::AccessId, v).to_owned();
        }
        stmt = stmt.and_where(Expr::col(Token::RefreshId).eq(value)).to_owned();
    } else {
        stmt = stmt
            .value(Token::RefreshId, new_refresh_id.clone())
            .and_where(Expr::col(Token::AccessId).eq(access_id.unwrap()))
            .to_owned();
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(
        match refresh_id {
            Some(value) => value.to_owned(),
            None => new_refresh_id
        }
    )
}

async fn delete_token(pool: &Pool<MySql>, 
    selector: TokenSelector
) -> Result<(), Error> 
{
    let mut stmt = Query::delete()
        .from_table(Token::Table)
        .to_owned();
    match selector {
        TokenSelector::Refresh(value) => {
            stmt = stmt.and_where(Expr::col((Token::Table, Token::RefreshId)).eq(value)).to_owned();
        },
        TokenSelector::Access(value) => {
            stmt = stmt.and_where(Expr::col((Token::Table, Token::AccessId)).eq(value)).to_owned();
        },
        TokenSelector::User(value) => {
            stmt = stmt.and_where(Expr::col((Token::Table, Token::UserId)).eq(value)).to_owned();
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_token_by_access(pool: &Pool<MySql>,
    access_id: u32
) -> Result<(), Error>
{
    delete_token(pool, TokenSelector::Access(access_id)).await
}

pub(crate) async fn delete_token_by_refresh(pool: &Pool<MySql>,
    refresh_id: &str,
) -> Result<(), Error>
{
    delete_token(pool, TokenSelector::Refresh(String::from(refresh_id))).await
}

pub(crate) async fn delete_token_by_user(pool: &Pool<MySql>,
    user_id: u32
) -> Result<(), Error>
{
    delete_token(pool, TokenSelector::User(user_id)).await
}
