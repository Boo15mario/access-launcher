use gtk4::glib;
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub categories: Vec<String>,
    pub path: PathBuf,
}

fn push_unique(dirs: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>, path: PathBuf) {
    if !seen.contains(&path) {
        seen.insert(path.clone());
        dirs.push(path);
    }
}

fn desktop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut seen = HashSet::new();

    let data_home = env::var("XDG_DATA_HOME")
        .ok()
        .and_then(|value| {
            if value.is_empty() {
                None
            } else {
                Some(PathBuf::from(value))
            }
        })
        .or_else(|| {
            env::var("HOME")
                .ok()
                .map(|home| PathBuf::from(home).join(".local/share"))
        });
    if let Some(data_home) = data_home {
        push_unique(&mut dirs, &mut seen, data_home.join("applications"));
        push_unique(
            &mut dirs,
            &mut seen,
            data_home.join("flatpak/exports/share/applications"),
        );
    }

    let mut added_xdg = false;
    if let Ok(data_dirs) = env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':').filter(|dir| !dir.is_empty()) {
            push_unique(
                &mut dirs,
                &mut seen,
                PathBuf::from(dir).join("applications"),
            );
            added_xdg = true;
        }
    }
    if !added_xdg {
        push_unique(
            &mut dirs,
            &mut seen,
            PathBuf::from("/usr/local/share/applications"),
        );
        push_unique(
            &mut dirs,
            &mut seen,
            PathBuf::from("/usr/share/applications"),
        );
    }

    push_unique(
        &mut dirs,
        &mut seen,
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    );

    // NixOS profiles are not always present in XDG_DATA_DIRS.
    push_unique(
        &mut dirs,
        &mut seen,
        PathBuf::from("/run/current-system/sw/share/applications"),
    );
    push_unique(
        &mut dirs,
        &mut seen,
        PathBuf::from("/nix/var/nix/profiles/default/share/applications"),
    );
    if let Ok(home) = env::var("HOME") {
        push_unique(
            &mut dirs,
            &mut seen,
            PathBuf::from(home).join(".nix-profile/share/applications"),
        );
    }
    if let Ok(user) = env::var("USER") {
        if !user.is_empty() {
            push_unique(
                &mut dirs,
                &mut seen,
                PathBuf::from(format!("/etc/profiles/per-user/{user}/share/applications")),
            );
        }
    }
    if let Ok(nix_profiles) = env::var("NIX_PROFILES") {
        for profile in nix_profiles.split_whitespace().filter(|p| !p.is_empty()) {
            push_unique(
                &mut dirs,
                &mut seen,
                PathBuf::from(profile).join("share/applications"),
            );
        }
    }
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
        } else if (file_type.is_file() || file_type.is_symlink())
            && path.extension().and_then(|ext| ext.to_str()) == Some("desktop")
        {
            files.push(path);
        }
    }
}

pub fn normalize_lang_tag(lang: &str) -> &str {
    lang.split(['.', '@']).next().unwrap_or("")
}

pub fn matches_lang_tag(tag: &str, lang: &str) -> bool {
    if tag.is_empty() || lang.is_empty() {
        return false;
    }
    let lang = normalize_lang_tag(lang);
    lang == tag
        || (lang.starts_with(tag) && lang.as_bytes().get(tag.len()) == Some(&b'_'))
        || tag.starts_with(lang)
}

pub fn parse_bool(value: &str) -> bool {
    let value = value.trim();
    value.eq_ignore_ascii_case("true") || value == "1" || value.eq_ignore_ascii_case("yes")
}

pub fn parse_desktop_entry(
    path: &Path,
    current_lang: Option<&str>,
    current_desktops: Option<&[String]>,
) -> Option<DesktopEntry> {
    let file = fs::File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut line_buf = String::new();

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

    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        let line = line_buf.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if in_entry {
                break;
            }
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

    // Optimization: avoid glib parse/allocation for common cases.
    // Most Exec lines are simple commands or absolute paths without quotes.
    if !exec.contains(['"', '\'', '\\']) {
        let command = exec.split_whitespace().next().unwrap_or("");
        if command.starts_with('/') {
            return Path::new(command).exists();
        } else {
            return true;
        }
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

fn cmp_ignore_ascii_case(a: &str, b: &str) -> std::cmp::Ordering {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let len = a_bytes.len().min(b_bytes.len());

    for i in 0..len {
        let c1 = a_bytes[i].to_ascii_lowercase();
        let c2 = b_bytes[i].to_ascii_lowercase();
        match c1.cmp(&c2) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    a_bytes.len().cmp(&b_bytes.len())
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

    let mut entries = Vec::new();
    let mut seen_ids = HashSet::new();

    for path in files {
        let id_str = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if id_str == "access-launcher.desktop" {
            continue;
        }

        if seen_ids.contains(id_str) {
            continue;
        }
        seen_ids.insert(id_str.to_string());

        if let Some(entry) =
            parse_desktop_entry(&path, current_lang.as_deref(), current_desktops.as_deref())
        {
            if exec_looks_valid(&entry.exec) {
                entries.push(entry);
            }
        }
    }

    entries.sort_by(|a, b| cmp_ignore_ascii_case(&a.name, &b.name));
    entries
}

pub fn build_category_map(entries: &[DesktopEntry]) -> BTreeMap<String, Vec<usize>> {
    let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, entry) in entries.iter().enumerate() {
        let bucket = map_categories(&entry.categories);
        map.entry(bucket.to_string()).or_default().push(i);
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
