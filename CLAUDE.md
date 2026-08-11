# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`rocrate` is a Rust library (single crate, no workspace) for reading, writing, building, validating, and navigating [RO-Crates](https://www.researchobject.org/ro-crate/): a strongly typed API over the RO-Crate JSON-LD model that preserves lossless serialization. It supports the base RO-Crate spec plus Workflow RO-Crates and Workflow Run Crates. It's used by [SciWIn](https://github.com/fairagro/sciwin) (sibling repo `../sciwin`) for RO-Crate support.

Supported specifications: RO-Crate 1.0/1.1/1.2, Workflow RO-Crate 1.0, Workflow Run Crates (Process Run, Workflow Run, Provenance Run), Workflow Testing RO-Crate terms.

### Module layout (`src/`)

- `lib.rs` — the `RoCrate` type itself (`@context` + `@graph`), and crate-level navigation: `descriptor()` (the metadata descriptor entity), `root()` (the root data entity via the descriptor's `about`), `main_entity()`, `conforms_to()`/`profiles()`/`claims()` (declared profiles), `data_entities()` (BFS over `hasPart`), `contextual_entities()`.
- `graph/` — the `@graph` array and `GraphNode` (a single JSON-LD node), the underlying entity model everything else is built on.
- `context.rs` — `@context` handling / automatic context management.
- `profile.rs` — the `Profile` enum (which RO-Crate/Workflow-RO-Crate/Run-Crate spec + version a crate conforms to).
- `terms.rs` — typed vocabulary/terms constants for the supported specs.
- `constants.rs` — fixed values such as `METADATA_FILE_NAMES`, `ROOT_ID`.
- `build/` — the fluent builder API (`RoCrate::builder()`, `build::Entity`) for constructing crates programmatically.
- `validate/` — profile validation (`base.rs`, `workflow.rs`, `run.rs`, orchestrated by `mod.rs`); validates a crate against its declared profiles or an arbitrary one.
- `views/` — typed, read-only views over graph nodes for common entity kinds (`base.rs`, `workflow.rs`, `run.rs`, `test.rs`), e.g. `crate_.workflow()`, `workflow.inputs()`, `workflow.steps()` — the preferred way to navigate a crate instead of walking `@graph` JSON-LD by hand.
- `io/` — reading/writing crates as directories or ZIP archives (`archive.rs`); `RoCrate::from_directory()`, `RoCrate::from_zip()`. ZIP support is behind the (default-on) `zip` feature flag.

Parsing (`RoCrate` via `serde`) is deliberately lenient.

## Common commands

```bash
cargo build
cargo clippy --workspace -- -W clippy::pedantic     # matches CI lint step
cargo nextest run --workspace --no-fail-fast        # matches CI test step (requires cargo-nextest)
cargo test some_test_name                           # run a single test
cargo run --example report                          # run the `report` example
```

Integration tests live in `tests/` (`build.rs`, `io.rs`, `smoke.rs`, `validate.rs`, `views.rs`), one file per major API surface.

CI (`.github/workflows/ci.yaml`) runs on Ubuntu: `cargo clippy --workspace -- -W clippy::pedantic`, `cargo nextest run --workspace --no-fail-fast`, then coverage via `cargo tarpaulin`.

To build without ZIP support: `cargo build --no-default-features` (or explicitly `--features zip` to keep it).
