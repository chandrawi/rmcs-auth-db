use sqlx::{Pool, Row, Error};
use sqlx::postgres::{Postgres, PgRow};
use sea_query::{PostgresQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;
use uuid::Uuid;

use crate::schema::profile::{ProfileRole, ProfileUser, RoleProfileSchema, UserProfileSchema, ProfileMode};
use rmcs_resource_db::schema::value::{DataValue, DataType};

pub(crate) async fn select_role_profile(pool: &Pool<Postgres>, 
    id: Option<i32>,
    role_id: Option<Uuid>,
) -> Result<Vec<RoleProfileSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            (ProfileRole::Table, ProfileRole::Id),
            (ProfileRole::Table, ProfileRole::RoleId),
            (ProfileRole::Table, ProfileRole::Name),
            (ProfileRole::Table, ProfileRole::Type),
            (ProfileRole::Table, ProfileRole::Mode)
        ])
        .to_owned();

    if let Some(id) = id {
        stmt = stmt.and_where(Expr::col((ProfileRole::Table, ProfileRole::Id)).eq(id)).to_owned();
    }
    else if let Some(role_id) = role_id {
        stmt = stmt.and_where(Expr::col((ProfileRole::Table, ProfileRole::RoleId)).eq(role_id)).to_owned();
    }
    let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: PgRow| {
            RoleProfileSchema {
                id: row.get(0),
                role_id: row.get(1),
                name: row.get(2),
                value_type: DataType::from(row.get::<i16,_>(3)),
                mode: ProfileMode::from(row.get::<i16,_>(4))
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub(crate) async fn insert_role_profile(pool: &Pool<Postgres>,
    role_id: Uuid,
    name: &str,
    value_type: DataType,
    mode: ProfileMode
) -> Result<i32, Error>
{
    let (sql, values) = Query::insert()
        .into_table(ProfileRole::Table)
        .columns([
            ProfileRole::RoleId,
            ProfileRole::Name,
            ProfileRole::Type,
            ProfileRole::Mode
        ])
        .values([
            role_id.into(),
            name.into(),
            i16::from(value_type).into(),
            i16::from(mode).into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(ProfileRole::Id)))
        .from(ProfileRole::Table)
        .to_string(PostgresQueryBuilder);
    let id: i32 = sqlx::query(&sql)
        .map(|row: PgRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_role_profile(pool: &Pool<Postgres>,
    id: i32,
    name: Option<&str>,
    value_type: Option<DataType>,
    mode: Option<ProfileMode>
) -> Result<(), Error>
{
    let mut stmt = Query::update()
        .table(ProfileRole::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(ProfileRole::Name, value).to_owned();
    }
    if let Some(value) = value_type {
        let value_type = i16::from(value);
        stmt = stmt.value(ProfileRole::Type, value_type).to_owned();
    }
    if let Some(value) = mode {
        let mode = i16::from(value);
        stmt = stmt.value(ProfileRole::Mode, mode).to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(ProfileRole::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_role_profile(pool: &Pool<Postgres>, 
    id: i32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(ProfileRole::Table)
        .and_where(Expr::col(ProfileRole::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn select_user_profile(pool: &Pool<Postgres>, 
    id: Option<i32>,
    user_id: Option<Uuid>,
) -> Result<Vec<UserProfileSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            (ProfileUser::Table, ProfileUser::Id),
            (ProfileUser::Table, ProfileUser::UserId),
            (ProfileUser::Table, ProfileUser::Name),
            (ProfileUser::Table, ProfileUser::Order),
            (ProfileUser::Table, ProfileUser::Value),
            (ProfileUser::Table, ProfileUser::Type)
        ])
        .to_owned();

    if let Some(id) = id {
        stmt = stmt.and_where(Expr::col((ProfileUser::Table, ProfileUser::Id)).eq(id)).to_owned();
    }
    else if let Some(user_id) = user_id {
        stmt = stmt.and_where(Expr::col((ProfileUser::Table, ProfileUser::UserId)).eq(user_id)).to_owned();
    }
    let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: PgRow| {
            let bytes = row.get(4);
            let type_ = DataType::from(row.get::<i16,_>(5));
            UserProfileSchema {
                id: row.get(0),
                user_id: row.get(1),
                name: row.get(2),
                value: DataValue::from_bytes(bytes, type_),
                order: row.get(3)
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub(crate) async fn insert_user_profile(pool: &Pool<Postgres>,
    user_id: Uuid,
    name: &str,
    value: DataValue
) -> Result<i32, Error>
{
    let (sql, values) = Query::select()
        .expr(Func::max(Expr::col(ProfileUser::Order)))
        .and_where(Expr::col(ProfileUser::UserId).eq(user_id))
        .and_where(Expr::col(ProfileUser::Name).eq(name))
        .from(ProfileUser::Table)
        .build_sqlx(PostgresQueryBuilder);
    let order: i16 = sqlx::query_with(&sql, values)
        .map(|row: PgRow| row.try_get(0))
        .fetch_one(pool)
        .await
        .unwrap_or(Ok(-1))
        .unwrap_or(-1);
    // new profile order is max order of profile with input user_id and name plus one
    // new profile order is zero if no profile with input user_id and name found
    let order = order + 1;

    let bytes = value.to_bytes();
    let type_ = i16::from(value.get_type());
    let (sql, values) = Query::insert()
        .into_table(ProfileUser::Table)
        .columns([
            ProfileUser::UserId,
            ProfileUser::Name,
            ProfileUser::Order,
            ProfileUser::Value,
            ProfileUser::Type
        ])
        .values([
            user_id.into(),
            name.into(),
            order.into(),
            bytes.into(),
            type_.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(ProfileUser::Id)))
        .from(ProfileUser::Table)
        .to_string(PostgresQueryBuilder);
    let id: i32 = sqlx::query(&sql)
        .map(|row: PgRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_user_profile(pool: &Pool<Postgres>,
    id: i32,
    name: Option<&str>,
    value: Option<DataValue>
) -> Result<(), Error>
{
    let mut stmt = Query::update()
        .table(ProfileUser::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(ProfileUser::Name, value).to_owned();
    }
    if let Some(value) = value {
        let bytes = value.to_bytes();
        let type_ = i16::from(value.get_type());
        stmt = stmt
            .value(ProfileUser::Value, bytes)
            .value(ProfileUser::Type, type_)
            .to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(ProfileUser::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_user_profile(pool: &Pool<Postgres>, 
    id: i32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(ProfileUser::Table)
        .and_where(Expr::col(ProfileUser::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
