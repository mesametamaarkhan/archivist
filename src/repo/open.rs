use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs};
use std::path::{Path, PathBuf};
use crate::crypto::{aead, kdf};
use crate::repo::local::LocalBackend;
use crate::repo::lock::RepoLock;

#[derive(Deserialize)]
struct RepoConfig {
    salt: [u8; 16],
    encrypted_repo_key: Vec<u8>,
    nonce: [u8; 24],
}

pub struct RepoContext {
    pub root: PathBuf,
    pub repo_key: [u8; 32],
    pub version: u32,
    pub backend: LocalBackend,
    _lock: Option<RepoLock>
}

pub fn open_repository(repo_path: &Path, write: bool) -> Result<RepoContext> {
    let repo_dir = repo_path.join("archivist");

    if !repo_dir.exists() {
        anyhow::bail!("not an archivist repository");
    }

    let lock = if write {
        Some(RepoLock::acquire(repo_path)?)
    } else {
        None
    };

    // read version
    let version_bytes = fs::read(repo_dir.join("version"))
        .context("failed to read repo version")?;
    let version_str = std::str::from_utf8(&version_bytes)?;
    let version: u32 = version_str.trim().parse()
        .context("invalid repo version")?;

    if version != 1 {
        anyhow::bail!("unsupported repository version {}", version);
    }

    // read encrypted config
    let config_bytes = fs::read(repo_dir.join("config.enc"))
        .context("failed to read config")?;
    let config: RepoConfig = serde_cbor::from_slice(&config_bytes)
        .context("invalid config format")?;

    // prompt for password
    let password = rpassword::prompt_password("Repository password: ")?;

    // derive master key
    let master_key = kdf::derive_master_key(&password, &config.salt);

    //derive repository key
    let repo_key_bytes = aead::decrypt(
        &*master_key,
        &config.encrypted_repo_key,
        &config.nonce,
        b"archivist-repo-key",
    )
    .map_err(|_| anyhow::anyhow!("invalid password or corrupted repository"))?;

    if repo_key_bytes.len() != 32 {
        anyhow::bail!("invalid repository key length");
    }

    let mut repo_key = [0u8; 32];
    repo_key.copy_from_slice(&repo_key_bytes);

    let backend = LocalBackend::new(&repo_dir);

    Ok(RepoContext { 
        root: repo_dir, 
        repo_key, 
        version,
        backend,
        _lock: lock
    })
}