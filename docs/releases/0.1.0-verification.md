# Release verification: 0.1.0

Verified on 2026-11-08. Runtime source commit measured: `b2a87292c5c4e46c42418574c5e928fb73273fab`.
Measured macOS ARM64 executable SHA-256: `c36ae1aacaa2ec79c482b14d5feb114da182f520fba3f850bad6708798810d13`.

## Correctness gates

- 18 Rust tests: semantic rendering, safe modes, source maps, overlays, alias contexts,
  compound table cells, passthrough, non-section destinations, diagrams, EPUB structure,
  hostile assets, and Unicode edits.
- 6 executable LSP protocol tests: initialization, malformed requests, recovery,
  atomic invalid edits, Unicode, unsaved include navigation, cancellation, stale
  snapshots, parsed outlines, and correct included-file diagnostic URIs.
- 7 compatibility-validator tests, including inline word boundaries, non-breaking
  spaces, links, formatting whitespace, and literal code whitespace.
- 428 IntelliJ preview/export/include tests across 41 suites, zero failures/errors/skips;
  includes real-filesystem native integration with unsaved includes and supplied diagrams.
- The Rust library compiles from a separate Cargo consumer outside this workspace.
  Its patched publication backend is a transitive path dependency, not a root-only patch.
- EPUBCheck 5.4.0 accepts both the feature book and supplied-diagram book.
- Poppler confirms expected PDF text and page counts; all three rendered QA pages
  were visually inspected for layout, table content, typography, links, and diagrams.
- SVG/PNG page counts match PDF; merged SVG and EPUB XML/container checks pass.
- Source formatting and strict Clippy checks pass.
- UTF-8 license metadata collection is tested under a non-UTF-8 locale, addressing
  the Windows CP1252 failure found in the first platform preflight.

## Performance gates

All 21 HTML corpora and all 20 edit variants pass strict structural parity against
Ruby Asciidoctor 2.0.26 and Asciidoctor.js 3.0.4. Across the final executable, the
suite records 6,350 latency samples and 69
peak-memory measurements. HTML, publication, LSP, strict parity, and confirmation
reports share the executable fingerprint above.

The 1 MiB warm conversion medians on Apple M2 Pro are:

| Engine | Median ms |
|---|---:|
| JavaScript | 1147.178 |
| Ruby | 592.625 |
| Rust | 101.735 |

The provisional sub-100 ms target at 1 MiB remains an optimization target; this
release publishes its measured result rather than claiming a latency SLA.

Two differences above 10% in an initial repeat did not reproduce under isolated
confirmation with 200 warm samples and 50 cold executions per engine/case. There
are no confirmed regressions above the threshold. Full samples, the prior comparison
baseline, and the confirmation results are in [the release evidence](benchmarks/baselines/release).

## Distribution gates

The release workflow runs native tests and executable protocol tests on macOS ARM64,
macOS x64, Linux ARM64, Linux x64, and Windows x64. An independent Linux job repeats
strict three-engine parity and EPUBCheck/Poppler publication validation. CI also
checks the declared Rust 1.96 minimum. A draft is created only after all distribution
jobs pass, and it is promoted only after source CI passes.

Archives include build provenance, binary fingerprints, full dependency/font licenses,
protocol documentation, verification notes, and raw performance evidence. Each archive
has its own SHA-256 checksum. The host archive is extracted and smoke-tested without
Ruby, Node, or external tools on PATH before publication.

## Boundaries

This is not an AsciiDoc TCK certification. The explicit native compatibility contract
is in [COMPATIBILITY.md](COMPATIBILITY.md). Ruby/JS extension execution, remote asset
fetching, and browser CSS/JavaScript in paginated output are outside that contract.
