//! Boilerplate for GTK resources. You do NOT want to be here.
// SPDX-License-Identifier: GPL-3.0-only

use crate::{common, local};
use adw::{glib, prelude::*, subclass::prelude::*};
use chrono::Datelike;

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/justdoitbetter/fits/initial_setup.ui")]
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

        if local::db::create_db().is_err() {
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
#[template(resource = "/io/github/justdoitbetter/fits/writer_window.ui")]
pub struct FitsWriterWindow {
    #[template_child]
    pub main_view: TemplateChild<adw::OverlaySplitView>,
    #[template_child]
    pub activities_source: TemplateChild<gtk::ListBox>,
    // Why do I have to use the type that is NOT in this module?
    #[template_child]
    pub weekly_view: TemplateChild<super::widgets::WeeklyView>,
    #[template_child]
    pub add_activity_row: TemplateChild<adw::EntryRow>,
    #[template_child]
    pub toast_overlay: TemplateChild<adw::ToastOverlay>,
    /// Holds the activities loaded from the database
    pub activities: std::cell::RefCell<Option<gtk::gio::ListStore>>,
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
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for FitsWriterWindow {
    fn constructed(&self) {
        self.parent_constructed();
        self.load_state();
        self.setup_activities();
        self.setup_days();
        // Load the possible activities and append them to the sidebar
    }
}

impl WidgetImpl for FitsWriterWindow {}
impl AdwApplicationWindowImpl for FitsWriterWindow {}
impl ApplicationWindowImpl for FitsWriterWindow {}
impl WindowImpl for FitsWriterWindow {}

#[gtk::template_callbacks]
impl FitsWriterWindow {
    pub fn get_db_connector(&self) -> Option<local::db::DbConnector> {
        if let Some(state) = self.state.take() {
            let db = state.get_db_connector();
            self.state.set(Some(state));
            Some(db)
        } else {
            log::warn!("Could not get state to get database connector");
            None
        }
    }

    #[template_callback]
    pub fn add_activity(&self) {
        let activity_name = self
            .add_activity_row
            .get()
            .text()
            .to_string()
            .trim()
            .to_string();
        if activity_name.is_empty() {
            let toast = adw::Toast::builder()
                .title("Activity name cannot be empty!")
                .build();
            self.toast_overlay.get().add_toast(toast);
            return;
        }
        let activity_object = super::widgets::ActivityObject::new(&activity_name);

        if let Some(db) = self.get_db_connector() {
            if let Err(e) = db.add_activity(&activity_name) {
                let toast = adw::Toast::builder()
                    .title("Saving activity failed, see log for details")
                    .build();
                self.toast_overlay.get().add_toast(toast);
                log::error!("Failed to add activity to database: {:#?}", e);
                return;
            }
        } else {
            log::warn!("Could not get state to add activity to database");
            return;
        }

        self.activities
            .borrow()
            .as_ref()
            .expect("Activities ListStore not initialized")
            .append(&activity_object);
    }

    fn create_activity_row(&self, activity: &super::widgets::ActivityObject) -> adw::ActionRow {
        let delete_button = create_delete_button();
        delete_button.connect_clicked(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[weak]
            activity,
            move |_| {
                window.remove_activity(&activity);
            }
        ));

        let row = adw::ActionRow::builder()
            .css_classes(vec!["rounded"])
            .build();

        row.add_suffix(&delete_button);
        activity
            .bind_property("name", &row, "title")
            .sync_create()
            .build();
        row
    }

    fn create_daily_activity_row(
        &self,
        list_store: &gtk::gio::ListStore,
        db: local::db::DbConnector,
        activity: &super::widgets::ActivityObject,
    ) -> adw::ActionRow {
        let delete_button = create_delete_button();

        let week = {
            if let Some(res) = self.weekly_view.imp().week.get() {
                res.to_owned()
            } else {
                log::warn!("Could not get week number to create daily activity row");
                return adw::ActionRow::builder()
                    .css_classes(vec!["rounded"])
                    .build();
            }
        };
        let year = {
            if let Some(res) = self.weekly_view.imp().year.get() {
                res.to_owned()
            } else {
                log::warn!("Could not get year number to create daily activity row");
                return adw::ActionRow::builder()
                    .css_classes(vec!["rounded"])
                    .build();
            }
        };

        delete_button.connect_clicked(glib::clone!(
            #[weak]
            activity,
            #[weak]
            list_store,
            move |button| {
                button.set_sensitive(false);
                let index = list_store
                    .find(&activity)
                    .expect("Activity not found in daily activities ListStore");
                db.remove_daily_activity(&activity.get_name(), index as i64, year, week)
            }
        ));

        let row = adw::ActionRow::builder()
            .css_classes(vec!["rounded"])
            .build();
        row.add_suffix(&delete_button);
        row
    }

    fn setup_activities(&self) {
        use adw::glib::prelude::*;

        let store = gtk::gio::ListStore::new::<super::widgets::ActivityObject>();
        self.activities.replace(Some(store));

        self.activities_source.bind_model(
            Some(
                self.activities
                    .borrow()
                    .as_ref()
                    .expect("Activities ListStore not initialized"),
            ),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[upgrade_or_panic]
                move |item| {
                    let activity = item
                        .downcast_ref::<super::widgets::ActivityObject>()
                        .expect("Item is not an ActivityObject");
                    let row = window.create_activity_row(activity);
                    row.upcast()
                }
            ),
        );

        let Some(db) = self.get_db_connector() else {
            log::warn!("Could not get state to load activities from database");
            return;
        };
        match db.get_activities() {
            Ok(activities) => {
                for activity in activities {
                    let activity_object = super::widgets::ActivityObject::new(&activity);
                    self.activities
                        .borrow()
                        .as_ref()
                        .expect("Activities ListStore not initialized")
                        .append(&activity_object);
                }
            }
            Err(e) => {
                log::error!("Failed to load activities from database: {:#?}", e);
            }
        }
    }

    fn create_day_view(&self, day_name: &str, activities: &[String]) -> gtk::Widget {
        let day = super::widgets::Day::new();
        day.imp().title_label.get().set_text(day_name);

        let store = gtk::gio::ListStore::new::<super::widgets::ActivityObject>();
        let list_box = &day.imp().activities_listbox.get();

        let Some(db) = self.get_db_connector() else {
            log::warn!("Could not get state to create day view");
            return adw::StatusPage::builder()
                .title("Could not load database")
                .description("Have you tried turning it off and on again?")
                .css_classes(["compact"])
                .icon_name("edit-delete-symbolic")
                .build()
                .upcast();
        };

        list_box.bind_model(
            Some(&store),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[weak]
                store,
                #[upgrade_or_panic]
                move |item| {
                    let activity = item
                        .downcast_ref::<super::widgets::ActivityObject>()
                        .expect("Item is not an ActivityObject");
                    let row = window.create_daily_activity_row(&store, db.clone(), activity);
                    row.upcast()
                }
            ),
        );

        /*
        store.connect_items_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |store, _, _, _| {
                let items = store
                    .iter()
                    .flatten()
                    .map(|activity: super::widgets::ActivityObject| {
                        let activity = activity.imp();
                    })
                    .collect();
            }
        ));
        */

        for activity in activities {
            let activity_object = super::widgets::ActivityObject::new(activity);
            store.append(&activity_object);
        }
        day.imp().activities_store.replace(Some(store.clone()));

        day.upcast()
    }

    fn setup_days(&self) {
        let Some(state) = self.state.take() else {
            log::warn!("Could not get state");
            return;
        };
        let db = state.get_db_connector();
        self.state.set(Some(state));

        let now = chrono::Utc::now().iso_week();
        let time = chrono::NaiveDate::from_isoywd_opt(now.year(), now.week(), chrono::Weekday::Mon)
            .expect("Could not get date for current week")
            .and_hms_opt(0, 0, 0)
            .expect("Could not get time for current week")
            .and_utc()
            .timestamp();

        let days = match db.get_weeks(time..time + 1) {
            Ok(weeks) if !weeks.is_empty() => {
                if weeks.len() > 1 {
                    log::warn!(
                        "Multiple weeks found in the database for the same time period, using the first one"
                    );
                }
                weeks[0].get_days()
            }
            Ok(_) => {
                log::info!("No week data found in the database, showing default weekdays.");
                super::setup::get_weekdays()
                    .into_iter()
                    .map(|day| (day, vec![]))
                    .collect()
            }
            Err(_) => {
                log::warn!("Could not get week data from database, showing default weekdays.");
                super::setup::get_weekdays()
                    .into_iter()
                    .map(|day| (day.to_string(), vec![]))
                    .collect()
            }
        };

        let mut sorted_days: Vec<_> = days.into_iter().collect();
        sorted_days.sort_by(|a, b| {
            local::dates::day_sort_key(&a.0).cmp(&local::dates::day_sort_key(&b.0))
        });

        let weekly_view = super::widgets::WeeklyView::new_with_data(
            None,
            now.year(),
            now.week(),
            sorted_days
                .iter()
                .map(|(day, activities)| self.create_day_view(day, activities))
                .collect(),
        );

        self.weekly_view.set_child(Some(&weekly_view));
    }

    /// Loads the state from disk
    fn load_state(&self) {
        let obj = self.obj();
        match common::block_on(local::load_state()) {
            Ok(state) => {
                obj.set_state(state);
            }
            Err(e) => {
                log::error!("Failed to load state from disk: {:#?}", e);
            }
        }
    }

    fn remove_activity(&self, activity: &super::widgets::ActivityObject) {
        log::debug!("Removing activity");
        let activities = self.activities.borrow();
        let Some(store) = activities.as_ref() else {
            log::warn!("Activities ListStore not initialized");
            return;
        };
        let Some(index) = store.find(activity) else {
            log::warn!("Activity '{}' not found in ListStore", activity.get_name());
            return;
        };
        store.remove(index);

        if let Some(db) = self.get_db_connector() {
            if let Err(e) = db.remove_activity(&activity.get_name()) {
                let toast = adw::Toast::builder()
                    .title("Removing activity failed, see log for details")
                    .build();
                self.toast_overlay.get().add_toast(toast);
                log::error!("Failed to remove activity from database: {:#?}", e);
            }
        } else {
            log::warn!("Could not get state to remove activity from database");
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActivityData {
    pub name: String,
}

impl ActivityData {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(glib::Properties, Default)]
#[properties(wrapper_type = super::widgets::ActivityObject)]
pub struct ActivityObject {
    #[property(name = "name", get, set, type = String, member = name)]
    pub data: std::cell::RefCell<ActivityData>,
}

#[glib::object_subclass]
impl ObjectSubclass for ActivityObject {
    const NAME: &'static str = "FitsActivityObject";
    type Type = super::widgets::ActivityObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for ActivityObject {}

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/justdoitbetter/fits/day.ui")]
pub struct Day {
    #[template_child]
    pub title_label: gtk::TemplateChild<gtk::Label>,
    /// TODO: Change to gtk::ListView
    #[template_child]
    pub activities_listbox: gtk::TemplateChild<gtk::ListBox>,
    pub activities_store: std::cell::RefCell<Option<gtk::gio::ListStore>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Day {
    const NAME: &'static str = "FitsDay";
    type Type = super::widgets::Day;
    type ParentType = adw::Bin;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Day {
    fn constructed(&self) {
        self.parent_constructed();
    }
}
impl WidgetImpl for Day {}
impl BinImpl for Day {}

/// Helper to avoid duplicate code
fn create_delete_button() -> gtk::Button {
    gtk::Button::builder()
        .icon_name("user-trash-symbolic")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(vec!["destructive-action"])
        .build()
}

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/io/github/justdoitbetter/fits/weekly_view.ui")]
pub struct WeeklyView {
    #[template_child]
    pub title_label: gtk::TemplateChild<gtk::Label>,
    #[template_child]
    pub day_list: gtk::TemplateChild<gtk::ListView>,
    day_store: std::cell::RefCell<Option<gtk::gio::ListStore>>,
    pub year: std::cell::OnceCell<i32>,
    pub week: std::cell::OnceCell<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for WeeklyView {
    const NAME: &'static str = "FitsWeeklyView";
    type Type = super::widgets::WeeklyView;
    type ParentType = adw::Bin;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for WeeklyView {
    fn constructed(&self) {
        self.parent_constructed();
        let store = gtk::gio::ListStore::new::<super::widgets::Day>();
    }
}
impl WidgetImpl for WeeklyView {}
impl BinImpl for WeeklyView {}

impl WeeklyView {
    pub fn get_day_store(&self) -> Option<gtk::gio::ListStore> {
        self.day_store.borrow().as_ref().cloned()
    }

    pub fn set_day_store(&self, store: gtk::gio::ListStore) {
        self.day_store.borrow_mut().replace(store);
    }

    pub fn add_day(&self, day: &super::widgets::Day) {
        if let Some(store) = self.get_day_store() {
            store.append(day);
        }
    }

    pub fn setup_day_list(&self) {
        let model = gtk::gio::ListStore::new::<super::widgets::Day>();
    }
}
