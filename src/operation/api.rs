use sqlx::{Pool, Row, Error};
use sqlx::mysql::{MySql, MySqlRow};
use sea_query::{MysqlQueryBuilder, Query, Expr, Order, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::api::{Api, ApiProcedure, ApiKind, ApiSchema, ProcedureSchema};

enum ApiSelector {
    Id(u32),
    Name(String),
    All
}

enum ProcedureSelector {
    Id(u32),
    Name(u32, String, String)
}

async fn select_api(pool: &Pool<MySql>, 
    kind: ApiKind, 
    selector: ApiSelector
) -> Result<Vec<ApiSchema>, Error>
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
        },
        ApiSelector::All => {}
    }
    let (sql, values) = stmt
        .order_by(Api::ApiId, Order::Asc)
        .order_by(ApiProcedure::ProcedureId, Order::Asc)
        .build_sqlx(MysqlQueryBuilder);

    let mut last_id: Option<u32> = None;
    let mut api_schema_vec: Vec<ApiSchema> = Vec::new();

    sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            // get last api_schema in api_schema_vec or default
            let mut api_schema = api_schema_vec.pop().unwrap_or_default();
            // on every new api_id found update last_id and insert new api_schema to api_schema_vec
            let api_id: u32 = row.get(0);
            if let Some(value) = last_id {
                if value != api_id {
                    api_schema_vec.push(api_schema.clone());
                    api_schema = ApiSchema::default();
                }
            }
            last_id = Some(api_id);
            api_schema.id = api_id;
            api_schema.name = row.get(1);
            api_schema.address = row.get(2);
            api_schema.description = row.get(3);
            // on every new procedure_id found add a procedure to api_schema
            let procedure_id = row.try_get(4);
            if let Ok(id) = procedure_id {
                api_schema.procedures.push(ProcedureSchema {
                    id,
                    api_id,
                    service: row.get(5),
                    procedure: row.get(6),
                    description: row.get(7)
                });
            }
            // update api_schema_vec with updated api_schema
            api_schema_vec.push(api_schema);
        })
        .fetch_all(pool)
        .await?;

    Ok(api_schema_vec)
}

pub(crate) async fn select_api_by_id(pool: &Pool<MySql>, 
    kind: ApiKind, 
    id: u32
) -> Result<ApiSchema, Error> 
{
    select_api(pool, kind, ApiSelector::Id(id)).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_api_by_name(pool: &Pool<MySql>, 
    kind: ApiKind, 
    name: &str
) -> Result<ApiSchema, Error> 
{
    select_api(pool, kind, ApiSelector::Name(name.to_owned())).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_multiple_api(pool: &Pool<MySql>, 
    kind: ApiKind
) -> Result<Vec<ApiSchema>, Error> 
{
    select_api(pool, kind, ApiSelector::All).await
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

async fn select_procedure(pool: &Pool<MySql>, 
    selector: ProcedureSelector
) -> Result<ProcedureSchema, Error> 
{
    let mut stmt = Query::select()
        .columns([
            ApiProcedure::ApiId,
            ApiProcedure::ProcedureId,
            ApiProcedure::Service,
            ApiProcedure::Procedure,
            ApiProcedure::Description
        ])
        .from(ApiProcedure::Table)
        .to_owned();

    match selector {
        ProcedureSelector::Id(id) => {
            stmt = stmt.and_where(Expr::col(ApiProcedure::ProcedureId).eq(id)).to_owned();
        },
        ProcedureSelector::Name(api_id, service, procedure) => {
            stmt = stmt
                .and_where(Expr::col(ApiProcedure::ApiId).eq(api_id))
                .and_where(Expr::col(ApiProcedure::Service).eq(service))
                .and_where(Expr::col(ApiProcedure::Procedure).eq(procedure))
                .to_owned();
        }
    }
    let (sql, values) = stmt.build_sqlx(MysqlQueryBuilder);

    let row = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            ProcedureSchema {
                api_id: row.get(0),
                id: row.get(1),
                service: row.get(2),
                procedure: row.get(3),
                description: row.get(4)
            }
        })
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub(crate) async fn select_procedure_by_id(pool: &Pool<MySql>, 
    id: u32
) -> Result<ProcedureSchema, Error> 
{
    select_procedure(pool, ProcedureSelector::Id(id)).await
}

pub(crate) async fn select_procedure_by_name(pool: &Pool<MySql>, 
    api_id: u32,
    service: &str,
    name: &str
) -> Result<ProcedureSchema, Error> 
{
    select_procedure(pool, ProcedureSelector::Name(api_id, service.to_owned(), name.to_owned())).await
}

pub(crate) async fn select_multiple_procedure(pool: &Pool<MySql>, 
    api_id: u32
) -> Result<Vec<ProcedureSchema>, Error> 
{
    let (sql, values) = Query::select()
        .columns([
            ApiProcedure::ApiId,
            ApiProcedure::ProcedureId,
            ApiProcedure::Service,
            ApiProcedure::Procedure,
            ApiProcedure::Description
        ])
        .from(ApiProcedure::Table)
        .and_where(Expr::col(ApiProcedure::ApiId).eq(api_id.to_string()))
        .build_sqlx(MysqlQueryBuilder);

    let rows = sqlx::query_with(&sql, values)
        .map(|row: MySqlRow| {
            ProcedureSchema {
                api_id: row.get(0),
                id: row.get(1),
                service: row.get(2),
                procedure: row.get(3),
                description: row.get(4)
            }
        })
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub(crate) async fn insert_procedure(pool: &Pool<MySql>, 
    api_id: u32,
    service: &str,
    name: &str,
    description: Option<&str>
) -> Result<u32, Error> 
{
    let (sql, values) = Query::insert()
        .into_table(ApiProcedure::Table)
        .columns([
            ApiProcedure::ApiId,
            ApiProcedure::Service,
            ApiProcedure::Procedure,
            ApiProcedure::Description
        ])
        .values([
            api_id.into(),
            service.into(),
            name.into(),
            description.unwrap_or_default().into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(ApiProcedure::ProcedureId)))
        .from(ApiProcedure::Table)
        .to_string(MysqlQueryBuilder);
    let id: u32 = sqlx::query(&sql)
        .map(|row: MySqlRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_procedure(pool: &Pool<MySql>, 
    id: u32,
    service: Option<&str>,
    name: Option<&str>,
    description: Option<&str>
) -> Result<(), Error> 
{
    let mut stmt = Query::update()
        .table(ApiProcedure::Table)
        .to_owned();

    if let Some(value) = service {
        stmt = stmt.value(ApiProcedure::Service, value).to_owned()
    }
    if let Some(value) = name {
        stmt = stmt.value(ApiProcedure::Procedure, value).to_owned()
    }
    if let Some(value) = description {
        stmt = stmt.value(ApiProcedure::Description, value).to_owned()
    }

    let (sql, values) = stmt
        .and_where(Expr::col(ApiProcedure::ProcedureId).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_procedure(pool: &Pool<MySql>, 
    id: u32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(ApiProcedure::Table)
        .and_where(Expr::col(ApiProcedure::ProcedureId).eq(id))
        .build_sqlx(MysqlQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
