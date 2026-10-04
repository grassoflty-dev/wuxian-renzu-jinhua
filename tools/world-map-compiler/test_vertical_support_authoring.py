"""Generic authoring fixtures; heights/bands are test tuning, not Gear Shaft content."""
import copy
import json
from pathlib import Path
import unittest
from compile_maps import MapError, compile_map
from test_compile_maps import ASSET_ID, valid_map, layer, obj, prop


def vertical_map():
    source = valid_map()
    for entry in source["layers"]:
        if entry["name"] in {"logic.interaction", "logic.door", "logic.trigger", "logic.checkpoint", "logic.transition", "logic.traversal"}:
            for item in entry["objects"]:
                item["properties"] += [prop("heightMinM", 0.0), prop("heightMaxM", 0.1)]
    source["layers"] += [
        layer("logic.standing_deck", [obj("upper_deck", x=192, y=192, width=256, height=128, properties=[("heightM", 2.0)])]),
        layer("logic.moving_support", [obj("lift", x=96, y=192, width=160, height=128, properties=[("lowerM", 0.0), ("upperM", 2.0), ("travelMs", 1000), ("endpointHoldMs", 750)])]),
        layer("logic.interaction", [obj("upper_console", x=256, y=256, properties=[("kind", "console"), ("heightM", 2.0), ("heightMinM", 1.9), ("heightMaxM", 2.1)])]),
        layer("logic.transition", [obj("upper_exit", x=224, y=224, width=64, height=64, properties=[("toSceneId", "gh_test_power"), ("spawnId", "gh_spawn"), ("heightMinM", 1.9), ("heightMaxM", 2.1)])]),
        layer("logic.traversal", [obj("upper_vault", x=256, y=256, properties=[("heightM", 2.0), ("toX", 416), ("toY", 256), ("toHeightM", 2.0), ("rangeM", 1.2), ("cooldownMs", 350), ("heightMinM", 1.9), ("heightMaxM", 2.1)])]),
    ]
    return source


def find(source, name):
    return next(item for entry in source["layers"] for item in entry["objects"] if item["name"] == name)


def set_property(item, name, value):
    existing = next((p for p in item["properties"] if p["name"] == name), None)
    if existing is None:
        item["properties"].append(prop(name, value))
    else:
        existing["value"] = value


