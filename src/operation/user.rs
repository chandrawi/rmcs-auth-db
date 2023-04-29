use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sea_query::{MysqlQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::auth_user::{AuthUser, UserFields};

enum UserSelector {
    Role(u32),
    User(u32),
    Name(String)
}

async fn select_user(pool: &Pool<MySql>, 
    selector: UserSelector
) -> Result<Vec<UserFields>, Error>
{
    let mut stmt = Query::select()
        .columns([
            AuthUser::RoleId,
            AuthUser::UserId,
            AuthUser::User,
            AuthUser::Password,
            AuthUser::PublicKey,
            AuthUser::PrivateKey,
            AuthUser::Email,
            AuthUser::Phone
        ])
        .from(AuthUser::Table)
        .to_owned();

    match selector {
        UserSelector::Role(value) => {
            stmt = stmt.and_where(Expr::col(AuthUser::RoleId).eq(value)).to_owned();
        },
        UserSelector::User(value) => {
            stmt = stmt.and_where(Expr::col(AuthUser::UserId).eq(value)).to_owned();
        },
        UserSelector::Name(value) => {
            stmt = stmt.and_where(Expr::col(AuthUser::User).eq(value)).to_owned();
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            UserFields {
                role_id: row.get(0),
                id: row.get(1),
                name: row.get(2),
                password: row.get(3),
                public_key: row.get(4),
                private_key: row.get(5),
                email: row.get(6),
                phone: row.get(7)
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub(crate) async fn select_user_by_id(pool: &Pool<MySql>,
    id: u32
) -> Result<UserFields, Error>
{
    let users = select_user(pool, UserSelector::User(id)).await;
    match users {
        Ok(value) => value.into_iter().next().ok_or(Error::RowNotFound),
        Err(e) => Err(e)
    }
}

pub(crate) async fn select_user_by_name(pool: &Pool<MySql>,
    name: &str
) -> Result<UserFields, Error>
{
    let users = select_user(pool, UserSelector::Name(name.to_owned())).await;
    match users {
        Ok(value) => value.into_iter().next().ok_or(Error::RowNotFound),
        Err(e) => Err(e)
    }
}

pub(crate) async fn select_multiple_user_by_role(pool: &Pool<MySql>,
    role_id: u32
) -> Result<Vec<UserFields>, Error>
{
    select_user(pool, UserSelector::Role(role_id)).await
}

pub(crate) async fn insert_user(pool: &Pool<MySql>, 
    role_id: u32, 
    name: &str, 
    password: &str, 
    public_key: &str, 
    private_key: &str,
    email: Option<&str>,
    phone: Option<&str>
) -> Result<u32, Error> 
{
    let (sql, values) = Query::insert()
        .into_table(AuthUser::Table)
        .columns([
            AuthUser::RoleId,
            AuthUser::User,
            AuthUser::Password,
            AuthUser::PublicKey,
            AuthUser::PrivateKey,
            AuthUser::Email,
            AuthUser::Phone
        ])
        .values([
            role_id.into(),
            name.into(),
            password.into(),
            public_key.into(),
            private_key.into(),
            email.unwrap_or_default().into(),
            phone.unwrap_or_default().into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(AuthUser::UserId)))
        .from(AuthUser::Table)
        .to_string(MysqlQueryBuilder);
    let id: u32 = sqlx::query(&sql)
        .map(|row: MySqlRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_user(pool: &Pool<MySql>, 
    id: u32, 
    name: Option<&str>, 
    password: Option<&str>, 
    public_key: Option<&str>, 
    private_key: Option<&str>,
    email: Option<&str>,
    phone: Option<&str>
) -> Result<(), Error> 
{
    let mut stmt = Query::update()
        .table(AuthUser::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(AuthUser::User, value).to_owned();
    }
    if let Some(value) = password {
        stmt = stmt.value(AuthUser::Password, value).to_owned();
    }
    if let Some(value) = public_key {
        stmt = stmt.value(AuthUser::PublicKey, value).to_owned();
    }
    if let Some(value) = private_key {
        stmt = stmt.value(AuthUser::PrivateKey, value).to_owned();
    }
    if let Some(value) = email {
        stmt = stmt.value(AuthUser::Email, value).to_owned();
    }
    if let Some(value) = phone {
        stmt = stmt.value(AuthUser::Phone, value).to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(AuthUser::RoleId).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_user(pool: &Pool<MySql>, 
    id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(AuthUser::Table)
        .and_where(Expr::col(AuthUser::UserId).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
