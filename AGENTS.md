# computer4

A DIY computer system monorepo. Each subdirectory is an **independent Rust crate** (no Cargo workspace). Build/test per-crate only.

## Monorepo Map

| Directory | What | Entry |
|---|---|---|
| `compiler/lli4/` | LLVM IR interpreter | `lli4::interpret()` |
| `compiler/rustc4/` | Rust → LLVM IR compiler | `src/main.rs` |
| `compiler/py4/` | Python interpreter (standalone `rustc`, no Cargo) | `py4.rs` + `lib4.rs` |
| `compiler/objdump/` | ELF analyzer | `src/main.rs` |
| `database/db6/` | KV+SQL+FTS+Msgq database (flagship crate) | `src/lib.rs`, `src/main.rs` (REPL), `src/server_main.rs` (server) |
| `database/sql4/` | SQLite-like with CJK FTS | — |
| `database/btree/` | BTree engine | — |
| `database/lsm/` | LSM-Tree engine | — |
| `database/fts/` | Full-text search | — |
| `database/swisstable/` | Swiss Table | — |
| `database/patricia-trie/` | Patricia trie | — |
| `database/redblacktree/` | Red-black tree | `src/lib.rs` — [AGENTS.md](database/redblacktree/AGENTS.md) |
| `database/inodefs/` | Inode-based VFS | — |
| `math4/` | Math library (statistics, plot, ndarray, algebra, calculus, linear algebra, geometry) | `src/lib.rs` — [math4/AGENTS.md](math4/AGENTS.md) |
| `crypto/ssl4/` | SSL/TLS (rustls + tokio-rustls) | — |
| `gui/win4/` | Window manager (eframe/egui) | `src/main.rs` |
| `web/browser4/` | Web browser (eframe, boa_engine) | `src/main.rs` |
| `web/md4browser/` | Markdown browser (eframe) | `src/main.rs` |
| `media/jpeg/` | JPEG codec | — |
| `media/mp3/` | MP3 codec (package: `mpeg_codec`) | — |
| `media/mpeg1/` | MPEG-1 decoder (stdlib only) | `src/main.rs` |
| `embed/rvboard4/` | RISC-V board BSP | `./build.sh`, `./run.sh`, `./build_sim.sh`, `./run_sim.sh` |
| `tool/lz4/` | LZ4 compression (edition 2024) | — |
| `tool/regex4/` | Regex engine (standalone `rustc`, no Cargo) | `regex4.rs` |
| `tool/vi4/` | Terminal text editor (crossterm) | `src/main.rs` |
| `eda/ruhdl/` | Hardware description (empty) | — |

## Commands

```sh
cargo build              # Build current crate (no --workspace flag)
cargo test               # Test current crate
cargo run                # Run binary (where applicable)
./test.sh                # Some crates have a custom test script
./git.sh <msg> <branch>  # git add . && commit -m "$msg-$branch" && push
```

Standalone `rustc` crates (`compiler/py4/`, `tool/regex4/`):
```sh
rustc py4.rs -o py4 && ./py4
rustc regex4.rs -o regex4 && ./regex4
```

## Conventions

- **No workspace root** — each crate has its own `Cargo.lock` and `target/`. Never `cargo build --workspace`.
- Most crates use `edition = "2021"`; 5 crates use `edition = "2024"` (`lz4`, `sql4`, `btree`, `patricia-trie`, `redblacktree`)
- Source files contain comments in Traditional Chinese
- `#![allow(dead_code)]` in `math4/src/lib.rs` and `db6/src/lib.rs`
- No CI/CD, no toolchain pinning (`rust-toolchain.toml`)
- Compiler pipeline: `rustc4` writes `.ir` → `lli4` interprets `.ir`

## Existing per-package instruction files

- [`math4/AGENTS.md`](math4/AGENTS.md) — math library conventions (NaN handling, polynomial ascending order, etc.)
- [`database/db6/AGENTS.md`](database/db6/AGENTS.md) — db6 architecture, REPL commands, engine traits
- [`database/redblacktree/AGENTS.md`](database/redblacktree/AGENTS.md) — redblacktree commands, structure, notes
