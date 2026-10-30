"""Offline installer regression tests: python3 -m unittest discover -s tests/installer -v."""
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

INSTALLER = Path(__file__).resolve().parents[2] / 'scripts/install.sh'
REPO = 'https://github.com/8b-is/smart-tree/releases/download'
TARGET = 'x86_64-unknown-linux-gnu'


class InstallerTests(unittest.TestCase):
    def run_installer(self, names=None, tag='v10.6.0', version=None,
                      system='Linux', arch='x86_64', raw=None, api_failure=False,
                      download_failure=False, bad_url=False):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            mock_bin = root / 'mock'
            install = root / 'install'
            mock_bin.mkdir()
            install.mkdir()
            binary = install / 'st'
            binary.write_text('existing binary\n')
            fixture = root / 'release.json'
            assets = [{'name': n, 'browser_download_url': f'{REPO}/{tag}/{n}'}
                      for n in (names or [])]
            if bad_url:
                assets[0]['browser_download_url'] = f'{REPO}/v1.0.0/{names[0]}'
            fixture.write_text(raw if raw is not None else json.dumps(
                {'tag_name': tag, 'assets': assets}))
            payload = b'#!/bin/sh\necho "st mocked release"\n'
            with tarfile.open(root / 'binary.tar.gz', 'w:gz') as archive:
                info = tarfile.TarInfo('st')
                info.size = len(payload)
                info.mode = 0o755
                archive.addfile(info, io.BytesIO(payload))
            (mock_bin / 'uname').write_text(
                '#!/bin/bash\nif [[ "$1" == -s ]]; then echo "$MOCK_OS"; else echo "$MOCK_ARCH"; fi\n')
            (mock_bin / 'curl').write_text('''#!/bin/bash
set -eu
url=""; output=""
while (($#)); do
    case "$1" in
        https://*) url="$1" ;;
        -o) shift; output="$1" ;;
    esac
    shift
done
echo "$url" >> "$MOCK_ROOT/requests"
if [[ "$url" == https://api.github.com/* ]]; then
    [[ "$MOCK_API_FAILURE" == 0 ]] || exit 22
    cat "$MOCK_ROOT/release.json"
else
    [[ "$MOCK_DOWNLOAD_FAILURE" == 0 ]] || exit 22
    cp "$MOCK_ROOT/binary.tar.gz" "$output"
fi
''')
            for path in mock_bin.iterdir():
                path.chmod(0o755)
            env = dict(os.environ, PATH=f'{mock_bin}:' + os.environ['PATH'],
                       INSTALL_DIR=str(install), MOCK_ROOT=tmp, MOCK_OS=system,
                       MOCK_ARCH=arch, MOCK_API_FAILURE=str(int(api_failure)),
                       MOCK_DOWNLOAD_FAILURE=str(int(download_failure)))
            env.pop('VERSION', None)
            if version is not None:
                env['VERSION'] = version
            result = subprocess.run(['bash', str(INSTALLER)], env=env, input='',
                                    text=True, capture_output=True, timeout=10)
            requests = (root / 'requests').read_text().splitlines()
            return result, requests, binary.read_text()

    def assert_success(self, name, **kwargs):
        result, requests, binary = self.run_installer([name], **kwargs)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(len(requests), 2)
        self.assertEqual(requests[-1], f'{REPO}/{kwargs.get("tag", "v10.6.0")}/{name}')
        self.assertIn('mocked release', binary)
        return requests

    def assert_failure(self, message, **kwargs):
        result, requests, binary = self.run_installer(**kwargs)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(message, result.stderr)
        self.assertEqual(binary, 'existing binary\n')
        self.assertEqual(len(requests), 1)  # No archive download or release fallback.
        return requests

    def test_current_unversioned_latest(self):
        requests = self.assert_success(f'st-{TARGET}.tar.gz')
        self.assertTrue(requests[0].endswith('/releases/latest'))

    def test_explicit_version(self):
        requests = self.assert_success(f'st-{TARGET}.tar.gz', version='v10.6.0')
        self.assertTrue(requests[0].endswith('/releases/tags/v10.6.0'))

    def test_historical_versioned_names(self):
        for tag, prefix in [('v5.0.8', 'v5.0.8'), ('v5.0.8', '5.0.8'), ('5.0.8', '5.0.8')]:
            with self.subTest(tag=tag, prefix=prefix):
                self.assert_success(f'st-{prefix}-{TARGET}.tar.gz', tag=tag, version=tag)

    def test_missing_platforms(self):
        for system, arch in [('Darwin', 'x86_64'), ('Darwin', 'arm64'), ('Linux', 'aarch64')]:
            with self.subTest(system=system, arch=arch):
                self.assert_failure('no supported published asset',
                                    names=[f'st-{TARGET}.tar.gz'], system=system, arch=arch)

    def test_empty_assets_explicit_version(self):
        self.assert_failure('no supported published asset', version='v10.6.0')

    def test_wrong_version_asset(self):
        self.assert_failure('no supported published asset', names=[f'st-v9.0.0-{TARGET}.tar.gz'])

    def test_wrong_release_returned(self):
        self.assert_failure('refusing to install another version', tag='v9.0.0', version='v10.6.0')

    def test_wrong_release_url(self):
        self.assert_failure('no supported published asset', names=[f'st-{TARGET}.tar.gz'], bad_url=True)

    def test_api_http_and_network_failures(self):
        for version in [None, 'v10.6.0']:
            with self.subTest(version=version):
                self.assert_failure('HTTP or network failure', api_failure=True, version=version)

    def test_invalid_api_metadata(self):
        for raw in ['not json', '{"message":"API rate limit exceeded"}',
                    '{"tag_name":"v10.6.0","assets":null}',
                    '{"tag_name":"v10.6.0","assets":[{}]}', '{}\n{}']:
            with self.subTest(raw=raw):
                self.assert_failure('Invalid GitHub release metadata', raw=raw)

    def test_download_failure_preserves_binary(self):
        result, requests, binary = self.run_installer([f'st-{TARGET}.tar.gz'], download_failure=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Download failed', result.stderr)
        self.assertEqual(len(requests), 2)
        self.assertEqual(binary, 'existing binary\n')

    def test_mixed_inventory_selects_exact_target(self):
        name = f'st-{TARGET}.tar.gz'
        result, requests, _ = self.run_installer([
            'st-aarch64-apple-darwin.tar.gz',
            f'st-v9.0.0-{TARGET}.tar.gz', name, name + '.sha256'])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(requests[-1], f'{REPO}/v10.6.0/{name}')

    def test_duplicate_asset_is_rejected(self):
        name = f'st-{TARGET}.tar.gz'
        self.assert_failure('no supported published asset', names=[name, name])

    def test_available_arm_and_macos_assets(self):
        for system, arch, target in [('Linux', 'arm64', 'aarch64-unknown-linux-gnu'),
                                     ('Darwin', 'arm64', 'aarch64-apple-darwin'),
                                     ('Darwin', 'x86_64', 'x86_64-apple-darwin')]:
            with self.subTest(target=target):
                self.assert_success(f'st-{target}.tar.gz', system=system, arch=arch)


if __name__ == '__main__':
    unittest.main()
