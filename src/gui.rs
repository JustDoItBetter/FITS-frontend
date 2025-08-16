// All things necessary for getting some graphical output
// SPDX-License-Identifier: GPL-3.0-only

use adw::glib;
use gtk::prelude::GtkWindowExt;

mod templates;

glib::wrapper! {
    pub struct InitialSetupWindow(ObjectSubclass<templates::InitialSetupWindow>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
    gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Native, gtk::Root,
    gtk::ShortcutManager;
}

impl InitialSetupWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}

/// Shows the setup dialog, prompting for the username and password.
///
/// TODO: Properly store the credentials in the system keyring
/// TODO: Check if the credentials are valid before saving them
pub fn build_setup_dialog(app: &adw::Application) {
    let window = InitialSetupWindow::new(app);
    window.present();
}

glib::wrapper! {
    pub struct FitsWindow(ObjectSubclass<templates::FitsWindow>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
    gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Native, gtk::Root,
    gtk::ShortcutManager;
}

impl FitsWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
