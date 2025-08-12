use crate::adapter::Adapter;
use crate::types::FilePath;
use expect_test::Expect;
use indent::indent_all_with;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tool_tool_base::result::ToolToolResult;

#[derive(Clone)]
pub struct MockAdapter {
    inner: Arc<RwLock<MockAdapterInner>>,
}

struct MockAdapterInner {
    configuration_string: String,
    args: Vec<String>,
    effects_string: String,
}

impl MockAdapter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MockAdapterInner {
                args: Vec::new(),
                configuration_string: r#"
                    tools {
                        lsd "0.17.0" {
                            download {
                                linux "https://github.com/Peltoche/lsd/releases/download/${version}/lsd-${version}-x86_64-unknown-linux-gnu.tar.gz"
                                windows "https://github.com/Peltoche/lsd/releases/download/${version}/lsd-${version}-x86_64-pc-windows-msvc.zip"
                            }
                            commands {
                                foobar "echo foobar"
                                bar "echo bar"
                            }
                            env {
                                FROBNIZZ "nizzle"
                                FIZZ "buzz"
                            }
                       }
                    }
                       "#.to_string(),
                effects_string: String::new(),
            })),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, MockAdapterInner> {
        self.inner
            .read()
            .expect("Unable to acquire read lock for mock adapter")
    }

    fn write(&self) -> RwLockWriteGuard<'_, MockAdapterInner> {
        self.inner
            .write()
            .expect("Unable to acquire write lock for mock adapter")
    }

    fn log_effect(&self, effect: impl AsRef<str>) {
        self.write().effects_string.push_str(effect.as_ref());
        self.write().effects_string.push('\n');
    }

    pub fn set_args(&self, args: &[&str]) {
        let mut all_args = vec!["./tool-tool.exe".to_string()];
        all_args.extend(args.iter().map(|s| s.to_string()));
        self.write().args = all_args;
    }

    pub fn set_configuration(&self, configuration: impl Into<String>) {
        self.write().configuration_string = configuration.into();
    }

    pub fn verify_effects(&self, expected: Expect) {
        expected.assert_eq(&self.read().effects_string);
    }

    pub fn get_effects(&self) -> String {
        self.read().effects_string.clone()
    }
}

impl Adapter for MockAdapter {
    fn args(&self) -> Vec<String> {
        self.read().args.clone()
    }

    fn print(&self, message: &str) {
        self.log_effect(format!("PRINT:\n{}", indent_all_with("\t", message)));
    }

    fn read_file(&self, path: &FilePath) -> ToolToolResult<String> {
        self.log_effect(format!("READ FILE: {path}"));
        Ok(self.read().configuration_string.clone())
    }

    fn exit(&self, exit_code: i32) {
        self.log_effect(format!("EXIT: {}", exit_code));
    }
}

impl std::fmt::Debug for MockAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockAdapter")
    }
}
