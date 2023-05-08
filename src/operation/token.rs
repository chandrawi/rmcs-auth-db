use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::types::chrono::{DateTime, Utc};
use sea_query::{MysqlQueryBuilder, Query, Expr};
use sea_query_binder::SqlxBinder;

use crate::schema::auth_token::{AuthToken, TokenSchema};

enum TokenSelector {
    Role(u32),
    User(u32)
}

pub(crate) async fn select_token(pool: &Pool<MySql>, 
    id: &str
) -> Result<TokenSchema, Error>
{
    let (sql, values) = Query::select()
        .columns([
            AuthToken::Id,
            AuthToken::RoleId,
            AuthToken::UserId,
            AuthToken::Expire,
            AuthToken::Limit,
            AuthToken::Ip
        ])
        .from(AuthToken::Table)
        .and_where(Expr::col(AuthToken::Id).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    let row = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            TokenSchema {
                id: row.get(0),
                role_id: row.get(1),
                user_id: row.get(2),
                expire: row.get(3),
                limit: row.get(4),
                ip: row.get(5)
            }
        })
        .fetch_one(pool)
        .await?;

    Ok(row)
}

async fn select_multiple_token(pool: &Pool<MySql>, 
    selector: TokenSelector
) -> Result<Vec<TokenSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            AuthToken::Id,
            AuthToken::RoleId,
            AuthToken::UserId,
            AuthToken::Expire,
            AuthToken::Limit,
            AuthToken::Ip
        ])
        .from(AuthToken::Table)
        .to_owned();

    match selector {
        TokenSelector::Role(value) => {
            stmt = stmt.and_where(Expr::col((AuthToken::Table, AuthToken::RoleId)).eq(value)).to_owned();
        },
        TokenSelector::User(value) => {
            stmt = stmt.and_where(Expr::col((AuthToken::Table, AuthToken::UserId)).eq(value)).to_owned();
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let row = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            TokenSchema {
                id: row.get(0),
                role_id: row.get(1),
                user_id: row.get(2),
                expire: row.get(3),
                limit: row.get(4),
                ip: row.get(5)
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(row)
}

pub(crate) async fn select_multiple_token_by_role(pool: &Pool<MySql>,
    role_id: u32
) -> Result<Vec<TokenSchema>, Error>
{
    select_multiple_token(pool, TokenSelector::Role(role_id)).await
}

pub(crate) async fn select_multiple_token_by_user(pool: &Pool<MySql>,
    user_id: u32
) -> Result<Vec<TokenSchema>, Error>
{
    select_multiple_token(pool, TokenSelector::User(user_id)).await
}

pub(crate) async fn insert_token(pool: &Pool<MySql>, 
    id: &str, 
    role_id: u32, 
    user_id: u32, 
    expire: Option<DateTime<Utc>>, 
    limit: Option<u32>,
    ip: Option<Vec<u8>>
) -> Result<(), Error> 
{
    let (sql, values) = Query::insert()
        .into_table(AuthToken::Table)
        .columns([
            AuthToken::Id,
            AuthToken::RoleId,
            AuthToken::UserId,
            AuthToken::Expire,
            AuthToken::Limit,
            AuthToken::Ip
        ])
        .values([
            id.into(),
            role_id.into(),
            user_id.into(),
            expire.unwrap_or_default().into(),
            limit.unwrap_or_default().into(),
            ip.unwrap_or_default().into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_token(pool: &Pool<MySql>, 
    id: &str
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(AuthToken::Table)
        .and_where(Expr::col(AuthToken::Id).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
