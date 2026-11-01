#!/usr/bin/env python3
"""Exercise the real hub with isolated Git, T8R, MEM8, HTTP, and an encoder fixture."""
import argparse
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def port():
    with socket.socket() as stream:
        stream.bind(("127.0.0.1", 0))
        return stream.getsockname()[1]


def eventually(check, timeout=45):
    end = time.monotonic() + timeout
    last = None
    while time.monotonic() < end:
        try:
            result = check()
            if result:
                return result
        except (OSError, AssertionError) as error:
            last = error
        time.sleep(0.15)
    raise AssertionError(f"Timed out: {last}")


class Encoder(BaseHTTPRequestHandler):
    documents = 0

    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        if not request["query"]:
            type(self).documents += len(request["texts"])
        vectors = []
        for text in request["texts"]:
            vector = [0.0] * 384
            vector[0 if any(word in text.lower() for word in ("cosmos", "stellar", "constellations")) else 1] = 1.0
            vectors.append(vector)
        payload = json.dumps({"model": "fixture-encoder-v1", "vectors": vectors}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, *_args):
        pass


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", default="target/debug/st-hub")
    parser.add_argument("--browser", action="store_true")
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    with tempfile.TemporaryDirectory(prefix="smart-tree-hub-") as temporary:
        root = Path(temporary).resolve()
        collection = root / "collection"
        repo = collection / "atlas"
        repo.mkdir(parents=True)
        def git(*arguments):
            return subprocess.check_output(["git", "-c", "core.hooksPath=/dev/null", "-C", str(repo), *arguments], stderr=subprocess.DEVNULL, text=True).strip()
        git("init", "-q")
        git("config", "user.name", "Hub Test")
        git("config", "user.email", "hub-test@example.invalid")
        git("remote", "add", "origin", "https://github.com/standardgalactic/atlas.git")
        (repo / "README.md").write_text("# Celestial atlas\nStellar cartography maps distant constellations.\nolduniquemarker\n")
        (repo / ".env").write_text("mustneverappear secretcontent\n")
        (repo / "image.bin").write_bytes(b"\x00privatebinary")
        git("add", ".")
        git("commit", "-qm", "initial")
        commit = git("rev-parse", "HEAD")
        (repo / "README.md").write_text("uncommittedprivatecontent\n")
        token = secrets.token_hex(32)
        token_file = root / "admin-token"
        token_file.write_text(token)
        token_file.chmod(0o600)
        encoder = ThreadingHTTPServer(("127.0.0.1", 0), Encoder)
        threading.Thread(target=encoder.serve_forever, daemon=True).start()
        bind = port()
        base = f"http://127.0.0.1:{bind}"
        command = [str(binary), "--bind", f"127.0.0.1:{bind}", "--state-dir", str(root / "state"), "--archive-dir", str(root / "archives"), "--import-roots", str(collection), "--admin-token-file", str(token_file), "--embed-url", f"http://127.0.0.1:{encoder.server_port}/embed"]
        log_path = Path("/tmp/smart-tree-hub-smoke.log")
        log = log_path.open("w")
        process = None
        def api(path, data=None, credential=None, method=None, expected=200):
            headers = {}
            if credential:
                headers["Authorization"] = f"Bearer {credential}"
            body = None
            if data is not None:
                body = json.dumps(data).encode()
                headers["Content-Type"] = "application/json"
            request = Request(base + path, data=body, headers=headers, method=method)
            try:
                response = urlopen(request, timeout=20)
            except HTTPError as error:
                response = error
            payload = response.read()
            assert response.status == expected, (path, response.status, payload[:500])
            return json.loads(payload)
        def start():
            nonlocal process
            process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
            eventually(lambda: api("/health")["status"] == "ok")
        def status(path, expected):
            try:
                response = urlopen(base + path, timeout=20)
            except HTTPError as error:
                response = error
            with response:
                assert response.status == expected, (path, response.status)
        def stop():
            nonlocal process
            process.terminate()
            process.wait(timeout=30)
            process = None
        try:
            start()
            for domain in ["8s.is", "api.8s.is", "FEEDBACK.8S.IS"]:
                status(f"/internal/tls-allow?domain={domain}", 200)
            for domain in ["example.com", "evil8s.is", "a.b.8s.is", "-bad.8s.is", "bad-.8s.is", "8s.is.evil.com"]:
                status(f"/internal/tls-allow?domain={domain}", 403)
            for number in range(35):
                status(f"/internal/tls-allow?domain=fixture-{number}.8s.is", 200)
            status("/internal/tls-allow?domain=exhausted.8s.is", 429)
            api("/api/v1/admin/import", {"root": str(collection)}, expected=401)
            api("/api/v1/archive-requests", {"source_url": "http://127.0.0.1/admin"}, expected=400)
            api("/api/feedback", {"title": "Invalid", "description": "Score validation", "impact_score": 100}, expected=400)
            imported = api("/api/v1/admin/import", {"root": str(collection)}, token)
            assert imported["imported"] == 1
            listing = api("/api/v1/repositories", credential=token)["repositories"]
            archive_id = listing[0]["id"]
            endpoint = f"/api/v1/repositories/{archive_id}"
            eventually(lambda: api(endpoint, credential=token)["status"] == "archived")
            assert api("/api/v1/repositories")["repositories"] == []
            api(endpoint, expected=404)
            assert api("/api/v1/recall", {"query": "stellar"})["results"] == []
            api(endpoint, {"public": False, "recall_opt_in": True}, token, "PATCH")
            eventually(lambda: api(endpoint, credential=token)["status"] == "ready")
            assert api("/api/v1/recall", {"query": "cosmos"})["results"] == []
            result = api("/api/v1/recall", {"query": "cosmos"}, token)
            assert result["mode"] == "hybrid" and result["results"]
            assert result["results"][0]["commit"] == commit
            assert "Stellar cartography" in result["results"][0]["text"]
            assert result["results"][0]["line_start"] == 1
            status(f"/git/{archive_id}.git/info/refs?service=git-upload-pack", 401)
            private_refs = subprocess.check_output(["git", "-c", f"http.extraHeader=Authorization: Bearer {token}", "ls-remote", f"{base}/git/{archive_id}.git"], text=True, timeout=30)
            assert commit in private_refs
            for query in ["mustneverappear", "uncommittedprivatecontent"]:
                assert api("/api/v1/recall", {"query": query, "mode": "keyword"}, token)["results"] == []
            print("PASS consent, private retrieval, committed-source evidence, and excluded credentials", flush=True)

            api(endpoint, {"public": True, "recall_opt_in": True}, token, "PATCH")
            assert api("/api/v1/recall", {"query": "cosmos"})["results"]
            clone = root / "clone"
            subprocess.run(["git", "-c", "core.hooksPath=/dev/null", "clone", "-q", f"{base}/git/{archive_id}.git", str(clone)], check=True, timeout=30)
            assert subprocess.check_output(["git", "-C", str(clone), "rev-parse", "HEAD"], text=True).strip() == commit
            request = api("/api/v1/archive-requests", {"source_url": "https://github.com/example/intake-test"})
            request_id, receipt_token = request["repository"]["id"], request["manage_token"]
            assert request["repository"]["public"] is False and request["repository"]["recall_opt_in"] is False
            api(f"/api/v1/repositories/{request_id}", expected=404)
            api(f"/api/v1/repositories/{request_id}", {"public": True, "recall_opt_in": True}, "wrong-token", "PATCH", expected=401)
            assert api(f"/api/v1/repositories/{request_id}", credential=receipt_token)["status"] == "pending_approval"
            api(f"/api/v1/admin/repositories/{request_id}/approve", {}, receipt_token, expected=401)
            feedback = api("/feedback", {"category": "bug", "title": "Smoke test", "description": "Private feedback persistence"})
            api("/api/v1/admin/feedback", expected=401)
            assert api("/api/v1/admin/feedback", credential=token)["feedback"][0]["id"] == feedback["feedback_id"]
            print("PASS Git clone, owner receipts, archive review queue, and private feedback", flush=True)

            if args.browser:
                subprocess.run(["node", "scripts/test_hub_browser.cjs", base, "/tmp/smart-tree-hub-preview"], check=True)
            docs_before = Encoder.documents
            stop()
            assert (root / "state/passages.t8").read_bytes().startswith(b"T8R\x00\x01")
            assert (root / "state/recall.m8").read_bytes()[8:12] == (0x4D454D38).to_bytes(4, "little")
            assert (root / "state/recall.m8").stat().st_mode & 0o777 == 0o600
            start()
            status("/internal/tls-allow?domain=fixture-0.8s.is", 200)
            status("/internal/tls-allow?domain=exhausted.8s.is", 429)
            assert api("/api/v1/recall", {"query": "cosmos"})["results"]
            api(f"/api/v1/admin/repositories/{archive_id}/reindex", {}, token)
            eventually(lambda: api(endpoint)["status"] == "ready")
            assert Encoder.documents == docs_before
            assert any(item["id"] == feedback["feedback_id"] for item in api("/api/v1/admin/feedback", credential=token)["feedback"])
            print("PASS persisted TLS domain boundaries and certificate issuance budget", flush=True)
            print("PASS native MEM8/T8R storage, restart recovery, and unchanged-commit reuse", flush=True)

            (repo / "README.md").write_text("# Updated atlas\nStellar cartography with newuniquemarker.\n")
            git("add", "README.md")
            git("commit", "-qm", "update")
            new_commit = git("rev-parse", "HEAD")
            api(f"/api/v1/admin/repositories/{archive_id}/reindex", {}, token)
            eventually(lambda: api(endpoint)["commit"] == new_commit)
            assert api("/api/v1/recall", {"query": "olduniquemarker", "mode": "keyword"})["results"] == []
            assert api("/api/v1/recall", {"query": "newuniquemarker", "mode": "keyword"})["results"][0]["commit"] == new_commit
            api(endpoint, {"public": True, "recall_opt_in": False}, token, "PATCH")
            assert api("/api/v1/recall", {"query": "cosmos"})["results"] == []
            stop()
            start()
            assert api("/api/v1/recall", {"query": "newuniquemarker", "mode": "keyword"})["results"] == []
            assert api(endpoint)["recall_opt_in"] is False
            print("PASS atomic index generation replacement and durable consent revocation", flush=True)
        finally:
            if process is not None:
                process.terminate()
                try:
                    process.wait(timeout=30)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
            encoder.shutdown()
            log.close()
            print(f"Hub log: {log_path}", flush=True)


if __name__ == "__main__":
    main()
