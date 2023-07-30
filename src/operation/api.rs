use sqlx::{Pool, Row, Error};
use sqlx::postgres::{Postgres, PgRow};
use sea_query::{PostgresQueryBuilder, Query, Expr, Order, Func};
use sea_query_binder::SqlxBinder;

use crate::schema::api::{Api, ApiProcedure, ApiSchema, ProcedureSchema};
use crate::schema::auth_role::{Role, RoleAccess};
use crate::utility;

enum ApiSelector {
    Id(i32),
    Name(String),
    Category(String)
}

enum ProcedureSelector {
    Id(i32),
    Name(i32, String),
    Api(i32)
}

async fn select_api(pool: &Pool<Postgres>, 
    selector: ApiSelector
) -> Result<Vec<ApiSchema>, Error>
{
    let mut stmt = Query::select()
        .columns([
            (Api::Table, Api::ApiId),
            (Api::Table, Api::Name),
            (Api::Table, Api::Address),
            (Api::Table, Api::Category),
            (Api::Table, Api::Description),
            (Api::Table, Api::Password),
            (Api::Table, Api::PublicKey),
            (Api::Table, Api::PrivateKey),
            (Api::Table, Api::AccessKey)
        ])
        .columns([
            (ApiProcedure::Table, ApiProcedure::ProcedureId),
            (ApiProcedure::Table, ApiProcedure::Name),
            (ApiProcedure::Table, ApiProcedure::Description)
        ])
        .columns([
            (Role::Table, Role::Name)
        ])
        .from(Api::Table)
        .left_join(ApiProcedure::Table, 
            Expr::col((Api::Table, Api::ApiId))
            .equals((ApiProcedure::Table, ApiProcedure::ApiId))
        )
        .left_join(RoleAccess::Table, 
            Expr::col((ApiProcedure::Table, ApiProcedure::ProcedureId))
            .equals((RoleAccess::Table, RoleAccess::ProcedureId))
        )
        .left_join(Role::Table, 
            Expr::col((RoleAccess::Table, RoleAccess::RoleId))
            .equals((Role::Table, Role::RoleId))
        )
        .to_owned();

    match selector {
        ApiSelector::Id(id) => {
            stmt = stmt.and_where(Expr::col((Api::Table, Api::ApiId)).eq(id)).to_owned();
        },
        ApiSelector::Name(name) => {
            stmt = stmt.and_where(Expr::col((Api::Table, Api::Name)).eq(name)).to_owned();
        },
        ApiSelector::Category(category) => {
            stmt = stmt.and_where(Expr::col((Api::Table, Api::Category)).eq(category.to_string())).to_owned();
        }
    }
    let (sql, values) = stmt
        .order_by((Api::Table, Api::ApiId), Order::Asc)
        .order_by((ApiProcedure::Table, ApiProcedure::ProcedureId), Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let mut last_id: Option<i32> = None;
    let mut last_procedure: Option<i32> = None;
    let mut role_vec: Vec<String> = Vec::new();
    let mut api_schema_vec: Vec<ApiSchema> = Vec::new();

    sqlx::query_with(&sql, values)
        .map(|row: PgRow| {
            // get last api_schema in api_schema_vec or default
            let mut api_schema = api_schema_vec.pop().unwrap_or_default();
            // on every new api_id found update last_id and insert new api_schema to api_schema_vec
            let api_id: i32 = row.get(0);
            if let Some(value) = last_id {
                if value != api_id {
                    api_schema_vec.push(api_schema.clone());
                    api_schema = ApiSchema::default();
                    last_procedure = None;
                    role_vec = Vec::new();
                }
            }
            last_id = Some(api_id);
            api_schema.id = api_id;
            api_schema.name = row.get(1);
            api_schema.address = row.get(2);
            api_schema.category = row.get(3);
            api_schema.description = row.get(4);
            api_schema.password = row.get(5);
            api_schema.public_key = row.get(6);
            api_schema.private_key = row.get(7);
            api_schema.access_key = row.get(8);
            // on every new procedure_id found add a procedure to api_schema
            let procedure_id = row.try_get(9).ok();
            let procedure_name: String = row.try_get(10).unwrap_or_default();
            if last_procedure == None || last_procedure != procedure_id {
                if let Some(id) = procedure_id {
                    api_schema.procedures.push(ProcedureSchema {
                        id,
                        api_id,
                        name: procedure_name.clone(),
                        description: row.get(11),
                        roles: Vec::new()
                    });
                }
            }
            last_procedure = procedure_id;
            // add role to api_schema procedures
            let role_name: Result<String, _> = row.try_get(12);
            if let Ok(name) = role_name {
                let mut procedure_schema = api_schema.procedures.pop().unwrap_or_default();
                procedure_schema.roles.push(name.clone());
                api_schema.procedures.push(procedure_schema);
                role_vec.push(name);
            }
            // update api_schema_vec with updated api_schema
            api_schema_vec.push(api_schema);
        })
        .fetch_all(pool)
        .await?;

    Ok(api_schema_vec)
}

pub(crate) async fn select_api_by_id(pool: &Pool<Postgres>, 
    id: i32
) -> Result<ApiSchema, Error> 
{
    select_api(pool, ApiSelector::Id(id)).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_api_by_name(pool: &Pool<Postgres>, 
    name: &str
) -> Result<ApiSchema, Error> 
{
    select_api(pool, ApiSelector::Name(name.to_owned())).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_api_by_category(pool: &Pool<Postgres>, 
    category: &str
) -> Result<Vec<ApiSchema>, Error> 
{
    select_api(pool, ApiSelector::Category(category.to_owned())).await
}

pub(crate) async fn insert_api(pool: &Pool<Postgres>, 
    name: &str, 
    address: &str, 
    category: &str, 
    description: &str,
    password: &str
) -> Result<i32, Error> 
{
    let password_hash = utility::hash_password(&password).or(Err(Error::WorkerCrashed))?;

    let (priv_key, pub_key) = utility::generate_keys().or(Err(Error::WorkerCrashed))?;
    let priv_der = utility::export_private_key(priv_key).or(Err(Error::WorkerCrashed))?;
    let pub_der = utility::export_public_key(pub_key).or(Err(Error::WorkerCrashed))?;

    let access_key = utility::generate_random_bytes(32);

    let (sql, values) = Query::insert()
        .into_table(Api::Table)
        .columns([
            Api::Name,
            Api::Address,
            Api::Category,
            Api::Description,
            Api::Password,
            Api::PublicKey,
            Api::PrivateKey,
            Api::AccessKey
        ])
        .values([
            name.into(),
            address.into(),
            category.into(),
            description.into(),
            password_hash.into(),
            pub_der.into(),
            priv_der.into(),
            access_key.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(Api::ApiId)))
        .from(Api::Table)
        .to_string(PostgresQueryBuilder);
    let id: i32 = sqlx::query(&sql)
        .map(|row: PgRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_api(pool: &Pool<Postgres>, 
    id: i32, 
    name: Option<&str>, 
    address: Option<&str>, 
    category: Option<&str>, 
    description: Option<&str>,
    password: Option<&str>,
    keys: Option<()>
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
    if let Some(value) = category {
        stmt = stmt.value(Api::Category, value).to_owned();
    }
    if let Some(value) = password {
        let password_hash = utility::hash_password(value).or(Err(Error::WorkerCrashed))?;
        stmt = stmt.value(Api::Password, password_hash).to_owned();
    }
    if let Some(value) = description {
        stmt = stmt.value(Api::Description, value).to_owned();
    }
    if let Some(_) = keys {
        let (priv_key, pub_key) = utility::generate_keys().or(Err(Error::WorkerCrashed))?;
        let priv_der = utility::export_private_key(priv_key).or(Err(Error::WorkerCrashed))?;
        let pub_der = utility::export_public_key(pub_key).or(Err(Error::WorkerCrashed))?;
        let access_key = utility::generate_random_bytes(32);
        stmt = stmt
            .value(Api::PublicKey, pub_der)
            .value(Api::PrivateKey, priv_der)
            .value(Api::AccessKey, access_key)
            .to_owned();
    }

    let (sql, values) = stmt
        .and_where(Expr::col(Api::ApiId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_api(pool: &Pool<Postgres>, 
    id: i32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(Api::Table)
        .and_where(Expr::col(Api::ApiId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

async fn select_procedure(pool: &Pool<Postgres>, 
    selector: ProcedureSelector
) -> Result<Vec<ProcedureSchema>, Error> 
{
    let mut stmt = Query::select()
        .columns([
            (ApiProcedure::Table, ApiProcedure::ProcedureId),
            (ApiProcedure::Table, ApiProcedure::ApiId),
            (ApiProcedure::Table, ApiProcedure::Name),
            (ApiProcedure::Table, ApiProcedure::Description)
        ])
        .columns([
            (Role::Table, Role::Name)
        ])
        .from(ApiProcedure::Table)
        .left_join(RoleAccess::Table, 
            Expr::col((ApiProcedure::Table, ApiProcedure::ProcedureId))
            .equals((RoleAccess::Table, RoleAccess::ProcedureId))
        )
        .left_join(Role::Table, 
            Expr::col((RoleAccess::Table, RoleAccess::RoleId))
            .equals((Role::Table, Role::RoleId))
        )
        .to_owned();

    match selector {
        ProcedureSelector::Id(id) => {
            stmt = stmt.and_where(Expr::col((ApiProcedure::Table, ApiProcedure::ProcedureId)).eq(id)).to_owned();
        },
        ProcedureSelector::Name(api_id, name) => {
            stmt = stmt
                .and_where(Expr::col((ApiProcedure::Table, ApiProcedure::ApiId)).eq(api_id))
                .and_where(Expr::col((ApiProcedure::Table, ApiProcedure::Name)).eq(name))
                .to_owned();
        }
        ProcedureSelector::Api(api_id) => {
            stmt = stmt.and_where(Expr::col((ApiProcedure::Table, ApiProcedure::ApiId)).eq(api_id)).to_owned();
        }
    }
    let (sql, values) = stmt
        .order_by(ApiProcedure::ProcedureId, Order::Asc)
        .build_sqlx(PostgresQueryBuilder);

    let mut last_id: Option<i32> = None;
    let mut proc_schema_vec: Vec<ProcedureSchema> = Vec::new();

    sqlx::query_with(&sql, values)
        .map(|row: PgRow| {
            // get last proc_schema in proc_schema_vec or default
            let mut proc_schema = proc_schema_vec.pop().unwrap_or_default();
            // on every new proc_id found update last_id and insert new proc_schema to proc_schema_vec
            let proc_id: i32 = row.get(0);
            if let Some(value) = last_id {
                if value != proc_id {
                    proc_schema_vec.push(proc_schema.clone());
                    proc_schema = ProcedureSchema::default();
                }
            }
            last_id = Some(proc_id);
            proc_schema.id = proc_id;
            proc_schema.api_id = row.get(1);
            proc_schema.name = row.get(2);
            proc_schema.description = row.get(3);
            // add role to proc_schema roles
            let role_name: Result<String, _> = row.try_get(4);
            if let Ok(name) = role_name {
                proc_schema.roles.push(name);
            }
            // update proc_schema_vec with updated proc_schema
            proc_schema_vec.push(proc_schema);
        })
        .fetch_all(pool)
        .await?;

    Ok(proc_schema_vec)
}

pub(crate) async fn select_procedure_by_id(pool: &Pool<Postgres>, 
    id: i32
) -> Result<ProcedureSchema, Error> 
{
    select_procedure(pool, ProcedureSelector::Id(id)).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_procedure_by_name(pool: &Pool<Postgres>, 
    api_id: i32,
    name: &str
) -> Result<ProcedureSchema, Error> 
{
    select_procedure(pool, ProcedureSelector::Name(api_id, name.to_owned())).await?.into_iter().next()
        .ok_or(Error::RowNotFound)
}

pub(crate) async fn select_procedure_by_api(pool: &Pool<Postgres>, 
    api_id: i32
) -> Result<Vec<ProcedureSchema>, Error> 
{
    select_procedure(pool, ProcedureSelector::Api(api_id)).await
}

pub(crate) async fn insert_procedure(pool: &Pool<Postgres>, 
    api_id: i32,
    name: &str,
    description: &str
) -> Result<i32, Error> 
{
    let (sql, values) = Query::insert()
        .into_table(ApiProcedure::Table)
        .columns([
            ApiProcedure::ApiId,
            ApiProcedure::Name,
            ApiProcedure::Description
        ])
        .values([
            api_id.into(),
            name.into(),
            description.into()
        ])
        .unwrap_or(&mut sea_query::InsertStatement::default())
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    let sql = Query::select()
        .expr(Func::max(Expr::col(ApiProcedure::ProcedureId)))
        .from(ApiProcedure::Table)
        .to_string(PostgresQueryBuilder);
    let id: i32 = sqlx::query(&sql)
        .map(|row: PgRow| row.get(0))
        .fetch_one(pool)
        .await?;

    Ok(id)
}

pub(crate) async fn update_procedure(pool: &Pool<Postgres>, 
    id: i32,
    name: Option<&str>,
    description: Option<&str>
) -> Result<(), Error> 
{
    let mut stmt = Query::update()
        .table(ApiProcedure::Table)
        .to_owned();

    if let Some(value) = name {
        stmt = stmt.value(ApiProcedure::Name, value).to_owned()
    }
    if let Some(value) = description {
        stmt = stmt.value(ApiProcedure::Description, value).to_owned()
    }

    let (sql, values) = stmt
        .and_where(Expr::col(ApiProcedure::ProcedureId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) async fn delete_procedure(pool: &Pool<Postgres>, 
    id: i32
) -> Result<(), Error> 
{
    let (sql, values) = Query::delete()
        .from_table(ApiProcedure::Table)
        .and_where(Expr::col(ApiProcedure::ProcedureId).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(pool)
        .await?;

    Ok(())
}
