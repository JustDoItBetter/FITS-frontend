//! The actual widgets created from the templates
// SPDX-License-Identifier: GPL-3.0-only

// Still boilerplate ahead

use crate::common;
use adw::prelude::*;

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

    pub fn add_day(&self, day: gtk::Box) {
        if let Some(res) = self.imp().weekly_view.get().child() {
            res.downcast::<gtk::Box>()
                .expect("Weekly view child is not a gtk::Box")
                .append(&day);
        } else {
            log::error!("Weekly view has no child to add day to");
        }
    }

    pub fn set_weekly_view(&self, weekly_view: gtk::Box) {
        self.imp().weekly_view.get().set_child(Some(&weekly_view));
    }
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

glib::wrapper! {
    pub struct ActivityObject(ObjectSubclass<templates::ActivityObject>);
}

impl ActivityObject {
    pub fn new(name: &str) -> Self {
        glib::Object::builder().property("name", name).build()
    }

    pub fn get_name(&self) -> String {
        self.property("name")
    }
}

glib::wrapper! {
    pub struct Day(ObjectSubclass<templates::Day>)
    @extends adw::Bin, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Day {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_day(&self, day: &str) {
        self.imp().title_label.set_label(day);
    }
}

impl Default for Day {
    fn default() -> Self {
        Self::new()
    }
}

glib::wrapper! {
    pub struct WeeklyView(ObjectSubclass<templates::WeeklyView>)
    @extends adw::Bin, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl WeeklyView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn new_with_data(
        title: Option<&str>,
        year: i32,
        week: u32,
        days: Vec<gtk::Widget>,
    ) -> Self {
        let weekly_view = Self::new();
        // We just created to WeeklyView, so it can only be uninitialized
        let _ = weekly_view.imp().year.set(year);
        let _ = weekly_view.imp().week.set(week);

        if let Some(title) = title {
            weekly_view.imp().title_label.set_label(title);
        } else {
            weekly_view
                .imp()
                .title_label
                .set_label(&format!("Week {} of {}", week, year));
        }
        let container = weekly_view.imp().day_box.get();
        for day in days {
            container.append(&day);
            let seperator = gtk::Separator::new(gtk::Orientation::Horizontal);
            container.append(&seperator);
        }

        weekly_view
    }
}

impl Default for WeeklyView {
    fn default() -> Self {
        Self::new()
    }
}
