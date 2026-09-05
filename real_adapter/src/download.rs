use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use tool_tool_base::result::{Context, ToolToolResult};
use ureq::tls::{RootCerts, TlsConfig};

pub struct Downloader {
    agent: ureq::Agent,
    show_progress: bool,
}

const QUEUED_TEMPLATE: &str =
    "{spinner:.dim} {prefix:<11} {msg:40!}  {'':32}  {'':21}  {'':12}  {'':8}";
const DOWNLOAD_TEMPLATE: &str = "{spinner:.green} {prefix:<11} {msg:40!}  [{bar:30.cyan/blue}]  {bytes:>10}/{total_bytes:<10}  {bytes_per_sec:>12}  {eta:>8}";
const UNKNOWN_LENGTH_TEMPLATE: &str = "{spinner:.green} {prefix:<11} {msg:40!}  {'':32}  {bytes:>10}/{'':10}  {bytes_per_sec:>12}  {'':8}";

impl Downloader {
    const MAX_CONCURRENT_DOWNLOADS: usize = 4;

    pub fn new(show_progress: bool) -> Self {
        let agent = ureq::config::Config::builder()
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .build()
            .new_agent();

        Self {
            agent,
            show_progress,
        }
    }

    pub fn download(&self, url: &str, destination_path: &Path) -> ToolToolResult<()> {
        self.download_files(&[(url.to_string(), destination_path.to_path_buf())])?
            .into_iter()
            .next()
            .expect("single download result")
    }

    pub fn download_files(
        &self,
        requests: &[(String, PathBuf)],
    ) -> ToolToolResult<Vec<ToolToolResult<()>>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let multi_progress = MultiProgress::new();
        let progress_bars = requests
            .iter()
            .map(|(url, _)| {
                if self.show_progress {
                    Ok(Some(create_queued_progress(&multi_progress, url)?))
                } else {
                    Ok(None)
                }
            })
            .collect::<ToolToolResult<Vec<_>>>()?;
        let next_request = AtomicUsize::new(0);
        let results = Mutex::new(
            (0..requests.len())
                .map(|_| None)
                .collect::<Vec<Option<ToolToolResult<()>>>>(),
        );
        let worker_count = requests.len().min(Self::MAX_CONCURRENT_DOWNLOADS);

        std::thread::scope(|scope| {
            for _ in 0..worker_count {
                scope.spawn(|| {
                    loop {
                        let index = next_request.fetch_add(1, Ordering::Relaxed);
                        let Some((url, destination_path)) = requests.get(index) else {
                            break;
                        };
                        let result = download_one(
                            &self.agent,
                            url,
                            destination_path,
                            progress_bars[index].as_ref(),
                        );
                        results.lock().unwrap()[index] = Some(result);
                    }
                });
            }
        });

        if self.show_progress {
            multi_progress.clear()?;
        }
        Ok(results
            .into_inner()
            .unwrap()
            .into_iter()
            .map(|result| result.expect("download worker did not produce a result"))
            .collect())
    }
}

fn download_one(
    agent: &ureq::Agent,
    url: &str,
    destination_path: &Path,
    progress: Option<&ProgressBar>,
) -> ToolToolResult<()> {
    let result = (|| -> ToolToolResult<()> {
        if let Some(progress) = progress {
            activate_progress(progress, None)?;
        }
        let response = agent.get(url).call()?;
        let content_length = response.body().content_length();
        let mut reader = response.into_body().into_reader();
        let mut output_file = std::fs::File::create(destination_path)?;
        if let Some(progress) = progress {
            activate_progress(progress, content_length)?;
            copy_with_progress(progress, &mut reader, &mut output_file)?;
        } else {
            std::io::copy(&mut reader, &mut output_file)?;
        }
        Ok(())
    })()
    .with_context(|| format!("Failed to download '{url}' to '{destination_path:?}'"));

    if let Some(progress) = progress {
        if result.is_ok() {
            progress.set_prefix("done");
            progress.finish();
        } else {
            progress.set_prefix("failed");
            progress.abandon();
        }
    }
    result
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new(false)
    }
}

fn create_queued_progress(
    multi_progress: &MultiProgress,
    url: &str,
) -> ToolToolResult<ProgressBar> {
    let progress = multi_progress.add(ProgressBar::new_spinner());
    progress.set_style(ProgressStyle::with_template(QUEUED_TEMPLATE)?);
    progress.set_prefix("queued");
    progress.set_message(download_name(url));
    Ok(progress)
}

