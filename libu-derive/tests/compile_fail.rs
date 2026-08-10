//! Compile-fail regression tests.
//!
//! The builder's compile_error diagnostics are pinned as exact compiler
//! output snapshots under tests/ui/. Regenerate with `TRYBUILD=overwrite
//! cargo test -p libu-derive` after intentionally changing a diagnostic.

#[test]
fn compile_fail() {
  let t = trybuild::TestCases::new();
  t.compile_fail("tests/ui/*.rs");
}
