"""Optimistic scene/event regression, not collision, combat or Game acceptance.

All local producers are assumed achievable on scene entry. This deliberately
does not prove Sentinel defeat, beacon ownership, settlement or player input.
Only authored inter-scene links and their requiresEvent gates constrain travel.
"""

from collections import deque
from copy import deepcopy
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def producers(scene):
    """Canonical producer fields used by compile_maps.validate_campaign."""
    events = {
        entry["event"]
        for key in ("interactions", "triggers")
        for entry in scene[key]
        if entry.get("event")
    }
    events.update(entry["event"] for entry in scene.get("interactionAggregates", []))
    return events


def links(scene):
    return scene["transitions"] + [
        door for door in scene["doors"] if door.get("toSceneId")
    ]


def validate_references(scenes):
    for scene in scenes.values():
        for link in links(scene):
            target = scenes[link["toSceneId"]]
            if target["worldId"] != scene["worldId"]:
                raise AssertionError(f"cross-world link: {link['id']}")
            if link["spawnId"] not in {spawn["id"] for spawn in target["spawns"]}:
                raise AssertionError(f"invalid target spawn: {link['id']}")


def require_reachable_campaign(scenes, required):
    """Explore (scene, events) states; never pool events across separate paths."""
    validate_references(scenes)
    pending = deque([("gh_entry_maintenance", frozenset())])
    visited = set()
    blocked = set()
    while pending:
        scene_id, acquired = pending.popleft()
        state = (scene_id, acquired)
        if state in visited:
            continue
        visited.add(state)
        scene = scenes[scene_id]
        local_events = frozenset(producers(scene) & required)
        enriched = acquired | local_events
        if enriched != acquired:
            pending.append((scene_id, enriched))
            continue
        for link in links(scene):
            needed = link.get("requiresEvent")
            if needed and needed not in acquired:
                blocked.add((scene_id, link["id"], link["toSceneId"], needed))
            else:
                pending.append((link["toSceneId"], acquired))

    reached = {scene_id for scene_id, _ in visited}
    acquired_anywhere = set().union(*(events for _, events in visited))
    unreachable = sorted(set(scenes) - reached)
    unavailable = sorted(required - acquired_anywhere)
    exit_complete = any(
        scene_id == "gh_exit" and required <= events
        for scene_id, events in visited
    )
    if unreachable or not exit_complete:
        raise AssertionError(
            f"unreachable scenes={unreachable}; unavailable events={unavailable}; "
            f"blocked state links={sorted(blocked)}; "
            f"exit with required events={exit_complete}"
        )
    return visited


class GreyHiveEventRouteReachability(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        paths = sorted((ROOT / "content/scenes/compiled").glob("gh_*.json"))
        paths.append(ROOT / "server-rs/data/world_progression_v1.json")
        cls.original_bytes = {path: path.read_bytes() for path in paths}
        scene_list = [json.loads(cls.original_bytes[path]) for path in paths[:-1]]
        cls.scenes = {scene["sceneId"]: scene for scene in scene_list}
        if len(scene_list) != 11 or len(cls.scenes) != 11:
            raise AssertionError("expected eleven unique authored GH scenes")
        catalog = json.loads(cls.original_bytes[paths[-1]])
        worlds = [w for w in catalog["worlds"] if w["worldId"] == "grey_hive"]
        if len(worlds) != 1:
            raise AssertionError("expected exactly one Grey Hive progression entry")
        cls.required = frozenset(worlds[0]["requiredEvents"])
        if len(cls.required) != 3:
            raise AssertionError("expected three required progression events")

    @classmethod
    def tearDownClass(cls):
        for path, original in cls.original_bytes.items():
            if path.read_bytes() != original:
                raise AssertionError(f"production input changed: {path}")

    def transition(self, scenes, ident):
        matches = [
            link for scene in scenes.values()
            for link in scene["transitions"] if link["id"] == ident
        ]
        self.assertEqual(len(matches), 1, ident)
        return matches[0]

    def assert_global_producers_and_references_valid(self, scenes):
        validate_references(scenes)
        emitted = set().union(*(producers(scene) for scene in scenes.values()))
        self.assertTrue(self.required <= emitted)

    def test_production_all_scenes_and_complete_exit_state(self):
        states = require_reachable_campaign(self.scenes, self.required)
        self.assertEqual({scene_id for scene_id, _ in states}, set(self.scenes))
        self.assertTrue(any(
            scene_id == "gh_exit" and self.required <= events
            for scene_id, events in states
        ))

    def test_entry_power_gate_rejects_self_lock(self):
        scenes = deepcopy(self.scenes)
        self.transition(scenes, "gh_entry_to_power_room")["requiresEvent"] = "hive_power"
        self.assert_global_producers_and_references_valid(scenes)
        with self.assertRaises(AssertionError) as caught:
            require_reachable_campaign(scenes, self.required)
        message = str(caught.exception)
        self.assertIn("gh_power_room", message)
        self.assertIn("hive_power", message)
        self.assertIn("gh_entry_to_power_room", message)
        self.assertIn("exit with required events=False", message)

    def test_shaft_lockdown_gate_rejects_downstream_deadlock(self):
        scenes = deepcopy(self.scenes)
        self.transition(scenes, "gh_shaft_to_lockdown")["requiresEvent"] = "hive_lockdown"
        self.assert_global_producers_and_references_valid(scenes)
        with self.assertRaises(AssertionError) as caught:
            require_reachable_campaign(scenes, self.required)
        message = str(caught.exception)
        self.assertIn("gh_lockdown", message)
        self.assertIn("hive_lockdown", message)
        self.assertIn("gh_shaft_to_lockdown", message)
        self.assertIn("exit with required events=False", message)

    def test_valid_arena_redirect_disconnects_beacon_and_exit(self):
        scenes = deepcopy(self.scenes)
        onward = self.transition(scenes, "gh_sentinel_arena_to_beacon")
        returning = self.transition(scenes, "gh_sentinel_arena_return_to_deep_decon")
        onward["toSceneId"] = returning["toSceneId"]
        onward["spawnId"] = returning["spawnId"]
        self.assert_global_producers_and_references_valid(scenes)
        with self.assertRaises(AssertionError) as caught:
            require_reachable_campaign(scenes, self.required)
        message = str(caught.exception)
        self.assertIn("unreachable scenes=['gh_beacon', 'gh_exit']", message)
        self.assertIn("unavailable events=['hive_extraction']", message)
        self.assertIn("exit with required events=False", message)


if __name__ == "__main__":
    unittest.main()
