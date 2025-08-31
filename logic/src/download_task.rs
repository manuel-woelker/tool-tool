use crate::configuration::platform::DownloadPlatform;
use crate::file_type::get_file_type_from_url;
use crate::hash::compute_sha512;
use crate::workspace::Workspace;
use tool_tool_base::result::{ToolToolResult, err};

pub fn run_download_task(context: &Workspace) -> ToolToolResult<()> {
    let adapter = context.adapter();
    // create .tool-tool directory if it doesn't exist
    let tool_tool_dir = context.tool_tool_dir();
    // TODO: make random temp dir
    let temp_dir = tool_tool_dir.join("tmp");
    adapter.create_directory_all(&temp_dir)?;
    adapter.create_directory_all(&tool_tool_dir)?;
    let host_platform = adapter.get_platform();
    let config = context.config();
    for tool in config.tools.iter() {
        let tool_path = tool_tool_dir.join(format!("{}-{}", tool.name, tool.version));
        adapter.create_directory_all(&tool_path)?;
        let mut tool_platform = host_platform;
        let download_artifact = tool
            .download_urls
            .get(&tool_platform)
            .or_else(|| {
                tool_platform = DownloadPlatform::Default;
                tool.download_urls.get(&tool_platform)
            })
            .ok_or_else(|| {
                err!(
                    "No download url found for tool '{}' on platform '{host_platform}'",
                    tool.name
                )
            })?;
        let download_path = temp_dir.join(format!("download-{}-{}", tool.name, tool.version));
        adapter.download_file(&download_artifact.url, &download_path)?;
        let mut download_file = adapter.read_file(&download_path)?;
        // TODO: compute and verify checksum
        let _sha512 = compute_sha512(download_file.as_mut())?;

        // get file type
        let _file_type = get_file_type_from_url(&download_artifact.url);
        dbg!(_file_type);
    }
    Ok(())
}
