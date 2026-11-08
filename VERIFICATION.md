# Release verification: 0.2.0

Version 0.2.0 includes the list, checklist, caption and special-section fixes
verified in the complete showcase. Normal CI checks the exact pinned Ruby/JS
golden output in addition to 29 Rust tests and the downstream library consumer.
The release pipeline repeats 270 reference cases, all 23 compatibility corpora
and 20 editing variants, EPUBCheck/Poppler publication validation, and native
archive relocation tests on all five supported targets. It attaches archives
only after all required jobs pass. The tagged source commit, executable digest
and archive checksum are recorded in BUILD.json and release-manifest.json.

The plugin integration was separately verified by 1,315 unit tests before the
managed installer was added; installer coverage is recorded with the plugin.
Performance figures identify their measured source build and platform; the
v0.1.0 measurements remain historical reference data.

See [compatibility evidence](benchmarks/baselines/showcase-compatibility),
[release workflow](.github/workflows/release.yml), and the
[historical 0.1.0 record](docs/releases/0.1.0-verification.md).
