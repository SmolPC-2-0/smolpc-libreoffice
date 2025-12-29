# SmolPC LibreOffice AI Integration

## Repository Overview

This repository contains two complementary projects that enable AI-powered interaction with LibreOffice applications:

1. **LibreOfficeAI** - A Windows desktop application with chat and voice interface
2. **libre-office-mcp** - A Model Context Protocol (MCP) server for LibreOffice integration

Both projects enable AI assistants to create, read, and edit LibreOffice documents and presentations through natural language commands.

---

## Project 1: LibreOfficeAI (Windows Desktop Application)

### Purpose
A complete Windows desktop application providing a chat-based AI assistant for LibreOffice Writer and Impress, with voice input support and fully local AI processing.

### Technology Stack
- **Framework**: .NET 8.0, WinUI 3 (Windows App SDK)
- **UI Pattern**: MVVM with CommunityToolkit.Mvvm
- **Dependency Injection**: Microsoft.Extensions.DependencyInjection
- **Target Platform**: Windows 10.0.17763.0+ (Windows 10 version 1809+)
- **Architecture**: x64 only

### Key Dependencies

```xml
<!-- AI & ML -->
<PackageReference Include="OllamaSharp" Version="5.3.6" />
<PackageReference Include="OllamaSharp.ModelContextProtocol" Version="5.3.6" />
<PackageReference Include="Whisper.net" Version="1.8.1" />
<PackageReference Include="Whisper.net.Runtime.Cuda" Version="1.8.1" />

<!-- Audio -->
<PackageReference Include="NAudio" Version="2.2.1" />

<!-- Framework -->
<PackageReference Include="CommunityToolkit.Mvvm" Version="8.4.0" />
<PackageReference Include="Microsoft.WindowsAppSDK" Version="1.7.250606001" />
```

### Architecture

#### Service Layer ([LibreOfficeAI/Services/](LibreOfficeAI/Services/))

**OllamaService.cs** ([LibreOfficeAI/Services/OllamaService.cs](LibreOfficeAI/Services/OllamaService.cs))
- Manages embedded Ollama AI service lifecycle
- Starts/stops Ollama process from bundled executable
- Provides two chat instances: ExternalChat (user-facing) and InternalChat (tool selection)
- Handles model downloading and readiness checks
- System prompt includes document folder path and available documents

**ChatService.cs** ([LibreOfficeAI/Services/ChatService.cs](LibreOfficeAI/Services/ChatService.cs))
- Orchestrates chat interactions between user and AI
- Manages chat message collection (ObservableCollection)
- Handles cancellation tokens for stopping AI responses
- Raises events for UI updates (scroll, focus, refresh)
- Integrates with ToolService for MCP tool execution

**DocumentService.cs** ([LibreOfficeAI/Services/DocumentService.cs](LibreOfficeAI/Services/DocumentService.cs))
- Manages document file operations
- Tracks available documents in configured folder
- Provides document list strings for AI context
- Handles presentation template discovery

**WhisperService.cs** ([LibreOfficeAI/Services/WhisperService.cs](LibreOfficeAI/Services/WhisperService.cs))
- Local speech-to-text using Whisper.net
- Uses bundled `ggml-base.bin` model
- Supports CUDA, Vulkan, OpenVino acceleration
- Transcribes recorded audio to text

**AudioService.cs** ([LibreOfficeAI/Services/AudioService.cs](LibreOfficeAI/Services/AudioService.cs))
- Handles microphone recording via NAudio
- Captures WAV format audio
- Provides start/stop recording functionality

**ToolService.cs** ([LibreOfficeAI/Services/ToolService.cs](LibreOfficeAI/Services/ToolService.cs))
- Integrates OllamaSharp with MCP server
- Routes tool calls to the bundled MCP server
- Parses and executes MCP tool responses

**ConfigurationService.cs** ([LibreOfficeAI/Services/ConfigurationService.cs](LibreOfficeAI/Services/ConfigurationService.cs))
- Loads settings from [settings.json](LibreOfficeAI/settings.json)
- Manages Ollama URI, model selection, document paths
- Provides paths to system/intent prompts

**UserPromptService.cs** ([LibreOfficeAI/Services/UserPromptService.cs](LibreOfficeAI/Services/UserPromptService.cs))
- Handles user prompt preprocessing
- Intent detection for tool selection

#### MVVM Layer

