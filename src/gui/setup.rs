//! Set things up so the GUI works
// SPDX-License-Identifier: GPL-3.0-only

use adw::prelude::*;

pub fn create_drop_zone(title: &str) -> gtk::Box {
    let drop_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
    drop_box.set_size_request(100, 300);

    let label = gtk::Label::new(Some(title));
    label.add_css_class("heading");
    drop_box.append(&label);

    let drop_target = gtk::DropTarget::new(gtk::glib::Type::STRING, gtk::gdk::DragAction::COPY);

    let drop_box_clone = drop_box.clone();
    drop_target.connect_enter(move |_target, _x, _y| {
        drop_box_clone.add_css_class("suggested-action");
        gtk::gdk::DragAction::COPY
    });

    let drop_box_clone = drop_box.clone();
    drop_target.connect_leave(move |_target| {
        drop_box_clone.remove_css_class("suggested-action");
    });

    let drop_box_clone = drop_box.clone();
    drop_target.connect_drop(move |_target, value, _x, _y| {
        drop_box_clone.remove_css_class("suggested-action");

        if let Ok(text) = value.get::<String>() {
            for line in text.lines() {
                let dropped_label = gtk::Label::new(Some(line));
                drop_box_clone.append(&dropped_label);
            }

            true
        } else {
            false
        }
    });

    drop_box.add_controller(drop_target);
    drop_box
}

pub fn create_day(name: &str) -> super::widgets::Day {
    let res = super::widgets::Day::new();
    res.set_day(name);
    res
}

/// TODO: Replace so we query the locale instead. Note that this will require rework
///   of the db schema because we rely on weekdays being saved as their english
///   name.
pub fn get_weekdays() -> Vec<String> {
    vec![
        "Monday".to_string(),
        "Tuesday".to_string(),
        "Wednesday".to_string(),
        "Thursday".to_string(),
        "Friday".to_string(),
        "Saturday".to_string(),
        "Sunday".to_string(),
    ]
}
