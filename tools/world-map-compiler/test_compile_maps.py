import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from compile_maps import MapError, compile_directory, compile_map, load_admitted_assets, validate_campaign


ASSET_ID = "runtime2d.grey_hive.floor_tiles.v1"
ASSET_SHA = "ac36eb55fe0e5404b8b11ee6904d886703de6b5873d5b34be8c8f1008e5fc1e6"


def prop(name, value):
    return {"name": name, "value": value}


def obj(name, x=64, y=64, properties=(), **extra):
    return {"name": name, "x": x, "y": y, "properties": [prop(*p) for p in properties], **extra}


def layer(name, objects):
    return {"name": name, "type": "objectgroup", "objects": objects}


def valid_map():
    return {
        "width": 10, "height": 10, "tilewidth": 64, "tileheight": 64,
        "properties": [prop("worldId", "grey_hive"), prop("sceneId", "gh_test"), prop("pixelsPerMeter", 64)],
        "layers": [
            layer("visual.floor", [obj("floor", properties=[("assetId", ASSET_ID)])]),
            layer("logic.navigation", [obj("nav_a", properties=[("neighbors", "nav_b")]), obj("nav_b", x=256, y=256)]),
            layer("logic.spawn", [obj("entry", properties=[("kind", "player"), ("navNode", "nav_a")])]),
            layer("logic.collision", [obj("wall", x=320, y=320, width=64, height=64)]),
            layer("logic.checkpoint", [obj("checkpoint", x=256, y=256, properties=[("navNode", "nav_b")])]),
            layer("logic.trigger", [obj("power", x=128, y=128, width=64, height=64, properties=[("event", "hive_power")])]),
            layer("logic.objective", [obj("restore_power", properties=[("completingEvent", "hive_power")])]),
        ],
    }


