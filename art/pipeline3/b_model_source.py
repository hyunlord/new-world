# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by Blender's build_b_model.py (bpy supplied by Blender).
"""Retain and reshape the CC0 Kenney skinned mesh, partitioning true anatomy."""

from __future__ import annotations

import bpy
from mathutils import Vector


def material(
    name: str, region: int, color: tuple[float, float, float]
) -> bpy.types.Material:
    """Make a render-neutral material carrying the contract region id."""
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*color, 1.0)
    mat.pass_index = region
    mat.use_nodes = True
    mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (
        *color,
        1.0,
    )
    mat.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
    return mat


def assign_layer(obj: bpy.types.Object, layer: str) -> None:
    """Put an object in exactly one named export collection."""
    collection = bpy.data.collections.get(layer)
    if collection is None:
        collection = bpy.data.collections.new(layer)
        bpy.context.scene.collection.children.link(collection)
    for old in list(obj.users_collection):
        old.objects.unlink(obj)
    collection.objects.link(obj)
    obj["layer"] = layer


def map_point(point: Vector, bone: str) -> Vector:
    """Sculpt source vertex positions without replacing source topology."""
    x, y, z = point
    if bone == "head":
        return Vector((x * 1.02, y * 0.78, 0.70 + (z - 0.3432498) * (0.30 / 0.3180752)))
    if bone.startswith("arm-"):
        sign = 1 if x >= 0 else -1
        hand = abs(x) > 0.23
        return Vector(
            (
                sign * (0.23 + (abs(x) - 0.0998968) * 1.02),
                (y - 0.01725) * (0.79 if hand else 0.60),
                0.64 + (z - 0.28775) * (0.96 if hand else 0.62),
            )
        )
    if bone.startswith("leg-"):
        sign = 1 if x >= 0 else -1
        foot = z < 0.081
        return Vector(
            (
                sign * 0.12 + (x - sign * 0.08357) * (1.40 if foot else 1.1),
                (y - 0.02875) * (1.05 if foot else 0.75),
                z * (0.115 / 0.08) if foot else 0.115 + (z - 0.08) * (0.205 / 0.09625),
            )
        )
    return Vector(
        (x * 1.65, (y - 0.02875) * 0.9, 0.32 + (z - 0.17625) * (0.38 / 0.167))
    )


def anatomy(bone: str, point: Vector) -> int:
    """Use source weight groups and source-space joints for semantic partition."""
    if bone == "head":
        return 1
    left = bone.endswith("left")
    if bone.startswith("arm-"):
        return (2 if left else 3) if abs(point.x) > 0.23 else (6 if left else 7)
    if bone.startswith("leg-"):
        return (4 if left else 5) if point.z < 0.081 else (8 if left else 9)
    return 10


def reshape_source(rig: bpy.types.Object) -> list[bpy.types.Object]:
    """Split original polygons, retaining UV-independent geometry and skin weights."""
    skin = material("skin_neutral", 1, (0.55, 0.55, 0.55))
    atlas = bpy.data.images["colormap"]
    pixels = list(atlas.pixels)
    width, height = atlas.size
    output = []
    for source_name in ("body-mesh", "head-mesh"):
        source = bpy.data.objects[source_name]
        mesh = source.data
        groups = {g.index: g.name for g in source.vertex_groups}
        partitions: dict[int, list[bpy.types.MeshPolygon]] = {}
        for poly in mesh.polygons:
            uv = mesh.uv_layers.active.data[poly.loop_start].uv
            offset = (int(uv.y * height) * width + int(uv.x * width)) * 4
            red, green, blue = pixels[offset : offset + 3]
            # The head's skin palette is brown; remove black hair and blue-grey glasses.
            if source_name == "head-mesh" and not (
                red > green * 1.25 and red > blue * 1.5
            ):
                continue
            vertex = mesh.vertices[poly.vertices[0]]
            bone = groups[max(vertex.groups, key=lambda g: g.weight).group]
            center = sum((mesh.vertices[i].co for i in poly.vertices), Vector()) / len(
                poly.vertices
            )
            partitions.setdefault(anatomy(bone, center), []).append(poly)
        for label, polygons in sorted(partitions.items()):
            indices = sorted({i for p in polygons for i in p.vertices})
            remap = {old: new for new, old in enumerate(indices)}
            coords = []
            bones = []
            for index in indices:
                vertex = mesh.vertices[index]
                bone = groups[max(vertex.groups, key=lambda g: g.weight).group]
                coords.append(map_point(vertex.co, bone))
                bones.append(bone)
            data = bpy.data.meshes.new(f"kenney_anatomy_{label}")
            data.from_pydata(
                coords, [], [[remap[i] for i in p.vertices] for p in polygons]
            )
            data.update()
            obj = bpy.data.objects.new(f"body_anatomy_{label}", data)
            bpy.context.scene.collection.objects.link(obj)
            assign_layer(obj, "BODY")
            obj["anatomical_id"] = label
            obj["source_mesh"] = source_name
            obj["source_polygon_count"] = len(polygons)
            obj.data.materials.append(skin)
            for bone in sorted(set(bones)):
                group = obj.vertex_groups.new(name=bone)
                group.add(
                    [i for i, name in enumerate(bones) if name == bone], 1.0, "REPLACE"
                )
            modifier = obj.modifiers.new("original_kenney_rig", "ARMATURE")
            modifier.object = rig
            obj.parent = rig
            output.append(obj)
        bpy.data.objects.remove(source, do_unlink=True)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="EDIT")
    for bone in rig.data.edit_bones:
        head = bone.head.copy()
        if bone.name == "root":
            bone.head = (0, 0, 0)
        elif bone.name == "head":
            bone.head = (0, 0, 0.70)
        elif bone.name.startswith("arm-"):
            bone.head = (0.23 if head.x > 0 else -0.23, 0, 0.64)
        elif bone.name.startswith("leg-"):
            bone.head = (0.12 if head.x > 0 else -0.12, 0, 0.32)
        else:
            bone.head = (0, 0, 0.32)
        bone.tail = bone.head + Vector((0, 0, 0.15))
    bpy.ops.object.mode_set(mode="OBJECT")
    return output
