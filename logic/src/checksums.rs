use crate::configuration;
use crate::workspace::Workspace;
use kdl::{KdlDocument, KdlNode};
use std::collections::BTreeMap;
use tool_tool_base::result::{Context, ToolToolResult, bail, err};
use tracing::info;

#[derive(Debug, Default, Clone)]
pub struct Checksums {
    pub(crate) sha512sums: BTreeMap<String, String>,
}

pub fn load_checksums(workspace: &mut Workspace) -> ToolToolResult<()> {
    let checksums_filename = workspace
        .tool_tool_dir()
        .join(configuration::CHECKSUM_FILE_NAME);
    let mut sha512sums = BTreeMap::new();
    if let Ok(checksum_file) = workspace.adapter().read_file(&checksums_filename) {
        let checksum_kdl = std::io::read_to_string(checksum_file)?;
        let result = checksum_kdl
            .parse::<KdlDocument>()
            .wrap_err_with(|| format!("Could not parse '{checksums_filename}'"))?;
        for node in result.nodes() {
            match node.name().value() {
                "sha512sums" => {
                    for child in node.children().iter().flat_map(|entry| entry.nodes()) {
                        let url = child.name().value().to_string();
                        let checksum = child
                            .get(0)
                            .ok_or_else(|| err!("expected checksum"))?
                            .to_string();
                        sha512sums.insert(url, checksum);
                    }
                }
                other => {
                    bail!("Unknown node '{other}' in checksums file '{checksums_filename}'");
                }
            }
        }
    } else {
        info!("Checksums file '{checksums_filename}' creating a new one");
    }

    workspace.checksums = Checksums { sha512sums };
    Ok(())
}

pub fn save_checksums(workspace: &Workspace) -> ToolToolResult<()> {
    let checksums_filename = workspace
        .tool_tool_dir()
        .join(configuration::CHECKSUM_FILE_NAME);
    let mut document = KdlDocument::new();
    let mut children = KdlDocument::new();
    for (url, checksum) in workspace.checksums.sha512sums.iter() {
        let mut entry = KdlNode::new(url.as_str());
        entry.insert(0, checksum.as_str());
        children.nodes_mut().push(entry);
    }
    let mut sums_node = KdlNode::new("sha512sums");
    sums_node.set_children(children);
    document.nodes_mut().push(sums_node);
    let mut checksums_file = workspace.adapter().create_file(&checksums_filename)?;
    checksums_file.write_all(document.to_string().as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{CHECKSUM_FILE_NAME, TOOL_TOOL_DIRECTORY};
    use crate::mock_adapter::MockAdapter;
    use crate::runner_initial::load_config;
    use expect_test::expect;
    use std::rc::Rc;

    #[test]
    fn test_load_checksums_no_file() -> ToolToolResult<()> {
        let adapter = MockAdapter::new();
        let config = load_config(&adapter)?;

        let mut workspace = Workspace::new(config, Rc::new(adapter));
        load_checksums(&mut workspace)?;
        expect![[r#"
            Checksums {
                sha512sums: {},
            }
        "#]]
        .assert_debug_eq(&workspace.checksums);
        Ok(())
    }

    #[test]
    fn test_load_checksums() -> ToolToolResult<()> {
        let adapter = MockAdapter::new();
        adapter.set_file(
            &format!("{TOOL_TOOL_DIRECTORY}/{CHECKSUM_FILE_NAME}"),
            r#"
            sha512sums {
                "foo" "bar"
            }
        "#,
        );

        let config = load_config(&adapter)?;

        let mut workspace = Workspace::new(config, Rc::new(adapter));
        load_checksums(&mut workspace)?;
        expect![[r#"
            Checksums {
                sha512sums: {
                    "foo": "bar",
                },
            }
        "#]]
        .assert_debug_eq(&workspace.checksums);
        Ok(())
    }
}
