use sea_query::Iden;

#[allow(unused)]
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

#[allow(unused)]
#[derive(Debug, Default, PartialEq)]
pub struct UserFields {
    pub role_id: u32,
    pub id: u32,
    pub name: String,
    pub password: String,
    pub public_key: String,
    pub private_key: String,
    pub email: String,
    pub phone: String
}
