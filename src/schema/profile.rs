use sea_query::Iden;
use uuid::Uuid;
use rmcs_auth_api::profile;
use rmcs_resource_db::schema::value::{DataValue, DataType};

#[derive(Iden)]
pub(crate) enum ProfileRole {
    Table,
    Id,
    RoleId,
    Name,
    Type,
    Mode
}

#[derive(Iden)]
pub(crate) enum ProfileUser {
    Table,
    Id,
    UserId,
    Name,
    Order,
    Value,
    Type
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum ProfileMode {
    #[default]
    SingleOptional,
    SingleRequired,
    MultipleOptional,
    MultipleRequired
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct RoleProfileSchema {
    pub id: i32,
    pub role_id: Uuid,
    pub name: String,
    pub value_type: DataType,
    pub mode: ProfileMode
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UserProfileSchema {
    pub id: i32,
    pub user_id: Uuid,
    pub name: String,
    pub value: DataValue,
    pub order: i16
}

impl From<profile::RoleProfileSchema> for RoleProfileSchema {
    fn from(value: profile::RoleProfileSchema) -> Self {
        Self {
            id: value.id,
            role_id: Uuid::from_slice(&value.role_id).unwrap_or_default(),
            name: value.name,
            value_type: DataType::from(value.value_type),
            mode: ProfileMode::from(value.mode)
        }
    }
}

impl Into<profile::RoleProfileSchema> for RoleProfileSchema {
    fn into(self) -> profile::RoleProfileSchema {
        profile::RoleProfileSchema {
            id: self.id,
            role_id: self.role_id.as_bytes().to_vec(),
            name: self.name,
            value_type: u32::from(self.value_type),
            mode: self.mode.into()
        }
    }
}

impl From<profile::UserProfileSchema> for UserProfileSchema {
    fn from(value: profile::UserProfileSchema) -> Self {
        Self {
            id: value.id,
            user_id: Uuid::from_slice(&value.user_id).unwrap_or_default(),
            name: value.name,
            order: value.order as i16,
            value: DataValue::from_bytes(value.value_bytes.as_slice(), DataType::from(value.value_type))
        }
    }
}

impl Into<profile::UserProfileSchema> for UserProfileSchema {
    fn into(self) -> profile::UserProfileSchema {
        profile::UserProfileSchema {
            id: self.id,
            user_id: self.user_id.as_bytes().to_vec(),
            name: self.name,
            order: self.order as u32,
            value_type: u32::from(self.value.get_type()),
            value_bytes: self.value.to_bytes()
        }
    }
}

impl From<i16> for ProfileMode {
    fn from(value: i16) -> Self {
        match value {
            1 => Self::SingleRequired,
            2 => Self::MultipleOptional,
            3 => Self::MultipleRequired,
            _ => Self::SingleOptional
        }
    }
}

impl From<ProfileMode> for i16 {
    fn from(value: ProfileMode) -> Self {
        match &value {
            ProfileMode::SingleOptional => 0,
            ProfileMode::SingleRequired => 1,
            ProfileMode::MultipleOptional => 2,
            ProfileMode::MultipleRequired => 3
        }
    }
}

impl From<u32> for ProfileMode {
    fn from(value: u32) -> Self {
        Self::from(value as i16)
    }
}

impl From<ProfileMode> for u32 {
    fn from(value: ProfileMode) -> Self {
        i16::from(value) as u32
    }
}
