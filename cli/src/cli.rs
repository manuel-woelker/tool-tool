use std::env::current_exe;
use std::path::PathBuf;
use tool_tool_base::result::{ToolToolResult, bail, err};
use tool_tool_logic::runner::CONFIG_FILENAME;
use tracing::info;
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
    let base_path = find_base_path()?;
    info!("Using base path: '{:?}'", base_path);
    let adapter = tool_tool_real_adapter::RealAdapter::new(base_path.to_path_buf());
    let mut runner = tool_tool_logic::runner::ToolToolRunner::new(adapter);
    runner.run();
    Ok(())
}

fn find_base_path() -> ToolToolResult<PathBuf> {
    let current_exe = current_exe()?;
    let mut candidate_path = current_exe.clone();
    let exe_parent = candidate_path
        .parent()
        .ok_or_else(|| {
            err!(
                "Could not find parent of tool-tool executable '{:?}'",
                &current_exe
            )
        })?
        .to_path_buf();
    loop {
        let Some(parent_path) = candidate_path.parent() else {
            break;
        };
        let config_path = parent_path.join(CONFIG_FILENAME);
        if config_path.exists() && config_path.is_file() {
            return Ok(parent_path.to_path_buf());
        }
        candidate_path = parent_path.to_path_buf();
    }
    bail!(
        "Could not find config file '{CONFIG_FILENAME}' base path from tool-tool executable '{:?}'",
        exe_parent
    )
}
