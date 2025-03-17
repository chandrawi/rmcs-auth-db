use rand::{thread_rng, Rng};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use sqlx::{Pool, Error, postgres::Postgres};

pub(crate) fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error>
{
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut thread_rng());
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn generate_access_key() -> Vec<u8>
{
    (0..32).map(|_| thread_rng().gen_range(0..255)).collect()
}

pub fn generate_token_string() -> String
{
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
    (0..32).map(|_| CHARSET[thread_rng().gen_range(0..64)] as char).collect()
}

pub async fn migrate(pool: &Pool<Postgres>) -> Result<(), Error>
{
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}
