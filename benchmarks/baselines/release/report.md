# HTML conversion benchmark

Platform: macOS-26.5.1-arm64-arm-64bit-Mach-O

Warm = worker conversion only. Cold = worker startup, one conversion, transport, and exit.
Cases with semantic mismatches are excluded. Peak process RSS is measured separately; browser painting is excluded.

| Case | Mode | Engine | Median ms | p95 ms | CV | conversions/s | Ruby / engine |
|---|---|---|---:|---:|---:|---:|---:|
| admonitions | warm | js | 16.052 | 17.843 | 0.042 | 62.0 | 0.33 |
| admonitions | warm | ruby | 5.225 | 5.690 | 0.043 | 190.1 | 1.00 |
| admonitions | warm | rust | 0.787 | 0.868 | 0.056 | 1267.4 | 6.64 |
| admonitions | cold | js | 136.124 | 139.239 | 0.014 | 7.4 | 0.61 |
| admonitions | cold | ruby | 82.683 | 84.523 | 0.013 | 12.1 | 1.00 |
| admonitions | cold | rust | 8.674 | 9.304 | 0.048 | 115.7 | 9.53 |
| attributes | warm | js | 13.636 | 14.553 | 0.034 | 73.0 | 0.70 |
| attributes | warm | ruby | 9.550 | 10.091 | 0.040 | 105.0 | 1.00 |
| attributes | warm | rust | 2.338 | 2.581 | 0.047 | 423.9 | 4.08 |
| attributes | cold | js | 135.996 | 138.641 | 0.012 | 7.3 | 0.64 |
| attributes | cold | ruby | 87.162 | 87.885 | 0.007 | 11.5 | 1.00 |
| attributes | cold | rust | 13.252 | 13.825 | 0.034 | 75.9 | 6.58 |
| basic | warm | js | 0.423 | 1.232 | 0.480 | 1920.4 | 0.35 |
| basic | warm | ruby | 0.146 | 0.194 | 1.198 | 5545.1 | 1.00 |
| basic | warm | rust | 0.061 | 0.107 | 0.230 | 14958.1 | 2.40 |
| basic | cold | js | 124.368 | 137.783 | 0.074 | 7.9 | 0.62 |
| basic | cold | ruby | 77.068 | 78.423 | 0.022 | 13.0 | 1.00 |
| basic | cold | rust | 10.002 | 11.180 | 0.078 | 97.8 | 7.71 |
| code | warm | js | 36.660 | 51.671 | 0.193 | 25.9 | 0.19 |
| code | warm | ruby | 6.857 | 7.442 | 0.043 | 144.9 | 1.00 |
| code | warm | rust | 2.236 | 5.127 | 0.314 | 396.7 | 3.07 |
| code | cold | js | 168.222 | 183.141 | 0.062 | 5.8 | 0.53 |
| code | cold | ruby | 89.163 | 99.906 | 0.052 | 11.1 | 1.00 |
| code | cold | rust | 17.291 | 22.555 | 0.186 | 54.9 | 5.16 |
| conditionals | warm | js | 9.043 | 10.124 | 0.047 | 109.2 | 0.31 |
| conditionals | warm | ruby | 2.841 | 3.372 | 0.119 | 337.6 | 1.00 |
| conditionals | warm | rust | 0.577 | 0.625 | 0.069 | 1755.8 | 4.93 |
| conditionals | cold | js | 127.552 | 129.342 | 0.011 | 7.8 | 0.62 |
| conditionals | cold | ruby | 78.856 | 80.710 | 0.019 | 12.7 | 1.00 |
| conditionals | cold | rust | 8.398 | 9.187 | 0.055 | 118.7 | 9.39 |
| features | warm | js | 0.838 | 1.036 | 0.116 | 1169.4 | 0.28 |
| features | warm | ruby | 0.231 | 0.339 | 0.818 | 3661.4 | 1.00 |
| features | warm | rust | 0.106 | 0.156 | 0.445 | 8451.1 | 2.18 |
| features | cold | js | 118.476 | 121.487 | 0.011 | 8.4 | 0.64 |
| features | cold | ruby | 76.130 | 77.827 | 0.021 | 13.2 | 1.00 |
| features | cold | rust | 7.895 | 9.066 | 0.077 | 125.2 | 9.64 |
| footnotes | warm | js | 4.977 | 5.643 | 0.070 | 196.8 | 0.42 |
| footnotes | warm | ruby | 2.103 | 2.531 | 0.101 | 471.8 | 1.00 |
| footnotes | warm | rust | 0.573 | 0.642 | 0.101 | 1736.5 | 3.67 |
| footnotes | cold | js | 122.174 | 124.537 | 0.010 | 8.2 | 0.64 |
| footnotes | cold | ruby | 78.408 | 80.741 | 0.021 | 12.7 | 1.00 |
| footnotes | cold | rust | 8.687 | 9.015 | 0.047 | 116.4 | 9.03 |
| nested-lists | warm | js | 17.457 | 19.111 | 0.040 | 56.9 | 0.23 |
| nested-lists | warm | ruby | 4.019 | 4.686 | 0.106 | 247.6 | 1.00 |
| nested-lists | warm | rust | 1.047 | 1.364 | 0.103 | 919.3 | 3.84 |
| nested-lists | cold | js | 135.765 | 138.132 | 0.008 | 7.4 | 0.60 |
| nested-lists | cold | ruby | 81.052 | 82.174 | 0.014 | 12.4 | 1.00 |
| nested-lists | cold | rust | 8.312 | 9.097 | 0.073 | 119.6 | 9.75 |
| sections | warm | js | 15.723 | 16.538 | 0.030 | 63.5 | 0.29 |
| sections | warm | ruby | 4.527 | 5.321 | 0.093 | 216.7 | 1.00 |
| sections | warm | rust | 1.131 | 1.312 | 0.076 | 873.0 | 4.00 |
| sections | cold | js | 135.867 | 140.164 | 0.018 | 7.3 | 0.60 |
| sections | cold | ruby | 81.540 | 83.525 | 0.016 | 12.3 | 1.00 |
| sections | cold | rust | 9.246 | 9.868 | 0.051 | 108.9 | 8.82 |
| tables | warm | js | 59.993 | 73.282 | 0.088 | 16.4 | 0.18 |
| tables | warm | ruby | 11.072 | 12.390 | 0.054 | 89.4 | 1.00 |
| tables | warm | rust | 5.165 | 5.365 | 0.029 | 193.6 | 2.14 |
| tables | cold | js | 160.216 | 162.943 | 0.014 | 6.3 | 0.57 |
| tables | cold | ruby | 91.815 | 93.552 | 0.015 | 10.9 | 1.00 |
| tables | cold | rust | 15.612 | 16.261 | 0.032 | 64.1 | 5.88 |
| unicode | warm | js | 12.074 | 12.957 | 0.039 | 82.4 | 0.86 |
| unicode | warm | ruby | 10.415 | 10.881 | 0.037 | 97.0 | 1.00 |
| unicode | warm | rust | 1.551 | 1.708 | 0.061 | 639.5 | 6.71 |
| unicode | cold | js | 139.649 | 142.276 | 0.011 | 7.2 | 0.64 |
| unicode | cold | ruby | 89.913 | 91.253 | 0.012 | 11.1 | 1.00 |
| unicode | cold | rust | 12.387 | 14.716 | 0.087 | 78.2 | 7.26 |
| paragraphs-100 | warm | js | 4.325 | 4.868 | 0.068 | 227.2 | 0.64 |
| paragraphs-100 | warm | ruby | 2.762 | 3.176 | 0.078 | 359.6 | 1.00 |
| paragraphs-100 | warm | rust | 0.612 | 0.693 | 0.070 | 1624.6 | 4.52 |
| paragraphs-100 | cold | js | 122.611 | 125.899 | 0.020 | 8.1 | 0.65 |
| paragraphs-100 | cold | ruby | 79.728 | 80.837 | 0.012 | 12.6 | 1.00 |
| paragraphs-100 | cold | rust | 9.872 | 10.687 | 0.052 | 100.6 | 8.08 |
| paragraphs-1000 | warm | js | 42.863 | 45.399 | 0.058 | 23.1 | 0.65 |
| paragraphs-1000 | warm | ruby | 27.954 | 30.465 | 0.037 | 35.5 | 1.00 |
| paragraphs-1000 | warm | rust | 5.186 | 5.879 | 0.050 | 189.5 | 5.39 |
| paragraphs-1000 | cold | js | 166.575 | 169.417 | 0.011 | 6.0 | 0.65 |
| paragraphs-1000 | cold | ruby | 108.514 | 111.180 | 0.013 | 9.2 | 1.00 |
| paragraphs-1000 | cold | rust | 17.026 | 19.707 | 0.063 | 57.8 | 6.37 |
| paragraphs-10000 | warm | js | 424.607 | 451.448 | 0.025 | 2.3 | 0.73 |
| paragraphs-10000 | warm | ruby | 309.812 | 1083.117 | 0.696 | 2.0 | 1.00 |
| paragraphs-10000 | warm | rust | 51.518 | 53.920 | 0.017 | 19.4 | 6.01 |
| paragraphs-10000 | cold | js | 621.699 | 645.198 | 0.039 | 1.6 | 0.62 |
| paragraphs-10000 | cold | ruby | 387.803 | 418.363 | 0.089 | 2.5 | 1.00 |
| paragraphs-10000 | cold | rust | 75.024 | 76.095 | 0.009 | 13.3 | 5.17 |
| bytes-1024 | warm | js | 1.049 | 1.297 | 0.113 | 933.6 | 0.58 |
| bytes-1024 | warm | ruby | 0.606 | 1.390 | 0.411 | 1429.3 | 1.00 |
| bytes-1024 | warm | rust | 0.180 | 0.221 | 0.106 | 5477.6 | 3.37 |
| bytes-1024 | cold | js | 116.062 | 117.592 | 0.011 | 8.6 | 0.65 |
| bytes-1024 | cold | ruby | 75.924 | 79.793 | 0.022 | 13.1 | 1.00 |
| bytes-1024 | cold | rust | 9.514 | 10.536 | 0.070 | 105.3 | 7.98 |
| bytes-10240 | warm | js | 8.166 | 9.526 | 0.078 | 120.5 | 0.63 |
| bytes-10240 | warm | ruby | 5.173 | 5.781 | 0.058 | 191.1 | 1.00 |
| bytes-10240 | warm | rust | 1.085 | 1.196 | 0.064 | 931.0 | 4.77 |
| bytes-10240 | cold | js | 130.169 | 132.070 | 0.013 | 7.7 | 0.63 |
| bytes-10240 | cold | ruby | 82.053 | 84.184 | 0.015 | 12.2 | 1.00 |
| bytes-10240 | cold | rust | 10.642 | 11.223 | 0.043 | 94.2 | 7.71 |
| bytes-102400 | warm | js | 74.898 | 81.844 | 0.037 | 13.2 | 0.73 |
| bytes-102400 | warm | ruby | 54.965 | 57.679 | 0.032 | 18.1 | 1.00 |
| bytes-102400 | warm | rust | 9.848 | 10.700 | 0.042 | 101.3 | 5.58 |
| bytes-102400 | cold | js | 205.783 | 218.767 | 0.034 | 4.8 | 0.66 |
| bytes-102400 | cold | ruby | 135.660 | 137.440 | 0.014 | 7.4 | 1.00 |
| bytes-102400 | cold | rust | 22.352 | 22.654 | 0.019 | 45.0 | 6.07 |
| bytes-1048576 | warm | js | 1147.194 | 1181.029 | 0.028 | 0.9 | 0.51 |
| bytes-1048576 | warm | ruby | 589.646 | 702.047 | 0.061 | 1.7 | 1.00 |
| bytes-1048576 | warm | rust | 102.182 | 105.552 | 0.012 | 9.8 | 5.77 |
| bytes-1048576 | cold | js | 1280.943 | 1312.899 | 0.016 | 0.8 | 0.54 |
| bytes-1048576 | cold | ruby | 696.768 | 1398.729 | 0.443 | 1.1 | 1.00 |
| bytes-1048576 | cold | rust | 136.334 | 137.842 | 0.007 | 7.3 | 5.11 |
| includes | warm | js | 0.472 | 0.610 | 0.164 | 2036.8 | 0.34 |
| includes | warm | ruby | 0.161 | 0.216 | 0.153 | 6022.6 | 1.00 |
| includes | warm | rust | 0.182 | 0.243 | 0.217 | 5598.8 | 0.88 |
| includes | cold | js | 116.577 | 119.673 | 0.015 | 8.6 | 0.66 |
| includes | cold | ruby | 76.904 | 78.431 | 0.017 | 13.0 | 1.00 |
| includes | cold | rust | 9.861 | 10.751 | 0.060 | 102.1 | 7.80 |
| include-selection | warm | js | 0.799 | 1.063 | 0.146 | 1211.7 | 0.28 |
| include-selection | warm | ruby | 0.225 | 0.313 | 0.720 | 3795.9 | 1.00 |
| include-selection | warm | rust | 0.121 | 0.188 | 0.175 | 7763.5 | 1.86 |
| include-selection | cold | js | 118.207 | 120.155 | 0.014 | 8.5 | 0.65 |
| include-selection | cold | ruby | 76.621 | 78.355 | 0.013 | 13.1 | 1.00 |
| include-selection | cold | rust | 10.290 | 11.300 | 0.055 | 97.2 | 7.45 |
| repeated-edits | warm | js | 35.605 | 36.785 | 0.040 | 28.0 | 0.46 |
| repeated-edits | warm | ruby | 16.256 | 17.961 | 0.049 | 61.1 | 1.00 |
| repeated-edits | warm | rust | 2.975 | 3.389 | 0.053 | 332.9 | 5.46 |
| repeated-edits | cold | js | 159.043 | 165.406 | 0.019 | 6.2 | 0.61 |
| repeated-edits | cold | ruby | 97.793 | 221.903 | 0.451 | 8.4 | 1.00 |
| repeated-edits | cold | rust | 13.053 | 13.822 | 0.034 | 76.1 | 7.49 |
| repeated-edits | edit | js | 35.499 | 37.007 | 0.020 | 28.1 | 0.45 |
| repeated-edits | edit | ruby | 16.076 | 18.191 | 0.041 | 61.8 | 1.00 |
| repeated-edits | edit | rust | 3.043 | 3.209 | 0.037 | 329.4 | 5.28 |

