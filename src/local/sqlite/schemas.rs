// Defines constants for the Sqlite schemas
// SPDX-License-Identifier: GPL-3.0-only

/// SQL for creating the table notes which saves the weekly notes.
///
/// date specifies the date for the note in the format $Year$Week, so a note for the
/// week from 2025-08-11 to 2025-08-17 would have the date 202533 (2025 for the
/// year, 33 because `date +%V --date=2025-08-11` or the calendar week is 33).
/// Recommendation: Increase this until $Week would hit 53, and instead add 49
///
/// note simply contains the note text in markdown.
///
/// TODO: Benchmark with large amounts of notes if compression is benefitial
pub static NOTES_TABLE: &'static str = "
    CREATE TABLE notes(
        date INTEGER PRIMARY KEY,
        note TEXT NOT NULL
    );
";

/// SQL for creating the table profiles to save daily activity.
///
/// date specifies the day for the entry as an ISO 8601 string, e.g. "2025-08-13".
///
/// begin_time and end_time give the beginning and end of the activity as a unix
/// timestamp.
///
/// place specifies the place as a string, e.g. "Work".
///
/// attendance specifies if the activity was attended with a "yes" or a reason for
/// not attending.
pub static PROFILE_TABLE: &'static str = "
    CREATE TABLE profiles(
        date TEXT PRIMARY KEY,
        begin_time INT NOT NULL,
        end_time INT NOT NULL,
        place TEXT NOT NULL,
        attendance
    );
";

// TODO: Create a table for patterns
