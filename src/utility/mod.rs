use rand::{thread_rng, Rng};
use rsa::{RsaPrivateKey, RsaPublicKey};
use pkcs8::{EncodePrivateKey, EncodePublicKey};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

pub(crate) fn generate_keys() -> Result<(RsaPrivateKey, RsaPublicKey), rsa::Error>
{
    let mut rng = thread_rng();
    let bits = 1024;
    let priv_key = RsaPrivateKey::new(&mut rng, bits)?;
    let pub_key = RsaPublicKey::from(&priv_key);
    Ok((priv_key, pub_key))
}

pub(crate) fn export_private_key(priv_key: RsaPrivateKey) -> Result<Vec<u8>, pkcs8::Error>
{
    let priv_der = priv_key.to_pkcs8_der()?.to_bytes().to_vec();
    Ok(priv_der)
}

pub(crate) fn export_public_key(pub_key: RsaPublicKey) -> Result<Vec<u8>, spki::Error>
{
    let pub_der = pub_key.to_public_key_der()?.to_vec();
    Ok(pub_der)
}

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
