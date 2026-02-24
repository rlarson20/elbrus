use clap::{Parser, Subcommand};
use elbrus_db::sqlite::SqliteBackend;
use elbrus_scryfall::bulk::BulkIngestor;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest a Scryfall bulk JSON file into the database
    Ingest {
        /// Path to the Scryfall `default-cards` JSON file
        path: PathBuf,

        /// Path to the SQLite database
        #[arg(short, long, default_value = "elbrus.db")]
        db: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest { path, db } => {
            let db_path = db
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid DB path"))?;
            let conn_str = format!("sqlite://{db_path}");
            let backend = SqliteBackend::open(&conn_str).await?;

            let ingestor = BulkIngestor::default();

            println!("Ingesting from {}...", path.display());
            let stats = ingestor.ingest_into_db(&path, &backend).await?;

            println!("Ingest completed in {:?}", stats.duration);
            println!("Cards processed: {}", stats.cards_processed);
            println!("Cards inserted: {}", stats.cards_inserted);
            println!("Cards updated: {}", stats.cards_updated);
        }
    }

    Ok(())
}
