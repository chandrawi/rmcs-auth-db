use sqlx::{Pool, Row, Error};
use sqlx::postgres::{Postgres, PgRow};
use sqlx::types::chrono::{DateTime, Utc};
use sea_query::{PostgresQueryBuilder, Query, Expr, Order, Func};
use sea_query_binder::SqlxBinder;
use uuid::Uuid;

use crate::schema::auth_token::{Token, TokenSchema};
use crate::utility;

pub(crate) enum TokenSelector {
    Access(i32),
    Auth(String),
    User(Uuid)
}

pub(crate) async fn select_token(pool: &Pool<Postgres>, 
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
    let (sql, values) = stmt
        .order_by(Token::AccessId, Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let row = sqlx::query_with(&sql, values)
        .map(|row: PgRow| {
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

pub(crate) async fn insert_token(pool: &Pool<Postgres>, 
    user_id: Uuid, 
    auth_token: Option<&str>,
    expire: DateTime<Utc>, 
    ip: &[u8],
    number: u32
) -> Result<Vec<(i32, String, String)>, Error> 
{
    let sql = Query::select()
        .expr(Func::max(Expr::col(Token::AccessId)))
        .from(Token::Table)
        .to_string(PostgresQueryBuilder);
    let mut access_id: i32 = sqlx::query(&sql)
        .map(|row: PgRow| row.try_get(0))
        .fetch_one(pool)
        .await
        .unwrap_or(Ok(0))
        .unwrap_or(0);

    let auth_token = match auth_token {
        Some(value) => value.to_owned(),
        None => utility::generate_token_string()
    };
    let gens: Vec<(i32, String, String)> = (0..number).map(|_| {
        access_id = if access_id < i32::MAX { access_id + 1 } else { 1 };
        let refresh_token = utility::generate_token_string();
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
    let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(gens)
}

pub(crate) async fn update_token(pool: &Pool<Postgres>, 
    access_id: Option<i32>,
    auth_token: Option<&str>,
    expire: Option<DateTime<Utc>>, 
    ip: Option<&[u8]>
) -> Result<(String, String), Error> 
{
    let refresh_token = utility::generate_token_string();
    let (auth_token, flag) = match auth_token {
        Some(value) => (value.to_owned(), true),
        None => (utility::generate_token_string(), false)
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
    let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok((refresh_token, auth_token))
}

pub(crate) async fn delete_token(pool: &Pool<Postgres>, 
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
    let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
