use access_launcher::desktop::{
    build_category_map, exec_looks_valid, matches_lang_tag, normalize_lang_tag, parse_bool,
    parse_desktop_entry, DesktopEntry,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(contents: &str, stem: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let mut path = env::temp_dir();
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        path.push(format!("{stem}-{pid}-{id}.desktop"));
        fs::write(&path, contents).expect("write temp desktop file");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn normalize_lang_tag_strips_variants() {
    assert_eq!(normalize_lang_tag("en_US.UTF-8"), "en_US");
    assert_eq!(normalize_lang_tag("fr@latin"), "fr");
    assert_eq!(normalize_lang_tag("de_DE"), "de_DE");
}

#[test]
fn matches_lang_tag_handles_prefixes() {
    assert!(matches_lang_tag("en", "en_US.UTF-8"));
    assert!(matches_lang_tag("en_US", "en"));
    assert!(!matches_lang_tag("", "en_US"));
    assert!(!matches_lang_tag("en_US", ""));
}

#[test]
fn parse_bool_accepts_common_true_values() {
    assert!(parse_bool("true"));
    assert!(parse_bool("1"));
    assert!(parse_bool("yes"));
    assert!(parse_bool("YeS"));
    assert!(!parse_bool("false"));
    assert!(!parse_bool("0"));
}

#[test]
fn parse_desktop_entry_reads_core_fields() {
    let file = TempFile::new(
        r#"
[Desktop Entry]
Type=Application
Name=Sample App
Exec=sample --flag
Categories=Utility;Development;
"#,
        "access-launcher-core",
    );
    let mut line_buf = String::new();
    let entry = parse_desktop_entry(&file.path, None, None, &mut line_buf).expect("entry present");
    assert_eq!(entry.name, "Sample App");
    assert_eq!(entry.exec, "sample --flag");
    assert_eq!(
        entry.categories,
        "Utility;Development;"
    );
}

#[test]
fn parse_desktop_entry_uses_localized_name() {
    let file = TempFile::new(
        r#"
[Desktop Entry]
Type=Application
Name=Default Name
Name[en_US]=Localized Name
Exec=app
"#,
        "access-launcher-localized",
    );
    let mut line_buf = String::new();
    let entry = parse_desktop_entry(&file.path, Some("en_US.UTF-8"), None, &mut line_buf).expect("entry present");
    assert_eq!(entry.name, "Localized Name");
}

#[test]
fn parse_desktop_entry_only_show_in_filters() {
    let file = TempFile::new(
        r#"
[Desktop Entry]
Type=Application
Name=Desktop Filter
Exec=app
OnlyShowIn=GNOME;
"#,
        "access-launcher-only-show-in",
    );
    let gnome = vec!["GNOME".to_string()];
    let kde = vec!["KDE".to_string()];
    let mut line_buf = String::new();
    assert!(parse_desktop_entry(&file.path, None, Some(&gnome), &mut line_buf).is_some());
    assert!(parse_desktop_entry(&file.path, None, Some(&kde), &mut line_buf).is_none());
}

#[test]
fn parse_desktop_entry_not_show_in_filters() {
    let file = TempFile::new(
        r#"
[Desktop Entry]
Type=Application
Name=Desktop Filter
Exec=app
NotShowIn=GNOME;
"#,
        "access-launcher-not-show-in",
    );
    let gnome = vec!["GNOME".to_string()];
    let kde = vec!["KDE".to_string()];
    let mut line_buf = String::new();
    assert!(parse_desktop_entry(&file.path, None, Some(&kde), &mut line_buf).is_some());
    assert!(parse_desktop_entry(&file.path, None, Some(&gnome), &mut line_buf).is_none());
}

#[test]
fn parse_desktop_entry_falls_back_to_filename_and_other_category() {
    let file = TempFile::new(
        r#"
[Desktop Entry]
Type=Application
Exec=app
"#,
        "access-launcher-fallback",
    );
    let mut line_buf = String::new();
    let entry = parse_desktop_entry(&file.path, None, None, &mut line_buf).expect("entry present");
    let stem = file
        .path
        .file_stem()
        .and_then(|name| name.to_str())
        .expect("stem");
    assert_eq!(entry.name, stem);
    // Categories should be empty string if missing from file
    assert_eq!(entry.categories, "");
}

#[test]
fn exec_looks_valid_handles_absolute_paths() {
    let temp = TempFile::new(
        r#"
[Desktop Entry]
Type=Application
Name=Exec Source
"#,
        "access-launcher-exec-path",
    );
    let existing = temp.path.to_string_lossy().to_string();
    assert!(exec_looks_valid(&existing));

    let mut missing = env::temp_dir();
    missing.push(format!(
        "access-launcher-missing-{}-{}",
        std::process::id(),
        99999
    ));
    let _ = fs::remove_file(&missing);
    let missing = missing.to_string_lossy().to_string();
    assert!(!exec_looks_valid(&missing));
    assert!(exec_looks_valid("relative-command"));
}

#[test]
fn exec_looks_valid_handles_complex_cases() {
    let temp = TempFile::new("", "access-launcher-quoted");
    let existing = temp.path.to_string_lossy().to_string();

    // Quoted absolute path (existing)
    let quoted_existing = format!("'{}'", existing);
    assert!(exec_looks_valid(&quoted_existing));

    // Quoted absolute path (missing)
    let quoted_missing = "'/non/existent/path'";
    assert!(!exec_looks_valid(quoted_missing));

    // Quoted relative path
    assert!(exec_looks_valid("'relative-command'"));

    // Double quotes
    let dquoted_existing = format!("\"{}\"", existing);
    assert!(exec_looks_valid(&dquoted_existing));

    // Complex args
    let complex = format!("{} --arg='val'", existing);
    assert!(exec_looks_valid(&complex));

    // Env with args
    assert!(exec_looks_valid("/usr/bin/env FOO=bar"));
}

#[test]
fn build_category_map_groups_entries_preserving_order() {
    let mut entries = vec![
        DesktopEntry {
            name: "bApp".to_string(),
            exec: "app".to_string(),
            categories: "Development".to_string(),
            path: PathBuf::from("/tmp/bapp.desktop"),
        },
        DesktopEntry {
            name: "Aapp".to_string(),
            exec: "app".to_string(),
            categories: "Development".to_string(),
            path: PathBuf::from("/tmp/aapp.desktop"),
        },
        DesktopEntry {
            name: "GameApp".to_string(),
            exec: "app".to_string(),
            categories: "Game".to_string(),
            path: PathBuf::from("/tmp/gameapp.desktop"),
        },
    ];
    // Pre-sort the entries to match how collect_desktop_entries works.
    entries.sort_by_cached_key(|entry| entry.name.to_ascii_lowercase());

    let map = build_category_map(&entries);
    let dev_indices = map.get("Development").expect("development category");
    assert_eq!(entries[dev_indices[0]].name, "Aapp");
    assert_eq!(entries[dev_indices[1]].name, "bApp");
    assert!(map.contains_key("Games"));
}
