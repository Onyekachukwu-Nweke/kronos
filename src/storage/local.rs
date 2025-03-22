use crate::error::{Error, Result};
use crate::utils::compression::compress_directory;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

pub struct LocalStorage {
    base_path: String,
}

impl LocalStorage {
    pub fn new(base_path: &str) -> Self {
        LocalStorage {
            base_path: base_path.to_string(),
        }
    }

    pub async fn store(&self, source_dir: &Path, backup_id: &str) -> Result<()> {
        let backup_filename = format!("{}.tar.gz", backup_id);
        let temp_output = PathBuf::from(&backup_filename);
        compress_directory(source_dir, &temp_output)?;

        let final_path = PathBuf::from(&self.base_path).join(&backup_filename);
        fs::create_dir_all(&self.base_path).map_err(Error::Io)?;
        async_fs::rename(&temp_output, &final_path)
            .await
            .map_err(Error::Io)?;

        Ok(())
    }
}