pub mod commands;

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use url::Url;

use crate::bulk;
use crate::config;
use crate::indexers;
use crate::parser::SitemapParser;
use crate::storage::{Storage, SubmissionRecord};
use crate::web;
use commands::{Cli, Commands, ConfigAction};

pub async fn run(cli: Cli, storage: Storage) -> anyhow::Result<()> {
    let verbose = cli.verbose;
    let quiet = cli.quiet;

    match cli.command {
        Commands::Submit {
            url,
            method,
            file,
            sitemap,
        } => handle_submit(url, method, file, sitemap, storage, verbose, quiet).await,
        Commands::Config { action } => handle_config(action, quiet),
        Commands::Status { url, export, limit } => handle_status(url, export, limit, storage),
        Commands::Dashboard { host, port } => {
            let cfg = config::load()?;
            if !quiet {
                println!("Starting dashboard on http://{host}:{port}");
            }
            let state = Arc::new(web::AppState {
                storage: Arc::new(RwLock::new(storage)),
                config: Arc::new(cfg),
            });
            web::serve(&host, port, state).await
        }
    }
}

async fn handle_submit(
    raw_url: String,
    method: String,
    file: Option<String>,
    sitemap: Option<String>,
    storage: Storage,
    verbose: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    let valid_methods = ["indexnow", "google", "ping", "all"];
    if !valid_methods.contains(&method.as_str()) {
        anyhow::bail!(
            "Unknown method '{}'. Valid options are: {}",
            method,
            valid_methods.join(", ")
        );
    }

    let cfg = config::load()?;
    let indexer_list = indexers::get_indexers(&cfg, &method);

    if indexer_list.is_empty() {
        if !quiet {
            eprintln!(
                "No indexers configured for method '{}'. Run `indexer config show` to check your configuration.",
                method
            );
        }
        return Ok(());
    }

    let urls = if let Some(file_path) = file {
        if verbose && !quiet {
            println!("Reading URLs from file: {file_path}");
        }
        let path = Path::new(&file_path);
        bulk::read_urls_from_file(path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read '{}': {}", file_path, e))?
    } else if let Some(sitemap_url_str) = sitemap {
        let sitemap_url = Url::parse(&sitemap_url_str).map_err(|_| {
            anyhow::anyhow!(
                "Invalid sitemap URL: '{}'. Make sure it starts with http:// or https://",
                sitemap_url_str
            )
        })?;
        if !quiet {
            println!("Fetching sitemap: {sitemap_url}");
        }
        SitemapParser::fetch_and_parse(&sitemap_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse sitemap '{}': {}", sitemap_url, e))?
    } else if !raw_url.is_empty() {
        let url = Url::parse(&raw_url).map_err(|_| {
            anyhow::anyhow!(
                "Invalid URL: '{}'. Make sure it starts with http:// or https://",
                raw_url
            )
        })?;
        vec![url]
    } else {
        anyhow::bail!("No URL provided. Use a URL argument, --file <path>, or --sitemap <url>");
    };

    if urls.is_empty() {
        if !quiet {
            println!("No URLs found to submit.");
        }
        return Ok(());
    }

    let total = urls.len();
    if total > 1 && !quiet {
        println!(
            "Submitting {total} URL(s) via {} indexer(s)...",
            indexer_list.len()
        );
    }

    for (i, url) in urls.iter().enumerate() {
        if total > 1 && !quiet {
            println!("[{}/{}] {url}", i + 1, total);
        }
        for indexer in &indexer_list {
            let (success, message) = match indexer.submit(url).await {
                Ok(result) => {
                    if !quiet {
                        println!(
                            "  [{}] {} — {}",
                            result.method,
                            if result.success { "OK" } else { "PARTIAL" },
                            result.message
                        );
                    }
                    (result.success, result.message.clone())
                }
                Err(e) => {
                    let msg = e.to_string();
                    if !quiet {
                        eprintln!("  [{}] FAIL — {msg}", indexer.name());
                        if verbose {
                            eprintln!(
                                "  Hint: check your API credentials with `indexer config show`"
                            );
                        }
                    }
                    (false, msg)
                }
            };

            let record = SubmissionRecord {
                id: None,
                url: url.to_string(),
                method: indexer.name().to_string(),
                success,
                message: Some(message),
                submitted_at: Utc::now(),
            };
            if let Err(e) = storage.insert(&record) {
                if !quiet {
                    eprintln!("Warning: failed to save submission record: {e}");
                }
            }
        }
    }

    Ok(())
}

fn handle_config(action: ConfigAction, quiet: bool) -> anyhow::Result<()> {
    match action {
        ConfigAction::Show => {
            match config::load() {
                Ok(cfg) => {
                    if quiet {
                        return Ok(());
                    }
                    println!("IndexNow:");
                    println!("  api_key:      {}", cfg.indexnow.api_key.as_deref().unwrap_or("<not set>"));
                    println!("  key_location: {}", cfg.indexnow.key_location.as_deref().unwrap_or("<not set>"));
                    println!("Google:");
                    println!("  service_account_json: {}", cfg.google.service_account_json.as_deref().unwrap_or("<not set>"));
                    println!("Ping:");
                    let services = if cfg.ping.services.is_empty() {
                        "google, bing (default)".to_string()
                    } else {
                        cfg.ping.services.join(", ")
                    };
                    println!("  services: {services}");
                    println!("General:");
                    println!("  default_method:        {}", cfg.general.default_method);
                    println!("  rate_limit_per_minute: {}", cfg.general.rate_limit_per_minute);
                    println!("Storage:");
                    println!("  path: {}", cfg.storage.path.as_deref().unwrap_or("<default>"));
                }
                Err(e) => eprintln!("Could not load configuration: {e}\nRun `indexer config init` to create a default config."),
            }
        }
        ConfigAction::Init => {
            let path = config::config_path()
                .map_err(|e| anyhow::anyhow!("Could not determine config directory: {e}"))?;
            if path.exists() {
                if !quiet {
                    println!("Config already exists at: {}", path.display());
                    println!("Edit it directly or delete it to re-initialize.");
                }
            } else {
                config::write_default(&path)?;
                if !quiet {
                    println!("Created default config at: {}", path.display());
                    println!("Edit the file to add your API keys.");
                }
            }
        }
    }
    Ok(())
}

fn handle_status(
    url_filter: Option<String>,
    export: Option<String>,
    limit: usize,
    storage: Storage,
) -> anyhow::Result<()> {
    let records = if let Some(ref url) = url_filter {
        storage
            .get_by_url(url)
            .map_err(|e| anyhow::anyhow!("Failed to query storage: {e}"))?
    } else {
        storage
            .get_history(limit)
            .map_err(|e| anyhow::anyhow!("Failed to query storage: {e}"))?
    };

    if records.is_empty() {
        println!("No submission history found.");
        return Ok(());
    }

    match export.as_deref() {
        Some("json") => {
            println!("{}", serde_json::to_string_pretty(&records)?);
        }
        Some("csv") => {
            println!("id,url,method,success,message,submitted_at");
            for r in &records {
                println!(
                    "{},{},{},{},{},{}",
                    r.id.unwrap_or(0),
                    r.url,
                    r.method,
                    r.success,
                    r.message.as_deref().unwrap_or(""),
                    r.submitted_at.to_rfc3339(),
                );
            }
        }
        Some(fmt) => {
            anyhow::bail!(
                "Unknown export format '{}'. Valid options are: json, csv",
                fmt
            );
        }
        None => {
            println!(
                "{:<6} {:<50} {:<10} {:<8} SUBMITTED AT",
                "ID", "URL", "METHOD", "STATUS"
            );
            println!("{}", "-".repeat(100));
            for r in &records {
                println!(
                    "{:<6} {:<50} {:<10} {:<8} {}",
                    r.id.unwrap_or(0),
                    truncate(&r.url, 50),
                    r.method,
                    if r.success { "OK" } else { "FAIL" },
                    r.submitted_at.format("%Y-%m-%d %H:%M:%S UTC"),
                );
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
