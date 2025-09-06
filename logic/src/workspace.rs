use crate::adapter::Adapter;
use crate::configuration::{CONFIGURATION_FILE_NAME, TOOL_TOOL_DIRECTORY, ToolToolConfiguration};
use crate::types::FilePath;

pub struct Workspace<'a> {
    config: &'a ToolToolConfiguration,
    adapter: &'a dyn Adapter,
}
impl<'a> Workspace<'a> {
    pub fn new(config: &'a ToolToolConfiguration, adapter: &'a dyn Adapter) -> Self {
        Self { config, adapter }
    }

    pub fn config(&self) -> &'a ToolToolConfiguration {
        self.config
    }

    pub fn adapter(&self) -> &'a dyn Adapter {
        self.adapter
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
}
