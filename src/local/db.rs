//! Locally store data with DataFusion
// SPDX-License-Identifier: GPL-3.0-only

pub use connector::DbConnector;

use crate::{common, local};

use std::sync::{Arc, mpsc};

mod connector;

pub async fn open() -> Result<DbConnector, common::LocalError> {
    let path = local::paths::get_db_path();
    let path_lit = path.to_str().unwrap();
    DbConnector::open(path_lit).await
}

pub struct DbRequest {
    pub command: DbCommand,
    pub receiver: mpsc::Sender<DbAnswer>,
}

pub enum DbCommand {
    /// Create a backup of the database.
    ///
    /// The range specifies the time for which to export in unix timestamp.
    ///
    /// The sender will recieve [DbAnswer::Backup] after requesting this.
    Backup(std::ops::Range<u64>),
    /// Write a weekly report to the database.
    ///
    /// Returns either [DbAnswer::Ok] on success or [DbAnswer::Err] on failure
    SaveData { data: Vec<common::WeeklyReport> },
}

pub enum DbAnswer {
    /// Confirmation that an operation succeeded.
    Ok,
    /// Alert that the requested operation did not succeed.
    Err,
    /// The response to a CreateBackup command.
    Backup(Vec<datafusion::arrow::record_batch::RecordBatch>),
}

/// Creates the local sqlite db with the schemas.
pub fn create_db() -> Result<(), common::LocalError> {
    log::debug!("Trying to create database");
    let path = local::paths::get_db_path();
    if path.exists() {
        log::info!("DB was already there, not overwriting it");
        return Err(common::LocalError::AlreadyExists);
    }

    let Ok(file_stream) = std::fs::File::create(&path) else {
        log::error!("Failed to create db at {:#?}", &path);
        return Err(common::LocalError::NotFound);
    };
    match datafusion::parquet::arrow::ArrowWriter::try_new(
        file_stream,
        Arc::new(connector::get_report_schema()),
        None,
    ) {
        Ok(_) => {
            log::debug!("Successfully loaded db at {:#?}", &path);
            Ok(())
        }
        Err(e) => {
            log::error!("Could not create db at {:#?}", &path);
            log::error!("Parquet produced the following error: {:#?}", e);
            Err(common::LocalError::DbError)
        }
    }
}
