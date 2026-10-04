#!/usr/bin/env python3
"""Deterministic format/size conversion of the approved original PNG, no artwork synthesis."""
from __future__ import annotations
import argparse, hashlib, io, json, struct
from pathlib import Path
from PIL import Image, __version__ as PILLOW_VERSION
ROOT = Path(__file__).resolve().parents[2]
SOURCE = "server-rs/icons/source/chatgpt-portal-emblem-20261003.png"
SOURCE_SHA256 = "ba7bb68b31b53ebe390b29eedf3bc4cda2ef9f42129b1cb35f3afb70db06cd21"
OUTPUT = "server-rs/icons/icon.ico"
# Tauri codegen 2.6.3 decodes entries()[0] for its default window RGBA.
# Put the largest source-derived frame first; Windows still has all small sizes.
SIZES = (256, 128, 64, 48, 32, 24, 16)
PILLOW_PIN = "12.3.0"

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def render(source: bytes) -> tuple[bytes, dict]:
    if PILLOW_VERSION != PILLOW_PIN:
        raise ValueError("E_ICON_PILLOW_VERSION")
    if sha(source) != SOURCE_SHA256:
        raise ValueError("E_ICON_SOURCE_SHA")
    with Image.open(io.BytesIO(source)) as opened:
        if opened.format != "PNG" or opened.mode != "RGBA" or opened.size != (1254, 1254):
            raise ValueError("E_ICON_SOURCE_FORMAT")
        image = opened.copy()
    alpha = image.getchannel("A")
    if alpha.getextrema() != (0, 255):
        raise ValueError("E_ICON_SOURCE_ALPHA")
    offset = 6 + 16 * len(SIZES)
    entries, payloads, frames = [], [], []
    for size in SIZES:
        frame = image.resize((size, size), Image.Resampling.LANCZOS)
        stream = io.BytesIO()
        frame.save(stream, format="PNG", optimize=False, compress_level=9)
        payload = stream.getvalue()
        entries.append(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(payload), offset))
        frames.append({"size": [size, size], "encoding": "PNG", "bytes": len(payload),
                       "sha256": sha(payload), "rgbaSha256": sha(frame.tobytes()),
                       "alphaExtrema": list(frame.getchannel("A").getextrema()), "offset": offset})
        payloads.append(payload)
        offset += len(payload)
    ico = struct.pack("<HHH", 0, 1, len(SIZES)) + b"".join(entries) + b"".join(payloads)
    return ico, {"sourcePath": SOURCE, "sourceSha256": SOURCE_SHA256, "sourceBytes": len(source),
                 "sourceSize": [1254, 1254], "sourceMode": "RGBA", "outputPath": OUTPUT,
                 "bytes": len(ico), "sha256": sha(ico), "frames": frames,
                 "transformation": "Pillow RGBA LANCZOS square resize only; no crop, recolor, added text, font or other art; deterministic PNG frames packed in ICO",
                 "pillowVersion": PILLOW_VERSION, "defaultWindowIconSize": [256, 256],
                 "icoFrameOrder": "largest-first; Tauri codegen 2.6.3 consumes first entry"}

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="verify existing ICO bytes without writing")
    args = parser.parse_args()
    ico, report = render((ROOT / SOURCE).read_bytes())
    if args.check:
        if (ROOT / OUTPUT).read_bytes() != ico:
            raise ValueError("E_ICON_OUTPUT_BYTES")
    else:
        (ROOT / OUTPUT).write_bytes(ico)
    print(json.dumps(report, indent=2))

if __name__ == "__main__":
    main()
