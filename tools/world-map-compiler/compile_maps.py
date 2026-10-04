#!/usr/bin/env python3
"""Compile Tiled TMJ maps into deterministic SceneDefinition JSON."""

from __future__ import annotations

import argparse
import json
import math
import re
from collections import deque
from pathlib import Path
from typing import Any


class MapError(ValueError):
    pass


def environment_object(raw: Any, label: str, required: set[str], optional: set[str] = frozenset()) -> dict[str, Any]:
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise MapError(f"duplicate {label} key: {key}")
            result[key] = value
        return result
    if not isinstance(raw, str) or len(raw) > 8192:
        raise MapError(f"{label} must be a bounded JSON string")
    try:
        value = json.loads(raw, object_pairs_hook=unique)
    except (ValueError, TypeError) as error:
        raise MapError(f"invalid {label}: {error}") from error
    if not isinstance(value, dict) or not required.issubset(value) or set(value) - required - optional:
        raise MapError(f"invalid {label} fields")
    return value


def environment_integer(value: Any, low: int, high: int, label: str) -> None:
    if isinstance(value, bool) or not isinstance(value, int) or not low <= value <= high:
        raise MapError(f"invalid environment {label}")


def environment_hazard(raw: Any, kind: str) -> dict[str, Any]:
    value = environment_object(raw, "environment hazard", {"tag", "warningMs", "activeMs", "recoveryMs", "damage", "damageIntervalMs"}, {"exposure"})
    expected_tag = {"heat_zone": "heat", "steam_jet": "pressure", "decon_mist": "toxin"}.get(kind)
    if expected_tag is None or value["tag"] != expected_tag:
        raise MapError("environment hazard kind/tag mismatch")
    for key, low, high in [("warningMs", 1, 10_000), ("activeMs", 100, 60_000), ("recoveryMs", 100, 60_000), ("damage", 1, 10_000), ("damageIntervalMs", 100, 60_000)]:
        environment_integer(value[key], low, high, key)
    if "exposure" in value:
        exposure = value["exposure"]
        if value["tag"] != "heat" or not isinstance(exposure, dict) or set(exposure) != {"maxUnits", "gainPerSecond", "lossPerSecond", "damageThreshold"}:
            raise MapError("invalid environment exposure fields")
        for key in ["maxUnits", "gainPerSecond", "lossPerSecond", "damageThreshold"]:
            environment_integer(exposure[key], 1, 10_000, key)
        if exposure["damageThreshold"] > exposure["maxUnits"]:
            raise MapError("environment exposure threshold exceeds maximum")
    return value


def environment_control(raw: Any, interaction: dict[str, Any]) -> dict[str, Any]:
    value = environment_object(raw, "environment control", {"targetHazardIds", "suppressionMs", "exposureReductionUnits"})
    targets = value["targetHazardIds"]
    if not isinstance(targets, list) or not 1 <= len(targets) <= 16 or any(not isinstance(target, str) or not re.fullmatch(r"[A-Za-z0-9_.:-]{1,128}", target) for target in targets) or len(set(targets)) != len(targets):
        raise MapError("invalid environment control targets")
    environment_integer(value["suppressionMs"], 100, 60_000, "suppressionMs")
    environment_integer(value["exposureReductionUnits"], 0, 10_000, "exposureReductionUnits")
    if interaction["kind"] != "environment_control" or interaction["event"] is not None or "rangeM" not in interaction:
        raise MapError("environment control requires explicit kind/range and no progression event")
    environment_integer(interaction.get("cooldownMs"), value["suppressionMs"], 60_000, "control cooldownMs")
    return value


LOGIC_LAYERS = {
    "logic.collision", "logic.navigation", "logic.spawn", "logic.interaction",
    "logic.door", "logic.trigger", "logic.hazard", "logic.checkpoint",
    "logic.camera", "logic.objective", "logic.transition", "logic.traversal",
    "logic.walkable", "logic.exploration", "logic.terrain",
    "logic.moving_support", "logic.standing_deck",
}
AUTHORING_ONLY_LAYERS = {"authoring.only"}
TERRAIN_TAGS = {"terrain.water_deep", "terrain.water_shallow", "conveyor"}
AUTHORING_ONLY_KINDS = {"pump_level_visual_marker", "shortcut_candidate"}
MIST_HARBOR_WATER_REGIONS = [
    ("mh_drowned_quay", "mh_water_depth_region", "terrain.water_shallow",
     [[13.0, 5.0], [19.0, 5.0], [19.0, 11.0], [13.0, 11.0]]),
    ("mh_pump_station", "mh_pump_east_channel_water", "terrain.water_deep",
     [[18.0, 5.5], [23.5, 5.5], [23.5, 10.5], [18.0, 10.5]]),
]
KNOWN_CAPABILITIES = {
    "information.local_map_i", "perception.rear_view_i",
    "information.enemy_vitals_basic", "body.regeneration_i",
    # Reserved by the v1 design; acquisition wiring is owned by the later scene runtime.
    "mobility.air_step_i",
}
TRAVERSAL_RANGE_M = 1.2
TRAVERSAL_COOLDOWN_MS = 350
PLAYER_RADIUS_M = 0.35
VISUAL_LAYERS = {
    "visual.background", "visual.floor", "visual.floor_detail", "visual.props_back",
    "visual.props_dynamic", "visual.foreground", "visual.occluders", "visual.vfx_markers",
}


