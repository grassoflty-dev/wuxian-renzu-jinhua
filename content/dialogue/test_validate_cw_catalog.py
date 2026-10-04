import copy
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from validate_cw_catalog import (  # noqa: E402
    CatalogError,
    DEFAULT_CATALOG,
    DEFAULT_PROGRESSION,
    DEFAULT_RELEASE,
    DEFAULT_RUNTIME,
    DEFAULT_SCENES,
    load_approved_assets,
    load_required_events,
    validate_catalog,
    validate_files,
)


class ClockworksNarrativeCatalogTests(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        self.scenes = {}
        for path in DEFAULT_SCENES.glob("cw_*.json"):
            scene = json.loads(path.read_text(encoding="utf-8"))
            self.scenes[scene["sceneId"]] = scene
        self.required_events = load_required_events(DEFAULT_PROGRESSION)
        self.runtime_assets, self.approved_assets = load_approved_assets(DEFAULT_RUNTIME, DEFAULT_RELEASE)

    def validate(self, catalog=None, scenes=None, required=None, runtime=None, approved=None):
        return validate_catalog(
            catalog if catalog is not None else self.catalog,
            scenes if scenes is not None else self.scenes,
            required if required is not None else self.required_events,
            runtime if runtime is not None else self.runtime_assets,
            approved if approved is not None else self.approved_assets,
        )

    def test_production_catalog_covers_all_scenes_and_preserves_only_source_confirmed_lines(self):
        result = validate_files()
        self.assertEqual(result["result"], "PASS")
        self.assertEqual(result["sceneCount"], 9)
        self.assertEqual(result["entryCount"], 19)
        self.assertEqual(result["confirmedBodyCount"], 3)
        self.assertEqual(result["pendingBodyCount"], 16)
        self.assertEqual(result["boundTriggerCount"], 1)
        self.assertEqual(result["pendingProgressionEventCount"], 3)
        confirmed = {entry["id"]: entry["body"] for entry in self.catalog["entries"]
                     if entry["bodyStatus"] == "confirmed"}
        self.assertEqual(confirmed, {
            "cw_sys_entry": "生产线无人值守，但压力循环仍在运行。",
            "cw_cy_heat_01": "炉心不是失控——它被维持在过载边缘。",
            "cw_cy_end_01": "原来门一直不止三扇。",
        })

    def test_rejects_duplicate_narrative_ids_and_duplicate_scene_references(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["id"] = catalog["entries"][0]["id"]
        with self.assertRaisesRegex(CatalogError, "duplicate narrative ID"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["scenes"].append(catalog["scenes"][0])
        with self.assertRaisesRegex(CatalogError, "duplicate scene reference"):
            self.validate(catalog=catalog)

    def test_rejects_scene_or_marker_references_that_do_not_match_compiled_content(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["sceneId"] = "cw_missing_scene"
        with self.assertRaisesRegex(CatalogError, "unknown compiled scene"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["triggerId"] = "cw_missing_marker"
        with self.assertRaisesRegex(CatalogError, "staged marker is missing"):
            self.validate(catalog=catalog)

    def test_confirmed_copy_must_match_the_reported_source_line(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["body"] = "生产线仍在运转。"
        with self.assertRaisesRegex(CatalogError, "confirmed body does not match source text"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["sourceLine"] = 694
        with self.assertRaisesRegex(CatalogError, "confirmed body does not match source text"):
            self.validate(catalog=catalog)

    def test_pending_copy_must_remain_empty_and_staged_markers_must_be_eventless(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["body"] = "三阀均已复位。"
        with self.assertRaisesRegex(CatalogError, "pending body must be null"):
            self.validate(catalog=catalog)
        scenes = copy.deepcopy(self.scenes)
        staged = next(item for item in scenes["cw_pressure_hall"]["interactions"]
                      if item["id"] == "cw_clockworks_valves_staged")
        staged["event"] = "clockworks_valves"
        with self.assertRaisesRegex(CatalogError, "eventless staged marker"):
            self.validate(scenes=scenes)

    def test_progression_event_refs_must_match_catalog_and_remain_pending(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["progressionEvents"][0]["markerId"] = "cw_pressure_valve_01_staged"
        with self.assertRaisesRegex(CatalogError, "exact eventless staged marker"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["relatedEventId"] = "clockworks_unknown"
        with self.assertRaisesRegex(CatalogError, "unknown progression event"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["progressionEvents"][0]["status"] = "complete"
        with self.assertRaisesRegex(CatalogError, "must remain pending"):
            self.validate(catalog=catalog)

    def test_compiled_scene_asset_refs_must_exist_and_be_release_approved(self):
        scenes = copy.deepcopy(self.scenes)
        scenes["cw_shutdown_exit"]["presentation"]["sprites"].append({
            "id": "unapproved_test_asset", "assetId": "runtime2d.world.clockworks.concept_only",
        })
        with self.assertRaisesRegex(CatalogError, "unknown runtime assets|unapproved assets"):
            self.validate(scenes=scenes)

    def test_emitted_event_cannot_hide_behind_a_pending_catalog_status(self):
        scenes = copy.deepcopy(self.scenes)
        marker = next(item for item in scenes["cw_regulator_core"]["interactions"]
                      if item["id"] == "cw_regulator_core_console_staged")
        marker["event"] = "clockworks_core"
        with self.assertRaisesRegex(CatalogError, "eventless staged marker"):
            self.validate(scenes=scenes)


if __name__ == "__main__":
    unittest.main()
