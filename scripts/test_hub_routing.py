#!/usr/bin/env python3
"""Check deployed release routing and public/private endpoint boundaries."""
import argparse
import json
import urllib.error
import urllib.request


def get(url):
    with urllib.request.urlopen(url, timeout=15) as response:
        return response.read(), response.headers


def expect_status(url, status):
    try:
        get(url)
    except urllib.error.HTTPError as error:
        assert error.code == status, (url, error.code)
    else:
        raise AssertionError(f'{url} should return {status}')


def check(hub, installer, aliases, existing):
    data, headers = get(installer + '/releases/smart-tree/latest.json')
    release = json.loads(data)
    assert headers['Cache-Control'] == 'no-store'
    assert release['cargo_features'] == ['full'] and release['assets']
    build, headers = get(installer + '/releases/smart-tree/latest.txt')
    assert build.decode().strip() == release['build_id'] and headers['Cache-Control'] == 'no-store'
    _, headers = get(installer + '/releases/smart-tree/' + release['build_id'] + '/version.txt')
    assert 'immutable' in headers['Cache-Control']
    for base in [hub, *aliases]:
        # This must run before any query to the internal certificate allow route.
        expect_status(base + '/internal/tls-allow', 404)
        expect_status(base + '/api/v1/admin/feedback', 401)
        data, headers = get(base + '/api/smart-tree/latest')
        assert json.loads(data)['build_id'] == release['build_id']
        assert headers['Cache-Control'] == 'no-store'
    expect_status(installer + '/api/certs', 404)
    expect_status(installer + '/releases/smart-tree/missing/version.txt', 404)
    for url in [hub + '/health', installer + '/health', *existing]:
        get(url)
    print('PASS current release metadata, cache policy, protected routes, aliases, and existing sites')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--hub', default='https://8s.is')
    parser.add_argument('--installer', default='https://i1.is')
    parser.add_argument('--alias', action='append', default=[])
    parser.add_argument('--existing-site', action='append', default=[])
    args = parser.parse_args()
    check(args.hub.rstrip('/'), args.installer.rstrip('/'), args.alias, args.existing_site)
