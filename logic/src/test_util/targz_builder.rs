use crate::test_util::archive_builder::ArchiveBuilder;
use flate2::write::GzEncoder;
use std::io::Cursor;
use tar::Header;
use tool_tool_base::result::ToolToolResult;

pub struct TarGzBuilder {
    tar_builder: tar::Builder<GzEncoder<Cursor<Vec<u8>>>>,
}

impl Default for TarGzBuilder {
    fn default() -> Self {
        let tar_builder = tar::Builder::new(GzEncoder::new(
            Cursor::new(Vec::new()),
            flate2::Compression::default(),
        ));
        Self { tar_builder }
    }
}

impl ArchiveBuilder for TarGzBuilder {
    fn add_file(&mut self, path: impl AsRef<str>, content: impl AsRef<[u8]>) -> ToolToolResult<()> {
        let mut header = Header::new_gnu();
        header.set_size(content.as_ref().len() as u64);
        self.tar_builder
            .append_data(&mut header, path.as_ref(), content.as_ref())?;
        Ok(())
    }

    fn add_directory(&mut self, path: impl AsRef<str>) -> ToolToolResult<()> {
        let mut header = Header::new_gnu();
        header.set_path(path.as_ref())?;
        header.set_entry_type(tar::EntryType::Directory);
        header.set_size(0);
        header.set_cksum();
        self.tar_builder.append(&mut header, std::io::empty())?;
        Ok(())
    }

    fn build(mut self) -> ToolToolResult<Vec<u8>> {
        self.tar_builder.finish()?;
        Ok(self.tar_builder.into_inner()?.finish()?.into_inner())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use expect_test::expect;
    use flate2::read::GzDecoder;
    use std::fmt::Write;
    use std::io::Read;
    use tar::Archive;

    #[test]
    fn test_targz_builder_empty() -> ToolToolResult<()> {
        let file = TarGzBuilder::default().build()?;
        let tar = GzDecoder::new(Cursor::new(file));
        let mut archive = Archive::new(tar);
        assert!(archive.entries()?.next().is_none());
        Ok(())
    }

    #[test]
    fn test_targz_builder_with_files() -> ToolToolResult<()> {
        let mut targz_builder = TarGzBuilder::default();
        targz_builder.add_file("foo", b"bar")?;
        targz_builder.add_directory("folder/2/3")?;
        targz_builder.add_file("fizz/buzz.txt", b"foobar")?;
        let file = targz_builder.build()?;
        let tar = GzDecoder::new(Cursor::new(file));
        let mut archive = Archive::new(tar);
        let mut content = String::new();
        for archive_entry in archive.entries()? {
            let mut archive_entry = archive_entry?;
            match archive_entry.header().entry_type() {
                tar::EntryType::Regular => {
                    let mut entry_content = String::new();
                    archive_entry.read_to_string(&mut entry_content)?;
                    writeln!(content, "{:?}: '{entry_content}'", archive_entry.path()?)?;
                }
                tar::EntryType::Directory => {
                    writeln!(content, "{:?} (DIR)", archive_entry.path()?)?;
                }
                _ => {
                    panic!(
                        "Unsupported entry type: {:?}",
                        archive_entry.header().entry_type()
                    );
                }
            }
        }
        expect![[r#"
            "foo": 'bar'
            "folder/2/3" (DIR)
            "fizz/buzz.txt": 'foobar'
        "#]]
        .assert_eq(&content);
        Ok(())
    }
}