**MainViewModel.cs** ([LibreOfficeAI/ViewModels/MainViewModel.cs](LibreOfficeAI/ViewModels/MainViewModel.cs))
- Primary UI state management
- Commands for sending messages, recording voice, canceling requests
- Bindings for chat messages, AI status, recording state

**SettingsViewModel.cs** ([LibreOfficeAI/ViewModels/SettingsViewModel.cs](LibreOfficeAI/ViewModels/SettingsViewModel.cs))
- Model selection, document folder configuration
- Settings persistence

**HelpViewModel.cs** ([LibreOfficeAI/ViewModels/HelpViewModel.cs](LibreOfficeAI/ViewModels/HelpViewModel.cs))
- Help content display

#### Views

**MainPage.xaml** ([LibreOfficeAI/Views/MainPage.xaml](LibreOfficeAI/Views/MainPage.xaml))
- Chat interface with message list
- Text input and voice recording controls
- Loading states and AI status indicators

**SettingsPage.xaml** ([LibreOfficeAI/Views/SettingsPage.xaml](LibreOfficeAI/Views/SettingsPage.xaml))
- Configuration interface

**HelpPage.xaml** ([LibreOfficeAI/Views/HelpPage.xaml](LibreOfficeAI/Views/HelpPage.xaml))
- User documentation

### AI Prompts

**SystemPrompt.txt** ([LibreOfficeAI/SystemPrompt.txt](LibreOfficeAI/SystemPrompt.txt))
```
You are an AI assistant designed to work with LibreOffice.
If there are missing parameters for a tool you need to use, make up suitable ones.
Do not ask the user for clarification. If you need a file_path, make sure you use the full path including the path to the Documents folder.
If you create a new document, use further tools to add content to it. Don't ask for confirmation, just use the tools immediately.
If it is not clear which part of a document/presentation the user is referring to, try to read the contents first to find out.
When finished, always check that you have actually made the tool calls, and not just stated you have made them.
If the tool returns an error, always tell the user it has not worked successfully.
```

**IntentPrompt.txt** ([LibreOfficeAI/IntentPrompt.txt](LibreOfficeAI/IntentPrompt.txt))
- Lists all available MCP tools
- Used for intent detection and tool selection
- Includes Writer functions (create, edit, format documents)
- Includes Impress functions (create, edit presentations)
- General functions (properties, list, copy)

### Bundled Components

