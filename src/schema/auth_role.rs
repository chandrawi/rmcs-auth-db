use sea_query::Iden;

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum AuthRole {
    Table,
    RoleId,
    Role,
    Secured,
    Multi,
    TokenExpire,
    TokenLimit
}

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum AuthAccess {
    Table,
    RoleId,
    ProcedureId
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct RoleSchema {
    pub id: u32,
    pub name: String,
    pub secured: bool,
    pub multi: bool,
    pub token_expire: u32,
    pub token_limit: u32,
    pub access: Vec<u32>
}
