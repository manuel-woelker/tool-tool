use crate::download;
use rand::Rng;
use rand::distr::Alphanumeric;
use std::env;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tool_tool_base::result::{Context, ToolToolResult};
use tool_tool_logic::adapter::{Adapter, ExecutionRequest, ReadSeek};
use tool_tool_logic::configuration::platform::DownloadPlatform;
use tool_tool_logic::types::{EnvPair, FilePath};

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

    fn file_exists(&self, path: &FilePath) -> ToolToolResult<bool> {
        let physical_path = self.resolve_path(path)?;
        Ok(physical_path.exists())
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
        std::fs::remove_dir_all(self.resolve_path(path)?)
            .with_context(|| format!("Failed to delete directory {path:?}"))?;
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

    fn execute(&self, request: ExecutionRequest) -> ToolToolResult<i32> {
        let path = self.resolve_path(&request.binary_path)?;
        let mut command = Command::new(path);
        command.args(request.args);
        // Start with a clean environment to prevent user envs impacting the execution
        command.env_clear();
        for EnvPair { key, value } in request.env {
            command.env(key, value);
        }
        let status = command.status()?;
        Ok(status.code().unwrap_or(255))
    }

    fn random_string(&self) -> ToolToolResult<String> {
        let random_string: String = rand::rng()
            .sample_iter(Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        Ok(random_string)
    }
}

impl Debug for RealAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RealAdapter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use test_temp_dir::{TestTempDir, test_temp_dir};

    struct TestContext {
        temp_dir: TestTempDir,
        adapter: RealAdapter,
    }

    fn setup() -> TestContext {
        let temp_dir = test_temp_dir!();
        let server = MockServer::start();
        let content = "download content";
        server.mock(|when, then| {
            when.method(GET).path("/download_url");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body(content);
        });
        let adapter = RealAdapter::new(temp_dir.as_path_untracked().to_path_buf());
        TestContext { temp_dir, adapter }
    }

    fn create_adapter_in_current_directory() -> RealAdapter {
        RealAdapter::new(PathBuf::from("."))
    }

    #[test]
    fn test_file_exists() {
        let adapter = create_adapter_in_current_directory();
        assert!(adapter.file_exists(&FilePath::from("Cargo.toml")).unwrap());
        assert!(
            !adapter
                .file_exists(&FilePath::from("non_existent_file"))
                .unwrap()
        );
    }

    #[test]
    fn test_read_file() {
        let adapter = create_adapter_in_current_directory();
        let mut file = adapter.read_file(&FilePath::from("Cargo.toml")).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains("workspace"));
    }

    #[test]
    fn test_create_file() {
        let context = setup();
        let file_path = "test.txt";
        let mut file = context
            .adapter
            .create_file(&FilePath::from(file_path))
            .unwrap();
        file.write_all(b"test").unwrap();
        file.flush().unwrap();
        drop(file);
        let actual =
            std::fs::read_to_string(context.temp_dir.as_path_untracked().join(file_path)).unwrap();
        assert_eq!(actual, "test");
    }

    #[test]
    fn create_directory_all() {
        let context = setup();
        let file_path = "foo/bar/baz";
        context
            .adapter
            .create_directory_all(&FilePath::from(file_path))
            .unwrap();
        // second time
        context
            .adapter
            .create_directory_all(&FilePath::from(file_path))
            .unwrap();

        let path = context.temp_dir.as_path_untracked().join(file_path);
        assert!(std::path::PathBuf::from(&path).exists());
        assert!(std::path::PathBuf::from(&path).is_dir());
    }

    #[test]
    fn delete_directory_all() {
        let context = setup();
        let file_path = "foo/bar/baz";
        let path = context.temp_dir.as_path_untracked().join(file_path);
        std::fs::create_dir_all(&path.join("fizzbuzz")).unwrap();
        context
            .adapter
            .delete_directory_all(&FilePath::from(file_path))
            .unwrap();

        let path = context.temp_dir.as_path_untracked().join(file_path);
        assert!(!std::path::PathBuf::from(&path).exists());
        assert!(
            std::path::PathBuf::from(context.temp_dir.as_path_untracked().join("foo/bar")).exists()
        );
    }

    #[test]
    fn random_string() {
        let adapter = create_adapter_in_current_directory();
        let random_string = adapter.random_string().unwrap();
        assert_eq!(random_string.len(), 16);
    }
}
