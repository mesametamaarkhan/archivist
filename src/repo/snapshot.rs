use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{any, fs};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::crypto::{aead, hash};
use crate::repo::backend::{Backend, ObjectType};
use crate::repo::open::RepoContext;

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

        let snapshot = Snapshot {
            root_tree,
            parent,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            hostname: hostname::get()?.to_string_lossy().to_string(),
            archivist_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let serialized = serde_cbor::to_vec(&snapshot)?;
        let hash_hex = hash::sha256_hex(&serialized);

        if !ctx.backend.exists(ObjectType::Snapshot, &hash_hex)? {
            let (ciphertext, nonce) =
                aead::encrypt(&ctx.repo_key, &serialized, b"archivist-snapshot-v1");

            let mut stored = Vec::with_capacity(24 + ciphertext.len());
            stored.extend_from_slice(&nonce);
            stored.extend_from_slice(&ciphertext);

            ctx.backend.put(ObjectType::Snapshot, &hash_hex, &stored)?;
        }

        Self::write_latest(ctx, &hash_hex)?;
        Ok(hash_hex)
    }

    pub fn load(ctx: &RepoContext, hash: &str) -> Result<Self> {
        let stored = ctx.backend.get(ObjectType::Snapshot, hash)?;

        if stored.len() < 24 {
            anyhow::bail!("corrupted snapshot object {}", hash);
        }

        let (nonce, ciphertext) = stored.split_at(24);

        let plaintext = aead::decrypt(
            &ctx.repo_key,
            ciphertext,
            nonce.try_into().unwrap(),
            b"archivist-snapshot-v1",
        )
        .map_err(|e| anyhow::anyhow!("decrypting snapshot failed: {:?}", e))?;

        let computed = hash::sha256_hex(&plaintext);
        if computed != hash {
            anyhow::bail!("snapshot hash mismatch {}", hash);
        }

        Ok(serde_cbor::from_slice(&plaintext)?)
    }

    pub fn restore(ctx: &RepoContext, hash: &str, target: &Path) -> Result<()> {
        let snapshot = Self::load(ctx, hash)?;
        crate::repo::tree::Tree::restore(ctx, &snapshot.root_tree, target)?;
        Ok(())
    }

    pub fn read_latest(ctx: &RepoContext) -> Result<String> {
        let path = ctx.root.join("index/latest");
        Ok(fs::read_to_string(&path)
            .context("failed to read snapshot index")?
            .trim()
            .to_string())
    }

    fn write_latest(ctx: &RepoContext, hash: &str) -> Result<()> {
        let index = ctx.root.join("index");
        fs::create_dir_all(&index)?;

        let tmp = index.join("latest.tmp");
        let final_path = index.join("latest");

        fs::write(&tmp, format!("{}\n", hash))?;
        fs::rename(&tmp, &final_path)?;
        Ok(())
    }
}
