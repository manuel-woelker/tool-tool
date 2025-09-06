use crate::adapter::{Adapter, AdapterBox};
use crate::configuration::{CONFIGURATION_FILE_NAME, TOOL_TOOL_DIRECTORY, ToolToolConfiguration};
use crate::types::FilePath;

pub struct Workspace {
    config: ToolToolConfiguration,
    adapter: AdapterBox,
}
impl Workspace {
    pub fn new(config: ToolToolConfiguration, adapter: AdapterBox) -> Self {
        Self { config, adapter }
    }

    pub fn config(&self) -> &ToolToolConfiguration {
        &self.config
    }

    pub fn adapter(&self) -> &dyn Adapter {
        self.adapter.as_ref()
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
