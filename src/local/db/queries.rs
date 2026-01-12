//! Commands to be executed on the db
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;
use rusqlite::{Connection, Result as SqliteResult, params};

/// Insert a weekly report into the database
pub fn insert_weekly_report(conn: &Connection, report: &common::WeeklyReport) -> SqliteResult<()> {
    conn.execute(
           "INSERT OR REPLACE INTO weekly_reports (year, week, signed, last_update) VALUES (?1, ?2, ?3, ?4)",
           params![report.get_year(), report.get_week(), report.is_signed() as i32, report.get_last_update()],
       )?;

    insert_activities_for_report(conn, report)
}

/// Insert activities for a weekly report
fn insert_activities_for_report(
    conn: &Connection,
    report: &common::WeeklyReport,
) -> SqliteResult<()> {
    // First, remove existing activities for this timestamp to avoid duplicates
    conn.execute(
        "DELETE FROM activities WHERE year = ?1 AND week = ?2",
        params![report.get_year(), report.get_week()],
    )?;

    for (day, activities) in report.get_days() {
        for activity_str in activities {
            if let Ok(activity_index) = activity_str.parse::<i64>() {
                conn.execute(
                    "INSERT INTO activities (year, week, day, activity) VALUES (?1, ?2, ?3, ?4)",
                    params![report.get_year(), report.get_week(), day, activity_index],
                )?;
            } else {
                log::warn!("Could not parse activity index: {}", activity_str);
            }
        }
    }
    Ok(())
}

/// Insert a new available activity
pub fn insert_available_activity(conn: &Connection, activity: &str) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO available_activities (activity) VALUES (?1)",
        params![activity],
    )?;
    Ok(())
}

/// Get weekly reports in a time range
///
/// start_time and end_time are tuples of (year, week)
pub fn get_weekly_reports(
    conn: &Connection,
    start_time: (i32, u32),
    end_time: (i32, u32),
) -> SqliteResult<Vec<common::WeeklyReport>> {
    // Get all weekly reports in the range
    let mut stmt = conn.prepare(
        "SELECT year, week, signed, last_update FROM weekly_reports
                 WHERE (year > ?1) OR (year = ?1 AND week >= ?2)
                 AND (year < ?3) OR (year = ?3 AND week <= ?4)
                 ORDER BY year DESC, week DESC",
    )?;

    let report_rows = stmt.query_map(
        params![start_time.0, start_time.1, end_time.0, end_time.1],
        |row| {
            let year: i32 = row.get(0)?;
            let week: u32 = row.get::<_, u32>(1)?;
            let signed: i32 = row.get(2)?;
            let last_update: chrono::NaiveDateTime = row.get(3)?;

            Ok((year, week, signed != 0, last_update))
        },
    )?;

    let mut results = Vec::new();

    for report_result in report_rows {
        let (year, week, signed, last_update) = report_result?;

        let activities = get_activities_for_week(conn, year, week)?;

        let report =
            common::WeeklyReport::from_raw_parts(signed, year, week, last_update, activities);

        results.push(report);
    }

    Ok(results)
}

/// Get activities for a specific timestamp, returning a HashMap as expected by WeeklyReport
fn get_activities_for_week(
    conn: &Connection,
    year: i32,
    week: u32,
) -> SqliteResult<std::collections::HashMap<String, Vec<String>>> {
    let mut stmt = conn.prepare(
        "SELECT day, activity FROM activities WHERE year = ?1 AND week = ?2 ORDER BY day",
    )?;

    let activity_rows = stmt.query_map(params![year, week], |row| {
        let day: String = row.get(0)?;
        let activity_index: i64 = row.get(1)?;
        Ok((day, activity_index))
    })?;

    let mut activities_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for activity_result in activity_rows {
        let (day, activity_index) = activity_result?;

        // Convert activity index to activity name
        let activity_name = get_activity_by_index(conn, activity_index)?.unwrap_or_else(|| {
            log::warn!("Unknown activity index: {}", activity_index);
            format!("Unknown({})", activity_index)
        });

        activities_map.entry(day).or_default().push(activity_name);
    }

    Ok(activities_map)
}

/// Get an activity name by its index
pub fn get_activity_by_index(conn: &Connection, index: i64) -> SqliteResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT activity FROM available_activities WHERE ident = ?1")?;
    let mut rows = stmt.query_map(params![index], |row| row.get::<_, String>(0))?;

    match rows.next() {
        Some(result) => result.map(Some),
        None => Ok(None),
    }
}

/// Get all available activities
pub fn get_all_activities(conn: &Connection) -> SqliteResult<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT activity FROM available_activities ORDER BY activity ASC")?;
    let activity_rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut activities = Vec::new();
    for activity_result in activity_rows {
        activities.push(activity_result?);
    }
    Ok(activities)
}

/// Remove an available activity by name
///
/// # Note
/// This will also remove all references to this activity in the activities table,
/// so inform the user before doing this.
pub fn remove_activity_by_name(conn: &Connection, name: &str) -> SqliteResult<()> {
    conn.execute(
        "DELETE FROM activities WHERE activity IN (SELECT ident FROM available_activities WHERE activity = ?1)",
        params![name],
    )?;
    conn.execute(
        "DELETE FROM available_activities WHERE activity = ?1",
        params![name],
    )?;
    Ok(())
}

/// Removes the activity specified by the index from the specified day in the
/// specified week.
///
/// Do note that day_position refers to the index **within the day**, which is the
/// index used by the gtk listmodel. This is **NOT** the activity ident index
/// used in the database.
pub fn remove_activity_from_day(
    conn: &Connection,
    year: i32,
    week: u32,
    day: &str,
    day_position: i64,
) -> SqliteResult<()> {
    conn.execute(
        "DELETE FROM activities WHERE year = ?1 AND week = ?2 AND day = ?3 AND position = ?4",
        params![year, week, day, day_position],
    )?;
    Ok(())
}

pub fn add_activity_to_day_by_index(
    conn: &Connection,
    year: i32,
    week: u32,
    day: &str,
    activity_index: i64,
) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO activities (year, week, day, activity) VALUES (?1, ?2, ?3, ?4)",
        params![year, week, day, activity_index],
    )?;
    Ok(())
}

pub fn add_activity_to_day_by_name(
    conn: &Connection,
    year: i32,
    week: u32,
    day: &str,
    activity_name: &str,
    position: i64,
) -> SqliteResult<()> {
    let mut stmt = conn.prepare("SELECT ident FROM available_activities WHERE activity = ?1")?;
    let mut rows = stmt.query_map(params![activity_name], |row| row.get::<_, i64>(0))?;

    if let Some(result) = rows.next() {
        let activity_index = result?;
        conn.execute(
            "INSERT INTO activities (year, week, day, activity, position) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![year, week, day, activity_index, position],
        )?;
    } else {
        log::warn!("Activity not found: {}", activity_name);
    }

    Ok(())
}
