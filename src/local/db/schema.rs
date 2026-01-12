//! SQL table schema
//!
//! TODO: This should probably be reworked so we can simply implement [From]
//!  [WeeklyReport] on [common::WeeklyReport] so there is no ugly parsing in random
//!  functions.
// SPDX-License-Identifier: GPL-3.0-only

/// SQL schema for creating the database tables
pub const CREATE_WEEKLY_REPORTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS weekly_reports (
    year INTEGER NOT NULL,
    week INTEGER NOT NULL,
    signed INTEGER NOT NULL,
    last_update DATETIME NOT NULL,
    primary key (year, week)
)
"#;

pub const CREATE_ACTIVITIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS activities (
    year INTEGER NOT NULL,
    week INTEGER NOT NULL,
    day TEXT NOT NULL,
    activity INTEGER NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (year, week, day, activity),
    FOREIGN KEY (year, week) REFERENCES weekly_reports(year, week),
    FOREIGN KEY (activity) REFERENCES available_activities(ident)
)
"#;

pub const CREATE_AVAILABLE_ACTIVITIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS available_activities (
    ident INTEGER PRIMARY KEY,
    activity TEXT NOT NULL UNIQUE
)
"#;

pub fn create_tables(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute(CREATE_WEEKLY_REPORTS_TABLE, [])?;
    conn.execute(CREATE_AVAILABLE_ACTIVITIES_TABLE, [])?;
    conn.execute(CREATE_ACTIVITIES_TABLE, [])?;
    Ok(())
}
