use std::io::BufRead;
fn foo(buf: &mut String) {
    let cursor = std::io::Cursor::new(b"hello\nworld");
    let mut reader = std::io::BufReader::new(cursor);
    loop {
        buf.clear();
        match reader.read_line(buf) {
            Ok(0) => break,
            Ok(_) => {},
            Err(_) => break,
        }
    }
}
fn main() {
    let mut s = String::new();
    foo(&mut s);
}
