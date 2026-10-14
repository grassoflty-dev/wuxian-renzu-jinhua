#!/usr/bin/env python3
"""Read-only Windows resource-byte preflight; no build, launch or admission.
Requires an operator-controlled checkout with no concurrent filesystem writers.
This is not protection against a hostile machine or concurrent parent replacement.
All output is JSON on stdout. No files are created, copied, changed or deleted.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys

MAP_BYTES = 11373
MAP_SHA256 = '5190db6d874183ae965ba639b04412cda2df28761eefab21435aa2503755811b'
BASE_COMMIT = '0d96883c6a0b661b711d5b94bcfdd3acadbf0ddb'
BASE_TREE = '64430cd9d27e2942a4d9cccd4d627dc45043e539'
REPARSE = 0x400
MAX_RESOURCE_BYTES = 64 * 1024 * 1024
RESERVED = {'CON', 'PRN', 'AUX', 'NUL', 'CONIN$', 'CONOUT$'} | {
    prefix + suffix for prefix in ('COM', 'LPT') for suffix in '123456789¹²³'
}

class Fault(Exception):
    """Only controlled non-sensitive codes are carried in this exception."""


def component(value: str) -> None:
    if (not value or value in ('.', '..') or value.endswith((' ', '.'))
            or any(ord(c) < 32 or ord(c) == 127 or c in '<>:"/\\|?*' for c in value)
            or value.split('.', 1)[0].upper() in RESERVED
            or value.casefold() in ('.git', '.svn', '.hg')):
        raise Fault('INVALID_PATH_COMPONENT')


def absolute(raw: str) -> Path:
    # Validate original spelling BEFORE Path can collapse lexical components.
    if not isinstance(raw, str) or not re.match(r'^[A-Za-z]:[\\/]', raw):
        raise Fault('ABSOLUTE_DRIVE_PATH_REQUIRED')
    parts = raw[3:].replace('\\', '/').split('/')
    if not parts or not all(parts):
        raise Fault('EMPTY_ABSOLUTE_COMPONENT')
    for part in parts:
        component(part)
    return Path(raw)


def relative(raw: object, prefix: str) -> str:
    if not isinstance(raw, str) or '\\' in raw:
        raise Fault('INVALID_RELATIVE_PATH')
    parts = raw.split('/')
    for part in parts:
        component(part)
    if not raw.startswith(prefix + '/') or not raw.endswith('.png'):
        raise Fault('PATH_OUTSIDE_ALLOWLIST_PREFIX')
    return raw


def identity_cross_source(info: os.stat_result) -> tuple:
    # Windows lstat/fstat can expose different ctime values for unchanged bytes.
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns)


def identity_same_source(info: os.stat_result) -> tuple:
    # Preserve ctime change detection within path or descriptor observations.
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def ordinary(info: os.stat_result, directory: bool) -> None:
    if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & REPARSE:
        raise Fault('LINK_OR_REPARSE_REJECTED')
    if directory:
        if not stat.S_ISDIR(info.st_mode):
            raise Fault('ANCESTOR_NOT_DIRECTORY')
    elif not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
        raise Fault('NONORDINARY_OR_HARDLINK_FILE')


def inspect(path: Path, directory: bool = False) -> os.stat_result:
    """Only traverse requested ancestors; never recursively scan unrelated trees."""
    current = Path(path.anchor)
    ordinary(os.lstat(current), True)
    parts = path.parts[1:]
    for index, part in enumerate(parts):
        # Enumeration does not follow entries; reject case aliasing of this name.
        with os.scandir(current) as entries:
            matches = [entry.name for entry in entries if entry.name.casefold() == part.casefold()]
        if not matches:
            raise Fault('MISSING')
        if len(matches) != 1 or matches[0] != part:
            raise Fault('CASE_COLLISION_OR_SPELLING')
        current = current / part
        info = os.lstat(current)
        ordinary(info, index < len(parts) - 1 or directory)
    return info


def fingerprint(path: Path, expected_size: int | None, keep: bool = False) -> dict:
    before = inspect(path)
    if before.st_size <= 0 or before.st_size > MAX_RESOURCE_BYTES:
        raise Fault('FILE_SIZE_OUT_OF_BOUND')
    if expected_size is not None and before.st_size != expected_size:
        raise Fault('SIZE_MISMATCH')
    digest = hashlib.sha256()
    git = hashlib.sha1(b'blob ' + str(before.st_size).encode('ascii') + b'\0')
    chunks = []
    count = 0
    with path.open('rb') as handle:
        opened = os.fstat(handle.fileno())
        ordinary(opened, False)
        if identity_cross_source(opened) != identity_cross_source(before):
            raise Fault('FILE_CHANGED_BEFORE_READ')
        while True:
            chunk = handle.read(1024 * 1024)
            if not chunk:
                break
            count += len(chunk)
            if count > before.st_size:
                raise Fault('FILE_CHANGED_DURING_READ')
            digest.update(chunk)
            git.update(chunk)
            if keep:
                chunks.append(chunk)
        after = os.fstat(handle.fileno())
        ordinary(after, False)
        if count != before.st_size or identity_same_source(after) != identity_same_source(opened):
            raise Fault('FILE_CHANGED_DURING_READ')
    if identity_same_source(inspect(path)) != identity_same_source(before):
        raise Fault('FILE_CHANGED_AFTER_READ')
    result = {'bytes': count, 'sha256': digest.hexdigest(), 'gitBlob': git.hexdigest()}
    if keep:
        result['data'] = b''.join(chunks)
    return result


def no_duplicates(pairs: list) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise Fault('DUPLICATE_JSON_KEY')
        result[key] = value
    return result


def reject_constant(value: str) -> None:
    raise Fault('NONFINITE_JSON_NUMBER')


def hex_string(value: object, length: int) -> bool:
    return isinstance(value, str) and re.fullmatch('[0-9a-f]{' + str(length) + '}', value) is not None


def load_manifest(path: Path) -> list:
    evidence = fingerprint(path, MAP_BYTES, keep=True)
    if evidence['sha256'] != MAP_SHA256:
        raise Fault('MANIFEST_HASH_MISMATCH')
    document = json.loads(evidence['data'].decode('utf-8'), object_pairs_hook=no_duplicates,
                          parse_constant=reject_constant)
    keys = {'schemaVersion', 'sourceCommit', 'sourceTree', 'publicSourceCount', 'externalTargetCount', 'targets'}
    if not isinstance(document, dict) or set(document) != keys:
        raise Fault('MANIFEST_SCHEMA')
    for key, value in [('schemaVersion', 1), ('publicSourceCount', 27), ('externalTargetCount', 4)]:
        if type(document[key]) is not int or document[key] != value:
            raise Fault('MANIFEST_COUNTS')
    if document['sourceCommit'] != BASE_COMMIT or document['sourceTree'] != BASE_TREE:
        raise Fault('MANIFEST_BASE_IDENTITY')
    rows = document['targets']
    if not isinstance(rows, list) or len(rows) != 31:
        raise Fault('MANIFEST_ROWS')
    targets, sources = set(), set()
    for row in rows:
        if not isinstance(row, dict) or set(row) != {'target', 'sha256', 'bytes', 'publicSource', 'publicGitBlob'}:
            raise Fault('ROW_SCHEMA')
        target = relative(row['target'], 'apps/godot_playable/assets')
        if target.casefold() in targets or not hex_string(row['sha256'], 64):
            raise Fault('TARGET_DUPLICATE_OR_HASH')
        targets.add(target.casefold())
        if row['publicSource'] is None:
            if row['publicGitBlob'] is not None or row['bytes'] is not None:
                raise Fault('EXTERNAL_METADATA_SCHEMA')
        else:
            source = relative(row['publicSource'], 'assets')
            if source.casefold() in sources or not hex_string(row['publicGitBlob'], 40):
                raise Fault('SOURCE_DUPLICATE_OR_GIT_HASH')
            sources.add(source.casefold())
            if type(row['bytes']) is not int or not 0 < row['bytes'] <= MAX_RESOURCE_BYTES:
                raise Fault('SOURCE_SIZE_SCHEMA')
    if len(sources) != 27 or sum(r['publicSource'] is None for r in rows) != 4:
        raise Fault('ACTUAL_ROW_COUNTS')
    return rows


def examine(root: Path, row: dict, source: bool) -> dict:
    relative_path = row['publicSource'] if source else row['target']
    result = {'kind': 'public_source' if source else 'target', 'path': relative_path}
    try:
        actual = fingerprint(root.joinpath(*relative_path.split('/')), row['bytes'])
        result.update(actual)
        if actual['sha256'] != row['sha256']:
            raise Fault('SHA256_MISMATCH')
        if source and actual['gitBlob'] != row['publicGitBlob']:
            raise Fault('GIT_BLOB_MISMATCH')
        result['state'] = 'MATCH'
    except Fault as error:
        result['state'] = 'MISSING' if str(error) == 'MISSING' else 'ERROR'
        result['reason'] = str(error)
    except OSError:
        result.update(state='ERROR', reason='FILESYSTEM_READ_ERROR')
    return result


def run(args: argparse.Namespace) -> tuple[dict, int]:
    if os.name != 'nt':
        raise Fault('WINDOWS_ONLY')
    if not args.no_concurrent_writers:
        raise Fault('NO_CONCURRENT_WRITERS_ACK_REQUIRED')
    root, mapping = absolute(args.candidate_root), absolute(args.manifest)
    inspect(root, directory=True)
    rows = load_manifest(mapping)
    results = []
    for row in rows:
        if row['publicSource'] is not None:
            results.append(examine(root, row, True))
        result = examine(root, row, False)
        result['hasPublicSource'] = row['publicSource'] is not None
        results.append(result)
    errors = sum(r['state'] == 'ERROR' for r in results)
    missing = sum(r['state'] == 'MISSING' for r in results)
    totals = {
        'publicSourcesExpected': 27, 'targetsExpected': 31,
        'publicSourcesMatched': sum(r['kind'] == 'public_source' and r['state'] == 'MATCH' for r in results),
        'targetsMatched': sum(r['kind'] == 'target' and r['state'] == 'MATCH' for r in results),
        'externalTargetsMissing': sum(r['kind'] == 'target' and not r['hasPublicSource'] and r['state'] == 'MISSING' for r in results),
        'missingChecks': missing, 'errorChecks': errors,
    }
    status, exit_code = ('ERROR', 2) if errors else (('BLOCKED_MISSING_RESOURCES', 3) if missing else ('RESOURCE_BYTES_MATCH_ONLY', 0))
    return {'schema': 'wuxian.resource-preflight/v1', 'status': status,
            'manifestSha256': MAP_SHA256, 'referenceSourceCommit': BASE_COMMIT,
            'checkoutGitIdentityVerified': False, 'runtime': 'NOT_RUN',
            'rightsOrAdmissionGranted': False, 'totals': totals, 'results': results}, exit_code


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate-root', required=True)
    parser.add_argument('--manifest', required=True)
    parser.add_argument('--no-concurrent-writers', action='store_true')
    try:
        args = parser.parse_args()
        report, code = run(args)
    except Fault as error:
        report, code = {'status': 'ERROR', 'reason': str(error), 'runtime': 'NOT_RUN'}, 2
    except (OSError, ValueError, TypeError, KeyError, UnicodeError):
        report, code = {'status': 'ERROR', 'reason': 'INPUT_OR_FILESYSTEM_ERROR', 'runtime': 'NOT_RUN'}, 2
    print(json.dumps(report, ensure_ascii=True, indent=2))
    return code

if __name__ == '__main__':
    sys.exit(main())
