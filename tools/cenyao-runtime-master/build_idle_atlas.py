#!/usr/bin/env python3
"""Pack the approved Cenyao eight-way idle masters into one deterministic atlas."""

from __future__ import annotations

import hashlib
import json
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, __version__ as PILLOW_VERSION


ROOT = Path(__file__).resolve().parents[2]
MASTER_META = Path("assets/metadata/cenyao-runtime-master-v1/cenyao-runtime-master-v1.json")
ATLAS_META = Path("assets/metadata/cenyao-runtime-master-v1/cenyao-idle-atlas-v1.json")
ATLAS_DIR = Path("assets/derived/cenyao-runtime-master-v1/atlas")
EVIDENCE_DIR = Path("artifacts/acceptance/cenyao-idle-atlas-v1")
ORDER = ("south", "south_west", "west", "north_east", "north", "north_west", "east", "south_east")
EXTRUDE = 2
COLS = 4


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_frames() -> tuple[dict, list[dict]]:
    meta = json.loads((ROOT / MASTER_META).read_text(encoding="utf-8"))
    frames = meta["frames"]
    if tuple(frame["direction"] for frame in frames) != ORDER:
        raise ValueError("E_IDLE_DIRECTION_ORDER")
    for frame in frames:
        for fmt in ("png", "webp"):
            source = ROOT / frame["outputs"][fmt]["path"]
            if sha(source) != frame["outputs"][fmt]["sha256"]:
                raise ValueError(f"E_IDLE_FRAME_SHA:{fmt}:{frame['direction']}")
        with Image.open(ROOT / frame["outputs"]["png"]["path"]) as png:
            with Image.open(ROOT / frame["outputs"]["webp"]["path"]) as webp:
                if png.size != webp.size or png.convert("RGBA").tobytes() != webp.convert("RGBA").tobytes():
                    raise ValueError(f"E_IDLE_FRAME_FORMAT_MISMATCH:{frame['direction']}")
    return meta, frames


def extrude(atlas: Image.Image, frame: Image.Image, x: int, y: int) -> None:
    """Copy actual edge pixels two pixels outward, including alpha."""
    w, h = frame.size
    atlas.paste(frame, (x, y))
    atlas.paste(frame.crop((0, 0, 1, h)).resize((EXTRUDE, h)), (x - EXTRUDE, y))
    atlas.paste(frame.crop((w - 1, 0, w, h)).resize((EXTRUDE, h)), (x + w, y))
    atlas.paste(frame.crop((0, 0, w, 1)).resize((w, EXTRUDE)), (x, y - EXTRUDE))
    atlas.paste(frame.crop((0, h - 1, w, h)).resize((w, EXTRUDE)), (x, y + h))
    for sx, sy, dx, dy in (
        (0, 0, x - EXTRUDE, y - EXTRUDE),
        (w - 1, 0, x + w, y - EXTRUDE),
        (0, h - 1, x - EXTRUDE, y + h),
        (w - 1, h - 1, x + w, y + h),
    ):
        atlas.paste(frame.crop((sx, sy, sx + 1, sy + 1)).resize((EXTRUDE, EXTRUDE)), (dx, dy))


