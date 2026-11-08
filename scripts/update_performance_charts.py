#!/usr/bin/env python3
"""Refresh README Mermaid charts from the archived, verified benchmark samples."""
import argparse
import json
import math
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parents[1]
ENGINES = ['js', 'ruby', 'rust']


def load(relative):
    report = json.loads((ROOT / relative).read_text())
    if not all(case['eligible'] for case in report['metadata']['cases']):
        raise ValueError(f'Unverified benchmark corpus: {relative}')
    return report


def medians(report, case, mode):
    return [statistics.median([
        sample['nanos'] for sample in report['samples']
        if sample['case'] == case and sample['mode'] == mode and sample['engine'] == engine
    ]) / 1e6 for engine in ENGINES]


def chart(title, axis, values, step, precision=3, headroom=1.05):
    if any(not math.isfinite(value) or value < 0 for value in values):
        raise ValueError('Chart values must be finite and nonnegative')
    maximum = math.ceil(max(values) * headroom / step) * step
    numbers = ', '.join(f'{value:.{precision}f}' for value in values)
    return '\n'.join([
        '```mermaid',
        '%%{init: ' + json.dumps({
            'xyChart': {'width': 700, 'height': 300},
            'themeVariables': {'xyChart': {'plotColorPalette': '#2f81f7'}},
        }) + '}%%',
        'xychart-beta',
        f'    title "{title}"',
        '    x-axis [JavaScript, Ruby, Rust]',
        f'    y-axis "{axis}" 0 --> {maximum:g}',
        f'    bar [{numbers}]',
        '```',
    ])


def replace_block(text, marker, content, anchor, before=False):
    start, end = f'<!-- {marker}:START -->', f'<!-- {marker}:END -->'
    block = start + '\n' + content + '\n' + end
    if start in text:
        left, tail = text.split(start, 1)
        _, right = tail.split(end, 1)
        return left + block + right
    if text.count(anchor) != 1:
        raise ValueError(f'Missing or ambiguous insertion point: {anchor}')
    return text.replace(anchor, block + '\n\n' + anchor if before else anchor + '\n\n' + block, 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Report stale charts without writing')
    args = parser.parse_args()
    release = load('benchmarks/baselines/release/raw.json')
    showcase = load('benchmarks/baselines/showcase-compatibility/showcase-performance/raw.json')
    release_chart = '\n\n'.join([
        '### Large-document conversion',
        'The v0.1.0 release benchmark for a roughly 1 MiB document; lower is better. '
        'Times measure warm conversion only, excluding IPC and browser painting.',
        chart('1 MiB document: warm conversion (v0.1.0)', 'Median time (ms)',
              medians(release, 'bytes-1048576', 'warm'), 200, headroom=1.02),
    ])
    memory = [next(sample['peakRssBytes'] / 1048576 for sample in showcase['memory']
                   if sample['case'] == 'showcase' and sample['engine'] == engine) for engine in ENGINES]
    showcase_charts = '\n\n'.join([
        'Lower is better in each chart. These measurements use the corrected, '
        'unreleased showcase build described above.',
        chart('Showcase: warm conversion', 'Median time (ms)', medians(showcase, 'showcase', 'warm'), 2),
        chart('Showcase: cold process execution', 'Median time (ms)', medians(showcase, 'showcase', 'cold'), 20),
        chart('Showcase: peak process memory', 'Peak RSS (MiB)', memory, 10, precision=1),
    ])
    readme = ROOT / 'README.md'
    original = readme.read_text()
    updated = replace_block(original, 'RELEASE-PERF-CHARTS', release_chart, '<!-- PERFORMANCE:END -->')
    updated = replace_block(updated, 'SHOWCASE-PERF-CHARTS', showcase_charts,
                            'A separate comparison with v0.1.0 collected 800 interleaved warm Rust conversion', before=True)
    if args.check:
        if updated != original:
            raise SystemExit('README charts are stale; run python3 scripts/update_performance_charts.py')
        print('README charts match the verified benchmark samples.')
    else:
        readme.write_text(updated)
        print('Updated four README Mermaid XY charts.')


if __name__ == '__main__':
    main()
