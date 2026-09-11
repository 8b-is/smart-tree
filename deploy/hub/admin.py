#!/usr/bin/env python3
"""Call the local hub without putting its administrator token in shell history."""
import argparse
import json
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import HTTPRedirectHandler, Request, build_opener


class NoRedirects(HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", help="Local /api/v1/... endpoint")
    parser.add_argument("--json", dest="body", help="JSON body; selects POST unless --method is set")
    parser.add_argument("--method", choices=["GET", "POST", "PATCH"])
    args = parser.parse_args()
    if not args.path.startswith("/api/v1/") or any(ord(char) < 32 for char in args.path):
        parser.error("Use a local /api/v1/... path")
    token = Path("/etc/smart-tree-hub/admin-token").read_text().strip()
    headers = {"Authorization": f"Bearer {token}"}
    data = None
    if args.body is not None:
        data = json.dumps(json.loads(args.body)).encode()
        headers["Content-Type"] = "application/json"
    request = Request("http://127.0.0.1:8428" + args.path, data=data, headers=headers, method=args.method)
    try:
        with build_opener(NoRedirects()).open(request, timeout=120) as response:
            print(json.dumps(json.load(response), indent=2))
    except HTTPError as error:
        print(f"HTTP {error.code}: {error.read().decode(errors='replace')}")
        raise SystemExit(1) from None


if __name__ == "__main__":
    main()
