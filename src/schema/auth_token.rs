use sea_query::Iden;
use sqlx::types::chrono::{DateTime, Utc, TimeZone};
use rmcs_auth_api::token;

#[derive(Iden)]
pub(crate) enum Token {
    Table,
    AccessId,
    UserId,
    RefreshToken,
    AuthToken,
    Expire,
    Ip
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TokenSchema {
    pub access_id: u32,
    pub user_id: u32,
    pub refresh_token: String,
    pub auth_token: String,
    pub expire: DateTime<Utc>,
    pub ip: Vec<u8>
}

impl From<token::TokenSchema> for TokenSchema {
    fn from(value: token::TokenSchema) -> Self {
        TokenSchema {
            access_id: value.access_id,
            user_id: value.user_id,
            refresh_token: value.refresh_token,
            auth_token: value.auth_token,
            expire: Utc.timestamp_nanos(value.expire),
            ip: value.ip
        }
    }
}

impl Into<token::TokenSchema> for TokenSchema {
    fn into(self) -> token::TokenSchema {
        token::TokenSchema {
            access_id: self.access_id,
            user_id: self.user_id,
            refresh_token: self.refresh_token,
            auth_token: self.auth_token,
            expire: self.expire.timestamp_nanos(),
            ip: self.ip
        }
    }
}
