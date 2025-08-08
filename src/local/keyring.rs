// Dealing with the keyring
// SPDX-License-Identifier: GPL-3.0-only
use crate::common;
use keyring::Entry;

pub fn get_password(user: &str) -> Result<String, common::LocalError> {
    let Ok(entry) = Entry::new("fits", user) else {
        generic_keyring_error();
        return Err(common::LocalError::KeyringError);
    };
    match entry.get_password() {
        Ok(pw) => Ok(pw),
        Err(e) => {
            log::warn!("The system keyring produced an error: {e}");
            Err(common::LocalError::KeyringError)
        }
    }
}

/// Get the username (which is also saved in the keyring)
/// This feels kinda hacky, but it really does not make sense to save it in sqlite
/// or to add a THIRD data location
pub fn get_username() -> Result<String, common::LocalError> {
    let Ok(entry) = Entry::new("fits", "username") else {
        generic_keyring_error();
        return Err(common::LocalError::KeyringError);
    };
    match entry.get_password() {
        Ok(username) => Ok(username),
        Err(e) => {
            log::warn!("The system keyring produced an error: {e}");
            Err(common::LocalError::KeyringError)
        }
    }
}

pub fn save_credentials(username: &str, password: &str) -> Result<(), common::LocalError> {
    let Ok(username_entry) = Entry::new("fits", "username") else {
        generic_keyring_error();
        return Err(common::LocalError::KeyringError);
    };
    if username_entry.set_password(username).is_err() {
        generic_keyring_error();
        return Err(common::LocalError::KeyringError);
    }

    let Ok(actual_entry) = Entry::new("fits", username) else {
        generic_keyring_error();
        return Err(common::LocalError::KeyringError);
    };
    if actual_entry.set_password(password).is_err() {
        generic_keyring_error();
        return Err(common::LocalError::KeyringError);
    }
    Ok(())
}

/// Helper function when accessing the error makes the code fairly unreadable.
/// Includes help for keyrings on Linux.
fn generic_keyring_error() {
    log::warn!("Failed to access the system keyring");
    #[cfg(target_os = "linux")]
    {
        log::warn!("Try installing the keyring implementation for your desktop");
        log::warn!("environment, like gnome-keyring or KDE wallet");
    }
}
