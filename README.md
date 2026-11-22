# markers

[![CircleCI](https://dl.circleci.com/status-badge/img/circleci/stZqvYiNAK17m5zy7ERCb/5KDDkHQnUroSHKP957pxaW/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/circleci/stZqvYiNAK17m5zy7ERCb/5KDDkHQnUroSHKP957pxaW/tree/main) [![npm](https://img.shields.io/npm/v/@farhnkrnapratma/markers/alpha?label=npm)](https://www.npmjs.com/package/@farhnkrnapratma/markers) [![crates.io](https://img.shields.io/crates/v/markers?label=crates.io)](https://crates.io/crates/markers) [![docs.rs](https://img.shields.io/docsrs/markers)](https://docs.rs/markers) ![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

Converts HTML to clean GitHub Flavored Markdown. It is built for turning web pages into text for large language models: page chrome and non-text elements are dropped, unsafe links are removed, and CSS selectors let you keep only the content you want. The converter is written in Rust and is available as a Rust crate and as an npm package for Node.js and Bun.

> **Alpha.** The API may still change before 1.0.0. On npm, install the `alpha` tag.

## Features

- **GitHub Flavored Markdown output**: headings, lists, task lists, tables, blockquotes, code blocks, strikethrough, links and images, plus definition lists, `<details>`, figures and abbreviations.
- **Clean text for LLMs**: `<head>`, `<script>`, `<style>`, `<noscript>`, `<template>`, `<nav>`, `<svg>`, `<iframe>`, `<object>`, `<canvas>` and `<select>` are dropped, and `javascript:` links are removed while their text is kept.
- **Selectors**: drop more of a page with CSS selectors, and keep the parts you name even inside dropped ones (for example `includeSelectors: ["nav"]` to keep navigation).
- **Relative URLs resolved** against a base URL you provide.
- **Robust on hostile input**: documents nested more than 5,000 levels deep are rejected instead of exhausting the stack.

## Install

```sh
npm install @farhnkrnapratma/markers@alpha   # or: bun add @farhnkrnapratma/markers@alpha
cargo add markers                            # Rust
```

The npm package ships prebuilt native addons for Linux (x64 and arm64, glibc and musl), macOS (x64 and arm64) and Windows (x64 and arm64), so nothing is compiled on install.

## Usage

### Node.js and Bun

```js
const { convert, convertAsync } = require("@farhnkrnapratma/markers");

convert("<h1>Hello</h1>"); // "# Hello"

convert(html, {
  baseUrl: "https://example.com/blog/",
  excludeSelectors: ["footer", ".ads"],
});

await convertAsync(html); // runs off the main thread
```

### Rust

```rust
use markers::{ConversionOptions, HeadingStyle};

let markdown = markers::convert("<h1>Hello</h1>");

let options = ConversionOptions::new()
    .heading_style(HeadingStyle::Setext)
    .base_url(Some("https://example.com".to_string()))
    .exclude_selectors(vec!["nav".to_string()]);
let markdown = markers::convert_with_options(html, &options)?;
```

`convert_with_options` returns an error when an include or exclude selector is not valid CSS.

## Options

| Option (Rust / JavaScript)               | Values                                                  | Default  |
| ---------------------------------------- | ------------------------------------------------------- | -------- |
| `heading_style` / `headingStyle`         | `atx` (`# Title`), `setext` (underlined)                | `atx`    |
| `link_style` / `linkStyle`               | `inline`, `referenced`                                  | `inline` |
| `code_fence` / `codeFence`               | `` ` ``, `~`                                            | `` ` ``  |
| `bullet_marker` / `bulletMarker`         | `-`, `*`, `+`                                           | `-`      |
| `base_url` / `baseUrl`                   | URL used to resolve relative links and images           | none     |
| `exclude_selectors` / `excludeSelectors` | CSS selectors to drop                                   | none     |
| `include_selectors` / `includeSelectors` | CSS selectors kept even if an exclude selector matches  | none     |

## WebAssembly

The crate's `wasm` feature builds WebAssembly bindings with the same API; [`examples/wasm.html`](https://github.com/farhnkrnapratma/markers/blob/main/examples/wasm.html) shows them in a browser. The npm package does not include a WebAssembly build yet.

## Contributing

Bug reports and pull requests are welcome. [`CONTRIBUTING.md`](https://github.com/farhnkrnapratma/markers/blob/main/CONTRIBUTING.md) explains how to build, test and submit changes.

## License

Licensed under either of the [Apache License, Version 2.0](https://github.com/farhnkrnapratma/markers/blob/main/LICENSE-APACHE) or the [MIT License](https://github.com/farhnkrnapratma/markers/blob/main/LICENSE-MIT), at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
