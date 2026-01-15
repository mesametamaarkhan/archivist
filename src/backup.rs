use anyhow::{Context, Result};
use std::{collections::HashSet, path::Path};

use crate::repo::{
    open::RepoContext,
    snapshot::{Snapshot},
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

pub fn list_snapshots(ctx: &RepoContext) -> Result<()> {
    let mut current = match Snapshot::read_latest(ctx) {
        Ok(h) => h,
        Err(_) => {
            println!("no snapshots found");
            return Ok(());
        }
    };

    println!("Snapshots (newest -> oldest): \n");

    loop {
        let snap = Snapshot::load(ctx, &current)?;
        println!("{} {} {}", current, snap.timestamp, snap.hostname);
        match snap.parent {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Ok(())
}

pub fn run_restore(ctx: &RepoContext, snapshot_ref: &str, target: &Path) -> Result<()> {
    if target.exists() {
        anyhow::bail!("restore target already exists");
    }

    let snapshot_id = if snapshot_ref == "latest" {
        Snapshot::read_latest(ctx)?
    } else {
        snapshot_ref.to_string()
    };

    println!("Loading snapshot {}", snapshot_id);
    let snapshot = Snapshot::load(ctx, &snapshot_id)?;

    println!("Restoring filesystem...");
    Tree::restore(ctx, &snapshot.root_tree, target)
        .context("restore failed")?;

    Ok(())
}

pub fn run_check(ctx: &RepoContext) -> Result<()> {
    let mut checked_trees = HashSet::new();
    let mut checked_blobs = HashSet::new();

    let mut current = match Snapshot::read_latest(ctx) {
        Ok(h) => h,
        Err(_) => {
            println!("No snapshots found.");
            return Ok(());
        }
    };

    println!("Checking snapshots...\n");

    loop {
        println!("Snapshot {}", current);
        let snap = Snapshot::load(ctx, &current)
            .context("snapshot corrupted")?;

        Tree::check(
            ctx, 
            &snap.root_tree,
            &mut checked_trees,
            &mut checked_blobs,
        )
        .context("tree verification failed")?;

        match snap.parent {
            Some(parent) => current = parent,
            None => break,
        }
    }

    println!("\nCheck complete.");
    println!("Verified:");
    println!("  Trees: {}", checked_trees.len());
    println!("  Blobs: {}", checked_blobs.len());

    Ok(())
}