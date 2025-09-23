//! Register actions for the application
// SPDX-License-Identifier: GPL-3.0-only

use adw::prelude::*;

/// Register all actions for [FitsWriterWindow]
pub fn register_writer_actions(app: &adw::Application, window: super::widgets::FitsWriterWindow) {
    register_about_dialog(app, window);
}

fn register_about_dialog(app: &adw::Application, window: super::widgets::FitsWriterWindow) {
    let about_dialog: adw::AboutDialog =
        gtk::Builder::from_resource("/io/github/justdoitbetter/fits/premade.ui")
            .object("about_dialog")
            .expect("Spelling is difficult");
    let about_action = gtk::gio::SimpleAction::new("about", None);

    about_action.connect_activate(move |_action, _| {
        about_dialog.present(Some(&window));
    });

    app.add_action(&about_action);
}
