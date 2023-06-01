pub mod schema;
pub(crate) mod operation;
pub(crate) mod crypto;

use sqlx::Pool;
use sqlx::mysql::{MySql, MySqlPoolOptions};
use sqlx::types::chrono::{DateTime, Utc};

pub use schema::api::{ApiSchema, ProcedureSchema};
pub use schema::auth_role::RoleSchema;
pub use schema::auth_user::UserSchema;
pub use schema::auth_token::TokenSchema;
use operation::api;
use operation::role;
use operation::user;
use operation::token;

#[derive(Debug, Clone)]
pub struct Auth {
    pub pool: Pool<MySql>,
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
        let pool = MySqlPoolOptions::new()
            .max_connections(100)
            .connect(url)
            .await
            .expect(&format!("Error connecting to {}", url));
        Auth {
            pool,
            options: AuthOptions::default()
        }
    }

    pub fn new_with_pool(pool: Pool<MySql>) -> Auth {
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

    pub async fn read_api(&self, id: u32)
        -> Result<ApiSchema, sqlx::Error>
    {
        api::select_api_by_id(&self.pool, id)
        .await
    }

    pub async fn read_api_by_name(&self, name: &str)
        -> Result<ApiSchema, sqlx::Error>
    {
        api::select_api_by_name(&self.pool, name)
        .await
    }

    pub async fn list_api_by_category(&self, category: &str)
        -> Result<Vec<ApiSchema>, sqlx::Error>
    {
        api::select_api_by_category(&self.pool, category).await
    }

    pub async fn create_api(&self, name: &str, address: &str, category: &str, description: Option<&str>)
        -> Result<u32, sqlx::Error>
    {
        api::insert_api(&self.pool, name, address, category, description)
        .await
    }

    pub async fn update_api(&self, id: u32, name: Option<&str>, address: Option<&str>, category: Option<&str>, description: Option<&str>)
        -> Result<(), sqlx::Error>
    {
        api::update_api(&self.pool, id, name, address, category, description)
        .await
    }

    pub async fn delete_api(&self, id: u32)
        -> Result<(), sqlx::Error>
    {
        api::delete_api(&self.pool, id)
        .await
    }

    pub async fn read_procedure(&self, id: u32)
        -> Result<ProcedureSchema, sqlx::Error>
    {
        api::select_procedure_by_id(&self.pool, id)
        .await
    }

    pub async fn read_procedure_by_name(&self, api_id: u32, service: &str, name: &str)
        -> Result<ProcedureSchema, sqlx::Error>
    {
        api::select_procedure_by_name(&self.pool, api_id, service, name)
        .await
    }

    pub async fn list_procedure_by_api(&self, api_id: u32)
        -> Result<Vec<ProcedureSchema>, sqlx::Error>
    {
        api::select_procedure_by_api(&self.pool, api_id)
        .await
    }

    pub async fn create_procedure(&self, api_id: u32, name: &str, description: Option<&str>)
        -> Result<u32, sqlx::Error>
    {
        api::insert_procedure(&self.pool, api_id, name, description)
        .await
    }

    pub async fn update_procedure(&self, id: u32, name: Option<&str>, description: Option<&str>)
        -> Result<(), sqlx::Error>
    {
        api::update_procedure(&self.pool, id, name, description)
        .await
    }

    pub async fn delete_procedure(&self, id: u32)
        -> Result<(), sqlx::Error>
    {
        api::delete_procedure(&self.pool, id)
        .await
    }

    pub async fn read_role(&self, id: u32)
        -> Result<RoleSchema, sqlx::Error>
    {
        role::select_role_by_id(&self.pool, id)
        .await
    }

    pub async fn read_role_by_name(&self, api_id: u32, name: &str)
        -> Result<RoleSchema, sqlx::Error>
    {
        role::select_role_by_name(&self.pool, api_id, name)
        .await
    }

    pub async fn list_role_by_api(&self, api_id: u32)
        -> Result<Vec<RoleSchema>, sqlx::Error>
    {
        role::select_role_by_api(&self.pool, api_id)
        .await
    }

    pub async fn list_role_by_user(&self, user_id: u32)
        -> Result<Vec<RoleSchema>, sqlx::Error>
    {
        role::select_role_by_user(&self.pool, user_id)
        .await
    }

    pub async fn create_role(&self, api_id: u32, name: &str, access_key: &str, multi: bool, ip_lock: bool, access_duration: Option<u32>, refresh_duration: Option<u32>)
        -> Result<u32, sqlx::Error>
    {
        role::insert_role(&self.pool, api_id, name, access_key, multi, ip_lock, access_duration, refresh_duration)
        .await
    }

    pub async fn update_role(&self, id: u32, name: Option<&str>, access_key: Option<&str>, multi: Option<bool>, ip_lock: Option<bool>, access_duration: Option<u32>, refresh_duration: Option<u32>)
        -> Result<(), sqlx::Error>
    {
        role::update_role(&self.pool, id, name, access_key, multi, ip_lock, access_duration, refresh_duration)
        .await
    }

    pub async fn delete_role(&self, id: u32)
        -> Result<(), sqlx::Error>
    {
        role::delete_role(&self.pool, id)
        .await
    }

    pub async fn add_role_access(&self, id: u32, procedure_id: u32)
        -> Result<(), sqlx::Error>
    {
        role::add_role_access(&self.pool, id, procedure_id)
        .await
    }

    pub async fn remove_role_access(&self, id: u32, procedure_id: u32)
        -> Result<(), sqlx::Error>
    {
        role::remove_role_access(&self.pool, id, procedure_id)
        .await
    }

    pub async fn read_user(&self, id: u32)
        -> Result<UserSchema, sqlx::Error>
    {
        user::select_user_by_id(&self.pool, id)
        .await
    }

    pub async fn read_user_by_name(&self, name: &str)
        -> Result<UserSchema, sqlx::Error>
    {
        user::select_user_by_name(&self.pool, name)
        .await
    }

    pub async fn list_user_by_role(&self, role_id: u32)
        -> Result<Vec<UserSchema>, sqlx::Error>
    {
        user::select_multiple_user_by_role(&self.pool, role_id)
        .await
    }

    pub async fn create_user(&self, name: &str, password: &str, public_key: &str, private_key: &str, email: Option<&str>, phone: Option<&str>)
        -> Result<u32, sqlx::Error>
    {
        user::insert_user(&self.pool, name, password, public_key, private_key, email, phone)
        .await
    }

    pub async fn update_user(&self, id: u32, name: Option<&str>, password: Option<&str>, public_key: Option<&str>, private_key: Option<&str>, email: Option<&str>, phone: Option<&str>)
        -> Result<(), sqlx::Error>
    {
        user::update_user(&self.pool, id, name, password, public_key, private_key, email, phone)
        .await
    }

    pub async fn delete_user(&self, id: u32)
        -> Result<(), sqlx::Error>
    {
        user::delete_user(&self.pool, id)
        .await
    }

    pub async fn read_refresh_token(&self, refresh_id: &str)
        -> Result<TokenSchema, sqlx::Error>
    {
        token::select_token_by_refresh(&self.pool, refresh_id)
        .await
    }

    pub async fn list_access_token(&self, access_id: u32)
        -> Result<Vec<TokenSchema>, sqlx::Error>
    {
        token::select_token_by_access(&self.pool, access_id)
        .await
    }

    pub async fn list_user_token(&self, user_id: u32)
        -> Result<Vec<TokenSchema>, sqlx::Error>
    {
        token::select_token_by_user(&self.pool, user_id)
        .await
    }

    pub async fn create_token(&self, user_id: u32, expire: DateTime<Utc>, ip: Option<&[u8]>)
        -> Result<(u32, String), sqlx::Error>
    {
        token::insert_token(&self.pool, user_id, expire, ip)
        .await
    }

    pub async fn delete_refresh_token(&self, refresh_id: &str)
        -> Result<(), sqlx::Error>
    {
        token::delete_token_by_refresh(&self.pool, refresh_id)
        .await
    }

    pub async fn delete_access_token(&self, access_id: u32)
        -> Result<(), sqlx::Error>
    {
        token::delete_token_by_access(&self.pool, access_id)
        .await
    }

    pub async fn delete_user_token(&self, user_id: u32)
        -> Result<(), sqlx::Error>
    {
        token::delete_token_by_user(&self.pool, user_id)
        .await
    }

}
