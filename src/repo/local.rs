use anyhow::{Context, Ok, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use super::backend::{Backend, ObjectType};

pub struct LocalBackend {
    root: PathBuf,
}

impl LocalBackend {
    pub fn new(repo_root: &Path) -> Self {
        Self {
            root: repo_root.join("objects"),
        }
    }

    fn object_path(&self, obj_type: ObjectType, hash: &str) -> PathBuf {
        let (a, b) = hash.split_at(2);

        let dir = match obj_type {
            ObjectType::Blob => "blobs",
            ObjectType::Tree => "trees",
            ObjectType::Snapshot => "snapshots",
        };

        self.root.join(dir).join(a).join(b).join(hash)
    }

    fn ensure_parent_dir(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

impl Backend for LocalBackend {
    fn put(&self, obj_type: ObjectType, hash: &str, data: &[u8]) -> Result<()> {
        let final_path = self.object_path(obj_type, hash);

        if final_path.exists() {
            return Ok(());
        }

        Self::ensure_parent_dir(&final_path);

        let tmp_path = final_path.with_extension("tmp");

        // write temp file
        let mut file = File::create(&tmp_path).context("failed to create temp object file")?;
        file.write_all(data)?;
        file.sync_all()?;

        // atomic rename
        fs::rename(&tmp_path, &final_path)
            .context("atomic rename failed")?;

        //fsync parent directory
        if let Some(parent) = final_path.parent() {
            let dir = File::open(parent)?;
            dir.sync_all();
        }

        Ok(())
    }

    fn get(&self, obj_type: ObjectType, hash: &str) -> Result<Vec<u8>> {
        let path = self.object_path(obj_type, hash);

        let data = fs::read(&path)
            .with_context(|| format!("object not found: {}", hash))?;

        Ok(data)
    }

    fn exists(&self, obj_type: ObjectType, hash: &str) -> Result<bool> {
        let path = self.object_path(obj_type, hash);
        Ok(path.exists())
    }
}