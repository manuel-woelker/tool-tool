use std::env;
use std::fmt::Debug;
use tool_tool_logic::adapter::Adapter;

pub struct RealAdapter {}

impl RealAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RealAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for RealAdapter {
    fn get_args(&self) -> Vec<String> {
        env::args().collect()
    }

    fn print(&self, message: &str) {
        eprintln!("{message}");
    }

    fn exit(&self, exit_code: i32) {
        std::process::exit(exit_code);
    }
}

impl Debug for RealAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RealAdapter")
    }
}
