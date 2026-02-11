## 2026-01-27 - Partial File Parsing Optimization
**Learning:** Even for small files like .desktop entries, `fs::read_to_string` can be a bottleneck if we only need the header. Using `BufReader` with a reusable string buffer reduced parsing time by ~16% by avoiding full file reads and repeated allocations.
**Action:** When parsing file headers or specific sections, prefer streaming with `BufReader` over reading the entire file into memory, especially in hot loops.

## 2026-05-22 - Redundant Sorting Removal
**Learning:** Sorting categorized subsets of data is redundant if the superset is already sorted and iteration order is preserved. By relying on `collect_desktop_entries` pre-sorting, we removed $O(N \log N)$ work and $N$ string allocations inside `build_category_map`.
**Action:** Always check if input data guarantees an ordering that allows skipping downstream sorts.

## 2026-06-15 - Shell Argument Parsing Optimization
**Learning:** `glib::shell_parse_argv` involves FFI and allocation which can be significant in a hot loop. For simple shell commands (no quotes, no escapes), manual string splitting and path checking is ~10x faster for relative commands and ~1.25x faster for absolute paths.
**Action:** When validating shell commands in a loop, implement a fast path for unquoted strings to avoid FFI overhead.

## 2026-06-25 - Shadowing Logic Optimization
**Learning:** Checking for file existence (ID shadowing) before parsing is significantly faster than parsing and then checking validity. Correct XDG shadowing requires masking lower-priority files even if the high-priority file is hidden or invalid.
**Action:** When implementing shadowing/overrides, use a `HashSet` of seen IDs to eagerly skip processing of lower-priority items.

## 2026-06-26 - Lazy Parsing Trade-offs
**Learning:** Delaying allocations for frequently present fields like `Categories` (Vec<String>) can be counter-productive due to the overhead of buffering the raw string (extra allocation/copy). However, for fields used primarily for filtering (like `OnlyShowIn`), storing the raw string and validating lazily avoids vector allocations entirely, which is a win.
**Action:** Use lazy parsing/validation for optional filter fields, but parse required/common fields eagerly to avoid double-allocation penalties.

## 2026-07-15 - Zero-Allocation Parsing Loop
**Learning:** For high-volume file parsing (like scanning hundreds of .desktop files), allocating a new `String` buffer for each file's `read_line` loop adds significant overhead. Passing a reusable mutable buffer from the caller eliminated these repeated allocations.
**Action:** When parsing many files in a loop, lift the buffer allocation out of the parsing function and pass it as `&mut String`.

## 2026-07-15 - Streamlined Directory Traversal
**Learning:** Collecting all file paths into a `Vec<PathBuf>` before processing them consumes unnecessary memory and delays processing. Using a `FnMut` callback allows processing files immediately as they are discovered, improving cache locality and reducing peak memory usage.
**Action:** Prefer callback-based traversal over collecting results into a vector when the consumer processes items sequentially.

## 2026-02-11 - Zero-Allocation Category Parsing
**Learning:** Parsing semicolon-separated lists (like `Categories`) into `Vec<String>` is costly due to multiple allocations per file. Storing the raw string and iterating over it lazily with `split(';')` avoids `Vec` and `String` allocations entirely, enabling zero-allocation category mapping.
**Action:** For string list fields that are primarily used for lookup or mapping, store the raw string and parse lazily using iterators.
