// Description and things
// Copyright (C) 2025 Bjarne Seger
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by the
// Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-only

use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

pub mod gui;
pub mod local;

const APP_ID: &str = "io.github.NoahJeanA.fck_ihk";

fn main() {
    gtk::gio::resources_register_include!("compiled.gresources")
        .expect("Failed to register resources.");

    let app = adw::Application::builder().application_id(APP_ID).build();
    if local::check_first_run() {
        app.connect_activate(gui::build_setup_dialog);
        app.run();
    }
}
