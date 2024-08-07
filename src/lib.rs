pub mod schema;
pub(crate) mod operation;
pub mod utility;

use sqlx::{Pool, Error};
use sqlx::postgres::{Postgres, PgPoolOptions};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

use operation::api;
use operation::role;
use operation::user;
use operation::token;
pub use schema::api::{ApiSchema, ProcedureSchema};
pub use schema::auth_role::RoleSchema;
pub use schema::auth_user::UserSchema;
pub use schema::auth_token::TokenSchema;
use token::TokenSelector;

#[derive(Debug, Clone)]
pub struct Auth {
    pub pool: Pool<Postgres>,
    options: AuthOptions
}

#[derive(Debug, Clone)]
pub struct AuthOptions {
    limit: u32,
    with_description: bool,
    order: Vec<OrderOption>
}

#[derive(Debug, Clone)]
pub enum OrderOption {
    IdAsc,
    IdDesc,
    NameAsc,
    NameDesc
}

impl Default for AuthOptions {
    fn default() -> Self {
        AuthOptions { 
            limit: 10000, 
            with_description: false, 
            order: vec![] 
        }
    }
}

impl Auth {

    pub async fn new(host: &str, username: &str, password: &str, database: &str) -> Auth {
        let url = format!("mysql://{}:{}@{}/{}", username, password, host, database);
        Auth::new_with_url(&url).await
    }

    pub async fn new_with_url(url: &str) -> Auth {
        let pool = PgPoolOptions::new()
            .max_connections(100)
            .connect(url)
            .await
            .expect(&format!("Error connecting to {}", url));
        Auth {
            pool,
            options: AuthOptions::default()
        }
    }

    pub fn new_with_pool(pool: Pool<Postgres>) -> Auth {
        Auth {
            pool,
            options: AuthOptions::default()
        }
    }

    pub fn set_limit(mut self, limit: u32) {
        self.options.limit = limit;
    }

    pub fn set_with_description(mut self, with_description: bool) {
        self.options.with_description = with_description;
    }

    pub fn set_order(mut self, order: Vec<OrderOption>) {
        self.options.order = order;
    }

