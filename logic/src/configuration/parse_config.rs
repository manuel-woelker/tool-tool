use crate::configuration::platform::DownloadPlatform;
use crate::configuration::{ToolConfiguration, ToolToolConfiguration};
use kdl::{KdlDocument, KdlNode};
use miette::{GraphicalReportHandler, GraphicalTheme};
use std::collections::BTreeMap;
use std::str::FromStr;
use tool_tool_base::result::{Context, ToolToolResult, bail};

pub fn parse_configuration_from_kdl(
    filename: &str,
    kdl: &str,
) -> ToolToolResult<ToolToolConfiguration> {
    let mut tools = vec![];
    let result = kdl.parse::<KdlDocument>();
    let result = match result {
        Err(mut err) => {
            let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode());
            let mut message = String::new();
            for diag in &mut err.diagnostics {
                handler.render_report(&mut message, diag)?;
            }
            return Err(err).context(format!("Failed to parse KDL file {filename}:\n{message}"));
        }
        Ok(result) => result,
    };
    let doc: KdlDocument = result;
    for document_node in doc.nodes() {
        match document_node.name().value() {
            "tools" => {
                for tool_node in children(document_node) {
                    let tool = parse_tool(tool_node)?;
                    tools.push(tool);
                }
            }
            _ => continue,
        }
    }
    let configuration = ToolToolConfiguration { tools };
    Ok(configuration)
}

fn parse_tool(tool_node: &KdlNode) -> ToolToolResult<ToolConfiguration> {
    let name = tool_node.name().value().to_string();
    let version = tool_node
        .entry(0)
        .expect("Expected tool version")
        .value()
        .as_string()
        .expect("Expected tool version to be a string");
    let mut download_urls = BTreeMap::new();
    let mut commands = BTreeMap::new();
    let mut env = BTreeMap::new();
    for tool_child in children(tool_node) {
        match tool_child.name().value() {
            "download" => {
                for download_child in children(tool_child) {
                    let os = download_child.name().value().to_string();
                    let url = download_child
                        .entry(0)
                        .expect("Expected download url")
                        .value()
                        .as_string()
                        .expect("Expected download url to be a string");
                    download_urls.insert(DownloadPlatform::from_str(&os)?, url.to_string());
                }
            }
            "commands" => {
                for command_child in children(tool_child) {
                    let command_name = command_child.name().value().to_string();
                    let command_binary = command_child
                        .entry(0)
                        .expect("Expected command binary")
                        .value()
                        .as_string()
                        .expect("Expected command to be a string")
                        .to_string();
                    commands.insert(command_name, command_binary);
                }
            }
            "env" => {
                for env_child in children(tool_child) {
                    let env_name = env_child.name().value().to_string();
                    // TODO: factor out getting parameters
                    let env_value = env_child
                        .entry(0)
                        .expect("Expected command binary")
                        .value()
                        .as_string()
                        .expect("Expected command to be a string")
                        .to_string();
                    env.insert(env_name, env_value);
                }
            }
            other => bail!("Unknown tool child: '{other}'"),
            // TODO: use miette spans for better error messages
        }
    }
    let tool = ToolConfiguration {
        name,
        version: version.to_string(),
        download_urls,
        commands,
        env,
    };
    Ok(tool)
}

fn children<'a>(node: &'a KdlNode) -> impl IntoIterator<Item = &'a KdlNode> + 'a {
    node.children().map(|doc| doc.nodes()).into_iter().flatten()
}

#[cfg(test)]
mod tests {
    use crate::configuration::parse_config::parse_configuration_from_kdl;
    use expect_test::{Expect, expect};
    use tool_tool_base::result::ToolToolResult;

    fn test_parse(kdl: &str, expected: Expect) -> ToolToolResult<()> {
        let config = parse_configuration_from_kdl(".tool-tool.v2.kdl", kdl)?;
        expected.assert_debug_eq(&config);
        Ok(())
    }

    macro_rules! test_parse(
        ($name:ident, $kdl:expr, $expected:expr) => {
            #[test]
            fn $name() -> ToolToolResult<()> {
                test_parse($kdl, $expected)
            }
            });

    test_parse!(
        empty,
        "",
        expect![[r#"
            ToolToolConfiguration {
                tools: [],
            }
        "#]]
    );

    test_parse!(
        empty_tools,
        "tools",
        expect![[r#"
            ToolToolConfiguration {
                tools: [],
            }
        "#]]
    );

    test_parse!(
        simple_tool,
        r#"tools {
            lsd "0.17.0"
        }"#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [
                    ToolConfiguration {
                        name: "lsd",
                        version: "0.17.0",
                        download_urls: {},
                        commands: {},
                        env: {},
                    },
                ],
            }
        "#]]
    );

    test_parse!(
        simple_tool_with_download,
        r#"tools {
            lsd "0.17.0" {
                download {
                    linux "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz"
                    windows "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-pc-windows-msvc.zip"
                }
            }
        }"#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [
                    ToolConfiguration {
                        name: "lsd",
                        version: "0.17.0",
                        download_urls: {
                            Linux: "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz",
                            Windows: "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-pc-windows-msvc.zip",
                        },
                        commands: {},
                        env: {},
                    },
                ],
            }
        "#]]
    );

    test_parse!(
        commands_and_env,
        r#"tools {
            lsd "0.17.0" {
                download {
                    default "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz"
                }
                commands {
                    foo "echo foo"
                }
                env {
                    FOO "bar"
                }
            }
        }"#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [
                    ToolConfiguration {
                        name: "lsd",
                        version: "0.17.0",
                        download_urls: {
                            Default: "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz",
                        },
                        commands: {
                            "foo": "echo foo",
                        },
                        env: {
                            "FOO": "bar",
                        },
                    },
                ],
            }
        "#]]
    );

    fn test_parse_fail(kdl: &str, expected: Expect) -> ToolToolResult<()> {
        let error =
            parse_configuration_from_kdl(".tool-tool.v2.kdl", kdl).expect_err("Expected error");
        expected.assert_eq(&error.to_string());
        Ok(())
    }

    macro_rules! test_parse_fail(
        ($name:ident, $kdl:expr, $expected:expr) => {
            #[test]
            fn $name() -> ToolToolResult<()> {
                test_parse_fail($kdl, $expected)
            }
            });

    test_parse_fail!(
        fail_misquote,
        r#""open quote only"#,
        expect![[r#"
            Failed to parse KDL file .tool-tool.v2.kdl:
              [31m×[0m Expected quoted string
               ╭────
             [2m1[0m │ "open quote only
               · [35;1m────────┬───────[0m
               ·         [35;1m╰── [35;1mnot quoted string[0m[0m
               ╰────
              [31m×[0m Found invalid node name
               ╭────
             [2m1[0m │ "open quote only
               · [35;1m────────┬───────[0m
               ·         [35;1m╰── [35;1mnot node name[0m[0m
               ╰────
            [36m  help: [0mThis can be any string type, including a quoted, raw, or multiline string, as well as a plain identifier string.
        "#]]
    );
}
