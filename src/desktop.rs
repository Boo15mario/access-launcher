use gtk4::glib;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub categories: Vec<String>,
    pub path: PathBuf,
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

pub fn normalize_lang_tag(lang: &str) -> String {
    lang.split(['.', '@']).next().unwrap_or("").to_string()
}

pub fn matches_lang_tag(tag: &str, lang: &str) -> bool {
    if tag.is_empty() || lang.is_empty() {
        return false;
    }
    let lang = normalize_lang_tag(lang);
    lang == tag || lang.starts_with(&format!("{tag}_")) || tag.starts_with(&lang)
}

pub fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes"
    )
}

pub fn parse_desktop_entry(
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

pub fn exec_looks_valid(exec: &str) -> bool {
    let exec = exec.trim();
    if exec.is_empty() {
        return false;
    }
    let argv = match glib::shell_parse_argv(exec) {
        Ok(argv) => argv,
        Err(_) => return true,
    };
    let Some(command) = argv.first().and_then(|arg| arg.to_str()) else {
        return true;
    };
    if command.starts_with('/') {
        Path::new(command).exists()
    } else {
        true
    }
}

pub fn collect_desktop_entries() -> Vec<DesktopEntry> {
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
    let mut entries_by_id: HashMap<String, DesktopEntry> = HashMap::new();

    for path in files {
        let id = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string());
        if let Some(id) = id {
            if let Some(entry) =
                parse_desktop_entry(&path, current_lang.as_deref(), current_desktops.as_deref())
            {
                let new_valid = exec_looks_valid(&entry.exec);
                match entries_by_id.get(&id) {
                    None => {
                        entries_by_id.insert(id, entry);
                    }
                    Some(existing) => {
                        let existing_valid = exec_looks_valid(&existing.exec);
                        if !existing_valid && new_valid {
                            entries_by_id.insert(id, entry);
                        }
                    }
                }
            }
        }
    }

    let mut entries: Vec<DesktopEntry> = entries_by_id.into_values().collect();
    entries.sort_by_key(|entry| entry.name.to_ascii_lowercase());
    entries
}

pub fn build_category_map(entries: &[DesktopEntry]) -> BTreeMap<String, Vec<DesktopEntry>> {
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
