from __future__ import annotations

import json
import unittest

from PIL import Image

import build_idle_atlas as atlas_tool


class IdleAtlasTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.meta = json.loads((atlas_tool.ROOT / atlas_tool.ATLAS_META).read_text(encoding="utf-8"))
        cls.master = json.loads((atlas_tool.ROOT / atlas_tool.MASTER_META).read_text(encoding="utf-8"))
        cls.png = Image.open(atlas_tool.ROOT / cls.meta["atlas"]["png"]["path"]).convert("RGBA")
        cls.webp = Image.open(atlas_tool.ROOT / cls.meta["atlas"]["webp"]["path"]).convert("RGBA")

    def test_source_and_atlas_hashes(self):
        self.assertEqual(atlas_tool.sha(atlas_tool.ROOT / atlas_tool.MASTER_META), self.meta["sourceMaster"]["sha256"])
        for fmt in ("png", "webp"):
            row = self.meta["atlas"][fmt]
            self.assertEqual(atlas_tool.sha(atlas_tool.ROOT / row["path"]), row["sha256"])
        for entry in self.meta["frames"]:
            for fmt in ("png", "webp"):
                row = entry["sourceFrame"][fmt]
                self.assertEqual(atlas_tool.sha(atlas_tool.ROOT / row["path"]), row["sha256"])

    def test_geometry_budget_directions_and_no_overlap(self):
        self.assertEqual(tuple(entry["direction"] for entry in self.meta["frames"]), atlas_tool.ORDER)
        self.assertEqual(self.png.size, tuple(self.meta["atlas"]["size"]))
        self.assertLessEqual(max(self.png.size), 4096)
        self.assertEqual(self.meta["atlas"]["extrusionPixels"], 2)
        seen = set()
        for entry in self.meta["frames"]:
            x, y, w, h = entry["rect"]
            ox, oy, ow, oh = entry["extrudedRect"]
            self.assertEqual((ox, oy, ow, oh), (x - 2, y - 2, w + 4, h + 4))
            self.assertGreaterEqual(ox, 0)
            self.assertGreaterEqual(oy, 0)
            self.assertLessEqual(ox + ow, self.png.width)
            self.assertLessEqual(oy + oh, self.png.height)
            for cy in range(oy, oy + oh):
                for cx in range(ox, ox + ow):
                    self.assertNotIn((cx, cy), seen)
                    seen.add((cx, cy))

    def test_pixels_anchor_and_extrusion(self):
        self.assertEqual(self.png.tobytes(), self.webp.tobytes())
        originals = {frame["direction"]: frame for frame in self.master["frames"]}
        for entry in self.meta["frames"]:
            x, y, w, h = entry["rect"]
            original = originals[entry["direction"]]
            frame = Image.open(atlas_tool.ROOT / original["outputs"]["png"]["path"]).convert("RGBA")
            self.assertEqual(self.png.crop((x, y, x + w, y + h)).tobytes(), frame.tobytes())
            self.assertEqual(entry["anchor"], original["anchor"])
            self.assertEqual(entry["sourceCell"], original["sourceCell"])
            for row in range(h):
                self.assertEqual(self.png.getpixel((x - 1, y + row)), frame.getpixel((0, row)))
                self.assertEqual(self.png.getpixel((x - 2, y + row)), frame.getpixel((0, row)))
                self.assertEqual(self.png.getpixel((x + w, y + row)), frame.getpixel((w - 1, row)))
                self.assertEqual(self.png.getpixel((x + w + 1, y + row)), frame.getpixel((w - 1, row)))
            for col in range(w):
                self.assertEqual(self.png.getpixel((x + col, y - 1)), frame.getpixel((col, 0)))
                self.assertEqual(self.png.getpixel((x + col, y + h)), frame.getpixel((col, h - 1)))

    def test_reproducibility_record_and_preview(self):
        record = json.loads((atlas_tool.ROOT / atlas_tool.EVIDENCE_DIR / "reproducibility.json").read_text(encoding="utf-8"))
        self.assertTrue(record["builtTwiceByteIdentical"])
        for name, expected in record["outputHashes"].items():
            path = atlas_tool.ROOT / (atlas_tool.ATLAS_META if name.endswith(".json") else atlas_tool.ATLAS_DIR / name)
            self.assertEqual(atlas_tool.sha(path), expected)
        preview = atlas_tool.ROOT / atlas_tool.EVIDENCE_DIR / "cenyao-idle-atlas-qa.png"
        self.assertEqual(atlas_tool.sha(preview), record["qaPreviewSha256"])
        with Image.open(preview) as image:
            self.assertEqual(image.size, self.png.size)


if __name__ == "__main__":
    unittest.main()
