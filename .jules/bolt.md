## 2024-05-19 - [Optimize lang_tag parsing string search]
**Learning:** In `src/desktop.rs`, searching for ASCII separators in strings (like language tags) using `.bytes().position(|b| matches!(b, b'.' | b'@'))` is significantly faster than using `.find(['.', '@'])`, as it avoids UTF-8 character boundary decoding overhead.
**Action:** Replace `lang.find(['.', '@'])` with `lang.bytes().position(|b| matches!(b, b'.' | b'@'))` to improve performance without impacting readability.

## 2024-05-19 - [Optimize lang_tag parsing string search]
**Learning:** In `src/desktop.rs`, searching for ASCII separators in strings (like language tags) using `.bytes().position(|b| matches!(b, b'.' | b'@'))` is significantly faster than using `.find(['.', '@'])`, as it avoids UTF-8 character boundary decoding overhead.
**Action:** Replace `lang.find(['.', '@'])` with `lang.bytes().position(|b| matches!(b, b'.' | b'@'))` to improve performance without impacting readability.
