use crate::configuration::platform::DownloadPlatform;
use crate::types::FilePath;
use std::fmt::Debug;
use std::io::{Read, Seek, Write};
use std::rc::Rc;
use tool_tool_base::result::ToolToolResult;

pub trait ReadSeek: Read + Seek + 'static {}

impl<T: Read + Seek + 'static> ReadSeek for T {}

pub trait Adapter: Debug + 'static {
    /**
       Get the command line arguments, the first one is the path to the binary
    */
    fn args(&self) -> Vec<String>;

    /**
    Get the program environment
    */
    fn env(&self) -> Vec<(String, String)>;

    /**
        Print a message to stderr
    */
    fn print(&self, message: &str);

    /**
    Check if a file exists
    */
    fn file_exists(&self, path: &FilePath) -> ToolToolResult<bool>;

    /*
       Read a file, the path is relative to parent directory of the tool-tool binary
    */
    fn read_file(&self, path: &FilePath) -> ToolToolResult<Box<dyn ReadSeek>>;

    /*
       Create a file to a string, the path is relative to parent directory of the tool-tool binary
    */
    fn create_file(&self, path: &FilePath) -> ToolToolResult<Box<dyn Write>>;

    /**
        Create a directory (including parent directories if they don't exist)
        the path is relative to parent directory of the tool-tool binary
    */
    fn create_directory_all(&self, path: &FilePath) -> ToolToolResult<()>;

    /**
    Delete a directory (including all contained files and directories)
    the path is relative to parent directory of the tool-tool binary
    */
    fn delete_directory_all(&self, path: &FilePath) -> ToolToolResult<()>;

    /**
        Exit the process with the given exit code
    */
    fn exit(&self, exit_code: i32);

    /**
        Download a file from a url
    */
    fn download_file(&self, url: &str, destination_path: &FilePath) -> ToolToolResult<()>;

    /**
        Get the currently running platform
    */
    fn get_platform(&self) -> DownloadPlatform;

    /**
    Execute the given binary with the given arguments
    */
    fn execute(&self, request: ExecutionRequest) -> ToolToolResult<()>;
}

pub type AdapterBox = Rc<dyn Adapter>;

pub struct ExecutionRequest {
    pub binary_path: FilePath,
}
