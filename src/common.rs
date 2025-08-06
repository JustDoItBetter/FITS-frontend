// Common types used througout the application
// SPDX-License-Identifier: GPL-3.0-only

/// Errors that are returned for things that can go wrong with **local** IO
///
/// TODO: Implement fmt to be somewhat helpful error messages to be displayed
#[non_exhaustive]
#[derive(Debug)]
pub enum LocalError {
    /// To be returned by functions where rusqlite returns an error
    SqliteError,
    /// To be returned if a path that is expected to exist does not
    NotYetFound,
    /// To be returned when trying to create something that already exists
    AlreadyExists,
    /// To be returned when the keyring returns an error
    KeyringError,
}

/// Stores all data that is needed at runtime
pub struct Config {
    conn: rusqlite::Connection,
    username: String,
    password: String,
}

impl Config {
    pub fn new(conn: rusqlite::Connection, username: String, password: String) -> Config {
        Config {
            conn,
            username,
            password,
        }
    }
}
