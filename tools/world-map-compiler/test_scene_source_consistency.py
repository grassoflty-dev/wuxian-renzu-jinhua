"""Read-only release-source reproducibility for every authored production scene."""
import hashlib
import json
import unittest
from pathlib import Path

from compile_maps import compile_directory


ROOT = Path(__file__).resolve().parents[2]
COMPILED = ROOT / "content/scenes/compiled"


class SceneSourceConsistencyTests(unittest.TestCase):
    def test_all_thirty_authored_scenes_reproduce_committed_bytes(self):
        scenes = compile_directory(
            ROOT / "design/maps",
            ROOT / "governance/assets/RUNTIME_ASSET_MANIFEST.json",
            ROOT / "governance/assets/AI_ASSET_RELEASE_MANIFEST.json",
            None,
            ROOT / "server-rs/data/world_progression_v1.json",
            ROOT / "content/enemies/entity-types.json",
        )
        self.assertEqual(len(scenes), 30)
        generated_names = {f"{scene['sceneId']}.json" for scene in scenes}
        self.assertEqual(len(generated_names), len(scenes))
        self.assertEqual(generated_names, {path.name for path in COMPILED.glob("*.json")})
        for scene in scenes:
            name = f"{scene['sceneId']}.json"
            with self.subTest(scene=name):
                regenerated = (json.dumps(scene, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")
                self.assertEqual(regenerated, (COMPILED / name).read_bytes(),
                                 f"authored source must reproduce {name}; do not edit compiled output alone")

    def test_return_station_toolbox_source_layer_matches_output_and_native_pin(self):
        authored = json.loads((ROOT / "design/maps/return_station/rs_core_room.tmj").read_text(encoding="utf-8"))
        toolbox_layers = [layer for layer in authored["layers"]
                          if any(obj.get("name") == "approved_floor_detail" for obj in layer.get("objects", []))]
        self.assertEqual(len(toolbox_layers), 1)
        self.assertEqual(toolbox_layers[0]["name"], "visual.props_dynamic")
        compiled_bytes = (COMPILED / "rs_core_room.json").read_bytes()
        compiled = json.loads(compiled_bytes)
        sprites = [sprite for sprite in compiled["presentation"]["sprites"] if sprite["id"] == "approved_floor_detail"]
        self.assertEqual(len(sprites), 1)
        self.assertEqual(sprites[0]["layer"], toolbox_layers[0]["name"])
        pin = hashlib.sha256(compiled_bytes).hexdigest()
        self.assertIn(f'"rs_core_room.json",\n    "{pin}"', (ROOT / "server-rs/build.rs").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
