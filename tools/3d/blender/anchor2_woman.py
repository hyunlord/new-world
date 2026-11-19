"""Bake the existing Tripo woman with a planted walk and s2 hide atlas.

Run Blender 5+ from this checkout:
blender -b --factory-startup -t 2 --python-exit-code 1 -P tools/3d/blender/anchor2_woman.py
"""

import json
import math
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from typing import Final

import bpy
from mathutils import Quaternion, Vector
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import chars_b as B
import chars_c as V
import common as C

SOURCE: Final = "872dd491717c0c41d157d0987b9eedc8bcad3e8c"
STRIDE: Final = 1.0
STANCE: Final = 0.56


def _offset(bone: bpy.types.PoseBone, rotation: Quaternion) -> Quaternion:
    rest = bone.bone.matrix_local.to_quaternion()
    return rest.inverted() @ rotation @ rest


def _foot(phase: float) -> tuple[float, float, float]:
    """Return ankle forward offset, lift and pitch for a normalized cycle."""
    reach = STRIDE * STANCE / 2
    if phase < STANCE:
        heel = max(0, 1 - phase / 0.09) ** 2
        toe = max(0, (phase - 0.46) / (STANCE - 0.46)) ** 2
        return reach - STRIDE * phase, 0.026 * heel + 0.042 * toe, 0.24 * toe - 0.22 * heel
    swing = (phase - STANCE) / (1 - STANCE)
    smooth = swing * swing * (3 - 2 * swing)
    tangent = -STRIDE * (1 - STANCE) * swing * (1 - swing) * (1 - 2 * swing)
    lift = 0.105 * math.sin(math.pi * swing) + 0.042 * (1 - smooth) + 0.026 * smooth
    return -reach + 2 * reach * smooth + tangent, lift, 0.24 * (1 - smooth) - 0.22 * smooth


def _leg(
    upper: bpy.types.PoseBone,
    lower: bpy.types.PoseBone,
    foot: bpy.types.PoseBone,
    phase: float,
) -> float:
    """Solve the ankle target with the knee bending in the rig's forward plane."""
    forward, lift, pitch = _foot(phase)
    start = upper.head.copy()
    rest_ankle = foot.bone.head_local
    ankle = rest_ankle + Vector((0, -forward, lift))
    axis = ankle - start
    length = axis.length
    axis /= length
    thigh, shin = upper.bone.length, lower.bone.length
    assert abs(thigh - shin) < length < thigh + shin, (phase, length, thigh, shin)
    along = (thigh * thigh - shin * shin + length * length) / (2 * length)
    bend_direction = Vector((0, -1, 0)) + axis * axis.y
    bend_direction.normalize()
    knee = start + along * axis + math.sqrt(thigh * thigh - along * along) * bend_direction
    _orient_at(upper, knee - start)
    _orient_at(lower, ankle - knee)
    pose = (Quaternion(C.X, pitch) @ foot.bone.matrix_local.to_quaternion()).to_matrix().to_4x4()
    pose.translation = foot.head
    foot.matrix = pose
    bpy.context.view_layer.update()
    return (foot.head - ankle).length


def _orient_at(bone: bpy.types.PoseBone, direction: Vector) -> None:
    """Align the local Y axis with a segment while preserving the joint position."""
    rest = bone.bone.matrix_local.to_quaternion()
    orientation = (rest @ C.Y).rotation_difference(direction) @ rest
    pose = orientation.to_matrix().to_4x4()
    pose.translation = bone.head
    bone.matrix = pose
    bpy.context.view_layer.update()


