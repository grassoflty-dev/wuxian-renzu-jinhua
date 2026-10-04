#!/usr/bin/env python3
"""Negative and positive contract tests for the B0 release gate."""
from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import asset_release_gate as gate


class AssetReleaseGateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = gate.load_manifest()
        cls.approved = next(row for row in cls.manifest["assets"] if row["release_status"] == "release_approved")
        cls.source_blob = gate.read_approved_source(cls.approved["relative_path"], gate.REPO)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="asset-gov-gate-")
        self.root = Path(self.temp.name)
        self.source_file = self.root / "approved-source.bin"
        self.source_file.write_bytes(self.source_blob)

    def tearDown(self):
        self.temp.cleanup()

    def test_approved_source_and_exact_sha_pass(self):
        result = gate.verify_asset(self.manifest, self.approved["asset_id"], self.source_file,
                                   self.approved["relative_path"])
        self.assertEqual(result["sha256"], self.approved["sha256"])
        self.assertEqual(result["admission"], "release_approved")

    def test_concept_only_hard_fails(self):
        row = next(row for row in self.manifest["assets"] if row["release_status"] == "concept_only")
        with self.assertRaisesRegex(gate.GateError, "E_ASSET_NOT_APPROVED"):
            gate.verify_asset(self.manifest, row["asset_id"], self.source_file, row["relative_path"])

    def test_reject_hard_fails(self):
        row = next(row for row in self.manifest["assets"] if row["release_status"] == "reject")
        with self.assertRaisesRegex(gate.GateError, "E_ASSET_NOT_APPROVED"):
            gate.verify_asset(self.manifest, row["asset_id"], self.source_file, row["relative_path"])

    def test_wrong_sha_hard_fails(self):
        self.source_file.write_bytes(self.source_blob + b"tamper")
        with self.assertRaisesRegex(gate.GateError, "E_SOURCE_SHA256_MISMATCH"):
            gate.verify_asset(self.manifest, self.approved["asset_id"], self.source_file,
                              self.approved["relative_path"])

    def test_tampered_manifest_sha_cannot_override_pinned_git_sha(self):
        tampered = copy.deepcopy(self.manifest)
        tampered_row = next(row for row in tampered["assets"] if row["asset_id"] == self.approved["asset_id"])
        tampered_row["sha256"] = "0" * 64
        with self.assertRaisesRegex(gate.GateError, "E_PINNED_SOURCE_SHA256_MISMATCH"):
            gate.verify_asset(tampered, self.approved["asset_id"], self.source_file,
                              self.approved["relative_path"])

    def test_missing_source_hard_fails(self):
        with self.assertRaisesRegex(gate.GateError, "E_SOURCE_MISSING"):
            gate.verify_asset(self.manifest, self.approved["asset_id"], self.root / "missing.png",
                              self.approved["relative_path"])

    def test_wrong_source_path_hard_fails(self):
        with self.assertRaisesRegex(gate.GateError, "E_SOURCE_PATH_MISMATCH"):
            gate.verify_asset(self.manifest, self.approved["asset_id"], self.source_file, "batch01/other.png")

    def test_cli_approved_source_exits_successfully(self):
        script = gate.REPO / "tools/asset-release-gate/asset_release_gate.py"
        result = subprocess.run(
            [sys.executable, str(script), "--manifest", str(gate.DEFAULT_MANIFEST),
             "--asset-id", self.approved["asset_id"], "--source-path", self.approved["relative_path"],
             "--file", str(self.source_file)],
            cwd=gate.REPO, capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('"result": "pass"', result.stdout)

    def test_cli_missing_source_exits_hard_failure(self):
        script = gate.REPO / "tools/asset-release-gate/asset_release_gate.py"
        result = subprocess.run(
            [sys.executable, str(script), "--manifest", str(gate.DEFAULT_MANIFEST),
             "--asset-id", self.approved["asset_id"], "--source-path", self.approved["relative_path"],
             "--file", str(self.root / "missing.png")],
            cwd=gate.REPO, capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("E_SOURCE_MISSING", result.stderr)

    def test_unknown_asset_hard_fails(self):
        with self.assertRaisesRegex(gate.GateError, "E_ASSET_UNKNOWN"):
            gate.verify_asset(self.manifest, "unknown.asset", self.source_file, self.approved["relative_path"])

    def test_pending_review_status_hard_fails(self):
        malformed = copy.deepcopy(self.manifest)
        malformed["assets"][0]["release_status"] = "pending_review"
        with self.assertRaisesRegex(gate.GateError, "E_UNKNOWN_RELEASE_STATUS"):
            gate.validate_manifest(malformed)

    def test_legacy_pending_review_manifest_hard_fails(self):
        with self.assertRaisesRegex(gate.GateError, "E_MANIFEST_SCHEMA"):
            gate.validate_manifest({"assets": [{"asset_id": "legacy", "status": "pending_review"}]})

    def test_missing_manifest_hard_fails(self):
        with self.assertRaisesRegex(gate.GateError, "E_MANIFEST_MISSING"):
            gate.load_manifest(self.root / "not-present.json")

    def test_duplicate_source_id_hard_fails(self):
        malformed = copy.deepcopy(self.manifest)
        malformed["assets"][1]["asset_id"] = malformed["assets"][0]["asset_id"]
        with self.assertRaisesRegex(gate.GateError, "E_DUPLICATE_OR_INVALID_ASSET_ID"):
            gate.validate_manifest(malformed)

    def test_path_escape_hard_fails(self):
        malformed = copy.deepcopy(self.manifest)
        malformed["assets"][0]["relative_path"] = "../outside.png"
        with self.assertRaisesRegex(gate.GateError, "E_UNSAFE_OR_DUPLICATE_SOURCE_PATH"):
            gate.validate_manifest(malformed)


if __name__ == "__main__":
    unittest.main(verbosity=2)
