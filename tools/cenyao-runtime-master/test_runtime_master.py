from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

from PIL import Image, ImageChops

import build_runtime_master as master


class CenyaoRuntimeMasterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source, cls.source_row, cls.parts_row = master.load_sources()
        cls.records = master.analyze(cls.source)
        cls.metadata = json.loads((master.ROOT / master.METADATA_PATH).read_text(encoding="utf-8"))
        cls.repro = json.loads((master.ROOT / master.ACCEPTANCE_DIR / "reproducibility.json").read_text(encoding="utf-8"))

    def file_sha(self, relative_path: str) -> str:
        hasher = hashlib.sha256()
        with (master.ROOT / relative_path).open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                hasher.update(chunk)
        return hasher.hexdigest()

    def test_approved_source_sha_and_reference_provenance(self):
        self.assertEqual(master.digest(master.ROOT / master.AUTHORITY_REL), master.AUTHORITY_SHA256)
        self.assertEqual(master.digest(master.ROOT / master.SOURCE_REL), master.SOURCE_SHA256)
        self.assertEqual(master.digest(master.ROOT / master.DECISION_DOC_REL), master.DECISION_DOC_SHA256)
        self.assertEqual(self.source_row["release_status"], "release_approved")
        self.assertEqual(self.parts_row["release_status"], "release_approved")
        self.assertEqual(self.metadata["source"]["sha256"], master.SOURCE_SHA256)
        self.assertEqual(self.metadata["b0Authority"]["sha256"], master.AUTHORITY_SHA256)
        self.assertEqual(self.metadata["partsReference"]["usage"].split(";")[0], "reference-only")

    def test_eight_direction_rows_are_complete_and_source_cells_are_isolated(self):
        self.assertEqual(len(self.records), 8)
        self.assertEqual({record["direction"] for record in self.records}, master.EXPECTED_DIRECTIONS)
        self.assertEqual(len({(record["row"], record["column"]) for record in self.records}), 8)
        mapping = {(record["row"], record["column"]): record["direction"] for record in self.records}
        self.assertEqual(mapping[(0, 1)], "south_west")
        self.assertEqual(mapping[(0, 2)], "west")
        self.assertEqual(mapping[(1, 2)], "east")
        self.assertEqual(mapping[(1, 3)], "south_east")
        for record in self.records:
            x, y, w, h = record["cellRect"]
            cx, cy, cw, ch = record["sourceAlphaBounds"]
            self.assertGreaterEqual(cx, x)
            self.assertGreaterEqual(cy, y)
            self.assertLessEqual(cx + cw, x + w)
            self.assertLessEqual(cy + ch, y + h)
            self.assertEqual(record["edgeAlphaPixelCount"], 0, record["direction"])
            self.assertGreater(record["alphaPixelCount"], 10_000)
            self.assertTrue(record["footBandSourceRect"])

    def test_frames_preserve_alpha_shape_and_feet_anchor_without_clipping(self):
        frames = {frame["direction"]: frame for frame in self.metadata["frames"]}
        self.assertEqual(set(frames), master.EXPECTED_DIRECTIONS)
        png_hashes = set()
        webp_hashes = set()
        for record in self.records:
            direction = record["direction"]
            frame_meta = frames[direction]
            png_path = master.ROOT / master.PNG_DIR / f"cenyao_idle_{direction}.png"
            webp_path = master.ROOT / master.WEBP_DIR / f"cenyao_idle_{direction}.webp"
            self.assertEqual(frame_meta["outputs"]["png"]["path"], f"{master.PNG_DIR.as_posix()}/cenyao_idle_{direction}.png")
            self.assertEqual(frame_meta["outputs"]["webp"]["path"], f"{master.WEBP_DIR.as_posix()}/cenyao_idle_{direction}.webp")
            png = Image.open(png_path).convert("RGBA")
            webp = Image.open(webp_path).convert("RGBA")
            self.assertEqual(png.size, tuple(frame_meta["outputSize"]))
            self.assertLessEqual(png.width, 1024)
            self.assertLessEqual(png.height, 1024)
            self.assertEqual(png.size, webp.size)
            color_diff = ImageChops.difference(png.convert("RGB"), webp.convert("RGB"))
            alpha_diff = ImageChops.difference(png.getchannel("A"), webp.getchannel("A"))
            self.assertIsNone(color_diff.getbbox(), direction)
            self.assertIsNone(alpha_diff.getbbox(), direction)
            alpha = png.getchannel("A")
            self.assertEqual(alpha.getextrema()[0], 0)
            self.assertIsNone(alpha.point(lambda value: 255 if value == 1 else 0).getbbox())
            bbox = alpha.getbbox()
            self.assertIsNotNone(bbox)
            self.assertGreaterEqual(bbox[0], master.MARGIN)
            self.assertGreaterEqual(bbox[1], master.MARGIN)
            self.assertLessEqual(bbox[2], png.width - master.MARGIN)
            self.assertLessEqual(bbox[3], png.height - master.MARGIN)
            self.assertAlmostEqual(frame_meta["anchor"]["y"], frame_meta["anchor"]["pixel"][1] / png.height)
            self.assertGreater(frame_meta["anchor"]["pixel"][1], bbox[3] - 2)
            self.assertEqual(frame_meta["sourceAlphaPreserved"], "alpha 2..255 exact; alpha 0..1 removed as border residue")
            self.assertEqual(self.file_sha(f"{master.PNG_DIR.as_posix()}/{png_path.name}"), frame_meta["outputs"]["png"]["sha256"])
            self.assertEqual(self.file_sha(f"{master.WEBP_DIR.as_posix()}/{webp_path.name}"), frame_meta["outputs"]["webp"]["sha256"])
            png_hashes.add(frame_meta["outputs"]["png"]["sha256"])
            webp_hashes.add(frame_meta["outputs"]["webp"]["sha256"])

            # Re-run the exact cell crop/translation in memory and compare every
            # RGBA sample, proving neither adjacent cells nor resize artifacts
            # entered the delivered frame.
            expected, _, _ = master.prepare_crop(record, png.size)
            self.assertIsNone(ImageChops.difference(png.convert("RGB"), expected.convert("RGB")).getbbox(), direction)
            self.assertIsNone(ImageChops.difference(png.getchannel("A"), expected.getchannel("A")).getbbox(), direction)
            cx, cy, cw, ch = frame_meta["sourceCropRect"]
            source_crop = self.source.crop((cx, cy, cx + cw, cy + ch))
            source_crop.putalpha(source_crop.getchannel("A").point(lambda value: 0 if value <= 1 else value))
            source_pixels = list(source_crop.get_flattened_data())
            source_crop.putdata([(0, 0, 0, 0) if pixel[3] == 0 else pixel for pixel in source_pixels])
            px, py = frame_meta["contentPlacement"]
            delivered_content = png.crop((px, py, px + cw, py + ch))
            self.assertIsNone(ImageChops.difference(delivered_content.convert("RGB"), source_crop.convert("RGB")).getbbox(), direction)
            self.assertIsNone(ImageChops.difference(delivered_content.getchannel("A"), source_crop.getchannel("A")).getbbox(), direction)
        self.assertEqual(len(png_hashes), 8)
        self.assertEqual(len(webp_hashes), 8)

    def test_two_independent_builds_are_byte_identical(self):
        self.assertTrue(self.repro["builtTwiceByteIdentical"])
        self.assertEqual(self.repro["frameCount"], 8)
        log_dir = master.ROOT / master.ACCEPTANCE_DIR
        build_one = json.loads((log_dir / "build-1.txt").read_text(encoding="utf-8"))
        build_two = json.loads((log_dir / "build-2.txt").read_text(encoding="utf-8"))
        self.assertEqual(build_one, build_two)
        self.assertTrue(build_one["builtTwiceByteIdentical"])
        self.assertEqual(sum(frame["removedAlphaOnePixelCount"] for frame in self.metadata["frames"]), 33_669)
        for relative, expected_sha in self.repro["outputHashes"].items():
            if relative.startswith("png/"):
                actual = self.file_sha(f"{master.PNG_DIR.as_posix()}/{relative.split('/', 1)[1]}")
            elif relative.startswith("webp/"):
                actual = self.file_sha(f"{master.WEBP_DIR.as_posix()}/{relative.split('/', 1)[1]}")
            elif relative == "cenyao-runtime-master-v1.json":
                actual = self.file_sha(master.METADATA_PATH.as_posix())
            else:
                actual = self.file_sha(f"{master.ACCEPTANCE_DIR.as_posix()}/{relative}")
            self.assertEqual(actual, expected_sha, relative)

    def test_transparent_and_labeled_qa_previews_exist(self):
        acceptance = master.ROOT / master.ACCEPTANCE_DIR
        with Image.open(acceptance / "cenyao-8way-contact-sheet.png") as contact:
            self.assertEqual(contact.mode, "RGB")
        with Image.open(acceptance / "cenyao-8way-transparent-preview.png") as source_preview:
            transparent = source_preview.convert("RGBA")
        self.assertEqual(transparent.getchannel("A").getextrema()[0], 0)
        self.assertIsNotNone(transparent.getchannel("A").getbbox())


if __name__ == "__main__":
    unittest.main()
