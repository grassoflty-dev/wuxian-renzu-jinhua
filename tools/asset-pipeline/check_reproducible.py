from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("asset_pipeline", Path(__file__).with_name("asset_pipeline.py"))
pipeline = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(pipeline)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def output_hashes() -> dict[str, str]:
    paths = [
        *((ROOT / "assets/source/approved").rglob("*")),
        *((ROOT / "assets/derived/asset-pipeline-v1").rglob("*")),
        *((ROOT / "assets/metadata/asset-pipeline-v1").rglob("*")),
        *((ROOT / "assets/derived/cenyao-runtime-master-v1").rglob("*")),
        ROOT / "assets/metadata/cenyao-runtime-master-v1/cenyao-runtime-master-v1.json",
        ROOT / "assets/metadata/cenyao-runtime-master-v1/cenyao-idle-atlas-v1.json",
        ROOT / "governance/assets/RUNTIME_ASSET_MANIFEST.json",
    ]
    files = sorted(path for path in paths if path.is_file())
    return {path.relative_to(ROOT).as_posix(): digest(path.read_bytes()) for path in files}


def main() -> int:
    first = pipeline.build(ROOT)
    first_hashes = output_hashes()
    second = pipeline.build(ROOT)
    second_hashes = output_hashes()
    if first_hashes != second_hashes:
        differing = sorted(set(first_hashes) | set(second_hashes))
        differing = [name for name in differing if first_hashes.get(name) != second_hashes.get(name)]
        print(json.dumps({"result": "fail", "differingFiles": differing}, ensure_ascii=False))
        return 1

    manifest = json.loads((ROOT / "governance/assets/RUNTIME_ASSET_MANIFEST.json").read_text(encoding="utf-8"))
    authority = pipeline.load_authority()
    approved = {row["asset_id"]: row for row in authority["assets"] if row["release_status"] == "release_approved"}
    b6a, b6a_frames = pipeline.load_cenyao_idle_metadata(ROOT, approved)
    adopted = {row["assetId"] for row in manifest.get("assets", [])}
    pending = {row["assetId"] for row in manifest.get("pendingAssets", [])}
    if adopted | pending != set(approved) or adopted & pending:
        raise SystemExit("runtime/admission and pending sets do not partition the exact 43 approved authority rows")
    pages = manifest["atlases"]
    page_by_index = {page["pageIndex"]: page for page in pages}
    occupied: dict[int, list[tuple[str, int, int, int, int]]] = {}
    for item in manifest["assets"]:
        source = approved[item["assetId"]]
        if item["sourceSha256"] != source["sha256"] or item["admission"] != "release_approved" or item["runtimeStatus"] != "ADOPT":
            raise SystemExit(f"runtime source provenance mismatch: {item['assetId']}")
        page = page_by_index[item["atlasPage"]]
        if page["category"] != item["category"]:
            raise SystemExit(f"atlas category mismatch: {item['assetId']}")
        max_side = pipeline.ATLAS_BUDGETS[item["category"]]
        width, height = page["size"]
        if item["category"] == "ui":
            valid_size = width <= 2048 and height <= 2048
        else:
            valid_size = width <= max_side and height <= max_side
        if not valid_size:
            raise SystemExit(f"category atlas budget exceeded: {page['pngPath']}")
        x, y, frame_width, frame_height = item["atlasFrame"]
        if x < 0 or y < 0 or x + frame_width > width or y + frame_height > height:
            raise SystemExit(f"atlas frame outside page: {item['assetId']}")
        for other_id, ox, oy, ow, oh in occupied.setdefault(item["atlasPage"], []):
            if x < ox + ow and ox < x + frame_width and y < oy + oh and oy < y + frame_height:
                raise SystemExit(f"overlapping frames: {item['assetId']} and {other_id}")
        occupied[item["atlasPage"]].append((item["assetId"], x, y, frame_width, frame_height))
        for output in item["outputs"].values():
            file = ROOT / output["path"]
            if digest(file.read_bytes()) != output["sha256"]:
                raise SystemExit(f"runtime sprite output SHA mismatch: {item['assetId']}")
    cenyao = next(item for item in manifest["assets"] if item["assetId"] == "runtime2d.actor.cenyao.base.v1")
    animation = cenyao.get("animation")
    page_index = len(manifest["atlases"]) - 1
    page = page_by_index[page_index]
    if page["category"] != "actor" or page["streamingGroup"] != "shared" or page["pngPath"] != b6a["atlas"]["png"]["path"] or page["webpPath"] != b6a["atlas"]["webp"]["path"]:
        raise SystemExit("B6a Cenyao atlas is not the dedicated final actor/shared page")
    if not isinstance(animation, dict) or animation.get("sourceMetadata") != {"path": pipeline.CENYAO_IDLE_METADATA.as_posix(), "sha256": pipeline.CENYAO_IDLE_METADATA_SHA256} or animation.get("sourceMasterSha256") != pipeline.CENYAO_MASTER_SHA256:
        raise SystemExit("Cenyao animation source metadata provenance mismatch")
    if animation.get("directions") != list(pipeline.CENYAO_IDLE_DIRECTIONS) or animation.get("states") != ["idle"] or len(animation.get("frames", [])) != 8:
        raise SystemExit("Cenyao animation does not contain the exact eight-way idle set")
    if animation.get("sourceAsset") != {"assetId": cenyao["assetId"], "path": cenyao["sourcePath"], "sha256": cenyao["sourceSha256"]}:
        raise SystemExit("Cenyao animation B0 source identity mismatch")
    for runtime_frame, b6a_frame in zip(animation["frames"], b6a_frames, strict=True):
        expected = {
            "direction": b6a_frame["direction"], "state": "idle", "atlasPage": page_index,
            "atlasFrame": b6a_frame["rect"], "extrudedFrame": b6a_frame["extrudedRect"],
            "anchor": [b6a_frame["anchor"]["x"], b6a_frame["anchor"]["y"]],
            "sourceFrame": b6a_frame["sourceFrame"],
        }
        if runtime_frame != expected:
            raise SystemExit(f"Cenyao frame differs from B6a metadata: {b6a_frame['direction']}")
        x, y, frame_width, frame_height = runtime_frame["atlasFrame"]
        if x < 0 or y < 0 or x + frame_width > page["size"][0] or y + frame_height > page["size"][1]:
            raise SystemExit(f"Cenyao frame outside B6a atlas: {b6a_frame['direction']}")
        for other_id, ox, oy, ow, oh in occupied.setdefault(page_index, []):
            if x < ox + ow and ox < x + frame_width and y < oy + oh and oy < y + frame_height:
                raise SystemExit(f"Cenyao frame overlaps {other_id}")
        occupied[page_index].append((f"cenyao:{runtime_frame['direction']}:idle", x, y, frame_width, frame_height))
    for atlas in manifest["atlases"]:
        for key in ("pngPath", "webpPath"):
            output_path = ROOT / atlas[key]
            sha_key = "pngSha256" if key == "pngPath" else "webpSha256"
            if digest(output_path.read_bytes()) != atlas[sha_key]:
                raise SystemExit(f"atlas SHA mismatch: {atlas[key]}")

    boss_assets = [item for item in manifest["assets"] if item["category"] == "boss"]
    boss_pages = {item["atlasPage"] for item in boss_assets}
    if len(boss_pages) != 1 or not boss_assets or page_by_index[next(iter(boss_pages))]["category"] != "boss":
        raise SystemExit("Boss assets must occupy one dedicated Boss page")
    if any(page["category"] == "boss" and page["streamingGroup"] != "sentinel" for page in pages):
        raise SystemExit("Boss page contains a non-Boss streaming group")

    if manifest["residency"]["p95MiB"] > 220:
        raise SystemExit(f"P95 resident texture budget exceeded: {manifest['residency']['p95MiB']} MiB")
    evidence = {
        "result": "pass",
        "sourceAuthority": "B0 ASSET-GOV-V1",
        "sourceCommit": pipeline.SOURCE_COMMIT,
        "approvedCount": len(approved),
        "runtimeAdoptedCount": len(adopted),
        "pendingCount": len(pending),
        "atlasPages": [{"category": page["category"], "streamingGroup": page["streamingGroup"], "size": page["size"], "decodedRgbaMiB": page["decodedRgbaMiB"]} for page in pages],
        "residency": manifest["residency"],
        "excludedConceptAndRejectRows": 51,
        "cenyaoIdleIntegration": {"directions": list(pipeline.CENYAO_IDLE_DIRECTIONS), "atlasPage": page_index, "metadataSha256": pipeline.CENYAO_IDLE_METADATA_SHA256, "sourceMasterSha256": pipeline.CENYAO_MASTER_SHA256, "sourceFramesValidated": 8},
        "buildOne": first,
        "buildTwo": second,
        "outputFileCount": len(first_hashes),
        "outputSetSha256": digest(json.dumps(first_hashes, sort_keys=True, separators=(",", ":")).encode("utf-8")),
        "allOutputSha256": first_hashes,
    }
    target = ROOT / "artifacts/acceptance/asset-pipeline-core/reproducibility.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(evidence, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in evidence.items() if key != "allOutputSha256"}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
