# Changelog

## Unreleased

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
