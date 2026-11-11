#!/usr/bin/env python3
"""Opt-in differential verification of continued admonitions against Ruby/JS.

Requires the pinned benchmark reference runtimes. Run from the repository with:
  python3 tests/list_continuations.py target/release/adoc-engine
"""
import argparse
import hashlib
import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'benchmarks'))
from run import Worker, canonical, comparison_tokens


def cases():
    labels = ['NOTE', 'TIP', 'IMPORTANT', 'CAUTION', 'WARNING']
    for label in labels:
        for marker in ['.', '*', '-', '1.']:
            following = '2.' if marker == '1.' else marker
            for form in ['prefix', 'style']:
                admonition = f'{label}: Keep the previous release.\nWait for health checks.' if form == 'prefix' else f'[{label}]\nKeep the previous release.\nWait for health checks.'
                source = f'{marker} Deploy the application.\n+\n{admonition}\n{following} Promote the verified release.\n'
                for endings in ['lf', 'crlf']:
                    yield f'{label}-{marker}-{form}-{endings}', source if endings == 'lf' else source.replace('\n', '\r\n')
        for form in ['prefix', 'style']:
            admonition = f'{label}: Keep the previous release.' if form == 'prefix' else f'[{label}]\nKeep the previous release.'
            yield f'{label}-top-level-{form}', f'{admonition}\n. This is still paragraph text.\n'
        for delimiter in ['====', '--']:
            yield f'{label}-compound-{delimiter}', f'. Deploy.\n+\n[{label}]\n{delimiter}\nKeep the previous release.\n\n. Internal step.\n{delimiter}\n. Promote the verified release.\n'
        yield f'{label}-ancestor', f'. Deploy.\n.. Check workers.\n+\n{label}: Keep the previous release.\n. Promote the verified release.\n'
    for name, separator in [('immediate', '\n'), ('blank', '\n\n'), ('comment', '\n// Keep the next item.\n'), ('indented-marker', '\n  ')]:
        yield f'boundary-{name}', '. Deploy.\n+\nTIP: Keep the previous release.' + separator + '. Promote the verified release.\n'
    yield 'showcase', (ROOT / 'benchmarks/fixtures/list-continuations.adoc').read_text()
    yield 'raw-literal', '. Deploy.\n+\n....\nTIP: This is raw text.\n. Also raw text.\n....\n. Promote the verified release.\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--node', default=shutil.which('node'))
    parser.add_argument('--ruby', default=shutil.which('ruby'))
    parser.add_argument('--output', type=Path, default=ROOT / 'verification/list-continuations/reference')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    workers = {
        'ruby': Worker([args.ruby, str(ROOT / 'benchmarks/ruby-worker.rb')]),
        'js': Worker([args.node, str(ROOT / 'benchmarks/js-worker.cjs')]),
        'rust': Worker([str(args.binary.resolve()), 'worker']),
    }
    results = []
    try:
        for index, (name, source) in enumerate(cases()):
            request = {'source': source, 'options': {'safeMode': 'secure', 'attributes': {}}}
            outputs = {engine: worker.render(request)['html'] for engine, worker in workers.items()}
            reference = canonical(outputs['ruby'])
            parity = {engine: comparison_tokens(engine, output) == reference for engine, output in outputs.items()}
            results.append({'case': name, 'sourceSha256': hashlib.sha256(source.encode()).hexdigest(), 'parity': parity})
            if not all(parity.values()):
                (args.output / f'failure-{index}.adoc').write_text(source)
                for engine, output in outputs.items():
                    (args.output / f'failure-{index}-{engine}.html').write_text(output)
                print('Parity mismatch:', name, parity)
    finally:
        for worker in workers.values():
            worker.close()
    report = {'binarySha256': hashlib.sha256(args.binary.read_bytes()).hexdigest(), 'cases': len(results), 'passed': sum(all(result['parity'].values()) for result in results), 'results': results}
    (args.output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f"Reference parity: {report['passed']}/{report['cases']} cases across Ruby, JavaScript and Rust")
    if report['passed'] != report['cases']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
