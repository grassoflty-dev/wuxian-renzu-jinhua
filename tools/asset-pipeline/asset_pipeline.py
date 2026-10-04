#!/usr/bin/env python3
"""Build deterministic runtime atlases from the canonical approved asset set.

This tool deliberately reads image bytes only after the B0 release manifest has
selected a row as ``release_approved`` and the pinned Git blob hash is verified.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path, PurePosixPath
from typing import Any

from PIL import Image, __version__ as PILLOW_VERSION, features

REPO = Path(__file__).resolve().parents[2]
MANIFEST = REPO / "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"
SOURCE_COMMIT = "030eb9bf8194ad5534dc77315567ff339cd2c625"
SOURCE_PREFIX = "chatgptimage/wuxian-renzu-jinhua_2d_asset_pack_10_batches_2026-09-25/"
TOOL_VERSION = "asset-pipeline-core/1.0.0"
EXPECTED_WEBP_ENCODER = "1.6.0"
OUT = Path("assets/derived/asset-pipeline-v1")
CENYAO_IDLE_METADATA = Path("assets/metadata/cenyao-runtime-master-v1/cenyao-idle-atlas-v1.json")
CENYAO_IDLE_METADATA_SHA256 = "a1d5f40c2a7ae47f45dbf2b85432291be3be0badb855f1f386ec9905d533641c"
CENYAO_MASTER_SHA256 = "80a810ec6fdbcda50a0daeebcf53cd2173c0a411f043861e869187061cb59f61"
CENYAO_IDLE_DIRECTIONS = ("south", "south_west", "west", "north_east", "north", "north_west", "east", "south_east")
ATLAS_BUDGETS = {"actor": 4096, "prop": 4096, "vfx": 4096, "ui": 2048, "world": 4096, "boss": 4096}


class PipelineError(RuntimeError):
    pass


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8")


def load_authority(path: Path = MANIFEST) -> dict[str, Any]:
    gate_dir = REPO / "tools/asset-release-gate"
    sys.path.insert(0, str(gate_dir))
    try:
        import asset_release_gate  # type: ignore
        return asset_release_gate.load_manifest(path)
    except Exception as exc:
        raise PipelineError(f"B0 manifest gate failed: {exc}") from exc
    finally:
        sys.path.pop(0)


def pinned_blob(path: str) -> bytes:
    if "\\" in path or path.startswith("/") or ".." in PurePosixPath(path).parts:
        raise PipelineError(f"unsafe source path: {path}")
    sys.path.insert(0, str(REPO / "tools/asset-release-gate"))
    try:
        from source_provenance import read_approved_source, ProvenanceError
        return read_approved_source(path, REPO)
    except (ProvenanceError, OSError) as exc:
        raise PipelineError(f"pinned source missing or invalid: {path}: {exc}") from exc
    finally:
        sys.path.pop(0)


def admitted_sources(manifest: dict[str, Any]) -> list[tuple[dict[str, Any], bytes]]:
    selected: list[tuple[dict[str, Any], bytes]] = []
    # Filter status before resolving a source Git object. Rejected/concept rows
    # are never sent to the provenance reader, Pillow, or the filesystem.
    for row in manifest["assets"]:
        if row["release_status"] != "release_approved":
            continue
        blob = pinned_blob(row["relative_path"])
        actual = sha256(blob)
        if actual != row["sha256"]:
            raise PipelineError(
                f"pinned SHA mismatch for {row['asset_id']}: expected {row['sha256']}, got {actual}"
            )
        selected.append((row, blob))
    if len(selected) != 43:
        raise PipelineError(f"expected 43 approved sources, found {len(selected)}")
    return selected


def clean_crop(blob: bytes) -> tuple[Image.Image, dict[str, Any]]:
    with Image.open(io.BytesIO(blob)) as opened:
        image = opened.convert("RGBA")
    alpha = image.getchannel("A")
    bbox = alpha.getbbox()
    crop = image.crop(bbox) if bbox else Image.new("RGBA", (1, 1), (0, 0, 0, 0))
    # Transparent RGB is normalized to zero so PNG output hashes are stable and
    # transparent fringe colors cannot leak into filtering.
    alpha = crop.getchannel("A")
    transparent = alpha.point(lambda value: 255 if value == 0 else 0)
    rgb = crop.convert("RGB")
    rgb.paste((0, 0, 0), mask=transparent)
    crop = Image.merge("RGBA", (*rgb.split(), alpha))
    return crop, {
        "sourceSize": list(image.size),
        "cropRect": list(bbox) if bbox else [0, 0, 0, 0],
        "outputSize": list(crop.size),
        "alphaCleanup": "crop-alpha-bounds; zero-rgb-where-alpha-zero",
    }


def anchor_for(asset_id: str) -> list[float]:
    lower = asset_id.lower()
    if any(token in lower for token in ("floor", "wall", "tiles", "water_edge", "fog", "vfx", "portrait", "map")):
        return [0.5, 0.5]
    return [0.5, 1.0]


def category_and_group(asset_id: str) -> tuple[str, str]:
    if asset_id.startswith("runtime2d.enemy.sentinel."):
        return "boss", "sentinel"
    if asset_id.startswith("runtime2d.actor."):
        return "actor", "shared"
    if asset_id.startswith("runtime2d.enemy."):
        if "mistharbor." in asset_id:
            return "actor", "mistharbor"
        if "clockworks." in asset_id:
            return "actor", "clockworks"
        return "actor", "grey_hive"
    if asset_id.startswith("runtime2d.prop."):
        return "prop", "grey_hive"
    if asset_id.startswith("runtime2d.vfx.") or ".fog_vfx." in asset_id:
        return "vfx", "shared"
    if asset_id.startswith("portrait.") or asset_id.startswith("runtime2d.ui."):
        return "ui", "shared"
    if asset_id.startswith("runtime2d.world.grey_hive."):
        return "world", "grey_hive"
    if asset_id.startswith("runtime2d.grey_hive."):
        return "world", "grey_hive"
    if asset_id.startswith("runtime2d.world.mistharbor."):
        return "world", "mistharbor"
    if asset_id.startswith("runtime2d.world.clockworks."):
        return "world", "clockworks"
    if asset_id.startswith("runtime2d.world.returnstation."):
        return "world", "returnstation"
    raise PipelineError(f"no budget category is defined for approved asset: {asset_id}")


def budget_exceeded(category: str, width: int, height: int) -> bool:
    limit = ATLAS_BUDGETS[category]
    if category == "world":
        return max(width, height) > limit
    return width > limit or height > limit


def load_cenyao_idle_metadata(dest: Path, approved: dict[str, Any]) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    metadata_path = dest / CENYAO_IDLE_METADATA
    metadata_bytes = metadata_path.read_bytes()
    metadata_sha = sha256(metadata_bytes)
    if metadata_sha != CENYAO_IDLE_METADATA_SHA256:
        raise PipelineError(f"B6a metadata SHA mismatch: expected {CENYAO_IDLE_METADATA_SHA256}, found {metadata_sha}")
    metadata = json.loads(metadata_bytes)
    source_row = approved.get("runtime2d.actor.cenyao.base.v1")
    if (metadata.get("schemaVersion") != "cenyao-idle-atlas-v1" or
            metadata.get("sourceMaster") != {"path": "assets/metadata/cenyao-runtime-master-v1/cenyao-runtime-master-v1.json", "sha256": CENYAO_MASTER_SHA256} or
            not source_row or metadata.get("approvedSource") != {
                "approved": True, "assetId": source_row["asset_id"], "path": f"assets/source/approved/{source_row['relative_path']}",
                "sha256": source_row["sha256"], "size": [1448, 1086]
            }):
        raise PipelineError("B6a source-master or B0-approved source provenance mismatch")
    atlas = metadata.get("atlas")
    if not isinstance(atlas, dict) or atlas.get("size") != [1360, 1128] or atlas.get("maxSide") != 4096 or atlas.get("extrusionPixels") != 2:
        raise PipelineError("B6a atlas size or extrusion metadata mismatch")
    for format_name in ("png", "webp"):
        output = atlas.get(format_name, {})
        path = output.get("path", "")
        if not path.startswith("assets/derived/cenyao-runtime-master-v1/atlas/") or ".." in PurePosixPath(path).parts:
            raise PipelineError(f"unsafe B6a atlas output path: {path}")
        if sha256((dest / path).read_bytes()) != output.get("sha256"):
            raise PipelineError(f"B6a atlas output SHA mismatch: {path}")
    frames = metadata.get("frames")
    if not isinstance(frames, list) or [frame.get("direction") for frame in frames] != list(CENYAO_IDLE_DIRECTIONS):
        raise PipelineError("B6a idle direction order is incomplete or changed")
    for frame in frames:
        rect = frame.get("rect")
        extruded = frame.get("extrudedRect")
        anchor = frame.get("anchor", {})
        if rect is None or rect[2:] != [336, 560] or extruded is None or extruded[2:] != [340, 564] or \
                extruded != [rect[0] - 2, rect[1] - 2, rect[2] + 4, rect[3] + 4] or \
                anchor.get("x") != 0.5 or anchor.get("y") != 0.98392857:
            raise PipelineError(f"B6a frame rectangle/anchor/extrusion mismatch: {frame.get('direction')}")
        for format_name in ("png", "webp"):
            source = frame.get("sourceFrame", {}).get(format_name, {})
            path = source.get("path", "")
            if not path.startswith(f"assets/derived/cenyao-runtime-master-v1/{format_name}/") or ".." in PurePosixPath(path).parts:
                raise PipelineError(f"unsafe B6a source frame path: {path}")
            if sha256((dest / path).read_bytes()) != source.get("sha256"):
                raise PipelineError(f"B6a source frame SHA mismatch: {path}")
    return metadata, frames


def pack(
    images: list[tuple[str, Image.Image]], padding: int = 2, max_side: int = 4096,
) -> tuple[list[Image.Image], dict[str, dict[str, Any]]]:
    # Stable row packing, sorted by descending height/width and then ID.
    ordered = sorted(images, key=lambda item: (-item[1].height, -item[1].width, item[0]))
    area = sum((im.width + padding) * (im.height + padding) for _, im in ordered)
    max_width = max(256, min(max_side, 2 ** (max(8, (int(area ** 0.5) - 1).bit_length()))))
    x = y = row_height = 0
    page = 0
    placements: dict[str, dict[str, Any]] = {}
    page_heights: list[int] = []
    page_items: list[tuple[str, Image.Image, int, int]] = []
    for asset_id, im in ordered:
        if im.width + padding * 2 > max_width or im.height + padding * 2 > max_side:
            raise PipelineError(f"asset exceeds atlas budget before packing: {asset_id} ({im.width}x{im.height}; limit={max_side})")
        if x and x + im.width + padding * 2 > max_width:
            x = 0
            y += row_height
            row_height = 0
        if y + im.height + padding * 2 > max_side:
            if y == 0:
                raise PipelineError(f"asset exceeds atlas page height: {asset_id} ({im.height}px)")
            page_heights.append(y + row_height)
            page += 1
            x = y = row_height = 0
            if im.height + padding * 2 > max_side:
                raise PipelineError(f"asset exceeds atlas page height: {asset_id} ({im.height}px)")
        placements[asset_id] = {"page": page, "rect": [x + padding, y + padding, im.width, im.height]}
        page_items.append((asset_id, im, x + padding, y + padding))
        x += im.width + padding * 2
        row_height = max(row_height, im.height + padding * 2)
    page_heights.append(y + row_height)
    atlases = [Image.new("RGBA", (max_width, max(1, height)), (0, 0, 0, 0)) for height in page_heights]
    for asset_id, im, px, py in page_items:
        atlas = atlases[placements[asset_id]["page"]]
        placements[asset_id]["rect"] = [px, py, im.width, im.height]
        atlas.paste(im, (px, py))
        # Two-pixel edge extrusion reduces transparent sampling seams.
        for i in range(1, padding + 1):
            atlas.paste(im.crop((0, 0, im.width, 1)), (px, py - i))
            atlas.paste(im.crop((0, im.height - 1, im.width, im.height)), (px, py + im.height - 1 + i))
            atlas.paste(im.crop((0, 0, 1, im.height)), (px - i, py))
            atlas.paste(im.crop((im.width - 1, 0, im.width, im.height)), (px + im.width - 1 + i, py))
    return atlases, placements


def build(dest: Path = REPO) -> dict[str, Any]:
    webp_version = features.version("webp")
    if webp_version != EXPECTED_WEBP_ENCODER:
        raise PipelineError(f"WebP encoder version mismatch: expected {EXPECTED_WEBP_ENCODER}, found {webp_version}")
    manifest = load_authority()
    admitted = admitted_sources(manifest)
    approved_by_id = {row["asset_id"]: row for row, _ in admitted}
    cenyao_metadata, cenyao_frames = load_cenyao_idle_metadata(dest, approved_by_id)
    source_dir = dest / "assets/source/approved"
    derived_dir = dest / OUT
    metadata_dir = dest / "assets/metadata/asset-pipeline-v1"
    sprites_png = derived_dir / "sprites/png"
    sprites_webp = derived_dir / "sprites/webp"
    atlas_dir = derived_dir / "atlases"
    for directory in (source_dir, sprites_png, sprites_webp, atlas_dir, metadata_dir):
        directory.mkdir(parents=True, exist_ok=True)

    sprite_groups: dict[tuple[str, str], list[tuple[str, Image.Image]]] = {}
    records: list[dict[str, Any]] = []
    pending: list[dict[str, Any]] = []
    for row, blob in admitted:
        source_rel = row["relative_path"]
        source_copy = source_dir / source_rel
        if not source_copy.is_file():
            raise PipelineError(f"approved source copy is missing (read-only in this release fix): {source_rel}")
        if sha256(source_copy.read_bytes()) != row["sha256"]:
            raise PipelineError(f"approved source copy SHA mismatch: {row['asset_id']}")
        image, crop_info = clean_crop(blob)
        png_buffer = io.BytesIO()
        image.save(png_buffer, format="PNG", optimize=False, compress_level=9)
        png_bytes = png_buffer.getvalue()
        webp_buffer = io.BytesIO()
        image.save(webp_buffer, format="WEBP", lossless=True, method=6, exact=True)
        webp_bytes = webp_buffer.getvalue()
        name = row["asset_id"].replace("/", "_")
        png_path = sprites_png / f"{name}.png"
        webp_path = sprites_webp / f"{name}.webp"
        png_path.write_bytes(png_bytes)
        webp_path.write_bytes(webp_bytes)
        category, streaming_group = category_and_group(row["asset_id"])
        record = {
            "assetId": row["asset_id"],
            "category": category,
            "streamingGroup": streaming_group,
            "sourcePath": source_rel,
            "sourceSha256": row["sha256"],
            "admission": row["release_status"],
            "crop": crop_info,
            "anchor": anchor_for(row["asset_id"]),
            "scale": 1.0,
            "outputs": {
                "png": {"path": png_path.relative_to(dest).as_posix(), "sha256": sha256(png_bytes)},
                "webp": {"path": webp_path.relative_to(dest).as_posix(), "sha256": sha256(webp_bytes)},
            },
        }
        records.append(record)
        if budget_exceeded(category, image.width, image.height):
            pending.append({
                "assetId": row["asset_id"],
                "category": category,
                "sourcePath": source_rel,
                "sourceSha256": row["sha256"],
                "dimensions": [image.width, image.height],
                "maxSide": ATLAS_BUDGETS[category],
                "status": "PENDING_AUTHORED_EXTRACTION",
                "requiredAction": "author frame/cell extraction metadata; do not scale or truncate",
            })
            record["runtimeStatus"] = "PENDING_AUTHORED_EXTRACTION"
        else:
            record["runtimeStatus"] = "ADOPT"
            sprite_groups.setdefault((category, streaming_group), []).append((row["asset_id"], image))

    atlas_outputs: list[dict[str, Any]] = []
    for stale in (*atlas_dir.glob("*.png"), *atlas_dir.glob("*.webp")):
        stale.unlink()
    record_by_id = {record["assetId"]: record for record in records}
    for category, streaming_group in sorted(sprite_groups):
        category_images = sprite_groups[(category, streaming_group)]
        max_side = ATLAS_BUDGETS[category]
        atlases, placements = pack(category_images, max_side=max_side)
        for local_page, atlas in enumerate(atlases):
            page_index = len(atlas_outputs)
            page_tag = f"{category}-{streaming_group}-{local_page:02d}"
            atlas_png_buffer = io.BytesIO()
            atlas.save(atlas_png_buffer, format="PNG", optimize=False, compress_level=9)
            atlas_png = atlas_png_buffer.getvalue()
            atlas_webp_buffer = io.BytesIO()
            atlas.save(atlas_webp_buffer, format="WEBP", lossless=True, method=6, exact=True)
            atlas_webp = atlas_webp_buffer.getvalue()
            png_path = atlas_dir / f"{page_tag}.png"
            webp_path = atlas_dir / f"{page_tag}.webp"
            png_path.write_bytes(atlas_png)
            webp_path.write_bytes(atlas_webp)
            atlas_outputs.append({
                "category": category,
                "streamingGroup": streaming_group,
                "pageIndex": page_index,
                "pngPath": png_path.relative_to(dest).as_posix(), "pngSha256": sha256(atlas_png),
                "webpPath": webp_path.relative_to(dest).as_posix(), "webpSha256": sha256(atlas_webp),
                "size": list(atlas.size),
                "maxSide": max_side,
                "decodedRgbaMiB": round(atlas.width * atlas.height * 4 / (1024 * 1024), 3),
            })
            for asset_id, placement in placements.items():
                if placement["page"] == local_page:
                    record_by_id[asset_id]["atlasPage"] = page_index
                    record_by_id[asset_id]["atlasFrame"] = placement["rect"]

    # B6a's independently reviewed Cenyao idle atlas is a dedicated actor/shared
    # page. Keep the existing 13 B1a pages and their placements byte-for-byte.
    cenyao_page_index = len(atlas_outputs)
    if cenyao_page_index != 13:
        raise PipelineError(f"expected 13 existing runtime atlas pages before B6a integration, found {cenyao_page_index}")
    cenyao_atlas = cenyao_metadata["atlas"]
    atlas_outputs.append({
        "category": "actor", "streamingGroup": "shared", "pageIndex": cenyao_page_index,
        "pngPath": cenyao_atlas["png"]["path"], "pngSha256": cenyao_atlas["png"]["sha256"],
        "webpPath": cenyao_atlas["webp"]["path"], "webpSha256": cenyao_atlas["webp"]["sha256"],
        "size": cenyao_atlas["size"], "maxSide": 4096,
        "decodedRgbaMiB": round(cenyao_atlas["size"][0] * cenyao_atlas["size"][1] * 4 / (1024 * 1024), 3),
    })
    cenyao_record = record_by_id.get("runtime2d.actor.cenyao.base.v1")
    if not cenyao_record or cenyao_record["atlasPage"] != 3:
        raise PipelineError("B1a Cenyao static base atlas placement changed")
    cenyao_record["animation"] = {
        "directions": list(CENYAO_IDLE_DIRECTIONS), "states": ["idle"],
        "sourceMetadata": {"path": CENYAO_IDLE_METADATA.as_posix(), "sha256": CENYAO_IDLE_METADATA_SHA256},
        "sourceMasterSha256": CENYAO_MASTER_SHA256,
        "sourceAsset": {"assetId": cenyao_record["assetId"], "path": cenyao_record["sourcePath"], "sha256": cenyao_record["sourceSha256"]},
        "frames": [{
            "direction": frame["direction"], "state": "idle", "atlasPage": cenyao_page_index,
            "atlasFrame": frame["rect"], "extrudedFrame": frame["extrudedRect"],
            "anchor": [frame["anchor"]["x"], frame["anchor"]["y"]],
            "sourceFrame": frame["sourceFrame"],
        } for frame in cenyao_frames],
    }

    active_worlds = ("grey_hive", "mistharbor", "clockworks", "returnstation")
    resident_groups: dict[tuple[str, str], float] = {}
    for atlas in atlas_outputs:
        key = (atlas["category"], atlas["streamingGroup"])
        resident_groups[key] = resident_groups.get(key, 0.0) + atlas["decodedRgbaMiB"]
    residency_profiles = []
    for world in active_worlds:
        keys = {("actor", "shared"), ("vfx", "shared"), ("ui", "shared"), ("world", world)}
        keys.add(("actor", world))
        if world == "grey_hive":
            keys.update({("prop", "grey_hive"), ("boss", "sentinel")})
        resident_mib = round(sum(resident_groups.get(key, 0.0) for key in keys), 3)
        residency_profiles.append({"activeWorld": world, "residentTextureMiB": resident_mib, "residentGroups": [list(key) for key in sorted(keys) if key in resident_groups]})
    p95_values = sorted(profile["residentTextureMiB"] for profile in residency_profiles)
    p95_mib = p95_values[max(0, __import__("math").ceil(0.95 * len(p95_values)) - 1)]
    if p95_mib > 220:
        raise PipelineError(f"P95 resident atlas estimate exceeds 220 MiB: {p95_mib}")

    runtime_manifest = {
        "schemaId": "runtime-asset-manifest/1",
        "manifestState": "ADOPT",
        "authority": {"path": "governance/assets/AI_ASSET_RELEASE_MANIFEST.json", "sourceCommit": SOURCE_COMMIT},
        "tool": {"version": TOOL_VERSION, "python": platform.python_version(), "pillow": PILLOW_VERSION, "webpEncoder": webp_version},
        "atlases": atlas_outputs,
        "assets": [record for record in records if record.get("runtimeStatus") == "ADOPT"],
        "pendingAssets": pending,
        "residency": {"method": "RGBA decoded texture footprint; lazy-load shared plus one active world; nearest-rank P95 over four world profiles", "budgetMiB": 220, "p95MiB": p95_mib, "profiles": residency_profiles},
    }
    (dest / "governance/assets/RUNTIME_ASSET_MANIFEST.json").parent.mkdir(parents=True, exist_ok=True)
    (dest / "governance/assets/RUNTIME_ASSET_MANIFEST.json").write_bytes(canonical_json(runtime_manifest))
    provenance = {
        "schemaId": "asset-pipeline-provenance/1",
        "tool": runtime_manifest["tool"],
        "sourceCommit": SOURCE_COMMIT,
        "sourceManifestSha256": sha256(MANIFEST.read_bytes()),
        "parameters": {"alpha": "trim non-zero alpha bounds", "transparentRgb": "zero", "anchor": "name-category heuristic", "scale": 1.0, "atlasPadding": 2, "atlasMaxSideByCategory": ATLAS_BUDGETS, "packOrder": "height,width,id; shelf", "png": "RGBA; optimize=false; compress_level=9", "webp": "lossless=true; method=6; exact=true", "cenyaoIdleAtlas": {"metadataPath": CENYAO_IDLE_METADATA.as_posix(), "metadataSha256": CENYAO_IDLE_METADATA_SHA256, "sourceMasterSha256": CENYAO_MASTER_SHA256, "integration": "dedicated actor/shared page; B6a bytes are reused without rewriting"}},
        "assets": records,
        "pendingAssets": pending,
        "atlases": atlas_outputs,
        "residency": runtime_manifest["residency"],
    }
    (metadata_dir / "provenance.json").write_bytes(canonical_json(provenance))
    return {"approved": len(records), "runtimeAdopted": len(runtime_manifest["assets"]), "pending": len(pending), "atlasPages": len(atlas_outputs), "atlasSizes": [atlas["size"] for atlas in atlas_outputs], "p95ResidentMiB": p95_mib, "runtimeManifestSha256": sha256(canonical_json(runtime_manifest)), "cenyaoIdleFrames": len(cenyao_frames), "cenyaoAtlasPage": cenyao_page_index, "cenyaoAtlasPngSha256": cenyao_atlas["png"]["sha256"], "cenyaoAtlasWebpSha256": cenyao_atlas["webp"]["sha256"]}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", nargs="?", choices=["build"], default="build")
    parser.add_argument("--dest", type=Path, default=REPO)
    args = parser.parse_args()
    try:
        print(json.dumps(build(args.dest), ensure_ascii=False, sort_keys=True))
        return 0
    except PipelineError as exc:
        print(f"ASSET PIPELINE FAIL CLOSED: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
