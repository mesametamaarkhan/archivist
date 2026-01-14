use argon2::{Argon2, PasswordHasher};
use rand::RngCore;
use zeroize::Zeroizing;

pub fn derive_master_key(password: &str, salt: &[u8]) -> Zeroizing<[u8; 32]> {
    let argon2 = Argon2::default();
    let mut key = Zeroizing::new([0u8;32]);

    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .expect("argon2 failed");

    key
}

pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::rng().fill_bytes(&mut salt);
    salt
}