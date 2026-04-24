use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "indexer", version, about = "White-hat SEO link indexer CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Submit URL(s) for indexing
    Submit {
        /// URL to submit (optional when --file or --sitemap is used)
        #[arg(default_value = "")]
        url: String,
        /// Indexing method: indexnow, google, ping, all
        #[arg(long, default_value = "all")]
        method: String,
        /// Bulk submit URLs from a file (one per line)
        #[arg(long, value_name = "PATH")]
        file: Option<String>,
        /// Parse and submit URLs from a sitemap
        #[arg(long, value_name = "URL")]
        sitemap: Option<String>,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Show submission history
    Status {
        /// Filter history by URL
        #[arg(long, value_name = "URL")]
        url: Option<String>,
        /// Export format: json or csv
        #[arg(long, value_name = "FORMAT")]
        export: Option<String>,
        /// Maximum number of records to show
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Start the web dashboard
    Dashboard {
        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Create default config file
    Init,
}
