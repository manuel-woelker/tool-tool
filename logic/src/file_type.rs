#[derive(Debug, PartialEq)]
pub enum FileType {
    Zip,
    TarGz,
    Other,
}

fn get_filename_from_url(url: &str) -> Option<&str> {
    // Remove any query string or fragment
    let url = url.split('?').next().unwrap_or(url);
    let url = url.split('#').next().unwrap_or(url);

    // Find the last segment after the last slash
    url.rsplit('/').next().filter(|s| !s.is_empty())
}

pub fn get_file_type_from_url(url: &str) -> FileType {
    let filename = get_filename_from_url(url);
    filename
        .map(|filename| {
            if filename.ends_with(".zip") {
                FileType::Zip
            } else if filename.ends_with(".tar.gz") {
                FileType::TarGz
            } else {
                FileType::Other
            }
        })
        .unwrap_or(FileType::Other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_type_from_url() {
        assert_eq!(
            get_file_type_from_url("https://example.com/file.txt"),
            FileType::Other
        );
        assert_eq!(
            get_file_type_from_url("https://example.com/file.tar.gz"),
            FileType::TarGz
        );
        assert_eq!(
            get_file_type_from_url("https://example.com/file.zip"),
            FileType::Zip
        );
        assert_eq!(
            get_file_type_from_url("https://example.com/file.tar.bzip2"),
            FileType::Other
        );

        assert_eq!(
            get_file_type_from_url("https://example.com/file.txt?foo=bar/x.zip"),
            FileType::Other
        );
        assert_eq!(
            get_file_type_from_url("https://example.com/file.tar.gz?foo=bar/x.zip"),
            FileType::TarGz
        );
        assert_eq!(
            get_file_type_from_url("https://example.com/file.zip?foo=bar/x.tar.gz"),
            FileType::Zip
        );
        assert_eq!(
            get_file_type_from_url("https://example.com/file.tar.bzip2?foo=bar/x.zip"),
            FileType::Other
        );
    }
}
