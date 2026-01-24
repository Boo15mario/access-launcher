use access_launcher::desktop::{
    build_category_map, collect_desktop_entries, exec_looks_valid, matches_lang_tag,
    normalize_lang_tag, parse_bool, parse_desktop_entry, DesktopEntry,
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
Exec=/usr/bin/sample --flag
Categories=Utility;Development;
"#,
        "access-launcher-core",
    );
    let entry = parse_desktop_entry(&file.path, None, None).expect("entry present");
    assert_eq!(entry.name, "Sample App");
    assert_eq!(entry.exec, "/usr/bin/sample --flag");
    assert_eq!(
        entry.categories,
        vec!["Utility".to_string(), "Development".to_string()]
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
    let entry = parse_desktop_entry(&file.path, Some("en_US.UTF-8"), None).expect("entry present");
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
    assert!(parse_desktop_entry(&file.path, None, Some(&gnome)).is_some());
    assert!(parse_desktop_entry(&file.path, None, Some(&kde)).is_none());
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
    assert!(parse_desktop_entry(&file.path, None, Some(&kde)).is_some());
    assert!(parse_desktop_entry(&file.path, None, Some(&gnome)).is_none());
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
    let entry = parse_desktop_entry(&file.path, None, None).expect("entry present");
    let stem = file
        .path
        .file_stem()
        .and_then(|name| name.to_str())
        .expect("stem");
    assert_eq!(entry.name, stem);
    assert_eq!(entry.categories, vec!["Other".to_string()]);
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
fn build_category_map_groups_and_sorts_entries() {
    let entries = vec![
        DesktopEntry {
            name: "bApp".to_string(),
            exec: "app".to_string(),
            categories: vec!["Development".to_string()],
            path: PathBuf::from("/tmp/bapp.desktop"),
        },
        DesktopEntry {
            name: "Aapp".to_string(),
            exec: "app".to_string(),
            categories: vec!["Development".to_string()],
            path: PathBuf::from("/tmp/aapp.desktop"),
        },
        DesktopEntry {
            name: "GameApp".to_string(),
            exec: "app".to_string(),
            categories: vec!["Game".to_string()],
            path: PathBuf::from("/tmp/gameapp.desktop"),
        },
    ];
    let map = build_category_map(&entries);
    let dev_entries = map.get("Development").expect("development category");
    assert_eq!(dev_entries[0].name, "Aapp");
    assert_eq!(dev_entries[1].name, "bApp");
    assert!(map.contains_key("Games"));
}

fn create_desktop_file_for_override(dir: &std::path::PathBuf, name: &str, exec: &str) {
    fs::create_dir_all(dir.join("applications")).unwrap();
    let content = format!(
        r#"[Desktop Entry]
Type=Application
Name={}
Exec={}
Categories=Utility;
"#,
        name, exec
    );
    fs::write(dir.join(format!("applications/{}.desktop", name)), content).unwrap();
}

#[test]
fn test_override_logic() {
    let temp_home = env::temp_dir().join(format!("test-override-home-{}", std::process::id()));
    let temp_data = env::temp_dir().join(format!("test-override-data-{}", std::process::id()));

    if temp_home.exists() {
        fs::remove_dir_all(&temp_home).unwrap();
    }
    if temp_data.exists() {
        fs::remove_dir_all(&temp_data).unwrap();
    }

    env::set_var("XDG_DATA_HOME", &temp_home);
    env::set_var("XDG_DATA_DIRS", &temp_data);
    env::remove_var("NIX_PROFILES");

    // Case 1: User (Invalid) vs System (Valid) -> System should win
    create_desktop_file_for_override(&temp_home, "case1", "/invalid/path/exec");
    create_desktop_file_for_override(&temp_data, "case1", "/bin/true");

    // Case 2: User (Valid) vs System (Valid) -> User should win (Optimization skips System)
    // We can verify User wins by checking a property (e.g. Exec is different, but both valid)
    // /bin/echo is valid. /bin/ls is valid.
    create_desktop_file_for_override(&temp_home, "case2", "/bin/echo user");
    create_desktop_file_for_override(&temp_data, "case2", "/bin/ls system");

    // Case 3: User (Valid) vs System (Invalid) -> User should win
    create_desktop_file_for_override(&temp_home, "case3", "/bin/true");
    create_desktop_file_for_override(&temp_data, "case3", "/invalid/path/exec");

    // Case 4: User (Invalid) vs System (Invalid) -> User should win (First encountered)
    create_desktop_file_for_override(&temp_home, "case4", "/invalid/user");
    create_desktop_file_for_override(&temp_data, "case4", "/invalid/system");

    let entries = collect_desktop_entries();

    let get_exec = |name: &str| -> String {
        entries
            .iter()
            .find(|e| e.path.file_stem().unwrap().to_str().unwrap() == name)
            .unwrap()
            .exec
            .clone()
    };

    // Case 1: System wins
    assert_eq!(
        get_exec("case1"),
        "/bin/true",
        "Case 1: System (valid) should override User (invalid)"
    );

    // Case 2: User wins
    assert_eq!(
        get_exec("case2"),
        "/bin/echo user",
        "Case 2: User (valid) should be kept over System (valid)"
    );

    // Case 3: User wins
    assert_eq!(
        get_exec("case3"),
        "/bin/true",
        "Case 3: User (valid) should be kept over System (invalid)"
    );

    // Case 4: User wins (First one)
    assert_eq!(
        get_exec("case4"),
        "/invalid/user",
        "Case 4: User (invalid) should be kept over System (invalid)"
    );

    fs::remove_dir_all(&temp_home).unwrap();
    fs::remove_dir_all(&temp_data).unwrap();
}
