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
        if let Some(sqlite_configs) = &self.config.databases.sqlite {
            for cfg in sqlite_configs {
                let db = SQLiteDatabase::new(cfg);
                db.backup(self.backup_path).await?;
            }
        }

        // Placeholder for other database types (MySQL, Postgres, MongoDB)
        Ok(())
    }
}