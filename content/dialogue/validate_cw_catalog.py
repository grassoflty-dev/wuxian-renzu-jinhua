#!/usr/bin/env python3
"""Validate the Clockworks Chinese narrative catalog against compiled scenes."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


ID_RE = re.compile(r"^[A-Za-z0-9_.:-]{1,128}$")
SHA256_RE = re.compile(r"^[a-f0-9]{64}$")
ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CATALOG = Path(__file__).with_name("clockworks_narrative_catalog_zh-CN_v1.json")
DEFAULT_SCENES = ROOT / "content/scenes/compiled"
DEFAULT_PROGRESSION = ROOT / "server-rs/data/world_progression_v1.json"
DEFAULT_RUNTIME = ROOT / "governance/assets/RUNTIME_ASSET_MANIFEST.json"
DEFAULT_RELEASE = ROOT / "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"
SCENE_SECTIONS = (
    "interactions", "doors", "triggers", "hazards", "checkpoints",
    "transitions", "objectives", "spawns", "collision",
)
CONFIRMED_COPY = {
    "cw_sys_entry": (695, "生产线无人值守，但压力循环仍在运行。"),
    "cw_cy_heat_01": (700, "炉心不是失控——它被维持在过载边缘。"),
    "cw_cy_end_01": (703, "原来门一直不止三扇。"),
}


class CatalogError(ValueError):
    pass


def _read_json(path: Path, label: str) -> Any:
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
    for path in sorted(compiled_root.glob("cw_*.json")):
        scene = _read_json(path, "compiled scene")
        if not isinstance(scene, dict) or scene.get("schemaVersion") != 1:
            raise CatalogError(f"invalid compiled scene schema: {path.name}")
        if scene.get("worldId") != "clockworks":
            raise CatalogError(f"Clockworks filename has another world identity: {path.name}")
        scene_id = scene.get("sceneId")
        if not isinstance(scene_id, str) or path.name != f"{scene_id}.json":
            raise CatalogError(f"compiled scene identity/path mismatch: {path.name}")
        if scene_id in scenes:
            raise CatalogError(f"duplicate compiled scene ID: {scene_id}")
        scenes[scene_id] = scene
    if not scenes:
        raise CatalogError("no compiled Clockworks scenes found")
    return scenes


def load_required_events(progression_path: Path) -> set[str]:
    data = _read_json(progression_path, "progression catalog")
    worlds = data.get("worlds") if isinstance(data, dict) else None
    world = next((row for row in worlds or [] if row.get("worldId") == "clockworks"), None)
    if not isinstance(world, dict) or not isinstance(world.get("requiredEvents"), list):
        raise CatalogError("Clockworks required events are missing from progression catalog")
    required = world["requiredEvents"]
    if any(not isinstance(item, str) or not ID_RE.fullmatch(item) for item in required):
        raise CatalogError("Clockworks required event ID is invalid")
    return set(required)


def load_approved_assets(runtime_path: Path, release_path: Path) -> tuple[set[str], set[str]]:
    runtime = _read_json(runtime_path, "runtime asset manifest")
    release = _read_json(release_path, "asset release manifest")
    runtime_rows = runtime.get("assets") if isinstance(runtime, dict) else None
    release_rows = release.get("assets") if isinstance(release, dict) else None
    if not isinstance(runtime_rows, list) or not isinstance(release_rows, list):
        raise CatalogError("asset manifest has no assets array")
    runtime_ids = {row.get("assetId") for row in runtime_rows if isinstance(row, dict)}
    approved_ids = {
        row.get("asset_id") for row in release_rows
        if isinstance(row, dict) and row.get("release_status") == "release_approved"
    }
    return runtime_ids, approved_ids


def _trigger_rows(scene: dict[str, Any]) -> dict[str, list[tuple[str, dict[str, Any]]]]:
    found: dict[str, list[tuple[str, dict[str, Any]]]] = {}
    for section in SCENE_SECTIONS:
        rows = scene.get(section, [])
        if not isinstance(rows, list):
            raise CatalogError(f"invalid compiled scene section: {scene['sceneId']}.{section}")
        for row in rows:
            if isinstance(row, dict) and isinstance(row.get("id"), str):
                found.setdefault(row["id"], []).append((section, row))
    logic = scene.get("logic", {})
    traversal = logic.get("traversal", []) if isinstance(logic, dict) else None
    if not isinstance(traversal, list):
        raise CatalogError(f"invalid compiled scene section: {scene['sceneId']}.logic.traversal")
    for row in traversal:
        if isinstance(row, dict) and isinstance(row.get("id"), str):
            found.setdefault(row["id"], []).append(("traversal", row))
    return found


def _scene_asset_ids(scene: dict[str, Any]) -> set[str]:
    assets: set[str] = set()
    presentation = scene.get("presentation", {})
    if not isinstance(presentation, dict):
        raise CatalogError(f"invalid presentation section: {scene['sceneId']}")
    for row in presentation.get("sprites", []):
        if not isinstance(row, dict) or not isinstance(row.get("assetId"), str):
            raise CatalogError(f"invalid scene sprite asset: {scene['sceneId']}")
        assets.add(row["assetId"])
    for row in scene.get("vfxMarkers", []):
        if not isinstance(row, dict) or not isinstance(row.get("assetId"), str):
            raise CatalogError(f"invalid scene VFX asset: {scene['sceneId']}")
        assets.add(row["assetId"])
    for row in scene.get("doors", []):
        if isinstance(row, dict) and isinstance(row.get("assetId"), str):
            assets.add(row["assetId"])
    return assets


def validate_catalog(
    catalog: dict[str, Any],
    scenes: dict[str, dict[str, Any]],
    required_events: set[str],
    runtime_asset_ids: set[str],
    approved_asset_ids: set[str],
) -> dict[str, Any]:
    if not isinstance(catalog, dict) or catalog.get("schemaVersion") != 1:
        raise CatalogError("unsupported narrative catalog schemaVersion")
    if catalog.get("locale") != "zh-CN":
        raise CatalogError("narrative catalog locale must be zh-CN")
    if not isinstance(catalog.get("catalogId"), str) or not ID_RE.fullmatch(catalog["catalogId"]):
        raise CatalogError("invalid catalogId")
    if not isinstance(catalog.get("designSourceSha256"), str) or not SHA256_RE.fullmatch(catalog["designSourceSha256"]):
        raise CatalogError("invalid design source SHA-256")

    scene_ids = catalog.get("scenes")
    if not isinstance(scene_ids, list) or any(not isinstance(value, str) for value in scene_ids):
        raise CatalogError("scenes must be an array of scene IDs")
    if len(scene_ids) != len(set(scene_ids)):
        raise CatalogError("duplicate scene reference in catalog")
    if set(scene_ids) != set(scenes):
        missing = sorted(set(scenes) - set(scene_ids))
        unknown = sorted(set(scene_ids) - set(scenes))
        raise CatalogError(f"compiled scene coverage mismatch: missing={missing}; unknown={unknown}")

    progression = catalog.get("progressionEvents")
    if not isinstance(progression, list):
        raise CatalogError("progressionEvents must be an array")
    progression_by_id: dict[str, dict[str, Any]] = {}
    if {row.get("id") for row in progression if isinstance(row, dict)} != required_events:
        raise CatalogError("catalog progression event references do not match Clockworks required events")
    emitted_events = {
        row.get("event")
        for scene in scenes.values()
        for section in ("interactions", "triggers")
        for row in scene.get(section, [])
        if isinstance(row, dict) and row.get("event")
    }
    for index, event in enumerate(progression):
        if not isinstance(event, dict):
            raise CatalogError(f"progressionEvents[{index}] must be an object")
        event_id = event.get("id")
        if not isinstance(event_id, str) or not ID_RE.fullmatch(event_id):
            raise CatalogError(f"progressionEvents[{index}] has invalid ID")
        if event_id in progression_by_id:
            raise CatalogError(f"duplicate progression event reference: {event_id}")
        progression_by_id[event_id] = event
        scene_id, marker_id, marker_kind = event.get("sceneId"), event.get("markerId"), event.get("markerKind")
        if scene_id not in scenes or not isinstance(marker_id, str) or not isinstance(marker_kind, str):
            raise CatalogError(f"{event_id} has an invalid staged marker reference")
        marker = next((row for row in scenes[scene_id].get("interactions", [])
                       if isinstance(row, dict) and row.get("id") == marker_id), None)
        if marker is None or marker.get("kind") != marker_kind or marker.get("event") is not None:
            raise CatalogError(f"{event_id} must reference its exact eventless staged marker")
        if event.get("status") != "pending_native_implementation":
            raise CatalogError(f"{event_id} must remain pending until native implementation exists")
        if event_id in emitted_events:
            raise CatalogError(f"{event_id} is marked pending but is emitted by a compiled scene")

    entries = catalog.get("entries")
    if not isinstance(entries, list):
        raise CatalogError("entries must be an array")
    ids: set[str] = set()
    counts = {"bound": 0, "staged_marker": 0, "pending_scene_hook": 0}
    confirmed_count = pending_count = 0
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
        trigger_id, trigger_kind = entry.get("triggerId"), entry.get("triggerKind")
        if not isinstance(trigger_id, str) or not ID_RE.fullmatch(trigger_id):
            raise CatalogError(f"{entry_id} has invalid triggerId")
        if not isinstance(trigger_kind, str) or not trigger_kind.strip():
            raise CatalogError(f"{entry_id} has invalid triggerKind")
        status = entry.get("triggerStatus")
        if status not in counts:
            raise CatalogError(f"{entry_id} has invalid triggerStatus")
        trigger_rows = _trigger_rows(scenes[scene_id]).get(trigger_id, [])
        if status == "bound":
            if len(trigger_rows) != 1:
                raise CatalogError(f"{entry_id} bound trigger is missing or ambiguous: {trigger_id}")
            section, row = trigger_rows[0]
            expected_kind = "controlled_traversal_marker" if section == "traversal" else row.get("kind")
            if expected_kind != trigger_kind:
                raise CatalogError(f"{entry_id} bound trigger kind mismatch: {trigger_id}")
        elif status == "staged_marker":
            if len(trigger_rows) != 1:
                raise CatalogError(f"{entry_id} staged marker is missing or ambiguous: {trigger_id}")
            section, row = trigger_rows[0]
            if section != "interactions" or row.get("kind") != trigger_kind or row.get("event") is not None:
                raise CatalogError(f"{entry_id} staged marker must be its exact eventless interaction")
        elif trigger_rows:
            raise CatalogError(f"{entry_id} pending scene hook unexpectedly exists: {trigger_id}")
        counts[status] += 1

        condition = entry.get("triggerCondition")
        if not isinstance(condition, str) or not condition.strip():
            raise CatalogError(f"{entry_id} has no trigger condition")
        source_line = entry.get("sourceLine")
        if isinstance(source_line, bool) or not isinstance(source_line, int) or source_line <= 0:
            raise CatalogError(f"{entry_id} has invalid sourceLine")
        if not isinstance(entry.get("evidence"), str) or not entry["evidence"].strip():
            raise CatalogError(f"{entry_id} has no design evidence note")
        body_status, body = entry.get("bodyStatus"), entry.get("body")
        if body_status == "confirmed":
            if entry_id not in CONFIRMED_COPY:
                raise CatalogError(f"{entry_id} has no source-confirmed copy")
            expected_line, expected_body = CONFIRMED_COPY[entry_id]
            if body != expected_body or source_line != expected_line:
                raise CatalogError(f"{entry_id} confirmed body does not match source text")
            if status != "staged_marker":
                raise CatalogError(f"{entry_id} confirmed text must remain on a staged marker")
            confirmed_count += 1
        elif body_status == "pending":
            if body is not None:
                raise CatalogError(f"{entry_id} pending body must be null")
            pending_count += 1
        else:
            raise CatalogError(f"{entry_id} has invalid bodyStatus")

        event_id = entry.get("relatedEventId")
        if event_id is not None:
            event = progression_by_id.get(event_id)
            if event is None:
                raise CatalogError(f"{entry_id} references an unknown progression event: {event_id}")
            if event.get("sceneId") != scene_id or event.get("markerId") != trigger_id:
                raise CatalogError(f"{entry_id} event reference is not attached to its exact staged marker")

    if set(CONFIRMED_COPY) != {entry["id"] for entry in entries if entry.get("bodyStatus") == "confirmed"}:
        raise CatalogError("catalog confirmed copy set does not match source-confirmed Clockworks lines")

    referenced_assets = set().union(*(_scene_asset_ids(scene) for scene in scenes.values()))
    missing_runtime = referenced_assets - runtime_asset_ids
    unapproved = referenced_assets - approved_asset_ids
    if missing_runtime:
        raise CatalogError(f"compiled Clockworks scenes reference unknown runtime assets: {sorted(missing_runtime)}")
    if unapproved:
        raise CatalogError(f"compiled Clockworks scenes reference unapproved assets: {sorted(unapproved)}")

    return {
        "result": "PASS",
        "catalogId": catalog["catalogId"],
        "sceneCount": len(scene_ids),
        "entryCount": len(entries),
        "confirmedBodyCount": confirmed_count,
        "pendingBodyCount": pending_count,
        "boundTriggerCount": counts["bound"],
        "stagedMarkerCount": counts["staged_marker"],
        "pendingSceneHookCount": counts["pending_scene_hook"],
        "pendingProgressionEventCount": len(progression_by_id),
        "approvedSceneAssetCount": len(referenced_assets),
    }


def validate_files(
    catalog_path: Path = DEFAULT_CATALOG,
    compiled_root: Path = DEFAULT_SCENES,
    progression_path: Path = DEFAULT_PROGRESSION,
    runtime_path: Path = DEFAULT_RUNTIME,
    release_path: Path = DEFAULT_RELEASE,
) -> dict[str, Any]:
    catalog = _read_json(catalog_path, "narrative catalog")
    scenes = load_compiled_scenes(compiled_root)
    runtime_ids, approved_ids = load_approved_assets(runtime_path, release_path)
    return validate_catalog(catalog, scenes, load_required_events(progression_path), runtime_ids, approved_ids)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    parser.add_argument("--compiled-scenes", type=Path, default=DEFAULT_SCENES)
    parser.add_argument("--progression", type=Path, default=DEFAULT_PROGRESSION)
    parser.add_argument("--runtime-manifest", type=Path, default=DEFAULT_RUNTIME)
    parser.add_argument("--release-manifest", type=Path, default=DEFAULT_RELEASE)
    args = parser.parse_args()
    try:
        print(json.dumps(validate_files(args.catalog, args.compiled_scenes, args.progression,
                                        args.runtime_manifest, args.release_manifest),
                         ensure_ascii=False, sort_keys=True))
    except (CatalogError, OSError) as exc:
        parser.exit(1, f"cw-narrative-catalog: {exc}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
