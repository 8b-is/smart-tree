#!/usr/bin/env python3
"""Regression checks for publishing immutable CLI releases."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from publish_i1_release import publish


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='i1-release-tests-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / 'staged'
        self.source.mkdir()
        self.public = self.root / 'public'
        self.build = '8.1.0-0123456789ab'
        self.manifest = {'build_id': self.build, 'version': '8.1.0',
                         'cargo_features': ['full'], 'binaries': ['st', 'std', 'm8', 'n8x']}
        assets = []
        for name in ('source.tar.gz', 'st-x86_64-unknown-linux-gnu.tar.gz'):
            data = ('fixture for ' + name).encode()
            (self.source / name).write_bytes(data)
            digest = hashlib.sha256(data).hexdigest()
            (self.source / (name + '.sha256')).write_text(digest + '  ' + name + '\n')
            assets.append({'name': name, 'digest': 'sha256:' + digest, 'size': len(data),
                           'browser_download_url': f'https://i1.is/releases/smart-tree/{self.build}/{name}'})
        self.manifest.update(source=assets[0], assets=assets[1:])
        self.save_manifest()
        for name, data in [('build-id.txt', self.build), ('version.txt', '8.1.0'), ('BUILD_FEATURES.txt', 'full')]:
            (self.source / name).write_text(data + '\n')

    def save_manifest(self):
        (self.source / 'manifest.json').write_text(json.dumps(self.manifest))

    def promote(self):
        with contextlib.redirect_stdout(io.StringIO()):
            publish(self.source, self.public)

    def test_verified_promotion_is_idempotent_and_excludes_build_files(self):
        (self.source / 'private-build.log').write_text('not for publication')
        self.promote()
        self.promote()
        self.assertEqual((self.public / 'current').readlink(), Path(self.build))
        self.assertEqual((self.public / 'versions/8.1.0.txt').read_text().strip(), self.build)
        self.assertFalse((self.public / self.build / 'private-build.log').exists())

    def test_corrupt_payload_does_not_create_a_publication(self):
        (self.source / 'source.tar.gz').write_bytes(b'corrupt')
        with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
            self.promote()
        self.assertFalse(self.public.exists())

    def test_published_build_cannot_be_changed(self):
        self.promote()
        self.manifest['features'] = ['changed metadata']
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, 'immutable release'):
            self.promote()
        self.assertEqual((self.public / 'current').readlink(), Path(self.build))
        published = json.loads((self.public / 'current/manifest.json').read_text())
        self.assertNotIn('features', published)

    def test_asset_path_escape_is_rejected(self):
        self.manifest['source']['name'] = '../outside.tar.gz'
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, 'asset name'):
            self.promote()
        self.assertFalse(self.public.exists())

    def test_metadata_must_describe_the_installed_feature_set(self):
        (self.source / 'BUILD_FEATURES.txt').write_text('google\n')
        with self.assertRaisesRegex(ValueError, 'Metadata mismatch'):
            self.promote()
        self.assertFalse(self.public.exists())


if __name__ == '__main__':
    unittest.main()
