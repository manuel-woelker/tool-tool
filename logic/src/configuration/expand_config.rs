use crate::configuration::ToolToolConfiguration;
use crate::configuration::platform::DownloadPlatform;
use crate::template_expander::TemplateExpander;
use crate::template_string::TemplateString;
use tool_tool_base::result::ToolToolResult;

pub fn expand_configuration_template_expressions(
    configuration: &mut ToolToolConfiguration,
    host_platform: DownloadPlatform,
) -> ToolToolResult<()> {
    let mut expander = TemplateExpander::default();

    for tool in &mut configuration.tools {
        expander.add_replacer("version", |_| tool.version.clone());
        for platform in DownloadPlatform::VALUES {
            if platform == host_platform {
                expander.add_replacer(platform.as_str(), |substitution| {
                    substitution.arguments[0].clone()
                });
            } else {
                expander.add_replacer(platform.as_str(), |_| String::new());
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
                        lsd "${linux:bin}/lsd{windows:.exe}"
                    }
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
                                command_string: "bin/lsd{windows:.exe}",
                                description: "",
                            },
                        ],
                        env: [],
                    },
                ],
            }
        "#]]
    );
}
