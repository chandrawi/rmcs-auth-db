use sea_query::Iden;
use rmcs_auth_api::user;

#[derive(Iden)]
pub(crate) enum User {
    Table,
    UserId,
    Name,
    Password,
    PublicKey,
    PrivateKey,
    Email,
    Phone
}

#[derive(Iden)]
pub(crate) enum UserRole {
    Table,
    UserId,
    RoleId
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UserSchema {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub password: String,
    pub roles: Vec<UserRoleSchema>
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UserRoleSchema {
    pub api_id: u32,
    pub role: String,
    pub multi: bool,
    pub ip_lock: bool,
    pub access_duration: u32,
    pub refresh_duration: u32,
    pub access_key: Vec<u8>
}

impl From<user::UserSchema> for UserSchema {
    fn from(value: user::UserSchema) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            phone: value.phone,
            public_key: value.public_key,
            private_key: Vec::new(),
            password: value.password,
            roles: value.roles.into_iter().map(|e| e.into()).collect()
        }
    }
}

impl Into<user::UserSchema> for UserSchema {
    fn into(self) -> user::UserSchema {
        user::UserSchema {
            id: self.id,
            name: self.name,
            email: self.email,
            phone: self.phone,
            public_key: self.public_key,
            password: self.password,
            roles: self.roles.into_iter().map(|e| e.into()).collect()
        }
    }
}

impl From<user::UserRoleSchema> for UserRoleSchema {
    fn from(value: user::UserRoleSchema) -> Self {
        Self {
            api_id: value.api_id,
            role: value.role,
            multi: value.multi,
            ip_lock: value.ip_lock,
            access_duration: value.access_duration,
            refresh_duration: value.refresh_duration,
            access_key: value.access_key
        }
    }
}

impl Into<user::UserRoleSchema> for UserRoleSchema {
    fn into(self) -> user::UserRoleSchema {
        user::UserRoleSchema {
            api_id: self.api_id,
            role: self.role,
            multi: self.multi,
            ip_lock: self.ip_lock,
            access_duration: self.access_duration,
            refresh_duration: self.refresh_duration,
            access_key: self.access_key
        }
    }
}