def props(value: dict[str, Any]) -> dict[str, Any]:
    raw = value.get("properties", [])
    if isinstance(raw, dict):
        return raw
    if not isinstance(raw, list) or any(not isinstance(p, dict) or "name" not in p for p in raw):
        raise MapError("invalid Tiled properties")
    names = [p["name"] for p in raw]
    if len(names) != len(set(names)):
        raise MapError("duplicate Tiled property")
    return {p["name"]: p.get("value") for p in raw}


def required_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise MapError(f"missing {label}")
    return value.strip()


def finite(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise MapError(f"invalid {label}")
    return float(value)


def polygon_area(points: list[list[float]]) -> float:
    return abs(sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(points, points[1:] + points[:1]))) / 2


def polygon(obj: dict[str, Any], ppm: float, width: float, depth: float) -> list[list[float]]:
    x, z = finite(obj.get("x"), "polygon x"), finite(obj.get("y"), "polygon y")
    raw = obj.get("polygon")
    if raw is not None:
        if not isinstance(raw, list) or len(raw) < 3:
            raise MapError("invalid polygon vertices")
        points = [[round((x + finite(p.get("x"), "vertex x")) / ppm, 6),
                   round((z + finite(p.get("y"), "vertex y")) / ppm, 6)] for p in raw]
    else:
        w, h = finite(obj.get("width", 0), "rectangle width"), finite(obj.get("height", 0), "rectangle height")
        points = [[round(px / ppm, 6), round(pz / ppm, 6)] for px, pz in
                  ((x, z), (x + w, z), (x + w, z + h), (x, z + h))]
    if len({tuple(p) for p in points}) < 3 or polygon_area(points) <= 1e-9:
        raise MapError("zero-area polygon")
    if any(not (0 <= x <= width and 0 <= z <= depth) for x, z in points):
        raise MapError("polygon outside bounds")
    def crosses(a: list[float], b: list[float], c: list[float], d: list[float]) -> bool:
        def side(p: list[float], q: list[float], r: list[float]) -> float:
            return (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
        return side(a, b, c) * side(a, b, d) < 0 and side(c, d, a) * side(c, d, b) < 0
    count = len(points)
    for i in range(count):
        for j in range(i + 2, count):
            if i == 0 and j == count - 1:
                continue
            if crosses(points[i], points[(i + 1) % count], points[j], points[(j + 1) % count]):
                raise MapError("self-intersecting polygon")
    return points


def point_hits_polygon(x: float, z: float, radius: float, points: list[list[float]]) -> bool:
    inside = False
    for index, (ax, az) in enumerate(points):
        bx, bz = points[(index + 1) % len(points)]
        if (az > z) != (bz > z):
            crossing_x = ax + (z - az) * (bx - ax) / (bz - az)
            if x < crossing_x:
                inside = not inside
        dx, dz = bx - ax, bz - az
        denom = dx * dx + dz * dz
        t = 0.0 if denom == 0 else max(0.0, min(1.0, ((x - ax) * dx + (z - az) * dz) / denom))
        if (x - (ax + t * dx)) ** 2 + (z - (az + t * dz)) ** 2 <= radius * radius:
            return True
    return inside


def traversal_point_clear(x: float, z: float, radius: float, width: float, depth: float,
                          collisions: list[dict[str, Any]]) -> bool:
    return (
        radius <= x <= width - radius
        and radius <= z <= depth - radius
        and not any(point_hits_polygon(x, z, radius, wall["polygon"]) for wall in collisions)
    )


def support_polygon(points: list[list[float]]) -> None:
    if not 3 <= len(points) <= 128 or len({tuple(p) for p in points}) != len(points):
        raise MapError("invalid support polygon vertices")
    winding = 0.0
    for i, a in enumerate(points):
        b, c = points[(i + 1) % len(points)], points[(i + 2) % len(points)]
        cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0])
        if not math.isfinite(cross) or abs(cross) < 1e-6 or winding and cross * winding < 0:
            raise MapError("support polygon must be strictly convex")
        winding = 1.0 if cross > 0 else -1.0
        for j, point in enumerate(points):
            if j in (i, (i + 1) % len(points)):
                continue
            side = (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0])
            if not math.isfinite(side) or side * winding <= 1e-6:
                raise MapError("support polygon is concave or self-intersecting")


def full_foot_inside(position: list[float], points: list[list[float]]) -> bool:
    x, z = position[0], position[2]
    if not point_hits_polygon(x, z, 0.0, points):
        return False
    for i, a in enumerate(points):
        b = points[(i + 1) % len(points)]
        dx, dz = b[0] - a[0], b[1] - a[1]
        length = dx * dx + dz * dz
        t = max(0.0, min(1.0, ((x - a[0]) * dx + (z - a[1]) * dz) / length))
        if (x - a[0] - t * dx) ** 2 + (z - a[1] - t * dz) ** 2 < PLAYER_RADIUS_M ** 2:
            return False
    return True


def standing_point(position: list[float], decks: list[dict[str, Any]]) -> bool:
    return abs(position[1]) <= 0.001 or any(
        abs(position[1] - deck["heightM"]) <= 0.001 and full_foot_inside(position, deck["polygon"])
        for deck in decks
    )


