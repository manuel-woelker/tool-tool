use std::fmt::Write;
use crate::configuration;
use crate::workspace::Workspace;
use std::collections::BTreeMap;
use toml_span::parse;
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
        let checksum_string = std::io::read_to_string(checksum_file)?;
        let document = parse(&checksum_string)?;
        let sha512sums_node = document.pointer("/sha512sums").ok_or_else(|| err!("expected sha512sums"))?;
        for (key, value) in sha512sums_node.as_table().ok_or_else(|| err!("expected sha512sums to be a table"))? {
            let url = key.name.as_ref();
            let checksum = value.as_str().ok_or_else(|| err!("expected checksum to be a string"))?;
            sha512sums.insert(url.to_string(), checksum.to_string());
        }
    } else {
        info!("Checksums file '{checksums_filename}' creating a new one");
    }

    workspace.checksums = Checksums { sha512sums };
    Ok(())
}

pub fn save_checksums(workspace: &Workspace) -> ToolToolResult<()> {
    let mut content = String::new();
    writeln!(content, "[sha512sums]")?;

    for (url, checksum) in workspace.checksums.sha512sums.iter() {
        // TODO: escape url and checksum
        writeln!(content, "\"{url}\"=\"{checksum}\"")?;
    }

    let checksums_filename = workspace
        .tool_tool_dir()
        .join(configuration::CHECKSUM_FILE_NAME);
    let mut checksums_file = workspace.adapter().create_file(&checksums_filename)?;
    checksums_file.write_all(content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{ToolToolConfiguration, CHECKSUM_FILE_NAME, TOOL_TOOL_DIRECTORY};
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
            [sha512sums]
            "foo"="bar"
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

    #[test]
    fn test_save_checksums() -> ToolToolResult<()> {
        let adapter = MockAdapter::new();
        let config = ToolToolConfiguration {
            tools: vec![],
        };

        let adapter_rc = Rc::new(adapter);
        let mut workspace = Workspace::new(config, adapter_rc.clone());
        workspace.checksums.sha512sums.insert("foo".to_string(), "bar".to_string());
        workspace.checksums.sha512sums.insert("http://example.com/?query=%22foo%22".to_string(), "baa1a3fc26533eb1578adee93b38044fb06e273ed90d23e52b686b9af59792440fc18ba3334d9050dfb07a223744cfa156747dbaef74b65349b806ffa739070e".to_string());
        save_checksums(&mut workspace)?;
        adapter_rc.verify_effects(
        expect![[r#"
            CREATE FILE: .tool-tool/v2/checksums.toml
            WRITE FILE: .tool-tool/v2/checksums.toml -> [sha512sums]
            "foo"="bar"
            "http://example.com/?query=%22foo%22"="baa1a3fc26533eb1578adee93b38044fb06e273ed90d23e52b686b9af59792440fc18ba3334d9050dfb07a223744cfa156747dbaef74b65349b806ffa739070e"

        "#]]);
        Ok(())
    }
}