def _walk(arm: bpy.types.Object) -> None:
    """Replace only the imported walk action, retaining topology, skin and other clips."""
    assert len(arm.data.bones) == 17
    binding = B._clip_bindings([arm])["walk"]
    B._play(binding)
    _, action, slot = binding[0]
    first, last = map(round, action.frame_range)
    assert last - first == C.FPS
    for layer in action.layers:
        for strip in layer.strips:
            for bag in strip.channelbags:
                for curve in list(bag.fcurves):
                    bag.fcurves.remove(curve)
    arm.animation_data.action_slot = slot
    scene = bpy.context.scene
    bones = arm.pose.bones
    previous = {}
    max_ankle_error = 0.0
    for frame in range(first, last + 1):
        scene.frame_set(frame)
        phase = ((frame - first) % C.FPS) / C.FPS
        swing = math.cos(math.tau * phase)
        rise = 0.5 - 0.5 * math.cos(2 * math.tau * phase)
        for bone in bones:
            bone.rotation_mode = "QUATERNION"
            bone.rotation_quaternion = Quaternion()
            bone.location = Vector()
        hips = bones["hips"]
        shift = Vector((0.024 * math.sin(math.tau * phase), 0.0, -0.075 + 0.027 * rise))
        hips.location = hips.bone.matrix_local.to_quaternion().inverted() @ shift
        hips.rotation_quaternion = _offset(hips, Quaternion(C.Z, -0.10 * swing))
        chest = bones["chest"]
        chest.rotation_quaternion = _offset(chest, Quaternion(C.X, 0.045) @ Quaternion(C.Z, 0.17 * swing))
        for side, sign in (("L", 1), ("R", -1)):
            shoulder = bones[f"upper_arm_{side}"]
            elbow = bones[f"lower_arm_{side}"]
            lowered = Quaternion(C.Y, sign * 1.41)
            shoulder.rotation_quaternion = _offset(shoulder, Quaternion(C.X, sign * 0.25 * swing) @ lowered)
            elbow.rotation_quaternion = _offset(elbow, Quaternion(lowered.inverted() @ C.X, -0.20 + sign * 0.08 * swing))
        bpy.context.view_layer.update()
        for side, offset in (("L", 0.0), ("R", 0.5)):
            error = _leg(bones[f"upper_leg_{side}"], bones[f"lower_leg_{side}"],
                         bones[f"foot_{side}"], (phase + offset) % 1.0)
            max_ankle_error = max(max_ankle_error, error)
        for bone in bones:
            rotation = bone.rotation_quaternion.copy()
            if bone.name in previous and rotation.dot(previous[bone.name]) < 0:
                rotation.negate()
            previous[bone.name] = rotation.copy()
            bone.rotation_quaternion = rotation
            bone.keyframe_insert("rotation_quaternion", frame=frame, group=bone.name)
            bone.keyframe_insert("location", frame=frame, group=bone.name)
    assert max_ankle_error < 0.002, f"ankle target error: {max_ankle_error:.4f} m"
    print(f"[anchor2] walk: stride={STRIDE:g} m, max ankle target error={max_ankle_error:.5f} m")


def _atlas(source: Path, target: Path) -> None:
    """Suppress fine photographic contrast within each painted material."""
    rgba = V._atlas_pixels(str(source)).astype(np.float32)
    r, g, b = np.moveaxis(rgba[:, :, :3], -1, 0)
    luma = 0.30 * r + 0.59 * g + 0.11 * b
    cloth = (r - g < 0.12) & (g - b > 0.045) & (luma > 0.23)
    skin = (r - g >= 0.12) & (luma > 0.24)
    trim = (luma >= 0.025) & ~(cloth | skin)
    h, w = luma.shape
    radius = max(1, w // 512)
    for mask, color, pivot, contrast in (
        (cloth, (0.39, 0.31, 0.23), 0.49, 0.30),
        (skin, (0.71, 0.52, 0.41), 0.46, 0.55),
        (trim, (0.23, 0.18, 0.14), 0.23, 0.62),
    ):
        assert mask.any(), f"atlas region missing: {target}"
        offsets = range(2 * radius + 1)
        masked = np.pad(luma * mask, radius, mode="edge")
        valid = np.pad(mask.astype(np.float32), radius, mode="edge")
        blur_sum = sum(masked[y:y + h, x:x + w] for y in offsets for x in offsets)
        blur_count = sum(valid[y:y + h, x:x + w] for y in offsets for x in offsets)
        local_luma = 0.2 * luma[mask] + 0.8 * blur_sum[mask] / blur_count[mask]
        rgba[mask, :3] = np.clip(np.asarray(color) + contrast * (local_luma[:, None] - pivot), 0, 1)
    image = bpy.data.images.new(target.stem, width=w, height=h, alpha=False)
    image.pixels.foreach_set(rgba.ravel())
    image.file_format = "PNG"
    image.filepath_raw = str(target)
    image.save()
    bpy.data.images.remove(image)


def main() -> None:
    """Bake two interchangeable tiers from the locally pinned source revision."""
    with TemporaryDirectory(prefix="anchor2-woman-") as temporary:
        scratch = Path(temporary)
        tiers = []
        for level, identifier in ((0, "s2_woman"), (2, "s2_woman_lod")):
            stem = f"tripo_woman_lod{level}"
            for suffix in (".glb", "_albedo.png"):
                source = f"assets/3d/models/{stem}{suffix}"
                data = subprocess.check_output(["git", "show", f"{SOURCE}:{source}"], cwd=C.REPO)
                (scratch / f"{stem}{suffix}").write_bytes(data)
            atlas = scratch / f"{identifier}.png"
            _atlas(scratch / f"{stem}_albedo.png", atlas)
            V.build_one(identifier, str(scratch / f"{stem}.glb"), "woman", str(atlas), B.HUMAN,
                        f"models/{stem}.glb", {}, add_tool=False, prepare_armature=_walk)
            metadata = Path(C.VAT_DIR) / f"{identifier}_vat.json"
            record = json.loads(metadata.read_text())
            record["source_commit"] = SOURCE
            record["source_provenance"] = "Owner-supplied Tripo GLB, existing 17-bone rig, s2 walk and atlas"
            record["walk_stride_m"] = STRIDE
            metadata.write_text(json.dumps(record, indent=1) + "\n")
            tiers.append(record)
        near, far = tiers
        assert near["clips"] == far["clips"]
        assert near["clips"]["walk"] == {"start": 15, "count": 15, "loop": True, "duration": 1.0}
        assert far["width"] < near["width"] / 4


if __name__ == "__main__":
    main()
