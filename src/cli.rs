use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "archivist")]
#[command(about = "Encrypted, append-only backup system")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        repo: PathBuf,
    },
    Open {
        repo: PathBuf,
    },
    Backup {
        path: PathBuf,
        repo: PathBuf,
    },
    Snapshots {
        repo: PathBuf,
    }
}