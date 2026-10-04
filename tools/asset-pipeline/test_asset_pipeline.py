from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path
from unittest.mock import patch

from PIL import Image

MODULE_PATH = Path(__file__).with_name("asset_pipeline.py")
SPEC = importlib.util.spec_from_file_location("asset_pipeline", MODULE_PATH)
pipeline = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(pipeline)


class PipelineContracts(unittest.TestCase):
    def test_non_approved_rows_are_not_resolved_or_decoded(self) -> None:
        manifest = {"assets": [
            {"asset_id": "concept", "relative_path": "batch01/concept.png", "sha256": "0" * 64, "release_status": "concept_only"},
            {"asset_id": "reject", "relative_path": "batch01/reject.png", "sha256": "0" * 64, "release_status": "reject"},
        ]}
        with patch.object(pipeline, "pinned_blob", side_effect=AssertionError("must not read unapproved source")) as resolve:
            with self.assertRaisesRegex(pipeline.PipelineError, "expected 43 approved sources"):
                pipeline.admitted_sources(manifest)
            resolve.assert_not_called()

    def test_approved_wrong_sha_fails_closed(self) -> None:
        data = b"not-the-pinned-image"
        manifest = {"assets": [{
            "asset_id": "approved", "relative_path": "batch01/approved.png",
            "sha256": "0" * 64, "release_status": "release_approved",
        }]}
        with patch.object(pipeline, "pinned_blob", return_value=data):
            with self.assertRaisesRegex(pipeline.PipelineError, "pinned SHA mismatch"):
                pipeline.admitted_sources(manifest)

    def test_crop_trims_transparency_and_normalizes_hidden_rgb(self) -> None:
        image = Image.new("RGBA", (8, 8), (77, 66, 55, 0))
        image.putpixel((2, 3), (10, 20, 30, 255))
        image.putpixel((3, 4), (77, 66, 55, 0))
        image.putpixel((4, 5), (30, 40, 50, 128))
        import io
        stream = io.BytesIO()
        image.save(stream, format="PNG")
        crop, meta = pipeline.clean_crop(stream.getvalue())
        self.assertEqual((3, 3), crop.size)
        self.assertEqual([2, 3, 5, 6], meta["cropRect"])
        self.assertEqual((10, 20, 30, 255), crop.getpixel((0, 0)))
        self.assertEqual((0, 0, 0, 0), crop.getpixel((1, 1)))
        self.assertEqual((30, 40, 50, 128), crop.getpixel((2, 2)))

    def test_atlas_packing_is_deterministic_and_page_bounded(self) -> None:
        images = [
            ("b", Image.new("RGBA", (1200, 2500), (2, 3, 4, 255))),
            ("a", Image.new("RGBA", (1200, 2500), (5, 6, 7, 255))),
            ("c", Image.new("RGBA", (700, 800), (8, 9, 10, 255))),
        ]
        first, first_places = pipeline.pack(images)
        second, second_places = pipeline.pack(list(reversed(images)))
        self.assertEqual(first_places, second_places)
        self.assertEqual([atlas.size for atlas in first], [atlas.size for atlas in second])
        self.assertTrue(all(width <= 4096 and height <= 4096 for width, height in (atlas.size for atlas in first)))

    def test_cenyao_idle_metadata_and_source_frames_are_pinned(self) -> None:
        manifest = pipeline.load_authority()
        approved = {row["asset_id"]: row for row in manifest["assets"] if row["release_status"] == "release_approved"}
        b6a, frames = pipeline.load_cenyao_idle_metadata(pipeline.REPO, approved)
        self.assertEqual(list(pipeline.CENYAO_IDLE_DIRECTIONS), [frame["direction"] for frame in frames])
        self.assertEqual([1360, 1128], b6a["atlas"]["size"])
        self.assertEqual(8, len(frames))
        with patch.object(pipeline, "CENYAO_IDLE_METADATA_SHA256", "0" * 64):
            with self.assertRaisesRegex(pipeline.PipelineError, "B6a metadata SHA mismatch"):
                pipeline.load_cenyao_idle_metadata(pipeline.REPO, approved)

    def test_category_budget_negative_cases_do_not_scale_or_pack(self) -> None:
        self.assertTrue(pipeline.budget_exceeded("actor", 4097, 1))
        self.assertTrue(pipeline.budget_exceeded("ui", 2049, 1))
        self.assertTrue(pipeline.budget_exceeded("world", 1, 4097))
        self.assertFalse(pipeline.budget_exceeded("ui", 2048, 2048))
        self.assertEqual(("boss", "sentinel"), pipeline.category_and_group("runtime2d.enemy.sentinel.base.v1"))
        self.assertEqual(("ui", "shared"), pipeline.category_and_group("runtime2d.ui.capability.local_map.v1"))
        oversized = [("too-tall", Image.new("RGBA", (200, 4097), (1, 1, 1, 255)))]
        with self.assertRaisesRegex(pipeline.PipelineError, "exceeds atlas budget"):
            pipeline.pack(oversized, max_side=4096)

    def test_webp_encoder_version_mismatch_fails_before_build(self) -> None:
        with patch.object(pipeline.features, "version", return_value="9.9.9"):
            with self.assertRaisesRegex(pipeline.PipelineError, "WebP encoder version mismatch"):
                pipeline.build()


if __name__ == "__main__":
    unittest.main()
