use sqlx::{Pool, Row, Error};
use sqlx::postgres::{Postgres, PgRow};
use sea_query::{PostgresQueryBuilder, Query, Expr, Order};
use sea_query_binder::SqlxBinder;
use uuid::Uuid;

use crate::schema::auth_user::{User, UserRole, UserSchema, UserRoleSchema};
use crate::schema::auth_role::Role;
use crate::schema::api::Api;
use crate::utility;

pub(crate) async fn select_user(pool: &Pool<Postgres>, 
    id: Option<Uuid>,
    api_id: Option<Uuid>,
    role_id: Option<Uuid>,
    name_exact: Option<&str>,
    name_like: Option<&str>
) -> Result<Vec<UserSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            (User::Table, User::UserId),
            (User::Table, User::Name),
            (User::Table, User::Password),
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

    if let Some(id) = id {
        stmt = stmt.and_where(Expr::col((User::Table, User::UserId)).eq(id)).to_owned();
    }
    else if let Some(name) = name_exact {
        stmt = stmt.and_where(Expr::col((User::Table, User::Name)).eq(name.to_owned())).to_owned();
    }
    else {
        if let Some(api_id) = api_id {
            stmt = stmt.and_where(Expr::col((Api::Table, Api::ApiId)).eq(api_id)).to_owned();
        }
        if let Some(role_id) = role_id {
            stmt = stmt.and_where(Expr::col((Role::Table, Role::RoleId)).eq(role_id)).to_owned();
        }
        if let Some(name) = name_like {
            let name_like = String::from("%") + name + "%";
            stmt = stmt.and_where(Expr::col((User::Table, User::Name)).like(name_like)).to_owned();
        }
    }

    let (sql, values) = stmt
        .order_by((User::Table, User::UserId), Order::Asc)
        .order_by((UserRole::Table, UserRole::RoleId), Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let mut last_id: Option<Uuid> = None;
    let mut user_schema_vec: Vec<UserSchema> = Vec::new();

    sqlx::query_with(&sql, values)
        .map(|row: PgRow| {
            // get last user_schema in user_schema_vec or default
            let mut user_schema = user_schema_vec.pop().unwrap_or_default();
            // on every new user_id found update last_id and insert new user_schema to user_schema_vec
            let user_id: Uuid = row.get(0);
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
            user_schema.email = row.get(3);
            user_schema.phone = row.get(4);
            // on every new role_id found add a role to user_schema
            let role_name = row.try_get(6).ok();
            if let Some(name) = role_name {
                user_schema.roles.push(UserRoleSchema {
                    api_id: row.get(5),
                    role: name,
                    multi: row.get(7),
                    ip_lock: row.get(8),
                    access_duration: row.get(9),
                    refresh_duration: row.get(10),
                    access_key: row.get(11)
                });
            }
            // update api_schema_vec with updated user_schema
            user_schema_vec.push(user_schema);
        })
        .fetch_all(pool)
        .await?;

    Ok(user_schema_vec)
}

pub(crate) async fn insert_user(pool: &Pool<Postgres>, 
    id: Uuid,
    name: &str, 
    email: &str,
    phone: &str,
    password: &str
) -> Result<Uuid, Error> 
{
    let password_hash = utility::hash_password(&password).or(Err(Error::WorkerCrashed))?;

    let (sql, values) = Query::insert()
        .into_table(User::Table)
        .columns([
            User::UserId,
            User::Name,
            User::Password,
            User::Email,
            User::Phone
        ])
        .values([
            id.into(),
            name.into(),
            password_hash.into(),
            email.into(),
            phone.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_user(pool: &Pool<Postgres>, 
    id: Uuid, 
    name: Option<&str>, 
    email: Option<&str>,
    phone: Option<&str>,
    password: Option<&str>
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

    let (sql, values) = stmt
        .and_where(Expr::col(User::UserId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_user(pool: &Pool<Postgres>, 
    id: Uuid
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(User::Table)
        .and_where(Expr::col(User::UserId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn add_user_role(pool: &Pool<Postgres>, 
    id: Uuid,
    role_id: Uuid
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
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn remove_user_role(pool: &Pool<Postgres>, 
    id: Uuid,
    role_id: Uuid
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(UserRole::Table)
        .and_where(Expr::col(UserRole::UserId).eq(id))
        .and_where(Expr::col(UserRole::RoleId).eq(role_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
