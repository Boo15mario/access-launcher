use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use access_launcher::desktop::parse_desktop_entry;

fn main() {
    let tmp_dir = env::temp_dir();
    let file_path = tmp_dir.join("bench_entry.desktop");

    let content = r#"
[Desktop Entry]
Type=Application
Name=Benchmark App
Exec=true
Icon=utilities-terminal
Categories=System;TerminalEmulator;
OnlyShowIn=GNOME;Unity;
NotShowIn=KDE;
Comment=A benchmark desktop entry
"#;

    let mut file = File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let iterations = 100_000;
    let mut line_buf = String::with_capacity(1024);

    // Warmup
    for _ in 0..100 {
        let _ = parse_desktop_entry(&file_path, None, Some(&["GNOME".to_string()]), &mut line_buf);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = parse_desktop_entry(&file_path, None, Some(&["GNOME".to_string()]), &mut line_buf);
    }
    let duration = start.elapsed();

    println!("Parsed {} entries in {:?}", iterations, duration);
    println!("Time per entry: {:?}", duration / iterations as u32);

    std::fs::remove_file(file_path).unwrap();
}
