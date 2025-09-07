use crate::download;
use std::env;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tool_tool_base::result::{Context, ToolToolResult};
use tool_tool_logic::adapter::{Adapter, ExecutionRequest, ReadSeek};
use tool_tool_logic::configuration::platform::DownloadPlatform;
use tool_tool_logic::types::FilePath;

pub struct RealAdapter {
    base_path: PathBuf,
    downloader: download::Downloader,
}

impl RealAdapter {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            downloader: download::Downloader::new(),
        }
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

    fn read_file(&self, path: &FilePath) -> ToolToolResult<Box<dyn ReadSeek>> {
        let physical_path = self.resolve_path(path)?;
        Ok(Box::new(File::open(&physical_path).with_context(|| {
            format!("Failed to read file {physical_path:?}")
        })?))
    }

    fn create_file(&self, path: &FilePath) -> ToolToolResult<Box<dyn Write>> {
        let physical_path = self.resolve_path(path)?;
        Ok(Box::new(File::create(&physical_path).with_context(
            || format!("Failed to create file {physical_path:?}"),
        )?))
    }

    fn create_directory_all(&self, path: &FilePath) -> ToolToolResult<()> {
        std::fs::create_dir_all(self.resolve_path(path)?)?;
        Ok(())
    }

    fn delete_directory_all(&self, path: &FilePath) -> ToolToolResult<()> {
        std::fs::remove_dir_all(self.resolve_path(path)?)?;
        Ok(())
    }

    fn exit(&self, exit_code: i32) {
        std::process::exit(exit_code);
    }

    fn download_file(&self, url: &str, destination_path: &FilePath) -> ToolToolResult<()> {
        self.downloader
            .download(url, &self.resolve_path(destination_path)?)?;
        Ok(())
    }

    fn get_platform(&self) -> DownloadPlatform {
        #[cfg(target_os = "macos")]
        return DownloadPlatform::Darwin;
        #[cfg(target_os = "linux")]
        return DownloadPlatform::Linux;
        #[cfg(target_os = "windows")]
        return DownloadPlatform::Windows;
    }

    fn execute(&self, _request: ExecutionRequest) -> ToolToolResult<()> {
        todo!()
    }
}

impl Debug for RealAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RealAdapter")
    }
}
