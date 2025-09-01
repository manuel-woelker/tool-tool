use crate::configuration::platform::DownloadPlatform;
use crate::file_type::{FileType, get_file_type_from_url};
use crate::hash::compute_sha512;
use crate::workspace::Workspace;
use relative_path::RelativePathBuf;
use tool_tool_base::result::{ToolToolResult, err};

pub fn run_download_task(workspace: &Workspace) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    // create .tool-tool directory if it doesn't exist
    let tool_tool_dir = workspace.tool_tool_dir();
    // TODO: make random temp dir
    let temp_dir = tool_tool_dir.join("tmp");
    adapter.create_directory_all(&temp_dir)?;
    adapter.create_directory_all(&tool_tool_dir)?;
    let host_platform = adapter.get_platform();
    let config = workspace.config();
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
        adapter.delete_directory_all(&tool_path)?;
        let file_type = get_file_type_from_url(&download_artifact.url);
        match file_type {
            FileType::Zip => {
                extract_zip(workspace, &download_path, &tool_path)?;
            }
            FileType::TarGz => {
                todo!()
            }
            FileType::Other => {
                todo!()
            }
        }
    }
    Ok(())
}

fn extract_zip(
    workspace: &Workspace,
    zip_path: &RelativePathBuf,
    destination_path: &RelativePathBuf,
) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    let mut archive = zip::ZipArchive::new(adapter.read_file(zip_path)?)?;

    for i in 0..archive.len() {
        let mut zip_entry = archive.by_index(i).unwrap();
        let outpath = match zip_entry.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        // TODO: check file does not escape
        let relative_path_buf = RelativePathBuf::from_path(outpath)?;
        // TODO: make skip_components configurable
        let mut components = relative_path_buf.components();
        components.next();
        let relative_path_buf = components.as_relative_path();
        let joined_path = destination_path.join(relative_path_buf);
        if zip_entry.is_dir() {
            adapter.create_directory_all(&joined_path)?;
        } else {
            if let Some(parent_path) = joined_path.parent() {
                adapter.create_directory_all(&parent_path.to_relative_path_buf())?;
            }
            let mut outfile = adapter.create_file(&joined_path)?;
            std::io::copy(&mut zip_entry, &mut outfile)?;
        }
    }
    Ok(())
}
