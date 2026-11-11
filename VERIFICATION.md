# Release verification: 0.4.0

[Version 0.4.0](https://github.com/Jetplugins/adoc-engine/releases/tag/v0.4.0)
was published on 2026-11-11 UTC from source commit
`6f31608a453f93b4afe5617b2dd56a3032915eb7` after the
[release workflow](https://github.com/Jetplugins/adoc-engine/actions/runs/43006851867)
passed all five native platforms and publication checks.
[Main CI](https://github.com/Jetplugins/adoc-engine/actions/runs/43006815451) also passed.

The public manifest and all five checksum files match the published asset sizes
and source commit. The macOS ARM64 archive was independently downloaded, its
archive and binary fingerprints verified, and its executable relocated and run
in five formats with empty PATH. See the
[release receipt](docs/releases/0.4.0-release-verification.json).

This release combines Renovate PRs #7–#17, updates engine dependencies and build
infrastructure, and adapts EPUB identifiers and the JavaScript benchmark worker
to the updated dependency APIs.

Local release gates passed on macOS ARM64:

- 42 Rust workspace tests, strict Clippy and formatting; downstream library-consumer check.
- 24 Python tests for benchmark comparison, EPUB validation and release packaging.
- Six executable worker/LSP protocol tests.
- All 23 HTML corpora and 20 editing variants, showcase compatibility and 111
  list-continuation cases against Ruby Asciidoctor 2.0.26 and Asciidoctor.js 4.2.1.
- Rust-to-Ruby HTML parity stays strict. The JavaScript comparison alone recognizes
  equivalent percentage column-width attributes and CSS declarations, with tests
  that reject different widths, conflicting styles and changes on other elements.
- PDF text checks and two rendered pages; HTML, SVG, merged SVG, PNG and JSON exports;
  supplied diagrams; atomic EPUB replacement and failure preservation.
- Six complete EPUB book variants pass EPUBCheck 5.4.0 and independent archive,
  resource, link, navigation and metadata checks. Existing Ruby reference defects
  remain recorded separately and do not relax native validation.
- Third-party dependency and embedded-font notices regenerated from the lockfile.

The [release workflow](.github/workflows/release.yml) must pass native tests,
protocol checks and relocated archive verification on macOS ARM64/x64, Linux
ARM64/x64 and Windows x64, plus publication/reference verification, before it
publishes. Release assets include checksums, BUILD.json provenance and a manifest
binding every archive to the tested source commit. Workflow results are available
in [GitHub Actions](https://github.com/Jetplugins/adoc-engine/actions/workflows/release.yml).

Historical reports: [0.3.0](docs/releases/0.3.0-verification.md),
[0.2.0](docs/releases/0.2.0-verification.md),
[0.1.0](docs/releases/0.1.0-verification.md).
