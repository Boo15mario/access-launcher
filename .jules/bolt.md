## 2026-01-27 - Partial File Parsing Optimization
**Learning:** Even for small files like .desktop entries, `fs::read_to_string` can be a bottleneck if we only need the header. Using `BufReader` with a reusable string buffer reduced parsing time by ~16% by avoiding full file reads and repeated allocations.
**Action:** When parsing file headers or specific sections, prefer streaming with `BufReader` over reading the entire file into memory, especially in hot loops.

## 2026-05-22 - Redundant Sorting Removal
**Learning:** Sorting categorized subsets of data is redundant if the superset is already sorted and iteration order is preserved. By relying on `collect_desktop_entries` pre-sorting, we removed $O(N \log N)$ work and $N$ string allocations inside `build_category_map`.
**Action:** Always check if input data guarantees an ordering that allows skipping downstream sorts.

## 2026-06-15 - Shell Argument Parsing Optimization
**Learning:** `glib::shell_parse_argv` involves FFI and allocation which can be significant in a hot loop. For simple shell commands (no quotes, no escapes), manual string splitting and path checking is ~10x faster for relative commands and ~1.25x faster for absolute paths.
**Action:** When validating shell commands in a loop, implement a fast path for unquoted strings to avoid FFI overhead.