class CompilerTests(unittest.TestCase):
    def test_conveyor_has_explicit_finite_generic_velocity(self):
        source = valid_map()
        source["layers"].append(layer("logic.terrain", [obj("belt", x=128, y=128, width=64, height=64,
            properties=[("tag", "conveyor"), ("surfaceVelocityXMps", -1.2), ("surfaceVelocityZMps", 0.0)])]))
        region = compile_map(source, {ASSET_ID})["terrainRegions"][0]
        self.assertEqual(region, {"id":"belt", "tag":"conveyor", "polygon":[[2.0,2.0],[3.0,2.0],[3.0,3.0],[2.0,3.0]],
            "surfaceVelocityMps":[-1.2,0.0]})

    def test_conveyor_rejects_missing_nonfinite_zero_or_excessive_velocity(self):
        for x,z in [(None,0), (True,0), (float("nan"),0), (float("inf"),0), (0,0), (4.1,0), (3,3)]:
            source=valid_map()
            source["layers"].append(layer("logic.terrain", [obj("belt", width=64,height=64,
                properties=[("tag","conveyor"),("surfaceVelocityXMps",x),("surfaceVelocityZMps",z)])]))
            with self.subTest(x=x,z=z), self.assertRaises(MapError): compile_map(source,{ASSET_ID})

    def test_velocity_on_other_terrain_is_not_silently_ignored(self):
        source=valid_map()
        source["layers"].append(layer("logic.terrain", [obj("water",width=64,height=64,
            properties=[("tag","terrain.water_shallow"),("surfaceVelocityXMps",1),("surfaceVelocityZMps",0)])]))
        with self.assertRaisesRegex(MapError,"surface velocity requires conveyor"): compile_map(source,{ASSET_ID})

    @staticmethod
    def moving_machinery_map():
        source = valid_map()
        source["layers"].append(layer("logic.hazard", [obj(
            "press_a", x=128, y=128, width=64, height=64,
            properties=[("kind", "moving_machinery"), ("damage", 16), ("periodMs", 750),
                        ("warningMs", 900), ("phase", "phase3"), ("translationXM", 4.0),
                        ("translationZM", 1.0), ("speedMps", 2.0), ("endpointHoldMs", 600)],
        )]))
        return source

    def test_compiles_generic_reciprocating_machinery(self):
        source = self.moving_machinery_map()
        compiled = compile_map(source, {ASSET_ID})
        self.assertEqual(compiled["hazards"], [{
            "id": "press_a", "kind": "moving_machinery",
            "polygon": [[2.0, 2.0], [3.0, 2.0], [3.0, 3.0], [2.0, 3.0]],
            "damage": 16, "periodMs": 750, "warningMs": 900, "phase": "phase3",
            "translationM": [4.0, 1.0], "speedMps": 2.0, "endpointHoldMs": 600,
        }])
        # Negative vectors and diagonal motion are valid data, not scene-specific
        # constants; translated polygons may touch the rectangular scene boundary.
        for entry in source["layers"][-1]["objects"][0]["properties"]:
            if entry["name"] in {"translationXM", "translationZM"}:
                entry["value"] = -2.0
        self.assertEqual(compile_map(source, {ASSET_ID})["hazards"][0]["translationM"], [-2.0, -2.0])

    def test_moving_machinery_rejects_incomplete_tuning(self):
        for missing in ("damage", "periodMs", "warningMs", "phase", "translationXM",
                        "translationZM", "speedMps", "endpointHoldMs"):
            source = self.moving_machinery_map()
            press = source["layers"][-1]["objects"][0]
            press["properties"] = [entry for entry in press["properties"] if entry["name"] != missing]
            with self.subTest(missing=missing), self.assertRaisesRegex(MapError, "incomplete moving machinery tuning"):
                compile_map(source, {ASSET_ID})
        source["layers"][-1]["objects"][0]["properties"] = [prop("kind", "moving_machinery")]
        with self.assertRaisesRegex(MapError, "incomplete moving machinery tuning"):
            compile_map(source, {ASSET_ID})

    def test_moving_machinery_rejects_malformed_finite_and_out_of_bounds_motion(self):
        invalid = [
            ("damage", 0), ("damage", 10_001), ("damage", True), ("damage", 1.5),
            ("periodMs", 99), ("periodMs", 60_001), ("periodMs", False),
            ("warningMs", 0), ("warningMs", 10_001), ("warningMs", 1.5),
            ("phase", "phase2"), ("phase", "phase4"),
            ("speedMps", 0), ("speedMps", -1), ("speedMps", 20.01),
            ("speedMps", float("nan")), ("speedMps", float("inf")), ("speedMps", True),
            ("translationXM", float("nan")), ("translationXM", float("inf")),
            ("translationZM", float("-inf")), ("translationZM", "2"), ("translationXM", False),
            ("endpointHoldMs", 99), ("endpointHoldMs", 10_001),
            ("endpointHoldMs", True), ("endpointHoldMs", 600.5),
        ]
        for name, value in invalid:
            source = self.moving_machinery_map()
            for entry in source["layers"][-1]["objects"][0]["properties"]:
                if entry["name"] == name:
                    entry["value"] = value
            with self.subTest(name=name, value=value), self.assertRaisesRegex(MapError, "moving machinery"):
                compile_map(source, {ASSET_ID})
        for dx, dz, error in [(0.0, 0.0, "translation"), (0.001, 0.001, "translation"),
                              (7.01, 0, "path outside bounds"), (-2.01, 0, "path outside bounds"),
                              (0, 7.01, "path outside bounds"), (0, -2.01, "path outside bounds")]:
            source = self.moving_machinery_map()
            for entry in source["layers"][-1]["objects"][0]["properties"]:
                if entry["name"] in {"translationXM", "translationZM"}:
                    entry["value"] = dx if entry["name"] == "translationXM" else dz
            with self.subTest(dx=dx, dz=dz), self.assertRaisesRegex(MapError, error):
                compile_map(source, {ASSET_ID})

    def test_machinery_motion_cannot_be_added_to_static_heat_or_unknown_hazard(self):
        for kind in ("heat_zone", "unknown"):
            source = self.moving_machinery_map()
            source["layers"][-1]["objects"][0]["properties"][0]["value"] = kind
            with self.subTest(kind=kind), self.assertRaises(MapError):
                compile_map(source, {ASSET_ID})
        source = self.moving_machinery_map()
        source["layers"][-1]["objects"][0]["width"] = 0
        with self.assertRaisesRegex(MapError, "zero-area polygon"):
            compile_map(source, {ASSET_ID})

    def test_compiles_and_validates_authored_heat_zone_tuning(self):
        source = valid_map()
        source["layers"].append(layer("logic.hazard", [obj(
            "heat_a", x=128, y=128, width=64, height=64,
            properties=[("kind", "heat_zone"), ("damage", 12), ("periodMs", 1000),
                        ("warningMs", 600), ("phase", "phase2")],
        )]))
        compiled = compile_map(source, {ASSET_ID})
        self.assertEqual(compiled["hazards"], [{
            "id": "heat_a", "kind": "heat_zone",
            "polygon": [[2.0, 2.0], [3.0, 2.0], [3.0, 3.0], [2.0, 3.0]],
            "damage": 12, "periodMs": 1000, "warningMs": 600, "phase": "phase2",
        }])
        for name, value, message in [
            ("damage", 0, "invalid heat hazard damage"),
            ("periodMs", 0, "invalid heat hazard periodMs"),
            ("warningMs", 0, "invalid heat hazard warningMs"),
            ("phase", "phase3", "invalid heat hazard phase"),
        ]:
            bad = json.loads(json.dumps(source))
            for entry in bad["layers"][-1]["objects"][0]["properties"]:
                if entry["name"] == name:
                    entry["value"] = value
            with self.subTest(name=name), self.assertRaisesRegex(MapError, message):
                compile_map(bad, {ASSET_ID})

        duplicate = json.loads(json.dumps(source))
        duplicate["layers"][-1]["objects"].append(duplicate["layers"][-1]["objects"][0])
        with self.assertRaisesRegex(MapError, "duplicate object id"):
            compile_map(duplicate, {ASSET_ID})

    def test_rejects_heat_zone_with_invalid_polygon(self):
        source = valid_map()
        source["layers"].append(layer("logic.hazard", [obj(
            "heat_a", x=128, y=128, width=0, height=64,
            properties=[("kind", "heat_zone"), ("damage", 12), ("periodMs", 1000),
                        ("warningMs", 600), ("phase", "phase2")],
        )]))
        with self.assertRaisesRegex(MapError, "zero-area polygon"):
            compile_map(source, {ASSET_ID})

    def test_compiles_actor_first_kill_collision_only_for_same_scene_enemy(self):
        source = valid_map()
        source["layers"][2]["objects"].append(obj(
            "elite", x=512, y=512,
            properties=[("kind", "enemy"), ("entityType", "enemy.test.elite")],
        ))
        source["layers"][3]["objects"].append(obj(
            "shutter", x=128, y=128, width=32, height=64,
            properties=[("requiresActorFirstKill", "elite")],
        ))
        compiled = compile_map(source, {ASSET_ID}, {"enemy.test.elite"})
        self.assertEqual(compiled["collision"][-1], {
            "id": "shutter",
            "polygon": [[2.0, 2.0], [2.5, 2.0], [2.5, 3.0], [2.0, 3.0]],
            "requiresActorFirstKill": "elite",
        })

        for value, message in [
            ("missing", "same-scene enemy spawn"),
            ("entry", "same-scene enemy spawn"),
            ("", "requiresActorFirstKill"),
            (False, "requiresActorFirstKill"),
        ]:
            bad = json.loads(json.dumps(source))
            bad["layers"][3]["objects"][-1]["properties"][0]["value"] = value
            with self.subTest(value=value), self.assertRaisesRegex(MapError, message):
                compile_map(bad, {ASSET_ID}, {"enemy.test.elite"})

        bad = json.loads(json.dumps(source))
        bad["layers"][2]["objects"][-1]["properties"][0]["value"] = "npc"
        with self.assertRaisesRegex(MapError, "same-scene enemy spawn"):
            compile_map(bad, {ASSET_ID}, {"enemy.test.elite"})

        bad = json.loads(json.dumps(source))
        bad["layers"][3]["objects"][-1]["properties"].append(prop("requiresEvent", "hive_power"))
        with self.assertRaisesRegex(MapError, "cannot combine progression and actor kill"):
            compile_map(bad, {ASSET_ID}, {"enemy.test.elite"})

    def test_compiles_reusable_all_activated_interaction_aggregate(self):
        source = valid_map()
        source["layers"].pop(5)
        source["layers"].append(layer("logic.interaction", [
            obj("valve_a", properties=[("kind", "valve_control")]),
            obj("valve_b", x=128, properties=[("kind", "valve_control")]),
            obj("valve_c", x=192, properties=[("kind", "valve_control")]),
            obj("valve_group_complete", x=256, properties=[
                ("kind", "three_valve_sequence_staged_marker"),
                ("aggregateMembers", "valve_a,valve_b,valve_c"),
                ("aggregateEvent", "hive_power"),
            ]),
        ]))
        required = {"grey_hive": {"hive_power", "hive_lockdown", "hive_extraction"}}
        scene = compile_map(source, {ASSET_ID}, None, required)
        self.assertEqual(scene["interactions"][0]["kind"], "valve_control")
        self.assertIsNone(scene["interactions"][0]["event"])
        self.assertEqual(scene["interactionAggregates"], [{
            "markerId": "valve_group_complete",
            "memberIds": ["valve_a", "valve_b", "valve_c"],
            "event": "hive_power",
        }])
        self.assertEqual(scene["objectives"][0]["completingEvent"], "hive_power")

        source["layers"][-1]["objects"][3]["properties"][2]["value"] = "unknown_event"
        with self.assertRaisesRegex(MapError, "unknown required event"):
            compile_map(source, {ASSET_ID}, None, required)

        source = valid_map()
        source["layers"].append(layer("logic.interaction", [
            obj("valve_a", properties=[("kind", "valve_control")]),
            obj("valve_group_complete", properties=[
                ("kind", "three_valve_sequence_staged_marker"),
                ("aggregateMembers", "missing_valve"),
                ("aggregateEvent", "hive_power"),
            ]),
        ]))
        with self.assertRaisesRegex(MapError, "invalid interaction aggregate member"):
            compile_map(source, {ASSET_ID}, None, required)

    def test_compiles_utf8_lf_deterministically_and_checks_asset_provenance(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            maps = root / "maps"
            maps.mkdir()
            source = valid_map()
            source["properties"][0] = prop("worldId", "灰巢")
            source["properties"][1] = prop("sceneId", "测试_场景")
            (maps / "scene.tmj").write_text(json.dumps(source, ensure_ascii=False), encoding="utf-8")
            release = root / "release.json"
            runtime = root / "runtime.json"
            release.write_text(json.dumps({"assets": [{"asset_id": ASSET_ID, "relative_path": "batch01/source.png", "sha256": ASSET_SHA, "release_status": "release_approved"}]}), encoding="utf-8")
            runtime.write_text(json.dumps({"atlases": [{"size": [1024, 1024], "pngPath": "atlas.png"}], "assets": [{"assetId": ASSET_ID, "sourcePath": "batch01/source.png", "sourceSha256": ASSET_SHA, "admission": "release_approved", "atlasPage": 0, "outputs": {"png": {"sha256": "1" * 64}, "webp": {"sha256": "2" * 64}}}]}), encoding="utf-8")
            first, second = root / "first", root / "second"
            scenes = compile_directory(maps, runtime, release, first, None)
            compile_directory(maps, runtime, release, second, None)
            first_bytes = (first / "测试_场景.json").read_bytes()
            second_bytes = (second / "测试_场景.json").read_bytes()
            self.assertEqual(first_bytes, second_bytes)
            self.assertNotIn(b"\r", first_bytes)
            self.assertTrue(first_bytes.endswith(b"\n"))
            self.assertFalse(first_bytes.endswith(b"\n\n"))
            self.assertIn("灰巢".encode("utf-8"), first_bytes)
            self.assertEqual(json.loads(first_bytes.decode("utf-8")), scenes[0])
            self.assertEqual(scenes[0]["objectives"][0]["completingEvent"], "hive_power")
            bad = json.loads(runtime.read_text(encoding="utf-8"))
            bad["assets"][0]["sourceSha256"] = "0" * 64
            runtime.write_text(json.dumps(bad), encoding="utf-8")
            with self.assertRaisesRegex(MapError, "SHA mismatch"):
                load_admitted_assets(runtime, release)
            bad["assets"][0]["sourceSha256"] = ASSET_SHA
            bad["atlases"][0]["size"] = [8192, 8192]
            runtime.write_text(json.dumps(bad), encoding="utf-8")
            with self.assertRaisesRegex(MapError, "exceeds release budget"):
                load_admitted_assets(runtime, release)

    def test_fails_closed_on_non_approved_asset(self):
        source = valid_map()
        with self.assertRaisesRegex(MapError, "unknown or unapproved assetId"):
            compile_map(source, set())

    def test_rejects_duplicate_object_and_zero_area_polygon(self):
        source = valid_map()
        source["layers"][3]["objects"].append(obj("wall", x=380, y=320, width=64, height=64))
        with self.assertRaisesRegex(MapError, "duplicate object id"):
            compile_map(source, {ASSET_ID})
        source["layers"][3]["objects"].pop()
        source["layers"][3]["objects"][0]["width"] = 0
        with self.assertRaisesRegex(MapError, "zero-area polygon"):
            compile_map(source, {ASSET_ID})
        source = valid_map()
        source["layers"][3]["objects"][0] = obj("wall", x=320, y=320,
            polygon=[{"x": 0, "y": 0}, {"x": 64, "y": 64}, {"x": 0, "y": 64}, {"x": 64, "y": 0}])
        with self.assertRaisesRegex(MapError, "zero-area polygon|self-intersecting polygon"):
            compile_map(source, {ASSET_ID})

    def test_rejects_unreachable_checkpoint_and_missing_objective_event(self):
        source = valid_map()
        source["layers"][1]["objects"][0]["properties"] = []
        with self.assertRaisesRegex(MapError, "unreachable checkpoint"):
            compile_map(source, {ASSET_ID})
        source = valid_map()
        source["layers"][5]["objects"][0]["properties"] = [prop("event", "other_event")]
        with self.assertRaisesRegex(MapError, "objective has no completing trigger"):
            compile_map(source, {ASSET_ID})

    def test_rejects_missing_transition_target_and_required_event(self):
        scene = compile_map(valid_map(), {ASSET_ID})
        scene["transitions"].append({"id": "exit", "toSceneId": "missing", "spawnId": "entry"})
        with self.assertRaisesRegex(MapError, "unknown transition target"):
            validate_campaign([scene], None)
        scene["transitions"].clear()
        progression = {"worlds": [{"worldId": "grey_hive", "requiredEvents": ["hive_power", "hive_extraction"]}]}
        with self.assertRaisesRegex(MapError, "required world events never emitted"):
            validate_campaign([scene], progression)

    def test_compiles_progression_gated_static_door_for_a_world_slice(self):
        source = valid_map()
        source["layers"].append(layer("logic.door", [obj(
            "gh_gate_a", properties=[
                ("assetId", "runtime2d.prop.gate_a.closed_open.v1"),
                ("requiresEvent", "hive_power"),
            ],
        )]))
        required = {"grey_hive": {"hive_power", "hive_lockdown", "hive_extraction"}}
        scene = compile_map(source, {ASSET_ID, "runtime2d.prop.gate_a.closed_open.v1"}, None, required)
        self.assertEqual(scene["doors"], [{
            "id": "gh_gate_a", "position": [1.0, 0.0, 1.0],
            "assetId": "runtime2d.prop.gate_a.closed_open.v1",
            "requiresEvent": "hive_power",
        }])
        progression = {"worlds": [{"worldId": "grey_hive", "requiredEvents": sorted(required["grey_hive"])}]}
        validate_campaign([scene], progression, allow_incomplete_world_slice=True)
        source["layers"][-1]["objects"][0]["properties"][1]["value"] = "not_catalogued"
        with self.assertRaisesRegex(MapError, "unknown required event"):
            compile_map(source, {ASSET_ID, "runtime2d.prop.gate_a.closed_open.v1"}, None, required)

    def test_compiles_and_validates_progression_gated_transition(self):
        source = valid_map()
        source["layers"].append(layer("logic.transition", [obj(
            "gh_gate_a_to_shaft", x=448, y=128, width=64, height=64,
            properties=[("toSceneId", "gh_central_shaft"), ("spawnId", "shaft_spawn"),
                        ("requiresEvent", "hive_power")],
        )]))
        required = {"grey_hive": {"hive_power", "hive_lockdown", "hive_extraction"}}
        scene = compile_map(source, {ASSET_ID}, None, required)
        self.assertEqual(scene["transitions"][0]["requiresEvent"], "hive_power")
        source["layers"][-1]["objects"][0]["properties"][2]["value"] = "not_catalogued"
        with self.assertRaisesRegex(MapError, "unknown required event"):
            compile_map(source, {ASSET_ID}, None, required)

    def test_world_slice_requires_catalog_and_emits_at_least_one_canonical_event(self):
        source = compile_map(valid_map(), {ASSET_ID})
        progression = {"worlds": [{"worldId": "grey_hive", "requiredEvents": ["hive_extraction", "hive_lockdown"]}]}
        with self.assertRaisesRegex(MapError, "emits no canonical progression event"):
            validate_campaign([source], progression, allow_incomplete_world_slice=True)

    def test_partial_campaign_requires_return_station_and_catalog_world_but_keeps_full_gate_strict(self):
        grey_hive = compile_map(valid_map(), {ASSET_ID})
        hub_source = {
            "width": 10, "height": 10, "tilewidth": 64, "tileheight": 64,
            "properties": [prop("worldId", "return_station"), prop("sceneId", "rs_core_room"), prop("pixelsPerMeter", 64)],
            "layers": [
                layer("logic.navigation", [obj("hub_nav")]),
                layer("logic.spawn", [obj("hub_spawn", properties=[("kind", "player"), ("navNode", "hub_nav")])]),
                layer("logic.interaction", [obj("gh_entry_marker", properties=[("kind", "grey_hive_entry_marker")])]),
            ],
        }
        hub = compile_map(hub_source, {ASSET_ID})
        progression = {"worlds": [
            {"worldId": "grey_hive", "requiredEvents": ["hive_power", "hive_extraction"]},
            {"worldId": "mist_harbor", "requiredEvents": ["mist_harbor_complete"]},
        ]}
        validate_campaign([grey_hive, hub], progression, allow_partial_campaign=True)
        self.assertEqual(hub["interactions"][0]["event"], None)
        with self.assertRaisesRegex(MapError, "required world events never emitted: grey_hive"):
            validate_campaign([grey_hive, hub], progression)
        with self.assertRaisesRegex(MapError, "requires return_station"):
            validate_campaign([grey_hive], progression, allow_partial_campaign=True)
        unsupported = dict(hub, worldId="unsupported_world")
        with self.assertRaisesRegex(MapError, "requires return_station"):
            validate_campaign([grey_hive, unsupported], progression, allow_partial_campaign=True)
        with self.assertRaisesRegex(MapError, "mutually exclusive"):
            validate_campaign([grey_hive, hub], progression, True, True)

    def test_rejects_unknown_enemy_type(self):
        source = valid_map()
        source["layers"][2]["objects"].append(obj("enemy", x=128, y=128, properties=[("kind", "enemy"), ("entityType", "enemy.unknown")]))
        with self.assertRaisesRegex(MapError, "unknown entityType"):
            compile_map(source, {ASSET_ID}, {"enemy.known"})

    def test_rejects_uncompiled_tile_layers_and_rotated_shapes(self):
        source = valid_map()
        source["layers"][0]["type"] = "tilelayer"
        with self.assertRaisesRegex(MapError, "unsupported layer type"):
            compile_map(source, {ASSET_ID})
        source = valid_map()
        source["layers"][3]["objects"][0]["rotation"] = 30
        with self.assertRaisesRegex(MapError, "rotated Tiled object"):
            compile_map(source, {ASSET_ID})

    def test_compiles_controlled_traversal_to_canonical_logic_section(self):
        source = valid_map()
        source["layers"].append(layer("logic.traversal", [obj(
            "air_step_a", x=64, y=64,
            properties=[("toX", 192), ("toY", 64), ("rangeM", 1.2),
                        ("cooldownMs", 350),
                        ("requiresCapabilities", "mobility.air_step_i")],
        )]))
        scene = compile_map(source, {ASSET_ID})
        self.assertEqual(scene["logic"]["traversal"], [{
            "id": "air_step_a", "from": [1.0, 0.0, 1.0], "to": [3.0, 0.0, 1.0],
            "rangeM": 1.2, "cooldownMs": 350,
            "requiredCapabilities": ["mobility.air_step_i"],
        }])

    def test_rejects_bad_traversal_range_cooldown_ability_and_destination(self):
        source = valid_map()
        source["layers"].append(layer("logic.traversal", [obj(
            "air_step_a", properties=[("toX", 192), ("toY", 64),
                                      ("rangeM", 1.3), ("cooldownMs", 350)],
        )]))
        with self.assertRaisesRegex(MapError, "range must be 1.2"):
            compile_map(source, {ASSET_ID})

        source["layers"][-1]["objects"][0]["properties"] = [
            prop("toX", 192), prop("toY", 64), prop("rangeM", 1.2),
            prop("cooldownMs", 351),
        ]
        with self.assertRaisesRegex(MapError, "cooldown must be 350"):
            compile_map(source, {ASSET_ID})

        source["layers"][-1]["objects"][0]["properties"] = [
            prop("toX", 192), prop("toY", 64), prop("rangeM", 1.2),
            prop("cooldownMs", 350), prop("requiresCapabilities", "mobility.unknown"),
        ]
        with self.assertRaisesRegex(MapError, "unknown traversal capability"):
            compile_map(source, {ASSET_ID})

        source["layers"][-1]["objects"][0]["properties"] = [
            prop("toX", 352), prop("toY", 352), prop("rangeM", 1.2),
            prop("cooldownMs", 350),
        ]
        with self.assertRaisesRegex(MapError, "collision-blocked"):
            compile_map(source, {ASSET_ID})

    def test_rejects_duplicate_traversal_ids_and_out_of_bounds_destination(self):
        source = valid_map()
        traversal = layer("logic.traversal", [obj(
            "air_step_a", properties=[("toX", 192), ("toY", 64),
                                      ("rangeM", 1.2), ("cooldownMs", 350)],
        )])
        source["layers"].append(traversal)
        traversal["objects"].append(obj(
            "air_step_a", x=128,
            properties=[("toX", 192), ("toY", 64), ("rangeM", 1.2), ("cooldownMs", 350)],
        ))
        with self.assertRaisesRegex(MapError, "duplicate object id"):
            compile_map(source, {ASSET_ID})

        traversal["objects"].pop()
        traversal["objects"][0]["properties"][0]["value"] = 1000
        with self.assertRaisesRegex(MapError, "out of bounds or collision-blocked"):
            compile_map(source, {ASSET_ID})

    def test_compiles_authoring_geometry_and_keeps_authoring_only_out_of_runtime(self):
        source = valid_map()
        source["layers"].extend([
            layer("logic.walkable", [obj("walkable", x=32, y=32, width=576, height=576)]),
            layer("logic.exploration", [obj("region", x=64, y=64, width=128, height=128)]),
            layer("logic.terrain", [obj("deep", x=192, y=192, width=128, height=128,
                                         properties=[("tag", "terrain.water_deep")])]),
            layer("authoring.only", [obj("shortcut", x=128, y=128,
                                          properties=[("kind", "shortcut_candidate")])]),
        ])
        scene = compile_map(source, {ASSET_ID})
        self.assertEqual(scene["walkablePolygons"], [{
            "id": "walkable", "polygon": [[0.5, 0.5], [9.5, 0.5], [9.5, 9.5], [0.5, 9.5]],
        }])
        self.assertEqual(scene["explorationRegions"], [{
            "id": "region", "polygon": [[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]],
        }])
        self.assertEqual(scene["terrainRegions"], [{
            "id": "deep", "tag": "terrain.water_deep",
            "polygon": [[3.0, 3.0], [5.0, 3.0], [5.0, 5.0], [3.0, 5.0]],
        }])
        self.assertNotIn("authoringOnly", scene)
        self.assertNotIn("walkablePolygons", compile_map(valid_map(), {ASSET_ID}))

    def test_new_authoring_layers_fail_closed(self):
        source = valid_map()
        source["layers"].append(layer("logic.terrain", [obj(
            "unknown", properties=[("tag", "terrain.water_other")],
        )]))
        with self.assertRaisesRegex(MapError, "unknown terrain tag"):
            compile_map(source, {ASSET_ID})

        source = valid_map()
        source["layers"].append(layer("logic.walkable", [obj(
            "outside", x=640, y=64, width=64, height=64,
        )]))
        with self.assertRaisesRegex(MapError, "object outside bounds|polygon outside bounds"):
            compile_map(source, {ASSET_ID})

        source = valid_map()
        source["layers"].append(layer("logic.exploration", [obj(
            "wall", x=64, y=64, width=64, height=64,
        )]))
        with self.assertRaisesRegex(MapError, "duplicate object id"):
            compile_map(source, {ASSET_ID})

        source = valid_map()
        source["layers"].append(layer("authoring.only", [obj(
            "not_authorized", properties=[("kind", "shortcut_runtime")],
        )]))
        with self.assertRaisesRegex(MapError, "unsupported authoring-only kind"):
            compile_map(source, {ASSET_ID})

        source = valid_map()
        source["layers"].append(layer("logic.exploration", [obj(
            "self_crossing", x=64, y=64,
            polygon=[{"x": 0, "y": 0}, {"x": 128, "y": 128},
                     {"x": 0, "y": 128}, {"x": 128, "y": 0}],
        )]))
        with self.assertRaisesRegex(MapError, "zero-area polygon|self-intersecting polygon"):
            compile_map(source, {ASSET_ID})

    def test_mist_harbor_water_contract_accepts_partial_scene_slices(self):
        source = valid_map()
        source["properties"] = [
            prop("worldId", "mist_harbor"), prop("sceneId", "mh_fog_pier"),
            prop("pixelsPerMeter", 64),
        ]
        scene = compile_map(source, {ASSET_ID})
        validate_campaign([scene], None, allow_incomplete_world_slice=True)

        pump_source = valid_map()
        pump_source.update(width=24, height=16, tilewidth=32, tileheight=32)
        pump_source["properties"] = [
            prop("worldId", "mist_harbor"), prop("sceneId", "mh_pump_station"),
            prop("pixelsPerMeter", 32),
        ]
        pump_source["layers"].append(layer("logic.terrain", [obj(
            "mh_pump_east_channel_water", x=576, y=176, width=176, height=160,
            properties=[("tag", "terrain.water_deep")],
        )]))
        pump_slice = compile_map(pump_source, {ASSET_ID})
        validate_campaign([pump_slice], None, allow_incomplete_world_slice=True)


if __name__ == "__main__":
    unittest.main()
