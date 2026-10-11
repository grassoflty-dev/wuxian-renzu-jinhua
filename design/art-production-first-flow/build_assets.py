"""Build the self-authored, editable Grey Hive first-flow Blender assets.

Run with Blender 5.2.2: blender --background --python build_assets.py
All scene dimensions are authored in Blender Z-up; the glTF exporter converts
them to glTF's Y-up coordinates. No external meshes or image sources are used.
"""
import bpy
import math
import os
import random
from array import array
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
SOURCE_DIR = os.path.dirname(__file__)
ASSET_DIR = os.path.join(ROOT, "server-rs", "release-ui", "assets", "grey-hive-first-flow")
os.makedirs(ASSET_DIR, exist_ok=True)


def clear_scene():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    # Materials are shared across the four deliverables and must survive scene resets.
    for datablocks in (bpy.data.meshes, bpy.data.curves, bpy.data.armatures, bpy.data.cameras, bpy.data.lights):
        for item in list(datablocks):
            if item.users == 0:
                datablocks.remove(item)


def material(name, color, metallic=0.0, roughness=0.65, emission=None, emission_strength=0.0):
    m = bpy.data.materials.new(name)
    m.diffuse_color = (*color, 1.0)
    m.use_nodes = True
    bs = m.node_tree.nodes.get("Principled BSDF")
    bs.inputs["Base Color"].default_value = (*color, 1.0)
    bs.inputs["Metallic"].default_value = metallic
    bs.inputs["Roughness"].default_value = roughness
    if emission:
        bs.inputs["Emission Color"].default_value = (*emission, 1.0)
        bs.inputs["Emission Strength"].default_value = emission_strength
    return m


def image_texture_material(name, color, filepath, metallic=0.35, roughness=0.62):
    m = material(name, color, metallic, roughness)
    image = bpy.data.images.load(filepath, check_existing=True)
    image.pack()
    tex = m.node_tree.nodes.new("ShaderNodeTexImage")
    tex.image = image
    tex.label = "Self-authored metal surface atlas"
    m.node_tree.links.new(tex.outputs["Color"], m.node_tree.nodes.get("Principled BSDF").inputs["Base Color"])
    return m


def make_atlas():
    path = os.path.join(SOURCE_DIR, "grey-hive-surface-atlas.png")
    if os.path.exists(path):
        return path
    random.seed(5202)
    w = h = 256
    pixels = array("f")
    for y in range(h):
        for x in range(w):
            # Fine painted-steel grain and a restrained edge-darkening gradient.
            n = random.random() * 0.12 - 0.06
            band = 0.035 * math.sin(y * 0.22) + 0.018 * math.sin(x * 0.11)
            edge = min(x, y, w - 1 - x, h - 1 - y) / 128.0
            v = max(0.08, min(0.78, 0.36 + n + band - (1.0 - edge) * 0.045))
            pixels.extend((v, v * 1.025, v * 1.04, 1.0))
    image = bpy.data.images.new("Grey Hive painted steel grain", width=w, height=h, alpha=False)
    image.pixels.foreach_set(pixels)
    image.file_format = "PNG"
    image.filepath_raw = path
    image.save()
    bpy.data.images.remove(image)
    return path


def cube(name, loc, scale, mat, bevel=0.04, rotation=None):
    bpy.ops.mesh.primitive_cube_add(size=1, location=loc)
    o = bpy.context.object
    o.name = name
    o.dimensions = scale
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if rotation:
        o.rotation_euler = rotation
    if mat:
        o.data.materials.append(mat)
    if bevel:
        mod = o.modifiers.new("Machined edge radii", "BEVEL")
        mod.width = bevel
        mod.segments = 2
        mod.affect = "EDGES"
        o.modifiers.new("Weighted corner normals", "WEIGHTED_NORMAL")
    return o


