use std::sync::LazyLock;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

static DUMMY_HASH: LazyLock<String> =
    LazyLock::new(|| hash("dummy password").expect("failed to generate dummy hash."));

pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string())
}

pub fn verify(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn verify_login(password: &str, stored_hash: Option<&str>) -> bool {
    match stored_hash {
        Some(stored_hash) => verify(password, stored_hash),
        None => {
            verify(password, &DUMMY_HASH);
            false
        }
    }
}
