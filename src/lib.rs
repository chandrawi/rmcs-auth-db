pub mod schema;
pub(crate) mod operation;

use sqlx::Pool;
use sqlx::mysql::{MySql, MySqlPoolOptions};
use sqlx::types::chrono::{DateTime, Utc};

use schema::api::{ApiKind, ApiFields, ProcedureFields};
use operation::api;

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

}
