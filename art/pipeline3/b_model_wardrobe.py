# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by Blender's build_b_model.py.
"""Author simple stone-age accessories over the retained CC0 source body."""

from __future__ import annotations

import math

import bpy
from b_model_source import assign_layer, material


def bind(obj: bpy.types.Object, rig: bpy.types.Object, bone: str) -> None:
    """Rigid skin a new wardrobe mesh to the original skeleton."""
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    obj.select_set(False)
    obj.parent = rig
    group = obj.vertex_groups.new(name=bone)
    group.add(list(range(len(obj.data.vertices))), 1.0, "REPLACE")
    modifier = obj.modifiers.new("rig_skin", "ARMATURE")
    modifier.object = rig


def ring_mesh(
    name: str, rings: list[tuple[float, float, float]], mat: bpy.types.Material
) -> bpy.types.Object:
    """Make a faceted drape from elliptical cross-sections; close both ends."""
    sides = 12
    verts = [
        (rx * math.cos(i * math.tau / sides), ry * math.sin(i * math.tau / sides), z)
        for z, rx, ry in rings
        for i in range(sides)
    ]
    faces = [
        tuple(reversed(range(sides))),
        tuple((len(rings) - 1) * sides + i for i in range(sides)),
    ]
    for row in range(len(rings) - 1):
        for i in range(sides):
            faces.append(
                (
                    row * sides + i,
                    row * sides + (i + 1) % sides,
                    (row + 1) * sides + (i + 1) % sides,
                    (row + 1) * sides + i,
                )
            )
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    obj.data.materials.append(mat)
    return obj


def ellipsoid(
    name: str, center: tuple[float, float, float], size: tuple[float, float, float]
) -> bpy.types.Object:
    """Create a deliberately low-poly accessory, never a replacement body."""
    bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=6, location=center)
    obj = bpy.context.object
    obj.name = name
    obj.scale = size
    return obj


def wardrobe(rig: bpy.types.Object) -> None:
    """Create all named outfit and hair variants at bind pose."""
    cloth = material("hide_primary", 3, (0.49, 0.35, 0.22))
    fur = material("fur_secondary", 4, (0.32, 0.25, 0.18))
    hair = material("hair_charcoal", 2, (0.12, 0.10, 0.08))
    loin = material("loincloth_region5", 5, (0.35, 0.25, 0.16))
    for name, rings, mat, layer, bone in [
        (
            "loincloth",
            [(0.26, 0.18, 0.115), (0.32, 0.22, 0.12), (0.39, 0.225, 0.12)],
            loin,
            "BODY",
            "torso",
        ),
        (
            "hide_wrap",
            [
                (0.31, 0.27, 0.15),
                (0.49, 0.29, 0.155),
                (0.64, 0.29, 0.15),
                (0.705, 0.20, 0.13),
            ],
            cloth,
            "TOP_hide_wrap",
            "torso",
        ),
        (
            "fur_cloak",
            [
                (0.28, 0.30, 0.17),
                (0.45, 0.33, 0.18),
                (0.64, 0.34, 0.18),
                (0.71, 0.22, 0.14),
            ],
            fur,
            "TOP_fur_cloak",
            "torso",
        ),
        (
            "hair_short",
            [(0.91, 0.175, 0.135), (0.98, 0.17, 0.13), (1.025, 0.10, 0.09)],
            hair,
            "HAIR_short",
            "head",
        ),
        (
            "hair_long_cap",
            [(0.91, 0.175, 0.135), (0.98, 0.17, 0.13), (1.025, 0.10, 0.09)],
            hair,
            "HAIR_long_tied",
            "head",
        ),
    ]:
        obj = ring_mesh(name, rings, mat)
        assign_layer(obj, layer)
        obj["anatomical_id"] = 10
        bind(obj, rig, bone)
    for i in range(5):
        obj = ellipsoid(
            f"fur_tuft_{i}",
            ((i - 2) * 0.12, -0.14, 0.57 - abs(i - 2) * 0.025),
            (0.075, 0.035, 0.13),
        )
        obj.data.materials.append(cloth)
        assign_layer(obj, "TOP_fur_cloak")
        bind(obj, rig, "torso")
    tail = ellipsoid("hair_tied_tail", (0, 0.17, 0.83), (0.07, 0.08, 0.16))
    tail.data.materials.append(hair)
    assign_layer(tail, "HAIR_long_tied")
    bind(tail, rig, "head")


def props(rig: bpy.types.Object) -> None:
    """Create the hand-held axe and shoulder log bundle in the idle world pose."""
    wood = material("wood_item", 5, (0.30, 0.20, 0.12))
    stone = material("stone_item", 5, (0.38, 0.38, 0.34))
    grip = bpy.data.objects.new("axe_grip", None)
    bpy.context.scene.collection.objects.link(grip)
    grip.location = (-0.33, -0.02, 0.39)
    assign_layer(grip, "ITEM_stone_axe")
    bpy.ops.mesh.primitive_cylinder_add(
        vertices=8, radius=0.018, depth=0.39, location=(-0.33, -0.06, 0.43)
    )
    handle = bpy.context.object
    handle.name = "axe_handle"
    handle.data.materials.append(wood)
    assign_layer(handle, "ITEM_stone_axe")
    blade = ellipsoid("axe_stone", (-0.29, -0.06, 0.60), (0.10, 0.035, 0.065))
    blade.data.materials.append(stone)
    assign_layer(blade, "ITEM_stone_axe")
    for obj in (handle, blade, grip):
        matrix = obj.matrix_world.copy()
        obj.parent = rig
        obj.parent_type = "BONE"
        obj.parent_bone = "arm-right"
        bpy.context.view_layer.update()
        obj.matrix_world = matrix
    for i in range(3):
        bpy.ops.mesh.primitive_cylinder_add(
            vertices=9,
            radius=0.045,
            depth=0.49,
            location=((i - 1) * 0.095, 0.12, 0.72 + (i % 2) * 0.065),
            rotation=(0, math.pi / 2, 0),
        )
        obj = bpy.context.object
        obj.name = f"log_{i}"
        obj.data.materials.append(wood)
        assign_layer(obj, "CARRY_log_bundle")
        bind(obj, rig, "torso")
