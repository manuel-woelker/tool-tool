use crate::adapter::DownloadRequest;
use crate::checksums::save_checksums;
use crate::configuration::ToolConfiguration;
use crate::file_type::{FileType, get_file_type_from_url};
use crate::hash::compute_sha512;
use crate::workspace::Workspace;
use flate2::read::GzDecoder;
use relative_path::RelativePathBuf;
use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use tar::EntryType;
use tool_tool_base::result::{ToolToolResult, err};
use tracing::{debug, info};

type Sha512Sums = BTreeMap<String, String>;

pub fn run_download_task(workspace: &mut Workspace) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    let sha512sums = &workspace.checksums.sha512sums;
    let mut new_sha512sums = sha512sums.clone();
    let config = workspace.config();
    let host_platform = adapter.get_platform();
    let mut tool_downloads = Vec::new();
    let mut checksum_downloads = Vec::new();
    let mut host_download_urls = HashSet::new();

    for tool in &config.tools {
        let artifact = tool
            .download_urls
            .get(&host_platform)
            .or(tool.default_download_artifact.as_ref())
            .ok_or_else(|| {
                err!(
                    "No download url found for tool '{}' on platform '{host_platform}'",
                    tool.name
                )
            })?;
        let tool_path = workspace.tool_dir(tool);
        let checksum_path = tool_path.join(".tool-tool.sha512");
        if tool_is_current(workspace, tool, &artifact.url, &checksum_path)? {
            continue;
        }

        let temp_dir = workspace.create_temp_dir(&tool.name)?;
        let download_path = temp_dir.join(format!(
            "download-{}-{}-{}",
            tool.name, tool.version, host_platform
        ));
        host_download_urls.insert(artifact.url.clone());
        tool_downloads.push(PlannedToolDownload {
            tool,
            url: artifact.url.clone(),
            tool_path,
            temp_dir,
            download_path,
        });
    }

    let mut planned_checksum_urls = HashSet::new();
    for tool in &config.tools {
        for (platform, artifact) in &tool.download_urls {
            if !new_sha512sums.contains_key(&artifact.url)
                && !host_download_urls.contains(&artifact.url)
                && planned_checksum_urls.insert(artifact.url.clone())
            {
                let temp_dir = workspace.create_temp_dir(&tool.name)?;
                let download_path = temp_dir.join(format!(
                    "download-{}-{}-{}",
                    tool.name, tool.version, platform
                ));
                checksum_downloads.push(PlannedChecksumDownload {
                    url: artifact.url.clone(),
                    temp_dir,
                    download_path,
                });
            }
        }
    }

    let requests = tool_downloads
        .iter()
        .map(PlannedToolDownload::request)
        .chain(
            checksum_downloads
                .iter()
                .map(PlannedChecksumDownload::request),
        )
        .collect::<Vec<_>>();
    let download_results = adapter.download_files(&requests)?;
    let (tool_results, checksum_results) = download_results.split_at(tool_downloads.len());
    let mut errors = Vec::new();

    for (download, result) in tool_downloads.iter().zip(tool_results) {
        let result = result
            .as_ref()
            .map_err(|error| err!("{error:#}"))
            .and_then(|_| process_tool_download(workspace, download, &mut new_sha512sums));
        if let Err(error) = result {
            errors.push(error);
        }
    }
    for (download, result) in checksum_downloads.iter().zip(checksum_results) {
        let result = result
            .as_ref()
            .map_err(|error| err!("{error:#}"))
            .and_then(|_| {
                let mut download_file = adapter.read_file(&download.download_path)?;
                let sha512 = compute_sha512(download_file.as_mut())?;
                new_sha512sums.insert(download.url.clone(), sha512);
                Ok(())
            });
        if let Err(error) = result {
            errors.push(error);
        }
    }
    collect_cleanup_errors(workspace, &tool_downloads, &checksum_downloads, &mut errors);

    if &new_sha512sums != sha512sums {
        workspace.checksums.sha512sums = new_sha512sums;
        save_checksums(workspace)?;
    }
    match errors.len() {
        0 => Ok(()),
        1 => Err(errors.pop().unwrap()),
        _ => Err(err!(
            "One or more downloads failed:\n{}",
            errors
                .iter()
                .map(|error| format!("- {error:#}"))
                .collect::<Vec<_>>()
                .join("\n")
        )),
    }
}

struct PlannedToolDownload<'a> {
    tool: &'a ToolConfiguration,
    url: String,
    tool_path: RelativePathBuf,
    temp_dir: RelativePathBuf,
    download_path: RelativePathBuf,
}

