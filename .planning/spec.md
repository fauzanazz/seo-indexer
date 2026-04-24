# SEO Link Indexer CLI - Specification

## Overview

**Project Name:** `indexer` (working title)  
**Type:** Open source CLI tool  
**Purpose:** White-hat SEO link indexing tool supporting multiple search engine APIs

## Target Users

Open source community - SEO practitioners, developers, and agencies who need a reliable, ethical way to request search engine indexing for their URLs.

## Core Features

### 1. Multi-Method Indexing Support

| Method | Target Engines | Requirements |
|--------|---------------|--------------|
| IndexNow API | Bing, Yandex, Seznam | API key (free) |
| Google Indexing API | Google | Google Cloud service account |
| Ping Services | Multiple aggregators | None |

### 2. URL Input Methods

- **Single URL:** `indexer submit https://example.com/page`
- **Bulk from file:** `indexer submit --file urls.txt`
- **Sitemap parsing:** `indexer submit --sitemap https://example.com/sitemap.xml`

### 3. Status Tracking & Reporting

- Track submission status per URL per method
- Generate reports (JSON, CSV, human-readable)
- Historical data stored in local SQLite database

### 4. Web Dashboard

- Embedded HTML with htmx (no separate frontend build)
- View submission history
- Monitor success/failure rates
- Single binary distribution (dashboard embedded)

## Technical Constraints

| Aspect | Decision |
|--------|----------|
| Language | Rust |
| Database | SQLite (embedded) |
| Web Framework | Axum + htmx |
| Distribution | Single binary |
| Config | TOML file + env vars |

## Out of Scope (v1)

- Multi-user authentication (single-user, API keys in config)
- Cloud/hosted version
- Browser extension
- Paid API integrations (SEMrush, Ahrefs, etc.)

## Success Criteria

1. User can submit single URL to all supported indexing methods
2. User can bulk submit from file or sitemap
3. All submissions are tracked in local SQLite database
4. User can view submission history via web dashboard
5. Single binary works on Linux, macOS, Windows
6. Well-documented with examples

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                    indexer CLI                       │
├─────────────────────────────────────────────────────┤
│  Commands: submit, status, dashboard, config        │
├─────────────────────────────────────────────────────┤
│                  Core Engine                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │  IndexNow   │ │   Google    │ │    Ping     │   │
│  │   Client    │ │   Client    │ │   Client    │   │
│  └─────────────┘ └─────────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────┤
│  Storage: SQLite │ Config: TOML │ Web: Axum+htmx   │
└─────────────────────────────────────────────────────┘
```

## Estimated Scope

This is a **multi-session project**. Suggested milestones:

1. **Session 1:** Project setup, CLI skeleton, IndexNow client
2. **Session 2:** Google Indexing API client, Ping services
3. **Session 3:** SQLite storage, bulk/sitemap support
4. **Session 4:** Web dashboard with htmx
5. **Session 5:** Polish, testing, documentation
