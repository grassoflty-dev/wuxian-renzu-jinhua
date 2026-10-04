#!/usr/bin/env python3
"""Validate the Return Station Chinese narrative catalog against its compiled room."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

ID_RE = re.compile(r"^[A-Za-z0-9_.:-]{1,128}$")
SHA256_RE = re.compile(r"^[a-f0-9]{64}$")
ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CATALOG = Path(__file__).with_name("return_station_narrative_catalog_zh-CN_v1.json")
DEFAULT_SCENES = ROOT / "content/scenes/compiled"
DEFAULT_PROGRESSION = ROOT / "server-rs/data/world_progression_v1.json"
DEFAULT_RUNTIME = ROOT / "governance/assets/RUNTIME_ASSET_MANIFEST.json"
DEFAULT_RELEASE = ROOT / "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"
SCENE_SECTIONS = (
    "interactions", "doors", "triggers", "hazards", "checkpoints",
    "transitions", "objectives", "spawns", "collision",
)
SOURCE_SHA256 = "fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1"
CONFIRMED_COPY = {
    "rs_intro": (724, "归航链路稳定。请选择首次回收坐标：灰巢设施。"),
    "rs_after_gh": (725, "灰巢信标已解析。检测到第二组低可信坐标：雾港余烬。"),
    "rs_after_mh": (727, "声学映射已固化。第三坐标的机械周期与信号高度同步。"),
    "rs_after_cw": (728, "三处坐标不是孤立事件。网络中仍存在未识别节点。"),
}
EXPECTED_STATES = {
    "new_journey": ("return_station", "implemented_entry_flow", True),
    "grey_hive_first_clear": ("grey_hive", "implemented_progression_state", True),
    "mist_harbor_first_clear": ("mist_harbor", "progression_prerequisite_only", False),
    "clockworks_first_clear": ("clockworks", "progression_prerequisite_only", False),
    "three_worlds_complete": ("campaign", "aggregate_prerequisite_only", False),
}


class CatalogError(ValueError):
    pass


def _read_json(path: Path, label: str) -> Any:
    try:
        raw = path.read_bytes().decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise CatalogError(f"{label} is not valid UTF-8: {path}") from exc
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise CatalogError(f"{label} is not valid JSON: {path}: {exc}") from exc


def load_compiled_scenes(compiled_root: Path) -> dict[str, dict[str, Any]]:
    path = compiled_root / "rs_core_room.json"
    if not path.is_file():
        raise CatalogError(f"compiled Return Station Core Room is missing: {path}")
    scene = _read_json(path, "compiled scene")
    if (not isinstance(scene, dict) or scene.get("schemaVersion") != 1
            or scene.get("sceneId") != "rs_core_room" or scene.get("worldId") != "return_station"):
        raise CatalogError("compiled Return Station Core Room identity or schema is invalid")
    return {"rs_core_room": scene}


def load_progression(progression_path: Path) -> dict[str, list[str]]:
    data = _read_json(progression_path, "progression catalog")
    worlds = data.get("worlds") if isinstance(data, dict) else None
    if not isinstance(worlds, list):
        raise CatalogError("progression catalog has no worlds array")
    result: dict[str, list[str]] = {}
    for world in worlds:
        if not isinstance(world, dict):
            raise CatalogError("progression catalog contains a malformed world")
        world_id, events = world.get("worldId"), world.get("requiredEvents")
        if not isinstance(world_id, str) or not isinstance(events, list):
            raise CatalogError("progression catalog world has invalid identity or required events")
        if any(not isinstance(value, str) or not ID_RE.fullmatch(value) for value in events):
            raise CatalogError(f"invalid required event ID in {world_id}")
        result[world_id] = events
    return result


def load_approved_assets(runtime_path: Path, release_path: Path) -> tuple[set[str], set[str]]:
    runtime = _read_json(runtime_path, "runtime asset manifest")
    release = _read_json(release_path, "asset release manifest")
    runtime_rows = runtime.get("assets") if isinstance(runtime, dict) else None
    release_rows = release.get("assets") if isinstance(release, dict) else None
    if not isinstance(runtime_rows, list) or not isinstance(release_rows, list):
        raise CatalogError("asset manifest has no assets array")
    runtime_ids = {row.get("assetId") for row in runtime_rows if isinstance(row, dict)}
    approved_ids = {row.get("asset_id") for row in release_rows
                    if isinstance(row, dict) and row.get("release_status") == "release_approved"}
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
        raise CatalogError("invalid compiled scene section: rs_core_room.logic.traversal")
    for row in traversal:
        if isinstance(row, dict) and isinstance(row.get("id"), str):
            found.setdefault(row["id"], []).append(("traversal", row))
    return found


def _scene_asset_ids(scene: dict[str, Any]) -> set[str]:
    assets: set[str] = set()
    presentation = scene.get("presentation", {})
    if not isinstance(presentation, dict):
        raise CatalogError("invalid Return Station presentation section")
    for row in presentation.get("sprites", []):
        if not isinstance(row, dict) or not isinstance(row.get("assetId"), str):
            raise CatalogError("invalid Return Station sprite asset")
        assets.add(row["assetId"])
    for row in scene.get("vfxMarkers", []):
        if not isinstance(row, dict) or not isinstance(row.get("assetId"), str):
            raise CatalogError("invalid Return Station VFX asset")
        assets.add(row["assetId"])
    for row in scene.get("doors", []):
        if isinstance(row, dict) and isinstance(row.get("assetId"), str):
            assets.add(row["assetId"])
    return assets


def validate_catalog(
    catalog: dict[str, Any], scene: dict[str, Any], progression: dict[str, list[str]],
    runtime_asset_ids: set[str], approved_asset_ids: set[str],
) -> dict[str, Any]:
    if not isinstance(catalog, dict) or catalog.get("schemaVersion") != 1:
        raise CatalogError("unsupported narrative catalog schemaVersion")
    if catalog.get("locale") != "zh-CN":
        raise CatalogError("narrative catalog locale must be zh-CN")
    if not isinstance(catalog.get("catalogId"), str) or not ID_RE.fullmatch(catalog["catalogId"]):
        raise CatalogError("invalid catalogId")
    if catalog.get("designSourceSha256") != SOURCE_SHA256 or not SHA256_RE.fullmatch(catalog.get("designSourceSha256", "")):
        raise CatalogError("design source SHA-256 does not match the verified design report")
    scene_ids = catalog.get("scenes")
    if scene_ids != ["rs_core_room"]:
        raise CatalogError("catalog must cover exactly the compiled rs_core_room scene")

    state_bindings = catalog.get("progressionStateBindings")
    if not isinstance(state_bindings, list):
        raise CatalogError("progressionStateBindings must be an array")
    state_by_id: dict[str, dict[str, Any]] = {}
    if len(state_bindings) != len(EXPECTED_STATES):
        raise CatalogError("progression state binding set is incomplete")
    for state in state_bindings:
        if not isinstance(state, dict):
            raise CatalogError("progression state binding must be an object")
        state_id = state.get("id")
        if state_id not in EXPECTED_STATES or state_id in state_by_id:
            raise CatalogError(f"unknown or duplicate progression state binding: {state_id}")
        expected_world, expected_status, expected_claim = EXPECTED_STATES[state_id]
        if (state.get("worldId"), state.get("status"), state.get("playabilityClaimed")) != (
                expected_world, expected_status, expected_claim):
            raise CatalogError(f"{state_id} progression state status or claim is invalid")
        if state_id in {"new_journey", "grey_hive_first_clear"}:
            expected_events: list[str] = [] if state_id == "new_journey" else progression.get("grey_hive", [])
        elif state_id == "three_worlds_complete":
            expected_events = [event for world_id in ("grey_hive", "mist_harbor", "clockworks")
                               for event in progression.get(world_id, [])]
        else:
            expected_events = progression.get(expected_world, [])
        if state.get("requiredEvents") != expected_events:
            raise CatalogError(f"{state_id} required events do not match authoritative progression catalog")
        state_by_id[state_id] = state

    claims = catalog.get("implementationClaims")
    expected_claims = {
        "newJourneyStartsInReturnStation": True,
        "greyHiveFirstClearReturnsToReturnStation": True,
        "mistHarborReturnAvailable": False,
        "clockworksReturnAvailable": False,
        "capabilityTerminalServiceImplemented": False,
        "threeWorldEpiloguePlayable": False,
    }
    if claims != expected_claims:
        raise CatalogError("implementation claims overstate Return Station or three-world functionality")

    entries = catalog.get("entries")
    if not isinstance(entries, list):
        raise CatalogError("entries must be an array")
    trigger_rows = _trigger_rows(scene)
    ids: set[str] = set()
    counts = {"staged_marker": 0, "pending_scene_hook": 0}
    confirmed = pending = 0
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            raise CatalogError(f"entries[{index}] must be an object")
        entry_id = entry.get("id")
        if not isinstance(entry_id, str) or not ID_RE.fullmatch(entry_id):
            raise CatalogError(f"entries[{index}] has invalid ID")
        if entry_id in ids:
            raise CatalogError(f"duplicate narrative ID: {entry_id}")
        ids.add(entry_id)
        if entry.get("sceneId") != "rs_core_room":
            raise CatalogError(f"{entry_id} references an unknown compiled scene")
        trigger_id, trigger_kind = entry.get("triggerId"), entry.get("triggerKind")
        if not isinstance(trigger_id, str) or not ID_RE.fullmatch(trigger_id):
            raise CatalogError(f"{entry_id} has invalid triggerId")
        if not isinstance(trigger_kind, str) or not trigger_kind.strip():
            raise CatalogError(f"{entry_id} has invalid triggerKind")
        state_id = entry.get("progressionStateId")
        if state_id not in state_by_id:
            raise CatalogError(f"{entry_id} has no known progression state binding")
        status = entry.get("triggerStatus")
        if status not in counts:
            raise CatalogError(f"{entry_id} has invalid triggerStatus")
        matches = trigger_rows.get(trigger_id, [])
        if status == "staged_marker":
            if len(matches) != 1:
                raise CatalogError(f"{entry_id} staged marker is missing or ambiguous: {trigger_id}")
            section, marker = matches[0]
            if section != "interactions" or marker.get("kind") != trigger_kind or marker.get("event") is not None:
                raise CatalogError(f"{entry_id} staged marker must match its eventless compiled interaction")
        elif matches:
            raise CatalogError(f"{entry_id} pending scene hook unexpectedly exists: {trigger_id}")
        counts[status] += 1
        condition, evidence = entry.get("triggerCondition"), entry.get("evidence")
        if not isinstance(condition, str) or not condition.strip():
            raise CatalogError(f"{entry_id} has no trigger condition")
        if not isinstance(evidence, str) or not evidence.strip():
            raise CatalogError(f"{entry_id} has no design evidence note")
        line = entry.get("sourceLine")
        if isinstance(line, bool) or not isinstance(line, int) or line not in range(707, 730):
            raise CatalogError(f"{entry_id} has invalid sourceLine")
        body_status, body = entry.get("bodyStatus"), entry.get("body")
        if body_status == "confirmed":
            if entry_id not in CONFIRMED_COPY:
                raise CatalogError(f"{entry_id} has no source-confirmed copy")
            expected_line, expected_body = CONFIRMED_COPY[entry_id]
            if line != expected_line or body != expected_body:
                raise CatalogError(f"{entry_id} confirmed body does not match source text")
            confirmed += 1
        elif body_status == "pending":
            if body is not None:
                raise CatalogError(f"{entry_id} pending body must be null")
            pending += 1
        else:
            raise CatalogError(f"{entry_id} has invalid bodyStatus")
    if {entry["id"] for entry in entries if entry.get("bodyStatus") == "confirmed"} != set(CONFIRMED_COPY):
        raise CatalogError("confirmed copy set does not match the four source-confirmed Return Station lines")
    if ids != {"rs_intro", "rs_after_gh", "rs_first_evolution", "rs_after_mh", "rs_after_cw", "rs_v1_epilogue"}:
        raise CatalogError("catalog entry set does not cover all six Return Station narrative nodes")

    assets = _scene_asset_ids(scene)
    missing_runtime, unapproved = assets - runtime_asset_ids, assets - approved_asset_ids
    if missing_runtime:
        raise CatalogError(f"compiled Return Station scene references unknown runtime assets: {sorted(missing_runtime)}")
    if unapproved:
        raise CatalogError(f"compiled Return Station scene references unapproved assets: {sorted(unapproved)}")
    return {
        "result": "PASS", "catalogId": catalog["catalogId"], "sceneCount": 1,
        "entryCount": len(entries), "confirmedBodyCount": confirmed, "pendingBodyCount": pending,
        "stagedMarkerCount": counts["staged_marker"], "pendingSceneHookCount": counts["pending_scene_hook"],
        "progressionStateBindingCount": len(state_by_id), "approvedSceneAssetCount": len(assets),
        "mistHarborReturnAvailable": False, "clockworksReturnAvailable": False,
        "capabilityTerminalServiceImplemented": False, "threeWorldEpiloguePlayable": False,
    }


def validate_files(catalog_path: Path = DEFAULT_CATALOG, compiled_root: Path = DEFAULT_SCENES,
                   progression_path: Path = DEFAULT_PROGRESSION, runtime_path: Path = DEFAULT_RUNTIME,
                   release_path: Path = DEFAULT_RELEASE) -> dict[str, Any]:
    catalog = _read_json(catalog_path, "narrative catalog")
    scene = load_compiled_scenes(compiled_root)["rs_core_room"]
    progression = load_progression(progression_path)
    runtime_ids, approved_ids = load_approved_assets(runtime_path, release_path)
    return validate_catalog(catalog, scene, progression, runtime_ids, approved_ids)


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
        parser.exit(1, f"rs-narrative-catalog: {exc}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
