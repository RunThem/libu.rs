# libu.rs — Project Guidelines

Rust utilities workspace. **Requires the nightly toolchain** — the root crate, `libu-derive` and `libu-timer` use `#![feature(proc_macro_hygiene)]`, so stable will not compile.

## Build and Test

- Build / check: `cargo build`, `cargo check`
- Test: `cargo test --workspace` — all unit tests and doctests pass. Doctests in member crates import the member crate directly (`use libu_derive::Builder;`, `use libu_trait::Pretty;`, `libu_macro::hash!`) — never `libu::`, since a member crate cannot dev-depend on the umbrella `libu` (that would create a dependency cycle). The `#[clone]` doctest additionally needs `#![feature(proc_macro_hygiene)]` (nightly only).
- Format: `cargo fmt` (2-space indent, see `.rustfmt.toml`)

## Architecture

Umbrella root crate re-exports all 6 workspace members (`pub use libu_*::*`) and exposes `pub mod dependency { pub mod parking_lot; pub mod flume; }`.

| Crate | Role |
|-------|------|
| `libu-derive` | proc-macros: `Builder` derive (`with_`-prefixed setters, `#[builder(prefix)]`/`skip`/`private`/`into`/`must`/`default = expr`; `Option` fields get dual setters, `String` fields default to `impl Into<String>`, const/default generics supported, contradictory attr combos (`must`+`skip`, `must`+`default`, `must`/`default` on `Option`) rejected at compile time, missing `Default` gets a custom `on_unimplemented` diagnostic, builder struct is encapsulated in a private module so its fields are unreachable crate-wide), **unsafe** `Send`/`Sync` derives, `#[clone]` attribute, `select!` macro |
| `libu-point` | pointer helpers: `iBox`, `Mrc<T> = Arc<parking_lot::Mutex<T>>`, `Urc<T> = Rc<RefCell<T>>`; all methods live in the single merged traits `MrcExt`/`UrcExt` (`at`, `with`/`with_mut`, `set`/`swap`/`take`, `try_with`/`try_with_mut`, `val_eq`/`ptr_eq`) plus the `iXxx` constructors; call sites use `use libu::prelude::*;` |
| `libu-timer` | 4096-bucket / 100ms-tick timing wheel; free fns `delay`/`ticker`, types `Timer`/`TimerHandle`. Depends on `libu-point` + `#[clone]` |
| `libu-chan` | `Chan<S,R>` bidirectional point-to-point channel over two flume `unbounded` channels; blocking/try/timeout send+recv, `iter`, `disassemble` into raw flume halves; `Debug` prints via `type_name` (no `tynm` dep) |
| `libu-macro` | `macro_rules!` collection/control-flow macros (`hmap!`, `brk_if!`, `count!`, …), no deps |
| `libu-trait` | `#[ext]`-based trait extensions, one module each (`pick`, `bzero`, `void`, `dur`, `remove_if`, `pretty`): `Pick`, `Bzero`, `Void`, `ToDur`(+`try_to_dur`, supports `h`/`d` and floats), `DurExt` numeric duration literals (`5.secs()`), `RemoveIf` (O(n) via `extract_if`), `Pretty` |

## Conventions

- Edition 2024, 2-space indent; most crates start with `#![allow(unused)]` + `#![allow(non_snake_case)]` (exceptions: `libu-macro`, `libu-trait`)
- Extension methods use an `i` prefix meaning "into" (`iBox`, `iMrc`, `iUrc`); type aliases are `Mrc`/`Urc`; `box.rs` uses the raw identifier `mod r#box`
- Dependencies pinned to exact versions (no `^`); always use `parking_lot` locks (never std `Mutex`); `flume` is always `default-features = false, features = ["select"]`
- Heavy `///` doc comments with Markdown tables and `rust` examples; `//!` module docs with macro reference tables
- Derive/attribute/function macros are re-exported through the root crate (`libu::Builder`, `libu::clone`, `libu::select!`)

## Gotchas

- `select!` expands to `::flume::Selector::new()` — `flume` must resolve as a top-level crate at the call site (the root re-export at `libu::dependency::flume` will **not** match)
- `#[clone(a, b)]` on an item requires `#![feature(proc_macro_hygiene)]` in the *using* crate (nightly only)
- The `Send`/`Sync` derives emit `unsafe impl` **intentionally** — keep their safety docs; do not "harden" them into safe impls
- `libu-timer` callbacks run inside `catch_unwind(AssertUnwindSafe)` — a panicking callback is isolated/removed, so don't rely on unwinding across the callback boundary
