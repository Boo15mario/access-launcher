## 2024-05-18 - Language Tag Matching Fast Path
**Learning:** Checking the first byte of `tag` and `lang` for exact mismatch (`if tag.as_bytes()[0] != lang.as_bytes()[0]`) before executing expensive string slicing and finding (`.find(['.', '@'])`) provides a measurable ~19% speedup when scanning a large number of unsupported localized keys in `.desktop` files.
**Action:** When performing substring or transformed comparisons in hot parsing loops, eagerly test the first byte if the keys usually start with distinct characters.

## 2024-05-18 - Avoiding Allocation with Env Var String Slices
**Learning:** `XDG_CURRENT_DESKTOP` parsing used to split the env var and map each segment into an owned `String`, creating an allocating `Vec<String>`. Keeping the raw env var string alive and producing `Vec<&str>` skips allocations and speeds up initialization significantly (~55% speedup in benchmark).
**Action:** In function signatures that accept slices of string-like data (e.g., filtering logic), use `Option<&[impl AsRef<str>]>` rather than `Option<&[String]>` so the caller isn't forced to allocate when they possess borrowed slices.
