use crate::test_util::archive_builder::ArchiveBuilder;
use std::io::{Cursor, Write};
use tool_tool_base::result::ToolToolResult;
use zip::write::SimpleFileOptions;
use zip::{DateTime, ZipArchive, ZipWriter};

pub struct ZipBuilder {
    zip_writer: ZipWriter<Cursor<Vec<u8>>>,
}

impl Default for ZipBuilder {
    fn default() -> Self {
        let zip_writer = ZipWriter::new(Cursor::new(Vec::new()));
        Self { zip_writer }
    }
}

fn create_file_options() -> SimpleFileOptions {
    SimpleFileOptions::default().last_modified_time(DateTime::default())
}

impl ArchiveBuilder for ZipBuilder {
    fn add_file(&mut self, path: impl AsRef<str>, content: impl AsRef<[u8]>) -> ToolToolResult<()> {
        self.zip_writer
            .start_file(path.as_ref().to_string(), create_file_options())?;
        self.zip_writer.write_all(content.as_ref())?;
        Ok(())
    }

    fn add_directory(&mut self, path: impl AsRef<str>) -> ToolToolResult<()> {
        self.zip_writer
            .add_directory(path.as_ref().to_string(), create_file_options())?;
        Ok(())
    }

    fn build(self) -> ToolToolResult<Vec<u8>> {
        let cursor = self.zip_writer.finish()?;
        Ok(cursor.into_inner())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use expect_test::expect;
    use std::fmt::Write;
    use std::io::Read;

    #[test]
    fn test_zip_builder_empty() -> ToolToolResult<()> {
        let file = ZipBuilder::default().build()?;
        let zip_archive = ZipArchive::new(Cursor::new(file))?;
        assert!(zip_archive.is_empty());
        Ok(())
    }

    #[test]
    fn test_zip_builder_with_files() -> ToolToolResult<()> {
        let mut zip_builder = ZipBuilder::default();
        zip_builder.add_file("foo", b"bar")?;
        zip_builder.add_directory("folder/2/3")?;
        zip_builder.add_file("fizz/buzz.txt", b"foobar")?;
        let file = zip_builder.build()?;
        let mut zip_archive = ZipArchive::new(Cursor::new(file))?;
        let mut content = String::new();
        for i in 0..zip_archive.len() {
            let mut zip_entry = zip_archive.by_index(i)?;
            if zip_entry.is_file() {
                let mut entry_content = String::new();
                zip_entry.read_to_string(&mut entry_content)?;
                writeln!(content, "{}: '{entry_content}'", zip_entry.name())?;
            } else if zip_entry.is_dir() {
                writeln!(content, "{} (DIR)", zip_entry.name())?;
            }
        }
        expect![[r#"
            foo: 'bar'
            folder/2/3/ (DIR)
            fizz/buzz.txt: 'foobar'
        "#]]
        .assert_eq(&content);
        Ok(())
    }
}
