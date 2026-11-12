# Optional library backends

These features are available from version 0.6.0.

The library keeps `full` enabled by default, preserving the existing API and all
output formats. The CLI explicitly enables `full`, including native equations,
highlighting and embedded publication fonts. Disabling default library features
provides HTML, JSON, semantic analysis, include handling, diagnostics, source maps
and supplied SVG diagrams without the Typst publication stack.

| Feature | Additional capabilities |
| --- | --- |
| `mathml` | Native LaTeX/AsciiMath MathML for HTML; no Typst dependency |
| `epub` | EPUB 3 packaging, metadata, local assets and fonts; includes `mathml` |
| `highlighting` | EPUB syntax highlighting with Syntect; includes `epub` |
| `pdf` | Native PDF and paginated LaTeX/AsciiMath through Typst/MiTeX |
| `svg` | Native per-page SVG and merged SVG through Typst |
| `png` | Native per-page PNG through Typst |
| `full` | All capabilities above; enabled by default |

`typesetting` groups shared implementation dependencies. Select `pdf`, `svg` or
`png` for usable output formats. Cargo features are additive: enabling `full`
through another dependency enables its backends for that resolved engine instance.

## Configure a library consumer

An HTML/JSON-only dependency on the Git release is:

```toml
[dependencies]
adoc-engine = { git = "https://github.com/Jetplugins/adoc-engine", tag = "v0.6.0", default-features = false }
```

Add `features = ["mathml"]` for static HTML equations, `features = ["epub"]`
for books, or `features = ["highlighting"]` for books with syntax highlighting.
Features can be combined, for example `features = ["pdf", "epub"]`.
Paginated math does not require the separate HTML MathML feature.

The `render`, `analyze`, `export` and `RenderOptions` interfaces remain available
in every configuration. Query compiled output formats before presenting export
choices:

```rust
for format in adoc_engine::supported_formats() {
    println!("{format}");
}
```

Formats use a stable order: HTML, PDF, EPUB, SVG, merged SVG, PNG, JSON, filtered
to the enabled backends. HTML and JSON are always present; `svg` enables both
`svg` and `svg-merged`.

A disabled format returns an error naming its Cargo feature before parsing input
or reading assets. A native HTML math request without `mathml`, or an EPUB
`source-highlighter` request without `highlighting`, also returns an actionable
error. Features are never silently replaced with a different rendering mode.
Default MathJax-compatible HTML remains usable without `mathml`.

## Dependency boundaries

HTML, native MathML and EPUB consumers do not depend on Typst. The vendored
publication emitter can generate Typst markup without linking its own PDF
compiler. That compiler is an optional vendor feature; the engine compiles
publications through one shared `typst-as-lib` 0.17 instance.

Output features control our export implementations and direct dependencies.
Typst itself shares libraries between formats, so an individual output feature
can still include transitive helpers associated with another format. For example,
Typst's own HTML support uses SVG internally. The isolated consumer checks enforce
that non-PDF configurations do not include `typst-pdf`.

Embedded fonts remain part of the publication backends. Core-only builds omit
those backends and their font data, while official binaries retain self-contained
rendering without requiring fonts to be installed.

## Verification and measurements

CI tests and lints core, MathML, EPUB, highlighted EPUB, PDF, SVG, PNG, full and
default configurations, and independently compiles the original vendor PDF API
against wrapper 0.17. Each profile also builds an independent downstream
workspace. This catches feature-unification mistakes that can be hidden by
building the full CLI and library together.

```sh
cargo test -p adoc-engine --no-default-features --locked
cargo test -p adoc-engine --no-default-features --features epub --locked
cargo test --workspace --all-features --locked
python3 tests/feature_consumer.py --profile core
python3 tests/feature_consumer.py --profile svg
```

To measure dependency count, clean build time and executable size:

```sh
python3 tests/feature_consumer.py --profile core --measure --output /tmp/core.json
python3 tests/feature_consumer.py --profile full --measure --output /tmp/full.json
```

Each measurement uses a fresh temporary target directory and a dynamic-format
consumer that retains every enabled export backend. Dependencies are already
downloaded, and compiler wrappers are disabled for measured builds. Executable
size refers to the unstripped consumer harness, not the packaged CLI.
Release builds use thin LTO, one codegen unit and four compilation
jobs by default. The report records compiler, host, source fingerprint, lockfile,
features, executable hash and size. Build time excludes the subsequent runtime
checks. These are single-run observations, not general speed guarantees.

On the recorded macOS ARM64 run, core used 48 normal dependency packages and
produced a 4.80 MiB executable in 25.23 seconds, versus 377 packages, 51.45 MiB
and 161.22 seconds for full. These figures describe the consumer harness and the
single-run methodology above. Raw results and all five profiles are in
[the dependency-scope baseline](../benchmarks/baselines/dependency-scope).
