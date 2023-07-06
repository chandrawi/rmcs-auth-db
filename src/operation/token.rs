use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::types::chrono::{DateTime, Utc};
use sea_query::{MysqlQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::auth_token::{Token, TokenSchema};
use crate::utility;

enum TokenSelector {
    Access(u32),
    Auth(String),
    User(u32)
}

async fn select_token(pool: &Pool<MySql>, 
    selector: TokenSelector
) -> Result<Vec<TokenSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            Token::AccessId,
            Token::UserId,
            Token::RefreshToken,
            Token::AuthToken,
            Token::Expire,
            Token::Ip
        ])
        .from(Token::Table)
        .to_owned();

    match selector {
        TokenSelector::Access(value) => {
            stmt = stmt.and_where(Expr::col(Token::AccessId).eq(value)).to_owned();
        },
        TokenSelector::Auth(value) => {
            stmt = stmt.and_where(Expr::col(Token::AuthToken).eq(value)).to_owned();
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
                user_id: row.get(1),
                refresh_token: row.get(2),
                auth_token: row.get(3),
                expire: row.get(4),
                ip: row.get(5)
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

pub(crate) async fn select_token_by_auth(pool: &Pool<MySql>,
    auth_token: &str
) -> Result<Vec<TokenSchema>, Error>
{
    select_token(pool, TokenSelector::Auth(String::from(auth_token))).await
}

pub(crate) async fn select_token_by_user(pool: &Pool<MySql>,
    user_id: u32
) -> Result<Vec<TokenSchema>, Error>
{
    select_token(pool, TokenSelector::User(user_id)).await
}

pub(crate) async fn insert_token(pool: &Pool<MySql>, 
    user_id: u32, 
    auth_token: Option<&str>,
    expire: DateTime<Utc>, 
    ip: &[u8],
    number: u32
) -> Result<Vec<(u32, String, String)>, Error> 
{
    let sql = Query::select()
        .expr(Func::max(Expr::col(Token::AccessId)))
        .from(Token::Table)
        .to_string(MysqlQueryBuilder);
    let mut access_id: u32 = sqlx::query(&sql)
        .map(|row: MySqlRow| row.try_get(0))
        .fetch_one(pool)
        .await
        .unwrap_or(Ok(0))
        .unwrap_or(0);

    let auth_token = match auth_token {
        Some(value) => value.to_owned(),
        None => utility::generate_random_base64(32)
    };
    let gens: Vec<(u32, String, String)> = (0..number).map(|_| {
        access_id = if access_id < u32::MAX { access_id + 1 } else { 1 };
        let refresh_token = utility::generate_random_base64(32);
        (access_id, refresh_token, auth_token.clone())
    })
    .collect();

    let mut stmt = Query::insert()
        .into_table(Token::Table)
        .columns([
            Token::AccessId,
            Token::UserId,
            Token::RefreshToken,
            Token::AuthToken,
            Token::Expire,
            Token::Ip
        ])
        .to_owned();
    for gen in gens.clone() {
        stmt = stmt.values([
            gen.0.into(),
            user_id.into(),
            gen.1.into(),
            gen.2.into(),
            expire.into(),
            ip.to_vec().into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .to_owned();
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(gens)
}

pub(crate) async fn update_token(pool: &Pool<MySql>, 
    access_id: Option<u32>,
    auth_token: Option<&str>,
    expire: Option<DateTime<Utc>>, 
    ip: Option<&[u8]>
) -> Result<(String, String), Error> 
{
    let refresh_token = utility::generate_random_base64(32);
    let (auth_token, flag) = match auth_token {
        Some(value) => (value.to_owned(), true),
        None => (utility::generate_random_base64(32), false)
    };

    let mut stmt = Query::update()
        .table(Token::Table)
        .value(Token::RefreshToken, refresh_token.clone()).to_owned()
        .to_owned();
    if let Some(value) = expire {
        stmt = stmt.value(Token::Expire, value).to_owned();
    }
    if let Some(value) = ip {
        stmt = stmt.value(Token::Ip, value).to_owned();
    }

    if let Some(value) = access_id {
        if flag {
            stmt = stmt.value(Token::AuthToken, auth_token.clone()).to_owned();
        }
        stmt = stmt.and_where(Expr::col(Token::AccessId).eq(value)).to_owned();
    } else {
        stmt = stmt.and_where(Expr::col(Token::AuthToken).eq(auth_token.clone())).to_owned();
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok((refresh_token, auth_token))
}

async fn delete_token(pool: &Pool<MySql>, 
    selector: TokenSelector
) -> Result<(), Error> 
{
    let mut stmt = Query::delete()
        .from_table(Token::Table)
        .to_owned();
    match selector {
        TokenSelector::Access(value) => {
            stmt = stmt.and_where(Expr::col((Token::Table, Token::AccessId)).eq(value)).to_owned();
        },
        TokenSelector::Auth(value) => {
            stmt = stmt.and_where(Expr::col((Token::Table, Token::AuthToken)).eq(value)).to_owned();
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

pub(crate) async fn delete_token_by_auth(pool: &Pool<MySql>,
    auth_token: &str,
) -> Result<(), Error>
{
    delete_token(pool, TokenSelector::Auth(String::from(auth_token))).await
}

pub(crate) async fn delete_token_by_user(pool: &Pool<MySql>,
    user_id: u32
) -> Result<(), Error>
{
    delete_token(pool, TokenSelector::User(user_id)).await
}
