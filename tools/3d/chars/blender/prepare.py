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
from rig import bind, carry_load, clips, skeleton
from topology import debris, landmark, weld

HEIGHTS = {"man": 1.7, "woman": 1.7, "child": 1.2, "goat": .75, "aurochs": 1.6}


def cleanup(source: Path, subject: str, force_remesh: bool = False, narrow_withers: bool = False) -> bpy.types.Object:
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
    world = mesh.matrix_world.copy()
    mesh.parent = None
    mesh.matrix_world = world
    mesh.modifiers.clear()
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    mesh["withers_width_fraction"] = .1 if narrow_withers else .4
    # glTF splits vertices at UV/normal seams; weld geometry while retaining per-loop UVs.
    weld(mesh)
    points = [v.co for v in mesh.data.vertices]
    lo = Vector(tuple(min(p[i] for p in points) for i in range(3)))
    hi = Vector(tuple(max(p[i] for p in points) for i in range(3)))
    assert hi.z > lo.z, "Zero-height input"
    shoulder = landmark(mesh, subject in ("goat", "aurochs")).z
    scale = HEIGHTS[subject]/(shoulder-lo.z)
    for vertex in mesh.data.vertices:
        vertex.co = (vertex.co-Vector(((lo.x+hi.x)/2, (lo.y+hi.y)/2, lo.z)))*scale
    mesh.name = subject
    source_mesh = mesh.copy()
    source_mesh.data = mesh.data.copy()
    source_mesh.name = "BakeSource"
    bpy.context.collection.objects.link(source_mesh)
    source_mesh.hide_render = True
    tris = sum(len(p.vertices)-2 for p in mesh.data.polygons)
    target = 2400 if subject in ("man", "woman", "child") else 2000
    if tris > target:
        modifier = mesh.modifiers.new("Game budget", "DECIMATE")
        modifier.ratio = target/tris
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    actual = sum(len(p.vertices)-2 for p in mesh.data.polygons)
    minimum, maximum = (1500, 3000) if subject in ("man", "woman", "child") else (1000, 2500)
    if actual > maximum or force_remesh:
        mesh.data = source_mesh.data.copy()
        mesh.data.remesh_voxel_size = HEIGHTS[subject]/160
        bpy.ops.object.voxel_remesh()
        triangles = sum(len(p.vertices)-2 for p in mesh.data.polygons)
        modifier = mesh.modifiers.new("Remesh game budget", "DECIMATE")
        modifier.ratio = target/triangles
        bpy.ops.object.modifier_apply(modifier=modifier.name)
        mesh["bake_source"] = source_mesh.name
        mesh["cleanup"] = "seam weld + voxel remesh + decimate"
        actual = sum(len(p.vertices)-2 for p in mesh.data.polygons)
    else:
        bpy.data.objects.remove(source_mesh, do_unlink=True)
        mesh["cleanup"] = "seam weld + decimate"
    debris(mesh)
    actual = sum(len(p.vertices)-2 for p in mesh.data.polygons)
    assert minimum <= actual <= maximum, f"Triangle budget failed: {actual} outside {minimum}..{maximum}"
    points = [v.co for v in mesh.data.vertices]
    low = Vector(tuple(min(p[i] for p in points) for i in range(3)))
    high = Vector(tuple(max(p[i] for p in points) for i in range(3)))
    offset = Vector(((low.x+high.x)/2, (low.y+high.y)/2, low.z))
    factor = HEIGHTS[subject]/(landmark(mesh, subject in ("goat", "aurochs")).z-low.z)
    for obj in (mesh, source_mesh) if mesh.get("bake_source") else (mesh,):
        for vertex in obj.data.vertices:
            vertex.co = (vertex.co-offset)*factor
    mesh["height_landmark_xyz"] = list(landmark(mesh, subject in ("goat", "aurochs")))
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
    bpy.ops.object.select_all(action="DESELECT")
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = mesh
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
    source_mesh = bpy.data.objects.get(mesh.get("bake_source", ""))
    scene.render.bake.use_selected_to_active = source_mesh is not None
    if source_mesh is not None:
        source_mesh.hide_render = False
        bpy.ops.object.select_all(action="DESELECT")
        source_mesh.select_set(True)
        mesh.select_set(True)
        bpy.context.view_layer.objects.active = mesh
        scene.render.bake.use_cage = True
        scene.render.bake.cage_extrusion = .04
        scene.render.bake.max_ray_distance = .08
    bpy.ops.object.bake(type="DIFFUSE", margin=4, use_selected_to_active=source_mesh is not None,
                        use_cage=source_mesh is not None, cage_extrusion=.04, max_ray_distance=.08)
    if source_mesh is not None:
        bpy.data.objects.remove(source_mesh, do_unlink=True)
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
    arguments = sys.argv[sys.argv.index("--")+1:]
    narrow_withers = "--narrow-withers" in arguments
    if narrow_withers:
        arguments.remove("--narrow-withers")
    force_remesh = "--force-remesh" in arguments
    if force_remesh:
        arguments.remove("--force-remesh")
    source, destination, subject = arguments
    assert subject in HEIGHTS, subject
    out = Path(destination).resolve()
    assert Path(source).resolve() != (out/f"{subject}.glb").resolve(), "Raw input and export destination must differ"
    assert not out.exists(), "Output directory must be new"
    out.mkdir(parents=True)
    mesh = cleanup(Path(source).resolve(), subject, force_remesh, narrow_withers)
    atlas(mesh, out)
    quadruped = subject in ("goat", "aurochs")
    arm = skeleton(HEIGHTS[subject], quadruped)
    bind(mesh, arm)
    names = clips(arm, quadruped, mesh)
    load = None if quadruped else carry_load(mesh, arm, names)
    for obj in (mesh, arm):
        assert all(abs(v-1) < 1e-6 for v in obj.scale), (obj.name, tuple(obj.scale))
        assert obj.location.length < 1e-6 and obj.rotation_euler.to_matrix().is_identity, obj.name
    bpy.ops.object.select_all(action="DESELECT")
    mesh.select_set(True)
    arm.select_set(True)
    if load:
        load.select_set(True)
        for track in load.animation_data.nla_tracks:
            track.mute = False
    arm.animation_data.action = None
    for track in arm.animation_data.nla_tracks:
        track.mute = False
    bpy.ops.export_scene.gltf(filepath=str(out/f"{subject}.glb"), export_format="GLB", use_selection=True,
                              export_animation_mode="NLA_TRACKS", export_force_sampling=True, export_optimize_animation_size=False, export_optimize_animation_keep_anim_object=True)
    for track in arm.animation_data.nla_tracks:
        track.mute = True
    if load:
        for track in load.animation_data.nla_tracks:
            track.mute = True
    capture(mesh, arm, out)
    receipt = {"subject": subject, "triangles": sum(len(p.vertices)-2 for p in mesh.data.polygons)+(12 if load else 0),
               "carry_load": "12-triangle atlas-textured package, visible only in carry" if load else None,
               "texture": "512x512 atlas.png", "bones": [b.name for b in arm.data.bones], "clips": names,
               "applied_object_transforms": True, "height_reference_m": HEIGHTS[subject],
               "cleanup": mesh["cleanup"],
               "debris_vertices_removed": mesh["debris_vertices_removed"],
               "rest_bounds_xyz": [[min(v.co[i] for v in mesh.data.vertices) for i in range(3)],
                                   [max(v.co[i] for v in mesh.data.vertices) for i in range(3)]],
               "height_landmark_xyz": list(mesh["height_landmark_xyz"]),
               "height_landmark_method": f"front trunk withers band (40% length from nose, central{mesh.get('withers_width_fraction', .4)*100:g}% width)" if quadruped else "crown",
               "weight_method": mesh["weight_method"], "heat_uncovered_vertices": mesh["heat_uncovered_vertices"],
               "shared_motion_transfer": "same names/hierarchy, scale root location by target/source height",
               "front": "Godot +Z / Blender -Y", "rig": "Blender automatic weights", "source": str(Path(source).resolve())}
    (out/"receipt.json").write_text(json.dumps(receipt, indent=2)+"\n")
    print("M3D2_PREPARE_DONE", subject)


if __name__ == "__main__":
    main()