def cylinder(name, loc, radius, depth, mat, vertices=20, rotation=None):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=loc)
    o = bpy.context.object
    o.name = name
    if rotation:
        o.rotation_euler = rotation
    if mat:
        o.data.materials.append(mat)
    bevel = o.modifiers.new("Turned edge", "BEVEL")
    bevel.width = min(radius * 0.16, 0.025)
    bevel.segments = 2
    o.modifiers.new("Weighted normals", "WEIGHTED_NORMAL")
    return o


def smart_uv(objects):
    mesh_objects = [o for o in objects if o and o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in mesh_objects:
        o.select_set(True)
    if mesh_objects:
        bpy.context.view_layer.objects.active = mesh_objects[0]
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.select_all(action="SELECT")
        bpy.ops.uv.smart_project(island_margin=0.015)
        bpy.ops.object.mode_set(mode="OBJECT")


def export_glb(path, animations=False):
    bpy.ops.object.select_all(action="SELECT")
    bpy.context.view_layer.objects.active = bpy.context.selected_objects[0] if bpy.context.selected_objects else None
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", use_selection=True,
                              export_apply=True, export_animations=animations,
                              export_skins=True, export_materials="EXPORT",
                              export_image_format="AUTO", export_yup=True)


def save_blend(path):
    bpy.ops.file.pack_all()
    bpy.ops.wm.save_as_mainfile(filepath=path)


def make_console(mats, atlas):
    clear_scene()
    console = []
    textured_steel = image_texture_material("Painted gunmetal | atlas", (0.24, 0.28, 0.30), atlas, 0.58, 0.48)
    dark = mats["dark"]
    ceramic = mats["ceramic"]
    cyan = mats["cyan"]
    amber = mats["amber"]
    # Footprint 1.45 x 0.88 m; top surface sits at 1.15 m.
    console.append(cube("Console | floor plinth", (0, 0, 0.10), (1.44, 0.90, 0.20), dark, 0.08))
    for x in (-0.56, 0.56):
        for y in (-0.30, 0.30):
            console.append(cube("Console | isolation foot", (x, y, 0.04), (0.20, 0.16, 0.08), textured_steel, 0.025))
    console.append(cube("Console | pedestal", (0, 0.03, 0.58), (1.12, 0.70, 0.88), textured_steel, 0.10))
    console.append(cube("Console | armored fascia", (0, -0.33, 0.63), (0.94, 0.08, 0.67), dark, 0.07))
    # The glazed operator face is oriented toward local -Y (game +Z after glTF export).
    screen = cube("Console | status glass", (0.02, -0.385, 0.94), (0.63, 0.035, 0.31), ceramic, 0.025)
    screen.data.materials.append(cyan)
    console.append(screen)
    console.append(cube("Console | display bezel", (0.02, -0.41, 0.94), (0.71, 0.045, 0.38), dark, 0.035))
    # Small bevelled bezel frame and readable color blocks remain geometry, so no external font asset is needed.
    console.append(cube("Console | live strip", (-0.36, -0.40, 0.94), (0.035, 0.025, 0.24), cyan, 0.01))
    console.append(cube("Console | standby lamp", (0.38, -0.405, 0.68), (0.07, 0.035, 0.07), amber, 0.025))
    for i in range(4):
        x = -0.32 + i * 0.21
        console.append(cube("Console | breaker %02d" % (i + 1), (x, -0.395, 0.48), (0.12, 0.07, 0.15), ceramic, 0.025))
        console.append(cube("Console | breaker lamp %02d" % (i + 1), (x, -0.438, 0.52), (0.045, 0.02, 0.025), cyan if i == 0 else amber, 0.008))
    # Sides: cable glands and armored trunking.
    for z in (0.28, 0.50):
        console.append(cylinder("Console | cable gland", (0.58, 0.02, z), 0.075, 0.08, dark, rotation=(0, math.pi / 2, 0)))
    console.append(cube("Console | rear service cover", (0, 0.40, 0.58), (0.76, 0.06, 0.50), dark, 0.04))
    for i in range(3):
        console.append(cube("Console | vent %02d" % i, (0, 0.366, 0.44 + i * 0.09), (0.48, 0.018, 0.018), textured_steel, 0.006))
    smart_uv(console)
    save_blend(os.path.join(SOURCE_DIR, "power-console.blend"))
    export_glb(os.path.join(ASSET_DIR, "power-console.glb"))


