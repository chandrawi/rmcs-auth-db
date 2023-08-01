use sqlx::{Pool, Row, Error};
use sqlx::postgres::{Postgres, PgRow};
use sea_query::{PostgresQueryBuilder, Query, Expr, Order};
use sea_query_binder::SqlxBinder;
use uuid::Uuid;

use crate::schema::auth_role::{Role, RoleAccess, RoleSchema};
use crate::schema::api::Api;
use crate::schema::auth_user::UserRole;

enum RoleSelector {
    Id(Uuid),
    Name(Uuid, String),
    Api(Uuid),
    User(Uuid)
}

async fn select_role(pool: &Pool<Postgres>, 
    selector: RoleSelector
) -> Result<Vec<RoleSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            (Role::Table, Role::RoleId),
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
        .columns([
            (RoleAccess::Table, RoleAccess::ProcedureId)
        ])
        .from(Role::Table)
        .inner_join(Api::Table, 
            Expr::col((Role::Table, Role::ApiId))
            .equals((Api::Table, Api::ApiId))
        )
        .left_join(RoleAccess::Table, 
            Expr::col((Role::Table, Role::RoleId))
            .equals((RoleAccess::Table, RoleAccess::RoleId))
        )
        .left_join(UserRole::Table,
            Expr::col((Role::Table, Role::RoleId))
            .equals((UserRole::Table, UserRole::RoleId))
        )
        .to_owned();

    match selector {
        RoleSelector::Id(value) => {
            stmt = stmt.and_where(Expr::col((Role::Table, Role::RoleId)).eq(value)).to_owned();
        },
        RoleSelector::Name(value1, value2) => {
            stmt = stmt
                .and_where(Expr::col((Role::Table, Role::ApiId)).eq(value1))
                .and_where(Expr::col((Role::Table, Role::Name)).eq(value2))
                .to_owned();
        },
        RoleSelector::Api(value) => {
            stmt = stmt.and_where(Expr::col((Role::Table, Role::ApiId)).eq(value)).to_owned();
        },
        RoleSelector::User(value) => {
            stmt = stmt.and_where(Expr::col((UserRole::Table, UserRole::UserId)).eq(value)).to_owned();
        }
    }
    let (sql, values) = stmt
        .order_by((Role::Table, Role::RoleId), Order::Asc)
        .order_by((RoleAccess::Table, RoleAccess::ProcedureId), Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let mut last_id: Option<Uuid> = None;
    let mut last_procedure: Option<Uuid> = None;
    let mut role_schema_vec: Vec<RoleSchema> = Vec::new();

    sqlx::query_with(&sql, values)
        .map(|row: PgRow| {
            // get last role_schema in role_schema_vec or default
            let mut role_schema = role_schema_vec.pop().unwrap_or_default();
            // on every new role_id found update last_id and insert new role_schema to role_schema_vec
            let role_id: Uuid = row.get(0);
            if let Some(value) = last_id {
                if value != role_id {
                    role_schema_vec.push(role_schema.clone());
                    role_schema = RoleSchema::default();
                    last_procedure = None;
                }
            }
            last_id = Some(role_id);
            role_schema.id = role_id;
            role_schema.api_id = row.get(1);
            role_schema.name = row.get(2);
            role_schema.multi = row.get(3);
            role_schema.ip_lock = row.get(4);
            role_schema.access_duration = row.get(5);
            role_schema.refresh_duration = row.get(6);
            role_schema.access_key = row.get(7);
            // on every new procedure_id found add a procedure to role_schema
            let procedure_id = row.try_get(8).ok();
            if last_procedure == None || last_procedure != procedure_id {
                if let Some(id) = procedure_id {
                    role_schema.procedures.push(id);
                }
            }
            last_procedure = procedure_id;
            // update role_schema_vec with updated role_schema
            role_schema_vec.push(role_schema);
        })
        .fetch_all(pool)
        .await?;

    Ok(role_schema_vec)
}

pub(crate) async fn select_role_by_id(pool: &Pool<Postgres>, 
    id: Uuid
) -> Result<RoleSchema, Error> 
{
    select_role(pool, RoleSelector::Id(id)).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_role_by_name(pool: &Pool<Postgres>, 
    api_id: Uuid,
    name: &str
) -> Result<RoleSchema, Error> 
{
    select_role(pool, RoleSelector::Name(api_id, name.to_owned())).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_role_by_api(pool: &Pool<Postgres>, 
    api_id: Uuid
) -> Result<Vec<RoleSchema>, Error> 
{
    select_role(pool, RoleSelector::Api(api_id)).await
}

pub(crate) async fn select_role_by_user(pool: &Pool<Postgres>, 
    user_id: Uuid
) -> Result<Vec<RoleSchema>, Error> 
{
    select_role(pool, RoleSelector::User(user_id)).await
}

pub(crate) async fn insert_role(pool: &Pool<Postgres>, 
    api_id: Uuid,
    name: &str, 
    multi: bool, 
    ip_lock: bool, 
    access_duration: i32,
    refresh_duration: i32,
) -> Result<Uuid, Error> 
{
    let role_id = Uuid::new_v4();

    let (sql, values) = Query::insert()
        .into_table(Role::Table)
        .columns([
            Role::RoleId,
            Role::ApiId,
            Role::Name,
            Role::Multi,
            Role::IpLock,
            Role::AccessDuration,
            Role::RefreshDuration
        ])
        .values([
            role_id.into(),
            api_id.into(),
            name.into(),
            multi.into(),
            ip_lock.into(),
            access_duration.into(),
            refresh_duration.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(role_id)
}

pub(crate) async fn update_role(pool: &Pool<Postgres>, 
    id: Uuid, 
    name: Option<&str>, 
    multi: Option<bool>, 
    ip_lock: Option<bool>, 
    access_duration: Option<i32>,
    refresh_duration: Option<i32>
) -> Result<(), Error> 
{
    let mut stmt = Query::update()
        .table(Role::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(Role::Name, value).to_owned();
    }
    if let Some(value) = multi {
        stmt = stmt.value(Role::Multi, value).to_owned();
    }
    if let Some(value) = ip_lock {
        stmt = stmt.value(Role::IpLock, value).to_owned();
    }
    if let Some(value) = access_duration {
        stmt = stmt.value(Role::AccessDuration, value).to_owned();
    }
    if let Some(value) = refresh_duration {
        stmt = stmt.value(Role::RefreshDuration, value).to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(Role::RoleId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_role(pool: &Pool<Postgres>, 
    id: Uuid
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(Role::Table)
        .and_where(Expr::col(Role::RoleId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn add_role_access(pool: &Pool<Postgres>, 
    id: Uuid,
    procedure_id: Uuid
) -> Result<(), Error> 
{
    let (sql, values) = Query::insert()
        .into_table(RoleAccess::Table)
        .columns([
            RoleAccess::RoleId,
            RoleAccess::ProcedureId
        ])
        .values([
            id.into(),
            procedure_id.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn remove_role_access(pool: &Pool<Postgres>, 
    id: Uuid,
    procedure_id: Uuid
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(RoleAccess::Table)
        .and_where(Expr::col(RoleAccess::RoleId).eq(id))
        .and_where(Expr::col(RoleAccess::ProcedureId).eq(procedure_id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
