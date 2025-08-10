use std::fmt::Debug;

pub trait Adapter: Debug + 'static {
    fn get_args(&self) -> Vec<String>;
    fn print(&self, message: &str);
    fn exit(&self, exit_code: i32);
}
