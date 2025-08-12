use tool_tool_base::result::ToolToolResult;

pub fn run_cli() -> ToolToolResult<()> {
    let adapter = tool_tool_real_adapter::RealAdapter::new();
    let mut runner = tool_tool_logic::runner::ToolToolRunner::new(adapter);
    runner.run();
    Ok(())
}
