## 2024-05-22 - [Optimized Desktop Entry Scanning]
**Learning:** In applications that scan multiple prioritized directories (like XDG_DATA_DIRS), checking if a high-priority item is already valid *before* parsing low-priority items can save significant I/O.
**Action:** Always check if an expensive operation is necessary based on already known state before performing it.
