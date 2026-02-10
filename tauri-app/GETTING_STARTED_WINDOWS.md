# Getting Started - Windows Setup Guide

**Complete setup guide for new Windows users with no dependencies installed.**

This guide assumes you're starting from scratch on a Windows 10 or Windows 11 machine.

---

## What is LibreOffice AI?

LibreOffice AI is a desktop application that lets you create and edit LibreOffice documents and presentations using natural language. Simply chat with the AI and it will create documents, add content, format text, and more.

**Example prompts:**
- "Create a document called Meeting Notes with a heading and bullet points"
- "Make a 5-slide presentation about climate change"
- "Add a table to my report with quarterly sales data"
- "Format the word 'Important' in bold red"

---

## Step 1: Install Prerequisites

You need to install 4 programs before running LibreOffice AI. Follow each section carefully.

### 1.1 Install Node.js

Node.js is required to run the application.

1. Visit: https://nodejs.org/
2. Download the **LTS version** (recommended for most users)
3. Run the installer
   - Accept the license agreement
   - Use default installation options
   - Check "Automatically install the necessary tools" if prompted
4. Restart your computer after installation

**Verify installation:**
Open Command Prompt (press `Win + R`, type `cmd`, press Enter) and run:
```cmd
node --version
npm --version
```

You should see version numbers (e.g., `v20.11.0` and `10.2.4`).

### 1.2 Install Rust

Rust is needed to build the application backend.

1. Visit: https://rustup.rs/
2. Download `rustup-init.exe`
3. Run the installer
   - Choose option **1** (Proceed with standard installation)
   - Wait for installation to complete (may take 5-10 minutes)
4. Close and reopen Command Prompt

**Verify installation:**
```cmd
rustc --version
cargo --version
```

You should see version numbers (e.g., `rustc 1.75.0`).

### 1.3 Install Python

Python 3.12 or newer is required for the document integration.

1. Visit: https://www.python.org/downloads/
2. Download **Python 3.12** or newer
3. **IMPORTANT**: Run the installer and check these boxes:
   - ✅ **"Add python.exe to PATH"** (very important!)
   - ✅ "Install pip"
4. Click "Install Now"
5. Close and reopen Command Prompt

**Verify installation:**
```cmd
python --version
pip --version
```

You should see `Python 3.12.x` or newer.

**Troubleshooting:**
- If `python` command not found, try `python3` instead
- If still not working, you may need to add Python to PATH manually:
  1. Search Windows for "Environment Variables"
  2. Click "Environment Variables" button
  3. Under "System variables", find "Path", click "Edit"
  4. Click "New" and add: `C:\Users\<YourUsername>\AppData\Local\Programs\Python\Python312`
  5. Click OK, restart Command Prompt

### 1.4 Install Ollama

Ollama runs the AI models locally on your computer.

1. Visit: https://ollama.com/download
2. Download **Ollama for Windows**
3. Run the installer
4. Ollama will start automatically (check system tray for Ollama icon)

**Download an AI model:**
Open Command Prompt and run:
```cmd
ollama pull qwen2.5-coder:7b
```

This downloads the qwen2.5-coder model (~4.7 GB). Wait for it to complete. This model supports tool calling which is required for document operations.

**Verify installation:**
```cmd
ollama list
```

You should see `qwen2.5-coder:7b` in the list.

**Alternative models** (optional):
```cmd
ollama pull qwen2.5:1.5b    # ~1 GB, fast on CPU, supports tool calling
ollama pull phi3             # ~2.3 GB, general purpose
ollama pull llama2           # ~3.8 GB, good general purpose
```

**Note:** For document creation features, you need a model that supports Ollama's tool calling format. qwen2.5-coder and qwen2.5 are verified to work.

### 1.5 Install LibreOffice

LibreOffice is needed to create and edit documents.

1. Visit: https://www.libreoffice.org/download/
2. Download **LibreOffice** (latest version)
3. Run the installer
   - Use default installation options
   - Complete the installation
4. You can close LibreOffice after installation - the app will control it

**Verify installation:**
Check if LibreOffice is installed at:
- `C:\Program Files\LibreOffice`
- or `C:\Program Files (x86)\LibreOffice`

---

## Step 2: Download and Setup LibreOffice AI

### 2.1 Get the Code

