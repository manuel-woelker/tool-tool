use crate::adapter::ExecutionRequest;
use crate::configuration::find_command;
use crate::lock_guard::LockGuard;
use crate::workspace::Workspace;
use shellish_parse::ParseOptions;
use std::time::Duration;
use tool_tool_base::result::{Context, ToolToolResult, bail};

pub fn run_command(workspace: &mut Workspace) -> ToolToolResult<()> {
    let mut command_args = workspace.adapter().args();
    // remove the tool-tool binary name
    command_args.remove(0);
    let command_name = command_args.remove(0);
    let config = workspace.config();
    let (tool_config, command_config) = find_command(&command_name, config)?;
    let extensions = workspace
        .adapter()
        .get_platform()
        .get_executable_extensions();
    let mut parsed_command =
        shellish_parse::parse(&command_config.command_string, ParseOptions::new())?;
    let binary = parsed_command.remove(0);
    let tool_path = workspace
        .cache_dir()
        .join(format!("{}-{}", tool_config.name, tool_config.version));
    let mut binary_path_maybe = None;
    let mut errors = vec![];
    let lock_guard = LockGuard::new(workspace.adapter())?;
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
    drop(lock_guard);
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
    let start_time = workspace.adapter().now()?;
    let exit_code = workspace.adapter().execute(ExecutionRequest {
        binary_path: binary_path.clone(),
        args: args.clone(),
        env: env.clone(),
    })?;
    let end_time = workspace.adapter().now()?;
    let duration = end_time - start_time;
    if duration > Duration::from_secs(4) {
        workspace.adapter().print(&format!(
            "🕑  Command took {} seconds\n",
            duration.as_secs()
        ));
    }
    if exit_code != 0 {
        workspace.adapter().print(&format!(
            "❗ Command '{command_name}' failed with exit code {exit_code}"
        ));
        workspace.adapter().print(&format!(
            "\tExecuted command was: {binary_path} {}",
            args.join(" ")
        ));
        workspace.adapter().print("\tEnvironment:");
        for env in env {
            workspace
                .adapter()
                .print(&format!("\t\t{}={}", env.key, env.value));
        }
    }
    Ok(())
}
