pub mod adapter;
#[cfg(test)]
mod mock_adapter;
pub mod runner;

pub const TOOL_TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
