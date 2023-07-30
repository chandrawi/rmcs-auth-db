use sea_query::Iden;
use rmcs_auth_api::role;

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum Role {
    Table,
    RoleId,
    ApiId,
    Name,
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
    pub id: i32,
    pub api_id: i32,
    pub name: String,
    pub multi: bool,
    pub ip_lock: bool,
    pub access_duration: i32,
    pub refresh_duration: i32,
    pub access_key: Vec<u8>,
    pub procedures: Vec<i32>
}

impl From<role::RoleSchema> for RoleSchema {
    fn from(value: role::RoleSchema) -> Self {
        Self {
            id: value.id,
            api_id: value.api_id,
            name: value.name,
            multi: value.multi,
            ip_lock: value.ip_lock,
            access_duration: value.access_duration,
            refresh_duration: value.refresh_duration,
            access_key: value.access_key,
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
            multi: self.multi,
            ip_lock: self.ip_lock,
            access_duration: self.access_duration,
            refresh_duration: self.refresh_duration,
            access_key: self.access_key,
            procedures: self.procedures
        }
    }
}