**Option A: Download from GitHub** (easier)
1. Visit your GitHub repository
2. Click the green "Code" button
3. Click "Download ZIP"
4. Extract the ZIP to a location like `C:\Users\<YourUsername>\Documents\LibreOfficeAI`

**Option B: Clone with Git** (if you have Git installed)
```cmd
cd C:\Users\%USERNAME%\Documents
git clone <your-repo-url>
cd SmolPC_LibreOffice
```

### 2.2 Install Dependencies

Open Command Prompt and navigate to the project:
```cmd
cd C:\Users\%USERNAME%\Documents\SmolPC_LibreOffice\tauri-app
```

Install Node.js dependencies:
```cmd
npm install
```

This will take 2-5 minutes. You'll see many packages being downloaded.

### 2.3 Setup Python MCP Server

The MCP server enables document integration. Set it up:

```cmd
cd resources\mcp_server
```

Install `uv` (Python package manager):
```cmd
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

Close and reopen Command Prompt, then continue:
```cmd
cd C:\Users\%USERNAME%\Documents\SmolPC_LibreOffice\tauri-app\resources\mcp_server

uv init
uv venv
uv add mcp[cli] httpx pillow
```

Go back to the tauri-app directory:
```cmd
cd ..\..
```

---

## Step 3: Run the Application

### 3.1 First Launch

From the `tauri-app` directory:
```cmd
npm run tauri dev
```

**What to expect:**
1. First launch will take 2-5 minutes (compiling Rust code)
2. A loading screen will appear checking dependencies:
   - ✅ Python detection
   - ✅ Ollama detection
   - ✅ LibreOffice detection
   - ✅ MCP Server startup
3. Once all dependencies are ready, the chat interface appears

**If something shows ❌ (not found):**
- Check the installation steps above
- See Troubleshooting section below

### 3.2 First-Time Usage

**Configure Settings:**
1. Click the settings button (⚙️) in the top-right corner
2. Verify the settings:
   - **Ollama URL**: Should be `http://localhost:11434`
   - **Selected Model**: Choose `qwen2.5-coder:7b` (or another model you downloaded)
   - **Python Path**: Should auto-detect (e.g., `C:\Users\<YourUsername>\AppData\Local\Programs\Python\Python312\python.exe`)
   - **Documents Path**: Default is `C:\Users\<YourUsername>\Documents`
   - **Theme**: Dark (default)
3. Click "Save Settings"

**Test the Chat:**
1. Close settings (click X)
2. In the chat, type: "Hello, can you help me create documents?"
3. Press Enter
4. You should see a streaming response from the AI

**Create Your First Document:**
1. Send: "Create a document called 'My First Document' in my Documents folder"
2. The AI will create the document
3. Check your Documents folder - you should see `My First Document.odt`
4. Open it in LibreOffice to verify it was created

---

## Step 4: Basic Usage

### Chat Interface

**Send a message:**
- Type in the text box at the bottom
- Press `Enter` to send
- Press `Shift + Enter` for a new line

**Example commands:**

**Create documents:**
```
Create a document called "Meeting Notes" with a heading "Team Standup"
```

**Add content:**
```
Add a paragraph to Meeting Notes saying "Discussed project timeline and deliverables"
```

**Format text:**
```
Make the word "Discussed" bold in Meeting Notes
```

**Create presentations:**
```
Create a presentation called "Sales Pitch" with 5 slides about our product features
```

**Add tables:**
```
Add a 3x4 table to Meeting Notes with headers "Task", "Owner", "Status"
```

### Settings

**Access settings:**
- Click the ⚙️ button in the top-right

**Available settings:**
- **Ollama Configuration**: Change AI model, adjust temperature (creativity level)
- **Paths**: Set custom document folder, Python path, LibreOffice location
- **Appearance**: Choose theme (currently only dark theme)
- **Advanced**: Custom system prompt, max tokens

**Save changes:**
- Click "Save Settings" button
- Settings persist between app restarts

**Reset to defaults:**
- Click "Reset to Defaults" button
- Click "Save Settings" to confirm

---

## Troubleshooting

### App won't start

**Error: "node is not recognized"**
- Node.js not installed or not in PATH
- Reinstall Node.js and check "Add to PATH"
- Restart Command Prompt

**Error: "rustc is not recognized"**
- Rust not installed
- Install from https://rustup.rs/
- Restart Command Prompt

**Error: "npm install failed"**
- Delete `node_modules` folder
- Run `npm install` again
- Check internet connection

