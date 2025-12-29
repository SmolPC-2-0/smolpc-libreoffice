# macOS MCP Testing Workflow

This document provides the workflow for testing MCP integration on macOS using the LibreOffice macro approach.

## Architecture on macOS

Due to macOS security restrictions, we use a different approach than Windows:

1. **LibreOffice Headless** - Runs on port 2002 (UNO socket)
2. **helper.py** - Runs as LibreOffice macro, listens on port 8765
3. **libre.py** - MCP server, communicates with helper.py via socket
4. **Tauri App** - Starts libre.py and connects via stdio

## Testing Steps

### Terminal 1: Start LibreOffice Headless

```bash
/tmp/start_libreoffice.sh
```

You should see it start and stay running (no output is normal).

### Step 2: Start helper.py from LibreOffice GUI

1. Open LibreOffice Writer (normal GUI, not headless)
2. Go to **Tools → Macros → Organize Macros → Python...**
3. In the dialog:
   - Expand "My Macros"
   - You should see "helper"
   - Click on it to see available functions
4. Select any function (or the module itself) and click **Run**

The helper.py server loop will start and you'll see it listening on port 8765.

**Note:** You won't see much output, but the macro is running and waiting for connections.

### Terminal 2: Verify Helper is Running

```bash
# Check if port 8765 is in use
lsof -i :8765
```

You should see something like:
```
COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
soffice   12345  mts   10u  IPv4 0x1234567890      0t0  TCP localhost:8765 (LISTEN)
```

### Terminal 3: Start Tauri App (which starts libre.py)

The app should already be running from your previous session:

```bash
cd /Users/mts/SmolPC_LibreOffice/tauri-app
npm run tauri dev
```

Check the console output - you should see:
- "macOS dev mode: Starting libre.py only"
- "Ensure these are running first: LibreOffice headless on port 2002"
- "Ensure these are running first: helper.py on port 8765"

### Step 3: Test MCP Integration

In the Tauri app chat interface, try these commands:

1. **List available tools:**
   ```
   What MCP tools do you have available?
   ```

2. **Create a test document:**
   ```
   Create a new document called "Test.odt" in ~/Documents with a heading "Test Document" and a paragraph "This is a test."
   ```

3. **List documents:**
   ```
   List all documents in ~/Documents
   ```

## Troubleshooting

### Port 2002 not accessible
```bash
# Kill any existing LibreOffice processes
killall soffice
# Restart from Terminal 1
```

### Port 8765 not in use
- Helper.py macro didn't start properly
- Try running it again from LibreOffice GUI
- Check ~/Library/Application Support/LibreOffice/4/user/Scripts/python/helper.log for errors

### MCP server can't connect to helper
- Verify both ports 2002 and 8765 are listening:
  ```bash
  lsof -i :2002
  lsof -i :8765
  ```

### LibreOffice crashes when running macro
- This shouldn't happen when running from GUI (Tools → Macros)
- If it does, check Console.app for crash logs

## Expected Flow

```
User message in Tauri app
  ↓
Chat UI (Svelte)
  ↓
Ollama (with MCP tools)
  ↓
Tool call detected
  ↓
mcpStore.callTool() (TypeScript)
  ↓
call_mcp_tool command (Rust)
  ↓
McpClient.call_tool() (Rust, stdio)
  ↓
libre.py (MCP server, Python)
  ↓
Socket to localhost:8765
  ↓
helper.py (LibreOffice macro)
  ↓
UNO socket to localhost:2002
  ↓
LibreOffice (headless)
  ↓
Document operations
```

## Logs to Monitor

1. **Tauri console** - Shows MCP client activity
2. **helper.py log** - `~/Library/Application Support/LibreOffice/4/user/Scripts/python/helper.log`
3. **Browser console** (if using web inspector in Tauri) - Shows frontend errors

## Clean Shutdown

1. Stop Tauri app (Ctrl+C or close window)
2. Stop helper.py (close LibreOffice GUI or stop the running macro)
3. Stop LibreOffice headless (Ctrl+C in Terminal 1)
