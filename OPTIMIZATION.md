# 工程优化记录 — 2026-08-28

> 全工程体检的点时快照。审计范围:根 crate 与全部 6 个成员(`libu-derive`、`libu-point`、`libu-timer`、`libu-chan`、`libu-macro`、`libu-trait`)。
> **注:`libu-timer` 已于 2026-08-28 整体移除,本文中相关条目随之失效。**
> 对应的最新提交基线:`e73a75f`。后续执行时按批次勾掉即可。

## 0. 已验证的事实(nightly 需求比文档写的更窄)

实测移除三处 `#![feature(proc_macro_hygiene)]` 后:

- 根 crate 与 `libu-derive`:照常编译,46 个测试全过 → **gate 属于可删的历史遗留**
- `libu-timer`:`timer.rs:266` 的 `#[clone(wheel, shutdown)]`(语句位置属性宏,issue #54727 至今未稳定)→ ~~nightly 仍是硬需求~~ **已随 crate 移除而失效**;现存的 nightly 依赖仅剩根 crate 与 `libu-derive` 的 crate 级 gate(实验证明均可删),以及任何下游在语句位置使用 `#[clone]` 的 crate

结论:AGENTS/README 应表述为「`libu-timer` 与语句级 `#[clone]` 使用者需要 nightly」,而非全工程。

## 1. 工程基建(收益最大、成本最低)

> 2026-08-28 进度:除 README 外全部完成。`rust-toolchain.toml` 钉的是 `nightly-x86_64-pc-windows-gnu`(工程历史实际使用的工具链,自带链接器);CI 中以 `RUSTUP_TOOLCHAIN` 覆盖为 Linux 三元组。

| 项 | 现状 | 建议 |
|----|------|------|
| `rust-toolchain.toml` | ~~不存在~~ 已新增 | 钉住 nightly(见上注) |
| CI | ~~无 `.github/`~~ 已新增 `ci.yml` | `cargo fmt --all --check` + `cargo test --workspace` |
| README.md | 仅 `# libu.rs` 一行 + 分割线 | 把 AGENTS.md 的 crate 表和构建说明搬过去(**待办**) |
| 依赖声明 | ~~各成员各自声明~~ 已集中 | `[workspace.dependencies]` 集中,成员 `workspace = true` |
| 「精确锁版」 | `version = "0.12.0"` 语义上仍是 caret 范围 | 真精确需 `"=0.12.0"`;以 Cargo.lock 为准(已采纳此路线) |
| resolver | ~~显式 `resolver = "2"`~~ 已删除 | edition 2024 默认 resolver 3 |

## 2. 确认的缺陷(按严重度)

1. **`RUST_KEYWORDS` 缺 `gen`**(`libu-derive/src/builder.rs`)。`gen` 是 edition 2024 保留字,而该 crate 正是 edition 2024;字段名为 `gen` 会生成非法 setter。一行修复 + trybuild 快照。
2. **`clone.rs` 四个潜在缺陷且零测试** —— **已完成**(`c69e135`),四个缺陷逐一转为 `compile_error!` 并补齐测试:
   - ~~`#[clone(x)] let y;` 触发 `init.unwrap()` panic~~ → 报「let 需带初始化器」
   - ~~`#[clone(a.b)]` 被静默拆成 `a`、`b` 两个 clone(非 ident token 丢失)~~ → 报「expected an identifier」(仅放行分隔逗号,不误伤 `#[clone(a, b)]`)
   - ~~非局部项/非表达式项静默 no-op,无任何诊断~~ → 编译错误;顺带表达式语句(`#[clone(x)] foo();`)从 no-op 改为真正生效(需 `#![feature(stmt_expr_attributes)]`)
   - ~~重复 ident 产生冗余克隆~~ → 报「duplicate identifier」
   新增测试:5 个 compile-fail 快照(`tests/ui/clone_*.rs`)、1 个运行时 pass 用例、7 个 `#[cfg(test)]` 单元断言。
3. **条件控制宏的 `(cond, stmt)` 形态** —— **已完成**(`d49eda2`)。探针实测**只有 `chk_if!` 受影响**:`$val:expr` 臂在前,裸表达式语句恒被该臂捕获(`chk_if!(s.is_empty(), drop(s))` 实际是「返回 `drop(s)` 的值」),「先执行语句再返回」永不成立;`brk_if!`/`cnt_if!` 的两参臂是 `$label:lifetime`,裸表达式可正常命中 `$stmt:stmt` 臂,**不受影响**(原条目把三个并列为同一缺陷属过度概括)。修法按建议把 `chk_if!` 语句臂改为 `$stmt:block`:`chk_if!(cond, { stmt })`、`chk_if!(cond, val, { stmt })` 先执行块再返回;值形式 `chk_if!(cond, val)` 不变(块臂声明在前,裸表达式仍落值臂)。`brk_if!`/`cnt_if!` 宏未改,仅把示例补成可运行 doctest(含块形态),钉住 `(cond, stmt)` 行为。
4. **Builder 次级问题**(不阻塞,值得排期):
   - 诊断一次只报一个错,应累积
   - `forward_attrs(cfg)` 把 `#[cfg]` 复制进 builder 字段,而 `build()` 无条件引用它们 → cfg 掉字段即 codegen 悬空
   - `to_tokens` 约 350 行单函数,可拆分

## 3. 模块引入与卫生

- `libu-point/src/mrc.rs` 的 `pub use parking_lot::{Mutex, MutexGuard}; pub use std::sync::Arc;`:与 `urc.rs`(未 pub use `Rc`/`RefCell`)不对称,且经伞形 glob 污染用户作用域(`libu::Arc`/`libu::Mutex`)。建议删除;需要 `Mutex` 的人走 `libu::dependency::parking_lot`。
- 三个 crate 级 `#![allow(unused)]`(根、`libu-point`、`libu-derive`、`libu-timer`):代码已是 curated 状态,全局豁免关闭了 dead-code 检测。删除后按报警点加窄的 `#[allow(dead_code)]`。
- `libu-derive` 的 `send.rs`/`sync.rs`:约 200 行 × 2 镜像复制,逻辑仅差 trait 名;抽共享生成函数可减半,测试同理。
- syn features 冗余:`parsing`/`printing`/`derive`/`clone-impls`/`proc-macro` 是默认特性,只需 `["full", "extra-traits"]`;激进选项为移除 darling(仅 Builder 使用)。
- 根 crate `mod test` 中 `fn tset` 为 typo,且仅测三个构造器;建议改名并扩展为 prelude 冒烟测试。

## 4. API 增补(按价值排序)

| 目标 | 内容 | 说明 |
|------|------|------|
| `libu-chan` | `Chan` 手写 `Clone` | 两个半端在 flume 中均 `Clone`;必须手写——derive 会错误地要求 `S: Clone, R: Clone` |
| `libu-derive` | `select!` 加 `timeout`/`default` 臂;Builder 结构级选项(`builder_name`/`visibility`);`#[clone]` 支持 path | `select!` 目前 recv-only;`select2` 内部命名建议合并 |
| 删除候选 | 无强建议 | `count!` 虽 `doc(hidden)` 但被 `$crate::count!` 依赖,保留 |

## 5. 建议执行批次

- **P0(真 bug)**:`gen` 关键字、~~clone.rs 加固 + 测试~~ ✔(`c69e135`)、~~条件宏 stmt 臂语义~~ ✔(`d49eda2`)、Builder 诊断累积
- **P1(基建)**:`rust-toolchain.toml` + CI + README + workspace.dependencies + resolver 3 + nightly 表述精确化
- **P2(卫生)**:Arc/Mutex pub-use 摘除、`allow(unused)` 收窄、send/sync 去重、syn features、根测试改写
- **P3(API)**:`Chan: Clone`、select!/Builder 扩展

## 附:当前已完成项(历史记录)

- `9e31962` 删除死模块 arc/rc/sptr(含 soundness 缺陷的 Sptr)
- `ae3ac5d` 移除 libu-log(无使用者的 log 薄封装)
- `475615b` libu-point 方法 trait 合并为 `MrcExt`/`UrcExt`(extend 宏支持方法级 where,已探针验证)+ 写侧糖 + try 系列 + val_eq/ptr_eq + 全量文档
- `2a0fcff` 伞形 `libu::prelude`(调用点单行导入;prelude 只留伞形层,成员层按设计不加)
- `49ae7a7` libu-trait:`RemoveIf` 换 `extract_if`(O(n))、`ToDur` 修 panic(新增 `try_to_dur`/`h`/`d`/浮点)、新增 `DurExt` 字面量糖、拆分六模块
- `e73a75f` libu-chan:send/recv 对称族、`disassemble`、移除 `tynm` 依赖、补文档与首个测试套件
- `c69e135` libu-derive:`#[clone]` 属性宏加固(四项缺陷全部转 `compile_error!`)+ 首个测试套件(5 快照 / 1 pass / 7 单测)
- `d49eda2` libu-macro:`chk_if!` 语句臂改 `$stmt:block`(先执行再返回可及;`brk_if!`/`cnt_if!` 探针实测不受影响)+ 三个宏补可运行 doctest
