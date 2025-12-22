# Agent Insights Backend

Rust backend API for scanning and aggregating AI agent usage data.

## Features

- Scans local agent directories (Claude, Gemini, Codex)
- Parses JSON/JSONL session files
- Aggregates data by date for heatmap visualization
- RESTful API with CORS support

## Architecture

```
backend/
├── src/
│   ├── domain/          # Domain models
│   │   ├── record.rs    # AgentRecord, TokenInfo
│   │   └── heatmap.rs   # HeatmapData, DayActivity
│   ├── scanner/         # File scanning
│   │   └── mod.rs       # FileScanner with walkdir
│   ├── agents/          # Agent-specific parsers
│   │   ├── claude.rs    # JSONL parser for ~/.claude/projects
│   │   ├── gemini.rs    # JSON parser for ~/.gemini/tmp
│   │   └── codex.rs     # Scanner for ~/.codex
│   ├── services/        # Business logic
│   │   ├── collect.rs   # CollectionService
│   │   └── aggregate.rs # AggregationService
│   ├── api/             # HTTP API
│   │   └── heatmap.rs   # Heatmap endpoints
│   └── main.rs          # Axum server
```

## API Endpoints

### GET /api/heatmaps
Returns all agent heatmaps
```json
{
  "Claude": {
    "agent": "Claude",
    "data": [
      {"date": "2025-12-02", "count": 20, "size": 5096527},
      {"date": "2025-12-03", "count": 27, "size": 1687467}
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
```json
{
  "agent": "Claude",
  "data": [...],
  "max_count": 82,
  "total_files": 666,
  "total_size": 123456789
}
```

## Running

```bash
cargo run
```

Server starts on http://127.0.0.1:3001

## Implementation Details

### Claude Scanner
- Location: `~/.claude/projects/`
- Format: JSONL (newline-delimited JSON)
- Extracts: session_id, token usage (input/output/cached)
- Pattern: One event per line

### Gemini Scanner
- Location: `~/.gemini/tmp/{hash}/chats/`
- Format: JSON
- Extracts: session_id, token usage
- Pattern: Multiple chat files per session

### Codex Scanner
- Location: `~/.codex/sessions/`
- Format: JSON/JSONL
- Extracts: Basic file metadata
- Pattern: Various file structures

## Performance

Startup scan results:
- 666 Claude records (20 days)
- 558 Gemini records (65 days)
- 493 Codex records (68 days)
- Total: 1717 records in ~1.3 seconds

## Dependencies

- `axum` - Web framework
- `tokio` - Async runtime
- `serde/serde_json` - JSON parsing
- `chrono` - Date/time handling
- `walkdir` - Recursive directory traversal
- `tower-http` - CORS middleware
- `tracing` - Logging
