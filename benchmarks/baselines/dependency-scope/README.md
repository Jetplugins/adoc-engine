# Dependency scope: clean-build baseline

Measured on macOS ARM64 on 2026-11-12 with Rust 1.99.1. These are development
sources based on commit `7f0cc921f7723188adafcf2b794f5847800af2b6`; the working tree was modified.
The raw reports bind the observations to a source fingerprint and lockfile SHA-256.

| Profile | Normal dependency packages | Clean release build, seconds | Consumer executable, MiB |
| --- | ---: | ---: | ---: |
| [HTML / JSON / analysis](core.json) | 48 | 25.23 | 4.80 |
| [Core + native MathML](mathml.json) | 66 | 28.17 | 5.38 |
| [EPUB + MathML](epub.json) | 92 | 31.45 | 8.25 |
| [EPUB + MathML + highlighting](highlighting.json) | 107 | 29.70 | 9.10 |
| [Full library](full.json) | 377 | 161.22 | 51.45 |

The pre-split engine graph had 378 normal dependency packages on this host.
The full profile now has 377 after unifying `typst-as-lib` on 0.17. The large
reduction comes from allowing consumers to omit backends, while preserving full
defaults and the fully featured official CLI.

## Method

- One build per profile, run sequentially after validation builds finished.
- Fresh temporary target directory for each build; incremental compilation and
  compiler wrappers disabled. Dependency downloads and the OS file cache are warm.
- Release optimization matches the workspace: thin LTO and one codegen unit, with
  four compilation jobs.
- The same standalone consumer source accepts a runtime output-format argument,
  retaining every enabled export backend. The size is its unstripped executable,
  not the packaged CLI or a compressed download.
- Counts come from the host-target normal dependency graph, excluding the engine
  and consumer packages. Build/dev dependencies are not included in that count.
- Every built consumer is executed after timing, checking HTML, analysis, enabled
  backends, disabled-feature errors, native MathML and highlighting behavior.
- Single-run timings include noise: the highlighted EPUB sample happened to build
  faster than the smaller EPUB profile. Do not infer universal build-time ratios
  or runtime conversion speeds from these observations.

## Reproduce

From the repository root:

```sh
for profile in core mathml epub highlighting full; do
  python3 tests/feature_consumer.py --profile "$profile" --measure \
    --output "/tmp/dependency-scope-$profile.json"
done
```

The temporary targets are removed after each measurement. The reports include
the feature list, supported formats, host/compiler, source and consumer hashes,
lockfile hash, executable hash, elapsed build time and byte size.

See [the feature guide](../../../docs/features.md) for API use and CI coverage.
