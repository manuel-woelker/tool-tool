use crate::TOOL_TOOL_VERSION;
use crate::adapter::Adapter;
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
        let args = self.adapter.get_args();
        for arg in args {
            match arg.as_str() {
                "--help" => {
                    self.adapter.print("help");
                }
                "--version" => {
                    self.adapter.print(&format!("{TOOL_TOOL_VERSION}\n"));
                }
                other => {
                    self.adapter.print(&format!("ERROR: Unknown argument: '{other}'\n\nTry --help for more information about supported arguments"));
                    self.adapter.exit(1);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::TOOL_TOOL_VERSION;
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
            format!("PRINT:\n\t{}\n\n", TOOL_TOOL_VERSION)
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
}
