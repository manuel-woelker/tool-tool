pub type FilePath = relative_path::RelativePathBuf;

#[derive(Debug, Clone)]
pub struct EnvPair {
    pub key: String,
    pub value: String,
}

impl EnvPair {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

pub type Env = Vec<EnvPair>;
