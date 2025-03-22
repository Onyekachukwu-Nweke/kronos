use crate::config::Config;
use crate::database::sqlite::SQLiteDatabase;
use crate::error::{Error, Result};
use std::path::Path;

pub struct BackupPerformer<'a> {
    config: &'a Config,
    backup_path: &'a Path,
}

impl<'a> BackupPerformer<'a> {
    pub fn new(config: &'a Config, backup_path: &'a Path) -> Self {
        BackupPerformer { config, backup_path }
    }

    pub async fn execute(&mut self) -> Result<()> {
        // Handle SQLite databases
        if let Some(sqlite_config) = &self.config.databases.sqlite {
            let db = SQLiteDatabase::new(sqlite_config);
            db.backup(self.backup_path).await?;
        }

        // Placeholder for other database types (MySQL, Postgres, MongoDB)
        Ok(())
    }
}