    pub async fn read_api(&self, id: Uuid)
        -> Result<ApiSchema, Error>
    {
        api::select_api(&self.pool, Some(id), None, None, None, None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn read_api_by_name(&self, name: &str)
        -> Result<ApiSchema, Error>
    {
        api::select_api(&self.pool, None, None, Some(name), None, None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn list_api_by_ids(&self, ids: &[Uuid])
        -> Result<Vec<ApiSchema>, Error>
    {
        api::select_api(&self.pool, None, Some(ids), None, None, None)
        .await
    }

    pub async fn list_api_by_name(&self, name: &str)
        -> Result<Vec<ApiSchema>, Error>
    {
        api::select_api(&self.pool, None, None, None, Some(name), None)
        .await
    }

    pub async fn list_api_by_category(&self, category: &str)
        -> Result<Vec<ApiSchema>, Error>
    {
        api::select_api(&self.pool, None, None, None, None, Some(category))
        .await
    }

    pub async fn list_api_option(&self, name: Option<&str>, category: Option<&str>)
        -> Result<Vec<ApiSchema>, Error>
    {
        api::select_api(&self.pool, None, None, None, name, category)
        .await
    }

    pub async fn create_api(&self, id: Uuid, name: &str, address: &str, category: &str, description: &str, password: &str, access_key: &[u8])
        -> Result<Uuid, Error>
    {
        api::insert_api(&self.pool, id, name, address, category, description, password, access_key)
        .await
    }

    pub async fn update_api(&self, id: Uuid, name: Option<&str>, address: Option<&str>, category: Option<&str>, description: Option<&str>, password: Option<&str>, access_key: Option<&[u8]>)
        -> Result<(), Error>
    {
        api::update_api(&self.pool, id, name, address, category, description, password, access_key)
        .await
    }

    pub async fn delete_api(&self, id: Uuid)
        -> Result<(), Error>
    {
        api::delete_api(&self.pool, id)
        .await
    }

    pub async fn read_procedure(&self, id: Uuid)
        -> Result<ProcedureSchema, Error>
    {
        api::select_procedure(&self.pool, Some(id), None, None, None, None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn read_procedure_by_name(&self, api_id: Uuid, name: &str)
        -> Result<ProcedureSchema, Error>
    {
        api::select_procedure(&self.pool, None, None, Some(api_id), Some(name), None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn list_procedure_by_ids(&self, ids: &[Uuid])
        -> Result<Vec<ProcedureSchema>, Error>
    {
        api::select_procedure(&self.pool, None, Some(ids), None, None, None)
        .await
    }

    pub async fn list_procedure_by_api(&self, api_id: Uuid)
        -> Result<Vec<ProcedureSchema>, Error>
    {
        api::select_procedure(&self.pool, None, None, Some(api_id), None, None)
        .await
    }

    pub async fn list_procedure_by_name(&self, name: &str)
        -> Result<Vec<ProcedureSchema>, Error>
    {
        api::select_procedure(&self.pool, None, None, None, None, Some(name))
        .await
    }

    pub async fn list_procedure_option(&self, api_id: Option<Uuid>, name: Option<&str>)
        -> Result<Vec<ProcedureSchema>, Error>
    {
        api::select_procedure(&self.pool, None, None, api_id, None, name)
        .await
    }

    pub async fn create_procedure(&self, id: Uuid, api_id: Uuid, name: &str, description: &str)
        -> Result<Uuid, Error>
    {
        api::insert_procedure(&self.pool, id, api_id, name, description)
        .await
    }

    pub async fn update_procedure(&self, id: Uuid, name: Option<&str>, description: Option<&str>)
        -> Result<(), Error>
    {
        api::update_procedure(&self.pool, id, name, description)
        .await
    }

    pub async fn delete_procedure(&self, id: Uuid)
        -> Result<(), Error>
    {
        api::delete_procedure(&self.pool, id)
        .await
    }

    pub async fn read_role(&self, id: Uuid)
        -> Result<RoleSchema, Error>
    {
        role::select_role(&self.pool, Some(id), None, None, None, None, None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn read_role_by_name(&self, api_id: Uuid, name: &str)
        -> Result<RoleSchema, Error>
    {
        role::select_role(&self.pool, None, None, Some(api_id), None, Some(name), None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn list_role_by_ids(&self, ids: &[Uuid])
        -> Result<Vec<RoleSchema>, Error>
    {
        role::select_role(&self.pool, None, Some(ids), None, None, None, None)
        .await
    }

    pub async fn list_role_by_api(&self, api_id: Uuid)
        -> Result<Vec<RoleSchema>, Error>
    {
        role::select_role(&self.pool, None, None, Some(api_id), None, None, None)
        .await
    }

    pub async fn list_role_by_user(&self, user_id: Uuid)
        -> Result<Vec<RoleSchema>, Error>
    {
        role::select_role(&self.pool, None, None, None, Some(user_id), None, None)
        .await
    }

    pub async fn list_role_by_name(&self, name: &str)
        -> Result<Vec<RoleSchema>, Error>
    {
        role::select_role(&self.pool, None, None, None, None, None, Some(name))
        .await
    }

    pub async fn list_role_option(&self, api_id: Option<Uuid>, user_id: Option<Uuid>, name: Option<&str>)
        -> Result<Vec<RoleSchema>, Error>
    {
        role::select_role(&self.pool, None, None, api_id, user_id, None, name)
        .await
    }

    pub async fn create_role(&self, id: Uuid, api_id: Uuid, name: &str, multi: bool, ip_lock: bool, access_duration: i32, refresh_duration: i32)
        -> Result<Uuid, Error>
    {
        role::insert_role(&self.pool, id, api_id, name, multi, ip_lock, access_duration, refresh_duration)
        .await
    }

    pub async fn update_role(&self, id: Uuid, name: Option<&str>, multi: Option<bool>, ip_lock: Option<bool>, access_duration: Option<i32>, refresh_duration: Option<i32>)
        -> Result<(), Error>
    {
        role::update_role(&self.pool, id, name, multi, ip_lock, access_duration, refresh_duration)
        .await
    }

    pub async fn delete_role(&self, id: Uuid)
        -> Result<(), Error>
    {
        role::delete_role(&self.pool, id)
        .await
    }

    pub async fn add_role_access(&self, id: Uuid, procedure_id: Uuid)
        -> Result<(), Error>
    {
        role::add_role_access(&self.pool, id, procedure_id)
        .await
    }

    pub async fn remove_role_access(&self, id: Uuid, procedure_id: Uuid)
        -> Result<(), Error>
    {
        role::remove_role_access(&self.pool, id, procedure_id)
        .await
    }

    pub async fn read_user(&self, id: Uuid)
        -> Result<UserSchema, Error>
    {
        user::select_user(&self.pool, Some(id), None, None, None, None, None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn read_user_by_name(&self, name: &str)
        -> Result<UserSchema, Error>
    {
        user::select_user(&self.pool, None, None, None, None, Some(name), None).await?
        .into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn list_user_by_ids(&self, ids: &[Uuid])
        -> Result<Vec<UserSchema>, Error>
    {
        user::select_user(&self.pool, None, Some(ids), None, None, None, None)
        .await
    }

    pub async fn list_user_by_api(&self, api_id: Uuid)
        -> Result<Vec<UserSchema>, Error>
    {
        user::select_user(&self.pool, None, None, Some(api_id), None, None, None)
        .await
    }

    pub async fn list_user_by_role(&self, role_id: Uuid)
        -> Result<Vec<UserSchema>, Error>
    {
        user::select_user(&self.pool, None, None, None, Some(role_id), None, None)
        .await
    }

    pub async fn list_user_by_name(&self, name: &str)
        -> Result<Vec<UserSchema>, Error>
    {
        user::select_user(&self.pool, None, None, None, None, None, Some(name))
        .await
    }

    pub async fn list_user_option(&self, api_id: Option<Uuid>, role_id: Option<Uuid>, name: Option<&str>)
        -> Result<Vec<UserSchema>, Error>
    {
        user::select_user(&self.pool, None, None, api_id, role_id, None, name)
        .await
    }

    pub async fn create_user(&self, id: Uuid, name: &str, email: &str, phone: &str, password: &str)
        -> Result<Uuid, Error>
    {
        user::insert_user(&self.pool, id, name, email, phone, password)
        .await
    }

    pub async fn update_user(&self, id: Uuid, name: Option<&str>, email: Option<&str>, phone: Option<&str>, password: Option<&str>)
        -> Result<(), Error>
    {
        user::update_user(&self.pool, id, name, email, phone, password)
        .await
    }

    pub async fn delete_user(&self, id: Uuid)
        -> Result<(), Error>
    {
        user::delete_user(&self.pool, id)
        .await
    }

    pub async fn add_user_role(&self, id: Uuid, role_id: Uuid)
        -> Result<(), Error>
    {
        user::add_user_role(&self.pool, id, role_id)
        .await
    }

    pub async fn remove_user_role(&self, id: Uuid, role_id: Uuid)
        -> Result<(), Error>
    {
        user::remove_user_role(&self.pool, id, role_id)
        .await
    }

    pub async fn read_access_token(&self, access_id: i32)
        -> Result<TokenSchema, Error>
    {
        match token::select_token(&self.pool, TokenSelector::Access(access_id)).await?.into_iter().next() {
            Some(value) => Ok(value),
            None => Err(Error::RowNotFound)
        }
    }

    pub async fn list_auth_token(&self, auth_token: &str)
        -> Result<Vec<TokenSchema>, Error>
    {
        token::select_token(&self.pool, TokenSelector::Auth(String::from(auth_token)))
        .await
    }

    pub async fn list_token_by_user(&self, user_id: Uuid)
        -> Result<Vec<TokenSchema>, Error>
    {
        token::select_token(&self.pool, TokenSelector::User(user_id))
        .await
    }

    pub async fn create_access_token(&self, user_id: Uuid, auth_token: &str, expire: DateTime<Utc>, ip: &[u8])
        -> Result<(i32, String, String), Error>
    {
        token::insert_token(&self.pool, user_id, Some(auth_token), expire, ip, 1)
        .await?.into_iter().next().ok_or(Error::RowNotFound)
    }

    pub async fn create_auth_token(&self, user_id: Uuid, expire: DateTime<Utc>, ip: &[u8], number: u32)
        -> Result<Vec<(i32, String, String)>, Error>
    {
        token::insert_token(&self.pool, user_id, None, expire, ip, number)
        .await
    }

    pub async fn update_access_token(&self, access_id: i32, expire: Option<DateTime<Utc>>, ip: Option<&[u8]>)
        -> Result<(String, String), Error>
    {
        token::update_token(&self.pool, Some(access_id), None, expire, ip)
        .await
    }

    pub async fn update_auth_token(&self, auth_token: &str, expire: Option<DateTime<Utc>>, ip: Option<&[u8]>)
        -> Result<(String, String), Error>
    {
        token::update_token(&self.pool, None, Some(auth_token), expire, ip)
        .await
    }

    pub async fn delete_access_token(&self, access_id: i32)
        -> Result<(), Error>
    {
        token::delete_token(&self.pool, TokenSelector::Access(access_id))
        .await
    }

    pub async fn delete_auth_token(&self, auth_token: &str)
        -> Result<(), Error>
    {
        token::delete_token(&self.pool, TokenSelector::Auth(auth_token.to_owned()))
        .await
    }

    pub async fn delete_token_by_user(&self, user_id: Uuid)
        -> Result<(), Error>
    {
        token::delete_token(&self.pool, TokenSelector::User(user_id))
        .await
    }

}
