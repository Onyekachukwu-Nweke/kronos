use crate::backup::performer::BackupPerformer;
use crate::config::Config;
use crate::error::Result;
use crate::storage::local::LocalStorage;
use log::info;

pub async fn run_backup(config: &Config) -> Result<()> {
    info!("Starting backup process");

    // Generate a unique backup ID using timestamp
    let backup_id = chrono::Utc::now().format("backup-%Y%m%dT%H%M%S").to_string();
    let temp_dir = tempfile::tempdir().map_err(|e| crate::error::Error::Io(e))?;
    let backup_path = temp_dir.path();

    // Perform backup
    let mut performer = BackupPerformer::new(config, backup_path);
    performer.execute().await?;

    // Compress and store
    let local_storage = LocalStorage::new(config.storage.path.as_ref().unwrap_or(&String::from("/backups")));
    local_storage.store(backup_path, &backup_id).await?;

    info!("Backup completed successfully: {}", backup_id);
    Ok(())
}