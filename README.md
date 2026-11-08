# adoc-engine

A fast, MIT-licensed Rust AsciiDoc engine with a library, CLI, and stdio language server.
Render HTML, PDF, EPUB 3, SVG, merged SVG, PNG, and JSON without installing Ruby,
Node, a browser, fonts, or an external typesetter. Node and Ruby are used only for
reference compatibility and performance verification.

## Install and run

Download the archive for your platform from [Releases](https://github.com/Jetplugins/adoc-engine/releases),
verify its SHA-256 checksum, and put `adoc-engine` on your PATH. macOS users may need
to authorize the unsigned executable through their operating system's normal security UI.
Build from source with Rust 1.96 or newer:

```sh
cargo build --release --locked
target/release/adoc-engine render document.adoc --standalone --output document.html
target/release/adoc-engine render document.adoc --format pdf --output document.pdf
target/release/adoc-engine render document.adoc --format epub --output document.epub
target/release/adoc-engine render document.adoc --format svg --output pages
target/release/adoc-engine render document.adoc --format svg-merged --output document.svg
target/release/adoc-engine lsp
```

`svg` and `png` write one artifact per page into the requested directory.
`svg-merged` produces one vertically joined SVG. Input can come from stdin;
`--base-dir`, `--safe-mode`, and repeated `--attribute name=value` configure conversion.
Safe mode defaults to `secure`, which disables filesystem includes and image embedding.
Use `safe` for local assets confined to the document directory; `unsafe` explicitly
allows access outside it. Network resource fetching is disabled.

## Library and structure

`crates/engine` contains the public API, include resolver, shared semantic analysis,
HTML presentation, EPUB packaging, and publication exports. `crates/cli` contains
CLI commands and the LSP. `vendor/adocers-typst` contains a narrowly patched,
licensed publication emitter; see [patch notes](vendor/PATCHES.md).

```rust
let result = adoc_engine::render("= Hello\n\nA *fast* document.", &Default::default())?;
println!("{}", result.html);
# Ok::<(), anyhow::Error>(())
```

`RenderOptions` accepts attributes, unsaved file overlays, include resolver aliases,
and pre-rendered SVG diagrams keyed by source. Render results include diagnostics,
include dependencies, and source mappings. All publication backends consume the same
parsed document model. Native integrations can customize source, options, and supplied
assets without executing scripting runtimes.

## Editor integration

The stdio LSP implements UTF-16 incremental synchronization, diagnostics, semantic
headings, completion, hover, and definitions across the current document and its
include graph. Two persistent native workers keep the event loop responsive;
cancellation terminates active work, workers restart after faults, and edits reject
stale results. Render/export requests use immutable snapshots of open buffers.
See [protocol](docs/protocol.md) for custom requests and lifecycle behavior.

The IntelliJ AsciiDoc Pro adapter passes original include directives, unsaved
snapshots, configuration, Antora aliases, and supplied diagrams to the engine. It
retains its existing PSI features and offers native preview and publication exports.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked
python3 tests/protocol.py target/release/adoc-engine
python3 tests/publications.py target/release/adoc-engine --epubcheck /path/to/epubcheck.jar
```

Publication verification uses EPUBCheck 5.4.0 plus Poppler text extraction and page
rendering. Protocol tests cover malformed requests, atomic invalid edits, Unicode,
unsaved includes, semantic navigation, cancellation, stale responses, and recovery.
Release jobs run native tests and protocol tests on macOS ARM64/x64, Linux ARM64/x64,
and Windows x64 before creating release archives. Every archive includes checksums,
third-party licenses, embedded font notices, and build provenance.

### Reproduce the three-engine benchmark

```sh
cd benchmarks
npm ci --ignore-scripts
bundle install
python3 run.py --update-readme
```

This suite covers generated document sizes through 1 MiB, sections, attributes,
conditionals, Unicode, tables, nested lists, footnotes, admonitions, source code,
selected/nested includes, and successive edit buffers. Structural HTML parity is
checked against Ruby before any case qualifies for a speed comparison. A mismatch
fails verification and preserves the outputs for inspection.

Warm conversion, cold process startup, edit conversions, throughput, p95 latency,
variation, and separate peak-process memory are recorded. JS runs under Node;
JCEF painting and plugin UI latency are not part of the core comparison. Run
`benchmarks/compare.py` on same-machine raw reports to flag regressions above 10%;
confirm flagged results with an isolated repeat before accepting or rejecting a release.

## Compatibility contract

The native parser and HTML backend target Ruby Asciidoctor 2.0.26; reference JS is
Asciidoctor.js 3.0.4. See [verified coverage and boundaries](COMPATIBILITY.md).
This release does not claim full AsciiDoc TCK certification. Ruby/JS extensions,
remote resource fetching, arbitrary browser CSS/JavaScript in PDF, and unrelated
workspace-wide symbol search are outside the native contract. Semantic HTML
passthrough text/emphasis/links and compound AsciiDoc table cells are supported in
publication output. Publication typography and pagination use a dedicated theme.

Authored project code is MIT licensed. Dependencies retain their licenses:
[complete license notices](THIRD_PARTY_LICENSES.html), [font notices](FONT_LICENSES),
and [inventory](THIRD_PARTY.md).
