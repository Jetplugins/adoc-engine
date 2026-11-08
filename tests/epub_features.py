#!/usr/bin/env python3
"""Generate real books with native/Ruby exporters; independently validate every archive.

Pin the reference gems as documented in docs/epub.md. Kotlin artifacts can be
supplied from the plugin's EpubComparisonTest without depending on that repository.
"""
import argparse
import hashlib
import re
import json
import os
import platform
import shutil
import statistics
import subprocess
import time
from pathlib import Path
from epub_contract import verify, check_case

ROOT = Path(__file__).resolve().parents[1]
CASES = {
    'book': ('book.adoc', {}), 'article': ('article.adoc', {}), 'media': ('media.adoc', {}),
    'depth': ('book.adoc', {'epub-chapter-level': '2'}),
    'custom': ('book.adoc', {'epub3-frontmatterdir': 'frontmatter', 'epub3-stylesdir': 'styles'}),
    'reader-fonts': ('book.adoc', {'epub-fonts': 'none'}),
}

def run(command, log=None):
    process = subprocess.run([str(c) for c in command], text=True, stdout=subprocess.PIPE,
                             stderr=subprocess.STDOUT, timeout=180)
    if log: log.write_text(process.stdout)
    if process.returncode: raise RuntimeError(f'{command}:\n{process.stdout}')
    return process.stdout.strip()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary', type=Path); p.add_argument('--ruby', default='ruby')
    p.add_argument('--reference', default='asciidoctor-epub3')
    p.add_argument('--epubcheck', type=Path, required=True)
    p.add_argument('--kotlin', type=Path); p.add_argument('--samples', type=int, default=0)
    p.add_argument('--output', type=Path, default=ROOT / 'verification/epub')
    args = p.parse_args(); output = args.output.resolve(); output.mkdir(parents=True, exist_ok=True)
    inputs = output / 'inputs'; shutil.copytree(ROOT / 'tests/fixtures/epub', inputs, dirs_exist_ok=True)
    font = inputs / 'styles/fonts/book.otf'; font.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / 'crates/engine/src/epub/fonts/LibertinusSerif-Regular.otf', font)
    # Adds a larger document without omitting features to make speed measurements useful.
    if args.samples:
        original = (inputs / 'book.adoc').read_text()
        (inputs / 'large.adoc').write_text(original + ''.join(f'\n\n== Performance chapter {i}\n\n' + ('A paragraph with *bold*, _emphasis_, `code` and café.\n\n' * 40) for i in range(1, 41)))
    cases = dict(CASES)
    if args.samples: cases['large'] = ('large.adoc', {})
    digest = hashlib.sha256()
    for file in sorted(p for base in ('crates', 'vendor') for p in (ROOT/base).rglob('*') if p.is_file() and p.suffix in ('.rs', '.toml', '.css', '.otf', '.ttf')):
        digest.update(str(file.relative_to(ROOT)).encode()); digest.update(file.read_bytes())
    digest.update((ROOT/'Cargo.lock').read_bytes())
    report = {'sourceTreeSha256': digest.hexdigest(), 'platform': platform.platform(), 'cpu': platform.processor(),
              'native': run([args.binary.resolve(), '--version']),
              'rubyRuntime': run([args.ruby, '--version']),
              'reference': run([args.ruby, args.reference, '--version']),
              'method': 'Full CLI process per sample; includes startup, parsing/includes, assets, font compression, archive creation and file writes. Two warmups; interleaved engines. EPUBCheck excluded.',
              'binarySha256': hashlib.sha256(args.binary.read_bytes()).hexdigest(),
              'fixtureSha256': {str(p.relative_to(inputs)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(inputs.rglob('*')) if p.is_file() and p.suffix in ('.adoc', '.html', '.css', '.png', '.jpg', '.svg', '.mp3', '.mp4') and not any(s in ('rust', 'ruby') for s in p.relative_to(inputs).parts)},
              'samples': args.samples, 'results': {}}
    for case, (source, attributes) in cases.items():
        commands = {}; paths = {}
        for engine in ('rust', 'ruby'):
            if engine == 'ruby' and case == 'reader-fonts': continue # native extension, not a reference attribute
            directory = inputs / engine; directory.mkdir(exist_ok=True); path = directory / f'{case}.epub'; paths[engine] = path
            if engine == 'rust':
                command = [args.binary.resolve(), 'render', inputs/source, '-f', 'epub', '--safe-mode', 'safe', '--base-dir', inputs, '-o', path]
            else: command = [args.ruby, args.reference, '-S', 'safe', '-B', inputs, '-o', path]
            for key, value in attributes.items(): command += ['-a', f'{key}={value}']
            if engine == 'ruby': command += [inputs/source]
            commands[engine] = command
        failures = {}
        measurements = {e: [] for e in commands}
        for path in paths.values(): path.unlink(missing_ok=True)
        for iteration in range(args.samples + 2 if args.samples else 1):
            for engine in (list(commands) if iteration % 2 == 0 else list(reversed(commands))):
                if engine in failures: continue
                started = time.perf_counter_ns()
                try: run(commands[engine], paths[engine].with_suffix('.conversion.txt'))
                except RuntimeError as error:
                    if engine != 'ruby' or case != 'custom' or 'epub3.scss' not in str(error): raise
                    failures[engine] = {'conversionError': str(error)}
                    continue
                if args.samples and iteration >= 2: measurements[engine].append((time.perf_counter_ns()-started)/1e6)
        if args.kotlin and (args.kotlin/f'{case}.epub').exists(): paths['kotlin'] = args.kotlin/f'{case}.epub'
        result = dict(failures)
        for engine, path in paths.items():
            if engine in failures: continue
            record = verify(path, strict=engine != 'ruby')
            check_case(record, 'book' if case == 'large' else case, reference=engine == 'ruby')
            check = subprocess.run(['java', '-jar', str(args.epubcheck.resolve()), str(path)], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=120)
            path.with_suffix('.epubcheck.txt').write_text(check.stdout)
            if engine != 'ruby': assert check.returncode == 0, check.stdout
            elif check.returncode:
                errors = [line for line in check.stdout.splitlines() if line.startswith(('ERROR', 'FATAL'))]
                assert errors and all('ERROR(RSC-012)' in line and 'Fragment identifier is not defined' in line for line in errors), check.stdout
                assert record['linkErrors'] and all("'#note-1'" in error for error in record['linkErrors']), record['linkErrors']
            record['epubcheck'] = 'passed' if check.returncode == 0 else 'failed (reference output; see log)'
            record['epubcheckLog'] = check.stdout
            if engine in measurements and measurements[engine]:
                record['samplesMs'] = measurements[engine]; record['medianMs'] = statistics.median(measurements[engine])
            result[engine] = record
        report['results'][case] = result
        (output/'results.json').write_text(json.dumps(report, indent=2, ensure_ascii=False)+'\n')
        print(f'{case}: ' + ', '.join(f'{engine}: ' + ('conversion failed (reference)' if 'conversionError' in result[engine] else result[engine]['epubcheck']) for engine in result), flush=True)
    print(output/'results.json')
if __name__ == '__main__': main()
