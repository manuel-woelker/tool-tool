use kdl::KdlDocument;
use miette::{GraphicalReportHandler, GraphicalTheme};
use tool_tool_base::result::{Context, ToolToolResult};

#[derive(Debug)]
pub struct ToolConfiguration {
    pub name: String,
}

#[derive(Debug)]
pub struct ToolToolConfiguration {
    #[allow(dead_code)]
    tools: Vec<ToolConfiguration>,
}

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
    for node in doc.nodes() {
        if node.name().value() == "tool" {
            let name = node
                .entry(0)
                .expect("Expected tool name")
                .value()
                .to_string();
            let tool = ToolConfiguration { name };
            tools.push(tool);
        }
    }
    let configuration = ToolToolConfiguration { tools };
    Ok(configuration)
}

#[cfg(test)]
mod tests {
    use crate::configuration::parse_configuration_from_kdl;
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
        "",
        expect![[r#"
            ToolToolConfiguration {
                tools: [],
            }
        "#]]
    );

    test_parse!(
        simple_tool,
        r#"tool lsd version="0.17.0""#,
        expect![[r#"
            ToolToolConfiguration {
                tools: [
                    ToolConfiguration {
                        name: "lsd",
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
