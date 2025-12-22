# Agent Insights

A full-stack application for tracking and visualizing AI agent usage across Claude, Gemini, and Codex with GitHub-style heatmap charts.

## Overview

Agent Insights scans your local agent directories and provides beautiful visualizations of your usage patterns over time. See when you're most active with each AI agent, track file generation, and monitor storage usage.

## Features

- **Multi-Agent Support**: Claude, Gemini, and Codex
- **GitHub-Style Heatmaps**: Year-long activity visualization
- **Real-time Scanning**: Automatically scans local agent directories
- **Usage Analytics**: Track files, dates, and storage size
- **Modern UI**: React + Tailwind CSS v4 + shadcn/ui
- **Fast Backend**: Rust + Axum for high-performance data processing

## Quick Start

```sh
git clone https://github.com/milisp/agent-insights
cd agent-insights
```

### Prerequisites
- Rust 1.91+
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

## Performance

Typical scan results:
- **666 Claude files** (20 days)
- **558 Gemini files** (65 days)
- **493 Codex files** (68 days)
- **Total: 1,717 files** scanned in ~1.3 seconds

## Development

[ARCHITECTURE](docs/ARCHITECTURE.md)

simple `just` to start backend and frontend or

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

## License

See [LICENSE](LICENSE) file for details.
