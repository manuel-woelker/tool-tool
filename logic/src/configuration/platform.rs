use std::fmt::Display;
use std::str::FromStr;
use tool_tool_base::result::{ToolToolError, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum DownloadPlatform {
    Default,
    Linux,
    MacOS,
    Windows,
}
impl FromStr for DownloadPlatform {
    type Err = ToolToolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(DownloadPlatform::Default),
            "windows" => Ok(DownloadPlatform::Windows),
            "linux" => Ok(DownloadPlatform::Linux),
            "macos" => Ok(DownloadPlatform::MacOS),
            other => bail!("Unknown download platform: '{other}'"),
        }
    }
}

impl Display for DownloadPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadPlatform::Default => write!(f, "default"),
            DownloadPlatform::Windows => write!(f, "windows"),
            DownloadPlatform::Linux => write!(f, "linux"),
            DownloadPlatform::MacOS => write!(f, "macos"),
        }
    }
}
