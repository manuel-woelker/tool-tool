use crate::checksums::save_checksums;
use crate::configuration::ToolConfiguration;
use crate::file_type::{FileType, get_file_type_from_url};
use crate::hash::compute_sha512;
use crate::workspace::Workspace;
use flate2::read::GzDecoder;
use relative_path::RelativePathBuf;
use std::collections::BTreeMap;
use std::io::Read;
use tar::EntryType;
use tool_tool_base::result::{ToolToolResult, err};
use tracing::info;

type Sha512Sums = BTreeMap<String, String>;

pub fn run_download_task(workspace: &mut Workspace) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    let sha512sums = &workspace.checksums.sha512sums;
    let mut new_sha512sums = sha512sums.clone();
    // create .tool-tool directory if it doesn't exist
    let config = workspace.config();
    // Download artifacts for current host
    for tool in config.tools.iter() {
        download_tool(workspace, tool, &mut new_sha512sums)?;
    }

    // Download missing artifacts to complete checksums
    for tool in config.tools.iter() {
        for (platform, artifact) in tool.download_urls.iter() {
            if !new_sha512sums.contains_key(&artifact.url) {
                let temp_dir = workspace.create_temp_dir(&tool.name)?;
                let download_path = temp_dir.join(format!(
                    "download-{}-{}-{}",
                    tool.name, tool.version, platform
                ));
                info!(
                    "Downloading {} to {} for checksum generation",
                    artifact.url, download_path
                );
                adapter.download_file(&artifact.url, &download_path)?;
                let mut download_file = adapter.read_file(&download_path)?;
                let sha512 = compute_sha512(download_file.as_mut())?;
                new_sha512sums.insert(artifact.url.clone(), sha512);
                adapter.delete_directory_all(&temp_dir)?;
            }
        }
    }
    if &new_sha512sums != sha512sums {
        workspace.checksums.sha512sums = new_sha512sums;
        save_checksums(workspace)?;
    }
    Ok(())
}

fn download_tool(
    workspace: &Workspace,
    tool: &ToolConfiguration,
    new_sha512sums: &mut Sha512Sums,
) -> ToolToolResult<()> {
    let cache_dir = workspace.cache_dir();
    let host_platform = workspace.adapter().get_platform();
    let sha512sums = &workspace.checksums.sha512sums;
    let adapter = workspace.adapter();
    let tool_path = cache_dir.join(format!("{}-{}", tool.name, tool.version));
    let download_artifact = tool
        .download_urls
        .get(&host_platform)
        .or(tool.default_download_artifact.as_ref())
        .ok_or_else(|| {
            err!(
                "No download url found for tool '{}' on platform '{host_platform}'",
                tool.name
            )
        })?;
    // Determine if tool is already downloaded
    let checksum_path = tool_path.join(".tool-tool.sha512");
    if let Some(expected_sha512) = sha512sums.get(&download_artifact.url) {
        if adapter.file_exists(&checksum_path)? {
            let mut checksum_file = adapter.read_file(&checksum_path)?;
            let mut checksum = String::new();
            checksum_file.read_to_string(&mut checksum)?;
            if checksum != *expected_sha512 {
                info!("Checksum mismatch for tool '{}', re-downloading", tool.name);
            } else {
                info!("Checksum match for tool '{}', skipping download", tool.name);
                return Ok(());
            }
        }
    }
    // TODO: make random temp dir
    let temp_dir = workspace.create_temp_dir(&tool.name)?;
    if adapter.file_exists(&temp_dir)? {
        adapter.delete_directory_all(&temp_dir)?;
    }
    adapter.create_directory_all(&temp_dir)?;
    if adapter.file_exists(&tool_path)? {
        adapter.delete_directory_all(&tool_path)?;
    }
    adapter.create_directory_all(&tool_path)?;
    let download_path = temp_dir.join(format!(
        "download-{}-{}-{}",
        tool.name, tool.version, host_platform
    ));
    info!("Downloading {} to {}", download_artifact.url, download_path);
    adapter.download_file(&download_artifact.url, &download_path)?;
    let mut download_file = adapter.read_file(&download_path)?;
    // Compute and verify checksum
    let sha512 = compute_sha512(download_file.as_mut())?;
    if let Some(expected_sha512) = sha512sums.get(&download_artifact.url) {
        if sha512 != *expected_sha512 {
            return Err(err!(
                "Checksum mismatch for tool '{}'\nExpected: {}\nActual:   {}",
                tool.name,
                expected_sha512,
                sha512
            ));
        }
    } else {
        info!(
            "Checksum not found for tool '{}' ({}) adding it",
            tool.name, host_platform
        );
        new_sha512sums.insert(download_artifact.url.clone(), sha512.clone());
    }

    adapter.delete_directory_all(&tool_path)?;
    // get file type
    let file_type = get_file_type_from_url(&download_artifact.url);
    extract_tool(workspace, &tool_path, &download_path, file_type)?;

    adapter.delete_directory_all(&temp_dir)?;
    // Last step is to create the checksum file
    let mut checksum_file = adapter.create_file(&checksum_path)?;
    checksum_file.write_all(sha512.as_bytes())?;
    Ok(())
}

fn extract_tool(
    workspace: &Workspace,
    tool_path: &RelativePathBuf,
    download_path: &RelativePathBuf,
    file_type: FileType,
) -> ToolToolResult<()> {
    match file_type {
        FileType::Zip => {
            extract_zip(workspace, download_path, tool_path)?;
        }
        FileType::TarGz => {
            extract_targz(workspace, download_path, tool_path)?;
        }
        FileType::Other => {
            todo!()
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

fn extract_targz(
    workspace: &Workspace,
    targz_path: &RelativePathBuf,
    destination_path: &RelativePathBuf,
) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    let mut archive = tar::Archive::new(GzDecoder::new(adapter.read_file(targz_path)?));
    for archive_entry in archive.entries()? {
        let mut archive_entry = archive_entry?;
        let outpath = archive_entry.path()?;

        // TODO: check file does not escape
        let relative_path_buf = RelativePathBuf::from_path(outpath)?;
        // TODO: make skip_components configurable
        let mut components = relative_path_buf.components();
        components.next();
        let relative_path_buf = components.as_relative_path();
        let joined_path = destination_path.join(relative_path_buf);
        match archive_entry.header().entry_type() {
            EntryType::Directory => {
                adapter.create_directory_all(&joined_path)?;
            }
            EntryType::Regular => {
                if let Some(parent_path) = joined_path.parent() {
                    adapter.create_directory_all(&parent_path.to_relative_path_buf())?;
                }
                let mut outfile = adapter.create_file(&joined_path)?;
                std::io::copy(&mut archive_entry, &mut outfile)?;
            }
            _ => {}
        }
    }
    Ok(())
}
