use sea_query::Iden;
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;
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
    pub access_id: i32,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub auth_token: String,
    pub expire: NaiveDateTime,
    pub ip: Vec<u8>
}

impl From<token::TokenSchema> for TokenSchema {
    fn from(value: token::TokenSchema) -> Self {
        TokenSchema {
            access_id: value.access_id,
            user_id: Uuid::from_slice(&value.user_id).unwrap_or_default(),
            refresh_token: value.refresh_token,
            auth_token: value.auth_token,
            expire: NaiveDateTime::from_timestamp_micros(value.expire).unwrap_or_default(),
            ip: value.ip
        }
    }
}

impl Into<token::TokenSchema> for TokenSchema {
    fn into(self) -> token::TokenSchema {
        token::TokenSchema {
            access_id: self.access_id,
            user_id: self.user_id.as_bytes().to_vec(),
            refresh_token: self.refresh_token,
            auth_token: self.auth_token,
            expire: self.expire.timestamp_micros(),
            ip: self.ip
        }
    }
}
