//! Handle some things related to dates
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;
use chrono::Datelike;

/// Ensures that all 7 days of the week are present in the days map
pub fn ensure_complete_week(
    days_map: &mut std::collections::HashMap<String, Vec<String>>,
    report: &common::WeeklyReport,
) -> std::collections::HashMap<String, Vec<String>> {
    let day_format = detect_day_format(days_map);

    let all_days = get_all_days_in_format(&day_format, report.get_week_start_timestamp());

    let mut complete_days = days_map.clone();

    for day in all_days {
        complete_days.entry(day).or_default();
    }

    complete_days
}

/// Detects the format being used for day names in the existing data
fn detect_day_format(days_map: &std::collections::HashMap<String, Vec<String>>) -> DayFormat {
    if days_map.is_empty() {
        return DayFormat::FullName;
    }

    for day in days_map.keys() {
        let day_lower = day.to_lowercase();

        if matches!(
            day_lower.as_str(),
            "monday" | "tuesday" | "wednesday" | "thursday" | "friday" | "saturday" | "sunday"
        ) {
            return DayFormat::FullName;
        }

        if matches!(
            day_lower.as_str(),
            "mon" | "tue" | "wed" | "thu" | "fri" | "sat" | "sun"
        ) {
            return DayFormat::Abbreviated;
        }

        if chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").is_ok() {
            return DayFormat::DateISO;
        }
        if chrono::NaiveDate::parse_from_str(day, "%m/%d/%Y").is_ok() {
            return DayFormat::DateUS;
        }
        if chrono::NaiveDate::parse_from_str(day, "%d/%m/%Y").is_ok() {
            return DayFormat::DateEU;
        }
    }

    DayFormat::FullName
}

/// Enum to represent different day formats
#[derive(Debug, Clone)]
enum DayFormat {
    /// Monday, Tuesday, etc.
    FullName,
    /// Mon, Tue, etc.
    Abbreviated,
    /// 2023-12-18
    DateISO,
    /// 12/18/2023
    DateUS,
    /// 18/12/2023
    DateEU,
}

/// Gets all 7 days of the week in the specified format
fn get_all_days_in_format(format: &DayFormat, timestamp: chrono::NaiveDateTime) -> Vec<String> {
    match format {
        DayFormat::FullName => vec![
            "Monday".to_string(),
            "Tuesday".to_string(),
            "Wednesday".to_string(),
            "Thursday".to_string(),
            "Friday".to_string(),
            "Saturday".to_string(),
            "Sunday".to_string(),
        ],
        DayFormat::Abbreviated => vec![
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
            "Sat".to_string(),
            "Sun".to_string(),
        ],
        DayFormat::DateISO | DayFormat::DateUS | DayFormat::DateEU => {
            let date = timestamp.date();
            let monday =
                date - chrono::Duration::days(date.weekday().num_days_from_monday() as i64);

            (0..7)
                .map(|i| {
                    let day_date = monday + chrono::Duration::days(i);
                    match format {
                        DayFormat::DateISO => day_date.format("%Y-%m-%d").to_string(),
                        DayFormat::DateUS => day_date.format("%m/%d/%Y").to_string(),
                        DayFormat::DateEU => day_date.format("%d/%m/%Y").to_string(),
                        _ => unreachable!(),
                    }
                })
                .collect()
        }
    }
}

/// Helper function to create a sort key for days to ensure chronological ordering
pub fn day_sort_key(day: &str) -> u8 {
    match day.to_lowercase().as_str() {
        "monday" | "mon" => 1,
        "tuesday" | "tue" | "tues" => 2,
        "wednesday" | "wed" => 3,
        "thursday" | "thu" | "thur" | "thurs" => 4,
        "friday" | "fri" => 5,
        "saturday" | "sat" => 6,
        "sunday" | "sun" => 7,
        // If it's not a recognizable day name, try to parse as a date
        _ => {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d") {
                return date.weekday().number_from_monday() as u8;
            }
            if let Ok(date) = chrono::NaiveDate::parse_from_str(day, "%m/%d/%Y") {
                return date.weekday().number_from_monday() as u8;
            }
            if let Ok(date) = chrono::NaiveDate::parse_from_str(day, "%d/%m/%Y") {
                return date.weekday().number_from_monday() as u8;
            }
            255
        }
    }
}