## Peak process memory

| Case | Engine | Peak RSS MiB |
|---|---|---:|
| admonitions | js | 94.0 |
| admonitions | ruby | 24.1 |
| admonitions | rust | 11.6 |
| attributes | js | 94.1 |
| attributes | ruby | 24.6 |
| attributes | rust | 14.4 |
| basic | js | 80.4 |
| basic | ruby | 22.8 |
| basic | rust | 12.7 |
| code | js | 129.6 |
| code | ruby | 29.9 |
| code | rust | 17.0 |
| conditionals | js | 92.4 |
| conditionals | ruby | 23.5 |
| conditionals | rust | 11.9 |
| features | js | 82.3 |
| features | ruby | 23.0 |
| features | rust | 11.9 |
| footnotes | js | 86.9 |
| footnotes | ruby | 23.3 |
| footnotes | rust | 12.3 |
| nested-lists | js | 93.9 |
| nested-lists | ruby | 23.4 |
| nested-lists | rust | 13.5 |
| sections | js | 94.0 |
| sections | ruby | 24.2 |
| sections | rust | 12.5 |
| tables | js | 110.2 |
| tables | ruby | 25.8 |
| tables | rust | 14.1 |
| unicode | js | 94.5 |
| unicode | ruby | 24.8 |
| unicode | rust | 14.0 |
| paragraphs-100 | js | 87.1 |
| paragraphs-100 | ruby | 23.0 |
| paragraphs-100 | rust | 13.2 |
| paragraphs-1000 | js | 110.9 |
| paragraphs-1000 | ruby | 25.1 |
| paragraphs-1000 | rust | 15.5 |
| paragraphs-10000 | js | 168.6 |
| paragraphs-10000 | ruby | 46.7 |
| paragraphs-10000 | rust | 44.7 |
| bytes-1024 | js | 81.6 |
| bytes-1024 | ruby | 23.0 |
| bytes-1024 | rust | 12.5 |
| bytes-10240 | js | 91.4 |
| bytes-10240 | ruby | 23.9 |
| bytes-10240 | rust | 13.0 |
| bytes-102400 | js | 131.7 |
| bytes-102400 | ruby | 27.0 |
| bytes-102400 | rust | 18.2 |
| bytes-1048576 | js | 196.7 |
| bytes-1048576 | ruby | 66.4 |
| bytes-1048576 | rust | 75.7 |
| includes | js | 79.8 |
| includes | ruby | 22.6 |
| includes | rust | 12.6 |
| include-selection | js | 80.9 |
| include-selection | ruby | 22.9 |
| include-selection | rust | 13.3 |
| repeated-edits | js | 111.4 |
| repeated-edits | ruby | 24.4 |
| repeated-edits | rust | 13.9 |
