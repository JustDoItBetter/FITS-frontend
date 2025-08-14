// Boilerplate for GTK resources. You do NOT want to be here.
// SPDX-License-Identifier: GPL-3.0-only

use crate::local;
use adw::{glib, subclass::prelude::*};
use gtk::prelude::EditableExt;

// Boilerplate to get the settings from the blueprint into a GObject into Rust.
#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/noahjeana/fits/initial_setup.ui")]
pub struct InitialSetupWindow {
    #[template_child]
    pub username_entry: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub password_entry: TemplateChild<adw::PasswordEntryRow>,
    #[template_child]
    pub toast_overlay: TemplateChild<adw::ToastOverlay>,
}

#[glib::object_subclass]
impl ObjectSubclass for InitialSetupWindow {
    const NAME: &'static str = "InitialSetupWindow";
    type Type = super::InitialSetupWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for InitialSetupWindow {}
impl WidgetImpl for InitialSetupWindow {}
impl AdwApplicationWindowImpl for InitialSetupWindow {}
impl ApplicationWindowImpl for InitialSetupWindow {}
impl WindowImpl for InitialSetupWindow {}

#[gtk::template_callbacks]
impl InitialSetupWindow {
    #[template_callback]
    fn check_signin(&self) {
        let username = self.username_entry.get().text().to_string();
        let password = self.password_entry.get().text().to_string();

        if local::keyring::save_credentials(&username, &password).is_err() {
            let toast = adw::Toast::builder()
                .title("Failed to save credentials!")
                .build();
            self.toast_overlay.get().add_toast(toast);
        }

        if local::sqlite::create_db().is_err() {
            let toast = adw::Toast::builder()
                .title("Failed to create persistent storage!")
                .build();
            self.toast_overlay.get().add_toast(toast);
        }
    }
}

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/noahjeana/fits/main_window.ui")]
pub struct FitsWindow {
    #[template_child]
    pub main_view: TemplateChild<adw::ViewStack>,
    #[template_child]
    pub top_switcher: TemplateChild<adw::ViewSwitcher>,
    #[template_child]
    pub bottom_switcher: TemplateChild<adw::ViewSwitcherBar>,
}

#[glib::object_subclass]
impl ObjectSubclass for FitsWindow {
    const NAME: &'static str = "FitsWindow";
    type Type = super::FitsWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for FitsWindow {}
impl WidgetImpl for FitsWindow {}
impl AdwApplicationWindowImpl for FitsWindow {}
impl ApplicationWindowImpl for FitsWindow {}
impl WindowImpl for FitsWindow {}
