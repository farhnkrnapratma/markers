# markers

High-performance HTML to Markdown conversion for LLMs, written in Rust.

One package, one `src/`. The bindings are optional Cargo features:

| Feature | Source | Purpose |
| --- | --- | --- |
| *(none)* | `src/lib.rs` | Rust library |
| `napi` | `src/bindings/node.rs` | Node.js / Bun bindings, published as `@mdrs/markers` |
| `wasm` | `src/bindings/wasm.rs` | WebAssembly bindings |

## Layout

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

## Usage

Rust:

```rust
let markdown = markers::convert("<h1>Hello</h1>");

let options = markers::ConversionOptions::new()
    .heading_style(markers::HeadingStyle::Setext)
    .base_url(Some("https://example.com".to_string()))
    .exclude_selectors(vec!["nav".to_string()]);
let markdown = markers::convert_with_options("<h1>Hello</h1>", &options);
```

Node.js / Bun:

```js
const { convert, convertAsync } = require('@mdrs/markers')

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

## Requirements

- Rust (stable or nightly) and Cargo
- [Bun](https://bun.sh) for the JavaScript side (`bun.lock` is the only lockfile used)

## Run

```sh
# Node.js / Bun bindings
bun install
bun run build
bun -e "console.log(require('./index.js').convert('<h1>Hello</h1>'))"
```

## Examples

```sh
cargo run --example basic     # default options
cargo run --example options   # headings, bullets, links, selectors
bun examples/node.js          # Node.js / Bun (after bun run build)
```

`examples/wasm.html` is a browser demo for the `wasm` build.

## Test

```sh
cargo test --features napi   # unit and integration tests, plus the napi module
bun run test                 # Node.js binding tests (after bun run build)
cargo clippy --all-targets --features napi
```

The WebAssembly build needs the `wasm32-unknown-unknown` target (`napi` is
compiled out on `wasm32`, so `--all-features` also works there):

```sh
cargo check --target wasm32-unknown-unknown --features wasm
```

## Git hooks

Install the pre-commit hook that scans staged files for secrets with [TruffleHog](https://github.com/trufflesecurity/trufflehog):

```sh
bash .githooks/install.sh
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

Copyright © 2026 Farhan Kurnia Pratama