def load_admitted_assets(runtime_path: Path, release_path: Path) -> set[str]:
    runtime = json.loads(runtime_path.read_text(encoding="utf-8"))
    release = json.loads(release_path.read_text(encoding="utf-8"))
    decisions = {row["asset_id"]: row for row in release["assets"]}
    atlases = runtime.get("atlases", [])
    for page in atlases:
        size = page.get("size", [])
        limit = 2048 if page.get("category") == "ui" else 4096
        if len(size) != 2 or any(not isinstance(n, int) or n <= 0 or n > limit for n in size):
            raise MapError(f"runtime atlas exceeds release budget: {page.get('pngPath')}")
    admitted: set[str] = set()
    for row in runtime["assets"]:
        asset_id = required_string(row.get("assetId"), "runtime assetId")
        decision = decisions.get(asset_id)
        if asset_id in admitted or not decision or decision["release_status"] != "release_approved":
            raise MapError(f"unapproved or duplicate runtime asset: {asset_id}")
        if row.get("admission") != "release_approved" or row.get("sourceSha256") != decision["sha256"]:
            raise MapError(f"runtime asset admission/SHA mismatch: {asset_id}")
        if row.get("sourcePath") != decision["relative_path"]:
            raise MapError(f"runtime asset source path mismatch: {asset_id}")
        page_index = row.get("atlasPage")
        if not isinstance(page_index, int) or not 0 <= page_index < len(atlases):
            raise MapError(f"runtime asset atlas missing: {asset_id}")
        outputs = row.get("outputs", {})
        if any(not isinstance(outputs.get(fmt, {}).get("sha256"), str) or len(outputs[fmt]["sha256"]) != 64 for fmt in ("png", "webp")):
            raise MapError(f"runtime output provenance missing: {asset_id}")
        admitted.add(asset_id)
    return admitted


