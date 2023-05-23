use sea_query::Iden;
use rmcs_auth_api::api;

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum Api {
    Table,
    ApiId,
    Name,
    Kind,
    Address,
    Description
}

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum ApiProcedure {
    Table,
    ApiId,
    ProcedureId,
    Service,
    Procedure,
    Description
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ApiKind {
    Resource,
    Application
}

impl std::string::ToString for ApiKind {
    fn to_string(&self) -> String {
        match &self {
            ApiKind::Resource => String::from("RESOURCE"),
            ApiKind::Application => String::from("APPLICATION")
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ApiSchema {
    pub id: u32,
    pub name: String,
    pub address: String,
    pub description: String,
    pub procedures: Vec<ProcedureSchema>
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ProcedureSchema {
    pub id: u32,
    pub api_id: u32,
    pub service: String,
    pub procedure: String,
    pub description: String
}

impl From<api::ApiSchema> for ApiSchema {
    fn from(value: api::ApiSchema) -> Self {
        ApiSchema {
            id: value.id,
            name: value.name,
            address: value.address,
            description: value.description,
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
            description: self.description,
            procedures: self.procedures.into_iter().map(|e| e.into()).collect()
        }
    }
}

impl From<api::ProcedureSchema> for ProcedureSchema {
    fn from(value: api::ProcedureSchema) -> Self {
        ProcedureSchema {
            id: value.id,
            api_id: value.api_id,
            service: value.service,
            procedure: value.procedure,
            description: value.description
        }
    }
}

impl Into<api::ProcedureSchema> for ProcedureSchema {
    fn into(self) -> api::ProcedureSchema {
        api::ProcedureSchema {
            id: self.id,
            api_id: self.api_id,
            service: self.service,
            procedure: self.procedure,
            description: self.description
        }
    }
}
