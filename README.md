# Agent Insights

<p align="center">
  <img src="https://img.shields.io/github/stars/milisp/agent-insights?style=flat-square" />
  <img src="https://img.shields.io/github/license/milisp/agent-insights?style=flat-square" />
  <img src="https://img.shields.io/github/release/milisp/agent-insights?style=flat-square" />
  <img src="https://img.shields.io/badge/Rust-Axum-orange?style=flat-square" />
</p>

A full-stack application for tracking and visualizing AI agent usage across Claude, Gemini, and Codex with GitHub-style heatmap charts.

## Overview

Agent Insights scans your local agent directories and provides beautiful visualizations of your usage patterns over time. See when you're most active with each AI agent, track file generation, and monitor storage usage.

## Privacy & Data Safety

Agent Insights is **fully local-first**:

- All data is scanned from your local filesystem
- No network requests, telemetry, or analytics
- No data is uploaded or shared
- You can safely run it completely offline

Your agent usage data never leaves your machine.

![demo](images/agent-insights.png)

## Features

- **Multi-Agent Support**: Claude, Gemini, and Codex
- **GitHub-Style Heatmaps**: Year-long activity visualization per agent
- **Local-First & Offline**: No internet connection required
- **Fast Local Scanning**: Efficient filesystem traversal in Rust
- **Usage Analytics**: File count, activity frequency, and storage usage
- **Modern UI**: React + Tailwind CSS v4 + shadcn/ui
- **High-Performance Backend**: Rust + Axum

## Quick Start

### Download & Run (Prebuilt)

Download the latest binary from GitHub Releases:

https://github.com/milisp/agent-insights/releases

```bash
chmod +x ./agent-insights
./agent-insights
```

The server starts at:

- Backend API: http://127.0.0.1:3001

Open your browser and visit: http://127.0.0.1:3001

## Build from source

```sh
git clone https://github.com/milisp/agent-insights
cd agent-insights
```

### Prerequisites
- Rust 1.91 or newer
- Bun (for frontend)

### Backend

```bash
cd apps/backend
cargo run
```

Server starts on http://127.0.0.1:3001

### Frontend

```bash
cd apps/frontend
bun install
bun dev
```

UI opens at http://localhost:5173

## How It Works

1. Scans known local directories for supported AI agents
2. Parses JSON / JSONL session and file metadata
3. Aggregates activity by date and agent
4. Exposes a local HTTP API
5. Renders GitHub-style heatmaps in the frontend

## API Endpoints

### GET /api/heatmaps
Returns all agent heatmaps

**Response:**
```json
{
  "Claude": {
    "agent": "Claude",
    "data": [
      {"date": "2025-12-02", "count": 20, "size": 5096527}
    ],
    "max_count": 82,
    "total_files": 666,
    "total_size": 123456789
  },
  "Gemini": {...},
  "Codex": {...}
}
```

### GET /api/heatmap/:agent
Returns heatmap for specific agent (claude, gemini, codex)

### Example (curl)

```bash
curl http://127.0.0.1:3001/api/heatmaps
curl http://127.0.0.1:3001/api/heatmap/claude
```

## Agent Detection

### Claude
- Location: `~/.claude/projects/`
- Format: JSONL (newline-delimited JSON)
- Features: Token usage tracking, session IDs

### Gemini
- Location: `~/.gemini/tmp/{hash}/chats/`
- Format: JSON
- Features: Aggregated sessions, chat history

### Codex
- Location: `~/.codex/sessions/`
- Format: JSON/JSONL
- Features: File metadata

## Use Cases

- Understand when and how often you use different AI agents
- Track long-term usage trends over months
- Compare activity across Claude, Gemini, and Codex
- Audit local agent-generated files and storage growth

## Performance

Typical scan results:
- **666 Claude files** (20 days)
- **558 Gemini files** (65 days)
- **493 Codex files** (68 days)
- **Total: 1,717 files** scanned in ~1.3 seconds

## Development

[ARCHITECTURE](docs/ARCHITECTURE.md)

Use `just` to start both backend and frontend, or run them separately:

### Backend Development
```bash
cd apps/backend
cargo watch -x run
```

### Frontend Development
```bash
cd apps/frontend
bun dev
```

### Build for Production
```bash
# Backend
cd apps/backend
cargo build --release

# Frontend
cd apps/frontend
bun run build
```

## Contributing

Contributions are welcome!

- Bug reports and feature requests via GitHub Issues
- Pull requests for fixes, improvements, or new agent support
- Documentation improvements are highly appreciated

Please keep code comments in English and follow existing project structure.

## License

See [LICENSE](LICENSE) file for details.
