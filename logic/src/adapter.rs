use crate::types::FilePath;
use std::fmt::Debug;
use tool_tool_base::result::ToolToolResult;

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

    /*
       Read a file to a string, the path is relative to parent directory of the tool-tool binary
    */
    fn read_file(&self, path: &FilePath) -> ToolToolResult<String>;

    /**
        Exit the process with the given exit code
    */
    fn exit(&self, exit_code: i32);
}
