# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains the core editor implementation (C and Lua), plus generators in `src/gen/` and manpage sources in `src/man/`.
- `runtime/` holds runtime scripts, docs in `runtime/doc/`, and bundled plugins under `runtime/pack/`.
- `test/` is the test suite: `test/functional/` (Lua specs), `test/unit/`, and `test/old/` for legacy Vim tests.
- `vxd/` is the trait-based Vim behavior specification crate; `vxd_tui/` is the TUI implementation.
- `cmake/`, `cmake.deps/`, `build.zig`, and `CMakeLists.txt` define build tooling; `scripts/` and `contrib/` contain helper utilities.

## Build, Test, and Development Commands
- `make` builds the project (CMake + Ninja if available). Example: `make CMAKE_BUILD_TYPE=RelWithDebInfo`.
- `VIMRUNTIME=runtime ./build/bin/nvim` runs the freshly built binary without installing.
- `make install` installs to the configured prefix (use `sudo` only if required).
- `make test` runs the test suite; see `runtime/doc/dev_test.txt` for detailed targets and flags.
- `VALGRIND=1 make test` or `CC=clang make CMAKE_FLAGS="-DENABLE_ASAN_UBSAN=ON"` runs analysis builds.
- Zig builds are supported: `zig build`, `zig build functionaltest -- test/functional/...`.
- For the Rust crates, run from each crate directory: `cargo test` (and `cargo run` in `vxd_tui/` if you are iterating on the TUI).

## Coding Style & Naming Conventions
- Follow the style guide in `runtime/doc/dev_style.txt`.
- Headers use `#pragma once` and are split into `foo.h` and `foo_defs.h` for symbols vs. types.
- Use `src/clint.lua` for style checks and `make format` (or `formatc`/`formatlua`) for formatting; `src/uncrustify.cfg` is authoritative for C formatting.
- Avoid VLAs/`alloca()` and prefer const pointers where appropriate; keep public integer types fixed-width.
- For Rust in `vxd/` and `vxd_tui/`, use standard rustfmt (`cargo fmt`) and idiomatic module naming.

## Testing Guidelines
- Functional tests are Lua `_spec.lua` files under `test/functional/`; add new coverage there when behavior changes.
- Unit tests live in `test/unit/`, and legacy Vim tests are mirrored under `test/old/`.
- Ensure PRs include test coverage and pass CI; run focused tests before broad test runs.

## Commit & Pull Request Guidelines
- Use Conventional Commits; format is `type(scope): subject` (scope optional). Allowed types include `build`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `test`, `vim-patch`. Run `make lintcommit` to validate.
- Open draft PRs while work is in progress, avoid unrelated cosmetic changes, and keep branches rebased.
- CI (Cirrus + GitHub Actions) must pass; warnings are treated as errors.
