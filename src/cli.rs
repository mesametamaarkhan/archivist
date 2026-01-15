use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Verbose output (debug-level)
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Suppress all non-error output
    #[arg(long, global = true)]
    pub quiet: bool,

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
    },
    Restore {
        snapshot: String,
        repo: PathBuf,
        target: PathBuf,
    },
    Check {
        repo: PathBuf,
    },
}