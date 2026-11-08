# EPUB reference fixtures

Authored for adoc-engine 0.3.0 under the repository's MIT license. The same files
are mirrored in the IntelliJ plugin's `src/test/testData/epub`.

`book.adoc` exercises metadata, includes, chapter IDs, navigation depths, cross
references, repeated footnotes, lists, checklists, captions, source highlighting,
callouts, a table, SVG and inline/block AsciiMath. `article.adoc` checks single-file
structure, byline and avatar. `media.adoc` includes local MP3 and MP4.

Cover and avatar are generated solid-color images; audio is generated silence and
video a generated black frame (FFmpeg). The SVG diagram and front matter are
original fixtures. They contain no third-party artwork or recordings.

The verification runners copy the bundled Libertinus Serif regular face into
`styles/fonts/book.otf` when staging inputs. Its OFL notice is already shipped
with the exporter. This avoids checking in another duplicate font file.
