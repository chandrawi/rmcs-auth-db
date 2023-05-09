use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sea_query::{MysqlQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::auth_role::{AuthRole, AuthAccess, RoleSchema};

enum RoleSelector {
    Id(u32),
    Name(String),
    All
}

async fn select_role(pool: &Pool<MySql>, 
    selector: RoleSelector
) -> Result<Vec<RoleSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            (AuthRole::Table, AuthRole::RoleId),
            (AuthRole::Table, AuthRole::Role),
            (AuthRole::Table, AuthRole::Secured),
            (AuthRole::Table, AuthRole::Multi),
            (AuthRole::Table, AuthRole::TokenExpire),
            (AuthRole::Table, AuthRole::TokenLimit)
        ])
        .columns([
            (AuthAccess::Table, AuthAccess::ProcedureId)
        ])
        .from(AuthRole::Table)
        .left_join(AuthAccess::Table, 
            Expr::col((AuthRole::Table, AuthRole::RoleId))
            .equals((AuthAccess::Table, AuthAccess::RoleId))
        )
        .to_owned();

    match selector {
        RoleSelector::Id(value) => {
            stmt = stmt.and_where(Expr::col((AuthRole::Table, AuthRole::RoleId)).eq(value)).to_owned();
        },
        RoleSelector::Name(value) => {
            stmt = stmt.and_where(Expr::col((AuthRole::Table, AuthRole::Role)).eq(value)).to_owned();
        },
        RoleSelector::All => {}
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let mut last_id: Option<u32> = None;
    let mut role_schema_vec: Vec<RoleSchema> = Vec::new();

    sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            // get last role_schema in role_schema_vec or default
            let mut role_schema = role_schema_vec.pop().unwrap_or_default();
            // on every new role_id found update last_id and insert new role_schema to role_schema_vec
            let role_id: u32 = row.get(0);
            if let Some(value) = last_id {
                if value != role_id {
                    role_schema_vec.push(role_schema.clone());
                    role_schema = RoleSchema::default();
                }
            }
            last_id = Some(role_id);
            role_schema.id = role_id;
            role_schema.name = row.get(1);
            role_schema.secured = row.get(2);
            role_schema.multi = row.get(3);
            role_schema.token_expire = row.get(4);
            role_schema.token_limit = row.get(5);
            // on every new procedure_id found add a procedure to role_schema
            let procedure_id = row.try_get(6);
            if let Ok(id) = procedure_id {
                role_schema.access.push(id);
            }
            // update role_schema_vec with updated role_schema
            role_schema_vec.push(role_schema);
        })
        .fetch_all(pool)
        .await?;

    Ok(role_schema_vec)
}

pub(crate) async fn select_role_by_id(pool: &Pool<MySql>, 
    id: u32
) -> Result<RoleSchema, Error> 
{
    select_role(pool, RoleSelector::Id(id)).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_role_by_name(pool: &Pool<MySql>, 
    name: &str
) -> Result<RoleSchema, Error> 
{
    select_role(pool, RoleSelector::Name(name.to_owned())).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_multiple_role(pool: &Pool<MySql>, 
) -> Result<Vec<RoleSchema>, Error> 
{
    select_role(pool, RoleSelector::All).await
}

pub(crate) async fn insert_role(pool: &Pool<MySql>, 
    name: &str, 
    secured: bool, 
    multi: bool, 
    token_expire: Option<u32>,
    token_limit: Option<u32>
) -> Result<u32, Error> 
{
    let (sql, values) = Query::insert()
        .into_table(AuthRole::Table)
        .columns([
            AuthRole::Role,
            AuthRole::Secured,
            AuthRole::Multi,
            AuthRole::TokenExpire,
            AuthRole::TokenLimit
        ])
        .values([
            name.into(),
            secured.into(),
            multi.into(),
            token_expire.unwrap_or_default().into(),
            token_limit.unwrap_or_default().into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(AuthRole::RoleId)))
        .from(AuthRole::Table)
        .to_string(MysqlQueryBuilder);
    let id: u32 = sqlx::query(&sql)
        .map(|row: MySqlRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_role(pool: &Pool<MySql>, 
    id: u32, 
    name: Option<&str>, 
    secured: Option<bool>, 
    multi: Option<bool>, 
    token_expire: Option<u32>,
    token_limit: Option<u32>
) -> Result<(), Error> 
{
    let mut stmt = Query::update()
        .table(AuthRole::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(AuthRole::Role, value).to_owned();
    }
    if let Some(value) = secured {
        stmt = stmt.value(AuthRole::Secured, value).to_owned();
    }
    if let Some(value) = multi {
        stmt = stmt.value(AuthRole::Multi, value).to_owned();
    }
    if let Some(value) = token_expire {
        stmt = stmt.value(AuthRole::TokenExpire, value).to_owned();
    }
    if let Some(value) = token_limit {
        stmt = stmt.value(AuthRole::TokenLimit, value).to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(AuthRole::RoleId).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_role(pool: &Pool<MySql>, 
    id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(AuthRole::Table)
        .and_where(Expr::col(AuthRole::RoleId).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn add_role_access(pool: &Pool<MySql>, 
    id: u32,
    procedure_id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::insert()
        .into_table(AuthAccess::Table)
        .columns([
            AuthAccess::RoleId,
            AuthAccess::ProcedureId
        ])
        .values([
            id.into(),
            procedure_id.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn remove_role_access(pool: &Pool<MySql>, 
    id: u32,
    procedure_id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(AuthAccess::Table)
        .and_where(Expr::col(AuthAccess::RoleId).eq(id))
        .and_where(Expr::col(AuthAccess::ProcedureId).eq(procedure_id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
