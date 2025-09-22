//! The actual widgets created from the templates
// SPDX-License-Identifier: GPL-3.0-only

// Still boilerplate ahead

use crate::common;

use super::templates;
use adw::{glib, subclass::prelude::ObjectSubclassIsExt};

glib::wrapper! {
    /// The window for writing (or for now just dragging and dropping)
    ///
    /// The window contains the main view for a week as well as the possible
    /// activities to add to it in the sidebar.
    pub struct FitsWriterWindow(ObjectSubclass<templates::FitsWriterWindow>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
    gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Native, gtk::Root,
    gtk::ShortcutManager;
}

impl FitsWriterWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn set_state(&self, state: common::State) {
        self.imp().state.replace(Some(state));
    }

    /// Get the state from the cell
    ///
    /// # IMPORTANT
    /// Because we get the value from the cell, after calling this, the cell is
    /// **EMPTY** and you **MUST** put the state back where it belongs after you are
    /// done through [set_state]
    pub fn get_state(&self) -> Option<common::State> {
        self.imp().state.take()
    }
}

glib::wrapper! {
    pub struct WeeklyView(ObjectSubclass<templates::WeeklyView>)
    @extends adw::Bin, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
    gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Native, gtk::Root,
    gtk::ShortcutManager;
}

glib::wrapper! {
    pub struct Activity(ObjectSubclass<templates::Activity>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

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
