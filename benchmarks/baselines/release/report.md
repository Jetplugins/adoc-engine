# HTML conversion benchmark

Platform: macOS-26.5.1-arm64-arm-64bit-Mach-O

Warm = worker conversion only. Cold = worker startup, one conversion, transport, and exit.
Cases with semantic mismatches are excluded. Peak process RSS is measured separately; browser painting is excluded.

| Case | Mode | Engine | Median ms | p95 ms | CV | conversions/s | Ruby / engine |
|---|---|---|---:|---:|---:|---:|---:|
| admonitions | warm | js | 15.824 | 17.686 | 0.054 | 62.8 | 0.32 |
| admonitions | warm | ruby | 5.143 | 5.582 | 0.048 | 195.5 | 1.00 |
| admonitions | warm | rust | 0.796 | 0.957 | 0.084 | 1224.1 | 6.46 |
| admonitions | cold | js | 134.960 | 136.974 | 0.009 | 7.4 | 0.60 |
| admonitions | cold | ruby | 81.396 | 84.298 | 0.016 | 12.2 | 1.00 |
| admonitions | cold | rust | 7.982 | 8.977 | 0.062 | 124.1 | 10.20 |
| attributes | warm | js | 13.560 | 14.233 | 0.043 | 73.3 | 0.69 |
| attributes | warm | ruby | 9.380 | 10.091 | 0.044 | 106.4 | 1.00 |
| attributes | warm | rust | 2.320 | 2.529 | 0.046 | 426.8 | 4.04 |
| attributes | cold | js | 134.278 | 136.403 | 0.012 | 7.5 | 0.64 |
| attributes | cold | ruby | 86.059 | 87.455 | 0.011 | 11.6 | 1.00 |
| attributes | cold | rust | 12.534 | 13.474 | 0.038 | 79.4 | 6.87 |
| basic | warm | js | 0.412 | 0.517 | 0.161 | 2367.9 | 0.45 |
| basic | warm | ruby | 0.183 | 0.326 | 0.877 | 4420.1 | 1.00 |
| basic | warm | rust | 0.095 | 0.113 | 0.123 | 10632.0 | 1.93 |
| basic | cold | js | 114.247 | 119.200 | 0.018 | 8.7 | 0.65 |
| basic | cold | ruby | 74.750 | 75.809 | 0.013 | 13.4 | 1.00 |
| basic | cold | rust | 9.222 | 10.072 | 0.058 | 107.8 | 8.11 |
| code | warm | js | 34.882 | 35.746 | 0.017 | 28.7 | 0.19 |
| code | warm | ruby | 6.759 | 7.281 | 0.036 | 147.2 | 1.00 |
| code | warm | rust | 2.133 | 2.251 | 0.032 | 468.2 | 3.17 |
| code | cold | js | 163.563 | 168.373 | 0.016 | 6.1 | 0.53 |
| code | cold | ruby | 86.997 | 87.864 | 0.012 | 11.5 | 1.00 |
| code | cold | rust | 13.596 | 15.121 | 0.051 | 72.4 | 6.40 |
| conditionals | warm | js | 9.565 | 10.361 | 0.047 | 105.2 | 0.29 |
| conditionals | warm | ruby | 2.780 | 3.353 | 0.085 | 351.6 | 1.00 |
| conditionals | warm | rust | 0.589 | 0.627 | 0.039 | 1696.4 | 4.72 |
| conditionals | cold | js | 133.149 | 174.618 | 0.103 | 7.2 | 0.59 |
| conditionals | cold | ruby | 78.734 | 81.153 | 0.019 | 12.7 | 1.00 |
| conditionals | cold | rust | 7.927 | 8.512 | 0.052 | 126.9 | 9.93 |
| features | warm | js | 0.853 | 1.083 | 0.114 | 1175.1 | 0.29 |
| features | warm | ruby | 0.243 | 0.342 | 0.795 | 3425.4 | 1.00 |
| features | warm | rust | 0.109 | 0.133 | 0.116 | 9058.1 | 2.23 |
| features | cold | js | 118.479 | 121.478 | 0.018 | 8.4 | 0.67 |
| features | cold | ruby | 79.361 | 127.517 | 0.196 | 11.5 | 1.00 |
| features | cold | rust | 7.527 | 8.457 | 0.079 | 133.8 | 10.54 |
| footnotes | warm | js | 4.815 | 5.729 | 0.083 | 204.9 | 0.43 |
| footnotes | warm | ruby | 2.050 | 2.570 | 0.104 | 481.6 | 1.00 |
| footnotes | warm | rust | 0.564 | 0.614 | 0.062 | 1775.3 | 3.63 |
| footnotes | cold | js | 122.307 | 126.833 | 0.017 | 8.1 | 0.63 |
| footnotes | cold | ruby | 77.369 | 80.392 | 0.022 | 12.9 | 1.00 |
| footnotes | cold | rust | 8.473 | 9.344 | 0.064 | 117.6 | 9.13 |
| nested-lists | warm | js | 17.204 | 18.918 | 0.049 | 57.5 | 0.23 |
| nested-lists | warm | ruby | 3.985 | 4.792 | 0.111 | 247.3 | 1.00 |
| nested-lists | warm | rust | 1.093 | 1.358 | 0.082 | 885.5 | 3.65 |
| nested-lists | cold | js | 133.976 | 138.765 | 0.017 | 7.4 | 0.61 |
| nested-lists | cold | ruby | 82.075 | 85.947 | 0.040 | 12.1 | 1.00 |
| nested-lists | cold | rust | 8.248 | 9.174 | 0.072 | 119.2 | 9.95 |
| sections | warm | js | 15.996 | 17.903 | 0.053 | 61.8 | 0.29 |
| sections | warm | ruby | 4.660 | 5.529 | 0.100 | 209.0 | 1.00 |
| sections | warm | rust | 1.146 | 1.269 | 0.070 | 870.9 | 4.07 |
| sections | cold | js | 134.188 | 138.645 | 0.046 | 7.4 | 0.60 |
| sections | cold | ruby | 81.116 | 82.279 | 0.012 | 12.4 | 1.00 |
| sections | cold | rust | 8.904 | 9.868 | 0.058 | 111.9 | 9.11 |
| tables | warm | js | 58.129 | 62.047 | 0.038 | 17.0 | 0.19 |
| tables | warm | ruby | 10.966 | 12.507 | 0.052 | 90.7 | 1.00 |
| tables | warm | rust | 5.100 | 5.479 | 0.032 | 193.8 | 2.15 |
| tables | cold | js | 159.258 | 162.249 | 0.016 | 6.3 | 0.58 |
| tables | cold | ruby | 91.916 | 111.045 | 0.072 | 10.6 | 1.00 |
| tables | cold | rust | 14.884 | 15.607 | 0.031 | 67.1 | 6.18 |
| unicode | warm | js | 12.719 | 13.694 | 0.034 | 78.2 | 0.77 |
| unicode | warm | ruby | 9.816 | 10.635 | 0.042 | 100.7 | 1.00 |
| unicode | warm | rust | 1.502 | 1.756 | 0.065 | 647.7 | 6.53 |
| unicode | cold | js | 137.722 | 140.852 | 0.014 | 7.3 | 0.63 |
| unicode | cold | ruby | 86.758 | 88.440 | 0.013 | 11.5 | 1.00 |
| unicode | cold | rust | 11.524 | 12.574 | 0.054 | 86.9 | 7.53 |
| paragraphs-100 | warm | js | 4.300 | 4.759 | 0.074 | 229.5 | 0.63 |
| paragraphs-100 | warm | ruby | 2.702 | 3.165 | 0.083 | 367.2 | 1.00 |
| paragraphs-100 | warm | rust | 0.555 | 0.637 | 0.056 | 1786.6 | 4.87 |
| paragraphs-100 | cold | js | 121.273 | 123.568 | 0.011 | 8.2 | 0.64 |
| paragraphs-100 | cold | ruby | 78.195 | 79.296 | 0.012 | 12.8 | 1.00 |
| paragraphs-100 | cold | rust | 9.292 | 10.252 | 0.060 | 106.4 | 8.42 |
| paragraphs-1000 | warm | js | 39.417 | 41.340 | 0.039 | 25.2 | 0.71 |
| paragraphs-1000 | warm | ruby | 27.967 | 30.200 | 0.033 | 35.5 | 1.00 |
| paragraphs-1000 | warm | rust | 5.169 | 5.796 | 0.055 | 191.7 | 5.41 |
| paragraphs-1000 | cold | js | 163.829 | 166.870 | 0.012 | 6.1 | 0.65 |
| paragraphs-1000 | cold | ruby | 106.875 | 108.478 | 0.013 | 9.4 | 1.00 |
| paragraphs-1000 | cold | rust | 15.529 | 16.479 | 0.045 | 65.2 | 6.88 |
| paragraphs-10000 | warm | js | 438.006 | 521.199 | 0.067 | 2.2 | 0.68 |
| paragraphs-10000 | warm | ruby | 299.333 | 307.778 | 0.016 | 3.3 | 1.00 |
| paragraphs-10000 | warm | rust | 52.961 | 85.660 | 0.338 | 17.1 | 5.65 |
| paragraphs-10000 | cold | js | 658.474 | 732.529 | 0.065 | 1.5 | 0.59 |
| paragraphs-10000 | cold | ruby | 390.575 | 478.698 | 0.079 | 2.5 | 1.00 |
| paragraphs-10000 | cold | rust | 76.935 | 81.612 | 0.030 | 12.9 | 5.08 |
| bytes-1024 | warm | js | 1.123 | 1.391 | 0.166 | 869.6 | 0.56 |
| bytes-1024 | warm | ruby | 0.624 | 1.746 | 0.489 | 1304.7 | 1.00 |
| bytes-1024 | warm | rust | 0.159 | 0.234 | 0.166 | 5986.3 | 3.91 |
| bytes-1024 | cold | js | 124.931 | 135.848 | 0.039 | 7.9 | 0.68 |
| bytes-1024 | cold | ruby | 85.395 | 170.726 | 0.342 | 10.1 | 1.00 |
| bytes-1024 | cold | rust | 10.598 | 11.662 | 0.080 | 94.2 | 8.06 |
| bytes-10240 | warm | js | 8.007 | 9.276 | 0.056 | 123.4 | 0.68 |
| bytes-10240 | warm | ruby | 5.415 | 5.839 | 0.051 | 185.5 | 1.00 |
| bytes-10240 | warm | rust | 1.082 | 1.239 | 0.079 | 913.1 | 5.00 |
| bytes-10240 | cold | js | 133.153 | 142.833 | 0.040 | 7.4 | 0.92 |
| bytes-10240 | cold | ruby | 122.290 | 242.639 | 0.395 | 6.9 | 1.00 |
| bytes-10240 | cold | rust | 10.940 | 14.726 | 0.190 | 84.9 | 11.18 |
| bytes-102400 | warm | js | 76.568 | 147.273 | 0.256 | 11.8 | 0.73 |
| bytes-102400 | warm | ruby | 55.623 | 62.326 | 0.056 | 17.8 | 1.00 |
| bytes-102400 | warm | rust | 10.102 | 17.595 | 0.364 | 89.0 | 5.51 |
| bytes-102400 | cold | js | 200.789 | 207.773 | 0.017 | 5.0 | 0.70 |
| bytes-102400 | cold | ruby | 140.147 | 205.814 | 0.205 | 6.5 | 1.00 |
| bytes-102400 | cold | rust | 23.838 | 26.674 | 0.050 | 41.4 | 5.88 |
| bytes-1048576 | warm | js | 1147.178 | 1241.067 | 0.043 | 0.9 | 0.52 |
| bytes-1048576 | warm | ruby | 592.625 | 611.288 | 0.017 | 1.7 | 1.00 |
| bytes-1048576 | warm | rust | 101.735 | 106.091 | 0.029 | 9.8 | 5.83 |
| bytes-1048576 | cold | js | 1325.261 | 1406.105 | 0.024 | 0.7 | 0.51 |
| bytes-1048576 | cold | ruby | 680.904 | 750.941 | 0.148 | 1.4 | 1.00 |
| bytes-1048576 | cold | rust | 135.510 | 141.547 | 0.020 | 7.4 | 5.02 |
| includes | warm | js | 0.547 | 1.149 | 0.355 | 1686.9 | 0.24 |
| includes | warm | ruby | 0.130 | 0.148 | 0.080 | 7639.4 | 1.00 |
| includes | warm | rust | 0.191 | 0.240 | 0.139 | 5179.7 | 0.68 |
| includes | cold | js | 125.023 | 130.675 | 0.050 | 8.0 | 0.61 |
| includes | cold | ruby | 75.652 | 79.450 | 0.021 | 13.2 | 1.00 |
| includes | cold | rust | 9.768 | 10.551 | 0.052 | 102.0 | 7.74 |
| include-selection | warm | js | 0.801 | 1.032 | 0.130 | 1217.8 | 0.27 |
| include-selection | warm | ruby | 0.220 | 0.308 | 0.645 | 3908.1 | 1.00 |
| include-selection | warm | rust | 0.124 | 0.197 | 0.200 | 7333.2 | 1.77 |
| include-selection | cold | js | 119.511 | 123.936 | 0.023 | 8.4 | 0.64 |
| include-selection | cold | ruby | 76.492 | 78.624 | 0.020 | 13.1 | 1.00 |
| include-selection | cold | rust | 10.562 | 11.585 | 0.049 | 93.7 | 7.24 |
| repeated-edits | warm | js | 37.201 | 41.694 | 0.060 | 26.8 | 0.44 |
| repeated-edits | warm | ruby | 16.197 | 17.881 | 0.050 | 61.6 | 1.00 |
| repeated-edits | warm | rust | 2.938 | 3.392 | 0.068 | 333.9 | 5.51 |
| repeated-edits | cold | js | 157.378 | 161.238 | 0.011 | 6.3 | 0.61 |
| repeated-edits | cold | ruby | 96.365 | 99.537 | 0.018 | 10.4 | 1.00 |
| repeated-edits | cold | rust | 13.781 | 14.220 | 0.032 | 73.2 | 6.99 |
| repeated-edits | edit | js | 35.545 | 36.978 | 0.025 | 28.1 | 0.45 |
| repeated-edits | edit | ruby | 15.950 | 17.955 | 0.050 | 62.1 | 1.00 |
| repeated-edits | edit | rust | 3.097 | 3.338 | 0.049 | 324.8 | 5.15 |

