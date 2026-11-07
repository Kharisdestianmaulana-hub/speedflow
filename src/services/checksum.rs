use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct ChecksumVerifier;

impl ChecksumVerifier {
    pub async fn compute_sha256(path: impl AsRef<Path>) -> anyhow::Result<String> {
        let mut file = File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    pub async fn verify_sha256(path: impl AsRef<Path>, expected_hash: &str) -> anyhow::Result<bool> {
        let computed = Self::compute_sha256(path).await?;
        Ok(computed.eq_ignore_ascii_case(expected_hash.trim()))
    }
}