fn activate_progress(progress: &ProgressBar, content_length: Option<u64>) -> ToolToolResult<()> {
    if let Some(length) = content_length {
        progress.set_length(length);
        progress.set_style(ProgressStyle::with_template(DOWNLOAD_TEMPLATE)?);
    } else {
        progress.set_style(ProgressStyle::with_template(UNKNOWN_LENGTH_TEMPLATE)?);
    }
    progress.set_prefix("downloading");
    progress.enable_steady_tick(std::time::Duration::from_millis(100));
    Ok(())
}

fn copy_with_progress(
    progress: &ProgressBar,
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> ToolToolResult<()> {
    let mut buffer = [0; 64 * 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
        progress.inc(bytes_read as u64);
    }
    Ok(())
}

fn download_name(url: &str) -> String {
    url.split(['?', '#'])
        .next()
        .and_then(|url| url.rsplit('/').next())
        .filter(|name| !name.is_empty())
        .unwrap_or("download")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::assert_starts_with;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};
    use test_temp_dir::{TestTempDir, test_temp_dir};

    struct TestContext {
        temp_dir: TestTempDir,
        server: MockServer,
        content: String,
        downloader: Downloader,
    }

    fn setup() -> TestContext {
        let temp_dir = test_temp_dir!();
        let server = MockServer::start();
        let content = "download content";
        server.mock(|when, then| {
            when.method(GET).path("/download_url");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body(content);
        });
        TestContext {
            temp_dir,
            server,
            content: content.to_string(),
            downloader: Downloader::new(false),
        }
    }

    #[test]
    fn test_download() {
        let ctx = setup();
        let local_path = ctx.temp_dir.used_by(|path| path.join("file_download"));
        ctx.downloader
            .download(&ctx.server.url("/download_url"), &local_path.as_path())
            .unwrap();
        let actual_content = std::fs::read_to_string(local_path.as_path()).unwrap();
        assert_eq!(actual_content, ctx.content);
    }

    #[test]
    fn test_download_with_progress() {
        let ctx = setup();
        let local_path = ctx
            .temp_dir
            .used_by(|path| path.join("file_download_with_progress"));
        Downloader::new(true)
            .download(&ctx.server.url("/download_url"), local_path.as_path())
            .unwrap();

        let actual_content = std::fs::read_to_string(local_path.as_path()).unwrap();
        assert_eq!(actual_content, ctx.content);
    }

    #[test]
    fn test_download_files_in_parallel() {
        let temp_dir = test_temp_dir!();
        let server = MockServer::start();
        let download = server.mock(|when, then| {
            when.method(GET).path("/slow-download");
            then.status(200)
                .delay(Duration::from_millis(300))
                .body("download content");
        });
        let requests = (0..6)
            .map(|index| {
                (
                    server.url("/slow-download"),
                    temp_dir
                        .used_by(|path| path.join(format!("download-{index}")))
                        .to_path_buf(),
                )
            })
            .collect::<Vec<_>>();

        let started = Instant::now();
        let results = Downloader::new(false).download_files(&requests).unwrap();

        assert!(
            started.elapsed() < Duration::from_millis(1_300),
            "downloads did not overlap: {:?}",
            started.elapsed()
        );
        download.assert_hits(6);
        assert!(results.into_iter().all(|result| result.is_ok()));
        for (_, path) in requests {
            assert_eq!(std::fs::read_to_string(path).unwrap(), "download content");
        }
    }

    #[test]
    fn test_download_name() {
        assert_eq!(
            download_name("https://example.com/releases/tool.tar.gz?token=secret"),
            "tool.tar.gz"
        );
        assert_eq!(download_name("https://example.com/"), "download");
    }

    #[test]
    fn progress_table_templates_are_valid() {
        for template in [QUEUED_TEMPLATE, DOWNLOAD_TEMPLATE, UNKNOWN_LENGTH_TEMPLATE] {
            ProgressStyle::with_template(template).unwrap();
        }
    }

    #[test]
    fn test_404_not_found() {
        let ctx = setup();

        ctx.server.mock(|when, then| {
            when.method(GET).path("/download_url_404");
            then.status(404);
        });

        let local_path = ctx.temp_dir.used_by(|path| path.join("file_download"));
        let url = ctx.server.url("/download_url_404");
        let error = ctx
            .downloader
            .download(&url, &local_path.as_path())
            .expect_err("Expected error");
        assert_starts_with!(error.to_string(), "Failed to download 'http");
    }

    #[test]
    fn test_invalid_path() {
        let ctx = setup();

        let url = ctx.server.url("/download");
        let error = ctx
            .downloader
            .download(&url, &PathBuf::from("invalid_path"))
            .expect_err("Expected error");
        assert_starts_with!(error.to_string(), "Failed to download 'http");
    }
}
