"""Clean a TRELLIS GLB, bake its colour atlas, rig, export and capture it.

blender -b --factory-startup -P tools/3d/chars/blender/prepare.py -- INPUT OUTPUT_DIR SUBJECT
SUBJECT: man, woman, child, goat, aurochs. Input must be upright and face Blender -Y.
"""
import json
import sys
from pathlib import Path

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from render import capture
from rig import bind, clips, skeleton

HEIGHTS = {"man": 1.7, "woman": 1.7, "child": 1.2, "goat": .75, "aurochs": 1.6}


def cleanup(source: Path, subject: str) -> bpy.types.Object:
    """Join mesh parts, decimate, apply transforms and place ground-centred origin."""
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    bpy.ops.import_scene.gltf(filepath=str(source))
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    assert meshes, "Input GLB has no mesh"
    bpy.ops.object.select_all(action="DESELECT")
    for mesh in meshes:
        mesh.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    mesh = bpy.context.object
    mesh.parent = None
    mesh.modifiers.clear()
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    points = [v.co for v in mesh.data.vertices]
    lo = Vector(tuple(min(p[i] for p in points) for i in range(3)))
    hi = Vector(tuple(max(p[i] for p in points) for i in range(3)))
    assert hi.z > lo.z, "Zero-height input"
    # shortcut: quadruped shoulder plane is estimated from the central trunk; pass anatomically aligned raw meshes.
    shoulder = max(p.z for p in points if abs(p.y-(lo.y+hi.y)/2) < (hi.y-lo.y)*.15) if subject in ("goat", "aurochs") else hi.z
    scale = HEIGHTS[subject]/(shoulder-lo.z)
    for vertex in mesh.data.vertices:
        vertex.co = (vertex.co-Vector(((lo.x+hi.x)/2, (lo.y+hi.y)/2, lo.z)))*scale
    mesh.name = subject
    tris = sum(len(p.vertices)-2 for p in mesh.data.polygons)
    target = 2400 if subject in ("man", "woman", "child") else 2000
    if tris > target:
        modifier = mesh.modifiers.new("Game budget", "DECIMATE")
        modifier.ratio = target/tris
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    actual = sum(len(p.vertices)-2 for p in mesh.data.polygons)
    minimum, maximum = (1500, 3000) if subject in ("man", "woman", "child") else (1000, 2500)
    assert minimum <= actual <= maximum, f"Triangle budget failed: {actual} outside {minimum}..{maximum}"
    bpy.context.scene.cursor.location = (0, 0, 0)
    bpy.ops.object.origin_set(type="ORIGIN_CURSOR")
    return mesh


def atlas(mesh: bpy.types.Object, out: Path) -> None:
    """Bake diffuse colour (without lighting) onto a new 512-square UV atlas."""
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 1
    source_uv = mesh.data.uv_layers.active.name
    for material in mesh.data.materials:
        material.use_nodes = True
        source = material.node_tree.nodes.new("ShaderNodeUVMap")
        source.uv_map = source_uv
        for node in list(material.node_tree.nodes):
            if node.type == "TEX_IMAGE" and not node.inputs["Vector"].is_linked:
                material.node_tree.links.new(source.outputs["UV"], node.inputs["Vector"])
    uv = mesh.data.uv_layers.new(name="Atlas512")
    mesh.data.uv_layers.active = uv
    uv.active_render = True
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(island_margin=.02)
    bpy.ops.object.mode_set(mode="OBJECT")
    image = bpy.data.images.new("Atlas512", width=512, height=512, alpha=True)
    for material in mesh.data.materials:
        material.use_nodes = True
        node = material.node_tree.nodes.new("ShaderNodeTexImage")
        node.image = image
        material.node_tree.nodes.active = node
    scene.render.bake.use_pass_direct = False
    scene.render.bake.use_pass_indirect = False
    scene.render.bake.use_pass_color = True
    bpy.ops.object.bake(type="DIFFUSE", margin=4)
    image.filepath_raw = str(out/"atlas.png")
    image.file_format = "PNG"
    image.save()
    material = bpy.data.materials.new("BakedAtlas")
    material.use_nodes = True
    texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    texture.image = image
    material.node_tree.links.new(texture.outputs["Color"], material.node_tree.nodes.get("Principled BSDF").inputs["Base Color"])
    mesh.data.materials.clear()
    mesh.data.materials.append(material)
    for polygon in mesh.data.polygons:
        polygon.material_index = 0
    for layer in list(mesh.data.uv_layers):
        if layer.name != "Atlas512":
            mesh.data.uv_layers.remove(layer)
    mesh.data.uv_layers.active_index = 0
    mesh.data.uv_layers[0].active_render = True


def main() -> None:
    source, destination, subject = sys.argv[sys.argv.index("--")+1:]
    assert subject in HEIGHTS, subject
    out = Path(destination).resolve()
    assert Path(source).resolve() != (out/f"{subject}.glb").resolve(), "Raw input and export destination must differ"
    out.mkdir(parents=True, exist_ok=True)
    mesh = cleanup(Path(source).resolve(), subject)
    atlas(mesh, out)
    quadruped = subject in ("goat", "aurochs")
    arm = skeleton(HEIGHTS[subject], quadruped)
    bind(mesh, arm)
    names = clips(arm, quadruped, mesh)
    bpy.ops.object.select_all(action="DESELECT")
    mesh.select_set(True)
    arm.select_set(True)
    arm.animation_data.action = None
    for track in arm.animation_data.nla_tracks:
        track.mute = False
    bpy.ops.export_scene.gltf(filepath=str(out/f"{subject}.glb"), export_format="GLB", use_selection=True,
                              export_animation_mode="NLA_TRACKS", export_force_sampling=True)
    for track in arm.animation_data.nla_tracks:
        track.mute = True
    capture(mesh, arm, out)
    receipt = {"subject": subject, "triangles": sum(len(p.vertices)-2 for p in mesh.data.polygons),
               "texture": "512x512 atlas.png", "bones": [b.name for b in arm.data.bones], "clips": names,
               "front": "Godot +Z / Blender -Y", "rig": "Blender automatic weights", "source": str(Path(source).resolve())}
    (out/"receipt.json").write_text(json.dumps(receipt, indent=2)+"\n")
    print("M3D2_PREPARE_DONE", subject)


if __name__ == "__main__":
    main()
