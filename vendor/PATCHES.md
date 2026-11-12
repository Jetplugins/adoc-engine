# Vendored dependency patches

adocers-typst 0.3.2 is vendored under its original MIT OR Apache-2.0 license.
Changes: recursively render AsciiDoc table cells; preserve semantic HTML passthrough
text, emphasis, and links through the existing inline translator.
Arbitrary browser CSS and JavaScript have no paginated equivalent and are not executed.

Further release fixes: explicit destinations for non-section block anchors; controlled
resource maps prevent filesystem reads in the emitter; supplied SVG diagrams are
embedded without invoking an external diagram runtime.

## Parser: continued admonitions

asciidoc-parser 0.31.2 is vendored from the crates.io package (upstream commit
`c4ea28b3cd6e3a310d123eebeddb991d882f865f`, `parser/`) under its original
MIT OR Apache-2.0 license. Original sources, tests, README and license texts
are retained. Explicit path dependencies make every backend and downstream library consumer
use this same parser instance, without depending on a root-only Cargo patch.

The block dispatcher now passes its enclosing list markers and continuation
state into the admonition parser. Simple admonition paragraphs, including the
`[TIP]` style form, use the existing list-aware paragraph parser so sibling and
ancestor list items terminate their content. Top-level and delimited compound
admonitions retain their existing parsing behavior. This fixes list structure
for HTML, EPUB and Typst publication exports without rewriting source or HTML.

Regression coverage: `crates/engine/tests/list_continuations.rs` and
`benchmarks/fixtures/list-continuations.adoc`.

The dependency bridge crates `asciidoc-html5` 0.2.3 (upstream commit
`4b3eb03fa7933f2fc053d1591b2af5aefdc44a4d`, `html5/`) and
`adocers-render-core` 0.3.2 (upstream commit
`2465e6a8a22a509247cfb8df5e1595738869f18b`, `crates/adocers-render-core/`)
are retained under their original MIT OR Apache-2.0 licenses. Their manifests point
to the local parser; render-core imports are formatted for repository checks.
Rendering behavior is unchanged.
`adocers-typst` likewise points to these local parser/render-core dependencies.

## Lists, captions, and special-section numbering

Nested list items preserve a blank-separated metadata boundary so the following
list owns its title and checklist state. Title lookahead avoids applying title
substitutions. Continuation-attached blocks retain their existing behavior.

Example/open/paragraph containers styled as admonitions do not consume example
caption counters. Admonitions also honor their explicit caption label override.

Bare `sectnums` enables ordinary section numbering; explicitly assigning `all`
also enables special sections. Bibliography and other special sections preserve
their unnumbered scope and leave following sibling counters unchanged. Numbered
special-section cross-references omit the ordinary section label. The native
publication numbering helper honors sections explicitly numbered by `all`.

Regression coverage: `crates/engine/tests/showcase_compatibility.rs`,
`tests/showcase_compatibility.py`, and the unchanged complete source/asset fixture
in `benchmarks/fixtures/showcase`.

The showcase's `expected.html` is the byte-identical output of pinned Ruby
Asciidoctor 2.0.26 and Asciidoctor.js 3.0.4. A normal Rust integration test compares
the complete document against it (allowing only the final newline), so CI checks
all showcased features without installing reference runtimes.

## EPUB processing defaults

`asciidoc-html5::Options::doctype_default` supplies a book default before API and
header directives, without changing normal HTML's article default. The EPUB
backend uses `processing_backend("epub3")` so backend conditionals are evaluated
for the publication target. Explicit API/header doctype values still win.
The engine's API attribute adapter now handles Asciidoctor's `@` soft defaults
and `!` unsets rather than passing their control characters as literal values.

## Native STEM rendering

The engine enables adocers-typst's math option and AsciiMath feature. Its notation
selection now recognizes `latex`/`tex` aliases and `[stem,notation]` overrides,
matching the HTML backend. MiTeX's old `planck.reduce` path maps to Typst's
`planck` (the reduced Planck constant). An initial escape that triggers an
upstream AsciiMath tokenizer underflow retains its source instead.

The HTML renderer suppresses its MathJax footer only when `stem-renderer=native`
is selected. Its default output stays reference-compatible. The engine performs
the native MathML conversion; this patch does not add a script runtime.
