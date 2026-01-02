use access_launcher::desktop::{build_category_map, collect_desktop_entries};
use access_launcher::ui::{
    append_text_row, build_list_box, build_pane, show_error_dialog, update_program_list,
};
use gtk4::prelude::*;
use gtk4::{self as gtk, gio, Application, ApplicationWindow, Orientation};
use std::rc::Rc;

fn main() {
    let app = Application::builder()
        .application_id("com.example.AccessLauncher")
        .build();

    app.connect_activate(|app| {
        let entries = Rc::new(collect_desktop_entries());
        let category_map = Rc::new(build_category_map(&entries));
        let categories = [
            "Accessories",
            "Audio/Video",
            "Development",
            "Games",
            "Graphics",
            "Text Editors",
            "Internet",
            "Office",
            "System",
            "Terminal Emulator",
            "Utilities",
            "Other",
        ];

        let categories_list = build_list_box("Categories list");
        for category in categories {
            append_text_row(&categories_list, category, Some("category"));
        }

        let programs_list = build_list_box("Programs list");
        update_program_list(&programs_list, &entries, &category_map, "Internet");

        {
            let entries = Rc::clone(&entries);
            let category_map = Rc::clone(&category_map);
            let programs_list = programs_list.clone();
            categories_list.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    if let Some(category) = unsafe { row.data::<String>("category") } {
                        let category = unsafe { category.as_ref() };
                        update_program_list(&programs_list, &entries, &category_map, category);
                    }
                }
            });
        }

        categories_list.select_row(categories_list.row_at_index(0).as_ref());

        let left_pane = build_pane("Categories", &categories_list);
        let right_pane = build_pane("Programs", &programs_list);

        let paned = gtk::Paned::new(Orientation::Horizontal);
        paned.set_start_child(Some(&left_pane));
        paned.set_end_child(Some(&right_pane));
        paned.set_resize_start_child(true);
        paned.set_resize_end_child(true);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);
        paned.set_wide_handle(true);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Access Launcher")
            .default_width(900)
            .default_height(600)
            .child(&paned)
            .build();

        let window_for_dialog = window.clone();
        programs_list.connect_row_activated(move |_, row| {
            if let Some(path) = unsafe { row.data::<String>("desktop-path") } {
                let path = unsafe { path.as_ref() };
                if let Some(app_info) = gio::DesktopAppInfo::from_filename(path) {
                    let files: Vec<gio::File> = Vec::new();
                    let launch_context =
                        gtk::prelude::WidgetExt::display(&window_for_dialog).app_launch_context();
                    if let Err(err) = app_info.launch(&files, Some(&launch_context)) {
                        eprintln!("Failed to launch {path}: {err}");
                        let app_name = app_info.name();
                        show_error_dialog(
                            &window_for_dialog,
                            &format!("Failed to launch {app_name}"),
                            err.message(),
                        );
                    }
                } else {
                    eprintln!("Failed to load desktop entry: {path}");
                    show_error_dialog(
                        &window_for_dialog,
                        "Failed to load application",
                        &format!("Could not read desktop entry at {path}"),
                    );
                }
            }
        });

        window.present();
    });

    app.run();
}