def compile_map(
    source: dict[str, Any],
    admitted: set[str],
    entity_types: set[str] | None = None,
    required_events_by_world: dict[str, set[str]] | None = None,
) -> dict[str, Any]:
    meta = props(source)
    world_id = required_string(meta.get("worldId"), "worldId")
    scene_id = required_string(meta.get("sceneId"), "sceneId")
    ppm = finite(meta.get("pixelsPerMeter"), "pixelsPerMeter")
    if ppm <= 0:
        raise MapError("pixelsPerMeter must be positive")
    width = finite(source.get("width"), "map width") * finite(source.get("tilewidth"), "tile width") / ppm
    depth = finite(source.get("height"), "map height") * finite(source.get("tileheight"), "tile height") / ppm
    if width <= 0 or depth <= 0:
        raise MapError("invalid map bounds")
    result: dict[str, Any] = {
        "schemaVersion": 1, "worldId": world_id, "sceneId": scene_id,
        "boundsM": {"x": 0, "z": 0, "width": round(width, 6), "depth": round(depth, 6)},
        "presentation": {"cameraProfile": meta.get("cameraProfile", "oblique_default"), "layers": [], "sprites": []},
        "collision": [], "navigation": {"nodes": [], "links": []}, "spawns": [],
        "interactions": [], "doors": [], "triggers": [], "hazards": [], "checkpoints": [],
        "cameraZones": [], "objectives": [], "transitions": [], "occluders": [], "vfxMarkers": [],
        "logic": {"traversal": []},
    }
    seen_ids: set[str] = set()
    seen_layers: set[str] = set()
    height_ranges: dict[str, list[float]] = {}

    def asset_id(value: Any) -> str:
        ident = required_string(value, "assetId")
        if ident not in admitted:
            raise MapError(f"unknown or unapproved assetId: {ident}")
        return ident

    def required_event(value: Any, label: str) -> str:
        event = required_string(value, label)
        if required_events_by_world is not None and event not in required_events_by_world.get(world_id, set()):
            raise MapError(f"unknown required event for {world_id}: {event}")
        return event

    def point(obj: dict[str, Any]) -> list[float]:
        x = round(finite(obj.get("x"), "object x") / ppm, 6)
        z = round(finite(obj.get("y"), "object y") / ppm, 6)
        if not (0 <= x <= width and 0 <= z <= depth):
            raise MapError(f"object outside bounds: {obj.get('name')}")
        height = finite(props(obj).get("heightM", 0.0), "object heightM")
        if not 0.0 <= height <= 3.0:
            raise MapError("object heightM must remain in the 0..3m footpoint envelope")
        return [x, height, z]

    for layer in source.get("layers", []):
        name = required_string(layer.get("name"), "layer name")
        if name in seen_layers or name not in LOGIC_LAYERS | VISUAL_LAYERS | AUTHORING_ONLY_LAYERS:
            raise MapError(f"duplicate or unknown layer: {name}")
        seen_layers.add(name)
        if layer.get("type") != "objectgroup":
            raise MapError(f"unsupported layer type for {name}: {layer.get('type')}")
        if name.startswith("visual."):
            result["presentation"]["layers"].append({"id": name, "zGroup": len(result["presentation"]["layers"]) * 10})
        for obj in layer.get("objects", []):
            if finite(obj.get("rotation", 0), "object rotation") != 0:
                raise MapError(f"rotated Tiled object needs authored geometry: {obj.get('name')}")
            data = props(obj)
            ident = required_string(data.get("id", obj.get("name")), "object id")
            height_layers = {"logic.interaction", "logic.door", "logic.trigger", "logic.checkpoint", "logic.transition", "logic.traversal", "logic.hazard"}
            point_layers = {"logic.navigation", "logic.spawn", "logic.interaction", "logic.door", "logic.checkpoint", "logic.objective", "logic.traversal", "logic.standing_deck"} | (VISUAL_LAYERS - {"visual.occluders"})
            if "heightM" in data and name not in point_layers:
                raise MapError("heightM is not supported on this planar layer")
            if any(k in data for k in ("lowerM", "upperM", "travelMs")) and name != "logic.moving_support":
                raise MapError("moving-support timing/height belongs on logic.moving_support")
            if "endpointHoldMs" in data and name not in {"logic.moving_support", "logic.hazard"}:
                raise MapError("endpointHoldMs belongs on a moving support or machinery hazard")
            if "toHeightM" in data and name != "logic.traversal":
                raise MapError("toHeightM requires an authored traversal")
            if any(k in data for k in ("heightMinM", "heightMaxM")):
                if name not in height_layers or not all(k in data for k in ("heightMinM", "heightMaxM")):
                    raise MapError("height range needs both endpoints on an actionable layer")
                low, high = (finite(data[k], k) for k in ("heightMinM", "heightMaxM"))
                if not 0.0 <= low <= high <= 3.0:
                    raise MapError("height range must stay within 0..3m")
                height_ranges[ident] = [low, high]
            allowed_height = {"heightM", "heightMinM", "heightMaxM", "toHeightM"}
            if any(k.startswith("height") or k.startswith("toHeight") for k in set(data) - allowed_height):
                raise MapError("unknown vertical authoring property")
            if ("environment" in data and name != "logic.hazard") or ("environmentControl" in data and name != "logic.interaction"):
                raise MapError("environment configuration on wrong layer")
            if ident in seen_ids:
                raise MapError(f"duplicate object id: {ident}")
            seen_ids.add(ident)
            position = point(obj)
            if name in {"logic.moving_support", "logic.standing_deck"}:
                required = {"heightM"} if name == "logic.standing_deck" else {"lowerM", "upperM", "travelMs", "endpointHoldMs"}
                if set(data) - {"id"} != required:
                    raise MapError("support properties must contain exactly the documented geometry/timing fields and optional id")
                shape = polygon(obj, ppm, width, depth)
                support_polygon(shape)
                if name == "logic.standing_deck":
                    result.setdefault("standingDecks", []).append({"id": ident, "polygon": shape, "heightM": finite(data["heightM"], "deck heightM")})
                else:
                    low, high = finite(data["lowerM"], "support lowerM"), finite(data["upperM"], "support upperM")
                    if not 0.0 <= low < high <= 3.0 or high - low < 0.1:
                        raise MapError("moving support height span must be at least 0.1m within 0..3m")
                    environment_integer(data["travelMs"], 250, 60_000, "support travelMs")
                    environment_integer(data["endpointHoldMs"], 0, 30_000, "support endpointHoldMs")
                    result.setdefault("movingSupports", []).append({"id": ident, "polygon": shape, "lowerM": low, "upperM": high,
                        "travelMs": data["travelMs"], "endpointHoldMs": data["endpointHoldMs"]})
            elif name == "logic.collision":
                collision = {"id": ident, "polygon": polygon(obj, ppm, width, depth)}
                if data.get("requiresEvent") is not None:
                    collision["requiresEvent"] = required_event(data.get("requiresEvent"), "collision requiresEvent")
                if "requiresActorFirstKill" in data:
                    actor_id = required_string(data.get("requiresActorFirstKill"), "collision requiresActorFirstKill")
                    if "requiresEvent" in collision:
                        raise MapError(f"collision cannot combine progression and actor kill requirements: {ident}")
                    collision["requiresActorFirstKill"] = actor_id
                result["collision"].append(collision)
            elif name == "logic.navigation":
                neighbors = data.get("neighbors", "")
                if not isinstance(neighbors, str):
                    raise MapError(f"invalid nav neighbors: {ident}")
                result["navigation"]["nodes"].append({"id": ident, "position": position})
                for neighbor in [n.strip() for n in neighbors.split(",") if n.strip()]:
                    result["navigation"]["links"].append({"from": ident, "to": neighbor})
            elif name == "logic.spawn":
                kind = required_string(data.get("kind"), "spawn kind")
                entity_type = data.get("entityType")
                if kind in {"enemy", "npc"}:
                    entity_type = required_string(entity_type, "entityType")
                    if entity_types is None or entity_type not in entity_types:
                        raise MapError(f"unknown entityType: {entity_type}")
                result["spawns"].append({"id": ident, "kind": kind, "entityType": entity_type, "position": position, "navNode": data.get("navNode")})
            elif name == "logic.interaction":
                event = data.get("event")
                if event is not None:
                    event = required_event(event, "interaction event")
                kind = required_string(data.get("kind"), "interaction kind")
                aggregate_members = data.get("aggregateMembers")
                aggregate_event = data.get("aggregateEvent")
                if aggregate_members is not None or aggregate_event is not None:
                    if aggregate_members is None or aggregate_event is None or event is not None or not kind.endswith("_marker"):
                        raise MapError(f"invalid interaction aggregate marker: {ident}")
                    if not isinstance(aggregate_members, str):
                        raise MapError(f"invalid interaction aggregate members: {ident}")
                    member_ids = [member.strip() for member in aggregate_members.split(",")]
                    if not member_ids or any(not member for member in member_ids) or len(set(member_ids)) != len(member_ids):
                        raise MapError(f"invalid interaction aggregate members: {ident}")
                    result.setdefault("interactionAggregates", []).append({
                        "markerId": ident,
                        "memberIds": member_ids,
                        "event": required_event(aggregate_event, "interaction aggregate event"),
                    })
                interaction = {"id": ident, "kind": kind, "event": event, "position": position}
                if "rangeM" in data:
                    interaction["rangeM"] = finite(data["rangeM"], f"interaction rangeM: {ident}")
                    if interaction["rangeM"] <= 0 or interaction["rangeM"] > 2.5:
                        raise MapError(f"invalid interaction rangeM: {ident}")
                if "cooldownMs" in data:
                    cooldown = data["cooldownMs"]
                    if isinstance(cooldown, bool) or not isinstance(cooldown, int) or cooldown < 0 or cooldown > 3_600_000:
                        raise MapError(f"invalid interaction cooldownMs: {ident}")
                    interaction["cooldownMs"] = cooldown
                if "environmentControl" in data:
                    interaction["environmentControl"] = environment_control(data["environmentControl"], interaction)
                elif kind == "environment_control":
                    raise MapError(f"environment_control needs configuration: {ident}")
                result["interactions"].append(interaction)
            elif name == "logic.door":
                to_scene_id, spawn_id = data.get("toSceneId"), data.get("spawnId")
                if (to_scene_id is None) != (spawn_id is None):
                    raise MapError(f"door transition needs both target scene and spawn: {ident}")
                door = {"id": ident, "position": position, "assetId": asset_id(data.get("assetId"))}
                if to_scene_id is not None:
                    door["toSceneId"] = required_string(to_scene_id, "door target")
                    door["spawnId"] = required_string(spawn_id, "door spawn")
                if data.get("requiresEvent") is not None:
                    door["requiresEvent"] = required_event(data.get("requiresEvent"), "door requiresEvent")
                if "toSceneId" not in door and "requiresEvent" not in door:
                    raise MapError(f"non-transition door requires a progression event: {ident}")
                result["doors"].append(door)
            elif name == "logic.trigger":
                result["triggers"].append({"id": ident, "event": required_event(data.get("event"), "trigger event"), "polygon": polygon(obj, ppm, width, depth)})
            elif name == "logic.hazard":
                shape = polygon(obj, ppm, width, depth)
                hazard = {"id": ident, "kind": required_string(data.get("kind"), "hazard kind"), "polygon": shape}
                timing_fields = {"damage", "periodMs", "warningMs", "phase"}
                motion_fields = {"translationXM", "translationZM", "speedMps", "endpointHoldMs"}
                moving = hazard["kind"] == "moving_machinery"
                if "environment" in data:
                    if any(key in data for key in timing_fields | motion_fields):
                        raise MapError(f"ambiguous environment and encounter hazard tuning: {ident}")
                    hazard["environment"] = environment_hazard(data["environment"], hazard["kind"])
                if moving or any(key in data for key in timing_fields | motion_fields):
                    label = "moving machinery" if moving else "heat hazard"
                    expected = timing_fields | motion_fields if moving else timing_fields
                    if not expected.issubset(data) or hazard["kind"] not in {"heat_zone", "moving_machinery"}:
                        raise MapError(f"incomplete {label} tuning: {ident}")
                    if not moving and any(key in data for key in motion_fields):
                        raise MapError(f"motion requires moving_machinery: {ident}")
                    for key, low, high in (("damage", 1, 10_000), ("periodMs", 100, 60_000), ("warningMs", 1, 10_000)):
                        value = data[key]
                        if isinstance(value, bool) or not isinstance(value, int) or not low <= value <= high:
                            raise MapError(f"invalid {label} {key}: {ident}")
                    phase = required_string(data["phase"], f"{label} phase")
                    if phase != ("phase3" if moving else "phase2"):
                        raise MapError(f"invalid {label} phase: {ident}")
                    hazard.update({"damage": data["damage"], "periodMs": data["periodMs"],
                                   "warningMs": data["warningMs"], "phase": phase})
                    if moving:
                        translation = [finite(data[key], f"moving machinery {key}")
                                       for key in ("translationXM", "translationZM")]
                        distance = math.hypot(*translation)
                        if not math.isfinite(distance) or distance < 0.01:
                            raise MapError(f"invalid moving machinery translation: {ident}")
                        speed = finite(data["speedMps"], "moving machinery speedMps")
                        if not 0.1 <= speed <= 20.0:
                            raise MapError(f"invalid moving machinery speedMps: {ident}")
                        hold = data["endpointHoldMs"]
                        if isinstance(hold, bool) or not isinstance(hold, int) or not 100 <= hold <= 10_000:
                            raise MapError(f"invalid moving machinery endpointHoldMs: {ident}")
                        # Linear translation in a rectangular map stays in bounds iff
                        # every vertex at both endpoints is in bounds.
                        if any(not (0 <= x + translation[0] <= width
                                    and 0 <= z + translation[1] <= depth) for x, z in shape):
                            raise MapError(f"moving machinery path outside bounds: {ident}")
                        hazard.update({"translationM": translation, "speedMps": speed,
                                       "endpointHoldMs": hold})
                terrain_tag = data.get("terrainTag")
                if terrain_tag is not None:
                    if not isinstance(terrain_tag, str) or terrain_tag not in TERRAIN_TAGS:
                        raise MapError(f"unknown terrain tag: {terrain_tag}")
                    if terrain_tag == "conveyor":
                        raise MapError("conveyor motion requires a dedicated logic.terrain region")
                    hazard["terrainTag"] = terrain_tag
                    result.setdefault("terrainRegions", []).append({"id": ident, "tag": terrain_tag, "polygon": shape})
                result["hazards"].append(hazard)
            elif name == "logic.checkpoint":
                result["checkpoints"].append({"id": ident, "position": position, "navNode": required_string(data.get("navNode"), "checkpoint navNode")})
            elif name == "logic.camera":
                result["cameraZones"].append({"id": ident, "polygon": polygon(obj, ppm, width, depth), "profile": data.get("profile", "oblique_default")})
            elif name == "logic.objective":
                result["objectives"].append({"id": ident, "completingEvent": required_event(data.get("completingEvent"), "objective completingEvent"), "position": position})
            elif name == "logic.transition":
                transition = {"id": ident, "toSceneId": required_string(data.get("toSceneId"), "transition target"), "spawnId": required_string(data.get("spawnId"), "transition spawn"), "polygon": polygon(obj, ppm, width, depth)}
                if data.get("requiresEvent") is not None:
                    transition["requiresEvent"] = required_event(data.get("requiresEvent"), "transition requiresEvent")
                result["transitions"].append(transition)
            elif name == "logic.traversal":
                to_x = finite(data.get("toX"), f"traversal toX: {ident}") / ppm
                to_z = finite(data.get("toY"), f"traversal toY: {ident}") / ppm
                range_m = finite(data.get("rangeM"), f"traversal rangeM: {ident}")
                cooldown_ms = finite(data.get("cooldownMs"), f"traversal cooldownMs: {ident}")
                if not math.isclose(range_m, TRAVERSAL_RANGE_M, abs_tol=1e-6):
                    raise MapError(f"traversal range must be {TRAVERSAL_RANGE_M} m: {ident}")
                if cooldown_ms != TRAVERSAL_COOLDOWN_MS:
                    raise MapError(f"traversal cooldown must be {TRAVERSAL_COOLDOWN_MS} ms: {ident}")
                required = data.get("requiresCapabilities", "")
                if isinstance(required, str):
                    required = [item.strip() for item in required.split(",") if item.strip()]
                if not isinstance(required, list) or any(not isinstance(item, str) or not item.strip() for item in required):
                    raise MapError(f"invalid traversal capabilities: {ident}")
                required = [item.strip() for item in required]
                if len(required) != len(set(required)):
                    raise MapError(f"duplicate traversal capability: {ident}")
                unknown = set(required) - KNOWN_CAPABILITIES
                if unknown:
                    raise MapError(f"unknown traversal capability: {sorted(unknown)}")
                result["logic"]["traversal"].append({
                    "id": ident,
                    "from": position,
                    "to": [round(to_x, 6), finite(data.get("toHeightM", 0.0), "traversal toHeightM"), round(to_z, 6)],
                    "rangeM": range_m,
                    "cooldownMs": int(cooldown_ms),
                    "requiredCapabilities": sorted(required),
                })
            elif name == "logic.walkable":
                result.setdefault("walkablePolygons", []).append({"id": ident, "polygon": polygon(obj, ppm, width, depth)})
            elif name == "logic.exploration":
                result.setdefault("explorationRegions", []).append({"id": ident, "polygon": polygon(obj, ppm, width, depth)})
            elif name == "logic.terrain":
                terrain_tag = data.get("tag")
                if not isinstance(terrain_tag, str) or terrain_tag not in TERRAIN_TAGS:
                    raise MapError(f"unknown terrain tag: {terrain_tag}")
                region = {"id": ident, "tag": terrain_tag, "polygon": polygon(obj, ppm, width, depth)}
                motion_keys = ("surfaceVelocityXMps", "surfaceVelocityZMps")
                if terrain_tag == "conveyor":
                    velocity = [finite(data.get(key), f"conveyor {key}: {ident}") for key in motion_keys]
                    if not 0.1 <= math.hypot(*velocity) <= 4.0:
                        raise MapError(f"conveyor speed must be 0.1..4 m/s: {ident}")
                    region["surfaceVelocityMps"] = velocity
                elif any(key in data for key in motion_keys):
                    raise MapError(f"surface velocity requires conveyor terrain: {ident}")
                result.setdefault("terrainRegions", []).append(region)
            elif name in AUTHORING_ONLY_LAYERS:
                kind = data.get("kind")
                if not isinstance(kind, str) or kind not in AUTHORING_ONLY_KINDS:
                    raise MapError(f"unsupported authoring-only kind: {kind}")
            elif name == "visual.occluders":
                result["occluders"].append({"id": ident, "polygon": polygon(obj, ppm, width, depth), "fadeTo": finite(data.get("fadeTo", 0.25), "occluder fade")})
            elif name == "visual.vfx_markers":
                result["vfxMarkers"].append({"id": ident, "assetId": asset_id(data.get("assetId")), "position": position})
            elif name.startswith("visual."):
                result["presentation"]["sprites"].append({"id": ident, "layer": name, "assetId": asset_id(data.get("assetId")), "position": position})

    actor_first_kill_ids = [
        collision["requiresActorFirstKill"]
        for collision in result["collision"]
        if "requiresActorFirstKill" in collision
    ]
    for actor_id in actor_first_kill_ids:
        actor = next((spawn for spawn in result["spawns"] if spawn["id"] == actor_id), None)
        if actor is None or actor["kind"] != "enemy":
            raise MapError(f"collision requiresActorFirstKill must reference a same-scene enemy spawn: {actor_id}")
    if sum(spawn["kind"] == "player" for spawn in result["spawns"]) != 1:
        raise MapError(f"scene {scene_id} requires exactly one player spawn")
    nodes = {n["id"] for n in result["navigation"]["nodes"]}
    if not nodes:
        raise MapError(f"scene {scene_id} has no navigation nodes")
    links = result["navigation"]["links"]
    if any(link["to"] not in nodes for link in links):
        raise MapError(f"unknown nav link in {scene_id}")
    start = next(spawn for spawn in result["spawns"] if spawn["kind"] == "player")["navNode"]
    if start not in nodes:
        raise MapError(f"player spawn has unknown navNode in {scene_id}")
    graph = {node: set() for node in nodes}
    for link in links:
        graph[link["from"]].add(link["to"])
        graph[link["to"]].add(link["from"])
    reachable = {start}
    queue = deque([start])
    while queue:
        for neighbor in graph[queue.popleft()]:
            if neighbor not in reachable:
                reachable.add(neighbor)
                queue.append(neighbor)
    if any(checkpoint["navNode"] not in reachable for checkpoint in result["checkpoints"]):
        raise MapError(f"unreachable checkpoint in {scene_id}")
    interaction_by_id = {entry["id"]: entry for entry in result["interactions"]}
    aggregate_members: set[str] = set()
    for aggregate in result.get("interactionAggregates", []):
        marker = interaction_by_id.get(aggregate["markerId"])
        if marker is None or not marker["kind"].endswith("_marker") or marker["event"] is not None:
            raise MapError(f"invalid interaction aggregate marker: {aggregate['markerId']}")
        for member_id in aggregate["memberIds"]:
            member = interaction_by_id.get(member_id)
            if member is None or member_id == aggregate["markerId"] or member["event"] is not None:
                raise MapError(f"invalid interaction aggregate member: {aggregate['markerId']} -> {member_id}")
            if member_id in aggregate_members:
                raise MapError(f"interaction is assigned to multiple aggregates: {member_id}")
            aggregate_members.add(member_id)
    environment_hazards = {h["id"] for h in result["hazards"] if "environment" in h}
    aggregate_markers = {a["markerId"] for a in result.get("interactionAggregates", [])}
    for interaction in result["interactions"]:
        control = interaction.get("environmentControl")
        if control and (interaction["id"] in aggregate_members | aggregate_markers or any(target not in environment_hazards for target in control["targetHazardIds"])):
            raise MapError(f"environment control has progression membership or unknown targets: {interaction['id']}")
    events = {entry["event"] for key in ("interactions", "triggers") for entry in result[key] if entry.get("event")}
    events.update(aggregate["event"] for aggregate in result.get("interactionAggregates", []))
    if any(objective["completingEvent"] not in events for objective in result["objectives"]):
        raise MapError(f"objective has no completing trigger in {scene_id}")
    decks = result.get("standingDecks", [])
    if len(decks) + len(result.get("movingSupports", [])) > 64:
        raise MapError("at most 64 total supports are permitted")
    for key in ("interactions", "doors", "triggers", "checkpoints", "transitions", "hazards"):
        for item in result[key]:
            if item["id"] in height_ranges:
                item["heightRangeM"] = height_ranges[item["id"]]
            elif decks:
                raise MapError(f"standing-deck scene requires explicit height range: {item['id']}")
            if "position" in item:
                position = item["position"]
                if "heightRangeM" in item and not item["heightRangeM"][0] <= position[1] <= item["heightRangeM"][1]:
                    raise MapError("actionable anchor is outside its height range")
                if position[1] > 0.001 and (not standing_point(position, decks) or not traversal_point_clear(position[0], position[2], PLAYER_RADIUS_M, width, depth, result["collision"])):
                    raise MapError("raised actionable anchor lacks stable standing support")
    for item in result["spawns"] + result["navigation"]["nodes"]:
        position = item["position"]
        if position[1] > 0.001 and (not standing_point(position, decks) or not traversal_point_clear(position[0], position[2], PLAYER_RADIUS_M, width, depth, result["collision"])):
            raise MapError("raised spawn/navigation point lacks stable standing support")
    for marker in result["logic"]["traversal"]:
        if marker["id"] in height_ranges:
            marker["fromHeightRangeM"] = height_ranges[marker["id"]]
            if not marker["fromHeightRangeM"][0] <= marker["from"][1] <= marker["fromHeightRangeM"][1]:
                raise MapError("traversal start outside authored height range")
        elif decks:
            raise MapError("standing-deck traversal requires an explicit start height range")
        for label, position in (("from", marker["from"]), ("to", marker["to"])):
            x, z = position[0], position[2]
            if not 0.0 <= position[1] <= 3.0 or not standing_point(position, decks):
                raise MapError(f"traversal {label} lacks stable endpoint support: {marker['id']}")
            if not traversal_point_clear(x, z, PLAYER_RADIUS_M, width, depth, result["collision"]):
                raise MapError(f"traversal {label} is out of bounds or collision-blocked: {marker['id']}")
        start, end = marker["from"], marker["to"]
        distance = math.hypot(end[0] - start[0], end[2] - start[2])
        samples = max(1, math.ceil(distance / 0.1))
        if any(not traversal_point_clear(
            start[0] + (end[0] - start[0]) * step / samples,
            start[2] + (end[2] - start[2]) * step / samples,
            PLAYER_RADIUS_M, width, depth, result["collision"]
        ) for step in range(samples + 1)):
            raise MapError(f"traversal path intersects collision: {marker['id']}")
    return result


