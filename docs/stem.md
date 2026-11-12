# STEM equations: LaTeX and AsciiMath

The engine renders math without a LaTeX installation, Node, MathJax, a browser
process or runtime package downloads. This support is for equations embedded in
AsciiDoc, not complete `.tex` documents or arbitrary LaTeX packages.

## Write equations

Select the default notation with `:stem: latexmath` (`latex` and `tex` are aliases).
Use `stem:[...]` inline and a `[stem]` passthrough block for a display equation.
Explicit `latexmath:[...]`, `asciimath:[...]`, `[latexmath]` and `[asciimath]` forms
can mix both notations in one document. A block such as `[stem,latexmath]`
overrides the document's default notation. With no value for `stem`, the default
notation is AsciiMath.

```asciidoc
= Equations
:stem: latexmath
:stem-renderer: native

Inline stem:[\frac{a}{b}] and asciimath:[sqrt(x^2+y^2)].

[stem]
++++
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
++++

[latexmath]
++++
\begin{pmatrix}a & b \\ c & d\end{pmatrix}
++++
```

Normal AsciiDoc macro escaping still applies; a closing `]` inside an inline
macro must be escaped. Block equations avoid that extra escaping.

## Output formats

| Output | Native equation rendering |
| --- | --- |
| PDF, SVG, merged SVG, PNG | Both notations, enabled by default; MiTeX translates LaTeX to Typst and the bundled AsciiMath translator handles AsciiMath. Fonts are embedded in the engine. |
| EPUB 3 | Both notations become static, accessible MathML, with inline/display mode and `mathml` manifest properties. No script execution. |
| HTML / JSON HTML field | Set `:stem-renderer: native` or pass `-a stem-renderer=native` to emit static MathML. The default HTML mode preserves Asciidoctor-compatible MathJax delimiters and its standalone MathJax loader. |

`stem-renderer` affects HTML only. PDF and EPUB always render equations natively.
Standalone native HTML includes basic math layout CSS and omits the MathJax
loader. For HTML fragments, the embedding page can include
[the math stylesheet](../crates/engine/src/math.css). EPUB includes this stylesheet
in its publication theme. MathML appearance and font availability depend on the
browser or EPUB reader.

```sh
adoc-engine render equations.adoc --standalone -a stem-renderer=native -o equations.html
adoc-engine render equations.adoc -f pdf -o equations.pdf
adoc-engine render equations.adoc -f epub -o equations.epub
```

## Supported scope and failures

The verified examples cover fractions, roots, sums, integrals, Greek symbols,
subscripts/superscripts, matrices, aligned equations, and the reduced Planck
constant (`\hbar`). Code and literal blocks remain source text.

LaTeX support follows the command subsets of [MiTeX](https://github.com/mitex-rs/mitex)
for paginated output and [math-core](https://github.com/tmke8/math-core) for MathML.
These are native Rust libraries, not a full TeX interpreter. They can differ in
command coverage and layout; arbitrary packages, document preambles, external
files and equation numbering/cross-reference workflows are not supported by this
integration. Unsupported MathML equations return an error, including the LaTeX
parser's explanation. MathML expressions are limited to 32 KiB. The Typst emitter
retains source text for expressions its translator cannot convert; a subsequent
Typst compilation error is returned to the caller.

## Verification

[The equation fixture](../tests/fixtures/stem/equations.adoc) is exercised through
all six visual output formats by [the export check](../tests/stem.py). It checks
static MathML, EPUBCheck validity, actual PDF text (including absence of raw
LaTeX commands), SVG XML and PNG generation. Rust integration tests additionally
cover the default HTML compatibility mode, API attributes, code preservation,
invalid equations and size limits.

```sh
cargo test --workspace --locked --test stem
python3 tests/stem.py target/debug/adoc-engine --epubcheck /path/to/epubcheck.jar
```

The publication release job runs the export check before publishing binaries.
