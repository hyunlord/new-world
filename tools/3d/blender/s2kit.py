"""Finish the S2 kit from UV-preserving, triangle-budgeted Tripo GLBs.

Pass the external s2kit evidence directory after --. Its prepared/ contains
the decimated asset tiers. Run through the shared GPU lock, as for ANCHOR2.
"""

import json
from pathlib import Path
import sys

import bpy
from mathutils import Matrix, Quaternion, Vector
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import anchor2_woman as anchor
import chars_b as B
import chars_c as V
import common as C

KIT = Path(C.REPO) / "assets/3d/anchor2/kit"
HEIGHT = 1.6004


def import_mesh(path: Path) -> bpy.types.Object:
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(path), merge_vertices=True)
    mesh, = [ob for ob in set(bpy.data.objects) - before if ob.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = mesh
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    return mesh


def base_color(mesh: bpy.types.Object) -> bpy.types.Image:
    nodes = mesh.data.materials[0].node_tree.nodes
    shader = next(node for node in nodes if node.type == "BSDF_PRINCIPLED")
    return shader.inputs["Base Color"].links[0].from_node.image


def paint(source: bpy.types.Image, path: Path, size: int, figure: bool) -> bpy.types.Image:
    """Area-filter the UV bake and compress contrast into the warm ANCHOR2 palette."""
    width, height = source.size
    assert width == height == 4096 and width % size == 0
    rgba = np.empty(width * height * 4, dtype=np.float32)
    source.pixels.foreach_get(rgba)
    block = width // size
    rgb = rgba.reshape(size, block, size, block, 4).mean(axis=(1, 3))[:, :, :3]
    luma = rgb @ np.array([0.30, 0.59, 0.11], dtype=np.float32)
    luma_blur = np.pad(luma, 1, mode="edge")
    smooth = sum(luma_blur[y:y + size, x:x + size] for y in range(3) for x in range(3)) / 9.0
    detail = 0.55 * luma + 0.45 * smooth
    if figure:
        r, g, b = np.moveaxis(rgb, -1, 0)
        skin = (r - g >= 0.12) & (luma > 0.24)
        cloth = (r - g < 0.12) & (g - b > 0.045) & (luma > 0.23)
        for mask, color, pivot, contrast in (
            (cloth, (.39, .31, .23), .49, .45),
            (skin, (.71, .52, .41), .46, .60),
            (~(cloth | skin), (.23, .18, .14), .23, .70),
        ):
            rgb[mask] = .18 * rgb[mask] + .82 * (color + contrast * (detail[mask, None] - pivot))
    else:
        rgb = 0.75 * rgb + 0.25 * detail[:, :, None]
        rgb = np.array([.46, .39, .30]) + .80 * (rgb - np.array([.46, .39, .30]))
    pixels = np.ones((size, size, 4), dtype=np.float32)
    pixels[:, :, :3] = np.clip(rgb, 0, 1)
    image = bpy.data.images.new(path.stem, width=size, height=size, alpha=False)
    image.pixels.foreach_set(pixels.ravel())
    image.filepath_raw = str(path)
    image.file_format = "PNG"
    image.save()
    image.pack()
    return image


def blend(value: float, lo: float, hi: float) -> float:
    t = min(1.0, max(0.0, (value - lo) / (hi - lo)))
    return t * t * (3.0 - 2.0 * t)


def weights(key: str, x: float, z: float) -> dict[str, float]:
    """Blend over joint bands on the source A pose, whose skirt covers both legs."""
    side = "L" if x >= 0 else "R"
    x = abs(x)
    shoulder = .76 if key == "p3" else .79
    shoulder_x = .112 if key in ("p2", "p3") else .135
    arm = blend(x, shoulder_x + max(0.0, shoulder - z) * .22 - .018,
                   shoulder_x + max(0.0, shoulder - z) * .22 + .026)
    arm *= blend(z, .39, .43) * (1 - blend(z, shoulder + .025, shoulder + .072))
    elbow = .64 if key == "p4" and side == "R" else .62
    wrist = .55 if key == "p4" and side == "R" else .50
    upper = blend(z, elbow - .04, elbow + .04)
    hand = 1 - blend(z, wrist - .035, wrist + .025)
    leg_top = .40 if key == "p3" else .43
    leg = (1 - blend(z, leg_top - .045, leg_top + .070)) * (1 - arm)
    # The shared rest rig places the knee at .26 and the ankle at .04 of height.

    thigh = blend(z, .235, .285)
    foot = 1 - blend(z, .025, .062)
    trunk = 1 - arm - leg
    head = blend(z, .82 if key == "p3" else .84, .87 if key == "p3" else .90)
    chest = blend(z, .63, .75)
    spine = blend(z, .49, .63)
    influences = {
        f"upper_arm_{side}": arm * upper,
        f"lower_arm_{side}": arm * (1 - upper) * (1 - hand),
        f"hand_{side}": arm * (1 - upper) * hand,
        f"upper_leg_{side}": leg * thigh,
        f"lower_leg_{side}": leg * (1 - thigh) * (1 - foot),
        f"foot_{side}": leg * (1 - thigh) * foot,
        "hips": trunk * (1 - head) * (1 - spine),
        "spine": trunk * (1 - head) * spine * (1 - chest),
        "chest": trunk * (1 - head) * spine * chest,
        "head": trunk * head,
    }
    if key == "p4" and side == "R":
        # The fused shaft lies beyond the forearm. Keep the whole shaft rigid and blend at the grip.
        grip = blend(x, .205, .235)
        influences = {name: weight * (1 - grip) for name, weight in influences.items()}
        influences["hand_R"] += grip
    return influences


def bind(mesh: bpy.types.Object, arm: bpy.types.Object, key: str, weights_for_vertex=None) -> None:
    """Inverse-skin the A pose into the unchanged ANCHOR2 skeleton rest pose."""
    animation = arm.animation_data
    animation.action = None
    for track in animation.nla_tracks:
        track.mute = True
    for bone in arm.pose.bones:
        bone.matrix_basis = Matrix.Identity(4)
        bone.rotation_mode = "QUATERNION"
    for side, sign in (("L", 1), ("R", -1)):
        shoulder = arm.pose.bones[f"upper_arm_{side}"]
        angle = 0.97 if key == "p4" and side == "R" else 1.14
        shoulder.rotation_quaternion = anchor._offset(shoulder, Quaternion(C.Y, sign * angle))
    bpy.context.view_layer.update()
    matrices = {bone.name: arm.matrix_world @ bone.matrix @ bone.bone.matrix_local.inverted()
                for bone in arm.pose.bones}
    groups = {name: mesh.vertex_groups.new(name=name) for name in matrices}
    vertices = mesh.data.vertices
    bottom = min(v.co.z for v in vertices)
    height = max(v.co.z for v in vertices) - bottom
    for vertex in vertices:
        point = vertex.co.copy()
        xx, zz = point.x / height, (point.z - bottom) / height
        influences = weights(key, xx, zz) if weights_for_vertex is None else weights_for_vertex(vertex.index, xx, zz)
        assert abs(sum(influences.values()) - 1) < 1e-6
        point = Vector((xx * HEIGHT, point.y * HEIGHT / height, zz * HEIGHT))
        matrix = Matrix(((0, 0, 0, 0),) * 4)
        for name, weight in influences.items():
            if weight > 0:
                matrix += matrices[name] * weight
                groups[name].add([vertex.index], weight, "REPLACE")
        vertex.co = matrix.inverted() @ point
    skin = mesh.modifiers.new("ANCHOR2 17-bone skeleton", "ARMATURE")
    skin.object = arm
    mesh.parent = arm
    for bone in arm.pose.bones:
        bone.matrix_basis = Matrix.Identity(4)


def people(root: Path, key: str) -> None:
    atlas = root / f"s2_{key}_atlas.png"
    for budget, tier in ((8000, ""), (2500, "_mid"), (800, "_lod")):
        C.reset_scene()
        mesh = import_mesh(root / "prepared" / f"{key}-{budget}.glb")
        if not tier:
            paint(base_color(mesh), atlas, 1024, True)
        mesh.name = key
        bpy.ops.import_scene.gltf(filepath=str(root / "template.glb"))
        arm = next(ob for ob in bpy.data.objects if ob.type == "ARMATURE")
        bind(mesh, arm, key)
        for other in list(bpy.data.objects):
            if other not in (mesh, arm):
                bpy.data.objects.remove(other, do_unlink=True)
        name = "s2_" + key + tier
        scratch = root / f"{key}{tier}-rig.glb"
        C.export_glb(str(scratch), [arm, mesh], animations=True)
        # Load the same rig and clips as ANCHOR2 and bake with its existing VAT exporter.
        entries = {}
        name_atlas = Path(C.VAT_DIR) / f"{name}_atlas.png"
        V.build_one(name, str(scratch), key, str(atlas), B.HUMAN, f"Tripo {key}", entries,
                    add_tool=False, prepare_armature=anchor._walk)
        meta = Path(C.VAT_DIR) / f"{name}_vat.json"
        record = json.loads(meta.read_text())
        record.update(atlas=f"s2_{key}_atlas.png", height=HEIGHT, walk_stride_m=anchor.STRIDE,
                      source_commit=anchor.SOURCE, source_provenance=f"Owner Tripo {key}; ANCHOR2 rig and clips")
        meta.write_text(json.dumps(record, indent=1) + "\n")
        if tier:
            name_atlas.unlink()
        count = entries[name]["triangles"]
        assert 0 < count <= budget
        print(f"S2_BAKED {name} {count} 1024", flush=True)


def static(root: Path, key: str, top: int, far: int, size: int) -> None:
    C.reset_scene()
    mesh = import_mesh(root / "prepared" / f"{key}-{top}.glb")
    paint(base_color(mesh), KIT / f"{key}.png", size, False)
    material = C.plain_material(key)
    material.use_nodes = True
    shader = material.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Roughness"].default_value = .88
    for budget, label in ((top, ""), (far, "_lod")):
        if label:
            mesh = import_mesh(root / "prepared" / f"{key}-{budget}.glb")
        bounds = [Vector(v) for v in mesh.bound_box]
        center = sum(bounds, Vector()) / 8
        center.z = min(v.z for v in bounds)
        for vertex in mesh.data.vertices:
            vertex.co -= center
        mesh.data.materials.clear()
        mesh.data.materials.append(material)
        C.export_glb(str(KIT / f"{key}{label}.glb"), [mesh])
        count = C.tri_count(mesh)
        assert 0 < count <= budget
        print(f"S2_BAKED {key}{label} {count} {size}", flush=True)


def main() -> None:
    root = Path(sys.argv[sys.argv.index("--") + 1])
    keys = sys.argv[sys.argv.index("--") + 2:]
    KIT.mkdir(parents=True, exist_ok=True)
    for key in keys:
        if key.startswith("p"):
            people(root, key)
        elif key.startswith("h"):
            static(root, key, 6000, 1500, 1024)
        else:
            top = 2000 if key == "x5" else 4000
            static(root, key, top, int(top * .30), 512)


if __name__ == "__main__":
    main()
