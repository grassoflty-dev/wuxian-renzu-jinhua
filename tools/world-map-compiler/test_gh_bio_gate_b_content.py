import hashlib
import json
import tempfile
import unittest
from collections import Counter
from pathlib import Path

from compile_maps import MapError, compile_directory, validate_campaign


ROOT = Path(__file__).resolve().parents[2]
MAPS = ROOT / "design/maps/grey_hive"
RUNTIME = ROOT / "governance/assets/RUNTIME_ASSET_MANIFEST.json"
RELEASE = ROOT / "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"
PROGRESSION = ROOT / "server-rs/data/world_progression_v1.json"
ENTITIES = ROOT / "content/enemies/entity-types.json"


class GreyHiveBioGateBContentTests(unittest.TestCase):
    def test_eleven_scene_route_assets_and_deterministic_compile(self):
        with tempfile.TemporaryDirectory() as temporary:
            first = Path(temporary) / "first"
            second = Path(temporary) / "second"
            scenes = compile_directory(MAPS, RUNTIME, RELEASE, first, PROGRESSION, ENTITIES, True)
            compile_directory(MAPS, RUNTIME, RELEASE, second, PROGRESSION, ENTITIES, True)

            self.assertEqual(len(scenes), 11)
            self.assertEqual({scene["sceneId"] for scene in scenes}, {
                "gh_entry_maintenance", "gh_power_room", "gh_gate_a", "gh_central_shaft",
                "gh_lockdown", "gh_bio_isolation", "gh_gate_b", "gh_deep_decon", "gh_sentinel_arena",
                "gh_beacon", "gh_exit",
            })
            outputs = sorted(first.glob("*.json"))
            self.assertEqual(len(outputs), 11)
            self.assertEqual(
                {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in outputs},
                {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(second.glob("*.json"))},
            )

            by_id = {scene["sceneId"]: scene for scene in scenes}
            release = json.loads(RELEASE.read_text(encoding="utf-8"))
            approved_assets = {
                row["asset_id"] for row in release["assets"] if row["release_status"] == "release_approved"
            }
            all_sprites = [sprite for scene in scenes for sprite in scene["presentation"]["sprites"]]
            self.assertTrue(all(sprite["assetId"] in approved_assets for sprite in all_sprites))
            forbidden_assets = {
                "runtime2d.world.grey_hive.deep_decon.v1.attempt",
                "runtime2d.vfx.decon_hazard.v1.attempt",
                "runtime2d.enemy.sentinel.telegraph.v1.attempt",
            }
            self.assertFalse(any(sprite["assetId"] in forbidden_assets for sprite in all_sprites))
            forward = next(item for item in by_id["gh_lockdown"]["transitions"] if item["toSceneId"] == "gh_bio_isolation")
            self.assertEqual(forward["spawnId"], "gh_bio_isolation_spawn")
            self.assertEqual(forward["requiresEvent"], "hive_lockdown")
            onward = by_id["gh_bio_isolation"]["transitions"]
            self.assertEqual(len(onward), 1)
            self.assertEqual(onward[0]["toSceneId"], "gh_gate_b")
            self.assertEqual(onward[0]["spawnId"], "gh_gate_b_spawn")
            self.assertEqual(onward[0]["requiresEvent"], "hive_lockdown")
            gate = next(item for item in by_id["gh_gate_b"]["doors"] if item["id"] == "gh_gate_b")
            self.assertEqual(gate["requiresEvent"], "hive_lockdown")
            to_decon = next(item for item in by_id["gh_gate_b"]["transitions"] if item["toSceneId"] == "gh_deep_decon")
            self.assertEqual(to_decon["spawnId"], "gh_deep_decon_spawn")
            self.assertEqual(to_decon["requiresEvent"], "hive_lockdown")
            to_arena = next(item for item in by_id["gh_deep_decon"]["transitions"] if item["toSceneId"] == "gh_sentinel_arena")
            self.assertEqual(to_arena["spawnId"], "gh_sentinel_arena_spawn")

            decon = by_id["gh_deep_decon"]
            self.assertEqual({hazard["kind"] for hazard in decon["hazards"]}, {"steam_jet", "decon_mist"})
            self.assertEqual(decon["checkpoints"][0]["id"], "gh_decon_safety_alcove_checkpoint")
            self.assertEqual(sum(spawn["entityType"] == "grey_hive.infected_maintenance_worker" for spawn in decon["spawns"]), 2)
            self.assertEqual(len([item for item in decon["interactions"] if "valve" in item["id"]]), 2)

            arena = by_id["gh_sentinel_arena"]
            back_to_decon = next(item for item in arena["transitions"] if item["toSceneId"] == "gh_deep_decon")
            self.assertEqual(back_to_decon["spawnId"], "gh_deep_decon_spawn")
            sentinel_spawns = [
                spawn for spawn in arena["spawns"]
                if spawn["kind"] == "enemy" and spawn["entityType"] == "enemy.grey_hive.sentinel"
            ]
            self.assertEqual(sentinel_spawns, [{
                "entityType": "enemy.grey_hive.sentinel",
                "id": "gh_sentinel_arena_sentinel_01",
                "kind": "enemy",
                "navNode": None,
                "position": [15.0, 0.0, 7.0],
            }])
            self.assertEqual(len(arena["occluders"]), 3)
            self.assertEqual(arena["presentation"]["sprites"][0]["assetId"], "runtime2d.world.grey_hive.sentinel_arena.v1.attempt")
            self.assertFalse(any(
                sprite["assetId"] == "runtime2d.enemy.sentinel.base.v1"
                or sprite["id"] == "gh_sentinel_arena_sentinel_01"
                for sprite in arena["presentation"]["sprites"]
            ))
            self.assertTrue(any(item["id"] == "gh_sys_sentinel_01" for item in arena["interactions"]))
            to_beacon = next(item for item in arena["transitions"] if item["toSceneId"] == "gh_beacon")
            self.assertEqual(to_beacon["spawnId"], "gh_beacon_spawn")
            self.assertNotIn("requiresEvent", to_beacon)

            beacon = by_id["gh_beacon"]
            to_exit = next(item for item in beacon["transitions"] if item["toSceneId"] == "gh_exit")
            self.assertEqual(to_exit["spawnId"], "gh_exit_spawn")
            self.assertTrue(any(sprite["assetId"] == "runtime2d.prop.beacon.folded.v1" for sprite in beacon["presentation"]["sprites"]))
            self.assertEqual({item["kind"] for item in beacon["interactions"]}, {
                "beacon_collect", "beacon_mount",
            })

            exit_scene = by_id["gh_exit"]
            extraction = next(item for item in exit_scene["interactions"] if item["id"] == "gh_exit_extraction_console")
            self.assertEqual(extraction["event"], "hive_extraction")
            self.assertFalse(any(sprite["assetId"] == "runtime2d.prop.beacon.deployed.v1" for sprite in exit_scene["presentation"]["sprites"]))
            self.assertTrue(any(sprite["assetId"] == "runtime2d.prop.lockdown_terminal.v1" for sprite in exit_scene["presentation"]["sprites"]))
            self.assertEqual(exit_scene["objectives"], [])
            self.assertEqual([(item["id"], item["toSceneId"], item["spawnId"]) for item in exit_scene["transitions"]], [("gh_exit_to_beacon", "gh_beacon", "gh_beacon_spawn")])
            self.assertFalse(any(spawn["kind"] == "enemy" for spawn in exit_scene["spawns"]))

            for scene_id in ("gh_bio_isolation", "gh_gate_b", "gh_deep_decon", "gh_sentinel_arena", "gh_beacon", "gh_exit"):
                scene = by_id[scene_id]
                self.assertFalse(any("baizhi" in (spawn["id"] + str(spawn["entityType"])).lower() for spawn in scene["spawns"]))
                brutes = [spawn for spawn in scene["spawns"] if "brute" in (spawn["id"] + str(spawn["entityType"])).lower()]
                if scene_id == "gh_gate_b":
                    self.assertEqual([(spawn["id"], spawn["entityType"], spawn["position"]) for spawn in brutes],
                                     [("gh_gate_b_brute_01", "enemy.grey_hive.brute", [17.0, 0.0, 8.0])])
                    self.assertEqual([(spawn["id"], spawn["position"]) for spawn in scene["spawns"]
                                      if spawn["entityType"] == "grey_hive.infected_security"],
                                     [("gh_gate_b_security_01", [7.0, 0.0, 4.5]), ("gh_gate_b_security_02", [17.0, 0.0, 11.5])])
                else:
                    self.assertEqual(brutes, [])

            expected_swarms = {
                "gh_lockdown": [("gh_lockdown_swarm_01", [7.0, 0.0, 8.0]), ("gh_lockdown_swarm_02", [17.0, 0.0, 8.0])],
                "gh_deep_decon": [("gh_deep_decon_swarm_01", [11.5, 0.0, 8.0]), ("gh_deep_decon_swarm_02", [20.0, 0.0, 5.0])],
            }
            for scene in scenes:
                swarms = [spawn for spawn in scene["spawns"] if spawn["entityType"] == "enemy.grey_hive.swarm"]
                self.assertEqual([(spawn["id"], spawn["position"]) for spawn in swarms], expected_swarms.get(scene["sceneId"], []))
            self.assertEqual(Counter(spawn["entityType"] for spawn in by_id["gh_lockdown"]["spawns"] if spawn["kind"] == "enemy"),
                             {"grey_hive.infected_maintenance_worker": 3, "grey_hive.infected_security": 3, "enemy.grey_hive.swarm": 2})
            self.assertEqual(Counter(spawn["entityType"] for spawn in by_id["gh_deep_decon"]["spawns"] if spawn["kind"] == "enemy"),
                             {"grey_hive.infected_maintenance_worker": 2, "enemy.grey_hive.swarm": 2})

    def test_return_station_grey_hive_mist_harbor_and_clockworks_compile_as_thirty_scenes(self):
        maps = ROOT / "design/maps"
        hub_source = json.loads((maps / "return_station/rs_core_room.tmj").read_text(encoding="utf-8"))
        object_ids = [obj["id"] for layer in hub_source["layers"] for obj in layer["objects"]]
        self.assertEqual(len(object_ids), len(set(object_ids)))
        self.assertGreater(hub_source["nextobjectid"], max(object_ids))
        mist_scene_ids = ("mh_fog_pier", "mh_tidal_warehouse", "mh_signal_yard", "mh_drowned_quay", "mh_breakwater", "mh_pump_station", "mh_resonance_tower", "mh_warden_arena", "mh_extraction")
        for scene_id in mist_scene_ids:
            source = json.loads((maps / f"mist_harbor/{scene_id}.tmj").read_text(encoding="utf-8"))
            source_ids = [obj["id"] for layer in source["layers"] for obj in layer["objects"]]
            self.assertEqual(len(source_ids), len(set(source_ids)), scene_id)
            self.assertGreater(source["nextobjectid"], max(source_ids), scene_id)
        clockworks_scene_ids = (
            "cw_entry_foundry", "cw_pressure_hall", "cw_conveyor_bridge", "cw_boiler_chamber",
            "cw_gear_shaft", "cw_furnace_heart", "cw_forged_guard_arena", "cw_regulator_core",
            "cw_shutdown_exit",
        )
        for scene_id in clockworks_scene_ids:
            source = json.loads((maps / f"clockworks/{scene_id}.tmj").read_text(encoding="utf-8"))
            source_ids = [obj["id"] for layer in source["layers"] for obj in layer["objects"]]
            self.assertEqual(len(source_ids), len(set(source_ids)), scene_id)
            self.assertGreater(source["nextobjectid"], max(source_ids), scene_id)
        with tempfile.TemporaryDirectory() as temporary:
            first = Path(temporary) / "first"
            second = Path(temporary) / "second"
            # Clockworks deliberately emits no event until three-valve aggregation exists.
            scenes = compile_directory(maps, RUNTIME, RELEASE, first, None, ENTITIES)
            compile_directory(maps, RUNTIME, RELEASE, second, None, ENTITIES)
            by_id = {scene["sceneId"]: scene for scene in scenes}
            hub = by_id["rs_core_room"]
            self.assertEqual(len(scenes), 30)
            source_hashes = {
                "rs_core_room": "aa04911df77ef3622a3f1317173b8fa148070ee1ef159a03b2d810f7af4bd355",
                "mh_extraction": "17f8a93fd1caecd7873113bb5ae0a156936d39cff4c7bb131e3895112e016008",
            }
            for scene_id, expected_hash in source_hashes.items():
                source = maps / ("return_station" if scene_id == "rs_core_room" else "mist_harbor") / f"{scene_id}.tmj"
                self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), expected_hash, scene_id)
                generated_json = (first / f"{scene_id}.json").read_bytes().replace(b"\r\n", b"\n")
                tracked_json = (ROOT / "content/scenes/compiled" / f"{scene_id}.json").read_bytes().replace(b"\r\n", b"\n")
                self.assertEqual(generated_json, tracked_json, scene_id)
            self.assertEqual(hub["worldId"], "return_station")
            self.assertEqual({item["kind"] for item in hub["interactions"]}, {
                "world_gate_marker", "grey_hive_entry_marker", "mission_terminal_marker",
                "capability_terminal_marker", "save_rest_terminal_marker", "storage_inventory_marker",
            })
            hub_gates = {item["id"]: item for item in hub["interactions"] if item["kind"] == "world_gate_marker"}
            self.assertEqual(set(hub_gates), {"rs_world_gate_marker", "rs_mh_world_gate_marker", "rs_cw_world_gate_marker"})
            self.assertNotEqual(hub_gates["rs_world_gate_marker"]["position"],
                                hub_gates["rs_mh_world_gate_marker"]["position"])
            self.assertEqual(hub["transitions"], [])
            self.assertTrue(all(item["event"] is None for item in hub["interactions"]))
            expected_tidebound = {
                "mh_drowned_quay": [("mh_drowned_quay_tidebound_01", [16.0, 0.0, 8.0])],
                "mh_breakwater": [("mh_breakwater_tidebound_01", [10.0, 0.0, 5.0]), ("mh_breakwater_tidebound_02", [17.0, 0.0, 11.0])],
                "mh_resonance_tower": [("mh_resonance_tower_tidebound_01", [12.0, 0.0, 10.5])],
            }
            for scene in scenes:
                tidebound = [spawn for spawn in scene["spawns"] if spawn["entityType"] == "enemy.mist_harbor.tidebound"]
                self.assertEqual([(spawn["id"], spawn["position"]) for spawn in tidebound], expected_tidebound.get(scene["sceneId"], []))
            for scene_id, old_type, old_count in [("mh_drowned_quay", "enemy.mist_harbor.drowned", 4),
                                                  ("mh_breakwater", "enemy.mist_harbor.signal_wraith", 2),
                                                  ("mh_resonance_tower", "enemy.mist_harbor.signal_wraith", 4)]:
                self.assertEqual(Counter(spawn["entityType"] for spawn in by_id[scene_id]["spawns"] if spawn["kind"] == "enemy"),
                                 {old_type: old_count, "enemy.mist_harbor.tidebound": len(expected_tidebound[scene_id])})
                generated = (first / f"{scene_id}.json").read_bytes()
                self.assertEqual(generated, (ROOT / "content/scenes/compiled" / f"{scene_id}.json").read_bytes())
            self.assertEqual(by_id["mh_drowned_quay"]["terrainRegions"], [{"id": "mh_water_depth_region",
                "tag": "terrain.water_shallow", "polygon": [[13.0, 5.0], [19.0, 5.0], [19.0, 11.0], [13.0, 11.0]]}])
            self.assertEqual(by_id["mh_breakwater"].get("terrainRegions", []), [])
            self.assertEqual(by_id["mh_resonance_tower"].get("terrainRegions", []), [])
            clockworks_scenes = tuple(by_id[scene_id] for scene_id in clockworks_scene_ids)
            entry_foundry = by_id["cw_entry_foundry"]
            pressure_hall = by_id["cw_pressure_hall"]
            conveyor_bridge = by_id["cw_conveyor_bridge"]
            boiler_chamber = by_id["cw_boiler_chamber"]
            gear_shaft = by_id["cw_gear_shaft"]
            furnace_heart = by_id["cw_furnace_heart"]
            forged_guard_arena = by_id["cw_forged_guard_arena"]
            regulator_core = by_id["cw_regulator_core"]
            regulator_spawns = [spawn for spawn in regulator_core["spawns"]
                                if spawn.get("id") == "cw_prime_regulator"
                                or spawn.get("entityType") == "enemy.clockworks.prime_regulator"]
            self.assertEqual(len(regulator_spawns), 1)
            self.assertEqual(regulator_spawns[0]["id"], "cw_prime_regulator")
            self.assertEqual(regulator_spawns[0]["kind"], "enemy")
            self.assertEqual(regulator_spawns[0]["entityType"], "enemy.clockworks.prime_regulator")
            self.assertEqual(regulator_spawns[0]["position"], [14.0, 0.0, 8.0])
            valve = next(item for item in regulator_core["interactions"]
                         if item["id"] == "cw_regulator_valve_furnace_link_staged")
            self.assertEqual((valve["kind"], valve["position"], valve["rangeM"], valve["cooldownMs"]),
                             ("coolant_valve", [18.0, 0.0, 11.0], 2.5, 8000))
            self.assertEqual(regulator_core["hazards"], [{
                "id": "cw_regulator_furnace_heat_zone", "kind": "heat_zone",
                "polygon": [[15.5, 3.5], [20.5, 3.5], [20.5, 6.0], [15.5, 6.0]],
                "damage": 12, "periodMs": 1000, "warningMs": 600, "phase": "phase2",
            }, {
                "id": "cw_regulator_reciprocating_press", "kind": "moving_machinery",
                "polygon": [[8.0, 11.5], [10.0, 11.5], [10.0, 13.0], [8.0, 13.0]],
                "damage": 16, "periodMs": 750, "warningMs": 900, "phase": "phase3",
                "translationM": [6.0, 0.0], "speedMps": 2.0, "endpointHoldMs": 600,
            }])
            # Ordinary Ground movement keeps the spawn and central z=7..9
            # corridor clear at every point of the reciprocating side-lane sweep.
            press = regulator_core["hazards"][1]
            for fraction in (0.0, 0.25, 0.5, 0.75, 1.0):
                translated = [[x + press["translationM"][0] * fraction,
                               z + press["translationM"][1] * fraction]
                              for x, z in press["polygon"]]
                self.assertTrue(all(7.0 < x < 17.0 and z >= 11.5 for x, z in translated))
            self.assertEqual(regulator_core["logic"]["traversal"], [])
            shutdown_exit = by_id["cw_shutdown_exit"]
            self.assertTrue(all(scene["worldId"] == "clockworks" for scene in clockworks_scenes))
            self.assertEqual(entry_foundry["transitions"][0]["toSceneId"], "cw_pressure_hall")
            self.assertEqual(entry_foundry["transitions"][0]["spawnId"], "cw_pressure_hall_spawn")
            self.assertEqual(pressure_hall["transitions"][0]["toSceneId"], "cw_entry_foundry")
            self.assertEqual(pressure_hall["transitions"][0]["spawnId"], "cw_entry_spawn")
            hall_targets = {item["toSceneId"]: item["spawnId"] for item in pressure_hall["transitions"]}
            self.assertEqual(hall_targets["cw_conveyor_bridge"], "cw_conveyor_bridge_spawn")
            bridge_targets = {item["toSceneId"]: item["spawnId"] for item in conveyor_bridge["transitions"]}
            self.assertEqual(bridge_targets, {
                "cw_pressure_hall": "cw_pressure_hall_spawn",
                "cw_boiler_chamber": "cw_boiler_chamber_spawn",
            })
            self.assertEqual(boiler_chamber["transitions"][0]["toSceneId"], "cw_conveyor_bridge")
            self.assertEqual(boiler_chamber["transitions"][0]["spawnId"], "cw_conveyor_bridge_spawn")
            boiler_targets = {item["toSceneId"]: item["spawnId"] for item in boiler_chamber["transitions"]}
            self.assertEqual(boiler_targets["cw_gear_shaft"], "cw_gear_shaft_spawn")
            gear_targets = {item["toSceneId"]: item["spawnId"] for item in gear_shaft["transitions"]}
            self.assertEqual(gear_targets, {
                "cw_boiler_chamber": "cw_boiler_chamber_spawn",
                "cw_furnace_heart": "cw_furnace_heart_spawn",
            })
            self.assertEqual(furnace_heart["transitions"][0]["toSceneId"], "cw_gear_shaft")
            self.assertEqual(furnace_heart["transitions"][0]["spawnId"], "cw_gear_shaft_spawn")
            furnace_targets = {item["toSceneId"]: item["spawnId"] for item in furnace_heart["transitions"]}
            self.assertEqual(furnace_targets["cw_forged_guard_arena"], "cw_forged_guard_arena_spawn")
            shutter_blockers = [
                collision for collision in forged_guard_arena["collision"]
                if collision.get("requiresActorFirstKill") is not None
            ]
            self.assertEqual(
                [(entry["id"], entry["requiresActorFirstKill"]) for entry in shutter_blockers],
                [
                    ("cw_arena_shutter_west_blocker", "cw_forged_guard_elite"),
                    ("cw_arena_shutter_east_blocker", "cw_forged_guard_elite"),
                ],
            )
            self.assertEqual(
                [item["id"] for item in forged_guard_arena["interactions"]],
                ["cw_arena_shutter_staged"],
                "Arena pressure wave is implemented by the live elite profile, not a staged interaction marker",
            )
            arena_targets = {item["toSceneId"]: item["spawnId"] for item in forged_guard_arena["transitions"]}
            self.assertEqual(arena_targets, {
                "cw_furnace_heart": "cw_furnace_heart_spawn",
                "cw_regulator_core": "cw_regulator_core_spawn",
            })
            self.assertEqual(regulator_core["transitions"][0]["toSceneId"], "cw_forged_guard_arena")
            self.assertEqual(regulator_core["transitions"][0]["spawnId"], "cw_forged_guard_arena_spawn")
            core_targets = {item["toSceneId"]: item["spawnId"] for item in regulator_core["transitions"]}
            self.assertEqual(core_targets["cw_shutdown_exit"], "cw_shutdown_exit_spawn")
            self.assertEqual(shutdown_exit["transitions"][0]["toSceneId"], "cw_regulator_core")
            self.assertEqual(shutdown_exit["transitions"][0]["spawnId"], "cw_regulator_core_spawn")
            self.assertTrue(all(transition["toSceneId"] in set(clockworks_scene_ids)
                                for scene in clockworks_scenes for transition in scene["transitions"]))
            shutdown_interactions = {item["id"]: item for item in shutdown_exit["interactions"]}
            self.assertEqual(shutdown_interactions["cw_master_shutdown_staged"]["kind"], "terminal")
            self.assertEqual(shutdown_interactions["cw_master_shutdown_staged"]["event"], "clockworks_shutdown")
            self.assertTrue(all(item["event"] is None for item_id, item in shutdown_interactions.items()
                                if item_id != "cw_master_shutdown_staged"))
            self.assertTrue({"air_step_grant_staged_marker",
                             "world_completion_staged_marker", "optional_enemy_wave_staged_marker",
                             "static_epilogue_dialogue_marker"}
                            .issubset({item["kind"] for item in shutdown_exit["interactions"]}))
            self.assertEqual(shutdown_exit["objectives"], [])
            self.assertEqual(shutdown_exit["triggers"], [])
            shutdown_source = json.loads((maps / "clockworks/cw_shutdown_exit.tmj").read_text(encoding="utf-8"))
            epilogue_source = next(obj for layer in shutdown_source["layers"] if layer["name"] == "logic.interaction"
                                   for obj in layer["objects"] if obj["name"] == "cw_cy_end_01_static_epilogue_marker")
            epilogue_properties = {item["name"]: item["value"] for item in epilogue_source["properties"]}
            self.assertEqual(epilogue_properties["dialogueText"], "原来门一直不止三扇。")
            self.assertEqual(regulator_core["objectives"], [])
            self.assertEqual(regulator_core["triggers"], [])
            self.assertEqual([item for scene in clockworks_scenes for item in scene["interactions"]
                              if item["event"] == "clockworks_core"], [{
                "id": "cw_regulator_core_console_staged", "kind": "terminal",
                "event": "clockworks_core", "position": [12.0, 0.0, 7.0], "rangeM": 2.5,
            }])
            forged_guard_roster = {
                "cw_entry_foundry": 2,
                "cw_pressure_hall": 2,
                "cw_conveyor_bridge": 3,
                "cw_gear_shaft": 2,
                "cw_furnace_heart": 2,
            }
            arena_elites = [spawn for spawn in forged_guard_arena["spawns"]
                            if spawn["kind"] == "enemy"]
            self.assertEqual([
                {"id": spawn["id"], "entityType": spawn["entityType"],
                 "position": spawn["position"]}
                for spawn in arena_elites
            ], [{
                "id": "cw_forged_guard_elite",
                "entityType": "enemy.clockworks.forged_guard_elite",
                "position": [12.0, 0.0, 7.0],
            }])
            self.assertEqual({
                scene["sceneId"]: sum(
                    spawn["kind"] == "enemy"
                    and spawn["entityType"] == "enemy.clockworks.forged_guard"
                    for spawn in scene["spawns"]
                )
                for scene in clockworks_scenes
                if scene["sceneId"] in forged_guard_roster
            }, forged_guard_roster)
            for scene in clockworks_scenes:
                actors = [spawn for spawn in scene["spawns"] if spawn["kind"] in {"enemy", "npc"}]
                if scene["sceneId"] == "cw_forged_guard_arena":
                    self.assertEqual(actors, arena_elites, scene["sceneId"])
                    continue
                if scene["sceneId"] == "cw_regulator_core":
                    self.assertEqual(actors, regulator_spawns, scene["sceneId"])
                    continue
                additions = {
                    "cw_pressure_hall": ("pressure_drone", [(14, 5), (18, 11)]),
                    "cw_conveyor_bridge": ("pressure_drone", [(14, 5), (19, 11)]),
                    "cw_gear_shaft": ("pressure_drone", [(11, 4), (16, 12), (20, 5)]),
                    "cw_boiler_chamber": ("furnace_hound", [(10, 4), (16, 12), (20, 5)]),
                    "cw_furnace_heart": ("furnace_hound", [(11, 4), (16, 12), (20, 5)]),
                }
                guard_count = forged_guard_roster.get(scene["sceneId"], 0)
                role, positions = additions.get(scene["sceneId"], (None, []))
                self.assertEqual(len(actors), guard_count + len(positions), scene["sceneId"])
                guards = actors[:guard_count]
                self.assertTrue(all(spawn["entityType"] == "enemy.clockworks.forged_guard"
                                    for spawn in guards), scene["sceneId"])
                self.assertTrue(all(3.0 <= spawn["position"][0] <= 12.0
                                    and spawn["position"][2] == 8.0
                                    for spawn in guards), scene["sceneId"])
                self.assertEqual([
                    (spawn["id"], spawn["entityType"], spawn["position"])
                    for spawn in actors[guard_count:]
                ], [(f"{scene['sceneId']}_{role}_{index:02}", f"enemy.clockworks.{role}", [x, 0, z])
                    for index, (x, z) in enumerate(positions, 1)], scene["sceneId"])
            self.assertEqual(conveyor_bridge["logic"]["traversal"], [{
                "id": "cw_conveyor_context_vault", "from": [8.0, 0.0, 8.0],
                "to": [10.0, 0.0, 8.0], "rangeM": 1.2, "cooldownMs": 350,
                "requiredCapabilities": [],
            }])
            self.assertTrue(any(item["id"] == "cw_moving_surface_staged" and item["event"] is None
                                for item in conveyor_bridge["interactions"]))
            boiler_markers = {item["id"]: item for item in boiler_chamber["interactions"]}
            self.assertEqual(boiler_markers["cw_coolant_valve_staged"]["event"], None)
            self.assertEqual({h["id"] for h in boiler_chamber["hazards"]}, {"cw_heat_accumulation_staged", "cw_boiler_vent_staged"})
            self.assertTrue(all("environment" in h for h in boiler_chamber["hazards"]))
            self.assertEqual([(t["id"], t["requiredCapabilities"]) for t in gear_shaft["logic"]["traversal"]],
                             [("cw_gear_upper_gap", []), ("cw_gear_earned_air_step", ["mobility.air_step_i"])])
            self.assertEqual(gear_shaft["interactions"], [])
            self.assertEqual([s["id"] for s in gear_shaft["movingSupports"]], ["cw_gear_main_lift"])
            self.assertEqual([s["id"] for s in gear_shaft["standingDecks"]], ["cw_gear_upper_dock", "cw_gear_furnace_landing"])
            self.assertEqual([h["id"] for h in furnace_heart["hazards"]], ["cw_heat_accumulation_staged"])
            self.assertEqual(furnace_heart["hazards"][0]["environment"]["tag"], "heat")
            self.assertTrue(all(item["event"] is None for item in furnace_heart["interactions"]))
            furnace_source = json.loads((maps / "clockworks/cw_furnace_heart.tmj").read_text(encoding="utf-8"))
            dialogue_source = next(obj for layer in furnace_source["layers"] if layer["name"] == "logic.interaction"
                                   for obj in layer["objects"] if obj["name"] == "cw_cy_heat_01_static_dialogue_marker")
            dialogue_properties = {item["name"]: item["value"] for item in dialogue_source["properties"]}
            self.assertEqual(dialogue_properties["dialogueText"], "炉心不是失控——它被维持在过载边缘。")
            valve_markers = [item for item in pressure_hall["interactions"]
                             if item["id"].startswith("cw_pressure_valve_")]
            self.assertEqual(len(valve_markers), 3)
            self.assertTrue(all(item["kind"] == "valve_control" and item["event"] is None
                                for item in valve_markers))
            self.assertEqual(pressure_hall["interactionAggregates"], [{
                "markerId": "cw_clockworks_valves_staged",
                "memberIds": [
                    "cw_pressure_valve_01_staged",
                    "cw_pressure_valve_02_staged",
                    "cw_pressure_valve_03_staged",
                ],
                "event": "clockworks_valves",
            }])
            self.assertTrue(any(item["id"] == "cw_clockworks_valves_staged" and item["event"] is None
                                for item in pressure_hall["interactions"]))
            self.assertNotIn("clockworks_valves", {item["event"] for scene in clockworks_scenes
                                                       for item in scene["interactions"] if item["event"]})
            fog_pier = by_id["mh_fog_pier"]
            warehouse = by_id["mh_tidal_warehouse"]
            signal_yard = by_id["mh_signal_yard"]
            drowned_quay = by_id["mh_drowned_quay"]
            breakwater = by_id["mh_breakwater"]
            pump_station = by_id["mh_pump_station"]
            tower = by_id["mh_resonance_tower"]
            arena = by_id["mh_warden_arena"]
            extraction = by_id["mh_extraction"]
            self.assertEqual(fog_pier["worldId"], "mist_harbor")
            self.assertEqual(warehouse["worldId"], "mist_harbor")
            self.assertEqual(fog_pier["transitions"][0]["toSceneId"], "mh_tidal_warehouse")
            self.assertEqual(fog_pier["transitions"][0]["spawnId"], "mh_warehouse_spawn")
            warehouse_targets = {item["toSceneId"]: item["spawnId"] for item in warehouse["transitions"]}
            self.assertEqual(warehouse_targets, {"mh_fog_pier": "mh_fog_pier_spawn", "mh_signal_yard": "mh_signal_yard_spawn"})
            yard_targets = {item["toSceneId"]: item["spawnId"] for item in signal_yard["transitions"]}
            self.assertEqual(yard_targets, {"mh_tidal_warehouse": "mh_warehouse_spawn", "mh_drowned_quay": "mh_drowned_quay_spawn"})
            quay_targets = {item["toSceneId"]: item["spawnId"] for item in drowned_quay["transitions"]}
            self.assertEqual(quay_targets, {"mh_signal_yard": "mh_signal_yard_spawn", "mh_breakwater": "mh_breakwater_spawn"})
            breakwater_targets = {item["toSceneId"]: item for item in breakwater["transitions"]}
            self.assertEqual(breakwater_targets["mh_drowned_quay"]["spawnId"], "mh_drowned_quay_spawn")
            self.assertEqual(breakwater_targets["mh_pump_station"]["spawnId"], "mh_pump_station_spawn")
            self.assertEqual(breakwater_targets["mh_pump_station"]["requiresEvent"], "mist_beacon_east")
            pump_targets = {item["toSceneId"]: item["spawnId"] for item in pump_station["transitions"]}
            self.assertEqual(pump_targets, {"mh_breakwater": "mh_breakwater_spawn", "mh_resonance_tower": "mh_resonance_tower_spawn"})
            tower_targets = {item["toSceneId"]: item for item in tower["transitions"]}
            self.assertEqual(tower_targets["mh_pump_station"]["spawnId"], "mh_pump_station_spawn")
            self.assertEqual(tower_targets["mh_warden_arena"]["spawnId"], "mh_warden_arena_spawn")
            self.assertEqual(tower_targets["mh_warden_arena"]["requiresEvent"], "mist_signal")
            arena_targets = {item["toSceneId"]: item["spawnId"] for item in arena["transitions"]}
            self.assertEqual(arena_targets, {"mh_resonance_tower": "mh_resonance_tower_spawn", "mh_extraction": "mh_extraction_spawn"})
            self.assertEqual(extraction["transitions"][0]["toSceneId"], "mh_warden_arena")
            self.assertEqual(extraction["transitions"][0]["spawnId"], "mh_warden_arena_spawn")
            self.assertFalse(any(item["event"] for item in extraction["interactions"]))
            extraction_kinds = {item["kind"] for item in extraction["interactions"]}
            self.assertIn("world_exit", extraction_kinds)
            self.assertTrue({"world_completion_staged_marker", "return_station_revisit_staged_marker"}
                            .issubset(extraction_kinds))
            self.assertNotIn("future_extraction_marker", extraction_kinds)
            exit_marker = next(item for item in extraction["interactions"] if item["id"] == "mh_extraction_exit")
            self.assertEqual(exit_marker["kind"], "world_exit")
            self.assertIsNone(exit_marker["event"])
            beacon = next(item for item in warehouse["interactions"] if item["id"] == "mh_west_beacon")
            self.assertEqual(beacon["kind"], "beacon")
            self.assertEqual(beacon["event"], "mist_beacon_west")
            self.assertTrue(any(item["kind"] == "foghorn_direction_marker" and item["event"] is None
                                for item in fog_pier["interactions"]))
            self.assertIn("signal_interference_zone", {item["kind"] for item in signal_yard["hazards"]})
            self.assertIn("water_depth_slowdown", {item["kind"] for item in drowned_quay["hazards"]})
            self.assertTrue(any(item["id"] == "mh_cy_signal_01_static_marker" and item["event"] is None
                                for item in signal_yard["interactions"]))
            self.assertTrue(any(item["kind"] == "water_depth_slowdown_marker" and item["event"] is None
                                for item in drowned_quay["interactions"]))
            east_beacon = next(item for item in breakwater["interactions"] if item["id"] == "mh_east_beacon")
            self.assertEqual(east_beacon["kind"], "beacon")
            self.assertEqual(east_beacon["event"], "mist_beacon_east")
            pump_control = next(item for item in pump_station["interactions"] if item["id"] == "mh_pump_control_primary")
            self.assertEqual(pump_control, {
                "id": "mh_pump_control_primary", "kind": "pump_control", "event": None,
                "position": [17.0, 0.0, 7.0],
            })
            self.assertFalse(any(item["event"] == "mist_pump" for item in pump_station["interactions"]))
            self.assertFalse(any(item["id"] in {"mh_pump_level_indicator", "mh_pump_shortcut_candidate_static_marker"}
                                 for item in pump_station["interactions"]))
            signal_console = next(item for item in tower["interactions"] if item["id"] == "mh_signal_console_staged")
            self.assertEqual(signal_console["kind"], "terminal")
            self.assertEqual(signal_console["event"], "mist_signal")
            self.assertTrue(any(item["id"] == "mh_acoustic_mapping_staged_marker" and item["event"] is None
                                for item in tower["interactions"]))
            self.assertFalse(any(item["id"] == "mh_resonance_warden_staged_marker" for item in arena["interactions"]))
            bosses = [spawn for spawn in arena["spawns"] if spawn["kind"] in {"enemy", "npc"}]
            self.assertEqual(len(bosses), 1)
            self.assertEqual(bosses[0]["id"], "mh_resonance_warden_staged_marker")
            self.assertEqual(bosses[0]["entityType"], "enemy.mist_harbor.resonance_warden")
            self.assertEqual(bosses[0]["position"], [18.0, 0.0, 8.0])
            self.assertFalse(any(item["event"] for item in arena["interactions"]))
            mist_scenes = tuple(by_id[scene_id] for scene_id in mist_scene_ids)
            exploration_specs = {
                "mh_fog_pier": [("mh_fp_entry_berth", .5, .5, 8, 15.5), ("mh_fp_foghorn_pier", 8, .5, 16, 15.5), ("mh_fp_warehouse_approach", 16, .5, 23.5, 15.5)],
                "mh_tidal_warehouse": [("mh_tw_west_loading", .5, .5, 8.5, 15.5), ("mh_tw_storage_floor", 8.5, .5, 16.5, 15.5), ("mh_tw_beacon_bay", 16.5, .5, 23.5, 15.5)],
                "mh_signal_yard": [("mh_sy_west_yard", .5, .5, 13, 15.5), ("mh_sy_interference_field", 13, .5, 19, 15.5), ("mh_sy_quay_approach", 19, .5, 23.5, 15.5)],
                "mh_drowned_quay": [("mh_dq_west_quay", .5, .5, 13, 15.5), ("mh_dq_flooded_channel", 13, .5, 19, 15.5), ("mh_dq_east_quay", 19, .5, 23.5, 15.5)],
                "mh_breakwater": [("mh_bw_west_seawall", .5, .5, 8.5, 15.5), ("mh_bw_center_seawall", 8.5, .5, 16.5, 15.5), ("mh_bw_east_beacon", 16.5, .5, 23.5, 15.5)],
                "mh_pump_station": [("mh_ps_intake", .5, .5, 10.5, 15.5), ("mh_ps_control_hall", 10.5, .5, 18, 15.5), ("mh_ps_east_channel", 18, .5, 23.5, 15.5)],
                "mh_resonance_tower": [("mh_rt_entry", .5, .5, 9.5, 15.5), ("mh_rt_resonance_floor", 9.5, .5, 16, 15.5), ("mh_rt_signal_section", 16, .5, 23.5, 15.5)],
                "mh_warden_arena": [("mh_wa_entry", .5, .5, 8, 15.5), ("mh_wa_arena_core", 8, .5, 20.5, 15.5), ("mh_wa_exit", 20.5, .5, 23.5, 15.5)],
                "mh_extraction": [("mh_ex_arrival", .5, .5, 9, 15.5), ("mh_ex_extraction_pad", 9, .5, 19, 15.5), ("mh_ex_exit_section", 19, .5, 23.5, 15.5)],
            }
            for scene_id, regions in exploration_specs.items():
                scene = by_id[scene_id]
                self.assertEqual(scene["walkablePolygons"], [{
                    "id": f"{scene_id}_walkable_area",
                    "polygon": [[.5,.5],[23.5,.5],[23.5,15.5],[.5,15.5]],
                }])
                expected = [{"id": ident, "polygon": [[xmin,zmin],[xmax,zmin],[xmax,zmax],[xmin,zmax]]}
                            for ident,xmin,zmin,xmax,zmax in regions]
                self.assertEqual(scene["explorationRegions"], expected, scene_id)
            water_regions = [region for scene in mist_scenes for region in scene.get("terrainRegions", [])]
            self.assertEqual(len(water_regions), 2)
            self.assertEqual(pump_station["terrainRegions"], [{
                "id": "mh_pump_east_channel_water", "tag": "terrain.water_deep",
                "polygon": [[18.0,5.5],[23.5,5.5],[23.5,10.5],[18.0,10.5]],
            }])
            self.assertEqual(drowned_quay["terrainRegions"], [{
                "id": "mh_water_depth_region", "tag": "terrain.water_shallow",
                "polygon": [[13.0,5.0],[19.0,5.0],[19.0,11.0],[13.0,11.0]],
            }])
            self.assertEqual(drowned_quay["hazards"], [{
                "id": "mh_water_depth_region", "kind": "water_depth_slowdown",
                "polygon": [[13.0,5.0],[19.0,5.0],[19.0,11.0],[13.0,11.0]],
                "terrainTag": "terrain.water_shallow",
            }])
            unexpected_water = dict(pump_station)
            unexpected_water["terrainRegions"] = pump_station["terrainRegions"] + [{
                "id": "unexpected_third_water", "tag": "terrain.water_shallow",
                "polygon": [[1.0,1.0],[2.0,1.0],[2.0,2.0],[1.0,2.0]],
            }]
            with self.assertRaisesRegex(MapError, "frozen two-region contract"):
                validate_campaign([unexpected_water if scene["sceneId"] == "mh_pump_station" else scene
                                   for scene in mist_scenes], None)
            pump_source = json.loads((maps / "mist_harbor/mh_pump_station.tmj").read_text(encoding="utf-8"))
            authoring = next(layer for layer in pump_source["layers"] if layer["name"] == "authoring.only")
            authoring_kinds = {item["name"]: {prop["name"]: prop["value"] for prop in item["properties"]}["kind"]
                               for item in authoring["objects"]}
            self.assertEqual(authoring_kinds, {
                "mh_pump_level_indicator": "pump_level_visual_marker",
                "mh_pump_shortcut_candidate_static_marker": "shortcut_candidate",
            })
            expected_mist_enemies = {
                "mh_fog_pier": {"enemy.mist_harbor.drowned": 2},
                "mh_tidal_warehouse": {"enemy.mist_harbor.drowned": 3, "enemy.mist_harbor.signal_wraith": 1},
                "mh_signal_yard": {"enemy.mist_harbor.signal_wraith": 3},
                "mh_drowned_quay": {"enemy.mist_harbor.drowned": 4, "enemy.mist_harbor.tidebound": 1},
                "mh_breakwater": {"enemy.mist_harbor.signal_wraith": 2, "enemy.mist_harbor.tidebound": 2},
                "mh_pump_station": {"enemy.mist_harbor.drowned": 3},
                "mh_resonance_tower": {"enemy.mist_harbor.signal_wraith": 4, "enemy.mist_harbor.tidebound": 1},
                "mh_warden_arena": {"enemy.mist_harbor.resonance_warden": 1},
                "mh_extraction": {},
            }
            for scene in mist_scenes:
                self.assertFalse(any(spawn["kind"] == "npc" for spawn in scene["spawns"]))
                actual = Counter(spawn["entityType"] for spawn in scene["spawns"] if spawn["kind"] == "enemy")
                self.assertEqual(actual, Counter(expected_mist_enemies[scene["sceneId"]]))
            emitted_mist_events = {item["event"] for scene in mist_scenes for item in scene["interactions"] if item["event"]}
            self.assertEqual(emitted_mist_events, {"mist_beacon_west", "mist_beacon_east", "mist_signal"})
            # Authoring completeness is distinct from staged runtime/world admission.
            validate_campaign(scenes, json.loads(PROGRESSION.read_text(encoding="utf-8")))
            validate_campaign(scenes, json.loads(PROGRESSION.read_text(encoding="utf-8")), allow_partial_campaign=True)
            cross_world = dict(by_id["gh_entry_maintenance"])
            cross_world["transitions"] = [{
                "id": "unsupported_cross_world_exit", "toSceneId": "rs_core_room", "spawnId": "rs_core_spawn",
            }]
            with self.assertRaisesRegex(MapError, "unknown transition target"):
                validate_campaign(
                    [cross_world if scene["sceneId"] == "gh_entry_maintenance" else scene for scene in scenes],
                    json.loads(PROGRESSION.read_text(encoding="utf-8")), allow_partial_campaign=True,
                )
            approved = {row["asset_id"] for row in json.loads(RELEASE.read_text(encoding="utf-8"))["assets"]
                        if row["release_status"] == "release_approved"}
            mist_asset_ids = {
                sprite["assetId"] for scene in mist_scenes for sprite in scene["presentation"]["sprites"]
            } | {marker["assetId"] for scene in mist_scenes for marker in scene["vfxMarkers"]}
            self.assertTrue(mist_asset_ids)
            self.assertTrue(mist_asset_ids.issubset(approved))
            self.assertTrue(all(asset_id.startswith(("runtime2d.world.mistharbor.", "runtime2d.enemy.mistharbor."))
                                for asset_id in mist_asset_ids))
            clockworks_asset_ids = {sprite["assetId"] for scene in clockworks_scenes
                                    for sprite in scene["presentation"]["sprites"]}
            self.assertTrue(clockworks_asset_ids)
            self.assertTrue(clockworks_asset_ids.issubset(approved))
            self.assertTrue(all(asset_id.startswith(("runtime2d.world.clockworks.", "runtime2d.enemy.clockworks."))
                                for asset_id in clockworks_asset_ids))
            first_hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(first.glob("*.json"))}
            second_hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(second.glob("*.json"))}
            self.assertEqual(first_hashes, second_hashes)


if __name__ == "__main__":
    unittest.main()
