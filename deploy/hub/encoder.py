"""Local CPU encoder. MEM8 storage and retrieval are owned by st-hub."""
import hashlib
import json
import os
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from fastembed import TextEmbedding

CACHE = Path(os.environ.get("ST_ENCODER_CACHE", "/models"))
NAME = "BAAI/bge-small-en-v1.5"
model = TextEmbedding(model_name=NAME, cache_dir=str(CACHE), threads=6)
digest = hashlib.sha256()
artifacts = sorted(CACHE.rglob("*.onnx")) + sorted(CACHE.rglob("tokenizer.json"))
if not artifacts:
    raise RuntimeError("No cached model artifacts found for identity verification")
for artifact in artifacts:
    digest.update(artifact.name.encode())
    with artifact.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
MODEL_ID = f"{NAME}@{digest.hexdigest()[:24]}/fastembed-0.8.0"
slots = threading.BoundedSemaphore(2)


class Encoder(BaseHTTPRequestHandler):
    def send_json(self, status, value):
        payload = json.dumps(value, allow_nan=False).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_GET(self):
        if self.path != "/health":
            return self.send_json(404, {"error": "Not found"})
        self.send_json(200, {"status": "ok", "model": MODEL_ID, "dimensions": 384})

    def do_POST(self):
        if self.path != "/embed":
            return self.send_json(404, {"error": "Not found"})
        if not slots.acquire(blocking=False):
            return self.send_json(503, {"error": "Encoder busy"})
        try:
            size = int(self.headers.get("Content-Length", "0"))
            if not 0 < size <= 128 * 1024:
                return self.send_json(413, {"error": "Invalid request size"})
            self.connection.settimeout(20)
            request = json.loads(self.rfile.read(size))
            texts = request.get("texts")
            if not isinstance(texts, list) or not 1 <= len(texts) <= 24:
                return self.send_json(400, {"error": "Expected 1 to 24 texts"})
            if any(not isinstance(text, str) or len(text.encode()) > 8192 for text in texts):
                return self.send_json(400, {"error": "Text exceeds limit"})
            if request.get("query") is True:
                vectors = list(model.query_embed(texts))
            else:
                vectors = list(model.passage_embed(texts, batch_size=24))
            self.send_json(200, {"model": MODEL_ID, "vectors": [vector.tolist() for vector in vectors]})
        except (ValueError, TypeError, KeyError):
            self.send_json(400, {"error": "Invalid embedding request"})
        except Exception as error:
            print(f"Encoding failed: {type(error).__name__}", flush=True)
            self.send_json(500, {"error": "Encoding failed"})
        finally:
            slots.release()

    def log_message(self, *_args):
        pass  # Do not retain query text or request headers in logs.


print(f"Encoder ready: {MODEL_ID}", flush=True)
ThreadingHTTPServer(("0.0.0.0", 8429), Encoder).serve_forever()
