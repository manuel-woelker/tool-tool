pub type ToolToolError = anyhow::Error;
pub type ToolToolResult<T> = Result<T, ToolToolError>;
pub use anyhow::Context;
pub use anyhow::bail;