def validate_campaign(
    scenes: list[dict[str, Any]],
    progression: dict[str, Any] | None,
    allow_incomplete_world_slice: bool = False,
    allow_partial_campaign: bool = False,
) -> None:
    if allow_incomplete_world_slice and allow_partial_campaign:
        raise MapError("world-slice and partial-campaign modes are mutually exclusive")
    by_id = {scene["sceneId"]: scene for scene in scenes}
    if len(by_id) != len(scenes):
        raise MapError("duplicate sceneId")
    for scene in scenes:
        targets = scene["transitions"] + [door for door in scene["doors"] if door.get("toSceneId")]
        for entry in targets:
            target = by_id.get(entry["toSceneId"])
            if target is None or target["worldId"] != scene["worldId"]:
                raise MapError(f"unknown transition target: {entry['toSceneId']}")
            if entry["spawnId"] not in {spawn["id"] for spawn in target["spawns"]}:
                raise MapError(f"unknown transition spawn: {entry['spawnId']}")
    mist_scenes = [scene for scene in scenes if scene["worldId"] == "mist_harbor"]
    if mist_scenes:
        included_scene_ids = {scene["sceneId"] for scene in mist_scenes}
        expected = sorted(region for region in MIST_HARBOR_WATER_REGIONS if region[0] in included_scene_ids)
        actual = sorted(
            (scene["sceneId"], region["id"], region["tag"], region["polygon"])
            for scene in mist_scenes for region in scene.get("terrainRegions", [])
        )
        if actual != expected:
            raise MapError("Mist Harbor terrain regions do not match the frozen two-region contract")
    if progression:
        catalog = {world["worldId"]: set(world["requiredEvents"]) for world in progression["worlds"]}
        if allow_partial_campaign:
            scene_worlds = {scene["worldId"] for scene in scenes}
            if "return_station" not in scene_worlds or not (scene_worlds & catalog.keys()):
                raise MapError("partial-campaign mode requires return_station and at least one catalog world")
            unsupported = scene_worlds - set(catalog) - {"return_station"}
            if unsupported:
                raise MapError(f"partial-campaign mode contains unsupported worlds: {sorted(unsupported)}")
            worlds = [world for world in progression["worlds"] if world["worldId"] in scene_worlds]
        elif allow_incomplete_world_slice:
            scene_worlds = {scene["worldId"] for scene in scenes}
            if len(scene_worlds) != 1 or not scene_worlds.issubset(catalog):
                raise MapError("incomplete world-slice mode requires scenes from exactly one catalog world")
            worlds = [world for world in progression["worlds"] if world["worldId"] in scene_worlds]
        else:
            worlds = progression["worlds"]
        for world in worlds:
            world_scenes = [scene for scene in scenes if scene["worldId"] == world["worldId"]]
            if not world_scenes:
                raise MapError(f"missing world scenes: {world['worldId']}")
            emitted = {entry["event"] for scene in world_scenes for key in ("interactions", "triggers") for entry in scene[key] if entry.get("event")}
            emitted.update(aggregate["event"] for scene in world_scenes for aggregate in scene.get("interactionAggregates", []))
            missing = set(world["requiredEvents"]) - emitted
            if missing and not allow_incomplete_world_slice and not allow_partial_campaign:
                raise MapError(f"required world events never emitted: {world['worldId']} {sorted(missing)}")
            if allow_incomplete_world_slice and not (set(world["requiredEvents"]) & emitted):
                raise MapError(f"world slice emits no canonical progression event: {world['worldId']}")
            if allow_partial_campaign and not (set(world["requiredEvents"]) & emitted):
                raise MapError(f"partial campaign emits no canonical progression event: {world['worldId']}")


