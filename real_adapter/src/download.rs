use tool_tool_base::result::{Context, ToolToolResult};
use ureq::tls::{RootCerts, TlsConfig};

pub struct Downloader {
    agent: ureq::Agent,
}

impl Downloader {
    pub fn new() -> Self {
        let agent = ureq::config::Config::builder()
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .build()
            .new_agent();

        Self { agent }
    }

    pub fn download(&self, url: &str, destination_path: &std::path::Path) -> ToolToolResult<()> {
        (|| -> ToolToolResult<()> {
            let response = self.agent.get(url).call()?;
            let mut reader = response.into_body().into_reader();
            let mut output_file = std::fs::File::create(destination_path)?;
            std::io::copy(&mut reader, &mut output_file)?;
            Ok(())
        })()
        .wrap_err_with(|| format!("Failed to download '{url}' to '{destination_path:?}'"))
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertables::assert_starts_with;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use std::path::PathBuf;
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
            downloader: Downloader::new(),
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
