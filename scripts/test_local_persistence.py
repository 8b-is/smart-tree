#!/usr/bin/env python3
"""Exercise local Smart Tree binaries using disposable files, ports, and stores."""

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


def eventually(check, timeout=20):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            result = check()
            if result:
                return result
        except (OSError, urllib.error.URLError):
            pass
        time.sleep(0.1)
    raise AssertionError("Timed out waiting for daemon state")


def string_payload(value):
    raw = value.encode()
    prefix = bytes([128 + len(raw)]) if len(raw) <= 126 else b"\xff" + struct.pack("<H", len(raw))
    return prefix + raw


class LocalDaemon:
    def __init__(self, binaries, root):
        self.binaries, self.root = binaries, root
        self.docs = root / "documents"
        self.memory = root / "memory"
        self.runtime = root / "runtime"
        for directory in (self.docs, self.memory, self.runtime):
            directory.mkdir()
        self.config = root / "config.toml"
        self.config.write_text('[security.database]\npath = ' + json.dumps(str(root / "security.db")) + '\n')
        self.token = "local-disposable-test-token"
        token_path = root / "daemon.token"
        token_path.write_text(self.token)
        token_path.chmod(0o600)
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1", 0))
            self.port = reservation.getsockname()[1]
        self.env = os.environ.copy()
        self.env.update(ST_CONFIG_PATH=str(self.config), ST_MEMORY_DIR=str(self.memory),
                        ST_TOKEN_PATH=str(token_path), ST_DAEMON_PORT=str(self.port),
                        XDG_RUNTIME_DIR=str(self.runtime), RUST_LOG="warn")
        self.process = None
        self.log = None
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))

    def request(self, route, body=None, method=None, authorized=True):
        headers = {"Content-Type": "application/json"}
        if authorized:
            headers["Authorization"] = "Bearer " + self.token
        data = None if body is None else json.dumps(body).encode()
        req = urllib.request.Request(f"http://127.0.0.1:{self.port}{route}", data=data,
                                     headers=headers, method=method)
        with self.opener.open(req, timeout=5) as response:
            return json.load(response) if "application/json" in response.headers.get("Content-Type", "") else response.read().decode()

    def start(self):
        self.log = (self.root / "daemon.log").open("a")
        self.process = subprocess.Popen([str(self.binaries / "std"), "start"], cwd=self.docs,
                                        env=self.env, stdout=self.log, stderr=self.log)
        def ready():
            if self.process.poll() is not None:
                raise AssertionError("Daemon exited: " + (self.root / "daemon.log").read_text())
            return self.request("/health") and (self.runtime / "st.sock").exists()
        eventually(ready)

    def stop(self):
        if self.process is None:
            return
        if self.process.poll() is None:
            self.request("/shutdown", {}, "POST")
            eventually(lambda: "Smart Tree Daemon stopped." in (self.root / "daemon.log").read_text())
            # The Unix socket listener currently runs until the process receives a signal.
            self.process.terminate()
            self.process.wait(timeout=10)
        for name in ("st.pid", "st.sock"):
            (self.runtime / name).unlink(missing_ok=True)
        self.log.close()
        self.process = None
        (self.root / "daemon.log").write_text("")

    def close(self):
        if self.process is not None:
            if self.process.poll() is None:
                self.process.kill()
            self.process.wait(timeout=10)
        if self.log is not None:
            self.log.close()

    def cli(self, *args, success=True):
        result = subprocess.run([str(self.binaries / "st"), "--no-update-check", *args],
                                env=self.env, cwd=self.docs, capture_output=True, text=True,
                                timeout=30)
        assert (result.returncode == 0) == success, result.stderr + result.stdout
        return result.stdout

    def wave(self, verb, payload=b""):
        escaped = payload.replace(b"\x1b", b"\x1b\x1b").replace(b"\x00", b"\x1b\x00")
        with socket.socket(socket.AF_UNIX) as stream:
            stream.settimeout(5)
            stream.connect(str(self.runtime / "st.sock"))
            stream.sendall(bytes([verb]) + escaped + b"\x00")
            raw = bytearray()
            while True:
                byte = stream.recv(1)
                assert byte, "Unexpected end of socket response"
                if byte == b"\x1b":
                    raw.extend(stream.recv(1))
                elif byte == b"\x00":
                    break
                else:
                    raw.extend(byte)
        assert raw[0] == 6, bytes(raw)
        return json.loads(raw[1:]) if len(raw) > 1 else None


