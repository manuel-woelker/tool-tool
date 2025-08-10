use std::fmt::Debug;
use tool_tool_logic::adapter::Adapter;

pub struct RealAdapter {}

impl Adapter for RealAdapter {
    fn get_args(&self) -> Vec<String> {
        todo!()
    }

    fn print(&self, _message: &str) {
        todo!()
    }

    fn exit(&self, _exit_code: i32) {
        todo!()
    }
}

impl Debug for RealAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RealAdapter")
    }
}