impl PlannedToolDownload<'_> {
    fn request(&self) -> DownloadRequest {
        DownloadRequest {
            url: self.url.clone(),
            destination_path: self.download_path.clone(),
        }
    }
}

struct PlannedChecksumDownload {
    url: String,
    temp_dir: RelativePathBuf,
    download_path: RelativePathBuf,
}

impl PlannedChecksumDownload {
    fn request(&self) -> DownloadRequest {
        DownloadRequest {
            url: self.url.clone(),
            destination_path: self.download_path.clone(),
        }
    }
}

fn tool_is_current(
    workspace: &Workspace,
    tool: &ToolConfiguration,
    url: &str,
    checksum_path: &RelativePathBuf,
) -> ToolToolResult<bool> {
    let adapter = workspace.adapter();
    if let Some(expected_sha512) = workspace.checksums.sha512sums.get(url)
        && adapter.file_exists(checksum_path)?
    {
        let mut checksum_file = adapter.read_file(checksum_path)?;
        let mut checksum = String::new();
        checksum_file.read_to_string(&mut checksum)?;
        if checksum == *expected_sha512 {
            info!("Checksum match for tool '{}', skipping download", tool.name);
            return Ok(true);
        }
        info!("Checksum mismatch for tool '{}', re-downloading", tool.name);
    }
    Ok(false)
}

fn process_tool_download(
    workspace: &Workspace,
    download: &PlannedToolDownload<'_>,
    new_sha512sums: &mut Sha512Sums,
) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    let mut download_file = adapter.read_file(&download.download_path)?;
    let sha512 = compute_sha512(download_file.as_mut())?;
    debug!("Checksum for tool '{}': {}", download.tool.name, sha512);
    if let Some(expected_sha512) = workspace.checksums.sha512sums.get(&download.url) {
        if sha512 != *expected_sha512 {
            return Err(err!(
                "Checksum mismatch for tool '{}'\nExpected: {}\nActual:   {}",
                download.tool.name,
                expected_sha512,
                sha512
            ));
        }
    } else {
        info!(
            "Checksum not found for tool '{}' ({}) adding it",
            download.tool.name,
            adapter.get_platform()
        );
        new_sha512sums.insert(download.url.clone(), sha512.clone());
    }

    let staging_path = download.temp_dir.join("extracted");
    debug!("Extracting tool '{}'", download.tool.name);
    extract_tool(
        workspace,
        download.tool,
        &staging_path,
        &download.download_path,
        get_file_type_from_url(&download.url),
    )?;
    let mut checksum_file = adapter.create_file(&staging_path.join(".tool-tool.sha512"))?;
    checksum_file.write_all(sha512.as_bytes())?;
    drop(checksum_file);
    replace_tool_directory(
        workspace,
        &staging_path,
        &download.tool_path,
        &download.temp_dir.join("previous"),
    )?;
    Ok(())
}

fn replace_tool_directory(
    workspace: &Workspace,
    staging_path: &RelativePathBuf,
    tool_path: &RelativePathBuf,
    backup_path: &RelativePathBuf,
) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    let had_previous_installation = adapter.file_exists(tool_path)?;
    if had_previous_installation {
        adapter.rename_path(tool_path, backup_path)?;
    }

    if let Err(install_error) = adapter.rename_path(staging_path, tool_path) {
        if had_previous_installation
            && let Err(rollback_error) = adapter.rename_path(backup_path, tool_path)
        {
            return Err(err!(
                "Failed to install staged tool: {install_error:#}\nFailed to restore previous installation: {rollback_error:#}"
            ));
        }
        return Err(install_error);
    }

    if had_previous_installation {
        adapter.delete_directory_all(backup_path)?;
    }
    Ok(())
}

