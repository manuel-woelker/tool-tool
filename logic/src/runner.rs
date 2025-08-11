use crate::adapter::Adapter;
use crate::configuration::{ToolToolConfiguration, parse_configuration_from_kdl};
use crate::types::FilePath;
use crate::version::get_version;
use tool_tool_base::result::ToolToolResult;

pub struct ToolToolRunner {
    adapter: Box<dyn Adapter>,
}

impl ToolToolRunner {
    pub fn new(adapter: impl Adapter) -> Self {
        Self {
            adapter: Box::new(adapter),
        }
    }

    pub fn run(&mut self) -> ToolToolResult<()> {
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
        let _ = self.load_config()?;
        Ok(())
    }

    fn print_version(&mut self) {
        self.adapter.print(&format!("{}\n", get_version()))
    }

    fn load_config(&self) -> ToolToolResult<ToolToolConfiguration> {
        let config_path = FilePath::from(".tool-tool.v2.kdl");
        let config_string = self.adapter.read_file(&config_path)?;
        let config = parse_configuration_from_kdl(config_path.as_ref(), &config_string)?;
        dbg!(&config);
        Ok(config)
    }
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
        runner.run()?;

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
        runner.run()?;

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
        runner.run()?;
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
        runner.run()?;
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
        "#]]);
        Ok(())
    }
}
