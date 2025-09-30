//! Common types used througout the application
// SPDX-License-Identifier: GPL-3.0-only

use crate::local;

use gtk::glib;
use std::future::Future;

/// Errors that are returned for things that can go wrong with **local** IO
///
/// TODO: Implement fmt to be somewhat helpful error messages to be displayed
#[non_exhaustive]
#[derive(Debug)]
pub enum LocalError {
    /// To be returned by functions where datafusion returns an error
    DbError,
    /// To be returned if a path that is expected to exist does not
    NotFound,
    /// To be returned when trying to create something that already exists
    AlreadyExists,
    /// To be returned when the keyring returns an error
    KeyringError,
    /// To be returned when loading the config fails
    ConfigError,
}

/// Stores all data that is needed at runtime
#[derive(Debug, Clone)]
pub struct State {
    conn: local::db::DbConnector,
    username: String,
    password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
/// The config for the application. These will be saved in the config file on disk.
///
/// ## Difference to [State]
/// Config holds data that is unlikely to change and is not so sensitive that it
/// should instead be put into the keyring or does not make sense to save into the
/// database.
///
/// See [State] for more information.
pub struct Config {
    /// Whether the user is just writing notes (true) or checking and signing
    /// notes (false).
    pub is_student: bool,
}

/// The state for the application.
///
/// ## Difference to [common::Config]
/// State holds data that is obtained at runtime and only valid during runtime,
/// while Config holds data that you would usually save in a config file (and should
/// be exposed to a user).
///
/// See [Config] for more information.
impl State {
    pub fn new(conn: local::db::DbConnector, username: String, password: String) -> State {
        State {
            conn,
            username,
            password,
        }
    }
}

impl Config {
    pub fn from_file(path: Option<&std::path::Path>) -> Result<Self, LocalError> {
        let Ok(raw_conf) =
            std::fs::read_to_string(path.unwrap_or(&local::paths::get_config_path()))
        else {
            return Err(LocalError::ConfigError);
        };
        let Ok(config) = toml::from_str(&raw_conf) else {
            return Err(LocalError::ConfigError);
        };
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config { is_student: true }
    }
}

pub struct WeeklyReport {
    signed: bool,
    timestamp: u64,
    days: Vec<Day>,
}

pub struct Day {
    activities: Vec<String>,
}

// Our own little async runtime, built on glib.
// ~~stolen~~ borrowed from
// https://mmstick.github.io/gtkrs-tutorials/1x03-glib-runtime.html

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| glib::MainContext::new())
}

pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    thread_context().block_on(future)
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    thread_context().spawn_local(future);
}
