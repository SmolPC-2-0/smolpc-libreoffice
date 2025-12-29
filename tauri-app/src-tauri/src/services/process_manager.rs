use parking_lot::RwLock;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;

pub struct ProcessManager {
    ollama_process: Arc<RwLock<Option<Child>>>,
    mcp_process: Arc<RwLock<Option<Child>>>,
    libreoffice_process: Arc<RwLock<Option<Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            ollama_process: Arc::new(RwLock::new(None)),
            mcp_process: Arc::new(RwLock::new(None)),
            libreoffice_process: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the MCP server process
    pub fn start_mcp_server(&self, python_path: &str, mcp_dir: PathBuf) -> Result<()> {
        // Check if already running
        if self.mcp_process.read().is_some() {
            return Ok(());
        }

        let main_py = mcp_dir.join("main.py");

        if !main_py.exists() {
            return Err(anyhow::anyhow!("MCP server main.py not found at {:?}", main_py));
        }

        let child = Command::new(python_path)
            .arg(main_py)
            .current_dir(&mcp_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        *self.mcp_process.write() = Some(child);
        Ok(())
    }

    /// Check if MCP server is running
    pub fn is_mcp_running(&self) -> bool {
        if let Some(ref mut process) = *self.mcp_process.write() {
            match process.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    *self.mcp_process.write() = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Stop the MCP server
    pub fn stop_mcp_server(&self) -> Result<()> {
        if let Some(mut process) = self.mcp_process.write().take() {
            process.kill()?;
            process.wait()?;
        }
        Ok(())
    }

    pub fn cleanup(&self) {
        if let Some(mut process) = self.ollama_process.write().take() {
            let _ = process.kill();
        }
        if let Some(mut process) = self.mcp_process.write().take() {
            let _ = process.kill();
        }
        if let Some(mut process) = self.libreoffice_process.write().take() {
            let _ = process.kill();
        }
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
