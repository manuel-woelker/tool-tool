use crate::adapter::Adapter;
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use tool_tool_base::result::{ToolToolResult, bail};

pub struct LockGuard<'a> {
    adapter: &'a dyn Adapter,
}

impl Debug for LockGuard<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockGuard").finish()
    }
}

impl<'a> LockGuard<'a> {
    pub fn new(adapter: &'a dyn Adapter) -> ToolToolResult<Self> {
        let mut has_messaged = false;
        for _ in 0..60 {
            if adapter.try_lock()? {
                return Ok(Self { adapter });
            }
            if !has_messaged {
                adapter.print("Acquiring exclusive lock...");
                has_messaged = true;
            }
            adapter.sleep(Duration::from_secs(1));
        }
        bail!("Failed to acquire lock after 60 seconds")
    }
}

impl<'a> Drop for LockGuard<'a> {
    fn drop(&mut self) {
        self.adapter.unlock().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::lock_guard::LockGuard;
    use crate::mock_adapter::MockAdapter;
    use expect_test::expect;
    use tool_tool_base::result::ToolToolResult;

    #[test]
    fn lock_immediate_success() -> ToolToolResult<()> {
        let adapter = MockAdapter::new();
        let lock_guard = LockGuard::new(&adapter)?;
        drop(lock_guard);
        adapter.verify_effects(expect![[r#"
            TRY LOCK
            UNLOCK
        "#]]);
        Ok(())
    }

    #[test]
    fn lock_delayed_success() -> ToolToolResult<()> {
        let adapter = MockAdapter::new();
        adapter.set_lock_results(vec![false; 3]);
        let lock_guard = LockGuard::new(&adapter)?;
        drop(lock_guard);
        adapter.verify_effects(expect![[r#"
            TRY LOCK
            PRINT:
            	Acquiring exclusive lock...
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            UNLOCK
        "#]]);
        Ok(())
    }

    #[test]
    fn lock_while_locked() -> ToolToolResult<()> {
        let adapter = MockAdapter::new();
        adapter.set_lock_results(vec![false; 120]);
        let error = LockGuard::new(&adapter).expect_err("Expected lock to fail");
        assert!(
            error
                .to_string()
                .contains("Failed to acquire lock after 60 seconds")
        );
        expect!["Failed to acquire lock after 60 seconds"].assert_eq(&error.to_string());
        adapter.verify_effects(expect![[r#"
            TRY LOCK
            PRINT:
            	Acquiring exclusive lock...
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
            TRY LOCK
            SLEEP: 1s
        "#]]);
        Ok(())
    }
}
