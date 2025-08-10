use crate::cli::run_cli;
use tool_tool_logic::version::get_version;

pub mod cli;

fn main() {
    match run_cli() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("ERROR running tool-tool ({}):\n{}\n", get_version(), e);
            std::process::exit(1);
        }
    }
}
