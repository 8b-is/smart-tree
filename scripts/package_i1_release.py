#!/usr/bin/env python3
"""Create immutable i1 source snapshots and matching, checksummed binary bundles."""
import argparse
import datetime as dt
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tomllib
import zipfile

BINARIES = ("st", "m8", "n8x")
ORIGIN = "https://i1.is/releases/smart-tree"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def archive(path, files):
    with path.open("wb") as output, gzip.GzipFile(filename="", fileobj=output, mode="wb", mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode="w") as bundle:
            for name, data, executable in sorted(files):
                entry = tarfile.TarInfo(name)
                entry.size = len(data)
                entry.mode = 0o755 if executable else 0o644
                entry.mtime = 0
                bundle.addfile(entry, io.BytesIO(data))
    path.with_name(path.name + ".sha256").write_text(f"{digest(path)}  {path.name}\n")


def archive_zip(path, files):
    with zipfile.ZipFile(path, mode="w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as bundle:
        for name, data, executable in sorted(files):
            info = zipfile.ZipInfo(filename=name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = (0o755 if executable else 0o644) << 16
            bundle.writestr(info, data)
    path.with_name(path.name + ".sha256").write_text(f"{digest(path)}  {path.name}\n")


def asset(path, build):
    return {"name": path.name, "browser_download_url": f"{ORIGIN}/{build}/{path.name}",
            "digest": "sha256:" + digest(path), "size": path.stat().st_size}


def snapshot(args):
    root = Path(__file__).resolve().parents[1]
    candidates = subprocess.check_output(["git", "ls-files", "-co", "--exclude-standard", "-z"], cwd=root).decode().split("\0")
    selected = {"Cargo.toml", "Cargo.lock", "README.md", "LICENSE", "ultra-agent.md"}
    for name in candidates:
        if name.startswith(("src/", "benches/", "systemd/", "skills/", "crates/wst_brain_scan/", "st-protocol/src/", "expert_prompt_engineer/src/")) or name in {
            "build.rs", "st-protocol/Cargo.toml", "expert_prompt_engineer/Cargo.toml",
        }:
            selected.add(name)
    contents = []
    fingerprint = hashlib.sha256()
    for name in sorted(selected):
        path = root / name
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"Expected a regular source file: {name}")
        data = path.read_bytes()
        if b"-----BEGIN PRIVATE KEY-----" in data or b"-----BEGIN OPENSSH PRIVATE KEY-----" in data:
            raise ValueError(f"Private key material in source selection: {name}")
        fingerprint.update(name.encode() + b"\0" + hashlib.sha256(data).digest())
        contents.append(("smart-tree/" + name, data, bool(path.stat().st_mode & 0o111)))
    version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
    build = version + "-" + fingerprint.hexdigest()[:12]
    destination = Path(args.output).resolve() / build
    destination.mkdir(parents=True, exist_ok=False)
    manifest = {
        "version": version, "tag_name": "v" + version, "build_id": build,
        "published_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "release_date": dt.datetime.now(dt.timezone.utc).isoformat(),
        "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "source_fingerprint": fingerprint.hexdigest(), "source_is_development_snapshot": True,
        "html_url": "https://i1.is/#smart-tree", "download_url": "https://i1.is/#smart-tree",
        "release_notes_url": "https://i1.is/releases/smart-tree/" + build + "/manifest.json",
        "features": ["Full build: local LLM inference, TUI, daemon, MEM8, Google Drive/Gmail sync, Marine VAD voice synthesis, and certificate scanning"],
        "cargo_features": ["full"], "ai_benefits": ["Persistent memory and token indexes", "Feedback through 8s.is", "Voice interaction and Google Drive synchronization"],
        "unavailable_features": [],
        "binaries": list(BINARIES), "assets": [],
    }
    contents.append(("smart-tree/BUILD_INFO.json", json.dumps(manifest, indent=2).encode(), False))
    contents.append(("smart-tree/BUILD_ID", (build + "\n").encode(), False))
    source = destination / "source.tar.gz"
    archive(source, contents)
    manifest["source"] = asset(source, build)
    (destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (destination / "build-id.txt").write_text(build + "\n")
    (destination / "version.txt").write_text(version + "\n")
    (destination / "BUILD_FEATURES.txt").write_text("full\n")
    with tarfile.open(source) as bundle:
        bundle.extractall(destination / "source", filter="data")
    print(destination)


def binaries(args):
    destination = Path(args.release).resolve()
    manifest = json.loads((destination / "manifest.json").read_text())
    source = Path(args.binaries).resolve()
    is_windows = "windows" in args.target
    st_bin = source / ("st.exe" if is_windows and (source / "st.exe").exists() else "st")
    environment = dict(os.environ, SMART_TREE_NO_UPDATE_CHECK="1")
    if is_windows:
        st_bytes = st_bin.read_bytes()
        if manifest["build_id"].encode() not in st_bytes or manifest["version"].encode() not in st_bytes:
            raise ValueError("The cross-platform binary does not identify this source snapshot")
    else:
        try:
            version = subprocess.check_output([str(st_bin), "--version"], env=environment, text=True)
            if manifest["build_id"] not in version or f"v{manifest['version']}" not in version:
                raise ValueError("The binary does not identify this source snapshot")
        except OSError:
            st_bytes = st_bin.read_bytes()
            if manifest["build_id"].encode() not in st_bytes or manifest["version"].encode() not in st_bytes:
                raise ValueError("The cross-platform binary does not identify this source snapshot")
    if is_windows:
        files = [
            (
                f"{name}.exe",
                (source / (f"{name}.exe" if (source / f"{name}.exe").exists() else name)).read_bytes(),
                True,
            )
            for name in BINARIES
        ]
        files.append(("BUILD_INFO.json", json.dumps(manifest, indent=2).encode(), False))
        path = destination / f"st-{args.target}.zip"
        if path.exists() and not getattr(args, "force", False):
            raise ValueError("Refusing to overwrite an immutable binary bundle")
        archive_zip(path, files)
    else:
        files = [(name, (source / name).read_bytes(), True) for name in BINARIES]
        files.append(("BUILD_INFO.json", json.dumps(manifest, indent=2).encode(), False))
        path = destination / f"st-{args.target}.tar.gz"
        if path.exists() and not getattr(args, "force", False):
            raise ValueError("Refusing to overwrite an immutable binary bundle")
        archive(path, files)
    new_asset = asset(path, manifest["build_id"])
    manifest["assets"] = [a for a in manifest["assets"] if a.get("name") != path.name]
    manifest["assets"].append(new_asset)
    (destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(path)


parser = argparse.ArgumentParser(description=__doc__)
commands = parser.add_subparsers(dest="command", required=True)
create = commands.add_parser("snapshot")
create.add_argument("--output", required=True)
create.set_defaults(run=snapshot)
add = commands.add_parser("binaries")
add.add_argument("--release", required=True)
add.add_argument("--binaries", required=True)
add.add_argument(
    "--target",
    required=True,
    choices=[
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
    ],
)
add.add_argument("--force", action="store_true", help="Replace existing target archive if present")
add.set_defaults(run=binaries)
arguments = parser.parse_args()
arguments.run(arguments)
