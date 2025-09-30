// All the things that need to happen on the local filesystem
// SPDX-License-Identifier: GPL-3.0-only

pub mod db;
pub mod keyring;
pub mod paths;

use crate::common;

/// Loads persistent data from the disk to build the GUI
///
/// Returns a config to be used for building the GUI and the state for the
/// application.
///
/// If a config cannot be found, an error will be printed and defaults will be used
/// for convenience.
pub async fn load_data() -> Result<(common::State, common::Config), common::LocalError> {
    let state = load_state().await?;
    let config = load_config().unwrap_or_else(|_| {
        let path = paths::get_config_path();
        log::warn!("Could not load config, using default values!");
        log::warn!("Creating default config at {:#?}", path);
        let config = common::Config::default();
        if std::fs::write(path, toml::to_string(&config).unwrap()).is_err() {
            log::warn!(
                "Failed to create default config, please review your filesystem permissions"
            );
            log::warn!("Still proceeding with defaults");
        }
        config
    });
    Ok((state, config))
}

/// Loads just the config and returns it appropriately
pub fn load_config() -> Result<common::Config, common::LocalError> {
    common::Config::from_file(None)
}

pub async fn load_state() -> Result<common::State, common::LocalError> {
    let conn = db::open().await?;
    let username = keyring::get_username()?;
    let password = keyring::get_password(&username)?;
    Ok(common::State::new(conn, username, password))
}
