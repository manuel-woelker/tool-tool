use crate::configuration::ToolToolConfiguration;
use crate::configuration::platform::DownloadPlatform;
use crate::template_expander::TemplateExpander;
use crate::template_string::TemplateString;
use tool_tool_base::result::{ToolToolResult, err};

pub fn expand_configuration_template_expressions(
    configuration: &mut ToolToolConfiguration,
    host_platform: DownloadPlatform,
) -> ToolToolResult<()> {
    let original_configuration = configuration.clone();
    let mut expander = TemplateExpander::default();
    expander.add_replace_fn("dir", |substitution| {
        let tool_name = &substitution.arguments[0];
        let tool = original_configuration
            .tools
            .iter()
            .find(|tool| tool.name == *tool_name)
            .ok_or_else(|| err!("Could not find tool '{tool_name}'"))?;
        Ok(format!(
            ".tool-tool/v2/cache/{}-{}",
            tool.name, tool.version
        ))
    });
    for tool in &mut configuration.tools {
        expander.add_replace_fn("version", |_| Ok(tool.version.clone()));
        for platform in DownloadPlatform::VALUES {
            if platform == host_platform {
                expander.add_replace_fn(platform.as_str(), |substitution| {
                    Ok(substitution.arguments[0].clone())
                });
            } else {
                expander.add_replace_fn(platform.as_str(), |_| Ok(String::new()));
            }
        }
        for download_artifact in tool.download_urls.values_mut() {
            let template_string = TemplateString::try_from(download_artifact.url.as_str())?;
            let new_url = expander.expand(template_string)?;
            download_artifact.url = new_url;
        }
        for command in tool.commands.iter_mut() {
            let template_string = TemplateString::try_from(command.command_string.as_str())?;
            let new_command_string = expander.expand(template_string)?;
            command.command_string = new_command_string;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::configuration::CONFIGURATION_FILE_NAME;
    use crate::configuration::expand_config::expand_configuration_template_expressions;
    use crate::configuration::parse_config::parse_configuration_from_kdl;
    use crate::configuration::platform::DownloadPlatform;
    use expect_test::{Expect, expect};
    use tool_tool_base::result::ToolToolResult;

    fn test_parse_and_expand(kdl: &str, expected: Expect) -> ToolToolResult<()> {
        let mut config = parse_configuration_from_kdl(CONFIGURATION_FILE_NAME, kdl)?;
        expand_configuration_template_expressions(&mut config, DownloadPlatform::Linux)?;
        expected.assert_debug_eq(&config);
        Ok(())
    }

    macro_rules! test_parse_and_expand(
        ($name:ident, $kdl:expr, $expected:expr) => {
            #[test]
            fn $name() -> ToolToolResult<()> {
                test_parse_and_expand($kdl, $expected)
            }
            });

    test_parse_and_expand!(
        test_expand_version,
        r#"tools {
                lsd "0.17.0" {
                    download {
                        linux "https://github.com/Peltoche/lsd/releases/download/${version}/lsd-${version}-x86_64-unknown-linux-gnu.tar.gz"
                        windows "https://github.com/Peltoche/lsd/releases/download/${version}/lsd-${version}-x86_64-pc-windows-msvc.zip"
                    }
                    commands {
                        lsd "${linux:bin}/lsd{windows:.exe} ${dir:foo}"
                    }
                }
                foo "1.2.3" {

                }
            }"#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [
                    ToolConfiguration {
                        name: "lsd",
                        version: "0.17.0",
                        default_download_artifact: None,
                        download_urls: {
                            Linux: DownloadArtifact {
                                url: "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz",
                            },
                            Windows: DownloadArtifact {
                                url: "https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-pc-windows-msvc.zip",
                            },
                        },
                        commands: [
                            Command {
                                name: "lsd",
                                command_string: "bin/lsd{windows:.exe} .tool-tool/v2/cache/foo-1.2.3",
                                description: "",
                            },
                        ],
                        env: [],
                    },
                    ToolConfiguration {
                        name: "foo",
                        version: "1.2.3",
                        default_download_artifact: None,
                        download_urls: {},
                        commands: [],
                        env: [],
                    },
                ],
            }
        "#]]
    );
}
