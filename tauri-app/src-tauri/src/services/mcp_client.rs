use crate::models::mcp::*;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct McpClient {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout_reader: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    next_id: AtomicU64,
    tools: Arc<Mutex<HashMap<String, McpTool>>>,
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout_reader: Arc::new(Mutex::new(None)),
            next_id: AtomicU64::new(1),
            tools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Extract stdin/stdout from child process and store them persistently.
    fn store_child_io(&self, child: &mut Child) -> Result<()> {
        let child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to take stdin from child process"))?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to take stdout from child process"))?;

        let mut stdin_guard = self.stdin.lock().unwrap();
        *stdin_guard = Some(child_stdin);

        let mut reader_guard = self.stdout_reader.lock().unwrap();
        *reader_guard = Some(BufReader::new(child_stdout));

        Ok(())
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

            log::info!(
                "Starting libre.py: {} {}",
                python_cmd,
                libre_script.display()
            );

            let mut child = Command::new(&python_cmd)
                .arg(&libre_script)
                .current_dir(&mcp_dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            self.store_child_io(&mut child)?;

            let mut process_guard = self.process.lock().unwrap();
            *process_guard = Some(child);

            std::thread::sleep(Duration::from_millis(500));
            return Ok(());
        }

        #[cfg(not(all(target_os = "macos", debug_assertions)))]
        {
            // Production or Windows: Use run.sh wrapper or main.py
            let run_script = mcp_dir.join("run.sh");
            let main_script = mcp_dir.join("main.py");

            let (command, args): (String, Vec<String>) = if run_script.exists() && cfg!(unix) {
                log::info!("Starting MCP server via wrapper: {}", run_script.display());
                (run_script.to_string_lossy().to_string(), vec![])
            } else if main_script.exists() {
                // Try venv python first, then fall back to system python
                let venv_python = if cfg!(target_os = "windows") {
                    mcp_dir.join(".venv/Scripts/python.exe")
                } else {
                    mcp_dir.join(".venv/bin/python")
                };
                let python_cmd = if venv_python.exists() {
                    venv_python.to_string_lossy().to_string()
                } else if cfg!(target_os = "windows") {
                    "python".to_string()
                } else {
                    "python3".to_string()
                };
                log::info!(
                    "Starting MCP server: {} {}",
                    python_cmd,
                    main_script.display()
                );
                (python_cmd, vec![main_script.to_string_lossy().to_string()])
            } else {
                return Err(anyhow!("MCP server scripts not found at {:?}", mcp_dir));
            };

            let mut child = Command::new(&command)
                .args(&args)
                .current_dir(&mcp_dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            self.store_child_io(&mut child)?;

            let mut process_guard = self.process.lock().unwrap();
            *process_guard = Some(child);

            // Short initial wait — retry logic in initialize() handles the rest
            std::thread::sleep(Duration::from_millis(3000));

            Ok(())
        }
    }

    /// Stop the MCP server process
    pub fn stop(&self) -> Result<()> {
        // Close stdin first (signals EOF to child)
        {
            let mut stdin_guard = self.stdin.lock().unwrap();
            *stdin_guard = None;
        }
        // Close stdout reader
        {
            let mut reader_guard = self.stdout_reader.lock().unwrap();
            *reader_guard = None;
        }
        // Kill process
        let mut process_guard = self.process.lock().unwrap();
        if let Some(mut child) = process_guard.take() {
            log::info!("Stopping MCP server");
            let _ = child.kill(); // Ignore error if already exited
            child.wait()?;
        }
        Ok(())
    }

    /// Check if the MCP server is running
    pub fn is_running(&self) -> bool {
        let mut process_guard = self.process.lock().unwrap();
        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(Some(_)) => {
                    log::warn!("MCP server process has exited unexpectedly");
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Initialize the MCP connection with retry logic
    pub fn initialize(&self) -> Result<()> {
        let params = McpInitParams::default();
        let max_attempts = 10;
        let retry_delay = Duration::from_millis(2000);
        let start_time = Instant::now();
        let max_wait = Duration::from_secs(30);

        for attempt in 1..=max_attempts {
            if start_time.elapsed() > max_wait {
                return Err(anyhow!("MCP initialization timed out after {:?}", max_wait));
            }

            let request = McpRequest::new(
                self.next_id(),
                "initialize".to_string(),
                Some(serde_json::to_value(&params)?),
            );

            match self.send_request(request) {
                Ok(response) => {
                    if let Some(error) = response.error {
                        return Err(anyhow!("MCP initialization failed: {}", error.message));
                    }
                    log::info!(
                        "MCP initialized successfully on attempt {} ({:?} elapsed)",
                        attempt,
                        start_time.elapsed()
                    );
                    return Ok(());
                }
                Err(e) => {
                    log::warn!(
                        "MCP initialize attempt {}/{} failed: {}. Retrying in {:?}...",
                        attempt,
                        max_attempts,
                        e,
                        retry_delay
                    );
                    if attempt < max_attempts {
                        std::thread::sleep(retry_delay);
                    }
                }
            }
        }

        Err(anyhow!(
            "MCP initialization failed after {} attempts",
            max_attempts
        ))
    }

    /// Discover available tools
    pub fn list_tools(&self) -> Result<Vec<McpTool>> {
        let request = McpRequest::new(self.next_id(), "tools/list".to_string(), None);

        let response = self.send_request(request)?;

        if let Some(error) = response.error {
            return Err(anyhow!("Failed to list tools: {}", error.message));
        }

        if let Some(result) = response.result {
            let tools: Vec<McpTool> = serde_json::from_value(
                result.get("tools").cloned().unwrap_or(Value::Array(vec![])),
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

        let request = McpRequest::new(self.next_id(), "tools/call".to_string(), Some(params));

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
        // Write request to stdin
        {
            let mut stdin_guard = self.stdin.lock().unwrap();
            let stdin = stdin_guard
                .as_mut()
                .ok_or_else(|| anyhow!("MCP server stdin not available"))?;

            let request_json = serde_json::to_string(&request)?;
            log::debug!("Sending MCP request: {}", request_json);

            writeln!(stdin, "{}", request_json)?;
            stdin.flush()?;
        } // stdin lock released

        // Read response from stdout with timeout
        let reader_arc = Arc::clone(&self.stdout_reader);
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let mut reader_guard = reader_arc.lock().unwrap();
            if let Some(reader) = reader_guard.as_mut() {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(_) => {
                        let _ = tx.send(Ok(line));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                }
            } else {
                let _ = tx.send(Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "MCP stdout reader not available",
                )));
            }
        });

        let timeout = Duration::from_secs(30);
        let response_line = match rx.recv_timeout(timeout) {
            Ok(Ok(line)) if line.trim().is_empty() => {
                return Err(anyhow!("MCP server closed stdout (empty response)"));
            }
            Ok(Ok(line)) => line,
            Ok(Err(e)) => return Err(anyhow!("Failed to read MCP response: {}", e)),
            Err(_) => return Err(anyhow!("MCP server response timed out after {:?}", timeout)),
        };

        log::debug!("Received MCP response: {}", response_line.trim());
        let response: McpResponse = serde_json::from_str(response_line.trim())?;
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
