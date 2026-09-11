#!/usr/bin/env python3
"""Verify a packaged i1 release and atomically promote its current-build pointer."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tempfile


def sha256(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def publish(source, root):
    manifest = json.loads((source / 'manifest.json').read_text())
    build = manifest['build_id']
    version = manifest['version']
    if not re.fullmatch(r'\d+\.\d+\.\d+-[a-f0-9]{12}', build) or build.rsplit('-', 1)[0] != version:
        raise ValueError('Invalid version or immutable build ID')
    if manifest['cargo_features'] != ['full'] or manifest['binaries'] != ['st', 'std', 'm8', 'n8x']:
        raise ValueError('Expected the supported full-feature binary set')
    assets = [manifest['source'], *manifest['assets']]
    if len(assets) < 2:
        raise ValueError('Publish at least one binary with the matching source')
    names = {'manifest.json', 'build-id.txt', 'version.txt', 'BUILD_FEATURES.txt'}
    for asset in assets:
        name = asset['name']
        if name != 'source.tar.gz' and not re.fullmatch(r'st-(aarch64|x86_64)-(apple-darwin|unknown-linux-gnu)\.tar\.gz', name):
            raise ValueError('Unexpected release asset name')
        expected_url = f'https://i1.is/releases/smart-tree/{build}/{name}'
        if asset['browser_download_url'] != expected_url:
            raise ValueError('Asset URL does not identify this build')
        path = source / name
        if path.is_symlink() or not path.is_file() or asset['digest'] != 'sha256:' + sha256(path):
            raise ValueError(f'Asset checksum mismatch: {name}')
        if asset['size'] != path.stat().st_size:
            raise ValueError(f'Asset size mismatch: {name}')
        if (source / (name + '.sha256')).read_text().split()[0] != sha256(path):
            raise ValueError(f'Sidecar checksum mismatch: {name}')
        names.update((name, name + '.sha256'))
    for name, expected in [('build-id.txt', build), ('version.txt', version), ('BUILD_FEATURES.txt', 'full')]:
        if (source / name).read_text().strip() != expected:
            raise ValueError(f'Metadata mismatch: {name}')
    for name in names:
        if (source / name).is_symlink() or not (source / name).is_file():
            raise ValueError(f'Expected a regular release file: {name}')

    root.mkdir(parents=True, exist_ok=True)
    destination = root / build
    if destination.exists():
        if destination.is_symlink() or {p.name for p in destination.iterdir()} != names:
            raise ValueError('Existing immutable release has different files')
        for name in names:
            if sha256(destination / name) != sha256(source / name):
                raise ValueError(f'Refusing to change an immutable release: {name}')
    else:
        stage = Path(tempfile.mkdtemp(prefix='.publish-', dir=root))
        try:
            for name in sorted(names):
                shutil.copyfile(source / name, stage / name)
                (stage / name).chmod(0o644)
            stage.chmod(0o755)
            stage.rename(destination)
        finally:
            if stage.exists():
                shutil.rmtree(stage)
    versions = root / 'versions'
    versions.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w', dir=versions, delete=False) as pointer:
        pointer.write(build + '\n')
        pointer.flush()
        os.fsync(pointer.fileno())
    os.chmod(pointer.name, 0o644)
    os.replace(pointer.name, versions / (version + '.txt'))
    current = root / 'current'
    previous = os.readlink(current) if current.is_symlink() else None
    with tempfile.TemporaryDirectory(prefix='.promote-', dir=root) as temporary:
        link = Path(temporary) / 'current'
        link.symlink_to(build)
        os.replace(link, current)
    print(json.dumps({'current': build, 'previous': previous, 'files': len(names)}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', type=Path, required=True)
    parser.add_argument('--root', type=Path, default=Path('/srv/i1-releases/smart-tree'))
    args = parser.parse_args()
    publish(args.release.resolve(), args.root.resolve())
