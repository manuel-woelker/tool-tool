use crate::adapter::Adapter;
use crate::configuration::ToolToolConfiguration;
use crate::types::FilePath;
use tool_tool_base::result::ToolToolResult;

pub fn run_download_task(
    adapter: &dyn Adapter,
    _config: ToolToolConfiguration,
) -> ToolToolResult<()> {
    // create .tool-tool directory if it doesn't exist
    let tool_tool_dir = FilePath::from(".tool-tool/v2/tools");
    adapter.create_directory_all(&tool_tool_dir)?;
    Ok(())
}
