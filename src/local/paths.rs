// Getting paths is surprisingly difficult
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

/// Gets the complete path where the sqlite db SHOULD be
pub fn get_sqlite_path() -> PathBuf {
    match std::env::consts::OS {
        "linux" | "openbsd" | "netbsd" | "freebsd" => get_path_xdg(),
        "windows" => todo!("Windows support is coming soon tm"),
        "macos" => todo!("MacOS support is coming soon tm"),
        _ => todo!("Feel free to add support for your OS!"),
    }
}

fn get_path_xdg() -> PathBuf {
    let Ok(xdg_base): Result<String, ()> = std::env::var("XDG_DATA_HOME").or_else(|_| {
        let mut home = std::env::var("HOME").expect("Please set $HOME");
        home.push_str("/.local/share");
        Ok(home)
    }) else {
        // This is unreachable because if $XDG_DATA_HOME is unset we just get $HOME
        // and add something to the end and return that so we always return Ok
        unreachable!();
    };
    PathBuf::from(format!("{}/fck-ihk/data.sqlite", xdg_base))
}
