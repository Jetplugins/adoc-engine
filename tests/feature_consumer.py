#!/usr/bin/env python3
"""Test isolated consumers and optionally measure fresh release builds per feature set."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
PROFILES = {
    'core': ([], ['html', 'json']),
    'mathml': (['mathml'], ['html', 'json']),
    'epub': (['epub'], ['html', 'epub', 'json']),
    'highlighting': (['highlighting'], ['html', 'epub', 'json']),
    'pdf': (['pdf'], ['html', 'pdf', 'json']),
    'svg': (['svg'], ['html', 'svg', 'svg-merged', 'json']),
    'png': (['png'], ['html', 'png', 'json']),
    'full': (['full'], ['html', 'pdf', 'epub', 'svg', 'svg-merged', 'png', 'json']),
    'default': (None, ['html', 'pdf', 'epub', 'svg', 'svg-merged', 'png', 'json']),
}


def run(command, **kwargs):
    result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{' '.join(map(str, command))}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile', choices=PROFILES, default='core')
    parser.add_argument('--vendor-pdf', action='store_true', help='Also compile the optional original vendor PDF API (pdf profile only)')
    parser.add_argument('--measure', action='store_true', help='Use a fresh target directory and release optimization')
    parser.add_argument('--output', type=Path)
    parser.add_argument('--jobs', type=int, default=4)
    args = parser.parse_args()
    if args.vendor_pdf and args.profile != 'pdf':
        parser.error('--vendor-pdf requires --profile pdf')
    features, formats = PROFILES[args.profile]
    host = next(line.split(': ', 1)[1] for line in run(['rustc', '-vV']).splitlines() if line.startswith('host:'))
    with tempfile.TemporaryDirectory(prefix='adoc-feature-consumer-') as tmp:
        project = Path(tmp)
        dependency = {'path': str(ROOT/'crates/engine')}
        if features is not None:
            dependency.update({'default-features': False, 'features': features})
        def value(item):
            return str(item).lower() if isinstance(item, bool) else json.dumps(item)
        declaration = ', '.join(f'{key} = {value(item)}' for key, item in dependency.items())
        (project/'Cargo.toml').write_text('[package]\nname="adoc-feature-consumer"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\nadoc-engine = { '+declaration+' }\n[profile.release]\nlto="thin"\ncodegen-units=1\n')
        if args.vendor_pdf:
            with (project/'Cargo.toml').open('a') as manifest_file:
                manifest_file.write('\n[dependencies.adocers-typst]\npath='+json.dumps(str(ROOT/'vendor/adocers-typst'))+'\ndefault-features=false\nfeatures=["pdf","math"]\n')
        (project/'src').mkdir()
        shutil.copyfile(ROOT/'tests/fixtures/feature_consumer/main.rs', project/'src/main.rs')
        if args.vendor_pdf:
            source = (project/'src/main.rs').read_text().replace('fn main() {', 'fn main() { let _vendor_pdf_api = adocers_typst::pdf;', 1)
            (project/'src/main.rs').write_text(source)
        shutil.copyfile(ROOT/'Cargo.lock', project/'Cargo.lock')
        env = os.environ.copy()
        env['CARGO_INCREMENTAL'] = '0'
        if args.measure:
            env['RUSTC_WRAPPER'] = ''
            env['RUSTC_WORKSPACE_WRAPPER'] = ''
        env['CARGO_TARGET_DIR'] = str(project/'target' if args.measure else ROOT/'target')
        manifest = ['--manifest-path', str(project/'Cargo.toml')]
        # The standalone workspace prevents the full CLI from unifying engine features.
        tree = run(['cargo', 'tree', '--offline', *manifest, '--target', host, '--edges', 'normal', '--prefix', 'none', '--format', '{p}'], env=env)
        packages = {line.removesuffix(' (*)') for line in tree.splitlines()}
        names = {line.split()[0] for line in packages}
        if args.profile in ('core', 'mathml', 'epub', 'highlighting'):
            assert not any(name == 'typst' or name.startswith('typst-') for name in names), tree
        if args.profile not in ('pdf', 'full', 'default'):
            assert 'typst-pdf' not in names, tree
        wrappers = {line.split()[1] for line in packages if line.startswith('typst-as-lib ')}
        assert len(wrappers) <= 1 and 'v0.16.0' not in wrappers, wrappers
        if args.profile == 'core':
            assert not {'zip', 'syntect', 'math-core', 'mathemascii'} & names, tree
        if args.profile == 'epub':
            assert 'syntect' not in names, tree
        command = ['cargo', 'build', '--offline', *manifest, '--jobs', str(args.jobs)]
        if args.measure: command.append('--release')
        started = time.perf_counter()
        run(command, env=env)
        elapsed = time.perf_counter() - started
        binary = Path(env['CARGO_TARGET_DIR'])/('release' if args.measure else 'debug')/('adoc-feature-consumer.exe' if os.name == 'nt' else 'adoc-feature-consumer')
        mathml = args.profile in ('mathml', 'epub', 'highlighting', 'full', 'default')
        highlighting = args.profile in ('highlighting', 'full', 'default')
        print(run([str(binary), ','.join(formats), str(mathml).lower(), str(highlighting).lower()]).strip(), flush=True)
        fingerprint = hashlib.sha256()
        inputs = [ROOT/'Cargo.toml', ROOT/'Cargo.lock', ROOT/'tests/fixtures/feature_consumer/main.rs']
        for base in ['crates/engine', 'vendor']:
            inputs.extend(path for path in (ROOT/base).rglob('*') if path.is_file() and path.suffix in ('.rs', '.toml', '.css', '.otf', '.ttf'))
        for path in sorted(inputs):
            fingerprint.update(str(path.relative_to(ROOT)).encode())
            fingerprint.update(path.read_bytes())
        report = {'sourceTreeSha256': fingerprint.hexdigest(), 'consumerSourceSha256': hashlib.sha256((project/'src/main.rs').read_bytes()).hexdigest(), 'vendorPdf': args.vendor_pdf, 'profile': args.profile, 'features': features, 'formats': formats,
                  'host': host, 'platform': platform.platform(), 'rustc': run(['rustc', '--version']).strip(),
                  'dependencyPackages': len(packages)-2,
                  'release': args.measure, 'freshTarget': args.measure, 'jobs': args.jobs,
                  'buildSeconds': round(elapsed, 3), 'binaryBytes': binary.stat().st_size,
                  'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                  'sourceCommit': run(['git', 'rev-parse', 'HEAD'], cwd=ROOT).strip(),
                  'workingTreeDirty': bool(run(['git', 'status', '--porcelain'], cwd=ROOT).strip()),
                  'lockfileSha256': hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest(),
                  'method': 'One build; dependencies already downloaded. Fresh release target when --measure; thin LTO, one codegen unit. Compiler wrappers disabled for measured builds. Unstripped dynamic-format consumer retains each enabled export backend.'}
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
