use access_launcher::desktop::collect_desktop_entries;
use std::env;
use std::fs;
use std::time::Instant;

#[test]
#[ignore]
fn bench_parsing_performance() {
    let temp_dir = env::temp_dir().join("bolt_bench_desktop");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(temp_dir.join("applications")).unwrap();

    let exec_path = env::current_exe().unwrap();
    let exec_str = exec_path.to_str().unwrap();

    println!("Generating 2000 desktop files...");
    // 1000 visible, 1000 hidden
    for i in 0..1000 {
        let content_visible = format!(
            "[Desktop Entry]\nType=Application\nName=App {}\nExec={}\nCategories=Utility;",
            i, exec_str
        );
        fs::write(
            temp_dir
                .join("applications")
                .join(format!("app-{}.desktop", i)),
            content_visible,
        )
        .unwrap();

        let content_hidden = format!(
            "[Desktop Entry]\nType=Application\nName=Hidden App {}\nExec={}\nNoDisplay=true\nCategories=Utility;",
            i, exec_str
        );
        fs::write(
            temp_dir
                .join("applications")
                .join(format!("hidden-{}.desktop", i)),
            content_hidden,
        )
        .unwrap();
    }

    env::set_var("XDG_DATA_HOME", &temp_dir);
    // Clear other env vars that might interfere
    env::remove_var("XDG_DATA_DIRS");

    // We also need to ensure desktop_dirs() picks up XDG_DATA_HOME correctly.
    // It checks XDG_DATA_HOME first.

    println!("Starting benchmark...");
    let start = Instant::now();
    let entries = collect_desktop_entries();
    let duration = start.elapsed();

    println!(
        "Parsed {} entries (1000 visible expected) in {:?}",
        entries.len(),
        duration
    );

    // Cleanup
    fs::remove_dir_all(&temp_dir).unwrap();
}
