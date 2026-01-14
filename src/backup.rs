use anyhow::{Context, Result};
use std::path::Path;

use crate::repo::{
    open::RepoContext,
    snapshot::Snapshot,
    tree::Tree,
};


pub fn run_backup(ctx: &RepoContext, source: &Path) -> Result<String> {
    if !source.exists() {
        anyhow::bail!("backup source does not exist");
    }

    if !source.is_dir() {
        anyhow::bail!("backup source must be a directory");
    }

    println!("Scanning filesystem...");
    let root_tree = Tree::from_dir(ctx, source)
        .context("failed to build filesystem tree")?;

    println!("Creating snapshot...");
    let snapshot_id = Snapshot::create(ctx, root_tree)
        .context("failed to create snapshot")?;

    Ok(snapshot_id)
}
