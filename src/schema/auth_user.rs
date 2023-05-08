use sea_query::Iden;

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
    pub role_id: u32,
    pub id: u32,
    pub name: String,
    pub password: String,
    pub public_key: String,
    pub private_key: String,
    pub email: String,
    pub phone: String
}
