use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sea_query::{MysqlQueryBuilder, Query, Expr, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::api::{Api, ApiProcedure, ApiKind, ApiFields, ApiJoin, ProcedureFields};

enum ApiSelector {
    Id(u32),
    Name(String)
}

enum ProcedureSelector {
    Id(u32),
    Name((u32, String, String))
}

async fn select_api(pool: &Pool<MySql>, 
    kind: ApiKind, 
    selector: ApiSelector
) -> Result<ApiFields, Error>
{
    let mut stmt = Query::select()
        .columns([
            (Api::Table, Api::ApiId),
            (Api::Table, Api::Name),
            (Api::Table, Api::Address),
            (Api::Table, Api::Description)
        ])
        .columns([
            (ApiProcedure::Table, ApiProcedure::ProcedureId),
            (ApiProcedure::Table, ApiProcedure::Service),
            (ApiProcedure::Table, ApiProcedure::Procedure),
            (ApiProcedure::Table, ApiProcedure::Description)
        ])
        .from(Api::Table)
        .left_join(ApiProcedure::Table, 
            Expr::col((Api::Table, Api::ApiId))
            .equals((ApiProcedure::Table, ApiProcedure::ApiId))
        )
        .and_where(Expr::col((Api::Table, Api::Kind)).eq(kind.to_string()))
        .to_owned();

    match selector {
        ApiSelector::Id(id) => {
            stmt = stmt.and_where(Expr::col((Api::Table, Api::ApiId)).eq(id)).to_owned();
        },
        ApiSelector::Name(name) => {
            stmt = stmt.and_where(Expr::col((Api::Table, Api::Name)).eq(name)).to_owned();
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            ApiJoin {
                id: row.get(0),
                name: row.get(1),
                address: row.get(2),
                description: row.get(3),
                procedure_id: row.try_get(4).ok(),
                service: row.try_get(5).ok(),
                procedure: row.try_get(6).ok(),
                procedure_description: row.try_get(7).ok(),
            }
        })
        .fetch_all(pool)
        .await?;

    let procedures: Vec<ProcedureFields> = rows.iter()
        .filter(|row| {
            row.procedure_id != None 
            && row.service != None 
            && row.procedure != None 
            && row.procedure_description != None
        })
        .map(|row| {
            ProcedureFields {
                id: row.procedure_id.unwrap_or_default(),
                api_id: None,
                service: row.service.clone().unwrap_or_default(),
                procedure: row.procedure.clone().unwrap_or_default(),
                description: row.procedure_description.clone()
            }
        })
        .collect();
    let first_row = rows.iter().next();

    match first_row {
        Some(value) => Ok(ApiFields {
                id: value.id,
                name: value.name.clone(),
                address: value.address.clone(),
                description: Some(value.description.clone()),
                procedures
            }),
        None => Err(Error::RowNotFound)
    }
}

pub(crate) async fn select_api_by_id(pool: &Pool<MySql>, 
    kind: ApiKind, 
    id: u32
) -> Result<ApiFields, Error> 
{
    select_api(pool, kind, ApiSelector::Id(id)).await
}

pub(crate) async fn select_api_by_name(pool: &Pool<MySql>, 
    kind: ApiKind, 
    name: &str
) -> Result<ApiFields, Error> 
{
    select_api(pool, kind, ApiSelector::Name(name.to_owned())).await
}

pub(crate) async fn select_multiple_api(pool: &Pool<MySql>, 
    kind: ApiKind
) -> Result<Vec<ApiFields>, Error> 
{
    let (sql, values) = Query::select()
        .columns([
            Api::ApiId,
            Api::Name,
            Api::Address,
            Api::Description
        ])
        .from(Api::Table)
        .and_where(Expr::col(Api::Kind).eq(kind.to_string()))
        .build_sqlx(MysqlQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            ApiFields {
                id: row.get(0),
                name: row.get(1),
                address: row.get(2),
                description: row.get(3),
                procedures: vec![]
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub(crate) async fn insert_api(pool: &Pool<MySql>, 
    kind: ApiKind, 
    name: &str, 
    address: &str, 
    description: Option<&str>
) -> Result<u32, Error> 
{
    let (sql, values) = Query::insert()
        .into_table(Api::Table)
        .columns([
            Api::Name,
            Api::Kind,
            Api::Address,
            Api::Description
        ])
        .values([
            name.into(),
            kind.to_string().into(),
            address.into(),
            description.unwrap_or_default().into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(Api::ApiId)))
        .from(Api::Table)
        .to_string(MysqlQueryBuilder);
    let id: u32 = sqlx::query(&sql)
        .map(|row: MySqlRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_api(pool: &Pool<MySql>, 
    kind: ApiKind, 
    id: u32, 
    name: Option<&str>, 
    address: Option<&str>, 
    description: Option<&str>
) -> Result<(), Error> 
{
    let mut stmt = Query::update()
        .table(Api::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(Api::Name, value).to_owned();
    }
    if let Some(value) = address {
        stmt = stmt.value(Api::Address, value).to_owned();
    }
    if let Some(value) = description {
        stmt = stmt.value(Api::Description, value).to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(Api::ApiId).eq(id))
        .and_where(Expr::col(Api::Kind).eq(kind.to_string()))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_api(pool: &Pool<MySql>, 
    kind: ApiKind, 
    id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(Api::Table)
        .and_where(Expr::col(Api::ApiId).eq(id))
        .and_where(Expr::col(Api::Kind).eq(kind.to_string()))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
