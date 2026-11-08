# HTML conversion benchmark

Platform: macOS-26.5.1-arm64-arm-64bit-Mach-O

Warm = worker conversion only. Cold = worker startup, one conversion, transport, and exit.
Cases with semantic mismatches are excluded. Peak process RSS is measured separately; browser painting is excluded.

| Case | Mode | Engine | Median ms | p95 ms | CV | conversions/s | Ruby / engine |
|---|---|---|---:|---:|---:|---:|---:|
| showcase | warm | js | 8.835 | 9.972 | 0.066 | 111.6 | 0.31 |
| showcase | warm | ruby | 2.709 | 3.500 | 0.134 | 355.6 | 1.00 |
| showcase | warm | rust | 0.745 | 0.949 | 0.125 | 1275.1 | 3.63 |
| showcase | cold | js | 140.807 | 145.018 | 0.015 | 7.1 | 0.57 |
| showcase | cold | ruby | 80.291 | 82.313 | 0.014 | 12.4 | 1.00 |
| showcase | cold | rust | 18.654 | 19.684 | 0.040 | 53.7 | 4.30 |

## Peak process memory

| Case | Engine | Peak RSS MiB |
|---|---|---:|
| showcase | js | 95.4 |
| showcase | ruby | 23.7 |
| showcase | rust | 18.5 |
