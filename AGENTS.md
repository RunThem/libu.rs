# libu.rs — Project Guidelines

Rust utilities workspace. **Requires the nightly toolchain** — the root crate and `libu-timer` use `#![feature(proc_macro_hygiene)]`, so stable will not compile.

## Build and Test

- Build / check: `cargo build`, `cargo check`
- Test: `cargo test --workspace` — 3 of the `libu-derive` doctests still fail (`E0432: unresolved import 'libu'`); the `Builder` doctest was fixed by importing `libu_derive` directly. Do **not** "fix" the rest by adding a `libu` dev-dependency to `libu-derive` (that would create a dependency cycle); the intended fix is `ignore`/`no_run` on the doctests.
- Format: `cargo fmt` (2-space indent, see `.rustfmt.toml`)
- **Do not run `xmake`** — the `test` task in `tasks.json` is stale; there is no `xmake.lua` in this repo.

## Architecture

Umbrella root crate re-exports all 7 workspace members (`pub use libu_*::*`) and exposes `pub mod dependency { pub mod parking_lot; pub mod flume; }`.

| Crate | Role |
|-------|------|
| `libu-derive` | proc-macros: `Builder` derive (`with_`-prefixed setters, `#[builder(prefix)]`/`skip`/`private`/`into`/`must`/`default = expr`; `Option` fields get dual setters, `String` fields default to `impl Into<String>`, const/default generics supported, contradictory attr combos (`must`+`skip`, `must`+`default`, `must`/`default` on `Option`) rejected at compile time, missing `Default` gets a custom `on_unimplemented` diagnostic), **unsafe** `Send`/`Sync` derives, `#[clone]` attribute, `select!` macro |
| `libu-point` | pointer helpers: `iBox`, `Mrc<T> = Arc<parking_lot::Mutex<T>>`, `Urc<T> = Rc<RefCell<T>>` with `iMrc`/`iUrc`, `at()`, `with()`/`with_mut()` |
| `libu-timer` | 4096-bucket / 100ms-tick timing wheel; free fns `delay`/`ticker`, types `Timer`/`TimerHandle`. Depends on `libu-point` + `#[clone]` |
| `libu-chan` | `Chan<S,R>` bidirectional point-to-point channel over two flume `unbounded` channels |
| `libu-log` | global logger via `log`; `init()` hardcodes the `Trace` level; re-exports `log::*` |
| `libu-macro` | `macro_rules!` collection/control-flow macros (`hmap!`, `brk_if!`, `count!`, …), no deps |
| `libu-trait` | `#[ext]`-based trait extensions (`Pick`, `Bzero`, `Void`, `ToDur`, `RemoveIf`) |

`libu-point` has **dead modules** `arc.rs`, `rc.rs`, `sptr.rs` — they are not declared in `lib.rs` (only `r#box`, `mrc`, `urc` are). `sptr.rs` is a hand-written unsafe refcounted pointer (WIP).

## Conventions

- Edition 2024, 2-space indent; most crates start with `#![allow(unused)]` + `#![allow(non_snake_case)]` (exceptions: `libu-macro`, `libu-trait`)
- Extension methods use an `i` prefix meaning "into" (`iBox`, `iMrc`, `iUrc`); type aliases are `Mrc`/`Urc`/`Sptr`; `box.rs` uses the raw identifier `mod r#box`
- Dependencies pinned to exact versions (no `^`); always use `parking_lot` locks (never std `Mutex`); `flume` is always `default-features = false, features = ["select"]`
- Heavy `///` doc comments with Markdown tables and `rust` examples; `//!` module docs with macro reference tables
- Derive/attribute/function macros are re-exported through the root crate (`libu::Builder`, `libu::clone`, `libu::select!`)

## Gotchas

- `select!` expands to `::flume::Selector::new()` — `flume` must resolve as a top-level crate at the call site (the root re-export at `libu::dependency::flume` will **not** match)
- `#[clone(a, b)]` on an item requires `#![feature(proc_macro_hygiene)]` in the *using* crate (nightly only)
- The `Send`/`Sync` derives emit `unsafe impl` **intentionally** — keep their safety docs; do not "harden" them into safe impls
- `libu-timer` callbacks run inside `catch_unwind(AssertUnwindSafe)` — a panicking callback is isolated/removed, so don't rely on unwinding across the callback boundary
- `libu-log::init()` is fixed at `Trace`; there is no runtime level configuration
