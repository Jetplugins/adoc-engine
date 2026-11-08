# HTML conversion benchmark

Platform: macOS-26.5.1-arm64-arm-64bit-Mach-O

Warm = worker conversion only. Cold = worker startup, one conversion, transport, and exit.
Cases with semantic mismatches are excluded. Peak process RSS is measured separately; browser painting is excluded.

| Case | Mode | Engine | Median ms | p95 ms | CV | conversions/s | Ruby / engine |
|---|---|---|---:|---:|---:|---:|---:|
| basic | warm | js | 0.342 | 0.486 | 0.228 | 2743.9 | 0.48 |
| basic | warm | ruby | 0.166 | 0.221 | 1.061 | 5026.1 | 1.00 |
| basic | warm | rust | 0.055 | 0.061 | 0.092 | 18153.8 | 3.03 |
| basic | cold | js | 113.197 | 123.365 | 0.030 | 8.7 | 0.66 |
| basic | cold | ruby | 74.393 | 76.368 | 0.018 | 13.4 | 1.00 |
| basic | cold | rust | 10.025 | 11.012 | 0.079 | 99.9 | 7.42 |
| features | warm | js | 0.784 | 0.951 | 0.088 | 1233.2 | 0.32 |
| features | warm | ruby | 0.247 | 0.310 | 0.737 | 3413.9 | 1.00 |
| features | warm | rust | 0.055 | 0.069 | 0.085 | 17565.4 | 4.46 |
| features | cold | js | 114.540 | 116.636 | 0.010 | 8.7 | 0.65 |
| features | cold | ruby | 74.699 | 75.896 | 0.017 | 13.5 | 1.00 |
| features | cold | rust | 7.380 | 7.733 | 0.060 | 137.3 | 10.12 |
| paragraphs-100 | warm | js | 4.189 | 4.692 | 0.077 | 234.9 | 0.65 |
| paragraphs-100 | warm | ruby | 2.716 | 3.120 | 0.092 | 362.3 | 1.00 |
| paragraphs-100 | warm | rust | 0.539 | 0.624 | 0.074 | 1805.1 | 5.04 |
| paragraphs-100 | cold | js | 119.027 | 121.346 | 0.012 | 8.4 | 0.64 |
| paragraphs-100 | cold | ruby | 76.749 | 78.636 | 0.019 | 13.0 | 1.00 |
| paragraphs-100 | cold | rust | 9.368 | 10.120 | 0.061 | 106.9 | 8.19 |
| paragraphs-1000 | warm | js | 44.313 | 47.137 | 0.061 | 22.5 | 0.61 |
| paragraphs-1000 | warm | ruby | 27.215 | 29.224 | 0.036 | 36.6 | 1.00 |
| paragraphs-1000 | warm | rust | 5.427 | 5.764 | 0.046 | 185.5 | 5.01 |
| paragraphs-1000 | cold | js | 161.775 | 166.404 | 0.014 | 6.2 | 0.64 |
| paragraphs-1000 | cold | ruby | 103.982 | 109.053 | 0.027 | 9.6 | 1.00 |
| paragraphs-1000 | cold | rust | 16.935 | 17.959 | 0.046 | 59.3 | 6.14 |
| paragraphs-10000 | warm | js | 419.673 | 443.565 | 0.035 | 2.4 | 0.70 |
| paragraphs-10000 | warm | ruby | 294.460 | 311.083 | 0.022 | 3.4 | 1.00 |
| paragraphs-10000 | warm | rust | 51.451 | 53.509 | 0.019 | 19.4 | 5.72 |
| paragraphs-10000 | cold | js | 610.826 | 637.331 | 0.047 | 1.7 | 0.62 |
| paragraphs-10000 | cold | ruby | 381.380 | 387.784 | 0.009 | 2.6 | 1.00 |
| paragraphs-10000 | cold | rust | 73.854 | 76.606 | 0.021 | 13.5 | 5.16 |
| includes | warm | js | 0.422 | 0.603 | 0.164 | 2305.5 | 0.38 |
| includes | warm | ruby | 0.160 | 0.216 | 0.138 | 6062.1 | 1.00 |
| includes | warm | rust | 0.161 | 0.209 | 0.171 | 6210.9 | 1.00 |
| includes | cold | js | 114.569 | 121.921 | 0.026 | 8.7 | 0.65 |
| includes | cold | ruby | 75.013 | 76.314 | 0.023 | 13.4 | 1.00 |
| includes | cold | rust | 9.484 | 11.096 | 0.078 | 103.1 | 7.91 |

## Peak process memory

| Case | Engine | Peak RSS MiB |
|---|---|---:|
| basic | js | 81.9 |
| basic | ruby | 22.7 |
| basic | rust | 12.4 |
| features | js | 81.4 |
| features | ruby | 23.0 |
| features | rust | 11.5 |
| paragraphs-100 | js | 86.2 |
| paragraphs-100 | ruby | 23.1 |
| paragraphs-100 | rust | 12.2 |
| paragraphs-1000 | js | 111.2 |
| paragraphs-1000 | ruby | 25.1 |
| paragraphs-1000 | rust | 14.8 |
| paragraphs-10000 | js | 168.4 |
| paragraphs-10000 | ruby | 46.6 |
| paragraphs-10000 | rust | 44.8 |
| includes | js | 80.4 |
| includes | ruby | 22.9 |
| includes | rust | 12.3 |
