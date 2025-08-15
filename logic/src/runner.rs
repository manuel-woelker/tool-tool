use crate::adapter::Adapter;
use crate::configuration::ToolToolConfiguration;
use crate::configuration::expand_config::expand_configuration_template_expressions;
use crate::configuration::parse_config::parse_configuration_from_kdl;
use crate::help::print_help;
use crate::types::FilePath;
use crate::version::get_version;
use miette::{GraphicalReportHandler, GraphicalTheme};
use std::collections::BTreeMap;
use std::fmt::Write;
use tool_tool_base::logging::info;
use tool_tool_base::result::Context;
use tool_tool_base::result::ToolToolResult;

pub struct ToolToolRunner {
    adapter: Box<dyn Adapter>,
    #[allow(dead_code)]
    report_handler: GraphicalReportHandler,
}

impl ToolToolRunner {
    pub fn new(adapter: impl Adapter) -> Self {
        let want_color = want_color(adapter.env());
        let theme = if want_color {
            GraphicalTheme::unicode()
        } else {
            GraphicalTheme::unicode_nocolor()
        };
        let report_handler = GraphicalReportHandler::new_themed(theme);
        Self {
            adapter: Box::new(adapter),
            report_handler,
        }
    }
    pub fn run(&mut self) {
        info!("Running tool-tool ({}):", get_version());
        match self.run_inner() {
            Ok(()) => {}
            Err(err) => {
                let mut message = format!("ERROR running tool-tool ({}):\n", get_version());
                writeln!(message, "{err:?}").unwrap();
                // TODO Use graphical error reporter
                self.adapter.print(&message);
                self.adapter.exit(1);
            }
        }
    }
    pub fn run_inner(&mut self) -> ToolToolResult<()> {
        let args = self.adapter.args();
        parse_configuration_from_kdl(".tool-tool.v2.kdl", "")?;
        let first_arg = args.get(1);
        let Some(first_arg) = first_arg else {
            self.print_help();
            return Ok(());
        };
        match first_arg.as_str() {
            "--help" => {
                self.print_help();
            }
            "--validate" => {
                self.validate_config()?;
            }
            "--expand-config" => {
                self.expand_config()?;
            }
            "--version" => {
                self.print_version();
            }
            other => {
                self.adapter.print(&format!("ERROR: Unknown argument: '{other}'\n\nTry --help for more information about supported arguments"));
                self.adapter.exit(1);
            }
        }
        Ok(())
    }

    fn print_help(&mut self) {
        print_help(self.adapter.as_ref());
    }

    fn validate_config(&mut self) -> ToolToolResult<()> {
        let _ = self
            .load_config()
            .context("Failed to validate tool-tool configuration file '.tool-tool.v2.kdl'")?;
        Ok(())
    }

    fn expand_config(&mut self) -> ToolToolResult<()> {
        let config = self.load_config()?;
        let mut output = String::new();
        output.push_str("Expanded tool-tool configuration:\n");

        for tool in config.tools {
            output.push_str(&format!("\t{} {}:\n", tool.name, tool.version));
            output_map(&mut output, "download urls", &tool.download_urls);
            output_map(&mut output, "commands", &tool.commands);
            output_map(&mut output, "env", &tool.env);
        }
        self.adapter.print(&output);

        fn output_map<K: std::fmt::Display, V: std::fmt::Display>(
            output: &mut String,
            title: &str,
            map: &BTreeMap<K, V>,
        ) {
            if map.is_empty() {
                return;
            }
            let mut width = 0;
            for key in map.keys() {
                width = width.max(key.to_string().len());
            }
            width += 1;
            output.push_str(&format!("\t\t{title}:\n"));
            for (key, value) in map {
                output.push_str(&format!(
                    "\t\t\t{:<width$} {}\n",
                    format!("{}:", key),
                    value,
                    width = width
                ));
            }
        }

        Ok(())
    }

    fn print_version(&mut self) {
        self.adapter.print(&format!("{}\n", get_version()))
    }

    fn load_config(&self) -> ToolToolResult<ToolToolConfiguration> {
        let config_path = FilePath::from(".tool-tool.v2.kdl");
        let config_string = self.adapter.read_file(&config_path)?;
        let mut config = parse_configuration_from_kdl(config_path.as_ref(), &config_string)?;
        expand_configuration_template_expressions(&mut config)?;
        Ok(config)
    }
}

fn want_color(env: Vec<(String, String)>) -> bool {
    let mut want_color = true;
    for (key, value) in env {
        if key == "NO_COLOR" && !value.is_empty() {
            want_color = false;
        }
    }
    want_color
}

#[cfg(test)]
mod tests {
    use crate::mock_adapter::MockAdapter;
    use crate::runner::ToolToolRunner;
    use expect_test::expect;
    use tool_tool_base::result::ToolToolResult;

    fn setup() -> (ToolToolRunner, MockAdapter) {
        let adapter = MockAdapter::new();
        let runner = ToolToolRunner::new(adapter.clone());
        (runner, adapter)
    }

    #[test]
    fn print_help() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--help"]);
        runner.run();

        adapter.verify_effects(expect![[r#"
            PRINT:
            	🔧  tool-tool (vTEST) - A versatile tool management utility
            PRINT:

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

            	For more information, please refer to the documentation.
        "#]]);
        Ok(())
    }

    #[test]
    fn print_version() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--version"]);
        runner.run();

        adapter.verify_effects(expect![[r#"
            PRINT:
            	vTEST

        "#]]);
        Ok(())
    }

    #[test]
    fn handle_unknown_argument() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--missing"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            PRINT:
            	ERROR: Unknown argument: '--missing'

            	Try --help for more information about supported arguments
            EXIT: 1
        "#]]);
        Ok(())
    }

    #[test]
    fn validate_config_success() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--validate"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
        "#]]);
        Ok(())
    }

    #[test]
    fn expand_config() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--expand-config"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            PRINT:
            	Expanded tool-tool configuration:
            		lsd 0.17.0:
            			download urls:
            				linux:   https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz
            				windows: https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-pc-windows-msvc.zip
            			commands:
            				bar:    echo bar
            				foobar: echo foobar
            			env:
            				FIZZ:     buzz
            				FROBNIZZ: nizzle

        "#]]);
        Ok(())
    }

    #[test]
    fn expand_config_with_syntax_error() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_configuration(r#"tools {"#);
        adapter.set_args(&["--expand-config"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            PRINT:
            	ERROR running tool-tool (vTEST):
            	Failed to parse KDL file '.tool-tool.v2.kdl'

            	Caused by:
            	   0: Could not parse '.tool-tool.v2.kdl'
            	   1: Failed to parse KDL document

            	Location:
            	    logic\src\configuration\parse_config.rs:23:14

            EXIT: 1
        "#]]);
        Ok(())
    }

    #[test]
    fn validate_config_with_unexpected_toplevel_item() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_configuration(r#"foo"#);
        adapter.set_args(&["--validate"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            PRINT:
            	ERROR running tool-tool (vTEST):
            	Failed to validate tool-tool configuration file '.tool-tool.v2.kdl'

            	Caused by:
            	   0: Failed to parse KDL file '.tool-tool.v2.kdl'
            	   1: Unexpected top-level item: 'foo'

            	Location:
            	    logic\src\configuration\parse_config.rs:50:32

            EXIT: 1
        "#]]);
        Ok(())
    }
}
