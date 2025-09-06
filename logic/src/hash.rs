use sha2::{Digest, Sha512};
use std::io::Read;
use tool_tool_base::result::ToolToolResult;

/// Computes the SHA-512 digest of any type that implements `Read`.
pub fn compute_sha512<R: Read>(mut read: R) -> ToolToolResult<String> {
    let mut hasher = Sha512::new();
    let mut buffer = [0u8; 8192]; // 8 KiB buffer

    loop {
        let n = read.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{result:x}")) // return as lowercase hex string
}

#[cfg(test)]
mod test {
    use crate::hash::compute_sha512;
    use std::io::Cursor;

    #[test]
    fn test_compute_sha512() {
        let data = b"test data";
        let result = compute_sha512(Cursor::new(data)).unwrap();
        assert_eq!(
            "0e1e21ecf105ec853d24d728867ad70613c21663a4693074b2a3619c1bd39d66b588c33723bb466c72424e80e3ca63c249078ab347bab9428500e7ee43059d0d",
            &result
        );
    }
}
