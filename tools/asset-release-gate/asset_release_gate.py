#!/usr/bin/env python3
"""Fail-closed admission checks for the canonical ASSET-GOV-V1 manifest."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

from source_provenance import read_approved_source, ProvenanceError

REPO = Path(__file__).resolve().parents[2]
DEFAULT_MANIFEST = REPO / "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"
SCHEMA_ID = "ai-asset-release-manifest/1"
SOURCE_REPORT_SHA256 = "fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1"
SOURCE_COMMIT = "030eb9bf8194ad5534dc77315567ff339cd2c625"
SOURCE_PREFIX = "chatgptimage/wuxian-renzu-jinhua_2d_asset_pack_10_batches_2026-09-25/"
EXPECTED_COUNTS = {"release_approved": 43, "concept_only": 30, "reject": 21}
LEGACY_MANIFESTS = [
    {"path": SOURCE_PREFIX + "manifest.csv", "sha256": "2833d9409c823ffdf731ac0cce2967343fb8059375bd0bed8e3cb72b56df25ba",
     "classification": "ARCHIVE", "runtimeAuthority": False},
    {"path": SOURCE_PREFIX + "manifest.json", "sha256": "100195406103f11d0b558aafa228d3a40b3ecfee5a39e12527375c1b7994f742",
     "classification": "ARCHIVE", "runtimeAuthority": False},
]
STATUSES = frozenset(EXPECTED_COUNTS)
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


class GateError(ValueError):
    """A release check failed; callers must stop admission."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _safe_relative_path(value: Any) -> bool:
    if not isinstance(value, str) or not value or "\\" in value or value.startswith("/"):
        return False
    parts = value.split("/")
    return len(parts) == 2 and re.fullmatch(r"batch(?:0[1-9]|10)", parts[0]) is not None and all(
        part not in {"", ".", ".."} for part in parts
    )


def validate_manifest(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict) or value.get("schemaId") != SCHEMA_ID or value.get("schemaVersion") != 1:
        raise GateError("E_MANIFEST_SCHEMA: unsupported or legacy manifest")
    required = {"schemaId", "schemaVersion", "decisionSource", "sourceTree", "counts", "byBatch", "legacyManifests", "assets"}
    if set(value) != required:
        raise GateError("E_MANIFEST_SCHEMA: unexpected or missing top-level fields")

    decision = value["decisionSource"]
    if not isinstance(decision, dict) or decision != {
        "path": "deep-research-report.md", "sha256": SOURCE_REPORT_SHA256,
    }:
        raise GateError("E_DECISION_SOURCE: authoritative report identity mismatch")
    source_tree = value["sourceTree"]
    if not isinstance(source_tree, dict) or source_tree != {"commit": SOURCE_COMMIT, "prefix": SOURCE_PREFIX}:
        raise GateError("E_SOURCE_TREE: source Git tree mismatch")

    assets = value["assets"]
    if not isinstance(assets, list) or len(assets) != 94:
        raise GateError("E_MANIFEST_ROW_COUNT: expected exactly 94 decisions")
    ids: set[str] = set()
    paths: set[str] = set()
    counts = {status: 0 for status in STATUSES}
    batches: dict[str, dict[str, int]] = {}
    for row in assets:
        if not isinstance(row, dict) or set(row) != {"relative_path", "asset_id", "sha256", "release_status", "suggested_use"}:
            raise GateError("E_MANIFEST_ROW_SCHEMA: malformed decision row")
        asset_id = row["asset_id"]
        relative_path = row["relative_path"]
        sha = row["sha256"]
        status = row["release_status"]
        if not isinstance(asset_id, str) or not asset_id.strip() or asset_id in ids:
            raise GateError("E_DUPLICATE_OR_INVALID_ASSET_ID")
        if not _safe_relative_path(relative_path) or relative_path in paths:
            raise GateError("E_UNSAFE_OR_DUPLICATE_SOURCE_PATH")
        if not isinstance(sha, str) or SHA256_RE.fullmatch(sha) is None:
            raise GateError(f"E_INVALID_SOURCE_SHA256:{asset_id}")
        if not isinstance(status, str) or status not in STATUSES:
            raise GateError(f"E_UNKNOWN_RELEASE_STATUS:{asset_id}:{status}")
        if not isinstance(row["suggested_use"], str):
            raise GateError(f"E_INVALID_SUGGESTED_USE:{asset_id}")
        ids.add(asset_id)
        paths.add(relative_path)
        counts[status] += 1
        batch = relative_path.split("/", 1)[0]
        per_batch = batches.setdefault(batch, {name: 0 for name in STATUSES})
        per_batch[status] += 1

    expected_counts = {"rows": 94, **EXPECTED_COUNTS}
    if value["counts"] != expected_counts or counts != EXPECTED_COUNTS:
        raise GateError("E_MANIFEST_COUNTS: decision counts do not match 94-row authority")
    if value["byBatch"] != batches or len(batches) != 10:
        raise GateError("E_MANIFEST_BATCH_COUNTS")
    if value["legacyManifests"] != LEGACY_MANIFESTS:
        raise GateError("E_LEGACY_MANIFEST_AUTHORITY")
    return value


def load_manifest(path: Path = DEFAULT_MANIFEST) -> dict[str, Any]:
    if not path.is_file():
        raise GateError(f"E_MANIFEST_MISSING:{path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise GateError(f"E_MANIFEST_INVALID:{exc}") from exc
    return validate_manifest(value)


def verify_asset(
    manifest: dict[str, Any], asset_id: str, source_file: Path, source_path: str,
) -> dict[str, str]:
    validated = validate_manifest(manifest)
    row = next((item for item in validated["assets"] if item["asset_id"] == asset_id), None)
    if row is None:
        raise GateError(f"E_ASSET_UNKNOWN:{asset_id}")
    if row["release_status"] != "release_approved":
        raise GateError(f"E_ASSET_NOT_APPROVED:{asset_id}:{row['release_status']}")
    if source_path != row["relative_path"]:
        raise GateError(f"E_SOURCE_PATH_MISMATCH:{asset_id}")
    if not source_file.is_file():
        raise GateError(f"E_SOURCE_MISSING:{source_file}")
    try:
        pinned_blob = read_approved_source(row["relative_path"], REPO)
    except (ProvenanceError, OSError) as exc:
        raise GateError(f"E_PINNED_SOURCE_MISSING:{asset_id}: {exc}") from exc
    pinned_sha = hashlib.sha256(pinned_blob).hexdigest()
    if pinned_sha != row["sha256"]:
        raise GateError(f"E_PINNED_SOURCE_SHA256_MISMATCH:{asset_id}")
    try:
        actual = sha256_file(source_file)
    except OSError as exc:
        raise GateError(f"E_SOURCE_UNREADABLE:{source_file}") from exc
    if actual != row["sha256"]:
        raise GateError(f"E_SOURCE_SHA256_MISMATCH:{asset_id}:expected={row['sha256']}:actual={actual}")
    return {"asset_id": asset_id, "source_path": row["relative_path"], "sha256": actual, "admission": "release_approved"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--asset-id", required=True)
    parser.add_argument("--source-path", required=True, help="exact relative_path from the canonical manifest")
    parser.add_argument("--file", required=True, type=Path, help="file to hash; no image decoding is performed")
    args = parser.parse_args()
    try:
        result = verify_asset(load_manifest(args.manifest), args.asset_id, args.file, args.source_path)
    except GateError as exc:
        print(f"ASSET GATE FAIL CLOSED: {exc}", file=sys.stderr)
        return 2
    print(json.dumps({"result": "pass", **result}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
