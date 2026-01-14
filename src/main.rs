use anyhow::Result;
use clap::Parser;
use archivist::cli::{Cli, Commands};

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
        }
    }

    Ok(())
}