use crate::adapter::Adapter;
use expect_test::Expect;
use indent::indent_all_with;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct MockAdapter {
    inner: Arc<RwLock<MockAdapterInner>>,
}

struct MockAdapterInner {
    args: Vec<String>,
    effects_string: String,
}

impl MockAdapter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MockAdapterInner {
                args: Vec::new(),
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
    }

    pub fn set_args(&self, args: &[&str]) {
        self.write().args = args.iter().map(|s| s.to_string()).collect();
    }

    pub fn verify_effects(&self, expected: Expect) {
        expected.assert_eq(&self.read().effects_string);
    }

    pub fn get_effects(&self) -> String {
        self.read().effects_string.clone()
    }
}

impl Adapter for MockAdapter {
    fn get_args(&self) -> Vec<String> {
        self.read().args.clone()
    }

    fn print(&self, message: &str) {
        self.log_effect(format!("PRINT:\n{}\n", indent_all_with("\t", message)));
    }

    fn exit(&self, exit_code: i32) {
        self.log_effect(format!("EXIT: {}\n", exit_code));
    }
}

impl std::fmt::Debug for MockAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockAdapter")
    }
}
