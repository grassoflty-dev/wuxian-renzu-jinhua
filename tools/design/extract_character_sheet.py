#!/usr/bin/env python3
"""Extract three same-character poses from a magenta-backed horizontal sheet.

Usage:
  python extract_character_sheet.py INPUT OUTPUT_DIR

The source is split into equal thirds. Magenta connected to the image edge is
removed, each figure is tightly cropped, then placed on a shared transparent
512x768 canvas with an identical foot baseline.
"""

from __future__ import annotations

import sys
from collections import deque
from pathlib import Path

from PIL import Image, ImageFilter


def magenta(pixel: tuple[int, int, int, int]) -> bool:
    r, g, b, _ = pixel
    return r > 165 and b > 115 and g < 175 and r - g > 45 and b - g > 25


def remove_edge_background(image: Image.Image) -> Image.Image:
    rgba = image.convert("RGBA")
    pixels = rgba.load()
    width, height = rgba.size
    seen = bytearray(width * height)
    queue: deque[tuple[int, int]] = deque()

    def enqueue(x: int, y: int) -> None:
        index = y * width + x
        if seen[index] or not magenta(pixels[x, y]):
            return
        seen[index] = 1
        queue.append((x, y))

    for x in range(width):
        enqueue(x, 0)
        enqueue(x, height - 1)
    for y in range(height):
        enqueue(0, y)
        enqueue(width - 1, y)

    while queue:
        x, y = queue.popleft()
        pixels[x, y] = (0, 0, 0, 0)
        if x:
            enqueue(x - 1, y)
        if x + 1 < width:
            enqueue(x + 1, y)
        if y:
            enqueue(x, y - 1)
        if y + 1 < height:
            enqueue(x, y + 1)

    # Remove antialiased chroma remnants globally. The character palette has no
    # saturated magenta, so this does not erase authored clothing or skin.
    for y in range(height):
        for x in range(width):
            r, g, b, a = pixels[x, y]
            chroma = min(r - g, b - g)
            if a and r > 120 and b > 90 and chroma > 28:
                if chroma >= 70:
                    pixels[x, y] = (r, g, b, 0)
                else:
                    pixels[x, y] = (r, g, b, round(a * (70 - chroma) / 42))

    alpha = rgba.getchannel("A")
    # A one-pixel soft edge avoids a hard chroma fringe after scaling.
    alpha = alpha.filter(ImageFilter.GaussianBlur(0.35))
    rgba.putalpha(alpha)
    return rgba


def keep_largest_component(image: Image.Image) -> Image.Image:
    """Drop disconnected fragments belonging to a neighbouring sheet pose."""
    rgba = image.copy()
    alpha = rgba.getchannel("A")
    width, height = rgba.size
    mask = alpha.load()
    seen = bytearray(width * height)
    components: list[list[tuple[int, int]]] = []
    for sy in range(height):
        for sx in range(width):
            index = sy * width + sx
            if seen[index] or mask[sx, sy] < 18:
                continue
            seen[index] = 1
            queue = deque([(sx, sy)])
            component: list[tuple[int, int]] = []
            while queue:
                x, y = queue.popleft()
                component.append((x, y))
                for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    if nx < 0 or ny < 0 or nx >= width or ny >= height:
                        continue
                    ni = ny * width + nx
                    if seen[ni] or mask[nx, ny] < 18:
                        continue
                    seen[ni] = 1
                    queue.append((nx, ny))
            components.append(component)
    if not components:
        return rgba
    keep = set(max(components, key=len))
    pixels = rgba.load()
    for y in range(height):
        for x in range(width):
            if mask[x, y] >= 18 and (x, y) not in keep:
                pixels[x, y] = (0, 0, 0, 0)
    return rgba


def normalize_pose(image: Image.Image) -> Image.Image:
    bbox = image.getbbox()
    if not bbox:
        raise ValueError("no foreground figure found")
    figure = image.crop(bbox)
    target_w, target_h = 512, 768
    max_w, max_h = 420, 716
    scale = min(max_w / figure.width, max_h / figure.height)
    size = (max(1, round(figure.width * scale)), max(1, round(figure.height * scale)))
    figure = figure.resize(size, Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (target_w, target_h), (0, 0, 0, 0))
    x = (target_w - figure.width) // 2
    y = 744 - figure.height
    canvas.alpha_composite(figure, (x, y))
    return canvas


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: extract_character_sheet.py INPUT OUTPUT_DIR", file=sys.stderr)
        return 2
    source = Image.open(sys.argv[1]).convert("RGBA")
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(parents=True, exist_ok=True)
    names = ("cenyao-side-idle.png", "cenyao-side-run-a.png", "cenyao-side-run-b.png")
    third = source.width / 3
    for index, name in enumerate(names):
        left = round(index * third)
        right = round((index + 1) * third)
        pose = source.crop((left, 0, right, source.height))
        result = normalize_pose(keep_largest_component(remove_edge_background(pose)))
        result.save(output_dir / name, optimize=True)
        print(f"{name}: {result.width}x{result.height} {result.mode}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
