
## Tech Stack

### Backend (Rust)
- **Axum** - Web framework
- **Tokio** - Async runtime
- **Serde** - JSON serialization
- **Walkdir** - Directory traversal
- **Chrono** - Date/time handling

### Frontend (React)
- **React 19** - UI framework
- **Vite** - Build tool
- **Tailwind CSS v4** - Styling
- **shadcn/ui** - UI components
- **Bun** - Package manager

## Project Structure

```
agent-insights/
├── apps/
│   ├── backend/          # Rust API server
│   │   ├── src/
│   │   │   ├── agents/   # Agent-specific scanners
│   │   │   ├── api/      # HTTP endpoints
│   │   │   ├── domain/   # Data models
│   │   │   ├── scanner/  # File system scanning
│   │   │   ├── services/ # Business logic
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── frontend/         # React app
│       ├── src/
│       │   ├── components/
│       │   ├── services/
│       │   ├── types/
│       │   └── App.tsx
│       └── package.json
└── README.md
```
