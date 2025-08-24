use std::env;
use std::fmt::Debug;
use std::path::PathBuf;
use tool_tool_base::result::{Context, ToolToolResult};
use tool_tool_logic::adapter::Adapter;
use tool_tool_logic::types::FilePath;

pub struct RealAdapter {
    base_path: PathBuf,
}

impl RealAdapter {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn resolve_path(&self, path: &FilePath) -> ToolToolResult<PathBuf> {
        Ok(path.to_path(&self.base_path))
    }
}

impl Adapter for RealAdapter {
    fn args(&self) -> Vec<String> {
        env::args().collect()
    }

    fn env(&self) -> Vec<(String, String)> {
        env::vars().collect()
    }

    fn print(&self, message: &str) {
        eprintln!("{message}");
    }

    fn read_file(&self, path: &FilePath) -> ToolToolResult<String> {
        let physical_path = self.resolve_path(path)?;
        std::fs::read_to_string(&physical_path)
            .with_context(|| format!("Failed to read file {physical_path:?}"))
    }

    fn create_directory_all(&self, path: &FilePath) -> ToolToolResult<()> {
        std::fs::create_dir_all(self.resolve_path(path)?)?;
        Ok(())
    }

    fn exit(&self, exit_code: i32) {
        std::process::exit(exit_code);
    }
}

impl Debug for RealAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RealAdapter")
    }
}
