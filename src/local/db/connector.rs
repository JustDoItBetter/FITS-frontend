//! Create abstractions that are nice to work with for the rest of the application
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;

use super::{DbAnswer, DbCommand, DbRequest};

use chrono::Datelike;
use rusqlite::Connection;
use std::fmt::Debug;
use std::sync::mpsc;

/// Wrapper over a [mpsc::Sender] for convenient communication with the database on
/// a separate thread.
///
/// Because this is essentially just a sender, it can be freely cloned, is Send and
/// is Sync.
#[derive(Clone, Debug)]
pub struct DbConnector {
    sender: mpsc::Sender<DbRequest>,
}

impl DbConnector {
    pub async fn open(path: &str) -> Result<Self, common::LocalError> {
        log::debug!("Opening database");

        // Check if database exists, create if not
        if !std::path::PathBuf::from(path).exists() {
            log::warn!("Database not yet found, trying to create it");
            if let Err(e) = super::create_db() {
                if e != common::LocalError::AlreadyExists {
                    log::error!("Failed to create database at {:#?}: {:#?}", &path, e);
                    return Err(common::LocalError::DbError);
                }
            } else {
                log::info!("Database created successfully at {:#?}", &path);
            }
        }

        let db_conn = match Connection::open(path) {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Failed to open database at {:#?}: {}", &path, e);
                return Err(common::LocalError::DbError);
            }
        };

        // Ensure tables exist
        if let Err(e) = super::schema::create_tables(&db_conn) {
            log::error!("Failed to create database tables: {}", e);
            return Err(common::LocalError::DbError);
        }

        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            run_db(db_conn, receiver);
        });

        Ok(DbConnector { sender })
    }

    /// Gets the requested weeks from the database. See
    /// [queries::get_weeks][super::queries::get_weeks] for more information, as
    /// this is just a thin wrapper over it.
    pub fn get_weeks(
        &self,
        time: std::ops::Range<i64>,
    ) -> Result<Vec<common::WeeklyReport>, common::LocalError> {
        let (sender, receiver) = mpsc::channel();

        let req = DbRequest {
            command: DbCommand::Read(time.clone()),
            receiver: sender,
        };

        if let Err(e) = self.sender.send(req) {
            log::error!("Failed to send command to database: {e}");
            return Err(common::LocalError::DbError);
        }

        let Ok(response) = receiver.recv() else {
            log::error!("Could not get response from database");
            return Err(common::LocalError::DbError);
        };

        match response {
            DbAnswer::Read(data) => Ok(data),
            DbAnswer::Err => {
                let start = chrono::DateTime::from_timestamp(time.start, 0)
                    .map(|dt| dt.to_utc().to_string())
                    .unwrap_or_else(|| time.start.to_string());
                let end = chrono::DateTime::from_timestamp(time.end, 0)
                    .map(|dt| dt.to_utc().to_string())
                    .unwrap_or_else(|| time.end.to_string());
                log::warn!(
                    "Database could not find the weeks from {} to {}",
                    start,
                    end
                );
                Err(common::LocalError::DbError)
            }
            _ => {
                log::error!("Got invalid response from database: {:#?}", response);
                Err(common::LocalError::DbError)
            }
        }
    }

    /// Adds an activity to the database. See
    /// [queries::add_activity][super::queries::add_activity] for more information, as
    /// this is just a thin wrapper over it.
    pub fn add_activity(&self, activity: &str) -> Result<(), common::LocalError> {
        log::debug!("Adding activity {activity}");
        let (sender, receiver) = mpsc::channel();

        let req = DbRequest {
            command: DbCommand::AddActivity(activity.to_string()),
            receiver: sender,
        };

        if let Err(e) = self.sender.send(req) {
            log::error!("Failed to send command to database: {e}");
            return Err(common::LocalError::DbError);
        }

        let Ok(response) = receiver.recv() else {
            log::error!("Could not get response from database");
            return Err(common::LocalError::DbError);
        };

        match response {
            DbAnswer::Ok => Ok(()),
            DbAnswer::Err => {
                log::warn!("Database could not add activity: {}", activity);
                Err(common::LocalError::DbError)
            }
            _ => {
                log::error!("Got invalid response from database: {:#?}", response);
                Err(common::LocalError::DbError)
            }
        }
    }

    pub fn get_activities(&self) -> Result<Vec<String>, common::LocalError> {
        log::debug!("Getting activities from database");
        let (sender, receiver) = mpsc::channel();

        let req = DbRequest {
            command: DbCommand::Activities,
            receiver: sender,
        };

        if let Err(e) = self.sender.send(req) {
            log::error!("Failed to send command to database: {e}");
            return Err(common::LocalError::DbError);
        }

        let Ok(response) = receiver.recv() else {
            log::error!("Could not get response from database");
            return Err(common::LocalError::DbError);
        };

        match response {
            DbAnswer::Activities(data) => Ok(data),
            DbAnswer::Err => {
                log::warn!("Database could not get activities");
                Err(common::LocalError::DbError)
            }
            _ => {
                log::error!("Got invalid response from database: {:#?}", response);
                Err(common::LocalError::DbError)
            }
        }
    }

    pub fn remove_activity(&self, activity: &str) -> Result<(), common::LocalError> {
        log::debug!("Removing activity {activity}");
        let (sender, receiver) = mpsc::channel();

        let req = DbRequest {
            command: DbCommand::RemoveActivity(activity.to_string()),
            receiver: sender,
        };

        if let Err(e) = self.sender.send(req) {
            log::error!("Failed to send command to database: {e}");
            return Err(common::LocalError::DbError);
        }

        let Ok(response) = receiver.recv() else {
            log::error!("Could not get response from database");
            return Err(common::LocalError::DbError);
        };

        match response {
            DbAnswer::Ok => Ok(()),
            DbAnswer::Err => {
                log::warn!("Database could not remove activity: {}", activity);
                Err(common::LocalError::DbError)
            }
            _ => {
                log::error!("Got invalid response from database: {:#?}", response);
                Err(common::LocalError::DbError)
            }
        }
    }

    pub fn add_daily_activity(
        &self,
        activity: &str,
        day: &str,
        position: i64,
        year: i32,
        week: u32,
    ) {
        log::debug!("Adding daily activity {activity}");
        let (sender, receiver) = mpsc::channel();
        let req = DbRequest {
            command: DbCommand::AddDailyActivity {
                year,
                week,
                position,
                day: day.to_string(),
                activity: activity.to_string(),
            },
            receiver: sender,
        };

        if let Err(e) = self.sender.send(req) {
            log::error!("Failed to send command to database: {e}");
            return;
        }
        let Ok(response) = receiver.recv() else {
            log::error!("Could not get response from database");
            return;
        };

        match response {
            DbAnswer::Ok => (),
            DbAnswer::Err => {
                log::warn!("Database could not add daily activity: {}", activity);
            }
            _ => {
                log::error!("Got invalid response from database: {:#?}", response);
            }
        }
    }

    pub fn remove_daily_activity(&self, day: &str, position: i64, year: i32, week: u32) {
        log::debug!("Removing daily activity at position {position} on day {day}");
        let (sender, receiver) = mpsc::channel();
        let req = DbRequest {
            command: DbCommand::RemoveDailyActivity {
                year,
                week,
                day: day.to_string(),
                position,
            },
            receiver: sender,
        };

        if let Err(e) = self.sender.send(req) {
            log::error!("Failed to send command to database: {e}");
            return;
        }
        let Ok(response) = receiver.recv() else {
            log::error!("Could not get response from database");
            return;
        };

        match response {
            DbAnswer::Ok => (),
            DbAnswer::Err => {
                log::warn!(
                    "Database could not remove daily activity at position {} on day {}",
                    position,
                    day
                );
            }
            _ => {
                log::error!("Got invalid response from database: {:#?}", response);
            }
        }
    }
}

