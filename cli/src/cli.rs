use std::env::current_dir;
use std::path::PathBuf;
use tool_tool_base::result::{Context, ToolToolResult, bail};
use tool_tool_logic::configuration::CONFIGURATION_FILE_NAME;
use tracing::info;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;

pub fn run_cli() -> ToolToolResult<()> {
    if let Err(err) = enable_ansi_support::enable_ansi_support() {
        eprintln!("Failed to enable ANSI support: {err}");
    }

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::ENTER)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let registry = tracing_subscriber::registry().with(fmt_layer);

    tracing::subscriber::set_global_default(registry)
        .expect("setting default logging subscriber failed");
    let base_path = find_base_path()?;
    info!("Using base path: '{:?}'", base_path);
    let adapter = tool_tool_real_adapter::RealAdapter::new(base_path.to_path_buf());
    let runner = tool_tool_logic::runner_initial::ToolToolRunnerInitial::new(adapter);
    runner.run();
    Ok(())
}

fn find_base_path() -> ToolToolResult<PathBuf> {
    let working_directory = current_dir().with_context(|| "Failed to get working directory")?;
    let mut candidate_path = working_directory.clone();
    loop {
        let config_path = candidate_path.join(CONFIGURATION_FILE_NAME);
        if config_path.exists() && config_path.is_file() {
            return Ok(candidate_path.to_path_buf());
        }
        let Some(parent_path) = candidate_path.parent() else {
            break;
        };
        candidate_path = parent_path.to_path_buf();
    }
    bail!(
        "Could not find config file '{CONFIGURATION_FILE_NAME}' base path from working directory '{:?}'",
        working_directory
    )
}
