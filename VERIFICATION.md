# Release verification: 0.2.0

[Version 0.2.0](https://github.com/Jetplugins/adoc-engine/releases/tag/v0.2.0)
was published on 2026-11-08 from source commit
`9bdb9b9565ee382b2d92fcc3d02340fdcf39c2b7`.
The [release workflow](https://github.com/Jetplugins/adoc-engine/actions/runs/42467800008)
completed successfully before publication.

All five native targets passed 29 Rust tests, six executable protocol tests,
and execution of the relocated distribution without Ruby or Node on PATH:
macOS ARM64/x64, Linux ARM64/x64, and Windows x64. Each archive includes its
licenses and BUILD.json. The published release-manifest.json binds all five
archive SHA-256 checksums to the tested source commit and worker protocol 1.

Reference verification passed 270 focused Ruby/JavaScript/Rust cases,
all 23 compatibility corpora and 20 editing variants. EPUBCheck 5.4.0 passed
for the generated books and supplied diagrams. Poppler verified PDF text,
page structure and rendered pages; SVG, merged SVG, PNG and JSON exports passed.
Normal CI also checks formatting, strict Clippy and the downstream library consumer.

The IntelliJ consumer passed 1,339 tests after forced source compilation with
filesystem watching disabled. Its opt-in live test downloaded public 0.2.0
through the IDE HTTP client without authentication, verified the manifest and
checksum, safely installed the macOS ARM64 archive, checked its executable
version and receipt, and rendered a real document. The downloaded archive SHA-256
was `9654b2b9dabeb3f541d0a711531a41870c10b4c220fd5bf22f358f20e5230d37`.
Installer coverage also checks corrupt downloads, unsafe paths and links,
failed-update recovery, license preservation and cached-binary corruption.

See the [machine-readable verification record](benchmarks/baselines/v0.2.0/release-verification.json),
[0.2.0 compatibility and performance evidence](benchmarks/baselines/v0.2.0),
[release workflow](.github/workflows/release.yml), and the
[historical 0.1.0 record](docs/releases/0.1.0-verification.md).
Performance figures identify their measured source build and platform;
the v0.1.0 measurements remain historical reference data.
