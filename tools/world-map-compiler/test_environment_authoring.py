import copy
import json
import tempfile
import unittest
from pathlib import Path
from compile_maps import MapError, compile_map, compile_directory, point_hits_polygon
from test_compile_maps import ASSET_ID, valid_map, layer, obj, prop

ROOT = Path(__file__).resolve().parents[2]
CONFIG = {"tag": "heat", "warningMs": 1000, "activeMs": 8000, "recoveryMs": 3000,
          "damage": 8, "damageIntervalMs": 1000,
          "exposure": {"maxUnits": 100, "gainPerSecond": 25, "lossPerSecond": 20, "damageThreshold": 50}}
CONTROL = {"targetHazardIds": ["heat"], "suppressionMs": 6000, "exposureReductionUnits": 40}


def sample(config=None, control=None):
    source = valid_map()
    source["layers"] += [
        layer("logic.hazard", [obj("heat", x=128, y=128, width=64, height=64,
            properties=[("kind", "heat_zone"), ("environment", json.dumps(CONFIG if config is None else config))])]),
        layer("logic.interaction", [obj("coolant", properties=[("kind", "environment_control"),
            ("rangeM", 2.5), ("cooldownMs", 8000), ("environmentControl", json.dumps(CONTROL if control is None else control))])]),
    ]
    return source


class EnvironmentAuthoringTests(unittest.TestCase):
    def test_emits_strict_nested_configs_and_keeps_old_maps_identical(self):
        compiled = compile_map(sample(), {ASSET_ID})
        self.assertEqual(compiled["hazards"][0]["environment"], CONFIG)
        self.assertEqual(compiled["interactions"][0]["environmentControl"], CONTROL)
        self.assertIsNone(compiled["interactions"][0]["event"])
        old = compile_map(valid_map(), {ASSET_ID})
        self.assertEqual(old["hazards"], [])
        self.assertNotIn("environment", json.dumps(old))

    def test_rejects_unknown_duplicate_incomplete_and_noninteger_environment_fields(self):
        malformed = [None, [], {}, {**CONFIG, "tag": None}, {**CONFIG, "tag": "toxin"},
            {**CONFIG, "grant": "hive_extraction"}, {**CONFIG, "warningMs": True},
            {**CONFIG, "activeMs": 0}, {**CONFIG, "damageIntervalMs": 60_001},
            {**CONFIG, "exposure": {**CONFIG["exposure"], "damageThreshold": 101}},
            {**CONFIG, "exposure": {**CONFIG["exposure"], "maxUnits": 1.5}},
            {**CONFIG, "exposure": {**CONFIG["exposure"], "immune": True}}]
        for config in malformed:
            with self.subTest(config=config):
                source = sample(); source["layers"][-2]["objects"][0]["properties"][-1]["value"] = json.dumps(config)
                with self.assertRaises(MapError): compile_map(source, {ASSET_ID})
        source = sample(); source["layers"][-2]["objects"][0]["properties"][-1]["value"] = json.dumps(CONFIG)[:-1] + ',"tag":"heat"}'
        with self.assertRaises(MapError): compile_map(source, {ASSET_ID})
        for extra in [("damage",8), ("warningMs",1000), ("translationXM",1.0)]:
            source=sample();source["layers"][-2]["objects"][0]["properties"].append(prop(*extra))
            with self.assertRaises(MapError): compile_map(source,{ASSET_ID})

    def test_controls_cannot_emit_events_join_aggregates_or_target_unknown_hazards(self):
        for control in [{**CONTROL,"targetHazardIds":[]}, {**CONTROL,"targetHazardIds":["unknown"]},
            {**CONTROL,"targetHazardIds":["heat","heat"]}, {**CONTROL,"targetHazardIds":[1]},
            {**CONTROL,"targetHazardIds":["/heat"]}, {**CONTROL,"suppressionMs":8001},
            {**CONTROL,"exposureReductionUnits":-1}, {**CONTROL,"event":"hive_extraction"}]:
            with self.subTest(control=control), self.assertRaises(MapError): compile_map(sample(control=control),{ASSET_ID})
        for extra in [("event","hive_power"),("kind","valve")]:
            source=sample();p=source["layers"][-1]["objects"][0]["properties"]
            p[:]=[v for v in p if v["name"]!=extra[0]];p.append(prop(*extra))
            with self.assertRaises(MapError):compile_map(source,{ASSET_ID})
        source=sample();source["layers"][-1]["objects"].append(obj("group", properties=[("kind","aggregate_marker"), ("aggregateMembers","coolant"), ("aggregateEvent","hive_power")]))
        with self.assertRaises(MapError):compile_map(source,{ASSET_ID})
        source=sample();source["layers"][-1]["objects"][0]["properties"].append(prop("environment",json.dumps(CONFIG)))
        with self.assertRaises(MapError):compile_map(source,{ASSET_ID})

    def test_all_thirty_committed_scenes_reproduce_and_three_regions_keep_safe_spawns_ground_lane(self):
        with tempfile.TemporaryDirectory() as temporary:
            out=Path(temporary)
            scenes=compile_directory(ROOT/"design/maps", ROOT/"governance/assets/RUNTIME_ASSET_MANIFEST.json",
                ROOT/"governance/assets/AI_ASSET_RELEASE_MANIFEST.json", out,
                ROOT/"server-rs/data/world_progression_v1.json", ROOT/"content/enemies/entity-types.json")
            self.assertEqual(len(scenes),30)
            for path in out.glob("*.json"):
                self.assertEqual(path.read_bytes(), (ROOT/"content/scenes/compiled"/path.name).read_bytes(),path.name)
            authored=[s for s in scenes if any("environment" in h for h in s["hazards"])]
            self.assertEqual({s["sceneId"] for s in authored},{"gh_deep_decon","cw_boiler_chamber","cw_furnace_heart"})
            self.assertEqual(sum(len(s["hazards"]) for s in authored),5)
            for scene in authored:
                spawn=next(s["position"] for s in scene["spawns"] if s["kind"]=="player")
                for hazard in scene["hazards"]:
                    self.assertFalse(point_hits_polygon(spawn[0],spawn[2],.35,hazard["polygon"]))
                    for x in range(2,23):self.assertFalse(point_hits_polygon(x,8,.35,hazard["polygon"]),(scene["sceneId"],x))
                controls=[i for i in scene["interactions"] if i["kind"]=="environment_control"]
                self.assertTrue(controls)
                self.assertTrue(all(i["event"] is None for i in controls))
                self.assertFalse(scene.get("interactionAggregates"))
                targets={h for c in controls for h in c["environmentControl"]["targetHazardIds"]}
                self.assertEqual(targets,{h["id"] for h in scene["hazards"]})

if __name__ == "__main__": unittest.main()
