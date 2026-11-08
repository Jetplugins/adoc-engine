# HTML conversion benchmark

Platform: macOS-26.5.1-arm64-arm-64bit-Mach-O

Warm = worker conversion only. Cold = worker startup, one conversion, transport, and exit.
Cases with semantic mismatches are excluded. Peak process RSS is measured separately; browser painting is excluded.

| Case | Mode | Engine | Median ms | p95 ms | CV | conversions/s | Ruby / engine |
|---|---|---|---:|---:|---:|---:|---:|
| basic | warm | js | 0.309 | 0.408 | 0.168 | 3127.4 | 0.53 |
| basic | warm | ruby | 0.165 | 0.252 | 0.665 | 5244.1 | 1.00 |
| basic | warm | rust | 0.058 | 0.086 | 0.178 | 15799.8 | 2.85 |
| basic | cold | js | 113.968 | 117.171 | 0.016 | 8.8 | 0.66 |
| basic | cold | ruby | 74.650 | 76.357 | 0.043 | 13.4 | 1.00 |
| basic | cold | rust | 9.026 | 10.948 | 0.102 | 108.9 | 8.27 |
| bytes-1024 | warm | js | 0.923 | 1.077 | 0.111 | 1060.5 | 0.63 |
| bytes-1024 | warm | ruby | 0.580 | 0.996 | 0.206 | 1608.6 | 1.00 |
| bytes-1024 | warm | rust | 0.162 | 0.199 | 0.134 | 6157.4 | 3.57 |
| bytes-1024 | cold | js | 115.818 | 125.347 | 0.035 | 8.6 | 0.65 |
| bytes-1024 | cold | ruby | 74.913 | 79.407 | 0.023 | 13.3 | 1.00 |
| bytes-1024 | cold | rust | 8.535 | 9.565 | 0.059 | 116.8 | 8.78 |

## Peak process memory

| Case | Engine | Peak RSS MiB |
|---|---|---:|
| basic | js | 80.2 |
| basic | ruby | 22.8 |
| basic | rust | 12.8 |
| bytes-1024 | js | 80.8 |
| bytes-1024 | ruby | 22.5 |
| bytes-1024 | rust | 12.5 |
