use anyhow::Result;
use clap::Parser;
use archivist::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { repo } => {
            archivist::repo::init::init_repository(&repo)?;
        }
    }

    Ok(())
}
