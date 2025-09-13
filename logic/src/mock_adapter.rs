use crate::adapter::{Adapter, ExecutionRequest, ReadSeek};
use crate::configuration::CONFIGURATION_FILE_NAME;
use crate::configuration::platform::DownloadPlatform;
use crate::types::FilePath;
use expect_test::Expect;
use indent::indent_all_with;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tool_tool_base::result::{ToolToolResult, err};

#[derive(Clone)]
pub struct MockAdapter {
    inner: Arc<RwLock<MockAdapterInner>>,
}

struct MockAdapterInner {
    args: Vec<String>,
    env: Vec<(String, String)>,
    effects_string: String,
    platform: DownloadPlatform,
    url_map: HashMap<String, Vec<u8>>,
    file_map: HashMap<FilePath, Vec<u8>>,
    exit_code: i32,
}

impl MockAdapter {
    pub fn new() -> Self {
        let mut file_map = HashMap::new();
        file_map.insert(
            FilePath::from(CONFIGURATION_FILE_NAME),
            r##"
                    tools {
                        lsd "1.2.3" {
                            download {
                                linux "https://example.com/test-1.2.3.tar.gz"
                                windows "https://example.com/test-1.2.3.zip"
                            }
                            commands {
                                foobar "echo foobar"
                                bar "fizz buzz"
                                tooly "tooly"
                                toolyv "tooly -v"
                                toolyhi #"tooly "Hello World!""#
                            }
                            env {
                                FROBNIZZ "nizzle"
                                FIZZ "buzz"
                            }
                       }
                    }
                       "##
            .as_bytes()
            .to_vec(),
        );
        Self {
            inner: Arc::new(RwLock::new(MockAdapterInner {
                env: vec![("NO_COLOR".to_string(), "1".to_string())],
                args: Vec::new(),
                platform: DownloadPlatform::Linux,
                url_map: HashMap::new(),
                file_map,
                effects_string: String::new(),
                exit_code: 0,
            })),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, MockAdapterInner> {
        self.inner
            .read()
            .expect("Unable to acquire read lock for mock adapter")
    }

    fn write(&self) -> RwLockWriteGuard<'_, MockAdapterInner> {
        self.inner
            .write()
            .expect("Unable to acquire write lock for mock adapter")
    }

    pub(crate) fn log_effect(&self, effect: impl AsRef<str>) {
        self.write().effects_string.push_str(effect.as_ref());
        self.write().effects_string.push('\n');
    }

    pub fn set_args(&self, args: &[&str]) {
        let mut all_args = vec!["./tool-tool.exe".to_string()];
        all_args.extend(args.iter().map(|s| s.to_string()));
        self.write().args = all_args;
    }

    pub fn set_configuration(&self, configuration: impl Into<String>) {
        self.set_file(CONFIGURATION_FILE_NAME, configuration.into().into_bytes());
    }

    pub fn set_platform(&self, platform: DownloadPlatform) {
        self.write().platform = platform;
    }

    pub fn set_url(&self, url: &str, content: Vec<u8>) {
        self.write().url_map.insert(url.to_string(), content);
    }

    pub fn set_file(&self, file_path: &str, content: impl Into<Vec<u8>>) {
        self.write()
            .file_map
            .insert(FilePath::from(file_path), content.into());
    }

    pub fn verify_effects(&self, expected: Expect) {
        expected.assert_eq(&self.read().effects_string);
        self.write().effects_string.clear();
    }

    pub fn set_exit_code(&self, exit_code: i32) {
        self.write().exit_code = exit_code;
    }

    #[allow(dead_code)]
    pub fn get_effects(&self) -> String {
        self.read().effects_string.clone()
    }
}

impl Adapter for MockAdapter {
    fn args(&self) -> Vec<String> {
        self.read().args.clone()
    }

    fn env(&self) -> Vec<(String, String)> {
        self.read().env.clone()
    }

    fn print(&self, message: &str) {
        self.log_effect(format!("PRINT:\n{}", indent_all_with("\t", message)));
    }

    fn file_exists(&self, path: &FilePath) -> ToolToolResult<bool> {
        self.log_effect(format!("FILE EXISTS?:\n{}", path));
        Ok(self.read().file_map.contains_key(path))
    }

    fn read_file(&self, path: &FilePath) -> ToolToolResult<Box<dyn ReadSeek>> {
        self.log_effect(format!("READ FILE: {path}"));
        Ok(Box::new(Cursor::new(
            self.read()
                .file_map
                .get(path)
                .ok_or_else(|| err!("File '{path}' does not exist"))?
                .clone(),
        )))
    }

    fn create_file(&self, path: &FilePath) -> ToolToolResult<Box<dyn Write>> {
        self.log_effect(format!("CREATE FILE: {path}"));
        Ok(Box::new(MockFile::new(path, self.clone())))
    }

    fn create_directory_all(&self, path: &FilePath) -> ToolToolResult<()> {
        self.log_effect(format!("CREATE DIR: {path}"));
        Ok(())
    }

    fn delete_directory_all(&self, path: &FilePath) -> ToolToolResult<()> {
        self.log_effect(format!("DELETE DIR: {path}"));
        Ok(())
    }

    fn exit(&self, exit_code: i32) {
        self.log_effect(format!("EXIT: {}", exit_code));
    }

    fn download_file(&self, url: &str, destination_path: &FilePath) -> ToolToolResult<()> {
        self.log_effect(format!("DOWNLOAD: {url} -> {destination_path}"));
        let content = self
            .read()
            .url_map
            .get(url)
            .ok_or_else(|| err!("URL '{url}' does not exist"))?
            .clone();
        self.write()
            .file_map
            .insert(destination_path.clone(), content);
        Ok(())
    }

    fn get_platform(&self) -> DownloadPlatform {
        self.read().platform
    }

    fn execute(&self, request: ExecutionRequest) -> ToolToolResult<i32> {
        self.log_effect(format!("EXECUTE: {}", request.binary_path));
        for arg in request.args {
            self.log_effect(format!("\tARG: {arg}"));
        }
        for env in request.env {
            self.log_effect(format!("\tENV: {}={}", env.key, env.value));
        }
        Ok(self.read().exit_code)
    }
}

impl std::fmt::Debug for MockAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockAdapter")
    }
}

struct MockFile {
    path: FilePath,
    data: Vec<u8>,
    mock_adapter: MockAdapter,
}

impl MockFile {
    fn new(path: &FilePath, mock_adapter: MockAdapter) -> Self {
        Self {
            path: path.clone(),
            data: vec![],
            mock_adapter,
        }
    }
}

impl Write for MockFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for MockFile {
    fn drop(&mut self) {
        self.mock_adapter.log_effect(format!(
            "WRITE FILE: {} -> {}",
            self.path,
            String::from_utf8_lossy(&self.data)
        ));
        self.mock_adapter
            .write()
            .file_map
            .insert(self.path.clone(), self.data.clone());
    }
}