def make_room(mats):
    clear_scene()
    c = mats["concrete"]
    dark = mats["dark"]
    steel = mats["steel"]
    cyan = mats["cyan"]
    amber = mats["amber"]
    pieces = []
    def add(name, loc, dims, mat, bevel=0.03):
        pieces.append(cube(name, loc, dims, mat, bevel))
    # Blender Z-up; local -Y maps to the level's +Z. Room dimensions follow the 1.2 contract.
    # Maintenance room: x[-4,4], game z[-3,6]. Main floor, corridor floor, and axial entry floor.
    add("Maintenance | floor slab", (0, -1.5, -0.12), (8.0, 9.0, 0.24), c, 0.03)
    add("Compression corridor | floor", (0, -8.0, -0.12), (3.0, 4.0, 0.24), c, 0.03)
    add("Central shaft entry | floor", (0, -15.0, -0.12), (16.0, 10.0, 0.24), c, 0.03)
    # Room side walls, with the north aperture to the corridor left clear.
    for x in (-4.15, 4.15):
        add("Maintenance | side wall", (x, -1.5, 1.65), (0.30, 9.0, 3.3), c, 0.04)
    # West wall and a power service recess.
    add("Maintenance | west wall", (0, 6.15, 1.65), (8.6, 0.30, 3.3), c, 0.04)
    # North wall segments define corridor width 3m (opening centered).
    for x in (-3.0, 3.0):
        add("Maintenance | north return", (x, -6.15, 1.65), (2.3, 0.30, 3.3), c, 0.04)
    # Corridor side walls up to gate and the shaft opening.
    for x in (-1.62, 1.62):
        add("Compression corridor | wall", (x, -8.0, 1.65), (0.24, 4.0, 3.3), c, 0.035)
    # Gate frame around 2.8m opening, centered at game z=8 (Blender y=-8).
    for x in (-1.55, 1.55):
        add("Gate A | jamb", (x, -8.0, 1.52), (0.22, 0.32, 3.04), steel, 0.04)
    add("Gate A | lintel", (0, -8.0, 3.10), (3.30, 0.32, 0.28), steel, 0.04)
    # Shaft sides begin after the southern gate, giving the entrance its wider blockout.
    for x in (-8.10, 8.10):
        add("Axial entry | side bulkhead", (x, -15.0, 1.65), (0.30, 10.0, 3.3), c, 0.04)
    # Recessed observation window and cable/pipe services.
    add("Observation | inset frame", (3.94, 1.4, 2.0), (0.08, 2.1, 1.2), dark, 0.04)
    add("Observation | cool glass", (3.88, 1.4, 2.0), (0.035, 1.82, 0.88), cyan, 0.02)
    for x in (-3.50, 3.50):
        add("Cable tray | vertical", (x, 0.1, 2.90), (0.14, 7.0, 0.14), dark, 0.03)
    for y in (-5.2, -6.0, -6.8):
        add("Cable tray | bridge", (0, y, 3.0), (3.1, 0.16, 0.13), steel, 0.02)
    # Wall-mounted warning light strips are unambiguous emissive accents.
    for x in (-3.85, 3.85):
        add("Wayfinding | cyan strip", (x, 0.7, 2.9), (0.05, 1.1, 0.06), cyan, 0.015)
    add("Gate status | amber beacon", (0, -8.0, 3.33), (0.50, 0.08, 0.08), amber, 0.025)
    # Pillars and floor service covers establish industrial scale without obstructing the path.
    for x in (-3.3, 3.3):
        for y in (-1.9, 3.6):
            add("Maintenance | corner pier", (x, y, 1.62), (0.30, 0.42, 3.24), steel, 0.035)
    for x in (-2.4, -0.8, 0.8, 2.4):
        add("Floor | cable hatch", (x, 2.9, 0.02), (0.62, 0.38, 0.055), dark, 0.025)
    smart_uv(pieces)
    save_blend(os.path.join(SOURCE_DIR, "grey-hive-first-flow-room.blend"))
    export_glb(os.path.join(ASSET_DIR, "grey-hive-first-flow-room.glb"))


