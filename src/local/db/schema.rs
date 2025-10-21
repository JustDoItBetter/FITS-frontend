//! SQL table schema
//!
//! TODO: This should probably be reworked so we can simply implement [From]
//!  [WeeklyReport] on [common::WeeklyReport] so there is no ugly parsing in random
//!  functions.
// SPDX-License-Identifier: GPL-3.0-only

use diesel::prelude::*;

diesel::table!(
    weekly_reports(timestamp) {
        signed -> Bool,
        timestamp -> Timestamp,
        last_update -> Timestamp,
    }
);

diesel::table!(
    activities(timestamp, day) {
        timestamp -> Timestamp,
        day -> Text,
        activity -> Text,
    }
);

diesel::joinable!(activities -> weekly_reports (timestamp));
diesel::allow_tables_to_appear_in_same_query!(activities, weekly_reports);

#[derive(Queryable, Identifiable, Selectable, Insertable)]
#[diesel(primary_key(timestamp))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WeeklyReport {
    pub signed: bool,
    pub timestamp: chrono::NaiveDateTime,
    pub last_update: chrono::NaiveDateTime,
}

#[derive(Queryable, Associations, Selectable, Insertable)]
#[diesel(table_name = activities)]
#[diesel(belongs_to(WeeklyReport, foreign_key=timestamp))]
#[diesel(primary_key(timestamp, day))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Activity {
    pub timestamp: chrono::NaiveDateTime,
    /// Must be one of "Monday", "Tuesday", "Wednesday", "Thursday", "Friday",
    /// "Saturday", "Sunday".
    pub day: String,
    pub activity: String,
}
