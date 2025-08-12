pub type ToolToolError = miette::Error;
pub type ToolToolResult<T> = Result<T, ToolToolError>;
pub use miette::Context;
pub use miette::bail;
pub use miette::miette as err;
