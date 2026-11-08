# Third-party components

Authored project code is MIT. Dependency licenses are independent and collected in
THIRD_PARTY_LICENSES.html with cargo-about from the locked dependency graph.
FONT_LICENSES contains the upstream typst-assets consolidated font notices and license.

- asciidoc-parser / asciidoc-html5: MIT OR Apache-2.0 (asciidoc-rs).
- adocers-typst / adocers-render-core: MIT OR Apache-2.0 (Alexander Thaller).
- The vendored parser and publication emitter retain their original copyright/license files;
  modifications are described in vendor/PATCHES.md.
- Typst and its libraries: Apache-2.0; typst-as-lib: MIT OR Apache-2.0.
- EPUB static math: mathemascii and alemat (Apache-2.0); syntax highlighting: Syntect (MIT).
- EPUB fonts: unmodified Libertinus Serif (OFL-1.1), DejaVu Sans Mono (DejaVu/Bitstream Vera);
  the consolidated notice is also embedded in each book using these fonts.
- Embedded fonts retain their individual licenses as listed in FONT_LICENSES.

Regenerate notices with `cargo about generate about.hbs -o THIRD_PARTY_LICENSES.html`
and `python3 scripts/font_licenses.py` before packaging changed dependencies.