class VerticalSupportCompilerTests(unittest.TestCase):
    def test_explicit_support_catalog_and_height_fields_compile_without_flattening(self):
        result = compile_map(vertical_map(), {ASSET_ID})
        self.assertEqual(result["standingDecks"][0]["heightM"], 2.0)
        self.assertEqual(result["movingSupports"][0]["upperM"], 2.0)
        self.assertEqual(result["interactions"][0]["position"], [4.0, 2.0, 4.0])
        self.assertEqual(result["transitions"][0]["heightRangeM"], [1.9, 2.1])
        marker = result["logic"]["traversal"][0]
        self.assertEqual(marker["from"], [4.0, 2.0, 4.0])
        self.assertEqual(marker["to"], [6.5, 2.0, 4.0])
        self.assertEqual(marker["fromHeightRangeM"], [1.9, 2.1])
        self.assertEqual(marker["requiredCapabilities"], [])

    def test_explicit_object_id_is_used_for_support_and_height_binding(self):
        source = vertical_map()
        set_property(find(source, "upper_deck"), "id", "deck_identity")
        set_property(find(source, "upper_console"), "id", "console_identity")
        result = compile_map(source, {ASSET_ID})
        self.assertEqual(result["standingDecks"][0]["id"], "deck_identity")
        self.assertEqual(result["interactions"][0]["id"], "console_identity")
        self.assertEqual(result["interactions"][0]["heightRangeM"], [1.9, 2.1])

    def test_rust_loader_fixture_is_exact_compiler_output(self):
        fixture = Path(__file__).resolve().parents[2] / "server-rs/tests/fixtures/standing-deck-compiled.json"
        expected = json.dumps(compile_map(vertical_map(), {ASSET_ID}), ensure_ascii=False, indent=2, sort_keys=True) + "\n"
        self.assertEqual(fixture.read_text(), expected)

    def test_missing_reversed_nonfinite_or_wrong_layer_height_fields_fail_closed(self):
        for mutation in ["missing", "reversed", "nan", "typo", "wrong_layer", "anchor_outside"]:
            source = vertical_map()
            item = find(source, "upper_console")
            if mutation == "missing":
                item["properties"] = [p for p in item["properties"] if p["name"] not in ("heightMinM", "heightMaxM")]
            elif mutation == "reversed":
                set_property(item, "heightMinM", 2.2)
            elif mutation == "nan":
                set_property(item, "heightMaxM", float("nan"))
            elif mutation == "typo":
                item["properties"].append(prop("heightMin", 1.9))
            elif mutation == "wrong_layer":
                find(source, "wall")["properties"].append(prop("heightM", 2.0))
            else:
                set_property(item, "heightMinM", 0.0)
                set_property(item, "heightMaxM", 0.1)
            with self.subTest(mutation=mutation), self.assertRaises(MapError):
                compile_map(source, {ASSET_ID})

    def test_support_timing_geometry_bounds_and_unknown_keys_are_rejected(self):
        for field, value in [("travelMs", 249), ("travelMs", True), ("endpointHoldMs", 30001), ("upperM", 3.1), ("lowerM", -1.0), ("travelMS", 1000)]:
            source = vertical_map()
            set_property(find(source, "lift"), field, value)
            with self.subTest(field=field, value=value), self.assertRaises(MapError):
                compile_map(source, {ASSET_ID})
        for points in [
            [{"x":0,"y":0},{"x":256,"y":128},{"x":256,"y":0},{"x":0,"y":128}],
            [{"x":0,"y":0},{"x":256,"y":0},{"x":64,"y":64},{"x":256,"y":128},{"x":0,"y":128}],
        ]:
            source = vertical_map()
            find(source, "upper_deck")["polygon"] = points
            with self.assertRaises(MapError):
                compile_map(source, {ASSET_ID})

    def test_unsupported_raised_traversal_and_spawn_cannot_create_floating_ground(self):
        source = vertical_map()
        set_property(find(source, "upper_vault"), "toHeightM", 1.0)
        with self.assertRaisesRegex(MapError, "stable endpoint support"):
            compile_map(source, {ASSET_ID})
        source = vertical_map()
        set_property(find(source, "entry"), "heightM", 2.0)
        with self.assertRaisesRegex(MapError, "stable standing support"):
            compile_map(source, {ASSET_ID})

    def test_duplicate_mixed_support_ids_and_combined_catalog_limit_fail_closed(self):
        source = vertical_map()
        find(source, "upper_deck")["name"] = "lift"
        with self.assertRaisesRegex(MapError, "duplicate object"):
            compile_map(source, {ASSET_ID})
        source = vertical_map()
        entry = next(l for l in source["layers"] if l["name"] == "logic.standing_deck")
        template = copy.deepcopy(entry["objects"][0])
        for number in range(64):
            item = copy.deepcopy(template)
            item["name"] = f"extra_deck_{number}"
            entry["objects"].append(item)
        with self.assertRaisesRegex(MapError, "total supports"):
            compile_map(source, {ASSET_ID})

    def test_hazard_height_band_is_preserved_and_required_on_deck_scenes(self):
        source = vertical_map()
        source["layers"].append(layer("logic.hazard", [obj("steam", x=64, y=64, width=64, height=64,
            properties=[("kind", "steam_jet"), ("heightMinM", 0.0), ("heightMaxM", 1.0)])]))
        self.assertEqual(compile_map(source, {ASSET_ID})["hazards"][0]["heightRangeM"], [0.0, 1.0])
        for values in [(1.9, 2.1), (0.0, 0.0)]:
            set_property(find(source, "steam"), "heightMinM", values[0])
            set_property(find(source, "steam"), "heightMaxM", values[1])
            self.assertEqual(compile_map(source, {ASSET_ID})["hazards"][0]["heightRangeM"], list(values))
        find(source, "steam")["properties"] = [prop("kind", "steam_jet")]
        with self.assertRaisesRegex(MapError, "explicit height range"):
            compile_map(source, {ASSET_ID})

    def test_original_flat_fixture_has_no_implicit_height_or_support_keys(self):
        result = compile_map(valid_map(), {ASSET_ID})
        self.assertNotIn("standingDecks", result)
        self.assertNotIn("movingSupports", result)
        for key in ("interactions", "doors", "triggers", "checkpoints", "transitions"):
            self.assertTrue(all("heightRangeM" not in item for item in result[key]))


if __name__ == "__main__":
    unittest.main()
