# LibreOffice AI - Tauri Edition

Cross-platform desktop application for AI-powered LibreOffice interaction, built with Tauri 2.0, Svelte 5, and Rust.

## Project Status

**Week 1: ✅ COMPLETED**
- Tauri 2.0 + Svelte 5 + Rust project structure
- Dependency detection (Python, Ollama, LibreOffice)
- MCP server integration with process management
- Loading screen with real-time status indicators
- Dark-themed UI with system status display

**Week 2-5: In Progress**
- See [MIGRATION_PLAN.md](../MIGRATION_PLAN.md) for full roadmap

## Tech Stack

### Frontend
- **Svelte 5** with TypeScript
- **Svelte Runes** ($state, $derived) for reactive state management
- **Vite 7** for build tooling
- Custom dark theme styling

### Backend
- **Rust** with Tauri 2.0
- **tokio** for async runtime
- **reqwest** for HTTP requests (Ollama API)
- **parking_lot** for thread-safe state management
- **anyhow** for error handling

### Integration
- **Python MCP Server** (bundled as resources)
- **Ollama** for local AI inference
- **LibreOffice** via Python UNO bridge

## Prerequisites

- **Node.js** 18+ and npm
- **Rust** 1.70+
- **Python** 3.12+
- **Ollama** (running on localhost:11434)
- **LibreOffice** (optional, for document features)

## Quick Start

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev
```

The app will:
1. Check for Python, Ollama, and LibreOffice
2. Show loading screen with status indicators
3. Attempt to start MCP server if dependencies are met
4. Display main UI when ready

### Build

```bash
# Build frontend
npm run build

# Build Tauri app (creates installer)
npm run tauri:build
```

## Project Structure

```
tauri-app/
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   │   └── LoadingScreen.svelte
│   │   ├── stores/
│   │   │   ├── app.svelte.ts     # App state (dependencies, MCP)
│   │   │   └── chat.svelte.ts    # Chat state (messages)
│   │   └── types/
│   │       ├── system.ts         # DependencyStatus, McpStatus
│   │       └── chat.ts           # ChatMessage
│   ├── App.svelte                # Main app component
│   └── main.ts                   # Entry point
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/
│   │   │   ├── system.rs         # check_python, check_ollama, check_libreoffice
│   │   │   └── mcp.rs            # start_mcp_server, check_mcp_status, stop_mcp_server
│   │   ├── models/
│   │   │   └── config.rs         # AppConfig
│   │   ├── services/
│   │   │   ├── config_service.rs # Settings persistence
│   │   │   └── process_manager.rs # Process lifecycle (Ollama, MCP, LibreOffice)
│   │   └── lib.rs                # App entry point
│   ├── resources/
│   │   └── mcp_server/           # Bundled Python MCP files
│   │       ├── main.py
│   │       ├── libre.py
│   │       ├── helper.py
│   │       ├── helper_utils.py
│   │       └── helper_test_functions.py
│   ├── Cargo.toml
│   └── tauri.conf.json
│
└── package.json
```

## Configuration

Settings are stored in `~/.config/libreoffice-ai/settings.json`:

```json
{
  "ollama_url": "http://localhost:11434",
  "selected_model": "qwen2.5:7b",
  "python_path": "python3",
  "documents_path": "~/Documents",
  "libreoffice_path": null,
  "theme": "dark"
}
```

## Architecture

### Dependency Checking
1. **Python**: Executes `python3 --version`
2. **Ollama**: HTTP GET to `http://localhost:11434/api/version`
3. **LibreOffice**: Checks filesystem paths (cross-platform)

### MCP Server Integration
1. App starts → Checks dependencies
2. If Python + Ollama ready → Starts MCP server process
3. MCP server spawned with: `python3 <resources>/mcp_server/main.py`
4. ProcessManager tracks MCP process lifecycle
5. Auto-cleanup on app shutdown

### State Flow
```
User opens app
  ↓
appStore.initialize() (onMount)
  ↓
Check Python/Ollama/LibreOffice in parallel
  ↓
If ready → start_mcp_server command
  ↓
ProcessManager spawns MCP process
  ↓
Update mcpStatus in store
  ↓
Svelte reactivity updates UI
  ↓
Show main app or loading screen
```

## Features (Week 1)

✅ **Dependency Detection**
- Automatically detects Python, Ollama, LibreOffice
- Shows installation instructions for missing dependencies
- Color-coded status badges (checking/ready/not-found)

✅ **MCP Server Management**
- Auto-starts MCP server when dependencies ready
- Process lifecycle management (start/stop/cleanup)
- Status monitoring with error reporting

✅ **UI/UX**
- Loading screen with real-time status
- Dark theme with modern styling
- System status display in main app
- Responsive layout

## Known Issues

- MCP server currently starts but may not fully initialize (Week 2 work)
- No chat interface yet (Week 2-3 work)
- No Ollama service integration yet (Week 2 work)
- Voice recording not implemented yet (Week 4-5 work)

## Testing

### Manual Testing Checklist
- [x] App builds successfully
- [x] Dependency detection works (Python, Ollama, LibreOffice)
- [x] Loading screen shows correct statuses
- [x] MCP server command executes without errors
- [ ] MCP server fully functional (pending Week 2)
- [ ] Chat interface (pending Week 2-3)

## Development Notes

### Svelte 5 Runes
This project uses Svelte 5 runes for state management:
- `$state()` for reactive state
- `$derived()` for computed values
- Must be class fields, not getters

### Tauri 2.0
- Commands registered in `lib.rs`
- Resource bundling via `tauri.conf.json`
- AppHandle required for resource path resolution

### Process Management
- All child processes tracked in ProcessManager
- Auto-cleanup via Drop trait
- Thread-safe with parking_lot RwLock

## Next Steps (Week 2)

See [MIGRATION_PLAN.md](../MIGRATION_PLAN.md) for detailed Week 2 plan:
1. OllamaService implementation
2. McpClient for stdio communication
3. Chat streaming support
4. Chat UI components
5. Integration testing

## License

See [../libre-office-mcp/LICENSE](../libre-office-mcp/LICENSE)

## Credits

Built as a cross-platform migration of [LibreOfficeAI](../LibreOfficeAI)
