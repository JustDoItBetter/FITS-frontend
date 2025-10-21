//! Locally store data with DataFusion
// SPDX-License-Identifier: GPL-3.0-only

pub use connector::DbConnector;

use crate::{common, local};

use diesel::prelude::*;
use std::sync::mpsc;

mod connector;
mod queries;
pub mod schema;

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
    /// Create a read with data from the database.
    ///
    /// The range specifies the time for which to export in unix timestamp.
    ///
    /// The sender will receive [DbAnswer::Backup] or [DbAnswer::Err] after
    /// requesting this.
    ///
    /// If there is no data in the specified time, Backup will return an empty vec
    /// but no error.
    Read(std::ops::Range<i64>),
    /// Write a weekly report to the database.
    ///
    /// Returns either [DbAnswer::Ok] on success or [DbAnswer::Err] on failure
    Save { data: Vec<common::WeeklyReport> },
    /// Create a backup and return it as bytes
    ///
    /// Returns either [DbAnswer::Backup] on success or [DbAnswer::Err] on failure
    Backup,
}

pub enum DbAnswer {
    /// Confirmation that an operation succeeded.
    Ok,
    /// Alert that the requested operation did not succeed.
    Err,
    /// The response to a Read.
    Read(Vec<common::WeeklyReport>),
}

/// Creates the local sqlite db with the schemas.
pub fn create_db() -> Result<(), common::LocalError> {
    log::debug!("Trying to create database");
    let path = local::paths::get_db_path();
    if path.exists() {
        log::info!("DB was already there, not overwriting it");
        return Err(common::LocalError::AlreadyExists);
    }

    let Some(path) = path.to_str() else {
        log::error!("DB path is not valid UTF-8");
        return Err(common::LocalError::DbError);
    };

    match SqliteConnection::establish(path) {
        Ok(_) => {
            log::debug!("Successfully loaded db at {:#?}", &path);
            Ok(())
        }
        Err(e) => {
            log::error!("Could not create db at {:#?}", &path);
            log::error!("SQL produced the following error: {:#?}", e);
            Err(common::LocalError::DbError)
        }
    }
}
