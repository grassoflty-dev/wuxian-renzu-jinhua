"""Write the B package hash, GLB structure, and validation evidence manifest."""
import hashlib
import json
import pathlib
import re
import struct

ROOT = pathlib.Path(__file__).resolve().parents[2]
ASSETS = ROOT / "server-rs" / "release-ui" / "assets" / "grey-hive-first-flow"
SOURCE = ROOT / "design" / "art-production-first-flow"
OUT = ROOT / "artifacts" / "gh-art-b"
OUT.mkdir(parents=True, exist_ok=True)


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return {"path": path.relative_to(ROOT).as_posix(), "bytes": path.stat().st_size, "sha256": h.hexdigest()}


def glb_summary(path):
    data = path.read_bytes()
    total, = struct.unpack_from("<I", data, 8)
    chunk_len, chunk_type = struct.unpack_from("<II", data, 12)
    if data[:4] != b"glTF" or total != len(data) or chunk_type != 0x4E4F534A:
        raise ValueError("Invalid GLB header/chunk: " + path.name)
    doc = json.loads(data[20:20 + chunk_len])
    return {
        "file": path.relative_to(ROOT).as_posix(), "bytes": len(data),
        "meshCount": len(doc.get("meshes", [])), "materialCount": len(doc.get("materials", [])),
        "imageCount": len(doc.get("images", [])), "skinCount": len(doc.get("skins", [])),
        "jointCount": sum(len(s.get("joints", [])) for s in doc.get("skins", [])),
        "animations": [{"name": a.get("name"), "channels": len(a.get("channels", [])), "samplers": len(a.get("samplers", []))} for a in doc.get("animations", [])],
    }


runtime = [ASSETS / name for name in ("field-operative-rig.glb", "power-console.glb", "gate-a.glb", "grey-hive-first-flow-room.glb")]
sources = [SOURCE / name for name in ("build_assets.py", "build_manifest.py", "validate_babylon_import.js", "validate_renderer_view.js", "validate_gltf.js", "README.md", "field-operative-rig.blend", "power-console.blend", "gate-a.blend", "grey-hive-first-flow-room.blend", "grey-hive-surface-atlas.png")]
loader = ROOT / "server-rs" / "release-ui" / "vendor" / "babylonjs.loaders.min.js"
loader_license = ROOT / "server-rs" / "release-ui" / "vendor" / "BABYLON-LOADERS-LICENSE.txt"
core = ROOT / "server-rs" / "release-ui" / "vendor" / "babylon.js"
core_text = core.read_text(encoding="utf-8", errors="ignore")
core_version = re.search(r'Version="([0-9.]+)"', core_text)

manifest = {
    "package": "GH_ART_B",
    "baselineSha": "51533dfce9c1e88de51860c94c6b1b4ea16aee55",
    "coordinateContract": "+Y up, +Z forward in runtime; metres; decorative assets do not own collision",
    "runtimeFiles": [digest(p) for p in runtime] + [digest(loader), digest(loader_license), digest(core)],
    "editableSources": [digest(p) for p in sources],
    "glbStructure": [glb_summary(p) for p in runtime],
    "dependencies": {
        "babylonCore": {"version": core_version.group(1) if core_version else "unknown", "license": "Apache-2.0", "sha256": hashlib.sha256(core.read_bytes()).hexdigest()},
        "babylonjsLoaders": {"version": "9.26.2", "license": "Apache-2.0", "packageTarballIntegrity": "sha512-c3pC2EtsnEkcIQYDdNLkuX4cm2nOzNtC8QB7jZBivhNpIWJz/HzCn/UhcKA+MO5R2u1FXoz+U1lS7tuQMscXFw==", "loaderSha256": hashlib.sha256(loader.read_bytes()).hexdigest(), "source": "https://registry.npmjs.org/babylonjs-loaders/-/babylonjs-loaders-9.26.2.tgz"},
        "gltfValidator": {"version": "2.0.0-dev.3.10", "license": "Apache-2.0", "distIntegrity": "sha512-odJ4k0tRkGXiDGn78yDBg+fBbAIvBnXxh3RwAta0emSxGtyagFE8B4xELB1oYe3S5RD8Ci3uZAsZaascH2LAEQ==", "source": "https://registry.npmjs.org/gltf-validator"},
        "blender": {"version": "5.2.2 LTS", "license": "GPL-3.0-or-later", "officialPortableZipSha256": "3849d17a682cba006075aaa3f3597ecb5c9c30ec31035b2e092c53e40679b535", "source": "https://download.blender.org/release/Blender5.2/blender-5.2.2.sha256"},
    },
    "authoredAssetLicense": "CC BY-SA 4.0; self-authored mesh and procedural texture; no external visual assets",
    "validation": {
        "nodeSyntax": "pass",
        "babylonNullEngineImport": "pass; all four GLBs loaded via SceneLoader.LoadAssetContainerAsync; 1 skin and three named clips imported; each animation group started, rendered one NullEngine frame, and stopped",
        "formalRendererViewFixture": "see renderer-view-fixture-smoke.json; verifies Rust-shaped actor placement, closed/open gate slide from DTO, and Idle/Walk/Run selection from velocityMps",
        "khronosGltfValidator": "see gltf-validation.json",
        "formalTauri": "pending CUA/sky screenshot and joint Gate A visual/collision smoke; do not claim accepted yet",
        "blenderExporterAdvisory": "Blender reports the skinned mesh is selected by its Armature modifier rather than parent relation; exported GLB has one skin, 16 joints, three clips and zero Khronos errors/warnings.",
    },
}
(OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(OUT / "manifest.json")
