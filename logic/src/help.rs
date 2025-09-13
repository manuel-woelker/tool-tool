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
    --commands          Show available commands
    --version           Display version information
    --validate          Validate the tool configuration file
    --expand-config     Expand and display the configuration with all templates resolved

EXAMPLES:
    # Execute the 'foo' command defined in .tool-tool.v2.kdl
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
    tool-tool looks for a configuration file named '.tool-tool.v2.kdl' in the current
    directory. This file should contain the tool configuration in KDL format.

For more information, please refer to the documentation."#;

    adapter.print(help_text);
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
    for command in commands {
        message.push_str(&format!("\t{}\n", command.0));
    }
    Some(message)
}
