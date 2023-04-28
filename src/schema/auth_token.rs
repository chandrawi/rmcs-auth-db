use sea_query::Iden;
use sqlx::types::chrono::{DateTime, Utc};

#[allow(unused)]
#[derive(Iden)]
pub(crate) enum AuthToken {
    Table,
    Id,
    RoleId,
    UserId,
    Expire,
    Limit,
    Ip
}

#[allow(unused)]
#[derive(Debug, Default, PartialEq)]
pub struct TokenFields {
    pub id: String,
    pub role_id: u32,
    pub user_id: u32,
    pub expire: DateTime<Utc>,
    pub limit: u32,
    pub ip: Vec<u8>
}
