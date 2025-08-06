// Locally store data with sqlite
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;
use rusqlite::Connection;

pub fn connect() -> Result<Connection, common::LocalError> {
    let path = super::paths::get_sqlite_path();
    if !path.exists() {
        return Err(common::LocalError::NotYetFound);
    }
    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(_) => Err(common::LocalError::SqliteError),
    }
}

/// Creates the local sqlite db with the schemas.
pub fn create_db() -> Result<Connection, common::LocalError> {
    let path = super::paths::get_sqlite_path();
    if path.exists() {
        return Err(common::LocalError::AlreadyExists);
    }
    // TODO: Actually decide on schemas based on things that will not happen for a
    // few days
    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(_) => Err(common::LocalError::SqliteError),
    }
}
