use crate::adapter::{Adapter, AdapterBox};
use crate::checksums::Checksums;
use crate::configuration::{CONFIGURATION_FILE_NAME, TOOL_TOOL_DIRECTORY, ToolToolConfiguration};
use crate::types::FilePath;
use tool_tool_base::result::ToolToolResult;

pub struct Workspace {
    config: ToolToolConfiguration,
    pub(crate) checksums: Checksums,
    adapter: AdapterBox,
}
impl Workspace {
    pub fn new(config: ToolToolConfiguration, adapter: AdapterBox) -> Self {
        Self {
            config,
            checksums: Checksums::default(),
            adapter,
        }
    }

    pub fn config(&self) -> &ToolToolConfiguration {
        &self.config
    }

    pub fn adapter(&self) -> &dyn Adapter {
        self.adapter.as_ref()
    }

    pub fn checksums(&self) -> &Checksums {
        &self.checksums
    }

    pub fn config_path(&self) -> FilePath {
        FilePath::from(CONFIGURATION_FILE_NAME)
    }
    pub fn tool_tool_dir(&self) -> FilePath {
        FilePath::from(TOOL_TOOL_DIRECTORY)
    }
    pub fn tools_dir(&self) -> FilePath {
        self.tool_tool_dir().join("tools")
    }

    pub fn create_temp_dir(&self, prefix: &str) -> ToolToolResult<FilePath> {
        let random_string = self.adapter.random_string()?;
        let temp_dir = self
            .tool_tool_dir()
            .join(format!("tmp/{prefix}-{random_string}"));
        self.adapter.create_directory_all(&temp_dir)?;
        Ok(temp_dir)
    }
}
