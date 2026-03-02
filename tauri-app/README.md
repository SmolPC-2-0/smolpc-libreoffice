# LibreOffice AI - Tauri Edition

Cross-platform desktop application for AI-powered LibreOffice interaction, built with Tauri 2.0, Svelte 5, and Rust.

## Project Status

**Week 1: ✅ COMPLETED**
- Tauri 2.0 + Svelte 5 + Rust project structure
- Dependency detection (Python, selected AI provider, LibreOffice)
- MCP server integration with process management
- Loading screen with real-time status indicators
- Dark-themed UI with system status display

**Week 2: ✅ COMPLETED**
- Ollama integration with HTTP streaming
- Modern chat UI with real-time token streaming
- Chat components (ChatMessage, ChatInput)
- MCP client with stdio JSON-RPC communication
- Tool discovery and calling infrastructure
- 27 LibreOffice document manipulation tools
- macOS development workflow documented

**Week 3: ✅ COMPLETED**
- Settings UI with full configuration management
- Settings persistence with Rust backend
- Model selection dropdown
- Path configuration (Python, Documents, LibreOffice)
- Temperature and max tokens controls
- System prompt customization
- Settings button in chat interface

**Week 4: ✅ COMPLETED**
- End-to-end tool calling verified on Windows
- Fixed socket framing mismatch between libre.py and helper.py
- Non-blocking chat streaming (tokio::spawn for HTTP requests)
- Ollama tool parameter validation fix
- Full document creation pipeline working

**Week 5: 🚧 IN PROGRESS (Engine Migration Kickoff)**
- Provider-aware AI architecture (`ollama` or `smolpc_engine`)
- New provider-agnostic chat streaming command (`chat_stream_ai`)
- `smolpc-engine` preview service integration (`/health`, `/generate`)
- Native smolpc-engine tool-call delta assembly for streamed partial arguments
- Provider-aware dependency checks and MCP startup conditions
- Settings updated with AI provider + engine URL
- Added fallback JSON tool-call extraction path for smolpc-engine responses
- Added provider-agnostic model listing command (`list_ai_models`)
- Settings save now re-initializes dependency/provider status in-app (no restart required)
- Added tool-chain safety guards (max tool calls per response + max tool recursion depth)
- See [MIGRATION_PLAN.md](../MIGRATION_PLAN.md) for full roadmap

---

## 📖 New to Windows? Start Here!

**→ [Complete Windows Setup Guide](GETTING_STARTED_WINDOWS.md)** - Step-by-step installation for beginners with no dependencies installed.

## Documentation

- [README.md](README.md) - project overview and quick start
- [GETTING_STARTED_WINDOWS.md](GETTING_STARTED_WINDOWS.md) - full Windows setup guide
- [SMOLPC_ENGINE_WINDOWS_TESTING.md](SMOLPC_ENGINE_WINDOWS_TESTING.md) - Windows validation steps for custom engine provider
- [ENGINEERING_ISSUES.md](ENGINEERING_ISSUES.md) - tracked engineering debt and follow-up issues
- [MIGRATION_PLAN.md](../MIGRATION_PLAN.md) - long-term migration roadmap
- [MACOS_TESTING.md](../MACOS_TESTING.md) - macOS dev workflow notes

## Tech Stack

### Frontend
- **Svelte 5** with TypeScript
- **Svelte Runes** ($state, $derived) for reactive state management
- **Vite 7** for build tooling
- Custom dark theme styling

### Backend
- **Rust** with Tauri 2.0
- **tokio** for async runtime
- **reqwest** for HTTP requests (Ollama API with streaming)
- **futures-util** for stream processing
- **parking_lot** for thread-safe state management
- **anyhow** for error handling
- **serde** and **serde_json** for serialization

### Integration
- **Python MCP Server** (bundled as resources)
- **Ollama** for local AI inference (full tool-calling support)
- **smolpc-engine** daemon (preview integration path)
- **LibreOffice** via Python UNO bridge

## Prerequisites

