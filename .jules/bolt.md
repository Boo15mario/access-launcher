## 2026-01-27 - Partial File Parsing Optimization
**Learning:** Even for small files like .desktop entries, `fs::read_to_string` can be a bottleneck if we only need the header. Using `BufReader` with a reusable string buffer reduced parsing time by ~16% by avoiding full file reads and repeated allocations.
**Action:** When parsing file headers or specific sections, prefer streaming with `BufReader` over reading the entire file into memory, especially in hot loops.
