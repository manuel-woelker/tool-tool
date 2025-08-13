use tool_tool_base::result::ToolToolResult;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;

pub fn run_cli() -> ToolToolResult<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::ENTER)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let registry = tracing_subscriber::registry().with(fmt_layer);

    tracing::subscriber::set_global_default(registry)
        .expect("setting default logging subscriber failed");

    let adapter = tool_tool_real_adapter::RealAdapter::new();
    let mut runner = tool_tool_logic::runner::ToolToolRunner::new(adapter);
    runner.run();
    Ok(())
}