- **Node.js** 18+ and npm
- **Rust** 1.70+
- **Python** 3.12+
- **One AI provider**:
  - **Ollama** (running on localhost:11434), or
  - **smolpc-engine daemon** (default URL: localhost:11435)
- **LibreOffice** (optional, for document features)

## Quick Start

### Development

```bash
# Install Node dependencies
npm install

# Install MCP server dependencies (Python)
cd resources/mcp_server
python3 -m venv .venv
source .venv/bin/activate  # or .venv\Scripts\activate on Windows
pip install mcp httpx pillow
cd ../..

# Run in development mode
npm run tauri dev
```

**macOS Development Note:**
See [MACOS_TESTING.md](../MACOS_TESTING.md) for the manual setup required on macOS (LibreOffice headless, helper macro).

The app will:
1. Check for Python, selected AI provider, and LibreOffice
2. Show loading screen with status indicators
3. Attempt to start MCP server if dependencies are met
4. Display chat interface when ready

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
│   │   │   ├── LoadingScreen.svelte
│   │   │   ├── ChatMessage.svelte    # NEW: Week 2
│   │   │   └── ChatInput.svelte      # NEW: Week 2
│   │   ├── stores/
│   │   │   ├── app.svelte.ts         # App state (dependencies, MCP)
│   │   │   ├── chat.svelte.ts        # Chat state (messages, streaming)
│   │   │   └── mcp.svelte.ts         # NEW: MCP tools management
│   │   └── types/
│   │       ├── system.ts             # DependencyStatus, McpStatus
│   │       ├── chat.ts               # ChatMessage
│   │       ├── ollama.ts             # NEW: Ollama types
│   │       └── mcp.ts                # NEW: MCP types
│   ├── App.svelte                    # Main app component (chat UI)
│   └── main.ts                       # Entry point
│
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── commands/
│   │   │   ├── system.rs             # check_python, check_ollama, check_libreoffice
│   │   │   ├── ai.rs                 # NEW: provider-aware chat_stream_ai
│   │   │   ├── mcp.rs                # start_mcp_server, list_mcp_tools, call_mcp_tool
│   │   │   └── ollama.rs             # NEW: chat_stream, list_ollama_models
│   │   ├── models/
│   │   │   ├── config.rs             # AppConfig
│   │   │   ├── ollama.rs             # NEW: Ollama types
│   │   │   └── mcp.rs                # NEW: MCP types
│   │   ├── services/
│   │   │   ├── config_service.rs     # Settings persistence
│   │   │   ├── process_manager.rs    # Process lifecycle
│   │   │   ├── ollama_service.rs     # NEW: HTTP streaming
│   │   │   ├── smolpc_engine_service.rs # NEW: engine daemon streaming
│   │   │   └── mcp_client.rs         # NEW: Stdio JSON-RPC client
│   │   └── lib.rs                    # App entry point
│   ├── resources/
│   │   └── mcp_server/               # Bundled Python MCP files
│   │       ├── main.py               # Process launcher (macOS support added)
│   │       ├── libre.py              # MCP protocol server
│   │       ├── helper.py             # LibreOffice UNO bridge
│   │       ├── mcp_helper.py         # macOS macro version
│   │       ├── helper_utils.py
│   │       ├── helper_test_functions.py
│   │       └── .venv/                # Python dependencies
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── MACOS_TESTING.md                  # NEW: macOS workflow guide
└── package.json
```

## Configuration

Settings are stored in `~/.config/libreoffice-ai/settings.json`:

```json
{
  "ai_provider": "ollama",
  "ollama_url": "http://localhost:11434",
  "selected_model": "qwen2.5:7b",
  "smolpc_engine_url": "http://localhost:11435",
  "python_path": "python3",
  "documents_path": "~/Documents",
  "libreoffice_path": null,
  "theme": "dark"
}
```

## Architecture

### Dependency Checking
1. **Python**: Executes `python3 --version`
2. **Selected AI provider**:
   - Ollama: HTTP GET to `/api/version`
   - smolpc-engine: HTTP GET to `/health`
3. **LibreOffice**: Checks filesystem paths (cross-platform)

### MCP Server Integration
1. App starts → Checks dependencies
2. If Python + selected AI provider ready → Starts MCP server process
3. MCP server spawned by `McpClient` (prefers bundled `.venv` Python when available)
4. `McpClient` tracks MCP process lifecycle and stdio JSON-RPC transport
5. Auto-cleanup on app shutdown

### State Flow
```
User opens app
  ↓
