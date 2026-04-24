# SEO Indexer

[![CI](https://github.com/fauzanazz/seo-indexer/actions/workflows/ci.yml/badge.svg)](https://github.com/fauzanazz/seo-indexer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

A fast, white-hat SEO link indexer CLI built in Rust. Submit URLs to search engines using official APIs and ping services — supports IndexNow, Google Indexing API, and sitemap-based bulk submissions.

**Why use this?**
- **Fast** — Single binary, no runtime dependencies
- **White-hat** — Uses only official APIs and documented ping endpoints
- **Bulk-friendly** — Submit thousands of URLs from sitemaps or text files
- **Trackable** — SQLite-backed history with web dashboard

## Features

- Submit single URLs or bulk lists to IndexNow, Google Indexing API, and ping services
- Parse and submit all URLs from a sitemap (including sitemap index files)
- Bulk submit from a plain-text file (one URL per line)
- Submission history stored in a local SQLite database
- Web dashboard for browsing history
- Environment variable overrides for CI/CD pipelines
- `--verbose` and `--quiet` flags for scripting
- Cross-platform: Linux, macOS, Windows

## Installation

### From crates.io

```sh
cargo install indexer
```

### Download binary

Pre-built binaries are available on the [Releases](https://github.com/fauzanazz/seo-indexer/releases) page:

| Platform | File |
|----------|------|
| Linux x64 | `indexer-linux-x64` |
| macOS x64 | `indexer-macos-x64` |
| macOS ARM (M-series) | `indexer-macos-arm64` |
| Windows x64 | `indexer-windows-x64.exe` |

### Build from source

```sh
git clone https://github.com/fauzanazz/seo-indexer
cd seo-indexer
cargo build --release
# binary at: target/release/indexer
```

## Quick Start

```sh
# 1. Create a default config file
indexer config init

# 2. Edit the config to add your API keys
# ~/.config/indexer/config.toml  (Linux/macOS)
# %APPDATA%\legali\indexer\config.toml  (Windows)

# 3. Submit a URL
indexer submit https://example.com/new-page

# 4. Submit all URLs from a sitemap
indexer submit --sitemap https://example.com/sitemap.xml

# 5. Check submission history
indexer status
```

## Configuration

Run `indexer config init` to create a default config file, then edit it:

```toml
[indexnow]
api_key = "your-api-key-here"
# key_location = "https://yoursite.com/your-api-key.txt"

[google]
service_account_json = "~/.config/indexer/google-sa.json"

[ping]
services = ["google", "bing"]

[general]
default_method = "all"
rate_limit_per_minute = 60
```

See `config.example.toml` for the full reference with comments.

### Environment Variables

All sensitive values can be set via environment variables (useful for CI/CD):

| Variable | Description |
|----------|-------------|
| `INDEXER_INDEXNOW_API_KEY` | IndexNow API key |
| `INDEXER_INDEXNOW_KEY_LOCATION` | IndexNow key file URL |
| `INDEXER_DEFAULT_METHOD` | Default indexing method |
| `INDEXER_RATE_LIMIT_PER_MINUTE` | Rate limit override |

### Getting API Keys

**IndexNow**: Generate a key at [indexnow.org](https://www.indexnow.org/) and host it at `https://yoursite.com/<key>.txt`.

**Google Indexing API**: Create a service account in [Google Cloud Console](https://console.cloud.google.com/iam-admin/serviceaccounts), enable the Indexing API, download the JSON key, and grant the service account access as an Owner in Google Search Console.

## CLI Reference

### `indexer submit`

```
indexer submit [OPTIONS] [URL]

Arguments:
  [URL]  URL to submit

Options:
  --method <METHOD>    indexnow | google | ping | all  [default: all]
  --file <PATH>        Bulk submit from a file (one URL per line)
  --sitemap <URL>      Parse and submit all URLs from a sitemap
  -v, --verbose        Show extra detail (hints on failure, file paths, etc.)
  -q, --quiet          Suppress all non-error output (for scripting)
```

Examples:

```sh
# Submit a single URL
indexer submit https://example.com/post

# Submit via a specific method
indexer submit --method indexnow https://example.com/post

# Bulk submit from a file
indexer submit --file urls.txt

# Parse a sitemap and submit all discovered URLs
indexer submit --sitemap https://example.com/sitemap.xml

# Quiet mode for cron jobs
indexer submit -q --sitemap https://example.com/sitemap.xml
```

### `indexer config`

```
indexer config show    Print current configuration (with masked secrets)
indexer config init    Create a default config file at the platform config path
```

### `indexer status`

```
indexer status [OPTIONS]

Options:
  --url <URL>          Filter history by URL
  --export <FORMAT>    Export as json or csv
  --limit <N>          Max records to show  [default: 50]
```

Examples:

```sh
# Show recent submissions
indexer status

# Filter by URL
indexer status --url https://example.com/post

# Export as JSON
indexer status --export json > history.json

# Export as CSV
indexer status --export csv > history.csv
```

### `indexer dashboard`

Start the web dashboard to browse submission history in your browser:

```sh
indexer dashboard
# Opens on http://127.0.0.1:3000

indexer dashboard --port 8080 --host 0.0.0.0
```

### Global Flags

```
-v, --verbose   More detailed output
-q, --quiet     Suppress all output except errors
```

## How It Works

```
┌─────────────────────────────────────────────────────┐
│                    indexer CLI                       │
├─────────────────────────────────────────────────────┤
│  Commands: submit, status, dashboard, config        │
├─────────────────────────────────────────────────────┤
│                  Indexing Methods                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │  IndexNow   │ │   Google    │ │    Ping     │   │
│  │ Bing/Yandex │ │ Indexing API│ │Google/Bing  │   │
│  └─────────────┘ └─────────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────┤
│  SQLite Storage  │  TOML Config  │  Web Dashboard   │
└─────────────────────────────────────────────────────┘
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`cargo test`)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## License

MIT — see [LICENSE](LICENSE).
