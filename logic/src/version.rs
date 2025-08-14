pub const TOOL_TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SUFFIX: Option<&str> = option_env!("TOOL_TOOL_REVISION");

pub fn get_version() -> String {
    if cfg!(test) {
        "vTEST".to_string()
    } else {
        let suffix = GIT_SUFFIX.unwrap_or("dev");
        format!("{TOOL_TOOL_VERSION}-{suffix}")
    }
}