### Dependencies not detected

**Python shows ❌**
- Verify Python 3.12+ is installed: `python --version`
- Check Python is in PATH
- Try setting Python path manually in settings

**Ollama shows ❌**
- Check Ollama is running (system tray icon)
- Verify: http://localhost:11434 in browser (should show "Ollama is running")
- Restart Ollama from Start menu

**LibreOffice shows ❌**
- Check LibreOffice is installed in `C:\Program Files\LibreOffice`
- Try setting LibreOffice path manually in settings
- Reinstall LibreOffice if needed

**MCP Server shows ❌**
- Check Python is working
- Verify MCP server dependencies:
  ```cmd
  cd resources\mcp_server
  .venv\Scripts\activate
  pip list
  ```
- Should show: mcp, httpx, pillow
- If missing, run: `uv add mcp[cli] httpx pillow`

### Chat not working

**No response when sending messages**
- Check Ollama is running
- Verify model is downloaded: `ollama list`
- Check Ollama URL in settings: `http://localhost:11434`
- Try selecting a different model in settings

**Response is very slow**
- Models run better on powerful computers (16GB+ RAM recommended)
- Try a smaller model: `ollama pull phi3` (~2.3 GB)
- Close other programs to free up RAM

**Error: "Model not found"**
- Download the model: `ollama pull phi3`
- Select it in settings dropdown
- Save settings and try again

### Document creation not working

**Documents not being created**
- Check Documents path in settings
- Verify LibreOffice is detected (green checkmark)
- Check MCP server is running (green checkmark)
- Look for error messages in chat

**Can't open created documents**
- Install LibreOffice if not already installed
- Right-click document → Open with → LibreOffice Writer
- Check file extension (.odt for documents, .odp for presentations)

**Path errors**
- Use full paths: `C:\Users\YourName\Documents\test.odt`
- Avoid special characters in filenames
- Don't use spaces without quotes

### Performance issues

**App is slow to start**
- First launch compiles Rust code (2-5 minutes) - this is normal
- Subsequent launches are faster (~30 seconds)
- Consider building a release version: `npm run tauri build`

**Chat responses are slow**
- Models require significant RAM and CPU
- Recommended: 16GB RAM, modern CPU
- Try smaller models (phi3 vs llama2)
- Lower temperature setting for faster responses

**High memory usage**
- Normal for AI models (4-8 GB typical)
- Close other programs
- Use smaller models
- Restart app periodically

### Getting help

**Check logs:**
- Command Prompt window shows real-time logs
- Look for error messages in red
- Frontend logs: Press `F12` in app window → Console tab

**Common error messages:**
- "Connection refused": Ollama not running
- "Model not found": Download model with `ollama pull`
- "Permission denied": Run as Administrator
- "Port already in use": Close other instances

**Still stuck?**
- Check existing issues on GitHub
- Create a new issue with:
  - Windows version (10/11)
  - Error messages
  - Steps you took
  - Console logs

---

## Next Steps

### Explore Features

**Try different prompts:**
```
Create a resume template
Make a grocery shopping list
Create a presentation about renewable energy
Add a professional header to my document
Insert a table with last quarter's sales data
```

**Experiment with settings:**
- Try different AI models
- Adjust temperature (0.1 = focused, 1.5 = creative)
- Set custom document folders
- Write your own system prompt

### Advanced Usage

**Model Selection:**
- **qwen2.5:1.5b**: Fast, supports tool calling, great for CPU-only machines (1 GB)
- **qwen2.5-coder:7b**: Best for document operations, supports tool calling (4.7 GB)
- **phi3**: General purpose, fast (2.3 GB)
- **llama2**: Larger, more capable for general chat (3.8 GB)

Download more models:
```cmd
ollama pull <model-name>
```

**Custom System Prompts:**
Add instructions to guide AI behavior:
```
You are a professional business writer.
Always use formal language and proper formatting.
Include headers and bullet points in all documents.
```

**Document Organization:**
- Create dedicated folders for different projects
- Set custom Documents path in settings
- Use descriptive filenames

### Building Production Version

For a faster, standalone executable:

```cmd
npm run tauri build
```

The installer will be created in:
```
tauri-app\src-tauri\target\release\bundle\
```

Install the `.exe` or `.msi` file on any Windows machine (dependencies still required).

---

## System Requirements

