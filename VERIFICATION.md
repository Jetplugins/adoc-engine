# Release verification: 0.3.0

[Version 0.3.0](https://github.com/Jetplugins/adoc-engine/releases/tag/v0.3.0)
was published on 2026-11-08 UTC from source commit
`9e7043bfb0af82a47b95cfd68569c67ec5967e42` after the
[release workflow](https://github.com/Jetplugins/adoc-engine/actions/runs/42489742703)
passed all five native targets and publication verification.

The published macOS ARM64 archive was independently downloaded, checksum/manifest
and BUILD.json verified, relocated, and executed in five formats with empty PATH.
The IntelliJ consumer also downloaded it without authentication through its IDE
HTTP client, installed it and rendered a document. Its archive SHA-256 is
`cdca125ccf0c43817a7cb92e42dd1160040d823d760e2db8dfd9ebc67e402b89`.
The published binary's complete feature book passes EPUBCheck and has the same
SHA-256 as the locally verified book. The final consumer CI passed 1,369 unit tests,
export checks, the actual packaged-plugin EPUB smoke test and four IDE verifiers.
See the [release receipt](benchmarks/baselines/v0.3.0/release-verification.json) and
[published-book checks](benchmarks/baselines/v0.3.0/published-book.json).

The EPUB release is verified against the pinned Asciidoctor EPUB3 2.3.0 reference,
Asciidoctor 2.0.26, asciimath 2.0.6 and Rouge 5.1.0. The complete comparison,
archive fingerprints, raw timings and validator output are in the
[0.3.0 evidence](benchmarks/baselines/v0.3.0).

Local release gates:

- 41 Rust workspace tests, strict Clippy and formatting; separate downstream library build.
- Six executable worker/LSP protocol tests, including cancellation, Unicode edits,
  unsaved include overlays and lifecycle behavior.
- All 23 existing Ruby/JavaScript/Rust HTML corpora and 20 editing variants retain
  strict structural parity; the complete showcase regression remains byte-for-byte.
- Seven Rust EPUB variants (including a generated larger book) and six Kotlin books
  pass EPUBCheck 5.4.0 and independent ZIP/XML, manifest/spine, navigation, local
  resource/fragment/CSS link, metadata, Unicode, highlighting and static math checks.
- The independent checker has deliberate corruption tests for missing anchors,
  images/fonts and duplicate IDs. Reference defects remain recorded separately.
- PDF text and two rendered pages, SVG/merged SVG/PNG page counts, JSON and supplied
  diagrams pass the existing Poppler/publication checks.
- The IntelliJ consumer passes 1,369 unit tests on the final sources, with fresh
  compilation and file watching disabled for the affected code. The obfuscated
  plugin distribution passes resource/template/dependency packaging checks.
- Browser review of actual extracted EPUB chapters verifies typography, checklist
  marks/callouts, tables, math, chapter links and footnote navigation. The review
  caught icon-font-dependent marks and led to explicit Unicode regression checks.

The [release workflow](.github/workflows/release.yml) builds and tests all five
native targets (macOS ARM64/x64, Linux ARM64/x64 and Windows x64), verifies relocated
archives, then runs independent publication/reference checks before publishing.
A release manifest binds archive checksums to the tested commit and worker protocol.
This workflow is required to succeed before publication; local measurements alone
are not cross-platform evidence.

The shared Kotlin/Rust EPUB theme, archive checker and source fixtures are checked
for byte-identical copies. Kotlin publication libraries have SHA-256 provenance,
redistribution notices and bounded offline JVM execution. Embedded font notices
accompany the distribution and each book containing those fonts.

Compatibility boundaries, reader-dependent MathML/media behavior, full-font rather
than subset embedding, highlighter differences and unsupported arbitrary extensions
are explicit in the [EPUB guide](docs/epub.md). This is not a complete AsciiDoc TCK claim.

Historical reports: [0.2.0](docs/releases/0.2.0-verification.md),
[0.1.0](docs/releases/0.1.0-verification.md).