def make_gate(mats):
    clear_scene()
    steel, dark, amber, cyan = mats["steel"], mats["dark"], mats["amber"], mats["cyan"]
    parts = []
    parts.append(cube("Gate A | armored leaf", (0, 0, 1.50), (2.72, 0.18, 2.96), steel, 0.06))
    for i in range(5):
        x = -1.05 + i * 0.525
        parts.append(cube("Gate A | vertical rib %02d" % i, (x, -0.105, 1.50), (0.10, 0.035, 2.70), dark, 0.025))
    parts.append(cube("Gate A | status lens", (0, -0.13, 0.28), (0.34, 0.04, 0.12), amber, 0.03))
    parts.append(cube("Gate A | energy seam", (0, -0.13, 2.75), (2.2, 0.03, 0.035), cyan, 0.012))
    smart_uv(parts)
    save_blend(os.path.join(SOURCE_DIR, "gate-a.blend"))
    export_glb(os.path.join(ASSET_DIR, "gate-a.glb"))


def add_bone(arm, name, head, tail, parent=None, connected=False):
    b = arm.edit_bones.new(name)
    b.head = head
    b.tail = tail
    if parent:
        b.parent = arm.edit_bones[parent]
        b.use_connect = connected
    return b


def make_character(mats, atlas):
    clear_scene()
    arm_data = bpy.data.armatures.new("Field operative | edit me")
    arm = bpy.data.objects.new("FieldOperative_Rig", arm_data)
    bpy.context.collection.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    arm.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    # Blender Z-up, face toward local -Y.
    add_bone(arm_data, "root", (0, 0, 0), (0, 0, 0.92))
    add_bone(arm_data, "pelvis", (0, 0, 0.86), (0, 0, 1.20), "root")
    add_bone(arm_data, "spine", (0, 0, 1.12), (0, 0, 1.72), "pelvis")
    add_bone(arm_data, "head", (0, 0, 1.68), (0, 0, 2.02), "spine")
    add_bone(arm_data, "upper_arm.L", (0.26, 0, 1.62), (0.48, 0, 1.27), "spine")
    add_bone(arm_data, "forearm.L", (0.48, 0, 1.27), (0.52, -0.02, 0.91), "upper_arm.L", True)
    add_bone(arm_data, "hand.L", (0.52, -0.02, 0.91), (0.52, -0.07, 0.78), "forearm.L", True)
    add_bone(arm_data, "upper_arm.R", (-0.26, 0, 1.62), (-0.48, 0, 1.27), "spine")
    add_bone(arm_data, "forearm.R", (-0.48, 0, 1.27), (-0.52, -0.02, 0.91), "upper_arm.R", True)
    add_bone(arm_data, "hand.R", (-0.52, -0.02, 0.91), (-0.52, -0.07, 0.78), "forearm.R", True)
    add_bone(arm_data, "thigh.L", (0.14, 0, 0.99), (0.16, 0, 0.56), "pelvis")
    add_bone(arm_data, "shin.L", (0.16, 0, 0.56), (0.16, 0, 0.15), "thigh.L", True)
    add_bone(arm_data, "foot.L", (0.16, 0, 0.15), (0.16, -0.22, 0.08), "shin.L", True)
    add_bone(arm_data, "thigh.R", (-0.14, 0, 0.99), (-0.16, 0, 0.56), "pelvis")
    add_bone(arm_data, "shin.R", (-0.16, 0, 0.56), (-0.16, 0, 0.15), "thigh.R", True)
    add_bone(arm_data, "foot.R", (-0.16, 0, 0.15), (-0.16, -0.22, 0.08), "shin.R", True)
    bpy.ops.object.mode_set(mode="OBJECT")
    arm.select_set(False)
    rigged = []

    def part(name, loc, dims, mat, bone, bevel=0.04):
        o = cube(name, loc, dims, mat, bevel)
        vg = o.vertex_groups.new(name=bone)
        vg.add(list(range(len(o.data.vertices))), 1.0, "REPLACE")
        mod = o.modifiers.new("Armature deformation", "ARMATURE")
        mod.object = arm
        o.parent = arm
        rigged.append(o)
        return o

    outer = image_texture_material("Operative | textile composite atlas", (0.18, 0.24, 0.27), atlas, 0.10, 0.86)
    dark = mats["dark"]
    ceramic = mats["ceramic"]
    amber = mats["amber"]
    cyan = mats["cyan"]
    # Anatomically readable technical survey dummy, deliberately neutral and unnamed.
    part("Operative | pelvis shell", (0, 0, 0.98), (0.48, 0.30, 0.31), outer, "pelvis")
    part("Operative | chest armor", (0, 0, 1.48), (0.62, 0.34, 0.56), outer, "spine")
    part("Operative | collar", (0, 0, 1.75), (0.32, 0.30, 0.14), dark, "spine")
    part("Operative | helmet", (0, 0.01, 1.92), (0.34, 0.34, 0.34), ceramic, "head")
    part("Operative | face shield", (0, -0.178, 1.92), (0.25, 0.035, 0.12), cyan, "head", 0.025)
    part("Operative | chest signal", (0, -0.19, 1.48), (0.13, 0.035, 0.13), amber, "spine", 0.025)
    for side, suffix in ((1, "L"), (-1, "R")):
        part("Operative | upper arm " + suffix, (side * 0.39, 0, 1.45), (0.24, 0.31, 0.45), outer, "upper_arm." + suffix)
        part("Operative | elbow guard " + suffix, (side * 0.49, -0.01, 1.22), (0.24, 0.34, 0.14), ceramic, "forearm." + suffix)
        part("Operative | forearm " + suffix, (side * 0.51, -0.01, 1.04), (0.20, 0.29, 0.34), outer, "forearm." + suffix)
        part("Operative | glove " + suffix, (side * 0.52, -0.04, 0.84), (0.18, 0.23, 0.18), dark, "hand." + suffix)
        part("Operative | thigh " + suffix, (side * 0.15, 0, 0.77), (0.25, 0.34, 0.45), outer, "thigh." + suffix)
        part("Operative | knee " + suffix, (side * 0.16, -0.03, 0.55), (0.25, 0.36, 0.16), ceramic, "shin." + suffix)
        part("Operative | shin " + suffix, (side * 0.16, 0, 0.34), (0.20, 0.30, 0.37), outer, "shin." + suffix)
        part("Operative | boot " + suffix, (side * 0.16, -0.10, 0.13), (0.24, 0.42, 0.20), dark, "foot." + suffix)
    # Join parts while preserving deform groups/modifiers; this is one clean skinned mesh.
    bpy.ops.object.select_all(action="DESELECT")
    for o in rigged:
        o.select_set(True)
    bpy.context.view_layer.objects.active = rigged[0]
    bpy.ops.object.join()
    body = bpy.context.object
    body.name = "FieldOperative_Body_SKINNED"
    body.parent = None
    # Keep the skinned mesh and skeleton as sibling roots. glTF parent
    # transforms on a skinned-mesh node are ignored by spec.
    if not any(m.type == "ARMATURE" for m in body.modifiers):
        mod = body.modifiers.new("Armature deformation", "ARMATURE")
        mod.object = arm
    smart_uv([body])
    # Real joint rotations, authored as three loopable Blender actions.
    def action(name, phase, stride, arm_swing):
        # Blender 5.2 keeps a layered Action active by default; clear it so
        # each pass becomes a separately named action rather than overwriting Run.
        arm.animation_data.action = None
        bpy.ops.object.select_all(action="DESELECT")
        arm.select_set(True)
        bpy.context.view_layer.objects.active = arm
        bpy.ops.object.mode_set(mode="POSE")
        bpy.context.scene.frame_start = 1
        bpy.context.scene.frame_end = 25
        for bone_name in ("thigh.L", "thigh.R", "shin.L", "shin.R", "upper_arm.L", "upper_arm.R", "forearm.L", "forearm.R", "spine", "head"):
            bone = arm.pose.bones[bone_name]
            bone.rotation_mode = "XYZ"
        for frame in (1, 7, 13, 19, 25):
            t = (frame - 1) / 24.0 * math.tau + phase
            for s, side in ((1, "L"), (-1, "R")):
                leg = arm.pose.bones["thigh." + side]
                shin = arm.pose.bones["shin." + side]
                upper = arm.pose.bones["upper_arm." + side]
                fore = arm.pose.bones["forearm." + side]
                leg.rotation_euler.x = stride * math.sin(t) * s
                shin.rotation_euler.x = max(0.0, stride * 0.8 * math.sin(t - 0.45) * s)
                upper.rotation_euler.x = arm_swing * math.sin(t + math.pi) * s
                fore.rotation_euler.x = -0.12 + abs(arm_swing) * 0.35 * max(0.0, math.sin(t))
                for b in (leg, shin, upper, fore):
                    b.keyframe_insert(data_path="rotation_euler", frame=frame, group=b.name)
            spine = arm.pose.bones["spine"]
            spine.rotation_euler.x = (0.025 if name == "Idle" else 0.045) * math.sin(t)
            spine.keyframe_insert(data_path="rotation_euler", frame=frame, group="spine")
            head = arm.pose.bones["head"]
            head.rotation_euler.x = -spine.rotation_euler.x * 0.7
            head.keyframe_insert(data_path="rotation_euler", frame=frame, group="head")
        bpy.ops.object.mode_set(mode="OBJECT")
        act = arm.animation_data.action
        act.name = name
        # The glTF animation groups are looped by Babylon's animation controller.
    arm.animation_data_create()
    action("Idle", 0.0, 0.018, 0.018)
    action("Walk", 0.0, 0.40, 0.26)
    action("Run", 0.0, 0.68, 0.46)
    # Keep distinct named actions for source editing; glTF export emits NLA tracks.
    for act in list(bpy.data.actions):
        arm.animation_data.action = act
        track = arm.animation_data.nla_tracks.new()
        track.name = act.name
        strip = track.strips.new(act.name, 1, act)
        strip.action_frame_start = 1
        strip.action_frame_end = 25
        strip.frame_end = 25
        track.mute = False
    arm.animation_data.action = None
    arm.hide_viewport = False
    # Add a tiny test key to ensure the native source opens with Idle selected.
    save_blend(os.path.join(SOURCE_DIR, "field-operative-rig.blend"))
    export_glb(os.path.join(ASSET_DIR, "field-operative-rig.glb"), animations=True)


def build():
    # One restrained neutral/cool material palette serves the first-flow sample.
    mats = {
        "concrete": material("Neutral concrete | cool cast", (0.31, 0.35, 0.36), 0.05, 0.94),
        "steel": material("Dark structural steel", (0.105, 0.135, 0.15), 0.72, 0.48),
        "dark": material("Graphite rubber and recess", (0.035, 0.052, 0.06), 0.08, 0.84),
        "ceramic": material("Grey ceramic paint", (0.42, 0.48, 0.49), 0.15, 0.58),
        "cyan": material("Signal cyan | emissive", (0.04, 0.42, 0.50), 0.15, 0.38, (0.05, 0.55, 0.68), 2.1),
        "amber": material("Warning amber | emissive", (0.72, 0.33, 0.06), 0.10, 0.42, (1.0, 0.22, 0.015), 1.25),
    }
    atlas = make_atlas()
    make_console(mats, atlas)
    make_room(mats)
    make_gate(mats)
    make_character(mats, atlas)


if __name__ == "__main__":
    build()
