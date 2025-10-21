// All things necessary for getting some graphical output
// SPDX-License-Identifier: GPL-3.0-only

use crate::{common, local};

use adw::prelude::*;

mod actions;
mod templates;
mod widgets;

pub fn run() {
    gtk::gio::resources_register_include!("compiled.gresources")
        .expect("Failed to register resources.");

    let app = adw::Application::builder()
        .application_id(super::APP_ID)
        .build();

    let res = common::block_on(local::load_data());
    // Check if everything is there, otherwise prompt for info
    let (state, config) = res.unwrap_or_else(|_| {
        app.connect_activate(build_setup_dialog);
        app.run();
        common::block_on(local::load_data()).expect("Setup failed. Try again?")
    });
    if config.is_student {
        // Need to construct a new app, see
        // https://github.com/JustDoItBetter/FITS-frontend/issues/19
        let app = adw::Application::builder()
            .application_id(super::APP_ID)
            .build();
        app.connect_activate(move |app| {
            build_writing_window(app, state.clone());
        });
    }
}

/// Shows the setup dialog, prompting for the username and password.
///
/// TODO: Check if the credentials are valid before saving them
pub fn build_setup_dialog(app: &adw::Application) {
    let window = widgets::InitialSetupWindow::new(app);
    window.present();
}

fn build_writing_window(app: &adw::Application, state: common::State) {
    let window = widgets::FitsWriterWindow::new(app);
    window.set_state(state);
    actions::register_writer_actions(app, window.clone());
    window.present();
}
