use sea_query::Iden;

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
