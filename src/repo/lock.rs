use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct RepoLock {
    path: PathBuf,
}

impl RepoLock {
    pub fn acquire(repo: &Path) -> Result<Self> {
        let path = repo.join(".lock");

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .with_context(|| "repository is already locked (another writer?)")?;

        writeln!(
            file,
            "pid={}\nhost={}\n",
            std::process::id(),
            hostname::get()?.to_string_lossy()
        )?;

        file.sync_all()?;

        Ok(Self { path })
    }
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
