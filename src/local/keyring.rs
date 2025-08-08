// Dealing with the keyring
// SPDX-License-Identifier: GPL-3.0-only
use crate::common;
use keyring::Entry;

pub fn get_password(user: &str) -> Result<String, common::LocalError> {
    let Ok(entry) = Entry::new("fits", user) else {
        return Err(common::LocalError::KeyringError);
    };
    match entry.get_password() {
        Ok(pw) => Ok(pw),
        Err(_) => Err(common::LocalError::KeyringError),
    }
}

/// Get the username (which is also saved in the keyring)
/// This feels kinda hacky, but it really does not make sense to save it in sqlite
/// or to add a THIRD data location
pub fn get_username() -> Result<String, common::LocalError> {
    let Ok(entry) = Entry::new("fits", "username") else {
        return Err(common::LocalError::KeyringError);
    };
    match entry.get_password() {
        Ok(username) => Ok(username),
        Err(_) => Err(common::LocalError::KeyringError),
    }
}

pub fn save_credentials(username: &str, password: &str) -> Result<(), common::LocalError> {
    let Ok(username_entry) = Entry::new("fits", "username") else {
        return Err(common::LocalError::KeyringError);
    };
    if username_entry.set_password(username).is_err() {
        return Err(common::LocalError::KeyringError);
    }

    let Ok(actual_entry) = Entry::new("fits", username) else {
        return Err(common::LocalError::KeyringError);
    };
    if actual_entry.set_password(password).is_err() {
        return Err(common::LocalError::KeyringError);
    }
    Ok(())
}
