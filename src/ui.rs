use gtk4::prelude::*;
use gtk4::{self as gtk, Orientation};
use std::collections::BTreeMap;

use crate::desktop::DesktopEntry;

fn set_uniform_margins<W: WidgetExt>(widget: &W, margin: i32) {
    widget.set_margin_top(margin);
    widget.set_margin_bottom(margin);
    widget.set_margin_start(margin);
    widget.set_margin_end(margin);
}

fn set_accessible_label<A: IsA<gtk::Accessible>>(widget: &A, label: &str) {
    widget.update_property(&[gtk::accessible::Property::Label(label)]);
}

fn set_accessible_description<A: IsA<gtk::Accessible>>(widget: &A, description: &str) {
    widget.update_property(&[gtk::accessible::Property::Description(description)]);
}

pub fn build_list_box(accessible_name: &str) -> gtk::ListBox {
    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.set_focusable(true);
    set_uniform_margins(&list_box, 6);
    set_accessible_label(&list_box, accessible_name);
    set_accessible_description(&list_box, "Use arrow keys to browse items.");
    list_box
}

pub fn append_text_row(list_box: &gtk::ListBox, label_text: &str, data_key: Option<&str>) {
    let row = gtk::ListBoxRow::new();
    let label = gtk::Label::new(Some(label_text));
    label.set_xalign(0.0);
    set_uniform_margins(&label, 6);
    row.set_child(Some(&label));
    set_accessible_label(&row, label_text);
    if let Some(key) = data_key {
        unsafe {
            row.set_data(key, label_text.to_string());
        }
    }
    list_box.append(&row);
}

fn append_program_row(list_box: &gtk::ListBox, entry: &DesktopEntry) {
    let row = gtk::ListBoxRow::new();
    let label = gtk::Label::new(Some(&entry.name));
    label.set_xalign(0.0);
    label.set_tooltip_text(Some(&entry.exec));
    set_uniform_margins(&label, 6);
    row.set_child(Some(&label));
    set_accessible_label(&row, &entry.name);
    set_accessible_description(&row, &entry.exec);
    unsafe {
        row.set_data("desktop-path", entry.path.to_string_lossy().to_string());
    }
    list_box.append(&row);
}

pub fn build_pane(title: &str, list_box: &gtk::ListBox) -> gtk::Box {
    let container = gtk::Box::new(Orientation::Vertical, 6);
    set_uniform_margins(&container, 12);

    let header = gtk::Label::new(Some(title));
    header.set_xalign(0.0);
    header.set_margin_bottom(6);

    let scroller = gtk::ScrolledWindow::new();
    scroller.set_hexpand(true);
    scroller.set_vexpand(true);
    scroller.set_child(Some(list_box));

    container.append(&header);
    container.append(&scroller);

    container
}

pub fn show_error_dialog(parent: &impl IsA<gtk::Window>, title: &str, details: &str) {
    let dialog = gtk::MessageDialog::builder()
        .message_type(gtk::MessageType::Error)
        .buttons(gtk::ButtonsType::Close)
        .text(title)
        .secondary_text(details)
        .build();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_destroy_with_parent(true);
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}

pub fn update_program_list(
    list_box: &gtk::ListBox,
    entries: &[DesktopEntry],
    category_map: &BTreeMap<String, Vec<usize>>,
    category: &str,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    let programs = category_map
        .get(category)
        .map(|items| items.as_slice())
        .unwrap_or(&[]);

    if programs.is_empty() {
        append_text_row(list_box, "No applications found", None);
        return;
    }

    for &index in programs {
        if let Some(entry) = entries.get(index) {
            append_program_row(list_box, entry);
        }
    }
}
