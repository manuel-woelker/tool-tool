use crate::adapter::Adapter;
use crate::configuration::ToolToolConfiguration;
use crate::version::get_version;

pub fn print_help(adapter: &dyn Adapter) {
    adapter.print(&format!(
        "🔧  tool-tool ({}) - A versatile tool management utility",
        get_version()
    ));
    let help_text = r#"
USAGE:
    tool-tool [OPTIONS]
    tool-tool [COMMAND]

OPTIONS:
    --help              Show this help message
    --skill             Show instructions for using tool-tool from an LLM agent
    --commands          Show available commands
    --version           Display version information
    --validate          Validate the tool configuration file
    --expand-config     Expand and display the configuration with all templates resolved
    --download          Download all configured tool artifacts

EXAMPLES:
    # Execute the 'foo' command defined in .tool-tool/tool-tool.v2.kdl
    # For available commands see below
    tool-tool foo

    # Show help
    tool-tool --help

    # Print version
    tool-tool --version

    # Validate configuration
    tool-tool --validate

    # View expanded configuration
    tool-tool --expand-config

CONFIGURATION:
    tool-tool looks for '.tool-tool/tool-tool.v2.kdl' in the current project.
    This file contains the tool configuration in KDL format.

For more information, please refer to the documentation."#;

    adapter.print(help_text);
}

pub fn print_skill(adapter: &dyn Adapter) {
    adapter.print(
        r#"# tool-tool agent guide

tool-tool pins, downloads, caches, and runs project-local development tools. Run it from the project root. Configuration lives at `.tool-tool/tool-tool.v2.kdl`.

## Command line

- `tool-tool --commands`: list configured commands. Start here when discovering a project.
- `tool-tool <command> [args...]`: download the owning tool if needed, then run the configured command. Extra arguments are forwarded.
- `tool-tool --validate`: validate the configuration.
- `tool-tool --expand-config`: show the parsed configuration after template expansion.
- `tool-tool --download`: pre-download configured artifacts and update checksums.
- `tool-tool --version`: print the tool-tool version.
- `tool-tool --help`: show user-facing help.
- `tool-tool --skill`: show this guide.

Examples:

```sh
tool-tool --commands
tool-tool pnpm install
tool-tool node --version
tool-tool --validate
```

Do not invoke binaries directly from `.tool-tool/v2/cache`; use their configured command names so the pinned version and environment are applied.

## Configuration format

The configuration is KDL. `cache-directory` optionally sets a project-relative cache path and defaults to `.tool-tool/v2/cache`. Each child of `tools` has a tool name and version. A tool may contain `download`, `commands`, and `env` blocks.

```kdl
cache-directory ".cache/tool-tool"

tools {
    node "24.13.0" {
        download {
            linux "https://nodejs.org/dist/v${version}/node-v${version}-linux-x64.tar.gz"
            windows "https://nodejs.org/dist/v${version}/node-v${version}-win-x64.zip"
        }
        commands {
            node "${linux:bin/}node${windows:.exe}" description="Run Node.js"
            npm "${linux:bin/}npm${windows:.cmd}" description="Run npm"
        }
    }
}
```

Download keys are `linux`, `windows`, `darwin`, or `default`. URLs without an archive extension are installed as a single executable named after the tool; `.exe`, `.zip`, and `.tar.gz` are also supported. Zip and tar.gz archives with one shared wrapper directory have that directory removed during extraction.

Each command value is a shell-like command string relative to the extracted tool directory. Its first token is the executable; remaining tokens are fixed arguments. `description` is optional. User arguments follow the fixed arguments.

Configured `env` values are passed to the command. Commands otherwise run with a clean environment, apart from required Windows system variables.

## Templates

- `${version}`: current tool version.
- `${dir:tool-name}`: absolute cache directory of a configured tool.
- `${base_path}`: project root.
- `${linux:text}`, `${windows:text}`, `${darwin:text}`: include `text` only on that platform.
- `${cmd:command-name}`: expanded command string of another configured command.
- `${env:NAME}`: value of an environment variable visible to tool-tool.

Use `tool-tool --validate` after editing configuration and `tool-tool --expand-config` to inspect template results."#,
    );
}

pub(crate) fn generate_available_commands_message(
    config: &ToolToolConfiguration,
) -> Option<String> {
    let mut commands = vec![];
    for tool in &config.tools {
        commands.extend(&tool.commands);
    }
    if commands.is_empty() {
        return None;
    }
    let mut message = String::from("\nThe following commands are available: \n");
    let mut width = 0;
    for command in &commands {
        width = width.max(command.name.len());
    }
    commands.sort_by_key(|command| &command.name);
    for command in commands {
        let mut description = &command.description;
        if description.is_empty() {
            description = &command.command_string;
        }
        message.push_str(&format!("\t{:width$} - {description}\n", command.name));
    }
    Some(message)
}
