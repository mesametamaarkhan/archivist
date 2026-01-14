use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use crate::crypto::{aead, hash};
use crate::repo::backend::Backend;
use crate::repo::{
    backend::ObjectType,
    blob::Blob,
    open::RepoContext,
};

#[derive(Serialize, Deserialize)]
pub enum TreeEntry {
    File {
        blob: String,
        size: u64,
        mode: u32,
        mtime: i64,
    },
    Dir {
        tree: String,
        mode: u32,
        mtime: i64,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Tree {
    entries: BTreeMap<String, TreeEntry>
}

impl Tree {
    pub fn from_dir(ctx: &RepoContext, path: &Path) -> Result<String> {
        let mut entries = BTreeMap::new();

        let read_dir =  fs::read_dir(path)
            .with_context(|| format!("failed to read dir {}", path.display()))?;

        for entry in read_dir {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let entry_path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                let blob = Blob::from_file(ctx, &entry_path)?;

                entries.insert(
                    file_name, 
                    TreeEntry::File { 
                        blob: blob.hash, 
                        size: metadata.size(), 
                        mode: metadata.mode(), 
                        mtime: metadata.mtime() 
                    },
                );
            } else if metadata.is_dir() {
                let tree_hash = Tree::from_dir(ctx, &entry_path)?;

                entries.insert(
                    file_name,
                    TreeEntry::Dir { 
                        tree: tree_hash, 
                        mode: metadata.mode(), 
                        mtime: metadata.mtime(), 
                    }
                );
            }
        }

        let tree = Tree { entries };

        // serialize deterministically
        let serialized = serde_cbor::to_vec(&tree)?;

        // hash serialized tree
        let tree_hash = hash::sha256_hex(&serialized);

        // deduplication
        if !ctx.backend.exists(ObjectType::Tree, &tree_hash)? {
            let (ciphertext, nonce) = aead::encrypt(
                &ctx.repo_key, 
                &serialized, 
                b"archivist-tree-v1",
            );

            let mut stored = Vec::with_capacity(24 + ciphertext.len());
            stored.extend_from_slice(&nonce);
            stored.extend_from_slice(&ciphertext);

            ctx.backend.put(ObjectType::Tree, &tree_hash, &stored)?;
        }

        Ok(tree_hash)
    }

    pub fn restore(ctx: &RepoContext, tree_hash: &str, target: &Path) -> Result<()> {
        let stored = ctx.backend.get(ObjectType::Tree, &tree_hash)?;
        if stored.len() < 24 {
            anyhow::bail!("corrupted tree {}", tree_hash);
        }

        let (nonce, ciphertext) = stored.split_at(24);
        
        let plaintext = aead::decrypt(
            &ctx.repo_key, 
            ciphertext, 
            nonce.try_into().unwrap(), 
            b"archivist-tree-v1",
        )
        .map_err(|e| anyhow::anyhow!("tree decryption failed: {:?}", e))?;

        let computed = hash::sha256_hex(&plaintext);
        if computed != tree_hash {
            anyhow::bail!("tree hash mismatch {}", tree_hash);
        }

        let tree: Tree = serde_cbor::from_slice(&plaintext)?;

        fs::create_dir_all(target)?;

        for (name, entry) in tree.entries {
            let path = target.join(name);

            match entry {
                TreeEntry::File { blob, size, mode, mtime } => {
                    Blob::restore(ctx, &blob, &path)?;
                    fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;

                    let ts = filetime::FileTime::from_unix_time(mtime, 0);
                    filetime::set_file_mtime(&path, ts)?;
                },
                TreeEntry::Dir { tree, mode, mtime } => {
                    Tree::restore(ctx, &tree, &path)?;
                    fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;

                    let ts = filetime::FileTime::from_unix_time(mtime, 0);
                    filetime::set_file_mtime(&path, ts);
                }
            }
        }

        Ok(())
    }
}