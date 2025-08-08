// Locally store data with sqlite
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;
use rusqlite::Connection;

pub fn connect() -> Result<Connection, common::LocalError> {
    let path = super::paths::get_sqlite_path();
    if !path.exists() {
        log::info!("SQL db was not yet there");
        return Err(common::LocalError::NotYetFound);
    }
    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(e) => {
            log::warn!("Sqlite failed to open db: {e}");
            Err(common::LocalError::SqliteError)
        }
    }
}

/// Creates the local sqlite db with the schemas.
pub fn create_db() -> Result<Connection, common::LocalError> {
    let path = super::paths::get_sqlite_path();
    if path.exists() {
        log::info!("DB was already there, not overwriting it");
        return Err(common::LocalError::AlreadyExists);
    }
    // TODO: Actually decide on schemas based on things that will not happen for a
    // few days
    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(e) => {
            log::warn!("Sqlite failed to open: {e}");
            Err(common::LocalError::SqliteError)
        }
    }
}
