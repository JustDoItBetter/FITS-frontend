// All the things that need to happen on the local filesystem
// SPDX-License-Identifier: GPL-3.0-only

pub mod keyring;
mod paths;
pub mod sqlite;

use crate::common;

/// Loads persistent data from the disk to build the GUI
///
/// If anything in here fails, the user will be prompted to log in again
///
/// Returns a config to be used for building the GUI
pub fn load_data() -> Result<common::Config, common::LocalError> {
    let conn = sqlite::connect()?;
    let username = keyring::get_username()?;
    let password = keyring::get_password(&username)?;
    Ok(common::Config::new(conn, username, password))
}
