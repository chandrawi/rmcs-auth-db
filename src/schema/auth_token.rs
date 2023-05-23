use sea_query::Iden;
use sqlx::types::chrono::{DateTime, Utc, TimeZone};
use rmcs_auth_api::token;

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

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TokenSchema {
    pub id: String,
    pub role_id: u32,
    pub user_id: u32,
    pub expire: DateTime<Utc>,
    pub limit: u32,
    pub ip: Vec<u8>
}

impl From<token::TokenSchema> for TokenSchema {
    fn from(value: token::TokenSchema) -> Self {
        TokenSchema {
            id: value.id,
            role_id: value.role_id,
            user_id: value.user_id,
            expire: Utc.timestamp_nanos(value.expire),
            limit: value.limit,
            ip: value.ip
        }
    }
}

impl Into<token::TokenSchema> for TokenSchema {
    fn into(self) -> token::TokenSchema {
        token::TokenSchema {
            id: self.id,
            role_id: self.role_id,
            user_id: self.user_id,
            expire: self.expire.timestamp_nanos(),
            limit: self.limit,
            ip: self.ip
        }
    }
}
