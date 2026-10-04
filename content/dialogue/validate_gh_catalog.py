#!/usr/bin/env python3
"""Validate the Grey Hive Chinese narrative catalog against compiled scenes."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


ID_RE = re.compile(r"^[A-Za-z0-9_.:-]{1,128}$")
CHOICE_ID_RE = re.compile(r"^[a-z][a-z0-9_]{0,31}$")
SHA256_RE = re.compile(r"^[a-f0-9]{64}$")
DESIGN_SOURCE_SHA256 = "fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1"
DESIGN_SOURCE_LINE_RANGE = "594-624"
ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CATALOG = Path(__file__).with_name("grey_hive_narrative_catalog_zh-CN_v1.json")
DEFAULT_SCENES = ROOT / "content/scenes/compiled"
TRIGGER_SECTIONS = (
    "interactions", "doors", "triggers", "hazards", "checkpoints",
    "transitions", "objectives", "spawns", "collision",
)


class CatalogError(ValueError):
    pass


def _read_json_utf8(path: Path, label: str) -> Any:
    try:
        text = path.read_bytes().decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise CatalogError(f"{label} is not valid UTF-8: {path}") from exc
    try:
        return json.loads(text)
    except json.JSONDecodeError as exc:
        raise CatalogError(f"{label} is not valid JSON: {path}: {exc}") from exc


def load_compiled_scenes(compiled_root: Path) -> dict[str, dict[str, Any]]:
    if not compiled_root.is_dir():
        raise CatalogError(f"compiled scene directory is missing: {compiled_root}")
    scenes: dict[str, dict[str, Any]] = {}
    for path in sorted(compiled_root.glob("*.json")):
        scene = _read_json_utf8(path, "compiled scene")
        if not isinstance(scene, dict) or scene.get("schemaVersion") != 1:
            raise CatalogError(f"invalid compiled scene schema: {path.name}")
        if scene.get("worldId") != "grey_hive":
            continue
        scene_id = scene.get("sceneId")
        if not isinstance(scene_id, str) or path.name != f"{scene_id}.json":
            raise CatalogError(f"compiled scene identity/path mismatch: {path.name}")
        if scene_id in scenes:
            raise CatalogError(f"duplicate compiled scene ID: {scene_id}")
        scenes[scene_id] = scene
    if not scenes:
        raise CatalogError("no compiled Grey Hive scenes found")
    return scenes


def _scene_trigger_ids(scene: dict[str, Any]) -> set[str]:
    ids: set[str] = set()
    for section in TRIGGER_SECTIONS:
        rows = scene.get(section, [])
        if not isinstance(rows, list):
            raise CatalogError(f"invalid compiled scene section: {scene['sceneId']}.{section}")
        for row in rows:
            if isinstance(row, dict) and isinstance(row.get("id"), str):
                ids.add(row["id"])
    return ids


def validate_catalog(catalog: dict[str, Any], scenes: dict[str, dict[str, Any]]) -> dict[str, Any]:
    if not isinstance(catalog, dict) or catalog.get("schemaVersion") != 1:
        raise CatalogError("unsupported narrative catalog schemaVersion")
    if catalog.get("locale") != "zh-CN":
        raise CatalogError("narrative catalog locale must be zh-CN")
    if not isinstance(catalog.get("catalogId"), str) or not ID_RE.fullmatch(catalog["catalogId"]):
        raise CatalogError("invalid catalogId")
    source_sha = catalog.get("designSourceSha256")
    if not isinstance(source_sha, str) or not SHA256_RE.fullmatch(source_sha) or source_sha != DESIGN_SOURCE_SHA256:
        raise CatalogError("designSourceSha256 does not match the authoritative design report")
    if catalog.get("sourceLineRange") != DESIGN_SOURCE_LINE_RANGE:
        raise CatalogError("sourceLineRange must cover the Grey Hive design section 594-624")

    scene_ids = catalog.get("scenes")
    if not isinstance(scene_ids, list) or any(not isinstance(value, str) for value in scene_ids):
        raise CatalogError("scenes must be an array of scene IDs")
    if len(scene_ids) != len(set(scene_ids)):
        raise CatalogError("duplicate scene reference in catalog")
    if set(scene_ids) != set(scenes):
        missing = sorted(set(scenes) - set(scene_ids))
        unknown = sorted(set(scene_ids) - set(scenes))
        raise CatalogError(f"compiled scene coverage mismatch: missing={missing}; unknown={unknown}")

    constraints = catalog.get("progressionConstraints")
    if not isinstance(constraints, dict):
        raise CatalogError("progressionConstraints is required")
    if constraints.get("gateBRequiredEvent") != "hive_lockdown":
        raise CatalogError("Gate B must retain the hive_lockdown requirement")
    if constraints.get("baizhiChoiceIsGateBKey") is not False:
        raise CatalogError("Bai Zhi choices must not act as a Gate B key")

    entries = catalog.get("entries")
    if not isinstance(entries, list):
        raise CatalogError("entries must be an array")
    ids: set[str] = set()
    bound_count = 0
    pending_hook_count = 0
    confirmed_body_count = 0
    pending_body_count = 0
    for index, entry in enumerate(entries):
        label = f"entries[{index}]"
        if not isinstance(entry, dict):
            raise CatalogError(f"{label} must be an object")
        entry_id = entry.get("id")
        if not isinstance(entry_id, str) or not ID_RE.fullmatch(entry_id):
            raise CatalogError(f"{label} has invalid ID")
        if entry_id in ids:
            raise CatalogError(f"duplicate narrative ID: {entry_id}")
        ids.add(entry_id)

        scene_id = entry.get("sceneId")
        if scene_id not in scenes:
            raise CatalogError(f"{entry_id} references unknown compiled scene: {scene_id}")
        trigger_id = entry.get("triggerId")
        if not isinstance(trigger_id, str) or not ID_RE.fullmatch(trigger_id):
            raise CatalogError(f"{entry_id} has invalid triggerId")
        trigger_status = entry.get("triggerStatus")
        if trigger_status not in {"bound", "pending_scene_hook"}:
            raise CatalogError(f"{entry_id} has invalid triggerStatus")
        present = trigger_id in _scene_trigger_ids(scenes[scene_id])
        if trigger_status == "bound" and not present:
            raise CatalogError(f"{entry_id} trigger is not present in {scene_id}: {trigger_id}")
        if trigger_status == "pending_scene_hook" and present:
            raise CatalogError(f"{entry_id} is marked pending but its trigger now exists: {trigger_id}")
        if trigger_status == "pending_scene_hook":
            pending_hook_count += 1
        else:
            bound_count += 1

        source_line = entry.get("sourceLine")
        if isinstance(source_line, bool) or not isinstance(source_line, int) or source_line <= 0:
            raise CatalogError(f"{entry_id} has invalid sourceLine")
        if not isinstance(entry.get("evidence"), str) or not entry["evidence"].strip():
            raise CatalogError(f"{entry_id} has no design evidence note")

        body_status, body = entry.get("bodyStatus"), entry.get("body")
        if body_status == "confirmed":
            if not isinstance(body, str) or not body.strip():
                raise CatalogError(f"{entry_id} confirmed body must contain text")
            confirmed_body_count += 1
        elif body_status == "pending":
            if body is not None:
                raise CatalogError(f"{entry_id} pending body must be null")
            pending_body_count += 1
        else:
            raise CatalogError(f"{entry_id} has invalid bodyStatus")

        title = entry.get("title")
        if title is not None and (not isinstance(title, str) or not title.strip()):
            raise CatalogError(f"{entry_id} title must be non-empty text or null")

        choices = entry.get("choices", [])
        if not isinstance(choices, list):
            raise CatalogError(f"{entry_id} choices must be an array")
        choice_ids: set[str] = set()
        for choice in choices:
            if not isinstance(choice, dict):
                raise CatalogError(f"{entry_id} has a malformed choice")
            choice_id = choice.get("id")
            if not isinstance(choice_id, str) or not CHOICE_ID_RE.fullmatch(choice_id):
                raise CatalogError(f"{entry_id} has illegal choice ID: {choice_id}")
            if choice_id in choice_ids:
                raise CatalogError(f"{entry_id} has duplicate choice ID: {choice_id}")
            choice_ids.add(choice_id)
            label_status, choice_label = choice.get("labelStatus"), choice.get("label")
            if label_status == "pending" and choice_label is not None:
                raise CatalogError(f"{entry_id}.{choice_id} pending label must be null")
            if label_status == "confirmed" and (not isinstance(choice_label, str) or not choice_label.strip()):
                raise CatalogError(f"{entry_id}.{choice_id} confirmed label must contain text")
            if label_status not in {"pending", "confirmed"}:
                raise CatalogError(f"{entry_id}.{choice_id} has invalid labelStatus")
            if choice.get("gateBKeyEffect") != "none":
                raise CatalogError(f"{entry_id}.{choice_id} must not grant a Gate B key")
        if entry_id == "gh_bz_first_01" and choice_ids != {"taken", "left", "unresolved"}:
            raise CatalogError("gh_bz_first_01 must preserve exactly taken/left/unresolved choice IDs")

    return {
        "result": "PASS",
        "catalogId": catalog["catalogId"],
        "sceneCount": len(scene_ids),
        "entryCount": len(entries),
        "boundTriggerCount": bound_count,
        "pendingSceneHookCount": pending_hook_count,
        "confirmedBodyCount": confirmed_body_count,
        "pendingBodyCount": pending_body_count,
        "choiceIds": sorted(ids for entry in entries for ids in [choice["id"] for choice in entry.get("choices", [])]),
    }


def validate_files(catalog_path: Path, compiled_root: Path = DEFAULT_SCENES) -> dict[str, Any]:
    catalog = _read_json_utf8(catalog_path, "narrative catalog")
    scenes = load_compiled_scenes(compiled_root)
    return validate_catalog(catalog, scenes)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    parser.add_argument("--compiled-scenes", type=Path, default=DEFAULT_SCENES)
    args = parser.parse_args()
    try:
        print(json.dumps(validate_files(args.catalog, args.compiled_scenes), ensure_ascii=False, sort_keys=True))
    except (CatalogError, OSError) as exc:
        parser.exit(1, f"gh-narrative-catalog: {exc}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