/// Runs the db and listens for incoming commands
///
/// This function should be run on its own thread (possibly async) because it spends
/// a lot of time waiting for I/O
fn run_db(mut conn: Connection, commands: mpsc::Receiver<DbRequest>) {
    use DbCommand::*;

    while let Ok(req) = commands.recv() {
        match req.command {
            Read(time) => get_weeks(time, req.receiver, &mut conn),
            Save { data } => save(data, req.receiver, &mut conn),
            Backup => todo!(),
            Activities => get_activities(req.receiver, &mut conn),
            AddActivity(name) => add_activity(&name, req.receiver, &mut conn),
            RemoveActivity(name) => remove_activity(&name, req.receiver, &mut conn),
            AddDailyActivity {
                year,
                week,
                day,
                position,
                activity,
            } => {
                if let Err(e) = super::queries::add_activity_to_day_by_name(
                    &conn, year, week, &day, &activity, position,
                ) {
                    log::error!(
                        "Could not add activity {} to day {} of week {}-{}: {}",
                        &activity,
                        &day,
                        &year,
                        &week,
                        e
                    );
                    let _ = req.receiver.send(DbAnswer::Err);
                } else {
                    let _ = req.receiver.send(DbAnswer::Ok);
                }
            }
            RemoveDailyActivity {
                year,
                week,
                day,
                position,
            } => {
                if let Err(e) =
                    super::queries::remove_activity_from_day(&conn, year, week, &day, position)
                {
                    log::error!(
                        "Could not remove activity from day {} of week {}-{}: {}",
                        &day,
                        &year,
                        &week,
                        e
                    );
                    let _ = req.receiver.send(DbAnswer::Err);
                } else {
                    let _ = req.receiver.send(DbAnswer::Ok);
                }
            }
        };
    }
}

