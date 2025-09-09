use crate::adapter::{Adapter, AdapterBox};
use crate::checksums::load_checksums;
use crate::configuration::expand_config::expand_configuration_template_expressions;
use crate::configuration::parse_config::parse_configuration_from_kdl;
use crate::configuration::{CONFIGURATION_FILE_NAME, ToolToolConfiguration};
use crate::download_task::run_download_task;
use crate::execute_tool::execute_tool;
use crate::help::print_help;
use crate::types::FilePath;
use crate::version::get_version;
use crate::workspace::Workspace;
use kdl::KdlError;
use miette::{GraphicalReportHandler, GraphicalTheme};
use std::collections::BTreeMap;
use std::rc::Rc;
use tool_tool_base::logging::info;
use tool_tool_base::result::ToolToolResult;
use tool_tool_base::result::{Context, MietteReportError, ToolToolError};

pub struct ToolToolRunnerInitial {
    adapter: AdapterBox,
    #[allow(dead_code)]
    report_handler: GraphicalReportHandler,
}

impl ToolToolRunnerInitial {
    pub fn new(adapter: impl Adapter) -> Self {
        let want_color = want_color(adapter.env());
        let theme = if want_color {
            GraphicalTheme::unicode()
        } else {
            GraphicalTheme::unicode_nocolor()
        };
        let report_handler = GraphicalReportHandler::new_themed(theme);
        Self {
            adapter: Rc::new(adapter),
            report_handler,
        }
    }
    pub fn run(&self) {
        info!("Running tool-tool ({}):", get_version());
        let adapter = self.adapter.clone();
        match self.run_inner() {
            Ok(()) => {}
            Err(err) => {
                if let Err(print_err) = self.print_error(err) {
                    adapter.print(&format!("ERROR: Failed to print error: {print_err}\n"));
                }
                self.adapter.exit(1);
            }
        }
    }

    fn print_error(&self, err: ToolToolError) -> ToolToolResult<()> {
        let mut message = format!("ERROR running tool-tool ({}): {err}\n", get_version());

        if err.source().is_some() {
            message.push_str("  Chain of causes:\n");
            err.chain().skip(1).enumerate().for_each(|(index, err)| {
                message.push_str(&format!("   {index}: {err}\n"));
            });
            message.push('\n');
            for err in err.chain() {
                if let Some(err) = err.downcast_ref::<KdlError>() {
                    self.report_handler.render_report(&mut message, err)?;
                } else if let Some(err) = err.downcast_ref::<MietteReportError>() {
                    self.report_handler
                        .render_report(&mut message, err.report().as_ref())?;
                }
            }
        }

        self.adapter.print(&message);
        Ok(())
    }

