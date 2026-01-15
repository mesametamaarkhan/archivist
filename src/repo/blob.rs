use anyhow::{Context, Ok, Result};
use std::{fs};
use std::path::Path;
use crate::crypto::{aead, hash};
use crate::repo::backend::Backend;
use crate::repo::{
    backend::ObjectType,
    open::RepoContext,
};

pub struct Blob {
    pub hash: String,
    pub size: u64,
}

impl Blob {
    pub fn from_file(ctx: &RepoContext, path: &Path) -> Result<Self> {
        let data = fs::read(path)
            .with_context(|| format!("failed to read file {}", path.display()))?;

        let hash = hash::sha256_hex(&data);

        //deduplication check
        if !ctx.backend.exists(ObjectType::Blob, &hash)? {
            let (ciphertext, nonce) = aead::encrypt(
                &ctx.repo_key, 
                &data, 
                b"archivist-blob-v1"
            );

            // store nonce + ciphertext together
            let mut stored = Vec::with_capacity(24 + ciphertext.len());
            stored.extend_from_slice(&nonce);
            stored.extend_from_slice(&ciphertext);

            ctx.backend.put(ObjectType::Blob, &hash, &stored)?;
        }

        Ok(Self {
            hash,
            size: data.len() as u64
        })
    }

    pub fn restore(ctx: &RepoContext, hash: &str, target: &Path) -> Result<()> {
        let stored = ctx.backend.get(ObjectType::Blob, hash)?;

        if stored.len() < 24 {
            anyhow::bail!("corrupted blob {}", hash);
        }

        let (nonce, ciphertext) = stored.split_at(24);

        let plaintext = aead::decrypt(
            &ctx.repo_key, 
            ciphertext, 
            nonce.try_into().unwrap(), 
            b"archivist-blob-v1"
        )
        .map_err(|e| anyhow::anyhow!("blob decryption failed: {:?}", e))?;

        // verify integrity explicitly
        let computed = hash::sha256_hex(&plaintext);
        if computed != hash {
            anyhow::bail!("hash mismatch for blob {}", hash);
        }

        fs::write(target, plaintext)
            .with_context(|| format!("failed to write {}", target.display()));

        Ok(())
    }

    pub fn load(ctx: &RepoContext, hash: &str) -> Result<Vec<u8>> {
        let stored = ctx.backend.get(ObjectType::Blob, hash)?;
        if stored.len() < 24 {
            anyhow::bail!("corrupted blob {}", hash);
        }

        let (nonce, ciphertext) = stored.split_at(24);

        let plaintext = aead::decrypt(
            &ctx.repo_key,
            ciphertext,
            nonce.try_into().unwrap(),
            b"archivist-blob-v1",
        )
        .map_err(|e| anyhow::anyhow!("blob decryption failed: {:?}", e))?;

        let computed = hash::sha256_hex(&plaintext);
        if computed != hash {
            anyhow::bail!("blob hash mismatch {}", hash);
        }

        Ok(plaintext)
    }
}