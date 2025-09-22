// All things necessary for getting some graphical output
// SPDX-License-Identifier: GPL-3.0-only

use crate::local;

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

    // Check if everything is there, otherwise prompt for info
    let (_, config) = local::load_data().unwrap_or_else(|_| {
        app.connect_activate(build_setup_dialog);
        app.run();
        local::load_data().expect("Setup failed. Try again?")
    });

    if config.is_student {
        app.connect_activate(build_writing_window);
        app.run();
    } else {
        todo!();
    }
}

/// Shows the setup dialog, prompting for the username and password.
///
/// TODO: Check if the credentials are valid before saving them
pub fn build_setup_dialog(app: &adw::Application) {
    let window = widgets::InitialSetupWindow::new(app);
    window.present();
}

fn build_writing_window(app: &adw::Application) {
    let window = widgets::FitsWriterWindow::new(app);
    let state = local::load_state()
        .expect("Loading the config failed. This should have been caught earlier.");
    window.set_state(state);
    actions::register_writer_actions(app, window.clone());
    window.present();
}
