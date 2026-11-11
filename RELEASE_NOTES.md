# adoc-engine 0.4.0

This maintenance release refreshes the engine dependencies and build tooling.

- Update SHA-256 (sha2 0.11), Base64 (0.23), ZIP (9.5.1) and typst-as-lib (0.17).
- Preserve existing deterministic EPUB book identifiers with the new SHA-256 API,
  with regression coverage for generated identifiers and explicit UUIDs.
- Update the JavaScript compatibility reference to Asciidoctor.js 4.2.1 and migrate
  the benchmark worker to its asynchronous API. JavaScript parity checks recognize
  equivalent table-column width markup; Rust-to-Ruby checks remain strict.
- Refresh checkout, setup and artifact GitHub Actions, Node.js 26 and Ubuntu tooling.

The library, CLI and language-server interfaces retain the 0.3.0 feature set,
including native HTML, PDF, EPUB 3, SVG, PNG and JSON exports.

Downloads include dependency/font notices, SHA-256 checksums, build provenance
and a release manifest. The release workflow verifies five native platforms and
publication compatibility before publishing the packages.