fn collect_cleanup_errors(
    workspace: &Workspace,
    tool_downloads: &[PlannedToolDownload<'_>],
    checksum_downloads: &[PlannedChecksumDownload],
    errors: &mut Vec<tool_tool_base::result::ToolToolError>,
) {
    let adapter = workspace.adapter();
    for temp_dir in tool_downloads
        .iter()
        .map(|download| &download.temp_dir)
        .chain(checksum_downloads.iter().map(|download| &download.temp_dir))
    {
        if let Err(error) = adapter.delete_directory_all(temp_dir) {
            errors.push(error);
        }
    }
}

fn extract_tool(
    workspace: &Workspace,
    tool: &ToolConfiguration,
    tool_path: &RelativePathBuf,
    download_path: &RelativePathBuf,
    file_type: FileType,
) -> ToolToolResult<()> {
    match file_type {
        FileType::Zip => {
            extract_zip(workspace, download_path, tool_path)?;
        }
        FileType::TarGz => {
            let adapter = workspace.adapter();
            extract_tar(
                workspace,
                tool_path,
                GzDecoder::new(adapter.read_file(download_path)?),
                GzDecoder::new(adapter.read_file(download_path)?),
            )?;
        }
        FileType::TarXz => {
            let adapter = workspace.adapter();
            extract_tar(
                workspace,
                tool_path,
                liblzma::read::XzDecoder::new(adapter.read_file(download_path)?),
                liblzma::read::XzDecoder::new(adapter.read_file(download_path)?),
            )?;
        }
        FileType::TarZstd => {
            let adapter = workspace.adapter();
            extract_tar(
                workspace,
                tool_path,
                zstd::stream::read::Decoder::new(adapter.read_file(download_path)?)?,
                zstd::stream::read::Decoder::new(adapter.read_file(download_path)?)?,
            )?;
        }
        FileType::Exe => {
            extract_exe(
                workspace,
                download_path,
                &tool_path.join(format!("{}.exe", tool.name)),
            )?;
        }
        FileType::None => {
            extract_exe(workspace, download_path, &tool_path.join(&tool.name))?;
        }
        FileType::Unknown => {
            return Err(err!(
                "Could not determine file type for '{}'",
                download_path
            ));
        }
        FileType::Other(extension) => {
            return Err(err!("Unsupported file extension: '{}'", extension));
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
    let mut common_root: Option<OsString> = None;
    let mut has_top_level_file = false;
    let mut links = Vec::new();

    for i in 0..archive.len() {
        let mut zip_entry = archive.by_index(i)?;
        let path = zip_entry
            .enclosed_name()
            .ok_or_else(|| err!("Archive contains unsafe path: '{}'", zip_entry.name()))?;
        validate_archive_path(zip_entry.name(), &path)?;
        if zip_entry.is_symlink() {
            let mut target = String::new();
            zip_entry.read_to_string(&mut target)?;
            links.push(ArchiveLink {
                path: path.clone(),
                target: PathBuf::from(target),
                hard: false,
            });
        }
        let mut components = path.components();
        let Some(first) = components.next() else {
            continue;
        };
        let first = first.as_os_str().to_owned();
        if common_root.as_ref().is_some_and(|root| root != &first) {
            common_root = None;
            has_top_level_file = true;
            continue;
        }
        common_root.get_or_insert(first);
        has_top_level_file |= !zip_entry.is_dir() && components.next().is_none();
    }
    let strip_common_root = common_root.is_some() && !has_top_level_file;
    let links = links
        .into_iter()
        .map(|link| prepare_archive_link(link, strip_common_root, destination_path))
        .collect::<ToolToolResult<Vec<_>>>()?;

    for i in 0..archive.len() {
        let mut zip_entry = archive.by_index(i).unwrap();
        let outpath = zip_entry
            .enclosed_name()
            .ok_or_else(|| err!("Archive contains unsafe path: '{}'", zip_entry.name()))?;
        validate_archive_path(zip_entry.name(), &outpath)?;

        let relative_path_buf = RelativePathBuf::from_path(outpath)?;
        let mut components = relative_path_buf.components();
        if strip_common_root {
            components.next();
        }
        let relative_path_buf = components.as_relative_path();
        let joined_path = destination_path.join(relative_path_buf);
        if zip_entry.is_symlink() {
            continue;
        } else if zip_entry.is_dir() {
            adapter.create_directory_all(&joined_path)?;
        } else {
            if let Some(parent_path) = joined_path.parent() {
                adapter.create_directory_all(&parent_path.to_relative_path_buf())?;
            }
            let mut outfile = adapter.create_file(&joined_path)?;
            std::io::copy(&mut zip_entry, &mut outfile)?;
        }
    }
    create_archive_links(adapter, &links)?;
    Ok(())
}

fn extract_tar<R: Read>(
    workspace: &Workspace,
    destination_path: &RelativePathBuf,
    first_pass: R,
    second_pass: R,
) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    let mut archive = tar::Archive::new(first_pass);
    let mut common_root: Option<OsString> = None;
    let mut has_top_level_file = false;
    let mut links = Vec::new();

    for archive_entry in archive.entries()? {
        let archive_entry = archive_entry?;
        let path = archive_entry.path()?;
        validate_archive_path(&path.display().to_string(), &path)?;
        let entry_type = archive_entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            let target = archive_entry
                .link_name()?
                .ok_or_else(|| err!("Archive link '{}' has no target", path.display()))?;
            links.push(ArchiveLink {
                path: path.to_path_buf(),
                target: target.into_owned(),
                hard: entry_type.is_hard_link(),
            });
        }
        let mut components = path.components();
        let Some(first) = components.next() else {
            continue;
        };
        let first = first.as_os_str().to_owned();
        if common_root.as_ref().is_some_and(|root| root != &first) {
            common_root = None;
            has_top_level_file = true;
            continue;
        }
        common_root.get_or_insert(first);
        has_top_level_file |= !entry_type.is_dir() && components.next().is_none();
    }
    let strip_common_root = common_root.is_some() && !has_top_level_file;
    let links = links
        .into_iter()
        .map(|link| prepare_archive_link(link, strip_common_root, destination_path))
        .collect::<ToolToolResult<Vec<_>>>()?;

    let mut archive = tar::Archive::new(second_pass);
    for archive_entry in archive.entries()? {
        let mut archive_entry = archive_entry?;
        let outpath = archive_entry.path()?;
        validate_archive_path(&outpath.display().to_string(), &outpath)?;
        let joined_path = destination_path.join(archive_output_path(&outpath, strip_common_root)?);
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
                if let Ok(mode) = archive_entry.header().mode()
                    && mode & 0o100 != 0
                {
                    adapter.make_file_executable(&joined_path)?;
                }
            }
            _ => {}
        }
    }
    create_archive_links(adapter, &links)?;
    Ok(())
}

