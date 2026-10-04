"""Verify the closed approved-source snapshot without importing Git history."""
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
PROOF_PATH = "governance/assets/APPROVED_SOURCE_TREE_PROOF.json"
PROOF_SHA256 = "1c84e1bb26261ea7d392d3e969aa81d6a7b8a860b22066573533aa0ee06eef50"
MANIFEST_PATH = "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"
MANIFEST_SHA256 = "c644b5953e9265d45796dfb8a6dad1c0fd8abfbffb3eb4ace84987d8238cee2d"
SOURCE_TREE = "d4cdb185b292466c10675c0f4bedcc7dc6e5beb0"
SOURCE_COMMIT = "030eb9bf8194ad5534dc77315567ff339cd2c625"
SOURCE_PREFIX = "chatgptimage/wuxian-renzu-jinhua_2d_asset_pack_10_batches_2026-09-25/"
SOURCE_DIR = "assets/source/approved"
EXPECTED_BYTES = 71233649


class ProvenanceError(ValueError):
    pass


def fail(message):
    raise ProvenanceError("E_SOURCE_PROVENANCE: " + message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def git_hash(kind, data):
    return hashlib.sha1(kind.encode() + b" " + str(len(data)).encode() + b"\0" + data).hexdigest()


def exact_file(root, relative):
    parts = relative.split("/")
    if not parts or any(p in ("", ".", "..") or "\\" in p for p in parts):
        fail("unsafe path")
    path = root
    for part in parts:
        path = path / part
        if path.is_symlink():
            fail("symlink path")
    if not path.is_file():
        fail("missing source or proof: " + relative)
    return path


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail("duplicate JSON key")
        result[key] = value
    return result


def load_proof(root=REPO):
    root = Path(root)
    data = exact_file(root, PROOF_PATH).read_bytes()
    if len(data) != 44708 or sha256(data) != PROOF_SHA256:
        fail("proof bytes changed")
    proof = json.loads(data, object_pairs_hook=unique_object)
    if (proof["sourceCommit"] != SOURCE_COMMIT or proof["sourceTree"] != SOURCE_TREE
            or proof["sourcePrefix"] != SOURCE_PREFIX or len(proof["trees"]) != 9):
        fail("source authority changed")
    manifest_bytes = exact_file(root, MANIFEST_PATH).read_bytes()
    if sha256(manifest_bytes) != MANIFEST_SHA256 or proof["manifestSha256"] != MANIFEST_SHA256:
        fail("manifest bytes changed")
    manifest = json.loads(manifest_bytes, object_pairs_hook=unique_object)
    trees = {}
    for tree in proof["trees"]:
        if tree["path"] in trees:
            fail("duplicate tree")
        entries = tree["entries"]
        names = set()
        raw = bytearray()
        for item in sorted(entries, key=lambda e: (e["path"] + ("/" if e["type"] == "tree" else "")).encode()):
            name = item["path"]
            if name in names or not name or "/" in name or "\0" in name or name in (".", ".."):
                fail("invalid tree entry")
            names.add(name)
            if item["mode"] not in ("040000", "100644", "100755", "120000", "160000"):
                fail("invalid Git mode")
            if not re.fullmatch("[0-9a-f]{40}", item["sha"]):
                fail("invalid Git object ID")
            mode = "40000" if item["mode"] == "040000" else item["mode"]
            raw.extend(mode.encode() + b" " + name.encode() + b"\0" + bytes.fromhex(item["sha"]))
        if git_hash("tree", bytes(raw)) != tree["sha"]:
            fail("tree hash mismatch")
        trees[tree["path"]] = tree
    if trees[""]["sha"] != SOURCE_TREE:
        fail("root tree mismatch")
    for name, tree in trees.items():
        if not name:
            continue
        parent, _, basename = name.rpartition("/")
        entry = next((x for x in trees[parent]["entries"] if x["path"] == basename), None)
        if not entry or entry["type"] != "tree" or entry["sha"] != tree["sha"]:
            fail("tree linkage mismatch")
    rows = {x["relative_path"]: x for x in manifest["assets"] if x["release_status"] == "release_approved"}
    sources = {x["path"]: x for x in proof["sources"]}
    if len(sources) != 43 or len(proof["sources"]) != 43 or set(sources) != set(rows):
        fail("approved set mismatch")
    if sum(x["size"] for x in sources.values()) != EXPECTED_BYTES:
        fail("source byte total mismatch")
    for name, source in sources.items():
        parent, _, basename = (SOURCE_PREFIX + name).rpartition("/")
        entry = next((x for x in trees[parent]["entries"] if x["path"] == basename), None)
        if not entry or entry["type"] != "blob" or entry["mode"] != "100644" or entry["sha"] != source["blob"]:
            fail("source tree linkage mismatch")
        if rows[name]["sha256"] != source["sha256"]:
            fail("source manifest linkage mismatch")
    base = root / SOURCE_DIR
    for part in (root / "assets", root / "assets/source", base):
        if part.is_symlink() or not part.is_dir():
            fail("source directory missing or symlinked")
    actual = set()
    expected_directories = {name.rpartition("/")[0] for name in sources}
    for path in base.rglob("*"):
        if path.is_symlink():
            fail("symlink source member")
        if path.is_file():
            actual.add(path.relative_to(base).as_posix())
        elif path.is_dir() and path.relative_to(base).as_posix() not in expected_directories:
            fail("extra source directory")
        elif not path.is_dir():
            fail("nonregular source member")
    if actual != set(sources):
        fail("extra or missing source member")
    return sources


def read_approved_source(relative_path, root=REPO):
    sources = load_proof(root)
    if relative_path not in sources:
        fail("source not approved")
    source = sources[relative_path]
    data = exact_file(Path(root), SOURCE_DIR + "/" + relative_path).read_bytes()
    if (len(data) != source["size"] or sha256(data) != source["sha256"]
            or git_hash("blob", data) != source["blob"]):
        fail("source byte identity mismatch")
    return data
