#!/usr/bin/env python3
import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_VERSION_FILE = Path('traits/sys/version/version.trait.toml')
SNAPSHOT_FILES = [
    'index.html',
    'CNAME',
    'robots.txt',
    'sitemap.xml',
    'traits/www/static/index.standalone.html',
    'traits/www/static/sdk-runtime.js',
    'traits/www/static/traits-worker.js',
    'traits/www/static/wasm-runtime.js',
]


def read_version(version_file: Path) -> str:
    for line in version_file.read_text().splitlines():
        stripped = line.strip()
        if not stripped.startswith('version') or '=' not in stripped:
            continue
        value = stripped.split('=', 1)[1].strip().strip('"').strip()
        if value:
            return value
    raise SystemExit(f'No version found in {version_file}')


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def build_manifest(site_root: Path, version: str, git_sha: str) -> dict:
    files = []
    for rel_path in SNAPSHOT_FILES:
        file_path = site_root / rel_path
        if not file_path.exists() or not file_path.is_file():
            continue
        files.append(
            {
                'path': rel_path,
                'size': file_path.stat().st_size,
                'sha256': sha256_file(file_path),
            }
        )

    return {
        'version': version,
        'git_sha': git_sha,
        'generated_at': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z'),
        'snapshot_artifact': f'slob-games-{version}.tar.gz',
        'files': files,
    }


def write_manifest(output_dir: Path, manifest: dict) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    version_path = output_dir / f"{manifest['version']}.json"
    latest_path = output_dir / 'latest.json'
    payload = json.dumps(manifest, indent=2, sort_keys=True) + '\n'
    version_path.write_text(payload)
    latest_path.write_text(payload)
    return version_path


def main() -> None:
    parser = argparse.ArgumentParser(description='Create a release snapshot manifest for a slob.games build.')
    parser.add_argument('--site-root', default='.', help='Repository root used to resolve files')
    parser.add_argument('--output-dir', help='Directory to write snapshot manifests into')
    parser.add_argument('--git-sha', default='unknown', help='Git SHA to record in the manifest')
    parser.add_argument('--version-file', default=str(DEFAULT_VERSION_FILE), help='Path to version.trait.toml')
    parser.add_argument('--print-version', action='store_true', help='Only print the resolved version')
    args = parser.parse_args()

    version = read_version(Path(args.version_file))
    if args.print_version:
        print(version)
        return

    if not args.output_dir:
        raise SystemExit('--output-dir is required unless --print-version is used')

    site_root = Path(args.site_root)
    manifest = build_manifest(site_root, version, args.git_sha)
    version_path = write_manifest(Path(args.output_dir), manifest)
    print(json.dumps({'ok': True, 'version': version, 'manifest': str(version_path)}))


if __name__ == '__main__':
    main()
