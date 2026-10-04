import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from validate_gh_catalog import CatalogError, DEFAULT_CATALOG, DEFAULT_SCENES, validate_catalog, validate_files


class GreyHiveNarrativeCatalogTests(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        self.scenes = {}
        for path in DEFAULT_SCENES.glob("*.json"):
            scene = json.loads(path.read_text(encoding="utf-8"))
            if scene["worldId"] == "grey_hive":
                self.scenes[scene["sceneId"]] = scene

    def test_production_catalog_covers_all_scenes_and_keeps_copy_gaps_explicit(self):
        result = validate_catalog(self.catalog, self.scenes)
        self.assertEqual(result["result"], "PASS")
        self.assertEqual(result["sceneCount"], 11)
        self.assertEqual(result["pendingSceneHookCount"], 1)
        self.assertEqual(result["choiceIds"], ["left", "taken", "unresolved"])
        bz = next(entry for entry in self.catalog["entries"] if entry["id"] == "gh_bz_first_01")
        self.assertEqual(bz["triggerStatus"], "pending_scene_hook")
        self.assertFalse(self.catalog["progressionConstraints"]["baizhiChoiceIsGateBKey"])

    def test_rejects_duplicate_narrative_ids(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["id"] = catalog["entries"][0]["id"]
        with self.assertRaisesRegex(CatalogError, "duplicate narrative ID"):
            validate_catalog(catalog, self.scenes)

    def test_requires_authoritative_source_hash_and_section_line_range(self):
        catalog = copy.deepcopy(self.catalog)
        catalog.pop("designSourceSha256")
        with self.assertRaisesRegex(CatalogError, "designSourceSha256"):
            validate_catalog(catalog, self.scenes)
        catalog = copy.deepcopy(self.catalog)
        catalog["designSourceSha256"] = "0" * 64
        with self.assertRaisesRegex(CatalogError, "designSourceSha256"):
            validate_catalog(catalog, self.scenes)
        catalog = copy.deepcopy(self.catalog)
        catalog.pop("sourceLineRange")
        with self.assertRaisesRegex(CatalogError, "sourceLineRange"):
            validate_catalog(catalog, self.scenes)
        catalog = copy.deepcopy(self.catalog)
        catalog["sourceLineRange"] = "612-622"
        with self.assertRaisesRegex(CatalogError, "sourceLineRange"):
            validate_catalog(catalog, self.scenes)

    def test_rejects_missing_compiled_scene_and_unbound_trigger(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["sceneId"] = "gh_missing_scene"
        with self.assertRaisesRegex(CatalogError, "unknown compiled scene"):
            validate_catalog(catalog, self.scenes)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["triggerId"] = "gh_missing_terminal"
        with self.assertRaisesRegex(CatalogError, "trigger is not present"):
            validate_catalog(catalog, self.scenes)

    def test_pending_scene_hook_must_be_absent_and_confirmed_text_must_exist(self):
        catalog = copy.deepcopy(self.catalog)
        bz = next(entry for entry in catalog["entries"] if entry["id"] == "gh_bz_first_01")
        bz["triggerId"] = "gh_bio_log_terminal"
        with self.assertRaisesRegex(CatalogError, "marked pending but its trigger now exists"):
            validate_catalog(catalog, self.scenes)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["body"] = None
        with self.assertRaisesRegex(CatalogError, "confirmed body must contain text"):
            validate_catalog(catalog, self.scenes)

    def test_choice_ids_are_legal_unique_and_never_grant_gate_b(self):
        catalog = copy.deepcopy(self.catalog)
        bz = next(entry for entry in catalog["entries"] if entry["id"] == "gh_bz_first_01")
        bz["choices"][0]["id"] = "Taken-Choice"
        with self.assertRaisesRegex(CatalogError, "illegal choice ID"):
            validate_catalog(catalog, self.scenes)
        catalog = copy.deepcopy(self.catalog)
        bz = next(entry for entry in catalog["entries"] if entry["id"] == "gh_bz_first_01")
        bz["choices"][0]["gateBKeyEffect"] = "unlocks_gate_b"
        with self.assertRaisesRegex(CatalogError, "must not grant a Gate B key"):
            validate_catalog(catalog, self.scenes)

    def test_rejects_invalid_utf8(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "invalid.json"
            path.write_bytes(b"\xff\xfe{\x00}")
            with self.assertRaisesRegex(CatalogError, "not valid UTF-8"):
                validate_files(path, DEFAULT_SCENES)


if __name__ == "__main__":
    unittest.main()
