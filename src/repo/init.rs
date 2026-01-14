use anyhow::{Context, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::crypto::{aead, kdf};

#[derive(Serialize, Deserialize)]
struct RepoConfig {
    salt: [u8; 16],
    encrypted_repo_key: Vec<u8>,
    nonce: [u8; 24],
}

pub fn init_repository(repo_path: &Path) -> Result<()> {
    let repo_dir = repo_path.join("archivist");
    
    if repo_dir.exists() {
        anyhow::bail!("repository already exists");
    }

    fs::create_dir_all(&repo_dir);

    // write version (plaintext, non-secret)
    fs::write(repo_dir.join("version"), b"1")?;

    // read password
    let password = rpassword::prompt_password("Repository password: ")?;

    // generate salt + derive master key
    let salt = kdf::generate_salt();
    let master_key = kdf::derive_master_key(&password, &salt);

    // generate repository key
    let mut repo_key = [0u8; 32];
    rand::rng().fill_bytes(&mut repo_key);

    // encrypt repository key
    let (ciphertext, nonce) =
        aead::encrypt(&*&master_key, &repo_key, b"archivist-repo-key");

    let config = RepoConfig {
        salt,
        encrypted_repo_key: ciphertext,
        nonce,
    };

    let encoded = serde_cbor::to_vec(&config)?;
    fs::write(repo_dir.join("config.enc"), encoded)
        .context("failed to write config")?;

    println!("Initialized Archivist repository at {}", repo_path.display());

    Ok(())
}