# adoc-engine 0.5.0

LaTeX and AsciiMath equations now render natively in publication exports, with
no LaTeX installation, Node runtime, browser process or runtime package downloads.

- Enable the bundled MiTeX/Typst equation renderer in PDF, SVG, merged SVG and PNG,
  including AsciiMath translation and embedded fonts.
- Convert LaTeX and AsciiMath to static, accessible MathML in EPUB 3, including
  inline/display modes, matrix compatibility and `mathml` manifest properties.
- Add opt-in static HTML MathML with `:stem-renderer: native` or
  `-a stem-renderer=native`. Standalone native HTML includes math layout CSS and
  omits the MathJax loader; default HTML preserves its Asciidoctor-compatible mode.
- Honor `latex`/`tex` notation aliases and positional block overrides, correct
  `\hbar`, and avoid an upstream AsciiMath tokenizer panic on initial escapes.

Select LaTeX with `:stem: latexmath`, then use expressions such as
`stem:[\frac{a}{b}]` or `[latexmath]` display blocks. Both notations can be mixed.
See the [STEM guide](https://github.com/Jetplugins/adoc-engine/blob/v0.5.0/docs/stem.md)
for examples and supported scope. This supports math equations, not complete TeX
documents or arbitrary packages; the native translators have different command
subsets and unsupported-expression behavior.

The release verification exercises ten mixed equations through all six visual
formats, checks real PDF text and EPUBCheck validity, and retains the existing
HTML compatibility, language-server and publication gates. Downloads include
checksums, dependency/font notices, source provenance and the release manifest.
