//! Compile-fail regression tests.
//!
//! The builder's compile_error diagnostics are pinned as exact compiler
//! output snapshots under tests/ui/. Regenerate with `TRYBUILD=overwrite
//! cargo test -p libu-derive` after intentionally changing a diagnostic.
//! Pass cases under tests/ui/pass/ compile and run the generated code.

#[test]
fn compile_fail() {
  let t = trybuild::TestCases::new();
  t.compile_fail("tests/ui/*.rs");
}

#[test]
fn compile_pass() {
  let t = trybuild::TestCases::new();
  t.pass("tests/ui/pass/*.rs");
}