def compile_directory(
    maps: Path,
    runtime: Path,
    release: Path,
    out: Path | None,
    progression: Path | None,
    entity_catalog: Path | None = None,
    allow_incomplete_world_slice: bool = False,
    allow_partial_campaign: bool = False,
) -> list[dict[str, Any]]:
    admitted = load_admitted_assets(runtime, release)
    files = sorted(maps.rglob("*.tmj"))
    if not files:
        raise MapError("no TMJ maps found")
    entity_types = set(json.loads(entity_catalog.read_text(encoding="utf-8"))["entityTypes"]) if entity_catalog else None
    progression_data = json.loads(progression.read_text(encoding="utf-8")) if progression else None
    if allow_incomplete_world_slice and progression_data is None:
        raise MapError("world-slice validation requires the progression catalog")
    if allow_partial_campaign and progression_data is None:
        raise MapError("partial-campaign validation requires the progression catalog")
    required = ({world["worldId"]: set(world["requiredEvents"]) for world in progression_data["worlds"]}
                if progression_data else None)
    scenes = [compile_map(json.loads(file.read_text(encoding="utf-8")), admitted, entity_types, required) for file in files]
    validate_campaign(scenes, progression_data, allow_incomplete_world_slice, allow_partial_campaign)
    if out is not None:
        out.mkdir(parents=True, exist_ok=True)
        for scene in scenes:
            output_path = out / f"{scene['sceneId']}.json"
            with output_path.open("w", encoding="utf-8", newline="\n") as output_file:
                output_file.write(json.dumps(scene, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
    return scenes


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--maps", type=Path, required=True)
    parser.add_argument("--runtime-manifest", type=Path, required=True)
    parser.add_argument("--release-manifest", type=Path, required=True)
    parser.add_argument("--progression", type=Path)
    parser.add_argument("--entity-catalog", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--validate-all", action="store_true")
    parser.add_argument("--allow-incomplete-world-slice", action="store_true",
                        help="validate one progression-catalog world slice without requiring its un-authored events")
    parser.add_argument("--allow-partial-campaign", action="store_true",
                        help="validate Return Station plus catalog worlds without requiring every campaign world/event")
    args = parser.parse_args()
    if args.allow_incomplete_world_slice and args.allow_partial_campaign:
        parser.error("--allow-incomplete-world-slice and --allow-partial-campaign cannot be combined")
    if not args.validate_all and args.out is None:
        parser.error("--out is required unless --validate-all is set")
    try:
        scenes = compile_directory(args.maps, args.runtime_manifest, args.release_manifest, None if args.validate_all else args.out, args.progression, args.entity_catalog, args.allow_incomplete_world_slice, args.allow_partial_campaign)
    except (MapError, KeyError, ValueError, OSError) as exc:
        parser.exit(1, f"world-map-compiler: {exc}\n")
    print(json.dumps({"scenes": len(scenes), "worlds": sorted({scene["worldId"] for scene in scenes}), "result": "PASS"}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
