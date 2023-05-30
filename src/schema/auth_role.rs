use sea_query::Iden;
use rmcs_auth_api::role;

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
pub(crate) enum RoleAccess {
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

impl From<role::RoleSchema> for RoleSchema {
    fn from(value: role::RoleSchema) -> Self {
        RoleSchema {
            id: value.id,
            name: value.name,
            secured: value.secured,
            multi: value.multi,
            token_expire: value.token_expire,
            token_limit: value.token_limit,
            access: value.access
        }
    }
}

impl Into<role::RoleSchema> for RoleSchema {
    fn into(self) -> role::RoleSchema {
        role::RoleSchema {
            id: self.id,
            name: self.name,
            secured: self.secured,
            multi: self.multi,
            token_expire: self.token_expire,
            token_limit: self.token_limit,
            access: self.access
        }
    }
}
