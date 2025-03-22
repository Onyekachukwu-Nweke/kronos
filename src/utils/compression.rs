use crate::error::{Error, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;

pub fn compress_directory(source_dir: &Path, output_path: &Path) -> Result<()> {
    let tar_gz = File::create(output_path).map_err(Error::Io)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", source_dir)
        .map_err(|e| Error::Backup(format!("Failed to create tar archive: {}", e)))?;
    tar.finish()
        .map_err(|e| Error::Backup(format!("Failed to finish tar archive: {}", e)))?;

    Ok(())
}