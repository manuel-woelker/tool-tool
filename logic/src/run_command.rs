use crate::adapter::ExecutionRequest;
use crate::configuration::find_command;
use crate::configuration::platform::DownloadPlatform;
use crate::lock_guard::LockGuard;
use crate::types::EnvPair;
use crate::workspace::Workspace;
use shellish_parse::ParseOptions;
use std::time::Duration;
use tool_tool_base::result::{Context, ToolToolResult, bail};
use tracing::warn;

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
    let tool_path = workspace.tool_dir(tool_config);
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
    let mut env = Vec::new();
    if workspace.adapter().get_platform() == DownloadPlatform::Windows {
        env.push(EnvPair::new("PATHEXT".into(), ".COM;.EXE;.BAT;.CMD".into()));
        let inherited_env_vars = ["SYSTEMDRIVE", "SYSTEMROOT", "TEMP", "TMP", "WINDIR", "OS"];
        for env_name in inherited_env_vars {
            let host_env = workspace.adapter().env();
            let system_root = host_env.iter().find(|(name, _)| name == env_name);
            if let Some((_, system_root)) = system_root {
                env.push(EnvPair::new(env_name.into(), system_root.to_string()));
            } else {
                warn!(
                    "Could not inherit environment variable '{env_name}'. Networking may not work."
                );
            }
        }
    }
    env.extend_from_slice(&tool_config.env);

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
