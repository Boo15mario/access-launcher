use gtk4::prelude::*;
use gtk4::{self as gtk, gio, Application, ApplicationWindow, Orientation};
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn set_uniform_margins<W: gtk::prelude::WidgetExt>(widget: &W, margin: i32) {
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

fn build_list_box(accessible_name: &str) -> gtk::ListBox {
    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.set_focusable(true);
    set_uniform_margins(&list_box, 6);
    set_accessible_label(&list_box, accessible_name);
    set_accessible_description(&list_box, "Use arrow keys to browse items.");
    list_box
}

fn append_text_row(list_box: &gtk::ListBox, label_text: &str, data_key: Option<&str>) {
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

fn build_pane(title: &str, list_box: &gtk::ListBox) -> gtk::Box {
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

#[derive(Clone, Debug)]
struct DesktopEntry {
    name: String,
    exec: String,
    categories: Vec<String>,
    path: PathBuf,
}

fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }
    dirs.push(PathBuf::from("/usr/local/share/applications"));
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs
}

fn walk_desktop_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            walk_desktop_files(&path, files);
        } else if file_type.is_file() || file_type.is_symlink() {
            if path.extension().and_then(|ext| ext.to_str()) == Some("desktop") {
                files.push(path);
            }
        }
    }
}

fn normalize_lang_tag(lang: &str) -> String {
    lang.split(['.', '@']).next().unwrap_or("").to_string()
}

fn matches_lang_tag(tag: &str, lang: &str) -> bool {
    if tag.is_empty() || lang.is_empty() {
        return false;
    }
    let lang = normalize_lang_tag(lang);
    lang == tag || lang.starts_with(&format!("{tag}_")) || tag.starts_with(&lang)
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes"
    )
}

fn parse_desktop_entry(
    path: &Path,
    current_lang: Option<&str>,
    current_desktops: Option<&[String]>,
) -> Option<DesktopEntry> {
    let contents = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut name: Option<String> = None;
    let mut localized_name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut categories: Vec<String> = Vec::new();
    let mut entry_type: Option<String> = None;
    let mut no_display = false;
    let mut hidden = false;
    let mut only_show_in: Option<Vec<String>> = None;
    let mut not_show_in: Option<Vec<String>> = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let (key, value) = match line.split_once('=') {
            Some(pair) => pair,
            None => continue,
        };
        let value = value.trim();
        if key == "Name" {
            name = Some(value.to_string());
        } else if let Some(tag) = key.strip_prefix("Name[").and_then(|k| k.strip_suffix(']')) {
            if let Some(lang) = current_lang {
                if matches_lang_tag(tag, lang) {
                    localized_name = Some(value.to_string());
                }
            }
        } else if key == "Exec" {
            exec = Some(value.to_string());
        } else if key == "Categories" {
            categories = value
                .split(';')
                .filter(|part| !part.is_empty())
                .map(|part| part.to_string())
                .collect();
        } else if key == "Type" {
            entry_type = Some(value.to_string());
        } else if key == "NoDisplay" {
            no_display = parse_bool(value);
        } else if key == "Hidden" {
            hidden = parse_bool(value);
        } else if key == "OnlyShowIn" {
            let values = value
                .split(';')
                .filter(|part| !part.is_empty())
                .map(|part| part.to_string())
                .collect::<Vec<_>>();
            only_show_in = Some(values);
        } else if key == "NotShowIn" {
            let values = value
                .split(';')
                .filter(|part| !part.is_empty())
                .map(|part| part.to_string())
                .collect::<Vec<_>>();
            not_show_in = Some(values);
        }
    }

    if entry_type.as_deref() != Some("Application") || no_display || hidden {
        return None;
    }

    if let Some(current_desktops) = current_desktops {
        if let Some(only) = &only_show_in {
            let matches = only
                .iter()
                .any(|item| current_desktops.iter().any(|c| c == item));
            if !matches {
                return None;
            }
        }
        if let Some(not) = &not_show_in {
            let matches = not
                .iter()
                .any(|item| current_desktops.iter().any(|c| c == item));
            if matches {
                return None;
            }
        }
    }

    let name = localized_name.or(name).or_else(|| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.to_string())
    })?;

    let exec = exec.unwrap_or_default();

    if categories.is_empty() {
        categories.push("Other".to_string());
    }

    Some(DesktopEntry {
        name,
        exec,
        categories,
        path: path.to_path_buf(),
    })
}

