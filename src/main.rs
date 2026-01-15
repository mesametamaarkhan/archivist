use anyhow::Result;
use clap::Parser;
use archivist::{backup, cli::{Cli, Commands}, repo};

fn validate_repo_path(path: &std::path::Path) -> anyhow::Result<()> {
    if path.as_os_str() == "-" {
        anyhow::bail!("'-' is not a valid repository path; use '.' for current directory");
    }
    Ok(())
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { repo } => {
            validate_repo_path(&repo)?;
            archivist::repo::init::init_repository(&repo)?;
        }
        Commands::Open { repo } => {
            validate_repo_path(&repo)?;
            let _ctx = archivist::repo::open::open_repository(&repo)?;
            println!("Repository opened successfully");
        },
        Commands::Backup { path, repo } => {
            validate_repo_path(&repo)?;
            let ctx = repo::open::open_repository(&repo)?;
            let snapshot = backup::run_backup(&ctx, &path)?;
            println!("Backup complete.");
            println!("Snapshot ID: {}", snapshot);
        },
        Commands::Snapshots { repo } => {
            validate_repo_path(&repo)?;
            let ctx = repo::open::open_repository(&repo)?;
            backup::list_snapshots(&ctx)?;
        }
    }

    Ok(())
}
