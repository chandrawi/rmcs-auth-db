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
    Password,
    PublicKey,
    PrivateKey,
    Description
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
    pub id: u32,
    pub name: String,
    pub address: String,
    pub category: String,
    pub description: String,
    pub public_key: Vec<u8>,
    pub private_key: Option<Vec<u8>>,
    pub password: Option<String>,
    pub procedures: Vec<ProcedureSchema>,
    pub access_keys: Vec<AccessKeySchema>
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ProcedureSchema {
    pub id: u32,
    pub api_id: u32,
    pub name: String,
    pub description: String,
    pub roles: Vec<String>
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct AccessKeySchema {
    pub role: String,
    pub access_key: Vec<u8>
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
            private_key: None,
            password: value.password,
            procedures: value.procedures.into_iter().map(|e| e.into()).collect(),
            access_keys: value.access_keys.into_iter().map(|e| e.into()).collect()
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
            procedures: self.procedures.into_iter().map(|e| e.into()).collect(),
            access_keys: self.access_keys.into_iter().map(|e| e.into()).collect()
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

impl From<api::AccessKeySchema> for AccessKeySchema {
    fn from(value: api::AccessKeySchema) -> Self {
        Self {
            role: value.role,
            access_key: value.access_key,
        }
    }
}

impl Into<api::AccessKeySchema> for AccessKeySchema {
    fn into(self) -> api::AccessKeySchema {
        api::AccessKeySchema {
            role: self.role,
            access_key: self.access_key,
        }
    }
}
