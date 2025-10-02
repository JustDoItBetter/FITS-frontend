//! Create abstractions that are nice to work with for the rest of the application
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;

use super::{DbAnswer, DbCommand, DbRequest};

use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::prelude::*;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::mpsc;

/// Wrapper over a [mpsc::Sender] for convenient communication with the database on
/// a seperate thread.
///
/// Because this is essentially just a sender, it can be freely cloned, is Send and
/// is Sync.
#[derive(Clone, Debug)]
pub struct DbConnector {
    sender: mpsc::Sender<DbRequest>,
}

impl DbConnector {
    pub async fn open(path: &str) -> Result<Self, common::LocalError> {
        let ctx = SessionContext::new();
        let Ok(df) = ctx
            .read_parquet(path, ParquetReadOptions::new().schema(&get_report_schema()))
            .await
        else {
            log::error!("Failed to read database at {:#?}", &path);
            return Err(common::LocalError::DbError);
        };

        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            run_db(ctx, df, receiver);
        });

        Ok(DbConnector { sender })
    }
}

/// Runs the db and listens for incoming commands
///
/// This function should be run on its own thread (possibly async) because it spends
/// a lot of time waiting for I/O
fn run_db(_ctx: SessionContext, df: DataFrame, commands: mpsc::Receiver<DbRequest>) {
    use DbCommand::*;

    while let Ok(req) = commands.recv() {
        match req.command {
            Backup(time) => common::block_on(create_backup(time, req.receiver, df.clone())),
            SaveData { data } => save(data, req.receiver),
        };
    }
}

/// TODO: Get the size of the dataframe and preallocate the vector accordingly.
async fn create_backup(time: std::ops::Range<u64>, ret: mpsc::Sender<DbAnswer>, df: DataFrame) {
    match df.filter(col("timestamp").lt_eq(lit(time.end)).gt_eq(lit(time.start))) {
        Ok(res) => {
            match res.collect().await {
                Ok(batches) => {
                    if let Err(e) = ret.send(DbAnswer::Backup(batches)) {
                        log::warn!("Failed to send backup to GUI with error: {e}");
                    }
                }
                Err(_) => {
                    log::warn!("Backup creation failed: could not collect results");
                }
            }
        }
        Err(_) => {
            log::warn!("Backup creation failed: could not filter dataframe");
        }
    }
}

fn save(_data: Vec<common::WeeklyReport>, _ret: mpsc::Sender<DbAnswer>) {}

/// Schema for the weekly reports db
///
/// # Structure:
/// - timestamp: u64
/// - signed: bool
/// - days: List<Day>
///              \-> day: List<Activity>
///                            \-> activity: String
pub fn get_report_schema() -> Schema {
    // TODO: experiment with Utf8View?
    let activity = Field::new("activity", DataType::Utf8, false);
    let day = Field::new("activities", DataType::List(Arc::new(activity)), false);

    let days = Field::new("days", DataType::List(Arc::new(day)), false);

    // timestamp does not use the builtin type because it must a specific day
    // (evenly divisible by 86_400_000, see
    // <https://docs.rs/arrow/56.2.0/arrow/datatypes/enum.DataType.html#variant.Date64>
    // for more).
    let timestamp = Field::new("timestamp", DataType::UInt64, false);
    let signed = Field::new("signed", DataType::Boolean, false);

    Schema::new(vec![timestamp, signed, days])
}
