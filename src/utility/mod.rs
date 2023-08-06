use rand::{thread_rng, Rng};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

pub(crate) fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error>
{
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut thread_rng());
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub(crate) fn generate_random_bytes(number: usize) -> Vec<u8>
{
    (0..number).map(|_| thread_rng().gen()).collect()
}

pub(crate) fn generate_random_base64(number: usize) -> String
{
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
    (0..number).map(|_| CHARSET[thread_rng().gen_range(0..64)] as char).collect()
}
