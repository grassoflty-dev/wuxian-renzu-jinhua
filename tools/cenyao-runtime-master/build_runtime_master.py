#!/usr/bin/env python3
"""Deterministically extract Cenyao's eight idle directions from the approved sheet."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, __version__ as PILLOW_VERSION

ROOT = Path(__file__).resolve().parents[2]
SOURCE_REL = "assets/source/approved/batch01/01_runtime2d.actor.cenyao.base.v1.png"
PARTS_REL = "assets/source/approved/batch01/02_runtime2d.actor.cenyao.parts.v1.png"
AUTHORITY_REL = "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"
AUTHORITY_SHA256 = "c644b5953e9265d45796dfb8a6dad1c0fd8abfbffb3eb4ace84987d8238cee2d"
DECISION_DOC_REL = "docs/source/deep-research-report.md"
DECISION_DOC_SHA256 = "fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1"
SOURCE_SHA256 = "1915e77897a4facea0bfee4c529daf5cd4961998fe4887efc4a340f253424801"
TOOL_VERSION = "cenyao-runtime-master-v1.0.0"
PNG_DIR = Path("assets/derived/cenyao-runtime-master-v1/png")
WEBP_DIR = Path("assets/derived/cenyao-runtime-master-v1/webp")
METADATA_PATH = Path("assets/metadata/cenyao-runtime-master-v1/cenyao-runtime-master-v1.json")
ACCEPTANCE_DIR = Path("artifacts/acceptance/cenyao-runtime-master-v1")

# The sheet is row-major. Directions are grounded in the visible body/head
# facing of each source pose; no pose is synthesized or mirrored.
POSES = [
    ("south", 0, 0),
    ("south_west", 0, 1),
    ("west", 0, 2),
    ("north_east", 0, 3),
    ("north", 1, 0),
    ("north_west", 1, 1),
    ("east", 1, 2),
    ("south_east", 1, 3),
]
EXPECTED_DIRECTIONS = {"north", "north_east", "east", "south_east", "south", "south_west", "west", "north_west"}
MARGIN = 8
FOOT_BAND_PX = 20


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def approved_rows() -> dict[str, dict]:
    manifest_path = ROOT / AUTHORITY_REL
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    return {row["relative_path"]: row for row in manifest["assets"]}


def load_sources() -> tuple[Image.Image, dict, dict]:
    rows = approved_rows()
    if digest(ROOT / AUTHORITY_REL) != AUTHORITY_SHA256:
        raise ValueError("E_CENYAO_B0_AUTHORITY_SHA")
    source_path = ROOT / SOURCE_REL
    parts_path = ROOT / PARTS_REL
    decision_doc_path = ROOT / DECISION_DOC_REL
    if digest(decision_doc_path) != DECISION_DOC_SHA256:
        raise ValueError("E_CENYAO_DECISION_DOC_SHA")
    source_sha = digest(source_path)
    if source_sha != SOURCE_SHA256:
        raise ValueError(f"E_CENYAO_SOURCE_SHA:{source_sha}")
    source_row = rows.get("batch01/01_runtime2d.actor.cenyao.base.v1.png")
    parts_row = rows.get("batch01/02_runtime2d.actor.cenyao.parts.v1.png")
    if not source_row or source_row["release_status"] != "release_approved" or source_row["sha256"] != source_sha:
        raise ValueError("E_CENYAO_SOURCE_NOT_APPROVED")
    if not parts_row or parts_row["release_status"] != "release_approved" or parts_row["sha256"] != digest(parts_path):
        raise ValueError("E_CENYAO_PARTS_REFERENCE_NOT_APPROVED")
    return Image.open(source_path).convert("RGBA"), source_row, parts_row


def cell_rect(width: int, height: int, row: int, col: int) -> tuple[int, int, int, int]:
    # Integer boundary calculation stays correct if a later approved sheet has
    # a dimension that is not exactly divisible by its grid count.
    x0, x1 = round(col * width / 4), round((col + 1) * width / 4)
    y0, y1 = round(row * height / 2), round((row + 1) * height / 2)
    return x0, y0, x1, y1


def analyze(source: Image.Image) -> list[dict]:
    width, height = source.size
    alpha = source.getchannel("A")
    records = []
    for direction, row, col in POSES:
        x0, y0, x1, y1 = cell_rect(width, height, row, col)
        cell = source.crop((x0, y0, x1, y1))
        raw_cell_alpha = cell.getchannel("A")
        # The approved sheet contains a one-level alpha residue at outer and
        # inter-cell borders. Clearing only alpha=1 isolates real pose edges;
        # all source alpha samples >=2 remain byte-for-byte unchanged.
        cell_alpha = raw_cell_alpha.point(lambda value: 0 if value <= 1 else value)
        bbox = cell_alpha.getbbox()
        if bbox is None:
            raise ValueError(f"E_CENYAO_EMPTY_CELL:{direction}")
        bx0, by0, bx1, by1 = bbox
        cell_w, cell_h = cell.size
        edge_samples = []
        for xx in range(cell_w):
            for yy, side in ((0, "top"), (cell_h - 1, "bottom")):
                value = cell_alpha.getpixel((xx, yy))
                if value > 0:
                    edge_samples.append([side, xx, value])
        for yy in range(1, cell_h - 1):
            for xx, side in ((0, "left"), (cell_w - 1, "right")):
                value = cell_alpha.getpixel((xx, yy))
                if value > 0:
                    edge_samples.append([side, yy, value])
        edge_pixels = len(edge_samples)
        crop = cell.crop(bbox)
        crop.putalpha(cell_alpha.crop(bbox))
        crop_alpha = crop.getchannel("A")
        crop_w, crop_h = crop.size
        bottom_band = crop_alpha.crop((0, max(0, crop_h - FOOT_BAND_PX), crop_w, crop_h))
        foot_bbox = bottom_band.getbbox()
        if foot_bbox is None:
            raise ValueError(f"E_CENYAO_NO_FOOT_SUPPORT:{direction}")
        foot_center_x = (foot_bbox[0] + foot_bbox[2] - 1) / 2
        foot_bottom_y = crop_h - 1
        partial_alpha = sum(1 for value in crop_alpha.get_flattened_data() if 0 < value < 255)
        alpha_pixels = sum(1 for value in crop_alpha.get_flattened_data() if value > 0)
        records.append({
            "direction": direction,
            "row": row,
            "column": col,
            "cellRect": [x0, y0, x1 - x0, y1 - y0],
            "sourceAlphaBounds": [x0 + bx0, y0 + by0, bx1 - bx0, by1 - by0],
            "cropRect": [x0 + bx0, y0 + by0, bx1 - bx0, by1 - by0],
            "crop": crop,
            "footBandSourceRect": [x0 + bx0 + foot_bbox[0], y0 + by0 + max(0, crop_h - FOOT_BAND_PX) + foot_bbox[1], foot_bbox[2] - foot_bbox[0], foot_bbox[3] - foot_bbox[1]],
            "footCenterInCrop": [foot_center_x, foot_bottom_y],
            "alphaPixelCount": alpha_pixels,
            "partialAlphaPixelCount": partial_alpha,
            "edgeAlphaPixelCount": edge_pixels,
            "edgeAlphaSamples": edge_samples,
            "removedAlphaOnePixelCount": sum(1 for value in raw_cell_alpha.get_flattened_data() if value == 1),
        })
    if {record["direction"] for record in records} != EXPECTED_DIRECTIONS or len(records) != 8:
        raise ValueError("E_CENYAO_DIRECTION_SET")
    return records


def describe_only() -> None:
    source, _, _ = load_sources()
    records = analyze(source)
    alpha = source.getchannel("A")
    zero_alpha_nonzero_rgb = 0
    alpha_one_pixels = 0
    pixels = source.load()
    for y in range(source.height):
        for x in range(source.width):
            red, green, blue, a = pixels[x, y]
            if a == 0 and (red or green or blue):
                zero_alpha_nonzero_rgb += 1
            if a == 1:
                alpha_one_pixels += 1
    print(json.dumps({
        "sourceSize": list(source.size),
        "sourceAlphaExtrema": list(alpha.getextrema()),
        "zeroAlphaNonzeroRgbPixels": zero_alpha_nonzero_rgb,
        "alphaOneResiduePixels": alpha_one_pixels,
        "poses": [{key: value for key, value in record.items() if key != "crop"} for record in records],
    }, ensure_ascii=False, indent=2))


def prepare_crop(record: dict, canvas_size: tuple[int, int]) -> tuple[Image.Image, tuple[int, int], tuple[int, int]]:
    crop: Image.Image = record["crop"]
    canvas_w, canvas_h = canvas_size
    foot_x, foot_y = record["footCenterInCrop"]
    anchor_x = (canvas_w - 1) / 2
    anchor_y = canvas_h - 1 - MARGIN
    left = round(anchor_x - foot_x)
    top = round(anchor_y - foot_y)
    if left < MARGIN or top < MARGIN or left + crop.width > canvas_w - MARGIN or top + crop.height > canvas_h - MARGIN:
        raise ValueError(f"E_CENYAO_CANVAS_CLIP:{record['direction']}:{left},{top},{crop.size}")
    clean = crop.copy()
    clean.putalpha(clean.getchannel("A").point(lambda value: 0 if value <= 1 else value))
    # Preserve every alpha sample >=2 from the approved source, while removing
    # alpha=1 border residue and hidden RGB under transparent pixels.
    data = list(clean.get_flattened_data())
    clean.putdata([(0, 0, 0, 0) if pixel[3] == 0 else pixel for pixel in data])
    canvas = Image.new("RGBA", canvas_size, (0, 0, 0, 0))
    canvas.alpha_composite(clean, (left, top))
    return canvas, (left, top), (round(anchor_x), round(anchor_y))


def make_checkerboard(size: tuple[int, int], tile: int = 12) -> Image.Image:
    image = Image.new("RGB", size, (220, 220, 220))
    draw = ImageDraw.Draw(image)
    for y in range(0, size[1], tile):
        for x in range(0, size[0], tile):
            if (x // tile + y // tile) % 2:
                draw.rectangle((x, y, min(size[0], x + tile) - 1, min(size[1], y + tile) - 1), fill=(174, 174, 174))
    return image


def render_bundle(target: Path) -> dict[str, str]:
    source, source_row, parts_row = load_sources()
    records = analyze(source)
    max_foot_extent = max(
        max(record["footCenterInCrop"][0], record["crop"].width - 1 - record["footCenterInCrop"][0])
        for record in records
    )
    max_h = max(record["crop"].height for record in records)
    canvas_w = math.ceil((max_foot_extent * 2 + 1 + MARGIN * 2) / 16) * 16
    canvas_h = math.ceil((max_h + MARGIN * 2) / 16) * 16
    if canvas_w > 1024 or canvas_h > 1024:
        raise ValueError(f"E_CENYAO_OUTPUT_BUDGET:{canvas_w}x{canvas_h}")

    png_dir, webp_dir = target / "png", target / "webp"
    png_dir.mkdir(parents=True, exist_ok=True)
    webp_dir.mkdir(parents=True, exist_ok=True)
    placed = []
    frame_metadata = []
    for record in records:
        frame, placement, anchor = prepare_crop(record, (canvas_w, canvas_h))
        direction = record["direction"]
        png_path = png_dir / f"cenyao_idle_{direction}.png"
        webp_path = webp_dir / f"cenyao_idle_{direction}.webp"
        frame.save(png_path, format="PNG", optimize=False, compress_level=9)
        frame.save(webp_path, format="WEBP", lossless=True, method=6, exact=True)
        frame_metadata.append({
            "direction": direction,
            "sourceCell": {"row": record["row"], "column": record["column"], "rect": record["cellRect"]},
            "sourceAlphaBounds": record["sourceAlphaBounds"],
            "sourceCropRect": record["cropRect"],
            "footSupportSourceRect": record["footBandSourceRect"],
            "outputSize": [canvas_w, canvas_h],
            "contentPlacement": [placement[0], placement[1]],
            "anchor": {
                "x": round(anchor[0] / canvas_w, 8),
                "y": round(anchor[1] / canvas_h, 8),
                "pixel": [anchor[0], anchor[1]],
                "method": f"x midpoint of nontransparent support pixels in final {FOOT_BAND_PX} crop rows; y bottommost source silhouette pixel with {MARGIN}px safe canvas padding",
            },
            "alphaPixelCount": record["alphaPixelCount"],
            "partialAlphaPixelCount": record["partialAlphaPixelCount"],
            "removedAlphaOnePixelCount": record["removedAlphaOnePixelCount"],
            "sourceAlphaPreserved": "alpha 2..255 exact; alpha 0..1 removed as border residue",
            "cleanup": "zero alpha 0..1 as verified grid/background residue; preserve source alpha 2..255 exactly; zero RGB where output alpha is zero",
            "outputs": {
                "png": {"path": (PNG_DIR / png_path.name).as_posix(), "sha256": digest(png_path)},
                "webp": {"path": (WEBP_DIR / webp_path.name).as_posix(), "sha256": digest(webp_path)},
            },
        })
        placed.append(frame)

    pad, label_h = 24, 28
    sheet_w, sheet_h = 4 * canvas_w + 5 * pad, 2 * (canvas_h + label_h) + 3 * pad
    checker = make_checkerboard((sheet_w, sheet_h)).convert("RGBA")
    transparent = Image.new("RGBA", (sheet_w, sheet_h), (0, 0, 0, 0))
    draw = ImageDraw.Draw(checker)
    transparent_draw = ImageDraw.Draw(transparent)
    font = ImageFont.load_default()
    for index, ((direction, row, col), frame) in enumerate(zip(POSES, placed, strict=True)):
        x = pad + col * (canvas_w + pad)
        y = pad + row * (canvas_h + label_h + pad)
        checker.alpha_composite(frame, (x, y + label_h))
        draw.text((x + 2, y + 5), direction.upper(), font=font, fill=(18, 22, 30, 255))
        transparent.alpha_composite(frame, (x, y + label_h))
        transparent_draw.text((x + 2, y + 5), direction.upper(), font=font, fill=(20, 28, 42, 235))
    contact_path = target / "cenyao-8way-contact-sheet.png"
    transparent_path = target / "cenyao-8way-transparent-preview.png"
    checker.convert("RGB").save(contact_path, format="PNG", optimize=False, compress_level=9)
    transparent.save(transparent_path, format="PNG", optimize=False, compress_level=9)

    source_width, source_height = source.size
    metadata = {
        "schemaId": "cenyao-runtime-master-v1/1",
        "tool": {"name": TOOL_VERSION, "pillow": PILLOW_VERSION, "python": "3.13+", "resampling": "none; crop and integer translation only", "webp": "lossless; method=6; exact=true"},
        "decisionAuthority": {"path": DECISION_DOC_REL, "sha256": DECISION_DOC_SHA256},
        "b0Authority": {"path": AUTHORITY_REL, "sha256": AUTHORITY_SHA256},
        "source": {"path": SOURCE_REL, "sha256": SOURCE_SHA256, "approved": True, "assetId": source_row["asset_id"], "size": [source_width, source_height]},
        "partsReference": {"path": PARTS_REL, "sha256": parts_row["sha256"], "approved": True, "usage": "reference-only; not sampled, composited, or used to synthesize frames; future rig boundaries may be assessed separately"},
        "layout": {"columns": 4, "rows": 2, "cellSize": [round(source_width / 4), round(source_height / 2)], "mapping": [{"direction": d, "row": r, "column": c} for d, r, c in POSES]},
        "canvas": {"size": [canvas_w, canvas_h], "maxSide": 1024, "preserveAspectRatio": True, "uniformAcrossFrames": True, "transparentPaddingPx": MARGIN},
        "extraction": {"crop": "cell alpha bounding box after verified alpha=1 border-residue removal", "sourceAlphaPolicy": "set alpha 0..1 to 0; preserve alpha 2..255 exactly", "hiddenRgbPolicy": "zero RGB where output alpha is zero", "frameClass": "idle directional pose only; no animation frames inferred"},
        "frames": frame_metadata,
    }
    metadata_path = target / "cenyao-runtime-master-v1.json"
    metadata_path.write_text(json.dumps(metadata, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8", newline="\n")

    return {path.relative_to(target).as_posix(): digest(path) for path in sorted(target.rglob("*")) if path.is_file()}


def copy_outputs(stage: Path) -> None:
    targets = [
        (stage / "png", ROOT / PNG_DIR),
        (stage / "webp", ROOT / WEBP_DIR),
    ]
    for source_dir, target_dir in targets:
        target_dir.mkdir(parents=True, exist_ok=True)
        for path in source_dir.iterdir():
            shutil.copyfile(path, target_dir / path.name)
    metadata_target = ROOT / METADATA_PATH
    metadata_target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(stage / "cenyao-runtime-master-v1.json", metadata_target)
    acceptance = ROOT / ACCEPTANCE_DIR
    acceptance.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(stage / "cenyao-8way-contact-sheet.png", acceptance / "cenyao-8way-contact-sheet.png")
    shutil.copyfile(stage / "cenyao-8way-transparent-preview.png", acceptance / "cenyao-8way-transparent-preview.png")


def build_twice() -> dict:
    tool_dir = ROOT / "tools/cenyao-runtime-master"
    tmp_root = tool_dir / ".repro-build"
    if tmp_root.exists():
        shutil.rmtree(tmp_root)
    tmp_root.mkdir(parents=True)
    try:
        first, second = tmp_root / "first", tmp_root / "second"
        hashes_first = render_bundle(first)
        hashes_second = render_bundle(second)
        if hashes_first != hashes_second:
            different = sorted(key for key in set(hashes_first) | set(hashes_second) if hashes_first.get(key) != hashes_second.get(key))
            raise ValueError(f"E_CENYAO_NONDETERMINISTIC_OUTPUT:{','.join(different)}")
        copy_outputs(first)
        record = {
            "result": "pass",
            "tool": TOOL_VERSION,
            "builtTwiceByteIdentical": True,
            "sourceSha256": SOURCE_SHA256,
            "frameCount": 8,
            "outputHashes": hashes_first,
        }
        acceptance = ROOT / ACCEPTANCE_DIR
        acceptance.mkdir(parents=True, exist_ok=True)
        (acceptance / "reproducibility.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        return record
    finally:
        shutil.rmtree(tmp_root)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inspect", action="store_true", help="print source-grid and alpha measurements without writing assets")
    args = parser.parse_args()
    if args.inspect:
        describe_only()
    else:
        print(json.dumps(build_twice(), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
