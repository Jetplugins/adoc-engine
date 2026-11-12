# adoc-engine 0.6.0

Rust integrations can now select the rendering backends they need. HTML, JSON and
semantic analysis work without the Typst publication stack; full defaults and the
official CLI keep every output format, native equations and embedded publication fonts.

- Add optional `mathml`, `epub`, `highlighting`, `pdf`, `svg` and `png` Cargo
  features. EPUB includes native MathML; highlighting includes EPUB.
- Add `supported_formats()` and actionable errors naming the feature required by
  a disabled export, native HTML math request or EPUB highlighting request.
- Make the vendored PDF compiler optional and use one `typst-as-lib` 0.17 version.
- Test nine feature configurations and isolated downstream consumers in CI,
  including dependency boundaries and the original vendor PDF API.

For an HTML/JSON-only library integration:

```toml
[dependencies]
adoc-engine = { git = "https://github.com/Jetplugins/adoc-engine", tag = "v0.6.0", default-features = false }
```

Add `features = ["mathml"]` for native HTML equations, `features = ["epub"]` for
books, or select the publication formats you need. See the
[feature guide](https://github.com/Jetplugins/adoc-engine/blob/v0.6.0/docs/features.md).

The recorded macOS ARM64 consumer measurement used 48 normal dependency packages
for core versus 377 for full, with 4.80 versus 51.45 MiB executables and 25.23 versus
161.22 seconds for clean release builds. These are single-run consumer measurements,
not CLI sizes or runtime conversion benchmarks. See the
[raw results and methodology](https://github.com/Jetplugins/adoc-engine/tree/v0.6.0/benchmarks/baselines/dependency-scope).

Native LaTeX/AsciiMath rendering and existing HTML, language-server and publication
behavior are retained. Release packages are verified for macOS ARM64/x64,
Linux ARM64/x64 and Windows x64, with checksums, dependency/font notices,
source provenance and a release manifest.
