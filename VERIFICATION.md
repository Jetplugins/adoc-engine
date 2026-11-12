# Release verification: 0.5.0

[Version 0.5.0](https://github.com/Jetplugins/adoc-engine/releases/tag/v0.5.0)
was published on 2026-11-12 UTC from source commit
`e1dcd732afebfff5ebb324a1baadfa00dea559e6` after
[main CI](https://github.com/Jetplugins/adoc-engine/actions/runs/43180586197) and
[the release workflow](https://github.com/Jetplugins/adoc-engine/actions/runs/43180616912)
passed, including all five native platforms and publication verification.

The published manifest, all five checksum files and GitHub asset digests match.
The macOS ARM64 archive was independently downloaded and its archive/binary
fingerprints and source provenance verified. Its relocated executable passes
five-format execution with empty PATH, and the actual published binary also
passes the ten-equation mixed LaTeX/AsciiMath fixture through all six visual
formats, with EPUBCheck and PDF text verification. See the
[release receipt](docs/releases/0.5.0-release-verification.json).

Version 0.5.0 adds native LaTeX and AsciiMath equations to paginated exports,
static LaTeX MathML to EPUB, and opt-in native MathML to HTML. The
[STEM guide](docs/stem.md) documents supported commands and format behavior.

The [release workflow](.github/workflows/release.yml) must pass native tests,
worker/LSP protocol checks and relocated archive execution on macOS ARM64/x64,
Linux ARM64/x64 and Windows x64, plus independent publication/reference checks,
before publishing any packages. Release checksums and BUILD.json provenance are
bound to the tagged source commit by the release manifest.

Local release gates passed on macOS ARM64:

- 48 Rust workspace tests, strict Clippy, formatting and downstream library compilation.
- Six worker/LSP protocol checks and 24 Python benchmark/EPUB/release tests.
- The 23 HTML corpora and 20 editing variants against Ruby Asciidoctor 2.0.26
  and Asciidoctor.js 4.2.1, including the existing JavaScript width equivalence.
- Ten mixed equations (four inline, six display) through HTML, EPUB, PDF, SVG,
  merged SVG and PNG. Static MathML is inspected, the EPUB passes EPUBCheck 5.4.0,
  and real PDF text confirms Greek symbols and absence of unrendered commands.
- LaTeX aliases and an explicit LaTeX block inside an AsciiMath document, validated
  through actual PDF text extraction. The PDF equation page was also visually reviewed.
- Existing PDF text/page, EPUB, diagram and SVG/PNG/JSON publication checks.
- Updated third-party notices for the native MathML and AsciiMath dependencies.

[The STEM export check](tests/stem.py) runs in the publication release job.
The default HTML contract remains unchanged unless `stem-renderer=native` is set.
Reader/browser MathML rendering and translator command subsets remain documented
limitations; this is not a complete AsciiDoc TCK or TeX implementation claim.

Historical reports: [0.4.0](docs/releases/0.4.0-verification.md),
[0.3.0](docs/releases/0.3.0-verification.md),
[0.2.0](docs/releases/0.2.0-verification.md),
[0.1.0](docs/releases/0.1.0-verification.md).
