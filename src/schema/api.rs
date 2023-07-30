use sea_query::Iden;
use rmcs_auth_api::api;

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum Api {
    Table,
    ApiId,
    Name,
    Address,
    Category,
    Description,
    Password,
    PublicKey,
    PrivateKey,
    AccessKey
}

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum ApiProcedure {
    Table,
    ApiId,
    ProcedureId,
    Name,
    Description
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ApiSchema {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub category: String,
    pub description: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub password: String,
    pub access_key: Vec<u8>,
    pub procedures: Vec<ProcedureSchema>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ProcedureSchema {
    pub id: i32,
    pub api_id: i32,
    pub name: String,
    pub description: String,
    pub roles: Vec<String>
}

impl From<api::ApiSchema> for ApiSchema {
    fn from(value: api::ApiSchema) -> Self {
        Self {
            id: value.id,
            name: value.name,
            address: value.address,
            category: value.category,
            description: value.description,
            public_key: value.public_key,
            private_key: Vec::new(),
            password: value.password,
            access_key: value.access_key,
            procedures: value.procedures.into_iter().map(|e| e.into()).collect()
        }
    }
}

impl Into<api::ApiSchema> for ApiSchema {
    fn into(self) -> api::ApiSchema {
        api::ApiSchema {
            id: self.id,
            name: self.name,
            address: self.address,
            category: self.category,
            description: self.description,
            public_key: self.public_key,
            password: self.password,
            access_key: self.access_key,
            procedures: self.procedures.into_iter().map(|e| e.into()).collect()
        }
    }
}

impl From<api::ProcedureSchema> for ProcedureSchema {
    fn from(value: api::ProcedureSchema) -> Self {
        Self {
            id: value.id,
            api_id: value.api_id,
            name: value.name,
            description: value.description,
            roles: value.roles.into_iter().map(|e| e.into()).collect()
        }
    }
}

impl Into<api::ProcedureSchema> for ProcedureSchema {
    fn into(self) -> api::ProcedureSchema {
        api::ProcedureSchema {
            id: self.id,
            api_id: self.api_id,
            name: self.name,
            description: self.description,
            roles: self.roles.into_iter().map(|e| e.into()).collect()
        }
    }
}