appStore.initialize() (onMount)
  ↓
Check Python/Selected AI Provider/LibreOffice in parallel
  ↓
If ready → start_mcp_server command
  ↓
McpClient spawns MCP process
  ↓
Update mcpStatus in store
  ↓
Svelte reactivity updates UI
  ↓
Show main app or loading screen
```

### AI Provider Routing
1. Frontend sends `chat_stream_ai` request with provider + URLs
2. Rust command dispatches:
   - `ollama` → `OllamaService` stream + tool call chunks
   - `smolpc_engine` → `SmolpcEngineService` stream (preview, sends tools/tool_choice contract, parses tool call payloads, and reassembles streamed partial tool arguments)
3. Frontend consumes unified `ai-stream-chunk` / `ai-stream-error` events
4. MCP tool execution runs on detected `tool_calls` for both providers
5. smolpc-engine path additionally supports JSON fallback extraction from assistant text
6. Tool execution is bounded by safety limits (call count + recursion depth)

## Features

### Week 1 ✅
**Dependency Detection**
- Automatically detects Python, selected AI provider, LibreOffice
- Shows installation instructions for missing dependencies
- Color-coded status badges (checking/ready/not-found)

**MCP Server Management**
- Auto-starts MCP server when dependencies ready
- Process lifecycle management (start/stop/cleanup)
- Status monitoring with error reporting

**UI/UX**
- Loading screen with real-time status
- Dark theme with modern styling
- System status display in main app
- Responsive layout

### Week 2 ✅
**Ollama Integration**
- HTTP streaming chat with token-by-token display
- Support for any Ollama model with tool calling (tested: qwen2.5-coder:7b, qwen2.5:1.5b)
- Real-time response generation with visual feedback
- Automatic model discovery
- Tool calling infrastructure for MCP integration

**Chat Interface**
- Modern, responsive chat UI
- Streaming message display with blinking cursor animation
- Auto-scroll to latest messages
- Message history preservation during session
- Support for user, assistant, and tool roles
- Enter to send, Shift+Enter for new line

**MCP Infrastructure**
- JSON-RPC 2.0 protocol implementation
- Stdio transport layer for communication
- Tool discovery (27 LibreOffice tools available)
- Tool invocation ready
- Platform-specific handling (macOS dev / Windows prod)

**Engine Migration Kickoff**
- Provider selection added to settings (`ollama` / `smolpc_engine`)
- Unified AI stream command implemented (`chat_stream_ai`)
- smolpc-engine daemon endpoints integrated for health + generation
- Added native tool-call delta accumulation to support split streamed function arguments
- Backend config save now updates in-memory app state
- smolpc-engine path can trigger MCP tool execution via stream payload or JSON fallback format
- Unified model discovery API (`list_ai_models`) across providers
- Saving settings now triggers dependency + MCP re-initialization using updated provider config
- Tool execution hardening: unknown tool filtering + bounded tool-call loops

**Available LibreOffice Tools (via MCP)**
- **General**: get_document_properties, list_documents, copy_document
- **Writer**: create/read documents, add text/headings/paragraphs/tables/images, format text, search/replace, delete content
- **Impress**: create/read presentations, add/edit/delete slides, format content, apply templates, insert images

## Known Issues

### macOS Development
- **Stdio Communication**: The Rust McpClient has issues with stdio BufReader on macOS. This is a development-only issue and doesn't affect the Windows production target.
- **Manual Process Management**: LibreOffice headless and helper.py must be started manually on macOS due to security restrictions. See [MACOS_TESTING.md](../MACOS_TESTING.md) for workflow.

### General
- **Model Compatibility**: Not all Ollama models support function calling. Tested working models: qwen2.5-coder:7b, qwen2.5:1.5b. Models must support Ollama's tool calling format.
- **CPU Performance**: On CPU-only machines, large models (7B+) with 27 tools can be slow (~9 tok/s). Consider smaller models like qwen2.5:1.5b for faster responses.
- **smolpc-engine Preview**: Native streamed tool-call payloads (including partial argument deltas) are supported, but non-standard runtimes may still require strict JSON fallback output.
- **Chat Persistence**: Messages are not saved between sessions
- **Voice Input**: Not implemented (future feature)
- **Tracked Engineering Debt**: See [ENGINEERING_ISSUES.md](ENGINEERING_ISSUES.md) for prioritized follow-up issues from code review.

## Testing

### Manual Testing Checklist

**Week 1 ✅**
- [x] App builds successfully
- [x] Dependency detection works (Python, selected AI provider, LibreOffice)
- [x] Loading screen shows correct statuses
- [x] MCP server command executes without errors

**Week 2 ✅**
- [x] Chat interface displays correctly
- [x] Can send messages to Ollama
- [x] Messages stream token-by-token
- [x] Auto-scroll works
- [x] Message history persists during session
- [x] MCP client can initialize connection
- [x] Tool discovery works (27 tools listed)

**Week 4 ✅**
- [x] Full end-to-end tool calling
- [x] Document creation via chat
- [ ] Presentation creation via chat
- [ ] Text formatting via chat

**Week 5 🚧**
- [x] Provider-aware AI routing (`ollama` / `smolpc_engine`)
- [x] Unified frontend stream event path (`ai-stream-*`)
- [x] Provider-aware dependency checks in loading flow
- [x] smolpc-engine native tool-calling integration
- [ ] End-to-end document operations on smolpc-engine provider

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
- MCP server process is currently managed by `McpClient`
- `ProcessManager` remains available for broader process lifecycle work
- Auto-cleanup via Drop trait
- Thread-safe with parking_lot RwLock

## Next Steps (Week 3+)

### Immediate Priorities
1. **Windows Testing** - Validate full end-to-end MCP tool calling
2. **Settings UI** - Model selection, configuration management
3. **Chat Persistence** - Save/load conversation history
4. **Error Handling** - Improve error messages and recovery

### Week 3 Planned Features
1. **Settings Page**
   - Model dropdown with all available Ollama models
   - Document folder configuration
   - Theme selection (dark/light)
   - System prompt customization

2. **Enhanced Chat**
   - Message editing/deletion
   - Conversation export (JSON, Markdown)
   - Clear chat button
   - Token usage tracking

3. **Document Management**
   - Recent documents sidebar
   - Quick document preview
   - Template library
   - Open in LibreOffice button

### Week 4-5 (Stretch Goals)
- Voice input via Whisper.cpp
- Multi-language support
- Advanced formatting tools
- Batch document operations

See [MIGRATION_PLAN.md](../MIGRATION_PLAN.md) for the complete roadmap.

## License

See [../libre-office-mcp/LICENSE](../libre-office-mcp/LICENSE)

## Credits

**Original Project**: [LibreOfficeAI](https://github.com/SmolPC-2-0/smolpc-libreoffice) (C# + WinUI 3 + OllamaSharp)

**MCP Server Base**: [libre-office-mcp](https://github.com/harshithb3304/libre-office-mcp)

**Technologies**:
- [Tauri 2.0](https://tauri.app/) - Desktop application framework
- [Svelte 5](https://svelte.dev/) - Frontend framework
- [Ollama](https://ollama.ai/) - Local AI model serving
- [Model Context Protocol](https://modelcontextprotocol.io/) - AI-tool integration standard
- [LibreOffice](https://www.libreoffice.org/) - Open source office suite

---

**Last Updated**: March 2, 2026
**Status**: Week 5 In Progress - Engine migration kickoff implemented
**Target Platform**: Windows (Development on macOS)