fn collect_desktop_entries() -> Vec<DesktopEntry> {
    let mut files = Vec::new();
    for dir in desktop_dirs() {
        walk_desktop_files(&dir, &mut files);
    }

    let current_lang = env::var("LANG").ok();
    let current_desktops = env::var("XDG_CURRENT_DESKTOP").ok().map(|value| {
        value
            .split(':')
            .filter(|entry| !entry.is_empty())
            .map(|entry| entry.to_string())
            .collect::<Vec<_>>()
    });
    let mut seen_ids = HashSet::new();
    let mut entries = Vec::new();

    for path in files {
        let id = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string());
        if let Some(id) = id {
            if seen_ids.contains(&id) {
                continue;
            }
            if let Some(entry) =
                parse_desktop_entry(&path, current_lang.as_deref(), current_desktops.as_deref())
            {
                seen_ids.insert(id);
                entries.push(entry);
            }
        }
    }

    entries.sort_by_key(|entry| entry.name.to_ascii_lowercase());
    entries
}

fn build_category_map(entries: &[DesktopEntry]) -> BTreeMap<String, Vec<DesktopEntry>> {
    let mut map: BTreeMap<String, Vec<DesktopEntry>> = BTreeMap::new();
    for entry in entries {
        let bucket = map_categories(&entry.categories);
        map.entry(bucket.to_string())
            .or_default()
            .push(entry.clone());
    }
    for programs in map.values_mut() {
        programs.sort_by_key(|entry| entry.name.to_ascii_lowercase());
    }
    map
}

fn map_categories(categories: &[String]) -> &'static str {
    let has = |needle: &str| categories.iter().any(|category| category == needle);

    if has("TerminalEmulator") || has("Terminal") {
        return "Terminal Emulator";
    }
    if has("Network") || has("WebBrowser") || has("Internet") {
        return "Internet";
    }
    if has("Game") || has("Games") {
        return "Games";
    }
    if has("Audio")
        || has("AudioVideo")
        || has("AudioVideoEditing")
        || has("Video")
        || has("VideoConference")
    {
        return "Audio/Video";
    }
    if has("Graphics") || has("Photography") {
        return "Graphics";
    }
    if has("Development") || has("IDE") || has("Programming") {
        return "Development";
    }
    if has("Accessory") || has("Accessories") {
        return "Accessories";
    }
    if has("TextEditor") || has("TextEditor") {
        return "Text Editors";
    }
    if has("Office") {
        return "Office";
    }
    if has("Utility") || has("Utilities") {
        return "Utilities";
    }
    if has("System") || has("Settings") {
        return "System";
    }
    "Other"
}

fn update_program_list(
    list_box: &gtk::ListBox,
    _entries: &[DesktopEntry],
    category_map: &BTreeMap<String, Vec<DesktopEntry>>,
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

    for entry in programs {
        append_program_row(list_box, entry);
    }
}

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

        programs_list.connect_row_activated(|_, row| {
            if let Some(path) = unsafe { row.data::<String>("desktop-path") } {
                let path = unsafe { path.as_ref() };
                if let Some(app_info) = gio::DesktopAppInfo::from_filename(path) {
                    let files: Vec<gio::File> = Vec::new();
                    if let Err(err) = app_info.launch(&files, None::<&gio::AppLaunchContext>) {
                        eprintln!("Failed to launch {path}: {err}");
                    }
                } else {
                    eprintln!("Failed to load desktop entry: {path}");
                }
            }
        });

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

        window.present();
    });

    app.run();
}
