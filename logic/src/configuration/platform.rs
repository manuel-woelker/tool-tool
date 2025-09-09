use std::fmt::Display;
use std::str::FromStr;
use tool_tool_base::result::{ToolToolError, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum DownloadPlatform {
    Linux,
    MacOS,
    Windows,
}

impl DownloadPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadPlatform::Windows => "windows",
            DownloadPlatform::Linux => "linux",
            DownloadPlatform::MacOS => "macos",
        }
    }

    pub fn get_executable_extensions(&self) -> &'static [&'static str] {
        match self {
            DownloadPlatform::Windows => &[".exe", ".bat", ".cmd"],
            DownloadPlatform::Linux => &[""],
            DownloadPlatform::MacOS => &[""],
        }
    }
}
impl FromStr for DownloadPlatform {
    type Err = ToolToolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "windows" => Ok(DownloadPlatform::Windows),
            "linux" => Ok(DownloadPlatform::Linux),
            "macos" => Ok(DownloadPlatform::MacOS),
            other => bail!("Unknown download platform: '{other}'"),
        }
    }
}

impl Display for DownloadPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
