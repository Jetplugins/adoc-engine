# HTML conversion benchmark

Platform: macOS-26.5.1-arm64-arm-64bit-Mach-O

Warm = worker conversion only. Cold = worker startup, one conversion, transport, and exit.
Cases with semantic mismatches are excluded. Peak process RSS is measured separately; browser painting is excluded.

| Case | Mode | Engine | Median ms | p95 ms | CV | conversions/s | Ruby / engine |
|---|---|---|---:|---:|---:|---:|---:|
| showcase | warm | js | 8.888 | 10.386 | 0.078 | 110.6 | 0.30 |
| showcase | warm | ruby | 2.709 | 3.394 | 0.131 | 361.8 | 1.00 |
| showcase | warm | rust | 0.793 | 0.954 | 0.107 | 1258.9 | 3.42 |
| showcase | cold | js | 139.547 | 142.865 | 0.012 | 7.2 | 0.56 |
| showcase | cold | ruby | 78.474 | 81.298 | 0.017 | 12.7 | 1.00 |
| showcase | cold | rust | 18.467 | 19.482 | 0.033 | 54.4 | 4.25 |

## Peak process memory

| Case | Engine | Peak RSS MiB |
|---|---|---:|
| showcase | js | 95.4 |
| showcase | ruby | 24.0 |
| showcase | rust | 18.5 |
