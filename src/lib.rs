pub mod schema;
pub(crate) mod operation;

use sqlx::Pool;
use sqlx::mysql::{MySql, MySqlPoolOptions};
use sqlx::types::chrono::{DateTime, Utc};

use schema::api::{ApiKind, ApiFields, ProcedureFields};
use schema::auth_role::RoleFields;
use schema::auth_user::UserFields;
use operation::api;
use operation::role;
use operation::user;

pub struct Auth {
    pub pool: Pool<MySql>,
    options: AuthOptions
}

#[derive(Debug)]
pub struct AuthOptions {
    limit: u32,
    with_description: bool,
    order: Vec<OrderOption>
}

#[derive(Debug)]
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

    pub async fn read_resource(&self, id: u32)
        -> Result<ApiFields, sqlx::Error>
    {
        api::select_api_by_id(&self.pool, ApiKind::Resource, id)
        .await
    }

    pub async fn read_resource_by_name(&self, name: &str)
        -> Result<ApiFields, sqlx::Error>
    {
        api::select_api_by_name(&self.pool, ApiKind::Resource, name)
        .await
    }

    pub async fn list_resource(&self)
        -> Result<Vec<ApiFields>, sqlx::Error>
    {
        api::select_multiple_api(&self.pool, ApiKind::Resource).await
    }

    pub async fn create_resource(&self, name: &str, address: &str, description: Option<&str>)
        -> Result<u32, sqlx::Error>
    {
        api::insert_api(&self.pool, ApiKind::Resource, name, address, description)
        .await
    }

    pub async fn update_resource(&self, id: u32, name: Option<&str>, address: Option<&str>, description: Option<&str>)
        -> Result<(), sqlx::Error>
    {
        api::update_api(&self.pool, ApiKind::Resource, id, name, address, description)
        .await
    }

    pub async fn delete_resource(&self, id: u32)
        -> Result<(), sqlx::Error>
    {
        api::delete_api(&self.pool, ApiKind::Resource, id)
        .await
    }

    pub async fn read_application(&self, id: u32)
        -> Result<ApiFields, sqlx::Error>
    {
        api::select_api_by_id(&self.pool, ApiKind::Application, id)
        .await
    }

    pub async fn read_application_by_name(&self, name: &str)
        -> Result<ApiFields, sqlx::Error>
    {
        api::select_api_by_name(&self.pool, ApiKind::Application, name)
        .await
    }

    pub async fn list_application(&self)
        -> Result<Vec<ApiFields>, sqlx::Error>
    {
        api::select_multiple_api(&self.pool, ApiKind::Application)
        .await
    }

    pub async fn create_application(&self, name: &str, address: &str, description: Option<&str>)
        -> Result<u32, sqlx::Error>
    {
        api::insert_api(&self.pool, ApiKind::Application, name, address, description)
        .await
    }

    pub async fn update_application(&self, id: u32, name: Option<&str>, address: Option<&str>, description: Option<&str>)
        -> Result<(), sqlx::Error>
    {
        api::update_api(&self.pool, ApiKind::Application, id, name, address, description)
        .await
    }

    pub async fn delete_application(&self, id: u32)
        -> Result<(), sqlx::Error>
    {
        api::delete_api(&self.pool, ApiKind::Application, id)
        .await
    }

    pub async fn read_procedure(&self, id: u32)
        -> Result<ProcedureFields, sqlx::Error>
    {
        api::select_procedure_by_id(&self.pool, id)
        .await
    }

    pub async fn list_procedure_by_api(&self, api_id: u32)
        -> Result<Vec<ProcedureFields>, sqlx::Error>
    {
        api::select_multiple_procedure(&self.pool, api_id)
        .await
    }

    pub async fn read_procedure_by_name(&self, api_id: u32, service: &str, name: &str)
        -> Result<ProcedureFields, sqlx::Error>
    {
        api::select_procedure_by_name(&self.pool, api_id, service, name)
        .await
    }

    pub async fn create_procedure(&self, api_id: u32, service: &str, name: &str, description: Option<&str>)
        -> Result<u32, sqlx::Error>
    {
        api::insert_procedure(&self.pool, api_id, service, name, description)
        .await
    }

    pub async fn update_procedure(&self, id: u32, service: Option<&str>, name: Option<&str>, description: Option<&str>)
        -> Result<(), sqlx::Error>
    {
        api::update_procedure(&self.pool, id, service, name, description)
        .await
    }

    pub async fn delete_procedure(&self, id: u32)
        -> Result<(), sqlx::Error>
    {
        api::delete_procedure(&self.pool, id)
        .await
    }

    pub async fn read_role(&self, id: u32)
        -> Result<RoleFields, sqlx::Error>
    {
        role::select_role_by_id(&self.pool, id)
        .await
    }

    pub async fn read_role_by_name(&self, name: &str)
        -> Result<RoleFields, sqlx::Error>
    {
        role::select_role_by_name(&self.pool, name)
        .await
    }

    pub async fn list_role(&self)
        -> Result<Vec<RoleFields>, sqlx::Error>
    {
        role::select_role_all(&self.pool)
        .await
    }

    pub async fn create_role(&self, name: &str, secured: bool, multi: bool, token_expire: Option<u32>, token_limit: Option<u32>)
        -> Result<u32, sqlx::Error>
    {
        role::insert_role(&self.pool, name, secured, multi, token_expire, token_limit)
        .await
    }

    pub async fn update_role(&self, id: u32, name: Option<&str>, secured: Option<bool>, multi: Option<bool>, token_expire: Option<u32>, token_limit: Option<u32>)
        -> Result<(), sqlx::Error>
    {
        role::update_role(&self.pool, id, name, secured, multi, token_expire, token_limit)
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
        -> Result<UserFields, sqlx::Error>
    {
        user::select_user_by_id(&self.pool, id)
        .await
    }

    pub async fn read_user_by_name(&self, name: &str)
        -> Result<UserFields, sqlx::Error>
    {
        user::select_user_by_name(&self.pool, name)
        .await
    }

    pub async fn list_user_by_role(&self, role_id: u32)
        -> Result<Vec<UserFields>, sqlx::Error>
    {
        user::select_multiple_user_by_role(&self.pool, role_id)
        .await
    }

    pub async fn create_user(&self, role_id: u32, name: &str, password: &str, public_key: &str, private_key: &str, email: Option<&str>, phone: Option<&str>)
        -> Result<u32, sqlx::Error>
    {
        user::insert_user(&self.pool, role_id, name, password, public_key, private_key, email, phone)
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

}
