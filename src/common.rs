//! Common types used throughout the application
// SPDX-License-Identifier: GPL-3.0-only

use crate::{gui, local};
use chrono::Datelike;
use std::collections::HashMap;

use gtk::glib;
use std::future::Future;

/// Errors that are returned for things that can go wrong with **local** IO
///
/// TODO: Implement fmt to be somewhat helpful error messages to be displayed
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum LocalError {
    /// To be returned by functions where rusqlite returns an error
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

    pub fn get_db_connector(&self) -> local::db::DbConnector {
        self.conn.clone()
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

/// The common format of a report to be passed through the application.
///
/// # Setters
/// Be aware that every setter for a private property will trigger the timestamp to
/// be recreated and will therefore reset the signature status on update.
///
/// ## For other devs working in FITS
/// Unless necessary, prefer to use this struct to relay information about a report.
/// If you must use a different format, keep it in the specific module, like and
/// parse it into this when talking to other modules.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WeeklyReport {
    signed: bool,
    /// Specifies the year this applies to
    year: i32,
    /// Specifies the week number this applies to
    week: u32,
    /// Specifies when this report was last written to (mostly relevant for backups)
    last_update: chrono::NaiveDateTime,
    days: HashMap<String, Vec<String>>,
}

impl WeeklyReport {
    /// Create a new WeeklyReport.
    ///
    /// If no timestamp is provided, the current timestamp will be used.
    ///
    /// Note that this function automatically sets the last_updated property for
    /// you. If you already have all the data for the report and are just parsing
    /// into WeeklyReport, you probably want to use [WeeklyReport::from_raw_parts()]
    /// instead.
    pub fn new(
        signed: bool,
        timestamp: Option<chrono::NaiveDateTime>,
        days: Option<HashMap<String, Vec<String>>>,
    ) -> Self {
        let timestamp = timestamp.unwrap_or_else(|| chrono::Utc::now().naive_utc());
        let iso_week = timestamp.iso_week();

        WeeklyReport {
            signed,
            year: iso_week.year(),
            week: iso_week.week(),
            last_update: chrono::Utc::now().naive_utc(),
            days: days.unwrap_or_default(),
        }
    }

    /// Create a new WeeklyReport completely from already existing data.
    ///
    /// If you want to create a new [WeeklyReport], use [WeeklyReport::new()]
    /// instead.
    ///
    /// This function is only really useful for parsing data from another format
    /// into a WeeklyReport.
    ///
    /// # Unsafety
    /// This function creates a WeeklyReport by naively trusting the inputs to be
    /// correct. ONLY use it for parsing from another source, such as the db.
    ///
    /// For creating a new WeeklyReport, use [WeeklyReport::new] instead.
    pub fn from_raw_parts(
        signed: bool,
        year: i32,
        week: u32,
        last_update: chrono::NaiveDateTime,
        days: HashMap<String, Vec<String>>,
    ) -> Self {
        WeeklyReport {
            signed,
            year,
            week,
            last_update,
            days,
        }
    }

    /// Adds either a new collection of activities for the specified day or adds the
    /// activity **to the back** of the activities.
    ///
    /// # Note
    /// This sets last_update to the current timestamp, so **INFORM THE USER**
    /// before doing this.
    pub fn add_day(&mut self, day: &str, activity: &str) {
        if let Some(mut prev) = self
            .days
            .insert(day.to_string(), vec![activity.to_string()])
        {
            prev.push(activity.to_string());
            self.days.insert(day.to_string(), prev);
        };

        self.last_update = chrono::Utc::now().naive_utc();
    }

    /// Set the last_update property.
    ///
    /// This should never be necessary.
    ///
    /// Not that this also does not update the signature status.
    pub fn set_last_update(&mut self, last_update: chrono::NaiveDateTime) {
        self.last_update = last_update;
    }

    /// Getter for the last update
    pub fn get_last_update(&self) -> chrono::NaiveDateTime {
        self.last_update
    }

    /// Getter for the year
    pub fn get_year(&self) -> i32 {
        self.year
    }

    /// Getter for the week number
    pub fn get_week(&self) -> u32 {
        self.week
    }

    /// Set ALL THE DAYS. At this point, you'll probably rather use either
    /// [WeeklyReport::new] or just [WeeklyReport::add_day].
    ///
    /// # Note
    /// This sets last_update to the current timestamp, so **INFORM THE USER**
    /// before doing this.
    pub fn set_days(&mut self, activities: HashMap<String, Vec<String>>) {
        self.days = activities;
        self.last_update = chrono::Utc::now().naive_utc();
        self.signed = false;
    }

    /// Getter for the activities.
    pub fn get_days(&self) -> HashMap<String, Vec<String>> {
        self.days.clone()
    }

    /// Attest that the current version of this report has been signed.
    pub fn set_signed(&mut self) {
        self.signed = true;
    }

    /// There is no good reason you should ever call this function.
    ///
    /// It is purely there to tell you this in the documentation before you try to
    /// do so anyway.
    pub fn revoke_signature(&mut self) {
        self.signed = false;
    }

    /// Getter for the signature status.
    pub fn is_signed(&self) -> bool {
        self.signed
    }

    /// Get a representative timestamp for this week (Monday 00:00:00).
    /// Useful for compatibility and display purposes.
    pub fn get_week_start_timestamp(&self) -> chrono::NaiveDateTime {
        chrono::NaiveDate::from_isoywd_opt(self.year, self.week, chrono::Weekday::Mon)
            .unwrap_or_else(|| {
                log::warn!("Invalid ISO week: {} week {}", self.year, self.week);
                chrono::NaiveDate::from_ymd_opt(self.year, 1, 1).unwrap()
            })
            .and_hms_opt(0, 0, 0)
            .unwrap()
    }

    /// Sets up a basic week view with all days in order.
    ///
    /// You still need to bind the ListStore because that is stored in
    /// FITSWriterWindow.
    pub fn get_day_widgets(&self) -> Vec<gui::widgets::Day> {
        let complete_days = local::dates::ensure_complete_week(&mut self.days.clone(), self);
        let mut sorted_days: Vec<(String, Vec<String>)> = complete_days.into_iter().collect();
        sorted_days.sort_by_key(|(day, _)| local::dates::day_sort_key(day));

        let mut res = Vec::new();
        for (day, _) in sorted_days {
            let day_box = gui::setup::create_day(&day);
            res.push(day_box);
        }
        res
    }
}

// Our own little async runtime, built on glib.
// ~~stolen~~ borrowed from
// https://mmstick.github.io/gtkrs-tutorials/1x03-glib-runtime.html

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_default()
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
