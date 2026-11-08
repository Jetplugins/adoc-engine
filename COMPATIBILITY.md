# Verified compatibility contract

Reference engines are Ruby Asciidoctor 2.0.26 and Asciidoctor.js 3.0.4. The release
performance corpus also acts as a differential HTML verification corpus. Every
eligible case must match Ruby's normalized HTML structure, attributes, text, links,
and code whitespace. Mismatches fail the gate and are saved for inspection.

| Capability | Verification |
|---|---|
| Metadata, sections, attributes, inline substitutions | Differential fixtures and native tests |
| Tables, nested lists, footnotes, admonitions, code | Differential fixtures and publication content checks |
| Includes, selection, conditionals, overlays | Differential fixtures and cross-file protocol tests |
| Root confinement, hostile paths, passthrough assets | Native asset-policy tests |
| HTML fragment and standalone | Native and differential tests |
| PDF | Poppler text/page checks and visual page inspection |
| EPUB 3 | EPUBCheck 5.4.0, XHTML/OPF parsing, manifest/container tests |
| SVG, merged SVG, PNG | XML/signature checks and page count consistency |
| Compound table cells and semantic HTML passthrough | Native tests and PDF text checks |
| Supplied SVG diagrams across all backends | Cross-format native tests |
| LSP lifecycle, malformed requests, Unicode, edits | Adversarial executable tests |
| Cancellation, stale results, worker recovery | Adversarial executable tests |
| Cross-file completion/hover/definition | Shared parser index over the include graph and unsaved overlays |
| IntelliJ native include preparation and transport | Real-filesystem integration tests |

## Explicit boundaries

This is a versioned release, not a claim of complete AsciiDoc TCK certification.
The upstream parser's remaining language differences are inherited. Safe mode uses
canonical paths and intentionally refuses symlinks that escape the allowed root,
which is stricter than Ruby's lexical safe-mode jail. Includes currently accept UTF-8.
Ruby and JS extension execution is excluded; native integrations supply attributes,
resolver aliases, unsaved files, and diagram assets instead. Remote fetching is disabled.
PDF preserves semantic passthrough content, not arbitrary browser CSS or JavaScript.
Publication themes and page breaks need not match Asciidoctor PDF's exact layout.
Language navigation covers the active document and included documents; it does not
scan unrelated workspace files or implement document rename/refactoring.
