use sea_query::Iden;
use uuid::Uuid;
use rmcs_auth_api::role;

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

#[derive(Iden)]
pub(crate) enum RoleAccess {
    Table,
    RoleId,
    ProcedureId
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct RoleSchema {
    pub id: Uuid,
    pub api_id: Uuid,
    pub name: String,
    pub multi: bool,
    pub ip_lock: bool,
    pub access_duration: i32,
    pub refresh_duration: i32,
    pub access_key: Vec<u8>,
    pub procedures: Vec<Uuid>
}

impl From<role::RoleSchema> for RoleSchema {
    fn from(value: role::RoleSchema) -> Self {
        Self {
            id: Uuid::from_slice(&value.id).unwrap_or_default(),
            api_id: Uuid::from_slice(&value.api_id).unwrap_or_default(),
            name: value.name,
            multi: value.multi,
            ip_lock: value.ip_lock,
            access_duration: value.access_duration,
            refresh_duration: value.refresh_duration,
            access_key: value.access_key,
            procedures: value.procedures.into_iter().map(|u| Uuid::from_slice(&u).unwrap_or_default()).collect()
        }
    }
}

impl Into<role::RoleSchema> for RoleSchema {
    fn into(self) -> role::RoleSchema {
        role::RoleSchema {
            id: self.id.as_bytes().to_vec(),
            api_id: self.api_id.as_bytes().to_vec(),
            name: self.name,
            multi: self.multi,
            ip_lock: self.ip_lock,
            access_duration: self.access_duration,
            refresh_duration: self.refresh_duration,
            access_key: self.access_key,
            procedures: self.procedures.into_iter().map(|u| u.as_bytes().to_vec()).collect()
        }
    }
}
