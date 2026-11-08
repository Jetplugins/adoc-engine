# Release verification: 0.3.0

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
