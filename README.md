# markers

[![CircleCI](https://dl.circleci.com/status-badge/img/circleci/stZqvYiNAK17m5zy7ERCb/5KDDkHQnUroSHKP957pxaW/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/circleci/stZqvYiNAK17m5zy7ERCb/5KDDkHQnUroSHKP957pxaW/tree/main) [![npm](https://img.shields.io/npm/v/@farhnkrnapratma/markers?label=npm)](https://www.npmjs.com/package/@farhnkrnapratma/markers) [![crates.io](https://img.shields.io/crates/v/markers?label=crates.io)](https://crates.io/crates/markers) ![Language](https://img.shields.io/badge/language-Rust-orange?logo=rust) ![Tests](https://img.shields.io/badge/tests-cargo%20test-orange?logo=rust) ![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)


High-performance HTML to Markdown conversion for LLMs, written in Rust.

## Status

Alpha (`v1.0.0-alpha1`). The API can still change before the first stable release.

| Registry | Name |
| --- | --- |
| crates.io | [`markers`](https://crates.io/crates/markers) |
| npm | [`@farhnkrnapratma/markers`](https://www.npmjs.com/package/@farhnkrnapratma/markers) |

One package, one `src/`. The bindings are optional Cargo features:

| Feature | Source | Purpose |
| --- | --- | --- |
| *(none)* | `src/lib.rs` | Rust library |
| `napi` | `src/bindings/node.rs` | Node.js / Bun bindings, published as `@farhnkrnapratma/markers` |
| `wasm` | `src/bindings/wasm.rs` | WebAssembly bindings |

## Install

```sh
cargo add markers                       # Rust
bun add @farhnkrnapratma/markers        # Node.js / Bun (or: npm install @farhnkrnapratma/markers)
```

The npm package ships prebuilt binaries for Linux (x64 and arm64, glibc and musl), macOS (x64 and arm64) and Windows (x64 and arm64) as optional dependencies, so nothing is compiled on install.

## Usage

Rust:

```rust
fn main() -> Result<(), String> {
    let _markdown = markers::convert("<h1>Hello</h1>");

    let options = markers::ConversionOptions::new()
        .heading_style(markers::HeadingStyle::Setext)
        .base_url(Some("https://example.com".to_string()))
        .exclude_selectors(vec!["nav".to_string()]);
    let markdown = markers::convert_with_options("<h1>Hello</h1>", &options)?;
    println!("{markdown}");
    Ok(())
}
```

`convert_with_options` returns an error when an include or exclude CSS selector is invalid.

Node.js / Bun:

```js
const { convert, convertAsync } = require('@farhnkrnapratma/markers')

convert('<h1>Hello</h1>')
convert('<h1>Hello</h1>', { headingStyle: 'setext', baseUrl: 'https://example.com' })
await convertAsync('<h1>Hello</h1>')
```

## Options

| Option (Rust / JS) | Values | Default |
| --- | --- | --- |
| `heading_style` / `headingStyle` | `atx`, `setext` | `atx` |
| `link_style` / `linkStyle` | `inline`, `referenced` | `inline` |
| `code_fence` / `codeFence` | `` ` ``, `~` | `` ` `` |
| `bullet_marker` / `bulletMarker` | `-`, `*`, `+` | `-` |
| `base_url` / `baseUrl` | URL used to resolve relative links and images | none |
| `exclude_selectors` / `excludeSelectors` | CSS selectors to drop | none |
| `include_selectors` / `includeSelectors` | CSS selectors kept even if an exclude selector matches | none |

## Repository layout

```
src/
  lib.rs        public API: convert, convert_with_options, ConversionOptions
  options.rs    ConversionOptions, HeadingStyle, LinkStyle
  pipeline/     the conversion: input_guard -> parse -> annotate -> render -> cleanup
  handlers/     one TagHandler per group of tags (block/ and inline/)
  text/         Markdown escaping, URL handling, whitespace
  bindings/     Node.js (napi) and WebAssembly (wasm) glue
tests/          integration tests and HTML fixtures
examples/       Rust, Node.js and browser demos
```

## Development

Requirements: Rust (stable) and Cargo, and [Bun](https://bun.sh) for the JavaScript side (`bun.lock` is the only lockfile).

```sh
bun install                  # JS dependencies and git hooks
bun run build                # Node.js / Bun binding for this platform
cargo test --features napi   # unit and integration tests, plus the napi module
bun run test                 # Node.js binding tests (after bun run build)
cargo clippy --all-targets --features napi
cargo bench --bench conversion   # set MARKERS_BENCH_ITERATIONS to change the run length
```

Examples:

```sh
cargo run --example basic     # default options
cargo run --example options   # headings, bullets, links, selectors
bun examples/node.js          # Node.js / Bun (after bun run build)
```

`examples/wasm.html` is a browser demo for the `wasm` build, which needs the `wasm32-unknown-unknown` target (`napi` is compiled out on `wasm32`, so `--all-features` also works there):

```sh
cargo check --target wasm32-unknown-unknown --features wasm
```

Releases are cut with git-flow (`git-flow release start/finish`). Pushing a `v*` tag makes CircleCI build the bindings for every platform and publish them to npm; the crate is published with `cargo publish`.

## Contributing

This repository uses [git-flow-next](https://git-flow.sh/) (classic preset): `main` is release-only, work happens on `feature/*` and `bugfix/*` branches off `develop`. Commits are signed off (`git commit -s`) and follow [Conventional Commits](https://www.conventionalcommits.org), enforced by commitlint in a `commit-msg` hook and in CI. Install the hooks with `bash .githooks/install.sh` (`bun install` does it through the `prepare` script). CI runs on CircleCI: `reuse lint`, format and lint, tests, `cargo deny`, a TruffleHog secret scan and commitlint on every push. The branch rules are written out in [`CLAUDE.md`](CLAUDE.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

Copyright © 2026 Farhan Kurnia Pratama
