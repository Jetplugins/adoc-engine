# adoc-engine 0.2.0

- Fix continued lists, separate titled checklists, example counters, and bibliography numbering and cross-references.
- Add complete-showcase golden coverage and 270 focused reference comparisons.
- Publish verified Mermaid performance charts and cross-platform binary packages.
- Build and attach archives automatically from GitHub Releases, or publish after verification using the Release engine workflow.

Native MIT AsciiDoc engine for libraries, command-line use, and editor integration.

Supports HTML, PDF, EPUB 3, SVG, merged SVG, PNG, and JSON; includes a stdio LSP and
native unsaved include resolution. Runtime conversions require no Ruby, Node, browser,
or external typesetter. Binary archives contain licenses, embedded font notices, and
build metadata; SHA-256 checksums accompany every archive.

See README.md for measured JS/Ruby/Rust performance and COMPATIBILITY.md for the
verified feature contract. This release does not claim full AsciiDoc TCK certification
or support for executing Ruby/JavaScript extensions or browser CSS in paginated output.
