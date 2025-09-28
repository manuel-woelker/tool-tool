use crate::configuration::platform::DownloadPlatform;
use crate::types::{Env, FilePath};
use std::fmt::Debug;
use std::io::{Read, Seek, Write};
use std::rc::Rc;
use std::time::Duration;
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
    fn execute(&self, request: ExecutionRequest) -> ToolToolResult<i32>;

    /**
    Create a random, unique string
    */
    fn random_string(&self) -> ToolToolResult<String>;

    /**
    Get a timestamp for measuring execution time
    Note that this is a _duration_ measuring the time elapsed since some arbitrary point in the past
    This is because there's no good way to create an _Instant_ in a platform-agnostic way
    */
    fn now(&self) -> ToolToolResult<Duration>;

    /**
    Try to acquire an exclusive lock on the lockfile
    */
    fn try_lock(&self) -> ToolToolResult<bool>;

    /**
    Release the lock on the lockfile
    */
    fn unlock(&self) -> ToolToolResult<()>;

    /**
    Sleep for the given duration
    */
    fn sleep(&self, duration: Duration);
}

pub type AdapterBox = Rc<dyn Adapter>;

#[derive(Debug)]
pub struct ExecutionRequest {
    pub binary_path: FilePath,
    pub args: Vec<String>,
    pub env: Env,
}
