use crate::configuration::ToolToolConfiguration;
use tool_tool_base::result::ToolToolResult;

pub fn expand_configuration_template_expressions(
    configuration: &mut ToolToolConfiguration,
) -> ToolToolResult<()> {
    for tool in &mut configuration.tools {
        for download_artifact in tool.download_urls.values_mut() {
            let new_url = download_artifact.url.replace("${version}", &tool.version);
            download_artifact.url = new_url;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::configuration::CONFIGURATION_FILE_NAME;
    use crate::configuration::expand_config::expand_configuration_template_expressions;
    use crate::configuration::parse_config::parse_configuration_from_kdl;
    use expect_test::{Expect, expect};
    use tool_tool_base::result::ToolToolResult;

    fn test_parse_and_expand(kdl: &str, expected: Expect) -> ToolToolResult<()> {
        let mut config = parse_configuration_from_kdl(CONFIGURATION_FILE_NAME, kdl)?;
        expand_configuration_template_expressions(&mut config)?;
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
                        commands: [],
                        env: [],
                    },
                ],
            }
        "#]]
    );
}