    pub fn run_inner(&self) -> ToolToolResult<()> {
        let args = self.adapter.args();
        parse_configuration_from_kdl(CONFIGURATION_FILE_NAME, "")?;
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
            "--download" => {
                self.download()?;
            }
            "--version" => {
                self.print_version();
            }
            other => {
                if other.starts_with('-') {
                    self.adapter.print(&format!("ERROR: Unknown argument: '{other}'\n\nTry --help for more information about supported arguments"));
                    self.adapter.exit(1);
                } else {
                    self.execute_tool()?;
                }
            }
        }
        Ok(())
    }

    fn execute_tool(&self) -> ToolToolResult<()> {
        let mut workspace = self.create_workspace()?;
        run_download_task(&mut workspace)?;
        execute_tool(&mut workspace)
    }

    fn print_help(&self) {
        print_help(self.adapter.as_ref());
    }

    fn validate_config(&self) -> ToolToolResult<()> {
        self.load_config()
            .context("Failed to validate tool-tool configuration file '.tool-tool.v2.kdl'")?;
        Ok(())
    }

    fn expand_config(&self) -> ToolToolResult<()> {
        let config = self.load_config()?;
        let mut output = String::new();
        output.push_str("Expanded tool-tool configuration:\n");

        for tool in &config.tools {
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

    fn print_version(&self) {
        self.adapter.print(&format!("{}\n", get_version()))
    }

    fn download(&self) -> ToolToolResult<()> {
        run_download_task(&mut self.create_workspace()?)
    }

    fn create_workspace(&self) -> ToolToolResult<Workspace> {
        let config = load_config(self.adapter.as_ref())?;
        let mut workspace = Workspace::new(config, self.adapter.clone());
        load_checksums(&mut workspace)?;
        Ok(workspace)
    }

    fn load_config(&self) -> ToolToolResult<ToolToolConfiguration> {
        load_config(self.adapter.as_ref())
    }
}

pub fn load_config(adapter: &dyn Adapter) -> ToolToolResult<ToolToolConfiguration> {
    let config_path = FilePath::from(CONFIGURATION_FILE_NAME);
    let config_string = std::io::read_to_string(adapter.read_file(&config_path)?)?;
    let mut config = parse_configuration_from_kdl(config_path.as_ref(), &config_string)?;
    expand_configuration_template_expressions(&mut config)?;
    Ok(config)
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
    use crate::configuration::platform::DownloadPlatform;
    use crate::mock_adapter::MockAdapter;
    use crate::runner_initial::ToolToolRunnerInitial;
    use crate::test_util::archive_builder::ArchiveBuilder;
    use crate::test_util::targz_builder::TarGzBuilder;
    use crate::test_util::zip_builder::ZipBuilder;
    use expect_test::expect;
    use tool_tool_base::result::ToolToolResult;

    fn setup() -> (ToolToolRunnerInitial, MockAdapter) {
        let adapter = MockAdapter::new();
        adapter.set_url(
            "https://example.com/test-1.2.3.zip",
            build_test_zip().unwrap(),
        );
        adapter.set_url(
            "https://example.com/test-1.2.3.tar.gz",
            build_test_targz().unwrap(),
        );
        let runner = ToolToolRunnerInitial::new(adapter.clone());
        (runner, adapter)
    }

    fn build_test_zip() -> ToolToolResult<Vec<u8>> {
        build_archive::<ZipBuilder>()
    }

    fn build_test_targz() -> ToolToolResult<Vec<u8>> {
        build_archive::<TarGzBuilder>()
    }

    fn build_archive<T: ArchiveBuilder>() -> ToolToolResult<Vec<u8>> {
        let mut archive_builder = T::default();
        archive_builder.add_file("upper/foo", b"bar")?;
        archive_builder.add_file("upper/fizz/buzz", b"bizz")?;
        Ok(archive_builder.build()?)
    }

    #[test]
    fn print_help() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
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
        let (runner, adapter) = setup();
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
        let (runner, adapter) = setup();
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
        let (runner, adapter) = setup();
        adapter.set_args(&["--validate"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
        "#]]);
        Ok(())
    }

    #[test]
    fn download_zip() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_platform(DownloadPlatform::Windows);
        adapter.set_args(&["--download"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            READ FILE: .tool-tool/v2/checksums.kdl
            CREATE DIR: .tool-tool/v2/tmp
            CREATE DIR: .tool-tool/v2/
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            DOWNLOAD: https://example.com/test-1.2.3.zip -> .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            DELETE DIR: .tool-tool/v2/lsd-1.2.3
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/foo
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/foo -> bar
            CREATE DIR: .tool-tool/v2/lsd-1.2.3/fizz
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz -> bizz
            DOWNLOAD: https://example.com/test-1.2.3.tar.gz -> .tool-tool/v2/tmp/download-lsd-1.2.3-linux
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-linux
            CREATE FILE: .tool-tool/v2/checksums.kdl
            WRITE FILE: .tool-tool/v2/checksums.kdl -> sha512sums{
            "https://example.com/test-1.2.3.tar.gz" c8c4fd942d21f30798773b441950f6febadbf5e6d965e65aa718a45d83e13f7df952ead930f3b72d02cdc7befefc94758453882f43744d8a003aa5449ed3d8f6
            "https://example.com/test-1.2.3.zip" fb7ad071d9053181b7ed676b14addd802008a0d2b0fa5aab930c4394a31b9686641d9bcc76432891a2611688c5f1504d85ae74c6a510db7e3595f58c5ff98e49
            }

        "#]]);
        Ok(())
    }

    #[test]
    fn download_zip_with_checksums() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_file(".tool-tool/v2/checksums.kdl", r#"
            sha512sums{
                "https://example.com/test-1.2.3.tar.gz" c8c4fd942d21f30798773b441950f6febadbf5e6d965e65aa718a45d83e13f7df952ead930f3b72d02cdc7befefc94758453882f43744d8a003aa5449ed3d8f6
                "https://example.com/test-1.2.3.zip" fb7ad071d9053181b7ed676b14addd802008a0d2b0fa5aab930c4394a31b9686641d9bcc76432891a2611688c5f1504d85ae74c6a510db7e3595f58c5ff98e49
            }
        "#);
        adapter.set_platform(DownloadPlatform::Windows);
        adapter.set_args(&["--download"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            READ FILE: .tool-tool/v2/checksums.kdl
            CREATE DIR: .tool-tool/v2/tmp
            CREATE DIR: .tool-tool/v2/
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            DOWNLOAD: https://example.com/test-1.2.3.zip -> .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            DELETE DIR: .tool-tool/v2/lsd-1.2.3
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/foo
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/foo -> bar
            CREATE DIR: .tool-tool/v2/lsd-1.2.3/fizz
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz -> bizz
        "#]]);
        Ok(())
    }

    #[test]
    fn download_zip_with_wrong_checksum() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_file(".tool-tool/v2/checksums.kdl", r#"
            sha512sums{
                "https://example.com/test-1.2.3.tar.gz" c8c4fd942d21f30798773b441950f6febadbf5e6d965e65aa718a45d83e13f7df952ead930f3b72d02cdc7befefc94758453882f43744d8a003aa5449ed3d8f6
                "https://example.com/test-1.2.3.zip" wrong_checksum
            }
        "#);
        adapter.set_platform(DownloadPlatform::Windows);
        adapter.set_args(&["--download"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            READ FILE: .tool-tool/v2/checksums.kdl
            CREATE DIR: .tool-tool/v2/tmp
            CREATE DIR: .tool-tool/v2/
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            DOWNLOAD: https://example.com/test-1.2.3.zip -> .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            PRINT:
            	ERROR running tool-tool (vTEST): Checksum mismatch for tool 'lsd'
            	Expected: wrong_checksum
            	Actual:   fb7ad071d9053181b7ed676b14addd802008a0d2b0fa5aab930c4394a31b9686641d9bcc76432891a2611688c5f1504d85ae74c6a510db7e3595f58c5ff98e49

            EXIT: 1
        "#]]);
        Ok(())
    }

    #[test]
    fn download_zip_with_wrong_targz_checksum() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_file(".tool-tool/v2/checksums.kdl", r#"
            sha512sums{
                // Other platforms are not checked
                "https://example.com/test-1.2.3.tar.gz" wrong_checksum
                "https://example.com/test-1.2.3.zip" fb7ad071d9053181b7ed676b14addd802008a0d2b0fa5aab930c4394a31b9686641d9bcc76432891a2611688c5f1504d85ae74c6a510db7e3595f58c5ff98e49
            }
        "#);
        adapter.set_platform(DownloadPlatform::Windows);
        adapter.set_args(&["--download"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            READ FILE: .tool-tool/v2/checksums.kdl
            CREATE DIR: .tool-tool/v2/tmp
            CREATE DIR: .tool-tool/v2/
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            DOWNLOAD: https://example.com/test-1.2.3.zip -> .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            DELETE DIR: .tool-tool/v2/lsd-1.2.3
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/foo
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/foo -> bar
            CREATE DIR: .tool-tool/v2/lsd-1.2.3/fizz
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz -> bizz
        "#]]);
        Ok(())
    }

    #[test]
    fn download_targz() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_platform(DownloadPlatform::Linux);
        adapter.set_args(&["--download"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            READ FILE: .tool-tool/v2/checksums.kdl
            CREATE DIR: .tool-tool/v2/tmp
            CREATE DIR: .tool-tool/v2/
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            DOWNLOAD: https://example.com/test-1.2.3.tar.gz -> .tool-tool/v2/tmp/download-lsd-1.2.3-linux
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-linux
            DELETE DIR: .tool-tool/v2/lsd-1.2.3
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-linux
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/foo
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/foo -> bar
            CREATE DIR: .tool-tool/v2/lsd-1.2.3/fizz
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz -> bizz
            DOWNLOAD: https://example.com/test-1.2.3.zip -> .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            CREATE FILE: .tool-tool/v2/checksums.kdl
            WRITE FILE: .tool-tool/v2/checksums.kdl -> sha512sums{
            "https://example.com/test-1.2.3.tar.gz" c8c4fd942d21f30798773b441950f6febadbf5e6d965e65aa718a45d83e13f7df952ead930f3b72d02cdc7befefc94758453882f43744d8a003aa5449ed3d8f6
            "https://example.com/test-1.2.3.zip" fb7ad071d9053181b7ed676b14addd802008a0d2b0fa5aab930c4394a31b9686641d9bcc76432891a2611688c5f1504d85ae74c6a510db7e3595f58c5ff98e49
            }

        "#]]);
        Ok(())
    }

    #[test]
    fn run_tool() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_platform(DownloadPlatform::Windows);
        adapter.set_args(&["bar"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            READ FILE: .tool-tool/v2/checksums.kdl
            CREATE DIR: .tool-tool/v2/tmp
            CREATE DIR: .tool-tool/v2/
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            DOWNLOAD: https://example.com/test-1.2.3.zip -> .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            DELETE DIR: .tool-tool/v2/lsd-1.2.3
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-windows
            CREATE DIR: .tool-tool/v2/lsd-1.2.3
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/foo
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/foo -> bar
            CREATE DIR: .tool-tool/v2/lsd-1.2.3/fizz
            CREATE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz
            WRITE FILE: .tool-tool/v2/lsd-1.2.3/fizz/buzz -> bizz
            DOWNLOAD: https://example.com/test-1.2.3.tar.gz -> .tool-tool/v2/tmp/download-lsd-1.2.3-linux
            READ FILE: .tool-tool/v2/tmp/download-lsd-1.2.3-linux
            CREATE FILE: .tool-tool/v2/checksums.kdl
            WRITE FILE: .tool-tool/v2/checksums.kdl -> sha512sums{
            "https://example.com/test-1.2.3.tar.gz" c8c4fd942d21f30798773b441950f6febadbf5e6d965e65aa718a45d83e13f7df952ead930f3b72d02cdc7befefc94758453882f43744d8a003aa5449ed3d8f6
            "https://example.com/test-1.2.3.zip" fb7ad071d9053181b7ed676b14addd802008a0d2b0fa5aab930c4394a31b9686641d9bcc76432891a2611688c5f1504d85ae74c6a510db7e3595f58c5ff98e49
            }

            FILE EXISTS?:
            .tool-tool/v2/lsd-1.2.3/fizz buzz.exe
            FILE EXISTS?:
            .tool-tool/v2/lsd-1.2.3/fizz buzz.bat
            FILE EXISTS?:
            .tool-tool/v2/lsd-1.2.3/fizz buzz.cmd
            PRINT:
            	ERROR running tool-tool (vTEST): Failed to find binary for command 'bar' in tool lsd, found no matching executable binaries: .tool-tool/v2/lsd-1.2.3/fizz buzz(.exe|.bat|.cmd)

            EXIT: 1
        "#]]);
        Ok(())
    }

    #[test]
    fn expand_config() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_args(&["--expand-config"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            PRINT:
            	Expanded tool-tool configuration:
            		lsd 1.2.3:
            			download urls:
            				linux:   https://example.com/test-1.2.3.tar.gz
            				windows: https://example.com/test-1.2.3.zip
            			commands:
            				bar:    fizz buzz
            				foobar: echo foobar
            			env:
            				FIZZ:     buzz
            				FROBNIZZ: nizzle

        "#]]);
        Ok(())
    }

    #[test]
    fn expand_config_with_syntax_error() -> ToolToolResult<()> {
        let (runner, adapter) = setup();
        adapter.set_configuration(r#"tools {"#);
        adapter.set_args(&["--expand-config"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            PRINT:
            	ERROR running tool-tool (vTEST): Failed to parse KDL file '.tool-tool.v2.kdl'
            	  Chain of causes:
            	   0: Could not parse '.tool-tool.v2.kdl'
            	   1: Failed to parse KDL document

            	  × Failed to parse KDL document

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
        let (runner, adapter) = setup();
        adapter.set_configuration(r#"foo"#);
        adapter.set_args(&["--validate"]);
        runner.run();
        adapter.verify_effects(expect![[r#"
            READ FILE: .tool-tool.v2.kdl
            PRINT:
            	ERROR running tool-tool (vTEST): Failed to validate tool-tool configuration file '.tool-tool.v2.kdl'
            	  Chain of causes:
            	   0: Failed to parse KDL file '.tool-tool.v2.kdl'
            	   1: Unexpected top-level item: 'foo'

            	configuration::parse_config::parse_kdl

            	  × Unexpected top-level item: 'foo'
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
