use crate::configuration::platform::DownloadPlatform;
use crate::configuration::{Command, DownloadArtifact, ToolConfiguration, ToolToolConfiguration};
use crate::types::EnvPair;
use miette::{LabeledSpan, Severity, miette};
use std::collections::BTreeMap;
use std::str::FromStr;
use toml_span::{parse, Value};
use toml_span::value::Key;
use tool_tool_base::logging::info;
use tool_tool_base::result::{
    Context, MietteReportError, ToolToolError, ToolToolResult, bail, err,
};
use tracing::info_span;

pub fn parse_configuration_from_kdl(
    filename: &str,
    kdl: &str,
) -> ToolToolResult<ToolToolConfiguration> {
    info!("Parsing TOML file '{filename}'");
    let _span = info_span!("Parse configuration from KDL ", filename).entered();
    (|| -> ToolToolResult<ToolToolConfiguration> {
        let mut tools = vec![];
        let doc = parse(kdl)
            .with_context(|| format!("Could not parse '{filename}'"))?;
        for (key, value) in doc.as_table().ok_or_else(||err!("Expected root to be a table"))?.iter() {
            match key.name.as_ref() {
                "tools" => {
                    for (tool_key, tool_value) in value.as_table().ok_or_else(||err!("Expected 'tools' to be a table"))?.iter() {
                        //let tool = parse_tool(tool_key, tool_value).expect("TODO: Handle error");
                        let tool = parse_tool(tool_key, tool_value).expect("TODO: Handle error");
                        tools.push(tool);
/*                        let version = tool_value.pointer("/version").ok_or_else(||err!("Expected 'version'"))?.as_str().ok_or_else(||err!("Expected 'version' to be a string"))?.to_string();
                        let tool = ToolConfiguration {
                            name: tool_key.name.as_ref().to_string(),
                            version,
                            default_download_artifact: None,
                            download_urls: BTreeMap::new(),
                            commands: vec![],
                            env: vec![],
                        };
                        tools.push(tool);*/
                    }
                }
                other => {
                    bail!("Unexpected top-level item: '{other}'");
                }
            }
        }
/*        let doc = parse(kdl)
            .with_context(|| format!("Could not parse '{filename}'"))?;
        for (key, value) in doc.as_table().unwrap().iter() {
            match key.name.as_ref() {
                "tools" => {
/*                    for tool_node in children(document_node) {
                        let tool = parse_tool(tool_node).expect("TODO: Handle error");
                        tools.push(tool);
                    }*/
                }
                other => {
                    bail!("Unexpected top-level item: '{other}'");
/*                    let report = miette!(
                        code = "configuration::parse_config::parse_kdl".to_string(),
                        severity = Severity::Error,
                        labels = vec![LabeledSpan::new_primary_with_span(
                            Some("unexpected".to_string()),
                            key.span
                        )],
                        help = "Valid top level items are: 'tools'",
                        "Unexpected top-level item: '{other}'"
                    )
                    .with_source_code(kdl.to_string());
                    // TODO: Report error
                    //
                    // return Err(report);
                    //                    report.anyhow_kind()
                    return Err(ToolToolError::new(MietteReportError::from(report)));
                    //bail!(report);*/
                }
            }
        }*/
        let configuration = ToolToolConfiguration { tools };
        Ok(configuration)
    })()
    .with_context(|| format!("Failed to parse tool-tool configuration file '{filename}'"))
}

