use crate::adapter::Adapter;
use crate::configuration::ToolToolConfiguration;
use crate::configuration::expand_config::expand_configuration_template_expressions;
use crate::configuration::parse_config::parse_configuration_from_kdl;
use crate::types::FilePath;
use crate::version::get_version;
use miette::{Context, GraphicalReportHandler, GraphicalTheme};
use std::collections::BTreeMap;
use tool_tool_base::logging::info;
use tool_tool_base::result::ToolToolResult;

pub struct ToolToolRunner {
    adapter: Box<dyn Adapter>,
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
                let _ignored = self
                    .report_handler
                    .render_report(&mut message, err.as_ref());
                self.adapter.print(&message);
                self.adapter.exit(1);
            }
        }
    }
    pub fn run_inner(&mut self) -> ToolToolResult<()> {
        let args = self.adapter.args();
        parse_configuration_from_kdl(".tool-tool.v2.kdl", "")?;
        for arg in args.iter().skip(1) {
            match arg.as_str() {
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
        }
        Ok(())
    }

    fn print_help(&mut self) {
        self.adapter.print("help");
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
    use crate::version::get_version;
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
            	help
        "#]]);
        Ok(())
    }

    #[test]
    fn print_version() -> ToolToolResult<()> {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--version"]);
        runner.run();

        assert_eq!(
            adapter.get_effects(),
            format!("PRINT:\n\t{}\n\n", get_version())
        );
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
            	ERROR running tool-tool (0.2.0-dev):
            	  × Failed to parse KDL file '.tool-tool.v2.kdl'
            	  ╰─▶ Failed to parse KDL document

            	Error: 
            	  × No closing '}' for child block
            	   ╭────
            	 1 │ tools {
            	   ·       ┬
            	   ·       ╰── not closed
            	   ╰────

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
            	ERROR running tool-tool (0.2.0-dev):
            	configuration::parse_config::parse_kdl

            	  × Failed to validate tool-tool configuration file '.tool-tool.v2.kdl'
            	  ├─▶ Failed to parse KDL file '.tool-tool.v2.kdl'
            	  ╰─▶ Unexpected top-level item: 'foo'
            	   ╭────
            	 1 │ foo
            	   · ─┬─
            	   ·  ╰── unexpected
            	   ╰────
            	  help: Valid top level items are: 'tools'

            EXIT: 1
        "#]]);
        Ok(())
    }
}
