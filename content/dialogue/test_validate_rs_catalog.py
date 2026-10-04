import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from validate_rs_catalog import (  # noqa: E402
    CatalogError,
    DEFAULT_CATALOG,
    DEFAULT_PROGRESSION,
    DEFAULT_RELEASE,
    DEFAULT_RUNTIME,
    DEFAULT_SCENES,
    load_approved_assets,
    load_compiled_scenes,
    load_progression,
    validate_catalog,
    validate_files,
)


class ReturnStationNarrativeCatalogTests(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        self.scene = json.loads((DEFAULT_SCENES / "rs_core_room.json").read_text(encoding="utf-8"))
        self.progression = load_progression(DEFAULT_PROGRESSION)
        self.runtime_assets, self.approved_assets = load_approved_assets(DEFAULT_RUNTIME, DEFAULT_RELEASE)

    def validate(self, catalog=None, scene=None, progression=None, runtime=None, approved=None):
        return validate_catalog(
            catalog if catalog is not None else self.catalog,
            scene if scene is not None else self.scene,
            progression if progression is not None else self.progression,
            runtime if runtime is not None else self.runtime_assets,
            approved if approved is not None else self.approved_assets,
        )

    def test_catalog_has_four_confirmed_lines_and_two_pending_bodies(self):
        result = validate_files()
        self.assertEqual(result["result"], "PASS")
        self.assertEqual(result["sceneCount"], 1)
        self.assertEqual(result["entryCount"], 6)
        self.assertEqual(result["confirmedBodyCount"], 4)
        self.assertEqual(result["pendingBodyCount"], 2)
        self.assertEqual(result["stagedMarkerCount"], 1)
        self.assertEqual(result["pendingSceneHookCount"], 5)
        self.assertFalse(result["mistHarborReturnAvailable"])
        self.assertFalse(result["clockworksReturnAvailable"])
        self.assertFalse(result["threeWorldEpiloguePlayable"])

    def test_rejects_duplicate_ids_and_missing_narrative_nodes(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["id"] = catalog["entries"][0]["id"]
        with self.assertRaisesRegex(CatalogError, "duplicate narrative ID"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"].pop()
        with self.assertRaisesRegex(CatalogError, "entry set does not cover"):
            self.validate(catalog=catalog)

    def test_requires_exact_compiled_core_room_scene(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["scenes"] = ["rs_missing"]
        with self.assertRaisesRegex(CatalogError, "exactly the compiled rs_core_room"):
            self.validate(catalog=catalog)
        scene = copy.deepcopy(self.scene)
        scene["worldId"] = "grey_hive"
        with tempfile.TemporaryDirectory() as temp_dir:
            Path(temp_dir, "rs_core_room.json").write_text(json.dumps(scene), encoding="utf-8")
            with self.assertRaisesRegex(CatalogError, "identity or schema"):
                load_compiled_scenes(Path(temp_dir))

    def test_confirmed_copy_requires_exact_source_wording_and_line(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][0]["body"] = "归航链路已稳定。"
        with self.assertRaisesRegex(CatalogError, "confirmed body does not match source text"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["sourceLine"] = 724
        with self.assertRaisesRegex(CatalogError, "confirmed body does not match source text"):
            self.validate(catalog=catalog)

    def test_pending_nodes_must_not_invent_dialogue(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][-1]["body"] = "三个世界已经归航。"
        with self.assertRaisesRegex(CatalogError, "pending body must be null"):
            self.validate(catalog=catalog)

    def test_capability_terminal_must_match_exact_eventless_marker(self):
        catalog = copy.deepcopy(self.catalog)
        entry = next(row for row in catalog["entries"] if row["id"] == "rs_first_evolution")
        entry["triggerKind"] = "mission_terminal_marker"
        with self.assertRaisesRegex(CatalogError, "eventless compiled interaction"):
            self.validate(catalog=catalog)
        scene = copy.deepcopy(self.scene)
        marker = next(row for row in scene["interactions"] if row["id"] == "rs_capability_terminal_marker")
        marker["event"] = "evolution_granted"
        with self.assertRaisesRegex(CatalogError, "eventless compiled interaction"):
            self.validate(scene=scene)

    def test_pending_dialogue_hooks_must_be_absent_from_compiled_scene(self):
        scene = copy.deepcopy(self.scene)
        scene["interactions"].append({"id": "rs_after_gh", "kind": "dialogue", "event": None})
        with self.assertRaisesRegex(CatalogError, "pending scene hook unexpectedly exists"):
            self.validate(scene=scene)

    def test_progression_state_bindings_match_authoritative_world_event_sets(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["progressionStateBindings"][1]["requiredEvents"] = ["hive_power"]
        with self.assertRaisesRegex(CatalogError, "do not match authoritative progression catalog"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["entries"][1]["progressionStateId"] = "unknown_state"
        with self.assertRaisesRegex(CatalogError, "no known progression state"):
            self.validate(catalog=catalog)

    def test_mh_cw_and_epilogue_must_remain_unclaimed_as_playable(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["progressionStateBindings"][2]["playabilityClaimed"] = True
        with self.assertRaisesRegex(CatalogError, "status or claim is invalid"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["implementationClaims"]["threeWorldEpiloguePlayable"] = True
        with self.assertRaisesRegex(CatalogError, "overstate"):
            self.validate(catalog=catalog)

    def test_source_hash_and_locale_are_checked(self):
        catalog = copy.deepcopy(self.catalog)
        catalog["designSourceSha256"] = "0" * 64
        with self.assertRaisesRegex(CatalogError, "verified design report"):
            self.validate(catalog=catalog)
        catalog = copy.deepcopy(self.catalog)
        catalog["locale"] = "en-US"
        with self.assertRaisesRegex(CatalogError, "locale must be zh-CN"):
            self.validate(catalog=catalog)

    def test_compiled_scene_assets_must_be_runtime_registered_and_approved(self):
        scene = copy.deepcopy(self.scene)
        scene["presentation"]["sprites"].append({
            "id": "test_asset", "assetId": "runtime2d.world.returnstation.not_approved",
        })
        with self.assertRaisesRegex(CatalogError, "unknown runtime assets|unapproved assets"):
            self.validate(scene=scene)

if __name__ == "__main__":
    unittest.main()