def exercise(binaries):
    # /tmp is intentionally excluded by the Linux scanner's default rules.
    with tempfile.TemporaryDirectory(prefix="st-check-", dir=Path.home()) as directory:
        daemon = LocalDaemon(binaries, Path(directory).resolve())
        try:
            document = daemon.docs / "moon-homework.md"
            document.write_text("Lunar geology homework. Original cometnotes.\n")
            risky = daemon.docs / "suspicious.sh"
            risky.write_text("npx example-fixture@latest\n")
            invalid_config = subprocess.run(
                [str(binaries / "st"), "--no-daemon", "--no-update-check",
                 "--cert-scan", str(document)],
                env={**daemon.env, "ST_CONFIG_PATH": ""}, cwd=daemon.docs,
                capture_output=True, text=True, timeout=10,
            )
            assert invalid_config.returncode != 0
            assert "ST_CONFIG_PATH must not be empty" in invalid_config.stderr
            markdown = daemon.cli("--no-daemon", "--mode", "markdown", str(daemon.docs))
            assert "Concerning Files" in markdown and "suspicious.sh" in markdown.split("Concerning Files", 1)[1] and "Dynamic NPX" in markdown
            print("PASS isolated configuration and Markdown concerning-files output", flush=True)

            cert_dir = daemon.root / "certificates"
            cert_dir.mkdir()
            pem, der = cert_dir / "fixture.pem", daemon.root / "fixture.der"
            subprocess.run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
                            "-keyout", str(daemon.root / "fixture.key"), "-out", str(pem),
                            "-days", "2", "-subj", "/CN=smart-tree-local-test/O=Fixture"],
                           check=True, capture_output=True, timeout=20)
            subprocess.run(["openssl", "x509", "-in", str(pem), "-outform", "DER", "-out", str(der)],
                           check=True, capture_output=True, timeout=10)
            embedded = cert_dir / "embedded.bin"
            embedded.write_bytes(b"fixture-prefix" + der.read_bytes() + b"fixture-suffix")
            fingerprint = hashlib.sha256(der.read_bytes()).hexdigest()
            result = json.loads(daemon.cli("--no-daemon", "--cert-scan", str(cert_dir), "--mode", "json"))
            certificates = [c for f in result["files"] for c in f["inspection"]["certificates"]]
            assert len(certificates) == 2 and all(c["fingerprint_sha256"] == fingerprint for c in certificates)
            assert any(c["offset"] == 14 for c in certificates), certificates
            malformed = daemon.root / "malformed.pem"
            malformed.write_text("-----BEGIN CERTIFICATE-----\nbroken\n")
            bad = json.loads(daemon.cli("--no-daemon", "--cert-scan", str(malformed), "--mode", "json", success=False))
            assert bad["files"][0]["inspection"]["issues"]
            print("PASS PEM, embedded DER, fingerprints, and malformed-certificate diagnostics", flush=True)

            daemon.start()
            try:
                daemon.request("/context/recall", {"query": "homework"}, authorized=False)
                raise AssertionError("Unauthenticated recall was accepted")
            except urllib.error.HTTPError as error:
                assert error.code == 401
            now = dt.datetime.now(dt.timezone.utc)
            monday = (now - dt.timedelta(days=now.weekday())).replace(hour=0, minute=0, second=0, microsecond=0)
            last_week = monday - dt.timedelta(days=3)
            association = {"path": str(document), "people": ["daughter"],
                           "notes": "Worked on lunar homework with my 10 year old daughter",
                           "occurred_at": last_week.isoformat()}
            assert daemon.request("/context/remember", association)["stored"]
            recall = daemon.request("/context/recall", {"query": "document with my daughter last week"})
            assert recall[0]["path"] == str(document) and recall[0]["evidence"]
            cli_recall = json.loads(daemon.cli("--recall", "daughter last week", "--daemon-port", str(daemon.port)))
            assert cli_recall[0]["path"] == str(document)
            payload = string_payload("A lunar homework memory") + string_payload("lunar,daughter") + string_payload("technical")
            memory_id = daemon.wave(0x1A, payload)["id"]
            assert daemon.wave(0x1C, string_payload("daughter"))["count"] == 1
            scan = daemon.request("/security/certs/scan", {"path": str(cert_dir)})
            assert len(scan["files"]) == 2
            history_route = "/security/history?" + urllib.parse.urlencode({"path": str(cert_dir), "kind": "certificates"})
            history = daemon.request(history_route)
            document.write_text("Lunar geology homework. Updated astrolabeindex.\n")
            eventually(lambda: daemon.request("/context/recall", {"query": "astrolabeindex"}))
            print("PASS authenticated HTTP/CLI retrieval, native socket memory, and live index updates", flush=True)
            daemon.stop()

            journal = daemon.memory / "file_index.t8"
            original_size = journal.stat().st_size
            assert journal.read_bytes().startswith(b"T8R\x00\x01\x00\x00\x00")
            for name in ("directory_context.m8", "proxy_memory.m8", "wave_memory.native.m8"):
                raw = (daemon.memory / name).read_bytes()
                assert len(raw) % 8192 == 0 and raw[8:12] == struct.pack("<I", 0x4D454D38), name
                assert (daemon.memory / name).stat().st_mode & 0o777 == 0o600, name
            assert (daemon.root / "security.scans.t8").read_bytes().startswith(b"T8R")
            daemon.start()
            assert daemon.request("/context/recall", {"query": "daughter last week"})[0]["path"] == str(document)
            assert daemon.wave(0x1C, string_payload("daughter"))["memories"][0]["id"] == memory_id
            assert daemon.request(history_route) == history
            time.sleep(1.5)
            assert journal.stat().st_size == original_size, "Unchanged files were rewritten after restart"
            daemon.wave(0x1D, memory_id.encode())
            daemon.request("/watch", {"path": str(daemon.docs)}, "DELETE")
            daemon.stop()
            daemon.start()
            assert daemon.wave(0x1C, string_payload("daughter"))["count"] == 0
            assert daemon.request("/context/recall", {"query": "astrolabeindex"}) == []
            daemon.stop()
            print("PASS restart recovery, persisted scan history, unchanged-index reuse, and durable deletion", flush=True)
        except Exception:
            log_path = daemon.root / "daemon.log"
            if log_path.exists():
                print(log_path.read_text())
            raise
        finally:
            daemon.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin-dir", type=Path, default=Path("target/release"))
    arguments = parser.parse_args()
    exercise(arguments.bin_dir.resolve())
