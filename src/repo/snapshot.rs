use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::crypto::{aead, hash};
use crate::repo::backend::Backend;
use crate::repo::{
    backend::ObjectType,
    open::RepoContext,
};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub root_tree: String,
    pub parent: Option<String>,
    pub timestamp: u64,
    pub hostname: String,
    pub archivist_version: String,
}

impl Snapshot {
    pub fn create(ctx: &RepoContext, root_tree: String) -> Result<String> {
        let parent = Self::read_latest(ctx).ok();

        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let snapshot = Snapshot {
            root_tree,
            parent,
            timestamp,
            hostname,
            archivist_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let serialized = serde_cbor::to_vec(&snapshot)?;
        let snapshot_hash = hash::sha256_hex(&serialized);

        if !ctx.backend.exists(ObjectType::Snapshot, &snapshot_hash)? {
            let (ciphertext, nonce) = aead::encrypt(
                &ctx.repo_key, 
                &serialized, 
                b"archivist-snapshot-v1",
            );

            let mut stored = Vec::with_capacity(24 + ciphertext.len());
            stored.extend_from_slice(&nonce);
            stored.extend_from_slice(&ciphertext);

            ctx.backend.put(ObjectType::Snapshot, &snapshot_hash, &stored)?;
        }

        Self::write_latest(ctx, &snapshot_hash)?;

        Ok(snapshot_hash)
    }

    pub fn restore(ctx: &RepoContext, hash: &str, target: &Path) -> Result<()> {
        let stored = ctx.backend.get(ObjectType::Snapshot, hash)?;

        if stored.len() < 24 {
            anyhow::bail!("corrupted snapshot {}", hash);
        }

        let (nonce, ciphertext) = stored.split_at(24);

        let plaintext = aead::decrypt(
            &ctx.repo_key,
            ciphertext,
            nonce.try_into().unwrap(),
            b"archivist-snapshot-v1",
        )
        .map_err(|e| anyhow::anyhow!("snapshot decryption failed: {:?}", e))?;

        let computed = hash::sha256_hex(&plaintext);
        if computed != hash {
            anyhow::bail!("snapshot hash mismatch");
        }

        let snapshot: Snapshot = serde_cbor::from_slice(&plaintext)?;
        crate::repo::tree::Tree::restore(ctx, &snapshot.root_tree, target)?;

        Ok(())
    }

    pub fn load(ctx: &RepoContext, hash: &str) -> Result<Snapshot> {
        let stored = ctx.backend.get(ObjectType::Snapshot, hash)?;

        if stored.len() < 24 {
            anyhow::bail!("corrupted snapshot {}", hash);
        }

        let (nonce, ciphertext) = stored.split_at(24);

        let plaintext = aead::decrypt(
            &ctx.repo_key,
            ciphertext,
            nonce.try_into().unwrap(),
            b"archivist-snapshot-v1",
        )
        .map_err(|e| anyhow::anyhow!("snapshot decryption failed: {:?}", e))?;

        let computed = hash::sha256_hex(&plaintext);
        if computed != hash {
            anyhow::bail!("snapshot hash mismatch");
        }

        let snapshot: Snapshot = serde_cbor::from_slice(&plaintext)?;
        Ok(snapshot)
    }

    pub fn read_latest(ctx: &RepoContext) -> Result<String> {
        let path = ctx.root.join("index").join("latest");
        let data = fs::read_to_string(&path)
            .context("failed to read snapshot index")?;

        Ok(data.trim().to_string())
    }

    fn write_latest(ctx: &RepoContext, hash: &str) -> Result<()> {
        let index_dir = ctx.root.join("index");
        fs::create_dir_all(&index_dir)?;

        let tmp = index_dir.join("latest.tmp");
        let final_path = index_dir.join("latest");

        fs::write(&tmp, format!("{}\n", hash))?;

        let dir = fs::File::open(&index_dir)?;
        fs::rename(&tmp, &final_path)?;
        dir.sync_all()?;

        Ok(())
    }
}