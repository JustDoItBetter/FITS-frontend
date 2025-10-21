//! Create abstractions that are nice to work with for the rest of the application
// SPDX-License-Identifier: GPL-3.0-only

use crate::common;

use super::{DbAnswer, DbCommand, DbRequest};
use diesel::prelude::*;

use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::Arc;

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
        let complete_path = "file://".to_owned() + path;
        let Ok(db_conn) = SqliteConnection::establish(&complete_path) else {
            log::error!("Failed to read database at {:#?}", &path);
            return Err(common::LocalError::DbError);
        };

        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            run_db(db_conn, receiver);
        });

        Ok(DbConnector { sender })
    }
}

/// Runs the db and listens for incoming commands
///
/// This function should be run on its own thread (possibly async) because it spends
/// a lot of time waiting for I/O
fn run_db(mut conn: SqliteConnection, commands: mpsc::Receiver<DbRequest>) {
    use DbCommand::*;

    while let Ok(req) = commands.recv() {
        match req.command {
            Read(time) => super::queries::get_weeks(time, req.receiver, &mut conn),
            Save { data } => super::queries::save(data, req.receiver, &mut conn),
            Backup => todo!(),
        };
    }
}
