use access_launcher::desktop::collect_desktop_entries;
use std::env;
use std::fs;
use std::time::Instant;

#[test]
fn bench_parsing_performance_fast() {
    let temp_dir = env::temp_dir().join("bolt_bench_desktop2");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(temp_dir.join("applications")).unwrap();

    let exec_path = env::current_exe().unwrap();
    let exec_str = exec_path.to_str().unwrap();

    println!("Generating 2000 desktop files...");
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
    env::remove_var("XDG_DATA_DIRS");

    println!("Starting benchmark...");
    let mut best = std::time::Duration::from_secs(100);
    for _ in 0..10 {
        let start = Instant::now();
        let entries = collect_desktop_entries();
        let duration = start.elapsed();
        if duration < best {
            best = duration;
        }
    }

    println!("Best 2000 parse time: {:?}", best);
    fs::remove_dir_all(&temp_dir).unwrap();
}
