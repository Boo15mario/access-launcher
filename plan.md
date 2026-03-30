1. Modify `build_category_map` in `src/desktop.rs` to return `BTreeMap<&'static str, Vec<usize>>`.
   - Update the signature from `BTreeMap<String, Vec<usize>>` to `BTreeMap<&'static str, Vec<usize>>`.
   - In the body, use `map.insert(bucket, vec![i]);` instead of `map.insert(bucket.to_string(), vec![i]);`.
2. Modify `update_program_list` in `src/ui.rs` to accept `&BTreeMap<&str, Vec<usize>>` instead of `&BTreeMap<String, Vec<usize>>`.
   - Change `category_map: &BTreeMap<String, Vec<usize>>` to `category_map: &BTreeMap<&str, Vec<usize>>`.
3. Verify that the tests and benchmarks still pass.
4. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
