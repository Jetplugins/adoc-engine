# Changelog

## Unreleased

## 0.6.0

- Add optional EPUB, highlighting, native MathML, PDF, SVG and PNG library features;
  preserve full defaults and explicitly enable all backends in the CLI.
- Expose `supported_formats()` and actionable errors for disabled capabilities.
- Make the vendored PDF compiler optional and align Typst wrappers on 0.17.
- Test each feature configuration and isolated downstream dependency boundaries;
  add reproducible cold-build, dependency-count and binary-size measurements.

## 0.5.0

- Enable native LaTeX and AsciiMath equations in PDF, SVG and PNG exports.
- Add LaTeX MathML to EPUB and opt-in native MathML HTML (`stem-renderer=native`).
- Align publication notation aliases/overrides with HTML and correct `\hbar`.
- Verify ten mixed equations across all visual exports, including EPUBCheck and
  PDF text extraction, in the publication release workflow.

## 0.4.0

- Update SHA-256, Base64, ZIP and Typst library dependencies.
- Preserve deterministic EPUB identifiers with the updated SHA-256 API.
- Update the JavaScript compatibility reference to Asciidoctor.js 4.2.1, adapt
  the worker to its asynchronous API and report the configured reference version.
- Refresh GitHub Actions, Node.js 26 and Ubuntu CI/release tooling.

## 0.3.0

- Expand EPUB 3 books: book/article structure, ID-based chapter filenames, configurable
  split depth, nested reader navigation and NCX, independently controlled visible TOC.
- Preserve metadata, CSV subjects, series, covers, front matter, local styles/fonts,
  audio/video and cross-chapter references/footnotes.
- Add shared reader-friendly typography, licensed fonts with opt-out, static native
  syntax highlighting, Unicode callouts and AsciiMath MathML.
- Stage file exports before replacement and validate completed books independently
  with EPUBCheck and a pinned Asciidoctor EPUB3 reference.
- Honor API attribute soft defaults and unsets and EPUB backend conditionals.

## 0.2.0

- Preserve separate titled checklists after nested lists, including the preceding
  list's ordinary bullet markers.
- Keep admonitions out of example numbering and honor caption overrides.
- Correct bibliography/special-section headings, TOC entries, references, and
  following section counters; preserve explicit `sectnums=all` behavior.

- Keep list items following a continuation-attached admonition as separate
  items in HTML, EPUB and publication exports, including styled admonition
  paragraphs, multiline content and returns to an ancestor list.

## 0.1.0

- Native AsciiDoc library, CLI, persistent worker, and stdio LSP.
- HTML, EPUB 3, PDF, SVG, merged SVG, PNG, and structured JSON exports.
- Unsaved include overlays, Antora resolver aliases, include source maps, and supplied SVG diagrams.
- Cancellable bounded native workers, stale-result rejection, UTF-16 synchronization, semantic navigation.
- Native compound table-cell and semantic HTML passthrough publication rendering.
- Compatibility, adversarial protocol, publication, and three-engine performance verification.
