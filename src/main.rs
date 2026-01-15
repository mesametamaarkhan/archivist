use anyhow::Result;
use clap::Parser;
use tracing::{info, error};

use archivist::{
    backup,
    cli::{Cli, Commands},
    logging,
    repo,
};

fn validate_repo_path(path: &std::path::Path) -> anyhow::Result<()> {
    if path.as_os_str() == "-" {
        anyhow::bail!("'-' is not a valid repository path; use '.' for current directory");
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    logging::init(cli.verbose, cli.quiet);

    match cli.command {
        Commands::Init { repo } => {
            validate_repo_path(&repo)?;
            repo::init::init_repository(&repo)?;
            info!("Repository initialized");
        }

        Commands::Open { repo } => {
            validate_repo_path(&repo)?;
            repo::open::open_repository(&repo, false)?;
            info!("Repository opened successfully");
        }

        Commands::Backup { path, repo } => {
            validate_repo_path(&repo)?;
            let ctx = repo::open::open_repository(&repo, true)?;
            info!("Scanning filesystem...");
            let snapshot = backup::run_backup(&ctx, &path)?;
            info!("Backup complete");
            info!("Snapshot ID: {}", snapshot);
        }

        Commands::Snapshots { repo } => {
            validate_repo_path(&repo)?;
            let ctx = repo::open::open_repository(&repo, false)?;
            backup::list_snapshots(&ctx)?;
        }

        Commands::Restore { snapshot, repo, target } => {
            validate_repo_path(&repo)?;
            let ctx = repo::open::open_repository(&repo, false)?;
            info!("Loading snapshot {}", snapshot);
            backup::run_restore(&ctx, &snapshot, &target)?;
            info!("Restore complete");
        }

        Commands::Check { repo } => {
            validate_repo_path(&repo)?;
            let ctx = repo::open::open_repository(&repo, false)?;
            info!("Checking repository integrity...");
            backup::run_check(&ctx)?;
            info!("Check complete");
        }
    }

    Ok(())
}
