use sea_query::Iden;
use rmcs_auth_api::role;

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum Role {
    Table,
    ApiId,
    RoleId,
    Name,
    AccessKey,
    Multi,
    IpLock,
    AccessDuration,
    RefreshDuration
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
    pub api_id: u32,
    pub name: String,
    pub access_key: String,
    pub multi: bool,
    pub ip_lock: bool,
    pub access_duration: u32,
    pub refresh_duration: u32,
    pub procedures: Vec<u32>
}

impl From<role::RoleSchema> for RoleSchema {
    fn from(value: role::RoleSchema) -> Self {
        Self {
            id: value.id,
            api_id: value.api_id,
            name: value.name,
            access_key: value.access_key,
            multi: value.multi,
            ip_lock: value.ip_lock,
            refresh_duration: value.refresh_duration,
            access_duration: value.access_duration,
            procedures: value.procedures
        }
    }
}

impl Into<role::RoleSchema> for RoleSchema {
    fn into(self) -> role::RoleSchema {
        role::RoleSchema {
            id: self.id,
            api_id: self.api_id,
            name: self.name,
            access_key: self.access_key,
            multi: self.multi,
            ip_lock: self.ip_lock,
            refresh_duration: self.refresh_duration,
            access_duration: self.access_duration,
            procedures: self.procedures
        }
    }
}
