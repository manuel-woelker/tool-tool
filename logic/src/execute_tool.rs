use crate::adapter::ExecutionRequest;
use crate::configuration::platform::DownloadPlatform;
use crate::workspace::Workspace;
use tool_tool_base::result::{ToolToolResult, bail};

pub fn execute_tool(workspace: &mut Workspace) -> ToolToolResult<()> {
    let mut args = workspace.adapter().args();
    // remove the tool-tool binary name
    args.remove(0);
    let command_name = args.remove(0);
    let config = workspace.config();
    let Some(tool_config) = config
        .tools
        .iter()
        .find(|tool| tool.commands.contains_key(&command_name))
    else {
        bail!("No tool found for command '{}'", command_name);
    };
    let command_config = tool_config.commands.get(&command_name).unwrap();
    // TODO: improve extension handling
    let extension = match workspace.adapter().get_platform() {
        DownloadPlatform::Default => unreachable!("Default platform should not be used here"),
        DownloadPlatform::Windows => ".exe",
        DownloadPlatform::Linux => "",
        DownloadPlatform::MacOS => "",
    };
    // TODO: split binary from command arguments
    let binary_path = workspace.tool_tool_dir().join(format!(
        "{}-{}/{}{}",
        tool_config.name, tool_config.version, command_config, extension
    ));
    workspace
        .adapter()
        .execute(ExecutionRequest { binary_path })?;
    Ok(())
}
