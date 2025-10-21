// Getting paths is surprisingly difficult
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

/// Gets the complete path where the db SHOULD be
///
/// It also checks if the folder exist and creates it if neccessary
pub fn get_db_path() -> PathBuf {
    let mut path = match std::env::consts::OS {
        "linux" | "openbsd" | "netbsd" | "freebsd" => get_xdg_data(),
        "windows" => todo!("Windows support is coming soon tm"),
        "macos" => todo!("MacOS support is coming soon tm"),
        _ => todo!("Feel free to add support for your OS!"),
    };
    path.push("fits/");
    if !path.exists() {
        // There is not really something we can do if this fails because if we
        // cannot create this the user already has a VERY broken system
        let _ = std::fs::create_dir_all(&path);
    }
    path.push("data.sqlite");
    path
}

fn get_xdg_data() -> PathBuf {
    let Ok(xdg_base): Result<String, ()> = std::env::var("XDG_DATA_HOME").or_else(|_| {
        let mut home = std::env::var("HOME").expect("Please set $HOME");
        home.push_str("/.local/share");
        Ok(home)
    }) else {
        // This is unreachable because if $XDG_DATA_HOME is unset we just get $HOME
        // and add something to the end and return that so we always return Ok
        unreachable!();
    };
    PathBuf::from(xdg_base)
}

pub fn get_config_path() -> PathBuf {
    let mut path = match std::env::consts::OS {
        "linux" | "openbsd" | "netbsd" | "freebsd" => get_xdg_config(),
        "windows" => todo!("Windows support is coming soon tm"),
        "macos" => todo!("MacOS support is coming soon tm"),
        _ => todo!("Feel free to add support for your OS!"),
    };
    path.push("fits/");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.push("config.toml");
    path
}

fn get_xdg_config() -> PathBuf {
    let Ok(xdg_config): Result<String, ()> = std::env::var("XDG_CONFIG_HOME").or_else(|_| {
        let mut home = std::env::var("HOME").expect("Please set $HOME");
        home.push_str("/.config");
        Ok(home)
    }) else {
        // This is unreachable because if $XDG_CONFIG_HOME is unset we just get $HOME
        // and add something to the end and return that so we always return Ok
        unreachable!();
    };
    PathBuf::from(xdg_config)
}
