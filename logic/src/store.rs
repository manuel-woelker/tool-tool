use crate::adapter::Adapter;
use crate::configuration::ToolToolConfiguration;
use crate::types::FilePath;

pub struct Store<'a> {
    config: &'a ToolToolConfiguration,
    adapter: &'a dyn Adapter,
}
impl<'a> Store<'a> {
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
        FilePath::from(".tool-tool.v2.kdl")
    }
    pub fn tool_tool_dir(&self) -> FilePath {
        FilePath::from(".tool-tool/v2/tools")
    }
    pub fn tools_dir(&self) -> FilePath {
        self.tool_tool_dir().join("tools")
    }
}
