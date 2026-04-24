mod bulk;
mod cli;
mod config;
mod error;
mod indexers;
mod parser;
mod storage;
mod web;

use std::path::PathBuf;

use clap::Parser;
use cli::commands::Cli;
use directories::ProjectDirs;
use storage::Storage;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let storage = open_storage()?;
    cli::run(cli, storage).await
}

fn open_storage() -> anyhow::Result<Storage> {
    let cfg = config::load()?;

    let db_path = if let Some(custom) = cfg.storage.path {
        PathBuf::from(custom)
    } else {
        default_db_path()?
    };

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    Storage::new(&db_path).map_err(|e| anyhow::anyhow!("Failed to open storage: {e}"))
}

fn default_db_path() -> anyhow::Result<PathBuf> {
    let dirs = ProjectDirs::from("ai", "legali", "indexer")
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
    Ok(dirs.data_local_dir().join("submissions.db"))
}