fn parse_tool(tool_key: &Key, tool_value: &Value) -> ToolToolResult<ToolConfiguration> {
    let version = tool_value.pointer("/version").ok_or_else(||err!("Expected 'version'"))?.as_str().ok_or_else(||err!("Expected 'version' to be a string"))?.to_string();
    let mut default_download_artifact = None;
    let mut download_urls = BTreeMap::new();
    if let Some(download) = tool_value.pointer("/download").and_then(|download| download.as_table()) {
        for (os, url_value) in download {
            let download_artifact = DownloadArtifact { url: url_value.as_str().ok_or_else(|| err!("Expected 'url' to be a string"))?.to_string() };
            if os.name.as_ref() == "default" {
                default_download_artifact = Some(download_artifact);
            } else {
                download_urls.insert(DownloadPlatform::from_str(os.name.as_ref())?, download_artifact);
            }
        }
    }
    let tool = ToolConfiguration {
        name: tool_key.name.as_ref().to_string(),
        version,
        default_download_artifact,
        download_urls,
        commands: vec![],
        env: vec![],
    };
    Ok(tool)
}
/*
fn parse_tool(tool_node: &KdlNode) -> ToolToolResult<ToolConfiguration> {
    let name = tool_node.name().value().to_string();
    let version = tool_node
        .entry(0)
        .ok_or_else(|| err!("Expected tool version"))?
        .value()
        .as_string()
        .expect("Expected tool version to be a string");
    let mut download_urls = BTreeMap::new();
    let mut commands = vec![];
    let mut env = vec![];
    let mut default_download_artifact = None;
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
                        .expect("Expected download url to be a string")
                        .to_string();
                    if os == "default" {
                        default_download_artifact = Some(DownloadArtifact { url });
                    } else {
                        download_urls
                            .insert(DownloadPlatform::from_str(&os)?, DownloadArtifact { url });
                    }
                }
            }
            "commands" => {
                for command_child in children(tool_child) {
                    let command_name = command_child.name().value().to_string();
                    // TODO: collect all keys as arguments?
                    let command_binary = command_child
                        .entry(0)
                        .expect("Expected command binary")
                        .value()
                        .as_string()
                        .expect("Expected command to be a string")
                        .to_string();
                    let description = command_child
                        .entry("description")
                        .and_then(|entry| entry.value().as_string())
                        .unwrap_or("")
                        .to_string()
                        .to_string();
                    commands.push(Command::new(command_name, command_binary, description));
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
                    env.push(EnvPair::new(env_name, env_value));
                }
            }
            other => bail!("Unknown tool child: '{other}'"),
            // TODO: use miette spans for better error messages
        }
    }
    let tool = ToolConfiguration {
        name,
        version: version.to_string(),
        default_download_artifact,
        download_urls,
        commands,
        env,
    };
    Ok(tool)
}

fn children(node: &KdlNode) -> impl IntoIterator<Item = &KdlNode> + '_ {
    node.children().map(|doc| doc.nodes()).into_iter().flatten()
}
*/
#[cfg(test)]
mod tests {
    use crate::configuration::CONFIGURATION_FILE_NAME;
    use crate::configuration::parse_config::parse_configuration_from_kdl;
    use expect_test::{Expect, expect};
    use tool_tool_base::result::ToolToolResult;

    fn test_parse(kdl: &str, expected: Expect) -> ToolToolResult<()> {
        let config = parse_configuration_from_kdl(CONFIGURATION_FILE_NAME, kdl)?;
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
        "[tools]",
        expect![[r#"
            ToolToolConfiguration {
                tools: [],
            }
        "#]]
    );

    test_parse!(
        simple_tool,
        r#"[tools]
           lsd = { version="0.17.0" }
        "#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [
                    ToolConfiguration {
                        name: "lsd",
                        version: "0.17.0",
                        default_download_artifact: None,
                        download_urls: {},
                        commands: [],
                        env: [],
                    },
                ],
            }
        "#]]
    );

    test_parse!(
        simple_tool_with_download,
        r#"[tools]
           lsd = { version="0.17.0",
                download = {
                    linux = "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz",
                    windows ="https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-pc-windows-msvc.zip",
                },
            }
        "#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [],
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
                    bar "echo foo" description="Go to the bar"
                }
                env {
                    FOO "bar"
                }
            }
        }"#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [],
            }
        "#]]
    );

    fn test_parse_fail(kdl: &str, expected: Expect) -> ToolToolResult<()> {
        let error =
            parse_configuration_from_kdl(CONFIGURATION_FILE_NAME, kdl).expect_err("Expected error");
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
        expect!["Failed to parse tool-tool configuration file '.tool-tool/tool-tool.v2.toml'"]
    );
}
