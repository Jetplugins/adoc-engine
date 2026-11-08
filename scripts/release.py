#!/usr/bin/env python3
"""Validate a release tag and collect the five verified native archives."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TARGETS = {
    'aarch64-apple-darwin', 'x86_64-apple-darwin',
    'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-gnu',
    'x86_64-pc-windows-msvc',
}


def version(root=ROOT):
    return tomllib.loads((root / 'Cargo.toml').read_text())['workspace']['package']['version']


def validate(tag, root=ROOT):
    value = version(root)
    if not re.fullmatch(r'v?\d+\.\d+\.\d+(?:-[\w.-]+)?', tag) or tag.removeprefix('v') != value:
        raise ValueError(f'Release tag {tag!r} must match Cargo version {value}')
    cli = tomllib.loads((root / 'crates/cli/Cargo.toml').read_text())
    if cli['dependencies']['adoc-engine']['version'] != value:
        raise ValueError('CLI library dependency version must match the workspace')
    return value


def build_metadata(archive):
    if archive.suffix == '.zip':
        with zipfile.ZipFile(archive) as bundle:
            names = [name for name in bundle.namelist() if name.count('/') == 1 and name.endswith('/BUILD.json')]
            if len(names) != 1:
                raise ValueError(f'Expected one BUILD.json in {archive}')
            return json.loads(bundle.read(names[0]))
    with tarfile.open(archive) as bundle:
        entries = [item for item in bundle.getmembers() if item.isfile() and item.name.count('/') == 1 and item.name.endswith('/BUILD.json')]
        if len(entries) != 1:
            raise ValueError(f'Expected one BUILD.json in {archive}')
        return json.load(bundle.extractfile(entries[0]))


def collect(directory, tag, commit):
    value = validate(tag)
    archives = sorted([*directory.rglob('*.tar.gz'), *directory.rglob('*.zip')])
    entries = []
    seen = set()
    for archive in archives:
        # Verification report ZIPs are not engine distributions.
        if not archive.name.startswith('adoc-engine-'):
            continue
        metadata = build_metadata(archive)
        target = metadata['target']
        if target not in TARGETS or target in seen:
            raise ValueError(f'Duplicate or unsupported release target: {target}')
        expected = f'adoc-engine-{value}-{target}' + ('.zip' if target.endswith('windows-msvc') else '.tar.gz')
        if archive.name != expected or metadata['version'] != value or metadata['commit'] != commit:
            raise ValueError(f'Archive provenance mismatch: {archive}')
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        checksum = archive.with_name(archive.name + '.sha256').read_text().strip().split()
        if checksum != [digest, archive.name]:
            raise ValueError(f'Archive checksum mismatch: {archive}')
        seen.add(target)
        entries.append({'name': archive.name, 'target': target, 'sha256': digest, 'size': archive.stat().st_size})
    if seen != TARGETS:
        raise ValueError(f'Missing platform distributions: {sorted(TARGETS-seen)}')
    return {'schemaVersion': 1, 'workerProtocolVersion': 1, 'version': value, 'tag': tag, 'commit': commit, 'assets': entries}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['validate', 'collect'])
    parser.add_argument('--tag', required=True)
    parser.add_argument('--artifacts', type=Path, default=Path('artifacts'))
    parser.add_argument('--output', type=Path, default=Path('release-manifest.json'))
    args = parser.parse_args()
    value = validate(args.tag)
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if args.command == 'collect':
        args.output.write_text(json.dumps(collect(args.artifacts, args.tag, commit), indent=2) + '\n')
    print(json.dumps({'version': value, 'tag': args.tag, 'commit': commit}))


if __name__ == '__main__':
    main()