**Ollama/** (excluded from repo, added during build)
- Full Ollama installation for Windows
- Includes model storage directory
- Downloaded from [ollama/ollama releases](https://github.com/ollama/ollama/releases)

**WhisperModels/** (excluded from repo, added during build)
- `ggml-base.bin` - Whisper base model
- Downloaded from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp)

**MCPServer/** ([LibreOfficeAI/MCPServer/](LibreOfficeAI/MCPServer/))
- `main.exe` - MCP server launcher (Nuitka-compiled)
- `libre.exe` - MCP server (Nuitka-compiled)
- `helper.py` - LibreOffice UNO bridge
- `helper_test_functions.py` - Testing utilities
- `helper_utils.py` - Shared utilities

### Build Process

1. Install Visual Studio with:
   - .NET desktop development
   - Python development
   - MSVC C++ x64/x86 build tools

2. Add Ollama to `Ollama/` folder

3. Add Whisper model to `WhisperModels/` folder

4. Publish project (creates self-contained deployment)

5. Run `LibreOfficeAI.exe`

### Configuration Files

**settings.json** ([LibreOfficeAI/settings.json](LibreOfficeAI/settings.json))
- User-configurable settings
- Model selection, paths, preferences

**server_config.json** ([LibreOfficeAI/server_config.json](LibreOfficeAI/server_config.json))
```json
{
  "mcpServers": {
    "libreoffice-server": {
      "command": "C:\\Users\\ben_t\\source\\repos\\LibreOfficeAI\\MCPServer\\main.exe",
      "args": []
    }
  }
}
```

---

## Project 2: libre-office-mcp (MCP Server)

### Purpose
A standalone Model Context Protocol server that exposes LibreOffice functionality as tools for AI assistants like Claude Desktop.

### Technology Stack
- **Language**: Python 3.12+
- **Protocol**: Model Context Protocol (MCP)
- **Build Tool**: uv (Python package manager)
- **LibreOffice API**: UNO (Universal Network Objects)

### Dependencies ([libre-office-mcp/libre-writer/pyproject.toml](libre-office-mcp/libre-writer/pyproject.toml))

```toml
[project]
name = "libre"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [
    "httpx>=0.28.1",
    "mcp[cli]>=1.6.0",
    "pillow>=11.3.0",
]
```

### Architecture

The MCP server uses a three-process architecture:

#### 1. main.py - Process Launcher ([libre-office-mcp/libre-writer/main.py](libre-office-mcp/libre-writer/main.py))

**Responsibilities:**
- Starts LibreOffice in headless mode
- Launches helper.py script
- Starts MCP server (libre.py)
- Manages process lifecycle and cleanup

**Key Functions:**

`get_office_path()` ([main.py:46-76](libre-office-mcp/libre-writer/main.py#L46-L76))
- Detects Collabora Office or LibreOffice installation
- Platform-specific path detection (Windows/Linux)
- Returns executable path

`get_python_path()` ([main.py:79-94](libre-office-mcp/libre-writer/main.py#L79-L94))
- Finds LibreOffice's bundled Python (required for UNO)
- Falls back to system Python on Linux

`start_office()` ([main.py:102-123](libre-office-mcp/libre-writer/main.py#L102-L123))
- Launches LibreOffice headless on port 2002
- Command: `soffice --headless --accept=socket,host=localhost,port=2002;urp;`
- Checks if port already in use

`start_helper()` ([main.py:125-138](libre-office-mcp/libre-writer/main.py#L125-L138))
- Launches helper.py with LibreOffice Python
- Helper listens on port 8765
- Executes actual LibreOffice operations

`start_mcp_server()` ([main.py:140-150](libre-office-mcp/libre-writer/main.py#L140-L150))
- Starts the MCP protocol server
- Uses system Python (not LibreOffice Python)

`cleanup_processes()` ([main.py:16-37](libre-office-mcp/libre-writer/main.py#L16-L37))
- Gracefully terminates all child processes
- Registered with `atexit` and signal handlers

#### 2. helper.py - LibreOffice UNO Bridge ([libre-office-mcp/libre-writer/helper.py](libre-office-mcp/libre-writer/helper.py))

**Responsibilities:**
- Socket server on port 8765
- Accepts JSON commands from libre.py
- Executes LibreOffice operations via UNO API
- Returns results as JSON

**Architecture:**
- Must run with LibreOffice's bundled Python (includes UNO module)
- Connects to LibreOffice via UNO socket (port 2002)
- Processes requests sequentially

**Key Imports:**
```python
import uno
from com.sun.star.beans import PropertyValue
from com.sun.star.text import ControlCharacter
from com.sun.star.text.TextContentAnchorType import AS_CHARACTER
from com.sun.star.awt import Size
# ... extensive UNO API imports
```

**Core Functions:**

`create_document(doc_type, file_path, metadata)` ([helper.py:73-135](libre-office-mcp/libre-writer/helper.py#L73-L135))
- Creates text/calc/impress documents
- Sets metadata (author, description, keywords)
- Saves to specified path

`list_documents(directory)` ([helper.py:137-150+](libre-office-mcp/libre-writer/helper.py#L137-L150))
- Lists all LibreOffice/MS Office documents in folder
- Returns file details

Additional functions (all 150+ documented in helper.py):
- Writer: add_text, add_heading, add_paragraph, add_table, format_text, etc.
- Impress: add_slide, edit_slide_content, format_slide_title, etc.
- General: copy_document, get_document_properties, etc.

**Utility Module:** [helper_utils.py](libre-office-mcp/libre-writer/helper_utils.py)
- `get_uno_desktop()` - Connects to LibreOffice via UNO
- `normalize_path()` - Cross-platform path handling
- `ensure_directory_exists()` - Directory creation
- `managed_document()` - Context manager for document lifecycle
- `create_property_value()` - UNO property creation

**Testing Module:** [helper_test_functions.py](libre-office-mcp/libre-writer/helper_test_functions.py)
- Verification functions for testing
- Text formatting checks, image detection, table validation

#### 3. libre.py - MCP Server ([libre-office-mcp/libre-writer/libre.py](libre-office-mcp/libre-writer/libre.py))

**Responsibilities:**
- Implements Model Context Protocol
- Exposes LibreOffice tools to MCP clients
- Routes requests to helper.py via socket
- Manages request queue and threading

**Architecture:**

```python
# Global Queue and Thread Pool
request_queue = Queue()
response_dict = {}
response_lock = threading.Lock()
thread_pool = ThreadPoolExecutor(max_workers=1)
```

`queue_worker()` ([libre.py:54-110+](libre-office-mcp/libre-writer/libre.py#L54-L110))
- Processes requests sequentially from queue
- Connects to helper.py on localhost:8765
- Sends JSON command, receives JSON response
- Handles timeouts and connection errors

**MCP Tool Definitions:**
- Each LibreOffice function exposed as MCP tool
- Tools include parameter schemas
- Tools grouped by application (Writer/Impress/General)

**Extension Handling:**
```python
writer_extensions = (".odt", ".docx", ".dotx", ".xml", ".doc", ".dot", ".rtf", ".wpd")
impress_extensions = (".odp", ".pptx", ".ppsx", ".ppmx", ".potx", ".pomx", ".ppt", ".pps", ".ppm", ".pot", ".pom")
```

### Available Tools

#### General Tools
1. `get_document_properties` - Get metadata (author, word count, etc.)
2. `list_documents` - List all documents in directory
3. `copy_document` - Create document copy

#### Writer (Text Document) Tools
1. `create_blank_document` - New document
2. `read_text_document` - Read document contents
3. `add_text` - Add text
4. `add_heading` - Add heading (levels 1-10)
5. `add_paragraph` - Add paragraph with styling
6. `add_table` - Create table with data
7. `insert_image` - Insert image with optional resize
8. `insert_page_break` - Add page break
9. `format_text` - Format specific text (bold, italic, color, font, size)
10. `search_replace_text` - Find and replace
11. `delete_text` - Remove specific text
12. `format_table` - Style table (borders, colors, header)
13. `delete_paragraph` - Remove paragraph by index
14. `apply_document_style` - Apply consistent formatting

#### Impress (Presentation) Tools
1. `create_blank_presentation` - New presentation
2. `read_presentation` - Read presentation text
3. `add_slide` - Add new slide
4. `edit_slide_content` - Edit slide content text
5. `edit_slide_title` - Edit slide title
6. `delete_slide` - Remove slide
7. `apply_presentation_template` - Apply built-in template
8. `format_slide_content` - Format slide content text
9. `format_slide_title` - Format slide title text
10. `insert_slide_image` - Insert image into slide

### Installation & Usage

#### Standalone Use (with Claude Desktop)

1. **Install dependencies:**
```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh  # macOS/Linux
# or
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"  # Windows

# Install packages
cd libre-office-mcp/libre-writer
uv init
uv venv
uv add mcp[cli] httpx pillow
```

2. **Install LibreOffice:**
```bash
# macOS
brew install --cask libreoffice

# Windows
# Download from https://www.libreoffice.org/download/
```

3. **Configure Claude Desktop:**

Edit configuration file:
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "libreoffice-server": {
      "command": "uv",
      "args": [
        "--directory",
        "/path/to/libre-office-mcp/libre-writer",
        "run",
        "main.py"
      ]
    }
  }
}
```

4. **Restart Claude Desktop**

#### As Part of LibreOfficeAI

The MCP server is bundled and auto-configured. See LibreOfficeAI build instructions above.

### Example Usage

**Create a document:**
```
"Create a new document called 'Meeting Notes' with a heading 'Team Standup' and a paragraph about today's tasks"
```

**Format text:**
```
"Make the text 'Important!' bold, red, and size 16 in the report document"
```

**Create presentation:**
```
"Create a presentation called 'Sales Pitch' with 3 slides about our product features"
```

**Add table:**
```
"Add a 3x4 table showing quarterly sales data to the Q4 Report document"
```

---

## Data Flow

### Complete Request Flow

```
User Input (Text or Voice)
  ↓
[LibreOfficeAI Desktop App]
  ↓
WhisperService (if voice) → Text
  ↓
ChatService.SendMessageAsync()
  ↓
OllamaService.ExternalChat (with SystemPrompt)
  ↓
OllamaService.InternalChat (tool selection via IntentPrompt)
  ↓
ToolService (MCP client)
  ↓
[MCP Protocol over stdio/socket]
  ↓
libre.py (MCP Server)
  ↓
request_queue → queue_worker()
  ↓
[Socket connection to localhost:8765]
  ↓
helper.py (receives JSON command)
  ↓
[UNO Socket to localhost:2002]
  ↓
LibreOffice (headless mode)
  ↓
Document/Presentation file system operations
  ↓
[Response chain back to user]
```

### Port Usage

| Port | Service | Protocol |
|------|---------|----------|
| 2002 | LibreOffice headless | UNO socket |
| 8765 | helper.py | JSON over TCP |
| stdio | libre.py (MCP) | MCP protocol |

---

## File Extensions Support

### Writer (Text Documents)
- `.odt` - OpenDocument Text (native)
- `.docx` - Microsoft Word 2007+
- `.dotx` - Word template
- `.doc` - Word 97-2003
- `.dot` - Word 97-2003 template
- `.rtf` - Rich Text Format
- `.wpd` - WordPerfect
- `.xml` - Various XML formats

### Impress (Presentations)
- `.odp` - OpenDocument Presentation (native)
- `.pptx` - PowerPoint 2007+
- `.ppsx` - PowerPoint Show
- `.ppmx` - PowerPoint macro-enabled
- `.potx` - PowerPoint template
- `.ppt` - PowerPoint 97-2003
- `.pps` - PowerPoint Show 97-2003
- `.ppm` - PowerPoint macro-enabled 97-2003
- `.pot` - PowerPoint template 97-2003

---

## System Requirements

### LibreOfficeAI
- **OS**: Windows 10 version 1809+ (build 17763+)
- **Runtime**: .NET 8.0 (included in self-contained build)
- **LibreOffice**: Any recent version
- **Disk Space**: ~2GB (includes Ollama, models, Whisper)
- **RAM**: 8GB minimum, 16GB recommended (for AI models)
- **GPU**: Optional (CUDA/Vulkan for Whisper acceleration)

### libre-office-mcp
- **Python**: 3.12+
- **LibreOffice**: 7.0+ (includes Python with UNO)
- **OS**: Windows, macOS, Linux
- **RAM**: 2GB minimum
- **Disk Space**: Minimal (~50MB for MCP server)

---

## Development Notes

### Key Design Decisions

1. **Two Python Interpreters Required:**
   - LibreOffice's Python (has UNO) runs helper.py
   - System Python (has MCP libs) runs libre.py
   - Communication via socket bridges the gap

2. **Sequential Request Processing:**
   - Queue with max_workers=1 ensures thread safety
   - LibreOffice UNO API not thread-safe for document operations

3. **Headless LibreOffice:**
   - Runs without GUI for performance
   - Enables automation and scripting
   - User sees results by opening saved files

4. **Local-First AI:**
   - All AI processing via Ollama (local)
   - Whisper for local speech-to-text
   - No cloud dependencies, data stays on device

5. **MCP as Integration Layer:**
   - Standard protocol for AI-tool communication
   - Enables use with any MCP-compatible client
   - Decouples AI logic from LibreOffice specifics

### Testing

**Test Files:**
- [libre-office-mcp/libre-writer/test_helper.py](libre-office-mcp/libre-writer/test_helper.py)
- [libre-office-mcp/libre-writer/tests/test_main.py](libre-office-mcp/libre-writer/tests/test_main.py)
- [libre-office-mcp/libre-writer/tests/test_mcp.py](libre-office-mcp/libre-writer/tests/test_mcp.py)

**Run tests:**
```bash
cd libre-office-mcp/libre-writer
pytest
```

### Known Limitations

1. **Presentation Layouts:**
   - Limited to simple title + content layout
   - Complex layouts require manual editing in Impress

2. **Image Paths:**
   - Require absolute paths
   - No automatic image resolution

3. **Template Names:**
   - Must match built-in LibreOffice templates exactly
   - No fuzzy matching

4. **Windows Defender:**
   - Nuitka-compiled executables may trigger false positives
   - `main.exe` and `libre.exe` may need manual approval

5. **LibreOffice Version:**
   - UNO API varies slightly between versions
   - Tested primarily with LibreOffice 7.x+

### Security Considerations

1. **Local Execution Only:**
   - All AI models run locally
   - No data sent to external servers
   - File operations restricted to configured directories

2. **Socket Security:**
   - Localhost-only binding (127.0.0.1)
   - No external network exposure
   - No authentication (assumes local trust)

3. **File Access:**
   - Full filesystem access via file_path parameters
   - No sandboxing or path restrictions
   - Users should trust prompt inputs

---

## Troubleshooting

### Common Issues

**"LibreOffice helper is not running"**
- Check LibreOffice installation path
- Verify port 8765 not in use: `netstat -an | grep 8765`
- Check helper.py has correct Python interpreter

**"Connection refused"**
- Ensure helper script started successfully
- Check firewall settings for localhost
- Verify port availability

**"Failed to connect to LibreOffice desktop"**
- LibreOffice may not be running headless
- Check port 2002: `netstat -an | grep 2002`
- Kill existing LibreOffice processes

**Ollama model not loading (LibreOfficeAI)**
- Check Ollama folder exists and is populated
- Verify internet connection for initial model download
- Check disk space for model storage

**Whisper transcription not working**
- Verify WhisperModels/ggml-base.bin exists
- Check microphone permissions
- Try different Whisper runtime (CPU vs CUDA)

**MCP tools not appearing in Claude Desktop**
- Restart Claude Desktop after config changes
- Check config file JSON syntax
- Verify paths in claude_desktop_config.json
- Check Claude Desktop logs

### Debug Logging

**LibreOfficeAI:**
- Unhandled exceptions → `unhandled_exception.txt`
- Launch errors → `launch_exception.txt`
- Init errors → `app_initialization_exception.txt`

**MCP Server:**
- `libre.log` - MCP server operations
- LibreOffice helper logs to stdout/stderr

---

## Future Enhancements

### Potential Features
- [ ] LibreCalc (spreadsheet) support
- [ ] Draw (graphics) support
- [ ] PDF export functionality
- [ ] Collaborative editing
- [ ] Multi-document operations
- [ ] Template creation/management
- [ ] Advanced formatting (styles, master pages)
- [ ] Chart creation
- [ ] Mail merge
- [ ] Version control integration

### Architecture Improvements
- [ ] gRPC instead of raw sockets (helper ↔ libre)
- [ ] Process pooling for concurrent requests
- [ ] Caching layer for document reads
- [ ] Async/await throughout Python code
- [ ] Better error recovery and retry logic
- [ ] Metrics and observability

---

## Contributing

### Code Structure Guidelines

**C# Code (LibreOfficeAI):**
- Follow MVVM pattern strictly
- Services should be stateless where possible
- Use dependency injection for all services
- ObservableObject for data binding
- Async/await for I/O operations

**Python Code (MCP Server):**
- Type hints for all function signatures
- Docstrings for public functions
- Error handling with HelperError exceptions
- Context managers for resource cleanup
- Logging for debugging

### Adding New LibreOffice Functions

1. **Implement in helper.py:**
```python
def new_function(param1, param2):
    """Description of function."""
    # Validate parameters
    # Get UNO desktop
    # Perform operation
    # Return result
```

2. **Add to libre.py:**
```python
@server.call_tool()
async def new_function(
    param1: str,
    param2: int
) -> list[TextContent]:
    # Add to queue
    # Return formatted result
```

3. **Update IntentPrompt.txt:**
```
X. new_function - Description for AI
```

4. **Add tests:**
```python
def test_new_function():
    # Test implementation
```

### Pull Request Guidelines
- Include tests for new functionality
- Update documentation (README, claude.md)
- Follow existing code style
- Describe changes in PR description

---

## License

See [libre-office-mcp/LICENSE](libre-office-mcp/LICENSE)

---

## Credits

**libre-office-mcp** builds on implementation by [harshithb3304](https://github.com/harshithb3304/libre-office-mcp)

**Technologies:**
- [LibreOffice](https://www.libreoffice.org/) - Open source office suite
- [Ollama](https://ollama.ai/) - Local AI model runtime
- [Whisper.net](https://github.com/sandrohanea/whisper.net) - .NET bindings for OpenAI Whisper
- [Model Context Protocol](https://modelcontextprotocol.io/) - AI-tool integration standard
- [WinUI 3](https://learn.microsoft.com/en-us/windows/apps/winui/) - Modern Windows UI framework

---

## Quick Reference

### Start LibreOfficeAI
```bash
# After build/publish
LibreOfficeAI.exe
```

### Start MCP Server Standalone
```bash
cd libre-office-mcp/libre-writer
uv run main.py
```

### Configure for Claude Desktop
```json
{
  "mcpServers": {
    "libreoffice-server": {
      "command": "uv",
      "args": ["--directory", "/path/to/libre-writer", "run", "main.py"]
    }
  }
}
```

### Example Prompts
- "Create a document called 'Notes' with a heading and 3 bullet points"
- "Add a table with 4 columns to the Report document"
- "Create a 5-slide presentation about AI"
- "Format the word 'Important' in red and bold"
- "Insert the image at /path/to/image.png into slide 3"
- "Apply the Blue Curve template to my presentation"

---

**Last Updated**: 2025-11-27
**Repository**: SmolPC_LibreOffice
