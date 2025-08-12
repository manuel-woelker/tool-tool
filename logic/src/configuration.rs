use crate::configuration::platform::DownloadPlatform;
use std::collections::BTreeMap;

pub mod expand_config;
pub mod parse_config;
pub mod platform;

#[derive(Debug)]
pub struct ToolConfiguration {
    pub name: String,
    pub version: String,
    pub download_urls: BTreeMap<DownloadPlatform, String>,
    pub commands: BTreeMap<String, String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct ToolToolConfiguration {
    pub tools: Vec<ToolConfiguration>,
}
