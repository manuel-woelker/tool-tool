use tool_tool_base::result::ToolToolResult;
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
        let response = self.agent.get(url).call()?;
        let mut reader = response.into_body().into_reader();
        let mut output_file = std::fs::File::create(destination_path)?;
        std::io::copy(&mut reader, &mut output_file)?;
        Ok(())
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
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use test_temp_dir::test_temp_dir;

    #[test]
    fn test_download() {
        let temp_dir = test_temp_dir!();
        // Start a lightweight mock server.
        let server = MockServer::start();

        let content = "download content";
        // Create a mock on the server.
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/download_url");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body(content);
        });
        let downloader = Downloader::new();
        let local_path = temp_dir.used_by(|path| path.join("file_download"));
        downloader
            .download(&server.url("/download_url"), &local_path.as_path())
            .unwrap();
        let actual_content = std::fs::read_to_string(local_path.as_path()).unwrap();
        assert_eq!(actual_content, content);
    }
}