fn create_archive_links(
    adapter: &dyn crate::adapter::Adapter,
    links: &[PreparedArchiveLink],
) -> ToolToolResult<()> {
    for link in links {
        if let Some(parent_path) = link.path.parent() {
            adapter.create_directory_all(&parent_path.to_relative_path_buf())?;
        }
        if link.hard {
            adapter.create_hard_link(&link.path, &link.target)?;
        } else {
            adapter.create_symbolic_link(&link.path, &link.target)?;
        }
    }
    Ok(())
}

struct ArchiveLink {
    path: PathBuf,
    target: PathBuf,
    hard: bool,
}

struct PreparedArchiveLink {
    path: RelativePathBuf,
    target: RelativePathBuf,
    hard: bool,
}

fn prepare_archive_link(
    link: ArchiveLink,
    strip_common_root: bool,
    destination_path: &RelativePathBuf,
) -> ToolToolResult<PreparedArchiveLink> {
    let path = archive_output_path(&link.path, strip_common_root)?;
    let target = if link.hard {
        validate_archive_path(&link.target.display().to_string(), &link.target)?;
        destination_path.join(archive_output_path(&link.target, strip_common_root)?)
    } else {
        validate_symbolic_link_target(&path, &link.target)?;
        RelativePathBuf::from_path(&link.target)?
    };
    Ok(PreparedArchiveLink {
        path: destination_path.join(path),
        target,
        hard: link.hard,
    })
}

fn archive_output_path(path: &Path, strip_common_root: bool) -> ToolToolResult<RelativePathBuf> {
    let relative_path = RelativePathBuf::from_path(path)?;
    let mut components = relative_path.components();
    if strip_common_root {
        components.next();
    }
    Ok(components.as_relative_path().to_relative_path_buf())
}

fn validate_symbolic_link_target(link_path: &RelativePathBuf, target: &Path) -> ToolToolResult<()> {
    let mut resolved = PathBuf::from(
        link_path
            .parent()
            .map(|path| path.as_str())
            .unwrap_or_default(),
    );
    for component in target.components() {
        match component {
            Component::Normal(component) => resolved.push(component),
            Component::CurDir => {}
            Component::ParentDir if resolved.pop() => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(err!(
                    "Archive link '{}' points outside the tool directory: '{}'",
                    link_path,
                    target.display()
                ));
            }
        }
    }
    Ok(())
}

fn validate_archive_path(name: &str, path: &Path) -> ToolToolResult<()> {
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(err!("Archive contains unsafe path: '{name}'"));
    }
    Ok(())
}

fn extract_exe(
    workspace: &Workspace,
    output_directory: &RelativePathBuf,
    destination_path: &RelativePathBuf,
) -> ToolToolResult<()> {
    let adapter = workspace.adapter();
    if let Some(parent_path) = destination_path.parent() {
        adapter.create_directory_all(&parent_path.to_relative_path_buf())?;
    }
    {
        let mut infile = adapter.read_file(output_directory)?;
        let mut outfile = adapter.create_file(destination_path)?;
        std::io::copy(&mut infile, &mut outfile)?;
    }
    adapter.make_file_executable(destination_path)?;
    Ok(())
}
