use crate::adapter::ExecutionRequest;
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
    let tool_path = workspace
        .tool_tool_dir()
        .join(format!("{}-{}", tool_config.name, tool_config.version));
    let binary_path = tool_path.join(command_config);
    workspace
        .adapter()
        .execute(ExecutionRequest { binary_path })?;
    Ok(())
}
