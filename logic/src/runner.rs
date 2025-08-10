use crate::TOOL_TOOL_VERSION;
use crate::adapter::Adapter;

pub struct ToolToolRunner {
    adapter: Box<dyn Adapter>,
}

impl ToolToolRunner {
    pub fn new(adapter: impl Adapter) -> Self {
        Self {
            adapter: Box::new(adapter),
        }
    }

    pub fn run(&mut self) {
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
    }
}

#[cfg(test)]
mod tests {
    use crate::TOOL_TOOL_VERSION;
    use crate::mock_adapter::MockAdapter;
    use crate::runner::ToolToolRunner;
    use expect_test::expect;

    fn setup() -> (ToolToolRunner, MockAdapter) {
        let adapter = MockAdapter::new();
        let runner = super::ToolToolRunner::new(adapter.clone());
        (runner, adapter)
    }

    #[test]
    fn print_help() {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--help"]);
        runner.run();

        adapter.verify_effects(expect![[r#"
            PRINT:
            	help
        "#]])
    }

    #[test]
    fn print_version() {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--version"]);
        runner.run();

        assert_eq!(
            adapter.get_effects(),
            format!("PRINT:\n\t{}\n\n", TOOL_TOOL_VERSION)
        );
    }

    #[test]
    fn handle_unknown_argument() {
        let (mut runner, adapter) = setup();
        adapter.set_args(&["--missing"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            PRINT:
            	ERROR: Unknown argument: '--missing'

            	Try --help for more information about supported arguments
            EXIT: 1
        "#]])
    }
}
