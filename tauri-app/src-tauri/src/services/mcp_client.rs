use crate::models::mcp::*;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub struct McpClient {
    process: Arc<Mutex<Option<Child>>>,
    next_id: AtomicU64,
    tools: Arc<Mutex<HashMap<String, McpTool>>>,
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            next_id: AtomicU64::new(1),
            tools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the MCP server process
    pub fn start(&self, _python_path: &str, mcp_dir: std::path::PathBuf) -> Result<()> {
        // On macOS in development, start only libre.py (expect LibreOffice + helper manual)
        #[cfg(all(target_os = "macos", debug_assertions))]
        {
            log::warn!("macOS dev mode: Starting libre.py only");
            log::info!("Ensure these are running first:");
            log::info!("  1. LibreOffice headless on port 2002");
            log::info!("  2. helper.py on port 8765");

            let libre_script = mcp_dir.join("libre.py");
            let venv_python = mcp_dir.join(".venv/bin/python");

            let python_cmd = if venv_python.exists() {
                venv_python.to_string_lossy().to_string()
            } else {
                "python3".to_string()
            };

            log::info!("Starting libre.py: {} {}", python_cmd, libre_script.display());

            let child = Command::new(&python_cmd)
                .arg(&libre_script)
                .current_dir(&mcp_dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let mut process_guard = self.process.lock().unwrap();
            *process_guard = Some(child);

            std::thread::sleep(std::time::Duration::from_millis(500));
            return Ok(());
        }

        // Production or Windows: Use run.sh wrapper or main.py
        let run_script = mcp_dir.join("run.sh");
        let main_script = mcp_dir.join("main.py");

        let (command, args): (String, Vec<String>) = if run_script.exists() && cfg!(unix) {
            log::info!("Starting MCP server via wrapper: {}", run_script.display());
            (run_script.to_string_lossy().to_string(), vec![])
        } else if main_script.exists() {
            log::info!("Starting MCP server: python3 {}", main_script.display());
            ("python3".to_string(), vec![main_script.to_string_lossy().to_string()])
        } else {
            return Err(anyhow!("MCP server scripts not found at {:?}", mcp_dir));
        };

        let child = Command::new(&command)
            .args(&args)
            .current_dir(&mcp_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut process_guard = self.process.lock().unwrap();
        *process_guard = Some(child);

        // Give the process a moment to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(())
    }

    /// Stop the MCP server process
    pub fn stop(&self) -> Result<()> {
        let mut process_guard = self.process.lock().unwrap();
        if let Some(mut child) = process_guard.take() {
            log::info!("Stopping MCP server");
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }

    /// Check if the MCP server is running
    pub fn is_running(&self) -> bool {
        let process_guard = self.process.lock().unwrap();
        process_guard.is_some()
    }

    /// Initialize the MCP connection
    pub fn initialize(&self) -> Result<()> {
        let params = McpInitParams::default();
        let request = McpRequest::new(
            self.next_id(),
            "initialize".to_string(),
            Some(serde_json::to_value(params)?),
        );

        let response = self.send_request(request)?;

        if let Some(error) = response.error {
            return Err(anyhow!("MCP initialization failed: {}", error.message));
        }

        log::info!("MCP initialized successfully");
        Ok(())
    }

    /// Discover available tools
    pub fn list_tools(&self) -> Result<Vec<McpTool>> {
        let request = McpRequest::new(
            self.next_id(),
            "tools/list".to_string(),
            None,
        );

        let response = self.send_request(request)?;

        if let Some(error) = response.error {
            return Err(anyhow!("Failed to list tools: {}", error.message));
        }

        if let Some(result) = response.result {
            let tools: Vec<McpTool> = serde_json::from_value(
                result.get("tools").cloned().unwrap_or(Value::Array(vec![]))
            )?;

            // Cache the tools
            let mut tools_map = self.tools.lock().unwrap();
            for tool in &tools {
                tools_map.insert(tool.name.clone(), tool.clone());
            }

            log::info!("Discovered {} MCP tools", tools.len());
            Ok(tools)
        } else {
            Ok(vec![])
        }
    }

    /// Invoke a tool
    pub fn call_tool(&self, name: String, arguments: Value) -> Result<ToolResult> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let request = McpRequest::new(
            self.next_id(),
            "tools/call".to_string(),
            Some(params),
        );

        let response = self.send_request(request)?;

        if let Some(error) = response.error {
            return Err(anyhow!("Tool invocation failed: {}", error.message));
        }

        if let Some(result) = response.result {
            let tool_result: ToolResult = serde_json::from_value(result)?;
            Ok(tool_result)
        } else {
            Err(anyhow!("No result from tool invocation"))
        }
    }

    /// Get cached tools
    pub fn get_tools(&self) -> Vec<McpTool> {
        let tools_map = self.tools.lock().unwrap();
        tools_map.values().cloned().collect()
    }

    /// Send a JSON-RPC request and receive response
    fn send_request(&self, request: McpRequest) -> Result<McpResponse> {
        let mut process_guard = self.process.lock().unwrap();

        let child = process_guard.as_mut()
            .ok_or_else(|| anyhow!("MCP server not running"))?;

        let stdin = child.stdin.as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdin"))?;

        let stdout = child.stdout.as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdout"))?;

        // Send request
        let request_json = serde_json::to_string(&request)?;
        log::debug!("Sending MCP request: {}", request_json);

        writeln!(stdin, "{}", request_json)?;
        stdin.flush()?;

        // Read response
        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();
        reader.read_line(&mut response_line)?;

        log::debug!("Received MCP response: {}", response_line);

        let response: McpResponse = serde_json::from_str(&response_line)?;
        Ok(response)
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
