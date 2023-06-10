use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sea_query::{MysqlQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::auth_user::{User, UserRole, UserSchema, UserRoleSchema};
use crate::schema::auth_role::Role;
use crate::schema::api::Api;
use crate::utility;

enum UserSelector {
    Id(u32),
    Name(String),
    Role(u32)
}

async fn select_user(pool: &Pool<MySql>, 
    selector: UserSelector
) -> Result<Vec<UserSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            (User::Table, User::UserId),
            (User::Table, User::Name),
            (User::Table, User::Password),
            (User::Table, User::PublicKey),
            (User::Table, User::PrivateKey),
            (User::Table, User::Email),
            (User::Table, User::Phone)
        ])
        .columns([
            (Role::Table, Role::ApiId),
            (Role::Table, Role::Name),
            (Role::Table, Role::Multi),
            (Role::Table, Role::IpLock),
            (Role::Table, Role::AccessDuration),
            (Role::Table, Role::RefreshDuration)
        ])
        .columns([
            (Api::Table, Api::AccessKey)
        ])
        .from(User::Table)
        .left_join(UserRole::Table,
            Expr::col((User::Table, User::UserId))
            .equals((UserRole::Table, UserRole::UserId))
        )
        .left_join(Role::Table,
            Expr::col((UserRole::Table, UserRole::RoleId))
            .equals((Role::Table, Role::RoleId))
        )
        .left_join(Api::Table,
            Expr::col((Role::Table, Role::ApiId))
            .equals((Api::Table, Api::ApiId))
        )
        .to_owned();

    match selector {
        UserSelector::Id(value) => {
            stmt = stmt.and_where(Expr::col((User::Table, User::UserId)).eq(value)).to_owned();
        },
        UserSelector::Name(value) => {
            stmt = stmt.and_where(Expr::col((User::Table, User::Name)).eq(value)).to_owned();
        }
        UserSelector::Role(value) => {
            stmt = stmt.and_where(Expr::col((Role::Table, Role::RoleId)).eq(value)).to_owned();
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let mut last_id: Option<u32> = None;
    let mut user_schema_vec: Vec<UserSchema> = Vec::new();

    sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            // get last user_schema in user_schema_vec or default
            let mut user_schema = user_schema_vec.pop().unwrap_or_default();
            // on every new user_id found update last_id and insert new user_schema to user_schema_vec
            let user_id: u32 = row.get(0);
            if let Some(value) = last_id {
                if value != user_id {
                    user_schema_vec.push(user_schema.clone());
                    user_schema = UserSchema::default();
                }
            }
            last_id = Some(user_id);
            user_schema.id = user_id;
            user_schema.name = row.get(1);
            user_schema.password = row.get(2);
            user_schema.public_key = row.get(3);
            user_schema.private_key = row.get(4);
            user_schema.email = row.get(5);
            user_schema.phone = row.get(6);
            // on every new role_id found add a role to user_schema
            let role_name = row.try_get(8).ok();
            if let Some(name) = role_name {
                user_schema.roles.push(UserRoleSchema {
                    api_id: row.get(7),
                    role: name,
                    multi: row.get(9),
                    ip_lock: row.get(10),
                    access_duration: row.get(11),
                    refresh_duration: row.get(12),
                    access_key: row.get(13)
                });
            }
            // update api_schema_vec with updated user_schema
            user_schema_vec.push(user_schema);
        })
        .fetch_all(pool)
        .await?;

    Ok(user_schema_vec)
}

pub(crate) async fn select_user_by_id(pool: &Pool<MySql>,
    id: u32
) -> Result<UserSchema, Error>
{
    let users = select_user(pool, UserSelector::Id(id)).await;
    match users {
        Ok(value) => value.into_iter().next().ok_or(Error::RowNotFound),
        Err(e) => Err(e)
    }
}

pub(crate) async fn select_user_by_name(pool: &Pool<MySql>,
    name: &str
) -> Result<UserSchema, Error>
{
    let users = select_user(pool, UserSelector::Name(name.to_owned())).await;
    match users {
        Ok(value) => value.into_iter().next().ok_or(Error::RowNotFound),
        Err(e) => Err(e)
    }
}

pub(crate) async fn select_user_by_role(pool: &Pool<MySql>,
    role_id: u32
) -> Result<Vec<UserSchema>, Error>
{
    select_user(pool, UserSelector::Role(role_id)).await
}

pub(crate) async fn insert_user(pool: &Pool<MySql>, 
    name: &str, 
    email: &str,
    phone: &str,
    password: &str
) -> Result<u32, Error> 
{
    let password_hash = utility::hash_password(&password).or(Err(Error::WorkerCrashed))?;

    let (priv_key, pub_key) = utility::generate_keys().or(Err(Error::WorkerCrashed))?;
    let priv_der = utility::export_private_key(priv_key).or(Err(Error::WorkerCrashed))?;
    let pub_der = utility::export_public_key(pub_key).or(Err(Error::WorkerCrashed))?;

    let (sql, values) = Query::insert()
        .into_table(User::Table)
        .columns([
            User::Name,
            User::Password,
            User::PublicKey,
            User::PrivateKey,
            User::Email,
            User::Phone
        ])
        .values([
            name.into(),
            password_hash.into(),
            pub_der.into(),
            priv_der.into(),
            email.into(),
            phone.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(User::UserId)))
        .from(User::Table)
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
    email: Option<&str>,
    phone: Option<&str>,
    password: Option<&str>, 
    keys: Option<()>
) -> Result<(), Error> 
{
    let mut stmt = Query::update()
        .table(User::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(User::Name, value).to_owned();
    }
    if let Some(value) = email {
        stmt = stmt.value(User::Email, value).to_owned();
    }
    if let Some(value) = phone {
        stmt = stmt.value(User::Phone, value).to_owned();
    }
    if let Some(value) = password {
        let password_hash = utility::hash_password(value).or(Err(Error::WorkerCrashed))?;
        stmt = stmt.value(User::Password, password_hash).to_owned();
    }
    if let Some(_) = keys {
        let (priv_key, pub_key) = utility::generate_keys().or(Err(Error::WorkerCrashed))?;
        let priv_der = utility::export_private_key(priv_key).or(Err(Error::WorkerCrashed))?;
        let pub_der = utility::export_public_key(pub_key).or(Err(Error::WorkerCrashed))?;
        stmt = stmt
            .value(User::PublicKey, pub_der)
            .value(User::PrivateKey, priv_der)
            .to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(User::UserId).eq(id))
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
        .from_table(User::Table)
        .and_where(Expr::col(User::UserId).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn add_user_role(pool: &Pool<MySql>, 
    id: u32,
    role_id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::insert()
        .into_table(UserRole::Table)
        .columns([
            UserRole::UserId,
            UserRole::RoleId
        ])
        .values([
            id.into(),
            role_id.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn remove_user_role(pool: &Pool<MySql>, 
    id: u32,
    role_id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(UserRole::Table)
        .and_where(Expr::col(UserRole::UserId).eq(id))
        .and_where(Expr::col(UserRole::RoleId).eq(role_id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
