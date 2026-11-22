# Contributing to markers

## Setup

You need Rust (stable) and [Bun](https://bun.sh). `bun.lock` is the only lockfile.

```sh
bun install    # JavaScript dependencies, commitlint and the git hooks
```

## Build and test

```sh
cargo test --locked --features napi          # unit and integration tests
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --check
cargo deny check
cargo llvm-cov --locked --summary-only       # must report no uncovered lines or functions
bun run build                                # Node.js binding for this platform
bun run test                                 # Node.js binding tests (after bun run build)
cargo check --target wasm32-unknown-unknown --features wasm
cargo bench --bench conversion               # MARKERS_BENCH_ITERATIONS sets the run length
```

Rust integration tests live in `src/specs/`, compiled only under `cfg(test)`, rather than in `tests/`: `cargo llvm-cov` counts a line as covered only when a single test binary runs it, so keep new Rust tests in that binary.

Examples:

```sh
cargo run --example basic     # default options
cargo run --example options   # headings, bullets, links, selectors
bun examples/node.js          # Node.js (after bun run build)
```

`examples/wasm.html` is a browser demo for the `wasm` feature.

## Repository layout

```
src/lib.rs      public API: convert, convert_with_options, ConversionOptions
src/options.rs  ConversionOptions, HeadingStyle, LinkStyle
src/pipeline/   the conversion: input_guard, annotate, render, cleanup
src/handlers/   one handler per group of tags (block/ and inline/)
src/text/       Markdown escaping, URL handling, whitespace
src/bindings/   Node.js (napi) and WebAssembly (wasm) glue
src/specs/      Rust integration tests
tests/          Node.js binding tests and HTML fixtures
examples/       Rust, Node.js and browser demos
benches/        conversion benchmark
```

## Submitting changes

- Branch from `develop` as `feature/<name>` or `bugfix/<name>`. The repository uses [git-flow-next](https://git-flow.sh/), and `main` only receives releases and hotfixes.
- Write commit messages in the [Conventional Commits](https://www.conventionalcommits.org) format and sign them off with `git commit -s`. The `commit-msg` hook and CI check both.
- Keep the tests, clippy, formatting and coverage passing, and add a regression test with every fix.

CI runs on CircleCI: `reuse lint`, formatting and lint, tests, coverage, `cargo deny`, a secret scan and commitlint on every push.

## Reporting security issues

Do not open a public issue for a vulnerability. Report it privately through [GitHub's security advisories](https://github.com/farhnkrnapratma/markers/security/advisories/new).
