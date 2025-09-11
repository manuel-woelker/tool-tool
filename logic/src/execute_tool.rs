use crate::adapter::ExecutionRequest;
use crate::workspace::Workspace;
use shellish_parse::ParseOptions;
use tool_tool_base::result::{Context, ToolToolResult, bail};

pub fn execute_tool(workspace: &mut Workspace) -> ToolToolResult<()> {
    let mut command_args = workspace.adapter().args();
    // remove the tool-tool binary name
    command_args.remove(0);
    let command_name = command_args.remove(0);
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
    let extensions = workspace
        .adapter()
        .get_platform()
        .get_executable_extensions();
    // TODO: handle whitespace in strings
    let mut parsed_command = shellish_parse::parse(command_config, ParseOptions::new())?;
    let binary = parsed_command.remove(0);
    let tool_path = workspace
        .tool_tool_dir()
        .join(format!("{}-{}", tool_config.name, tool_config.version));
    let mut binary_path_maybe = None;
    let mut errors = vec![];
    'extension_loop: for extension in extensions {
        let candidate = tool_path.join(format!("{binary}{extension}"));
        match workspace.adapter().file_exists(&candidate) {
            Ok(true) => {
                binary_path_maybe = Some(candidate);
                break 'extension_loop;
            }
            Ok(false) => { /* do nothing */ }
            Err(err) => {
                errors.push(err);
            }
        }
    }
    let Some(binary_path) = binary_path_maybe else {
        if errors.is_empty() {
            bail!(
                "Failed to find binary for command '{command_name}' in tool {}, found no matching executable binaries: {}({})",
                tool_config.name,
                tool_path.join(binary),
                extensions.join("|")
            );
        } else {
            return Err(errors.remove(0)).with_context(|| {
                format!(
                    "Failed to find binary for command '{command_name}' in tool {}",
                    tool_config.name
                )
            });
        }
    };
    let mut args = parsed_command;
    args.extend(command_args);
    let env = tool_config.env.clone();
    workspace.adapter().execute(ExecutionRequest {
        binary_path,
        args,
        env,
    })?;
    Ok(())
}
