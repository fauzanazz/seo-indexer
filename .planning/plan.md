# SEO Link Indexer - Implementation Plan

## Architecture Decision

**Approach:** Monolithic single-crate  
**Rationale:** Simpler MVP, can refactor to workspace later if needed

## Project Structure

```
seo-indexer/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs      # CLI command definitions
│   │   └── output.rs        # Formatting output
│   ├── config/
│   │   ├── mod.rs
│   │   └── schema.rs        # Config structs
│   ├── indexers/
│   │   ├── mod.rs           # Indexer trait
│   │   ├── indexnow.rs
│   │   ├── google.rs
│   │   └── ping.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   └── models.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   └── sitemap.rs
│   └── web/
│       ├── mod.rs
│       ├── routes.rs
│       └── templates/
├── templates/               # htmx templates
├── tests/
└── config.example.toml
```

## Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
rusqlite = { version = "0.31", features = ["bundled"] }
axum = "0.7"
askama = "0.12"                    # htmx templating
askama_axum = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
thiserror = "1"
url = "2"
quick-xml = "0.31"                 # sitemap parsing
directories = "5"                  # XDG paths for config
chrono = { version = "0.4", features = ["serde"] }
```

---

## Session Milestones

### Session 1: Foundation + IndexNow

**Goal:** Working CLI that can submit a single URL via IndexNow

**Tasks:**
1. [ ] Initialize Cargo project with dependencies
2. [ ] Set up CLI skeleton with clap (submit, config, status commands)
3. [ ] Implement config system (load TOML, env var overrides)
4. [ ] Define `Indexer` trait
5. [ ] Implement IndexNow client
6. [ ] Add basic error handling with thiserror
7. [ ] Write tests for IndexNow client (mock HTTP)

**Tests (TDD anchor for Session 2):**
- `test_indexnow_submit_single_url`
- `test_config_load_from_file`
- `test_config_env_override`

**Exit criteria:** `indexer submit https://example.com --method indexnow` works

---

### Session 2: Google API + Ping Services

**Goal:** All three indexing methods working

**Tasks:**
1. [ ] Implement Google Indexing API client
2. [ ] Handle Google service account auth (JWT)
3. [ ] Implement Ping services client
4. [ ] Unified submission across all methods
5. [ ] Add --method flag (indexnow, google, ping, all)

**Tests:**
- `test_google_indexing_submit`
- `test_ping_services_submit`
- `test_submit_all_methods`

**Exit criteria:** `indexer submit URL --method all` submits to all services

---

### Session 3: Storage + Bulk Operations

**Goal:** Persistent history, bulk URL support

**Tasks:**
1. [ ] Set up SQLite schema (submissions table)
2. [ ] Implement storage layer (insert, query history)
3. [ ] Bulk URL processing from file
4. [ ] Sitemap XML parser
5. [ ] Add `status` command to view history
6. [ ] Rate limiting / backoff logic

**Tests:**
- `test_storage_insert_submission`
- `test_storage_query_history`
- `test_bulk_url_from_file`
- `test_sitemap_parser`

**Exit criteria:** 
- `indexer submit --file urls.txt` works
- `indexer submit --sitemap https://example.com/sitemap.xml` works
- `indexer status` shows history

---

### Session 4: Web Dashboard

**Goal:** Embedded web dashboard with htmx

**Tasks:**
1. [ ] Set up Axum server
2. [ ] Create askama templates for dashboard
3. [ ] Dashboard home (recent submissions, stats)
4. [ ] Submission history view with filtering
5. [ ] Manual submit form
6. [ ] Add `dashboard` command to start server

**Tests:**
- `test_dashboard_routes`
- `test_dashboard_renders`

**Exit criteria:** `indexer dashboard` opens http://localhost:3000 with working UI

---

### Session 5: Polish & Release

**Goal:** Production-ready open source release

**Tasks:**
1. [ ] Improve error messages
2. [ ] Add --verbose and --quiet flags
3. [ ] Write README.md
4. [ ] Add CONTRIBUTING.md
5. [ ] Set up GitHub Actions CI
6. [ ] Cross-compile binaries (Linux, macOS, Windows)
7. [ ] Create GitHub release workflow

**Exit criteria:** GitHub repo ready with binaries and docs

---

## Config File Format

```toml
# ~/.config/indexer/config.toml

[indexnow]
api_key = "your-indexnow-key"
key_location = "https://yoursite.com/indexnow-key.txt"

[google]
service_account_path = "~/.config/indexer/google-sa.json"

[ping]
services = ["pingomatic", "google", "bing"]

[general]
default_method = "all"
rate_limit_per_minute = 100

[dashboard]
port = 3000
host = "127.0.0.1"
```

## CLI Commands Overview

```bash
# Submit URLs
indexer submit <URL>                           # Submit single URL
indexer submit --file urls.txt                 # Bulk from file  
indexer submit --sitemap https://site/sitemap.xml  # From sitemap
indexer submit <URL> --method indexnow         # Specific method

# Status & History
indexer status                                 # Recent submissions
indexer status --url <URL>                     # Status for specific URL
indexer status --export csv                    # Export history

# Config
indexer config show                            # Show current config
indexer config init                            # Create default config

# Dashboard
indexer dashboard                              # Start web dashboard
indexer dashboard --port 8080                  # Custom port
```
