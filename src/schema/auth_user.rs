use sea_query::Iden;
use rmcs_auth_api::user;

#[derive(Iden)]
pub(crate) enum AuthUser {
    Table,
    RoleId,
    UserId,
    User,
    Password,
    PublicKey,
    PrivateKey,
    Email,
    Phone
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UserSchema {
    pub id: u32,
    pub role_id: u32,
    pub name: String,
    pub password: String,
    pub public_key: String,
    pub private_key: String,
    pub email: String,
    pub phone: String
}

impl From<user::UserSchema> for UserSchema {
    fn from(value: user::UserSchema) -> Self {
        UserSchema {
            id: value.id,
            role_id: value.role_id,
            name: value.name,
            password: value.password,
            public_key: value.public_key,
            private_key: value.private_key,
            email: value.email,
            phone: value.phone
        }
    }
}

impl Into<user::UserSchema> for UserSchema {
    fn into(self) -> user::UserSchema {
        user::UserSchema {
            id: self.id,
            role_id: self.role_id,
            name: self.name,
            password: self.password,
            public_key: self.public_key,
            private_key: self.private_key,
            email: self.email,
            phone: self.phone
        }
    }
}
