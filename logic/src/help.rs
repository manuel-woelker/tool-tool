use crate::adapter::Adapter;
use crate::version::get_version;

pub fn print_help(adapter: &dyn Adapter) {
    adapter.print(&format!(
        "🔧  tool-tool ({}) - A versatile tool management utility",
        get_version()
    ));
    let help_text = r#"
USAGE:
    tool-tool [OPTIONS]

OPTIONS:
    --help              Show this help message and exit
    --version           Display version information and exit
    --validate          Validate the tool configuration file
    --expand-config     Expand and display the configuration with all templates resolved

EXAMPLES:
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
