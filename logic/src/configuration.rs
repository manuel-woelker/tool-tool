use crate::configuration::platform::DownloadPlatform;
use std::collections::BTreeMap;
use std::fmt::Display;

pub mod expand_config;
pub mod parse_config;
pub mod platform;

#[derive(Debug)]
pub struct DownloadArtifact {
    pub url: String,
}

#[derive(Debug)]
pub struct ToolConfiguration {
    pub name: String,
    pub version: String,
    pub download_urls: BTreeMap<DownloadPlatform, DownloadArtifact>,
    pub commands: BTreeMap<String, String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct ToolToolConfiguration {
    pub tools: Vec<ToolConfiguration>,
}

impl ToolToolConfiguration {
    pub(crate) fn initial() -> Self {
        Self { tools: Vec::new() }
    }
}

impl Display for DownloadArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}