**Minimum:**
- Windows 10 (version 1809+) or Windows 11
- 8 GB RAM
- 4-core CPU
- 10 GB free disk space
- Internet connection (for initial setup)

**Recommended:**
- Windows 11
- 16 GB RAM
- Modern CPU (Intel i5/i7, AMD Ryzen 5/7)
- 20 GB free disk space
- SSD for faster performance

**Disk Space Breakdown:**
- Node.js: ~200 MB
- Rust: ~2 GB
- Python: ~100 MB
- LibreOffice: ~1 GB
- Ollama + models: ~5-10 GB (depends on models)
- Project dependencies: ~500 MB

---

## What Gets Installed Where

**Program Files:**
- Node.js: `C:\Program Files\nodejs\`
- Python: `C:\Users\<You>\AppData\Local\Programs\Python\Python312\`
- Rust: `C:\Users\<You>\.cargo\` and `C:\Users\<You>\.rustup\`
- Ollama: `C:\Users\<You>\AppData\Local\Programs\Ollama\`
- LibreOffice: `C:\Program Files\LibreOffice\`

**Project Files:**
- App code: `C:\Users\<You>\Documents\SmolPC_LibreOffice\`
- Node modules: `SmolPC_LibreOffice\tauri-app\node_modules\`
- Python venv: `SmolPC_LibreOffice\tauri-app\resources\mcp_server\.venv\`

**Data/Config:**
- App settings: `C:\Users\<You>\AppData\Roaming\com.libreoffice.ai\config.json`
- Ollama models: `C:\Users\<You>\.ollama\models\`
- Documents: `C:\Users\<You>\Documents\` (configurable)

---

## Uninstalling

If you want to remove everything:

1. **Uninstall programs** (Settings → Apps):
   - Node.js
   - Python 3.12
   - Ollama
   - LibreOffice

2. **Remove Rust:**
   ```cmd
   rustup self uninstall
   ```

3. **Delete project folder:**
   ```cmd
   rmdir /s "C:\Users\%USERNAME%\Documents\SmolPC_LibreOffice"
   ```

4. **Delete config files:**
   - `C:\Users\<You>\AppData\Roaming\com.libreoffice.ai\`
   - `C:\Users\<You>\.ollama\` (if you want to remove downloaded models)

---

## FAQ

**Q: Do I need internet after setup?**
A: No, everything runs locally. Internet only needed for initial downloads.

**Q: Is my data sent to the cloud?**
A: No, all AI processing happens on your computer. No data is sent anywhere.

**Q: How much does this cost?**
A: Everything is free and open source. No subscriptions or fees.

**Q: Can I use this offline?**
A: Yes, once everything is installed, it works completely offline.

**Q: Which AI model should I use?**
A: Start with `qwen2.5-coder:7b` for best document tool support. On slower machines, try `qwen2.5:1.5b` (faster but less capable).

**Q: Can I use Microsoft Office files?**
A: Yes! LibreOffice supports .docx, .pptx, .xlsx, etc.

**Q: Will this slow down my computer?**
A: While running, it uses 4-8 GB RAM. Close when not needed.

**Q: Can I use ChatGPT instead of Ollama?**
A: Not currently. This app is designed for local, private AI.

**Q: What if LibreOffice isn't detected?**
A: Manually set the path in settings to `C:\Program Files\LibreOffice\program\soffice.exe`

**Q: Can I change where documents are saved?**
A: Yes, set custom Documents path in settings (⚙️).

---

## Tips & Tricks

**Faster startup:**
- Build release version: `npm run tauri build`
- Use smaller AI models (phi3 instead of llama2)
- Close LibreOffice before starting the app

**Better responses:**
- Be specific in your requests
- Mention the document name explicitly
- Use natural language like you're talking to a person

**Organize documents:**
- Create project folders: `C:\Projects\Marketing\`
- Set as Documents path in settings
- All new documents go there automatically

**Multiple models:**
- Download several models with Ollama
- Switch between them in settings
- Use coding models for technical docs
- Use general models for creative writing

**Keyboard shortcuts:**
- `Enter`: Send message
- `Shift + Enter`: New line in message
- `F12`: Open developer tools (for debugging)

---

**Last Updated**: 2026-02-10
**Version**: Week 4 - End-to-end tool calling working
**Support**: Create an issue on GitHub for help

---

**Enjoy using LibreOffice AI!** 🚀
