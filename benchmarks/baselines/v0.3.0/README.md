# EPUB 0.3.0 verification and performance

macOS-26.5.1-arm64-arm-64bit-Mach-O

Full CLI process per sample; includes startup, parsing/includes, assets, font compression, archive creation and file writes. Two warmups; interleaved engines. EPUBCheck excluded.

All seven Rust outputs and all six Kotlin fixtures passed EPUBCheck 5.4.0 and the independent ZIP/XML/link/metadata/feature checker. The extra Rust case is a generated 40-chapter performance book. The reference article and media pass; its repeated cross-chapter footnote fails RSC-012 in book/depth/large. The custom CSS case requests a missing epub3.scss, despite the documented CSS input. Logs and raw records retain these reference failures.

| Document | Rust CLI median, ms | Ruby CLI median, ms | Rust speedup |
| --- | ---: | ---: | ---: |
| article | 58.80 | 429.37 | 7.30× |
| book | 77.06 | 545.69 | 7.08× |
| media | 52.15 | 414.72 | 7.95× |
| large | 102.18 | 627.37 | 6.14× |

Nine samples and two warmups, interleaved engines. Includes fonts and archive compression. The Ruby book/large timings describe generation of reference output with the documented link defect; they do not imply validation passed. This is one local source build, not a cross-platform guarantee.

## Kotlin (separate timing method)

In-process JVM, three warmups and nine samples. Includes conversion, assets, compression and writes; excludes IDE/JVM startup and include resolution. Do not read this as a cold CLI comparison.

| Document | Warm JVM median, ms |
| --- | ---: |
| book | 199.28 |
| article | 103.46 |
| media | 104.48 |
| depth | 193.06 |
| custom | 200.40 |
| reader-fonts | 86.82 |

Asciidoctor.js supplies HTML rendering; the IntelliJ JavaScript preview selection uses the Kotlin EPUB exporter. Existing three-renderer HTML results in the README are kept separately labeled.

Source tree and binary SHA-256, all timing samples, fixture fingerprints, archive hashes and EPUBCheck logs are in `epub-results.json`. Kotlin archive fingerprints and timing method are in the adjacent JSON files. The canonical source fixtures and reproducible runner are under `tests/fixtures/epub` and `tests/epub_features.py`.
