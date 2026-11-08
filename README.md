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

External Rust projects can use the verified Git release:

```toml
[dependencies]
adoc-engine = { git = "https://github.com/Jetplugins/adoc-engine", tag = "v0.1.0" }
```

```rust
fn main() -> anyhow::Result<()> {
    let result = adoc_engine::render("= Hello\n\nA *fast* document.", &Default::default())?;
    println!("{}", result.html);
    Ok(())
}
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

<!-- PERFORMANCE:START -->
## Verified performance

Measured 2026-11-08T04:12:11.962068+00:00 on Apple M2 Pro (macOS-26.5.1-arm64-arm-64bit-Mach-O).
Pinned Asciidoctor.js 3.0.4 (v26.8.2), Ruby Asciidoctor 2.0.26 (ruby 4.0.6 (2026-07-14 revision 03b6d3f889) +PRISM [arm64-darwin25]), and release-built Rust.
Each case: 10 warm-ups, 50 warm samples, 20 cold executions per engine. All 21 cases passed structural HTML parity against Ruby.

Warm median conversion times (milliseconds); excludes IPC and browser painting:

| Corpus | Input bytes | JS | Ruby | Rust | Ruby / Rust |
|---|---:|---:|---:|---:|---:|
| admonitions | 13963 | 16.052 | 5.225 | 0.787 | 6.64x |
| attributes | 20743 | 13.636 | 9.550 | 2.338 | 4.08x |
| basic | 83 | 0.423 | 0.146 | 0.061 | 2.40x |
| code | 298929 | 36.660 | 6.857 | 2.236 | 3.07x |
| conditionals | 20112 | 9.043 | 2.841 | 0.577 | 4.93x |
| features | 141 | 0.838 | 0.231 | 0.106 | 2.18x |
| footnotes | 8087 | 4.977 | 2.103 | 0.573 | 3.67x |
| nested-lists | 6397 | 17.457 | 4.019 | 1.047 | 3.84x |
| sections | 6397 | 15.723 | 4.527 | 1.131 | 4.00x |
| tables | 20859 | 59.993 | 11.072 | 5.165 | 2.14x |
| unicode | 30819 | 12.074 | 10.415 | 1.551 | 6.71x |
| paragraphs-100 | 5101 | 4.325 | 2.762 | 0.612 | 4.52x |
| paragraphs-1000 | 51901 | 42.863 | 27.954 | 5.186 | 5.39x |
| paragraphs-10000 | 528901 | 424.607 | 309.812 | 51.518 | 6.01x |
| bytes-1024 | 990 | 1.049 | 0.606 | 0.180 | 3.37x |
| bytes-10240 | 10224 | 8.166 | 5.173 | 1.085 | 4.77x |
| bytes-102400 | 102402 | 74.898 | 54.965 | 9.848 | 5.58x |
| bytes-1048576 | 1048590 | 1147.194 | 589.646 | 102.182 | 5.77x |
| includes | 36 | 0.472 | 0.161 | 0.182 | 0.88x |
| include-selection | 91 | 0.799 | 0.225 | 0.121 | 1.86x |
| repeated-edits | 30026 | 35.605 | 16.256 | 2.975 | 5.46x |

See [full report](benchmarks/baselines/release/report.md) and [raw samples and metadata](benchmarks/baselines/release/raw.json) for p95, variation, cold startup, repeated edits, peak RSS, and the executable checksum.
Results describe this corpus and machine; they are not universal speed guarantees.
The edit workload measures successive complete conversions in persistent processes; it does not claim incremental parsing.
<!-- PERFORMANCE:END -->

<!-- EXPORT-PERF:START -->
### Publication export performance

End-to-end CLI medians in milliseconds: 10 executions after two warm-ups per corpus/format. These include startup, layout, serialization, and filesystem writes with a warm OS cache; they are measured separately from HTML conversion.

| Format | Small book | Medium book |
|---|---:|---:|
| html | 11.42 | 11.02 |
| pdf | 39.72 | 41.37 |
| epub | 12.00 | 12.31 |
| svg | 39.78 | 50.17 |
| svg-merged | 40.28 | 46.83 |
| png | 58.64 | 155.34 |
| json | 12.03 | 12.44 |

[Raw publication samples and p95](benchmarks/baselines/release/exports.json).
<!-- EXPORT-PERF:END -->

<!-- LSP-PERF:START -->
### Language-server performance

End-to-end JSON-RPC render latency with persistent native workers, ten warm-ups and 50 measured requests per case. IPC and client response decoding are included.

| Paragraphs | Median ms | p95 ms |
|---|---:|---:|
| 10 | 0.32 | 0.39 |
| 1000 | 4.99 | 5.92 |
| 10000 | 49.44 | 53.92 |

[Raw LSP samples](benchmarks/baselines/release/lsp.json).
<!-- LSP-PERF:END -->
