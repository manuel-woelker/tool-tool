use tool_tool_base::result::ToolToolResult;

pub(crate) trait ArchiveBuilder: Default {
    fn add_file(&mut self, path: impl AsRef<str>, content: impl AsRef<[u8]>) -> ToolToolResult<()>;

    fn add_directory(&mut self, path: impl AsRef<str>) -> ToolToolResult<()>;

    fn build(self) -> ToolToolResult<Vec<u8>>;
}
