import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('release', ROOT / 'scripts/release.py')
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / 'crates/cli').mkdir(parents=True)
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion="0.2.0"\n')
        (self.root / 'crates/cli/Cargo.toml').write_text('[dependencies.adoc-engine]\nversion="0.2.0"\n')

    def tearDown(self):
        self.tmp.cleanup()

    def test_version_matches_tag_with_or_without_v(self):
        for tag in ['v0.2.0', '0.2.0']:
            self.assertEqual(release.validate(tag, self.root), '0.2.0')

    def test_invalid_or_mismatched_tag_is_rejected(self):
        for tag in ['v0.1.0', 'main', '../0.2.0', 'v0.2.0;evil']:
            with self.assertRaises(ValueError): release.validate(tag, self.root)

    def test_cli_version_must_match(self):
        (self.root / 'crates/cli/Cargo.toml').write_text('[dependencies.adoc-engine]\nversion="0.1.0"\n')
        with self.assertRaises(ValueError): release.validate('v0.2.0', self.root)

    def archives(self):
        for target in release.TARGETS:
            name = f'adoc-engine-0.2.0-{target}'
            metadata = json.dumps({'version': '0.2.0', 'target': target, 'commit': 'verified-source'}).encode()
            archive = self.root / (name + ('.zip' if target.endswith('windows-msvc') else '.tar.gz'))
            if archive.suffix == '.zip':
                with zipfile.ZipFile(archive, 'w') as bundle: bundle.writestr(name + '/BUILD.json', metadata)
            else:
                with tarfile.open(archive, 'w:gz') as bundle:
                    entry = tarfile.TarInfo(name + '/BUILD.json'); entry.size = len(metadata)
                    bundle.addfile(entry, io.BytesIO(metadata))
            archive.with_name(archive.name + '.sha256').write_text(hashlib.sha256(archive.read_bytes()).hexdigest() + '  ' + archive.name + '\n')

    def collect(self):
        with patch.object(release, 'validate', return_value='0.2.0'):
            return release.collect(self.root, 'v0.2.0', 'verified-source')

    def test_all_five_platforms_are_collected(self):
        self.archives(); manifest = self.collect()
        self.assertEqual(len(manifest['assets']), 5)
        self.assertEqual(manifest['workerProtocolVersion'], 1)

    def test_missing_platform_is_rejected(self):
        self.archives(); next(self.root.glob('*.zip')).unlink()
        with self.assertRaises(ValueError): self.collect()

    def test_wrong_checksum_is_rejected(self):
        self.archives(); next(self.root.glob('*.sha256')).write_text('0' * 64 + '  wrong-name\n')
        with self.assertRaises(ValueError): self.collect()

    def test_wrong_source_commit_is_rejected(self):
        self.archives()
        with patch.object(release, 'validate', return_value='0.2.0'):
            with self.assertRaises(ValueError): release.collect(self.root, 'v0.2.0', 'different-source')


if __name__ == '__main__': unittest.main()
