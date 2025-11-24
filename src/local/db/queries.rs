//! Commands to be executed on the db
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;
use crate::local::db::DbAnswer;

use diesel::prelude::*;
use std::sync::mpsc;

/// Get data for the specified timespan
pub fn get_weeks(
    time: std::ops::Range<i64>,
    ret: mpsc::Sender<DbAnswer>,
    conn: &mut SqliteConnection,
) {
    use super::schema::*;

    let Some(time_start) = chrono::DateTime::from_timestamp(time.start, 0) else {
        let _ = ret.send(DbAnswer::Err);
        log::error!("Failed to parse time for timestamp {}", time.start);
        return;
    };

    let Some(time_end) = chrono::DateTime::from_timestamp(time.end, 0) else {
        let _ = ret.send(DbAnswer::Err);
        log::error!("Failed to parse time for timestamp {}", time.end);
        return;
    };

    // Get all the activities in the timespan
    let Ok(entries) = weekly_reports::table
        .filter(weekly_reports::timestamp.ge(time_start.naive_utc()))
        .filter(weekly_reports::timestamp.le(time_end.naive_utc()))
        .inner_join(activities::table)
        .order(activities::timestamp.desc())
        .select(activities::all_columns)
        .load::<Activity>(conn)
    else {
        let _ = ret.send(DbAnswer::Err);
        log::error!(
            "Failed to load weekly reports for {} to {}",
            &time_start,
            &time_end
        );
        return;
    };

    // Get whether they are signed
    let Ok(mut signed) = weekly_reports::table
        .filter(weekly_reports::timestamp.ge(time_start.naive_utc()))
        .filter(weekly_reports::timestamp.le(time_end.naive_utc()))
        .order(weekly_reports::timestamp.desc())
        .select(weekly_reports::signed)
        .load(conn)
    else {
        let _ = ret.send(DbAnswer::Err);
        log::error!(
            "Failed to load signature status for {} to {}",
            &time_start,
            &time_end
        );
        return;
    };

    let mut res = Vec::new();
    for entry in entries {
        let index = res.len() - 1;

        if check_for_new_entry(&res, &entry) {
            let Some(is_signed) = signed.pop() else {
                let _ = ret.send(DbAnswer::Err);
                log::error!("Found no signature status for date: {}", entry.timestamp);
                return;
            };

            let mut report = common::WeeklyReport::new(is_signed, entry.timestamp, None);
            report.add_day(&entry.day, &entry.activity);
            res.push(report);
            continue;
        } else {
            res[index].add_day(&entry.day, &entry.activity);
        }
    }

    // If all reports were added successfully, this MUST be empty
    assert!(signed.is_empty());

    let _ = ret.send(DbAnswer::Read(res));
}

fn check_for_new_entry(res: &[common::WeeklyReport], current: &super::schema::Activity) -> bool {
    res.is_empty() || res[res.len() - 1].get_timestamp() != current.timestamp
}

/// Save the given data to the db.
///
/// TODO: Clean up the needless complexity in parsing by overthinking decisions made
///  when creating the database format.
pub fn save(
    data: Vec<common::WeeklyReport>,
    ret: mpsc::Sender<DbAnswer>,
    conn: &mut SqliteConnection,
) {
    use super::schema::*;

    let mut reports = Vec::with_capacity(data.len());
    let mut activities = Vec::new();

    for report in data {
        for (day, actions) in report.get_days() {
            for action in actions {
                activities.push(Activity {
                    timestamp: report.get_timestamp(),
                    day: day.clone(),
                    activity: action,
                });
            }
        }
        let parsed_report = WeeklyReport {
            signed: report.is_signed(),
            last_update: report.get_last_update(),
            timestamp: report.get_timestamp(),
        };
        reports.push(parsed_report);
    }

    if let Err(e) = diesel::insert_into(weekly_reports::table)
        .values(&reports)
        .execute(conn)
    {
        log::error!("Could not save weekly reports with error: {}", e);
        let _ = ret.send(DbAnswer::Err);
    }

    if let Err(e) = diesel::insert_into(activities::table)
        .values(&activities)
        .execute(conn)
    {
        log::error!("Could not save activities with error: {}", e);
        let _ = ret.send(DbAnswer::Err);
    }

    let _ = ret.send(DbAnswer::Ok);
}
