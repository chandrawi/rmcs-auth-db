use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sea_query::{MysqlQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::auth_role::{AuthRole, AuthAccess, RoleSchema, RoleJoin};

enum RoleSelector {
    Id(u32),
    Name(String)
}

async fn select_role(pool: &Pool<MySql>, 
    selector: RoleSelector
) -> Result<RoleSchema, Error>
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
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            RoleJoin {
                id: row.get(0),
                name: row.get(1),
                secured: row.get(2),
                multi: row.get(3),
                token_expire: row.get(4),
                token_limit: row.get(5),
                procedure_id: row.try_get(6).ok(),
            }
        })
        .fetch_all(pool)
        .await?;

    let access: Vec<u32> = rows.iter()
        .filter(|row| {
            row.procedure_id != None 
        })
        .map(|row| {
            row.procedure_id.unwrap_or_default()
        })
        .collect();
    let first_row = rows.iter().next();

    match first_row {
        Some(value) => Ok(RoleSchema {
                id: value.id,
                name: value.name.clone(),
                secured: value.secured,
                multi: value.multi,
                token_expire: value.token_expire,
                token_limit: value.token_limit,
                access,
            }),
        None => Err(Error::RowNotFound)
    }
}

pub(crate) async fn select_role_by_id(pool: &Pool<MySql>, 
    id: u32
) -> Result<RoleSchema, Error> 
{
    select_role(pool, RoleSelector::Id(id)).await
}

pub(crate) async fn select_role_by_name(pool: &Pool<MySql>, 
    name: &str
) -> Result<RoleSchema, Error> 
{
    select_role(pool, RoleSelector::Name(name.to_owned())).await
}

pub(crate) async fn select_role_all(pool: &Pool<MySql>, 
) -> Result<Vec<RoleSchema>, Error> 
{
    let (sql, values) = Query::select()
        .columns([
            AuthRole::RoleId,
            AuthRole::Role,
            AuthRole::Secured,
            AuthRole::Multi,
            AuthRole::TokenExpire,
            AuthRole::TokenLimit
        ])
        .from(AuthRole::Table)
        .build_sqlx(MysqlQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            RoleSchema {
                id: row.get(0),
                name: row.get(1),
                secured: row.get(2),
                multi: row.get(3),
                token_expire: row.get(4),
                token_limit: row.get(5),
                access: vec![],
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(rows)
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
