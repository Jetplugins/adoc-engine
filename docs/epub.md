# EPUB 3 books

Version 0.3 adds book-oriented EPUB output without Ruby, JavaScript, Pandoc or a
browser at runtime. The engine code remains MIT. Fonts and dependencies retain
separate redistribution notices, included in the native distributions.

```sh
adoc-engine render manuscript.adoc --format epub --safe-mode safe -o manuscript.epub
```

`secure` is the CLI default and prohibits filesystem assets/includes. Use `safe`
for a book whose includes and assets live under its base directory. The IntelliJ
plugin uses its selected safe mode and configured include/resource policy.

```asciidoc
= My Book
Jane Writer; John Reader
:doctype: book
:lang: en
:uuid: urn:uuid:7faf5703-6dec-4af2-8b5b-c7afe73abf03
:producer: Example Press
:copyright: Copyright 2026 Jane Writer
:description: An example technical book.
:keywords: EPUB, "books, publishing", Rust
:revdate: 2026-11-09
:imagesdir: images
:front-cover-image: image:cover.png[]
:toc:
:toclevels: 2
:outlinelevels: 3
:source-highlighter: rouge
:stem: asciimath

A short introduction.

include::chapters/introduction.adoc[]
include::chapters/reference.adoc[]
```

## Structure and metadata

Books are the publication default. Each level-one (`==`) section starts a new
XHTML chapter; title/preamble content uses `_preamble.xhtml`. Section IDs determine
chapter filenames. `epub-chapter-level` selects another split level. Explicit
`:doctype: article` keeps the document in `content.xhtml` and adds its author
byline, optional `{imagesdir}/avatars/{username}.jpg` avatar, and end mark.

Navigation includes an EPUB navigation document, landmarks, and nested NCX
fallback. `toc` adds visible contents; `toclevels` and `outlinelevels` independently
control visible and reader navigation depths. Cross references and footnotes are
relocated across chapters. Unresolved local references fail the export.

Metadata includes title, language, publication identifier, multiple authors,
producer/publisher, copyright/rights, description, CSV subjects, publication date,
source, and `series-name`, `series-volume`, `series-id`. Authors are creators with
`aut` roles and contributors; the tool/`creator` has the `mfr` role. The identifier
default is a SHA-256 URN derived from the title. Set an explicit `uuid` when book
identity must survive a title change.

## Presentation and assets

Both the Rust engine and Kotlin plugin exporter use the same reflowable theme.
It has relative text sizes, reader-controlled colors, sensible spacing, static
black-and-white syntax emphasis, Unicode callouts, captions, table borders and
page-break hints. Checklist marks and numbered callouts use Unicode, without an icon font dependency.
Reader support for MathML, media and CSS varies.

Eight unmodified font faces ship with the exporter: Libertinus Serif for prose
(SIL Open Font License 1.1) and DejaVu Sans Mono for code (DejaVu/Bitstream Vera
terms). They are free to use and embed; `font-licenses.txt` accompanies them inside
the book. `:epub-fonts: none` omits the bundled fonts and uses reader defaults.
The reference `scripts` values are accepted, but we embed full faces with
Latin/Greek/Cyrillic coverage, rather than selecting smaller subsets. Font files
account for most of a small book's size and export time.

PNG/JPEG/GIF/SVG and base64 data images are bundled. Supplied diagrams become static
SVG images. Local MP3, M4A and MP4 media are packaged. Assets obey canonical-path
and symlink boundaries and a 32 MiB per-resource limit. Remote assets must be
provided locally. External hyperlinks remain hyperlinks.

`epub3-frontmatterdir` includes `front-matter*.html` pages in alphabetical order;
otherwise a local `front-matter.html` is used when allowed. `epub3-stylesdir` loads
`epub3.css` and optional `epub3-css3-only.css`. Local `@import`, font and image URLs
are bundled relative to each stylesheet, with import cycle/depth checks. Put
custom fonts beneath the permitted resource directory and supply their licenses.

AsciiMath and LaTeX are converted to static MathML, inline and in blocks. Rust
uses native mathemascii and math-core; Kotlin uses bundled AsciiMath2Tex and KaTeX
on the JVM. See the [STEM guide](stem.md) for examples and command coverage. `source-highlighter` enables static
highlighting: Rust uses Syntect, Kotlin uses bundled highlight.js on the JVM.
Ruby highlighter-specific themes/options are not executed. No Node runtime or
browser is required by either exporter; arbitrary document scripts and extensions
are not executed in books. `epub-properties` accepts only static `svg`/`mathml`.

Exports are staged beside the destination before replacement. Failed exports
preserve existing files. `epub-modified=2026-11-09T00:00:00Z` or `SOURCE_DATE_EPOCH`
provides reproducible modification metadata; fixed inputs/assets then produce
byte-identical archives within each exporter. Cross-engine ZIP bytes differ.

## Comparison and verification

The baseline is [Asciidoctor EPUB3 2.3](https://docs.asciidoctor.org/epub3-converter/latest/).
The shared fixture exercises its documented publication features:

| Feature | Rust (current source) | Kotlin exporter | Reference EPUB3 2.3 |
| --- | --- | --- | --- |
| Book/article, section IDs and split level | Yes | Yes | Yes |
| Rich metadata, cover, series | Yes | Yes | Yes |
| Visible TOC + separate outline depth + NCX | Yes | Yes | Yes |
| Cross-chapter links and repeated footnotes | Validated | Validated | Repeated cross-chapter note target can be missing |
| Front matter, alternate CSS, local fonts | Yes | Yes | Yes |
| Reader typography, Unicode callouts, captions | Yes | Yes | Yes |
| Static highlighting | Syntect | highlight.js/JVM | Rouge/CodeRay/Pygments |
| Static math | AsciiMath + LaTeX | AsciiMath + TeX | AsciiMath |
| Local audio/video | Yes | Yes | Yes |
| Font subsets selected by `scripts` | Full faces | Full faces | Subsets |
| Arbitrary Ruby extensions/browser scripts | No | No | Ruby extensions |

This is feature-oriented publication compatibility, not a claim that Kotlin's
AsciiDoc parser or the Rust parser passes the entire Asciidoctor language suite.
Layouts, font families, highlighting tokenization and MathML trees can differ.

Reproduce archive checks with Python, Java, EPUBCheck 5.4.0 and pinned reference gems:

```sh
gem install asciidoctor -v 2.0.26 --no-document
gem install asciidoctor-epub3 -v 2.3.0 --no-document
gem install asciimath -v 2.0.6 --no-document
gem install rouge -v 5.1.0 --no-document
python tests/epub_features.py target/release/adoc-engine --epubcheck /path/to/epubcheck.jar
# Add --samples 9 for full-process measurements, --kotlin DIR for plugin-generated books.
```

The independent verifier parses XML, checks ZIP rules, manifest/spine/navigation,
all local resource/fragment/CSS references, metadata and feature preservation.
EPUBCheck runs on every generated book. Our outputs must pass both; reference
failures are retained in the report rather than weakening our checks to reproduce
them. In the pinned reference we observed quoted CSV splitting, double escaping
of description ampersands, and a repeated cross-chapter footnote link defect. The
pinned reference also requests `epub3.scss` for a custom style directory containing
the documented CSS files; our exporters accept those CSS files directly. These
reference outcomes are retained with the comparison evidence.
