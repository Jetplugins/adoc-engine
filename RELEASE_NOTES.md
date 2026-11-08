# adoc-engine 0.3.0

EPUB export now produces structured, reader-friendly books with rich metadata,
covers, configurable chapter splitting, nested navigation/NCX and separate visible
contents. Cross-chapter links and footnotes are relocated to valid targets.

Books can include front matter, local CSS and fonts, images/SVG, MP3/M4A/MP4 media,
static syntax highlighting and AsciiMath MathML. The shared Kotlin/Rust book theme
includes free Libertinus/DejaVu fonts with redistribution notices; `epub-fonts=none`
uses reader fonts. No Ruby, JavaScript or browser runtime is needed by the engine.

File exports are staged before replacing existing output. API soft attribute
defaults/unsets and EPUB backend conditionals are now honored.

The release verification adds independently parsed complete-book checks and
EPUBCheck comparisons with pinned Asciidoctor EPUB3 2.3.0. See the EPUB guide for
supported attributes, reference differences, reproducible verification and timing
methodology. The established HTML benchmarks remain separately labeled.

Downloads include MIT project licensing, dependency and embedded-font notices,
source commit and binary SHA-256 provenance, and a release manifest for managed
plugin installations.
