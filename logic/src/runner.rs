use crate::adapter::Adapter;
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
        let args = self.adapter.get_args();
        for arg in args.iter().skip(1) {
            match arg.as_str() {
                "--help" => {
                    self.adapter.print("help");
                }
                "--version" => {
                    self.adapter.print(&format!("{}\n", get_version()));
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
}
