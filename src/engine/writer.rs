use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

pub struct SegmentFileWriter {
    path: PathBuf,
}

impl SegmentFileWriter {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Pre-allocates the file to the total size if known.
    pub async fn preallocate(&self, total_bytes: u64) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.path)
            .await?;

        file.set_len(total_bytes).await?;
        Ok(())
    }

    /// Writes a chunk of data at a specific byte offset.
    pub async fn write_at(&self, offset: u64, data: &[u8]) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .await?;

        file.seek(SeekFrom::Start(offset)).await?;
        file.write_all(data).await?;
        file.flush().await?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