def render(output: Path) -> dict[str, str]:
    master, frames = source_frames()
    sizes = {tuple(frame["outputSize"]) for frame in frames}
    if len(sizes) != 1:
        raise ValueError("E_IDLE_FRAME_SIZE")
    fw, fh = sizes.pop()
    cell_w, cell_h = fw + 2 * EXTRUDE, fh + 2 * EXTRUDE
    aw, ah = COLS * cell_w, 2 * cell_h
    if aw > 4096 or ah > 4096:
        raise ValueError("E_IDLE_ATLAS_BUDGET")
    atlas = Image.new("RGBA", (aw, ah), (0, 0, 0, 0))
    entries = []
    occupied = []
    for i, frame in enumerate(frames):
        col, row = i % COLS, i // COLS
        x, y = col * cell_w + EXTRUDE, row * cell_h + EXTRUDE
        with Image.open(ROOT / frame["outputs"]["png"]["path"]) as image:
            rgba = image.convert("RGBA")
        extrude(atlas, rgba, x, y)
        outer = [x - EXTRUDE, y - EXTRUDE, cell_w, cell_h]
        for ox, oy, ow, oh in occupied:
            if max(outer[0], ox) < min(outer[0] + outer[2], ox + ow) and max(outer[1], oy) < min(outer[1] + outer[3], oy + oh):
                raise ValueError("E_IDLE_ATLAS_OVERLAP")
        occupied.append(outer)
        entries.append({
            "direction": frame["direction"],
            "sourceFrame": frame["outputs"],
            "rect": [x, y, fw, fh],
            "extrudedRect": outer,
            "anchor": frame["anchor"],
            "sourceCell": frame["sourceCell"],
        })

    output.mkdir(parents=True, exist_ok=True)
    png = output / "cenyao-idle-atlas-v1.png"
    webp = output / "cenyao-idle-atlas-v1.webp"
    atlas.save(png, format="PNG", optimize=False, compress_level=9)
    atlas.save(webp, format="WEBP", lossless=True, method=6, exact=True)
    with Image.open(webp) as decoded:
        if atlas.tobytes() != decoded.convert("RGBA").tobytes():
            raise ValueError("E_IDLE_ATLAS_WEBP_LOSS")

    metadata = {
        "schemaVersion": "cenyao-idle-atlas-v1",
        "sourceMaster": {"path": MASTER_META.as_posix(), "sha256": sha(ROOT / MASTER_META)},
        "approvedSource": master["source"],
        "atlas": {"size": [aw, ah], "maxSide": 4096, "extrusionPixels": EXTRUDE,
                  "png": {"path": (ATLAS_DIR / png.name).as_posix(), "sha256": sha(png)},
                  "webp": {"path": (ATLAS_DIR / webp.name).as_posix(), "sha256": sha(webp)}},
        "frames": entries,
        "tool": {"name": "build_idle_atlas.py", "pillowVersion": PILLOW_VERSION},
    }
    metadata_path = output / ATLAS_META.name
    metadata_path.write_bytes((json.dumps(metadata, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    return {path.name: sha(path) for path in (png, webp, metadata_path)}


def publish() -> dict:
    with tempfile.TemporaryDirectory(prefix="cenyao-idle-atlas-a-") as first, tempfile.TemporaryDirectory(prefix="cenyao-idle-atlas-b-") as second:
        a, b = Path(first), Path(second)
        hashes_a, hashes_b = render(a), render(b)
        if hashes_a != hashes_b:
            raise ValueError("E_IDLE_ATLAS_NONDETERMINISTIC")
        (ROOT / ATLAS_DIR).mkdir(parents=True, exist_ok=True)
        (ROOT / ATLAS_META).parent.mkdir(parents=True, exist_ok=True)
        (ROOT / EVIDENCE_DIR).mkdir(parents=True, exist_ok=True)
        for name in ("cenyao-idle-atlas-v1.png", "cenyao-idle-atlas-v1.webp"):
            (ROOT / ATLAS_DIR / name).write_bytes((a / name).read_bytes())
        (ROOT / ATLAS_META).write_bytes((a / ATLAS_META.name).read_bytes())
        atlas = Image.open(a / "cenyao-idle-atlas-v1.png").convert("RGBA")
        preview = Image.new("RGB", atlas.size, (205, 205, 205))
        draw = ImageDraw.Draw(preview)
        for y in range(0, atlas.height, 16):
            for x in range(0, atlas.width, 16):
                if (x // 16 + y // 16) % 2:
                    draw.rectangle((x, y, x + 15, y + 15), fill=(150, 150, 150))
        preview.paste(atlas, (0, 0), atlas)
        draw = ImageDraw.Draw(preview)
        for entry in json.loads((a / ATLAS_META.name).read_text(encoding="utf-8"))["frames"]:
            x, y, w, h = entry["rect"]
            draw.rectangle((x, y, x + w - 1, y + h - 1), outline=(255, 0, 0), width=2)
            draw.text((x + 8, y + 8), entry["direction"], fill=(255, 255, 0), stroke_width=2, stroke_fill=(0, 0, 0))
        preview.save(ROOT / EVIDENCE_DIR / "cenyao-idle-atlas-qa.png", format="PNG", optimize=False, compress_level=9)
        evidence = {"builtTwiceByteIdentical": True, "outputHashes": hashes_a, "qaPreviewSha256": sha(ROOT / EVIDENCE_DIR / "cenyao-idle-atlas-qa.png")}
        (ROOT / EVIDENCE_DIR / "reproducibility.json").write_bytes((json.dumps(evidence, indent=2) + "\n").encode("utf-8"))
        return evidence


if __name__ == "__main__":
    print(json.dumps(publish(), indent=2))
