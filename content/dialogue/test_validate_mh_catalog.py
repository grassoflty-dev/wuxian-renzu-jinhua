import copy
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from validate_mh_catalog import (  # noqa: E402
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


class MistHarborNarrativeCatalogTests(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        self.scenes = {}
        for path in DEFAULT_SCENES.glob("mh_*.json"):
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

    def test_catalog_covers_all_scenes_and_only_confirms_source_text(self):
        result = validate_files()
        self.assertEqual(result["result"], "PASS")
        self.assertEqual(result["sceneCount"], 9)
        self.assertEqual(result["entryCount"], 17)
        self.assertEqual(result["confirmedBodyCount"], 5)
        self.assertEqual(result["pendingBodyCount"], 12)
        self.assertEqual(result["pendingSceneHookCount"], 1)
        self.assertEqual(result["compiledEventEntryCount"], 3)
        self.assertFalse(result["nativePlayableClaimed"])
        self.assertFalse(result["nativeCapabilityGrantClaimed"])

    def test_rejects_duplicate_narrative_ids_and_scene_references(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["id"] = catalog["entries"][0]["id"]
        with self.assertRaisesRegex(CatalogError, "duplicate narrative ID"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["scenes"].append(catalog["scenes"][0])
        with self.assertRaisesRegex(CatalogError, "duplicate scene reference"):
            self.validate(catalog=catalog)

    def test_rejects_incomplete_compiled_scene_coverage(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["scenes"].remove("mh_extraction")
        with self.assertRaisesRegex(CatalogError, "scene coverage mismatch"):
            self.validate(catalog=catalog)

    def test_confirmed_copy_must_match_exact_source_text_and_line(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["body"] = "远端目标丢失。"
        with self.assertRaisesRegex(CatalogError, "confirmed body does not match source text"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["sourceLine"] = 649
        with self.assertRaisesRegex(CatalogError, "confirmed body does not match source text"):
            self.validate(catalog=catalog)

    def test_pending_copy_must_remain_null(self):
        catalog = copy.deepcopy(self.catalog)
        pending = next(row for row in catalog["entries"] if row["bodyStatus"] == "pending")
        pending["body"] = "已成功启动。"
        with self.assertRaisesRegex(CatalogError, "pending body must be null"):
            self.validate(catalog=catalog)

    def test_staged_marker_must_be_exact_and_eventless(self):
        catalog = copy.deepcopy(self.catalog)
        entry = next(row for row in catalog["entries"] if row["id"] == "mh_cy_signal_01")
        entry["triggerKind"] = "terminal"
        with self.assertRaisesRegex(CatalogError, "exact eventless interaction"):
            self.validate(catalog=catalog)
        scenes = copy.deepcopy(self.scenes)
        marker = next(row for row in scenes["mh_signal_yard"]["interactions"]
                      if row["id"] == "mh_cy_signal_01_static_marker")
        marker["event"] = "unexpected_event"
        with self.assertRaisesRegex(CatalogError, "exact eventless interaction"):
            self.validate(scenes=scenes)

    def test_pending_scene_hook_must_be_missing_and_may_carry_confirmed_text(self):
        catalog = copy.deepcopy(self.catalog)
        entry = next(row for row in catalog["entries"] if row["id"] == "mh_sys_beacon_sync")
        self.assertEqual(entry["bodyStatus"], "confirmed")
        scenes = copy.deepcopy(self.scenes)
        scenes["mh_breakwater"]["interactions"].append({
            "id": "mh_sys_beacon_sync", "kind": "system_message", "event": None,
        })
        with self.assertRaisesRegex(CatalogError, "pending scene hook unexpectedly exists"):
            self.validate(catalog=catalog, scenes=scenes)

    def test_progression_events_match_compiled_interactions_without_playability_claims(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["progressionEvents"][0]["triggerId"] = "mh_west_beacon_log_marker"
        with self.assertRaisesRegex(CatalogError, "exact authored compiled interaction"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["progressionEvents"][0]["nativePlayableClaimed"] = True
        with self.assertRaisesRegex(CatalogError, "without a native-playability claim"):
            self.validate(catalog=catalog)

    def test_progression_event_set_must_match_world_catalog(self):
        with self.assertRaisesRegex(CatalogError, "do not match Mist Harbor required events"):
            self.validate(required={"mist_beacon_west"})

    def test_acoustic_mapping_stays_staged_and_unimplemented(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["capabilities"][0]["nativeGrantClaimed"] = True
        with self.assertRaisesRegex(CatalogError, "without a native grant claim"):
            self.validate(catalog=catalog)
        scenes = copy.deepcopy(self.scenes)
        marker = next(row for row in scenes["mh_resonance_tower"]["interactions"]
                      if row["id"] == "mh_acoustic_mapping_staged_marker")
        marker["event"] = "acoustic_mapping_granted"
        with self.assertRaisesRegex(CatalogError, "exact eventless staged marker"):
            self.validate(scenes=scenes)

    def test_rejects_unknown_related_events_and_unapproved_assets(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["relatedEventIds"] = ["unknown_event"]
        with self.assertRaisesRegex(CatalogError, "unknown progression event"):
            self.validate(catalog=catalog)
        scenes = copy.deepcopy(self.scenes)
        scenes["mh_fog_pier"]["presentation"]["sprites"].append({
            "id": "test_asset", "assetId": "runtime2d.world.mistharbor.not_approved",
        })
        with self.assertRaisesRegex(CatalogError, "unknown runtime assets|unapproved assets"):
            self.validate(scenes=scenes)


if __name__ == "__main__":
    unittest.main()