pub fn timestamp_range_to_weeks(time: std::ops::Range<i64>) -> Option<((i32, u32), (i32, u32))> {
    let start_dt = chrono::DateTime::from_timestamp(time.start, 0)?;
    let end_dt = chrono::DateTime::from_timestamp(time.end, 0)?;

    let start_week = start_dt.naive_utc().iso_week();
    let end_week = end_dt.naive_utc().iso_week();

    Some((
        (start_week.year(), start_week.week()),
        (end_week.year(), end_week.week()),
    ))
}

/// Get data for the specified timespan
pub fn get_weeks(time: std::ops::Range<i64>, ret: mpsc::Sender<DbAnswer>, conn: &mut Connection) {
    let Some(((start_year, start_week), (end_year, end_week))) =
        timestamp_range_to_weeks(time.clone())
    else {
        let _ = ret.send(DbAnswer::Err);
        log::error!("Failed to parse time for timestamp {}", &time.start);
        return;
    };

    if chrono::DateTime::from_timestamp(time.end, 0).is_none() {
        let _ = ret.send(DbAnswer::Err);
        log::error!("Failed to parse time for timestamp {}", time.end);
        return;
    };

    // Get weekly reports directly as common::WeeklyReport
    let Ok(reports) =
        super::queries::get_weekly_reports(conn, (start_year, start_week), (end_year, end_week))
    else {
        let _ = ret.send(DbAnswer::Err);
        log::error!(
            "Failed to load weekly reports for {}-{} to {}-{}",
            &start_year,
            &start_week,
            &end_year,
            &end_week
        );
        return;
    };

    let _ = ret.send(DbAnswer::Read(reports));
}

/// Save the given data to the db.
pub fn save(data: Vec<common::WeeklyReport>, ret: mpsc::Sender<DbAnswer>, conn: &mut Connection) {
    log::debug!("Saving data to database");
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Could not start transaction: {}", e);
            let _ = ret.send(DbAnswer::Err);
            return;
        }
    };

    for report in &data {
        if let Err(e) = super::queries::insert_weekly_report(&tx, report) {
            log::error!("Could not save weekly report: {}", e);
            let _ = ret.send(DbAnswer::Err);
            return;
        }
    }

    // Commit the transaction
    if let Err(e) = tx.commit() {
        log::error!("Could not commit transaction: {}", e);
        let _ = ret.send(DbAnswer::Err);
        return;
    }

    let _ = ret.send(DbAnswer::Ok);
}

/// Get all available activities from the database
pub fn get_activities(ret: mpsc::Sender<DbAnswer>, conn: &mut Connection) {
    let Ok(activities) = super::queries::get_all_activities(conn) else {
        log::error!("Could not load available activities from database");
        let _ = ret.send(DbAnswer::Err);
        return;
    };

    let _ = ret.send(DbAnswer::Activities(activities));
}

/// Add a new activity to the database
pub fn add_activity(name: &str, ret: mpsc::Sender<DbAnswer>, conn: &mut Connection) {
    if let Err(e) = super::queries::insert_available_activity(conn, name) {
        log::error!("Could not add new activity with error: {}", e);
        let _ = ret.send(DbAnswer::Err);
        return;
    }

    let _ = ret.send(DbAnswer::Ok);
}

/// Remove an activity from the database
pub fn remove_activity(name: &str, ret: mpsc::Sender<DbAnswer>, conn: &mut Connection) {
    if let Err(e) = super::queries::remove_activity_by_name(conn, name) {
        log::error!("Could not remove activity with error: {}", e);
        let _ = ret.send(DbAnswer::Err);
        return;
    }

    let _ = ret.send(DbAnswer::Ok);
}
