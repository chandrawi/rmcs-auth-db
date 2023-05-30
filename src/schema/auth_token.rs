use sea_query::Iden;
use sqlx::types::chrono::{DateTime, Utc, TimeZone};
use rmcs_auth_api::token;

#[derive(Iden)]
pub(crate) enum Token {
    Table,
    RefreshId,
    AccessId,
    UserId,
    Expire,
    Ip
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TokenSchema {
    pub refresh_id: String,
    pub access_id: u32,
    pub user_id: u32,
    pub expire: DateTime<Utc>,
    pub ip: Vec<u8>
}

impl From<token::TokenSchema> for TokenSchema {
    fn from(value: token::TokenSchema) -> Self {
        TokenSchema {
            refresh_id: value.refresh_id,
            access_id: value.access_id,
            user_id: value.user_id,
            expire: Utc.timestamp_nanos(value.expire),
            ip: value.ip
        }
    }
}

impl Into<token::TokenSchema> for TokenSchema {
    fn into(self) -> token::TokenSchema {
        token::TokenSchema {
            refresh_id: self.refresh_id,
            access_id: self.access_id,
            user_id: self.user_id,
            expire: self.expire.timestamp_nanos(),
            ip: self.ip
        }
    }
}