## Peak process memory

| Case | Engine | Peak RSS MiB |
|---|---|---:|
| admonitions | js | 93.3 |
| admonitions | ruby | 24.0 |
| admonitions | rust | 11.7 |
| attributes | js | 93.8 |
| attributes | ruby | 24.5 |
| attributes | rust | 14.0 |
| basic | js | 81.6 |
| basic | ruby | 22.6 |
| basic | rust | 12.8 |
| code | js | 129.7 |
| code | ruby | 29.7 |
| code | rust | 17.2 |
| conditionals | js | 91.0 |
| conditionals | ruby | 23.3 |
| conditionals | rust | 11.9 |
| features | js | 80.8 |
| features | ruby | 22.7 |
| features | rust | 12.1 |
| footnotes | js | 86.7 |
| footnotes | ruby | 23.2 |
| footnotes | rust | 12.2 |
| nested-lists | js | 93.3 |
| nested-lists | ruby | 23.7 |
| nested-lists | rust | 13.6 |
| sections | js | 93.9 |
| sections | ruby | 23.9 |
| sections | rust | 12.7 |
| tables | js | 110.4 |
| tables | ruby | 25.8 |
| tables | rust | 14.1 |
| unicode | js | 93.7 |
| unicode | ruby | 24.9 |
| unicode | rust | 13.7 |
| paragraphs-100 | js | 85.8 |
| paragraphs-100 | ruby | 23.2 |
| paragraphs-100 | rust | 12.7 |
| paragraphs-1000 | js | 110.9 |
| paragraphs-1000 | ruby | 25.0 |
| paragraphs-1000 | rust | 15.4 |
| paragraphs-10000 | js | 170.2 |
| paragraphs-10000 | ruby | 47.4 |
| paragraphs-10000 | rust | 44.1 |
| bytes-1024 | js | 81.5 |
| bytes-1024 | ruby | 22.8 |
| bytes-1024 | rust | 12.5 |
| bytes-10240 | js | 91.6 |
| bytes-10240 | ruby | 23.9 |
| bytes-10240 | rust | 13.0 |
| bytes-102400 | js | 118.8 |
| bytes-102400 | ruby | 27.1 |
| bytes-102400 | rust | 18.3 |
| bytes-1048576 | js | 195.2 |
| bytes-1048576 | ruby | 65.3 |
| bytes-1048576 | rust | 76.0 |
| includes | js | 81.8 |
| includes | ruby | 22.9 |
| includes | rust | 12.8 |
| include-selection | js | 80.1 |
| include-selection | ruby | 22.8 |
| include-selection | rust | 13.3 |
| repeated-edits | js | 111.6 |
| repeated-edits | ruby | 24.2 |
| repeated-edits | rust | 14.3 |
