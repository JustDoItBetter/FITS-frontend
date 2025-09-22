//! Boilerplate for GTK resources. You do NOT want to be here.
// SPDX-License-Identifier: GPL-3.0-only

use crate::{common, local};
use adw::{glib, prelude::*, subclass::prelude::*};

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/noahjeana/fits/initial_setup.ui")]
pub struct InitialSetupWindow {
    #[template_child]
    pub server_addr: TemplateChild<adw::EntryRow>,
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
    type Type = super::widgets::InitialSetupWindow;
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

        // GtkWindowExt::close(&self);
        // Does not work because self is gui::templates::InitialSetupWindow and
        // IsA<gtk::Window> is only implemented for gui::InitialSetupWindow
        // YAY
        let obj = self.obj();
        obj.close();
    }
}

/// Main window
///
/// Holds the view of the application for writing
#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/noahjeana/fits/writer_window.ui")]
pub struct FitsWriterWindow {
    #[template_child]
    pub main_view: TemplateChild<adw::OverlaySplitView>,
    #[template_child]
    pub activities_source: TemplateChild<gtk::Box>,
    #[template_child]
    pub weekly_view: TemplateChild<adw::Bin>,
    // Needs to be an Cell because interior mutability
    //
    // # Safety
    // This cell MUST be initialized before the window is passed on because the
    // entire application assumes it to be.
    pub state: std::cell::Cell<Option<common::State>>,
}

#[glib::object_subclass]
impl ObjectSubclass for FitsWriterWindow {
    const NAME: &'static str = "FitsWriterWindow";
    type Type = super::widgets::FitsWriterWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for FitsWriterWindow {
    fn constructed(&self) {
        self.parent_constructed();
        // Load the possible activities and append them to the sidebar

        self.load_css();
    }
}
impl WidgetImpl for FitsWriterWindow {}
impl AdwApplicationWindowImpl for FitsWriterWindow {}
impl ApplicationWindowImpl for FitsWriterWindow {}
impl WindowImpl for FitsWriterWindow {}

impl FitsWriterWindow {
    fn load_css(&self) {
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_resource("/io/github/noahjeana/fits/style.css");

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/noahjeana/fits/weekly_view.ui")]
pub struct WeeklyView {
    #[template_child]
    pub monday_column: TemplateChild<gtk::Box>,
    #[template_child]
    pub tuesday_column: TemplateChild<gtk::Box>,
    #[template_child]
    pub wednesday_column: TemplateChild<gtk::Box>,
}

#[glib::object_subclass]
impl ObjectSubclass for WeeklyView {
    const NAME: &'static str = "WeeklyView";
    type Type = super::widgets::WeeklyView;
    type ParentType = adw::Bin;
}

impl WidgetImpl for WeeklyView {}
impl ObjectImpl for WeeklyView {}
impl BinImpl for WeeklyView {}

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/noahjeana/fits/activity.ui")]
pub struct Activity {
    #[template_child]
    pub label: TemplateChild<gtk::Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for Activity {
    const NAME: &'static str = "Activity";
    type Type = super::widgets::Activity;
    type ParentType = gtk::Widget;
}

impl WidgetImpl for Activity {}
impl ObjectImpl for Activity {}
