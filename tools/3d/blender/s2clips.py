"""Bake deterministic S2 VAT extensions and sample hand sockets on the same rows.

Usage under the shared GPU lock: blender -b -P s2clips.py -- S2KIT_DIR OUT review
"""

import json
from pathlib import Path
import sys
from typing import TYPE_CHECKING

import bmesh
import bpy
from mathutils import Matrix, Vector
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import anchor2_woman as anchor
import chars_b as B
import common as C
import s2clips_pose as P
import s2kit as kit

if TYPE_CHECKING:
    from s2clips_capture import ReviewAction

FPS = 15
CHANGE_BASIS = Matrix(((1, 0, 0), (0, 0, 1), (0, -1, 0)))
BODIES = ("p1", "p2", "p3", "p4")
# Centres of the four scanned palms in the source A-pose, in body heights.
PALMS = {
    "p1": ((-.239, -.008, .463), (.237, -.008, .460)),
    "p2": ((-.201, -.011, .459), (.201, -.011, .462)),
    "p3": ((-.217, -.022, .451), (.219, -.025, .452)),
    "p4": ((-.275, -.114, .585), (.286, -.078, .467)),
}


def prop_mesh(name: str) -> bpy.types.Object:
    """Handheld meshes with the grip at the origin and the tool along local +Z."""
    model = C.Model("s2_grip_" + name)
    match name:
        case "axe":
            model.rod("wood_dark", (0, 0, -.132), (.007, 0, .53), .025, segs=8, r2=.034)
            wedge = [model.bm.verts.new(coords) for coords in (
                (-.062, -.033, .46), (-.062, .033, .46),
                (.042, -.030, .434), (.042, .030, .434),
                (.142, -.005, .50), (.142, .005, .50),
                (.115, 0., .55), (.052, -.027, .543), (.052, .027, .543),
            )]
            for face in ((0, 1, 3, 2), (0, 2, 4, 7), (1, 8, 5, 3), (2, 3, 5, 4),
                         (4, 5, 6), (4, 6, 7), (5, 8, 6), (6, 8, 7), (7, 8, 1, 0)):
                model.bm.faces.new([wedge[i] for i in face])
            model._finish_part(wedge, "stone", None)
            for at in (.438, .456, .474, .492, .510):
                model.rod("hide", (.002, 0, at), (.002, 0, at + .012), .044, segs=9)
        case "ladle":
            model.rod("wood", (0, 0, -.07), (0, 0, .46), .013, segs=7, r2=.019)
            model.ball("wood_dark", .085, (0, 0, .46), (.76, .43, .64), subdiv=2)
            model.ball("wood", .072, (0, .025, .46), (.78, .12, .64), subdiv=2)
        case "basket":
            # Right and left palms grasp the upper rim 34 cm apart.
            model.lathe("wood", ((.103, -.24), (.135, -.219), (.169, -.04), (.173, 0.)),
                        loc=(.17, 0., 0.), scale=(1., .83, 1.), segs=20)
            for z in (-.183, -.14, -.096, -.052, 0.):
                radius = .17 + .24 * z
                model.lathe("wood_dark", ((radius, z - .009), (radius + .009, z - .006),
                                         (radius + .009, z + .006), (radius, z + .009)),
                            loc=(.17, 0., 0.), scale=(1., .83, 1.), segs=20)
        case "logs":
            for y, z, span, radius in ((-.022, .028, .282, .044), (.058, .048, .257, .042),
                                        (.010, .104, .269, .043)):
                model.rod("trunk", (.24 - span, y, z), (.24 + span, y, z), radius, segs=8, cap_mat="wood")
                model.rod("wood", (.239 - span, y, z), (.24 - span, y, z), radius * .58, segs=8)
        case _:
            raise ValueError(name)
    return model.build_object()


def remove_staff(mesh: bpy.types.Object) -> None:
    """Trim the scanned shaft above and below the hand, closing the two small cuts."""
    data = bmesh.new()
    data.from_mesh(mesh.data)
    bottom = min(v.co.z for v in data.verts)
    height = max(v.co.z for v in data.verts) - bottom
    staff = [v for v in data.verts if v.co.x / height < -.241
             and not .552 <= (v.co.z - bottom) / height <= .613]
    assert staff, "p4 source has no staff"
    bmesh.ops.delete(data, geom=staff, context="VERTS")
    bmesh.ops.holes_fill(data, edges=[e for e in data.edges if e.is_boundary and
                                     all(v.co.x / height < -.236 for v in e.verts)], sides=0)
    data.to_mesh(mesh.data)
    data.free()


def action_weights(mesh: bpy.types.Object, body: str) -> list[dict[str, float]]:
    """Find the separated forearms below the armpits before assigning joint bands."""
    vertices = mesh.data.vertices
    bottom = min(v.co.z for v in vertices)
    height = max(v.co.z for v in vertices) - bottom
    z = [(v.co.z - bottom) / height for v in vertices]
    neighbors = [[] for _ in vertices]
    for edge in mesh.data.edges:
        a, b = edge.vertices
        neighbors[a].append(b)
        neighbors[b].append(a)
    arms: set[int] = set()
    for sign in (-1, 1):
        wrist = .58 if body == "p4" and sign == -1 else .47
        root_x = .164 if body == "p4" and sign == -1 else .128
        band = [v.index for v in vertices if abs(z[v.index] - wrist) <= .040]
        pending = [max(band, key=lambda i: sign * vertices[i].co.x)]
        while pending:
            i = pending.pop()
            if i in arms or not (.402 < z[i] < .715 and sign * vertices[i].co.x > (root_x + .22 * (.715 - z[i])) * height):
                continue
            arms.add(i)
            pending.extend(neighbors[i])
    print(f"S2_CLIPS_ARM_SCAN {body} {len(arms)}/{len(vertices)}", flush=True)

    def per_vertex(index: int, x: float, z: float) -> dict[str, float]:
        side = "R" if x < 0 else "L"
        base = kit.weights(body, x, z)
        arm_keys = (f"upper_arm_{side}", f"lower_arm_{side}", f"hand_{side}")
        if index not in arms:
            # Adjacent cloth belongs to the trunk below the armpit.
            if z < .715:
                kept = kit.blend(z, .670, .715)
                spill = 0.
                for key in arm_keys:
                    spill += (1 - kept) * base[key]
                    base[key] *= kept
                chest = kit.blend(z, .53, .69)
                base["chest"] += spill * chest
                base["hips"] += spill * (1 - chest)
            return base
        upper = kit.blend(z, .583, .666)
        elder_hand = body == "p4" and side == "R"
        hand = 1 - kit.blend(z, .535 if elder_hand else .462, .613 if elder_hand else .533)
        target = dict(zip(arm_keys, (upper, (1 - upper) * (1 - hand), (1 - upper) * hand)))
        mix = 1 - kit.blend(z, .672, .715)
        return {key: val * (1 - mix) + target.get(key, 0.) * mix for key, val in base.items()}

    raw = [per_vertex(v.index, v.co.x / height, z[v.index]) for v in vertices]
    # Smooth along connected sleeve loops. Nearby surfaces across the scan's arm gap stay separate.
    keys = tuple(dict.fromkeys(key for row in raw for key in row))
    weights = np.array([[row.get(key, 0.) for key in keys] for row in raw])
    driven = [i for i, key in enumerate(keys) if "_arm_" in key or key.startswith("hand_")]
    edges = np.asarray([tuple(edge.vertices) for edge in mesh.data.edges])
    starts = np.concatenate((edges[:, 0], edges[:, 1]))
    ends = np.concatenate((edges[:, 1], edges[:, 0]))
    points = np.array([tuple(v.co) for v in vertices])
    conductance = 1. / np.maximum(.012 * height, np.linalg.norm(points[starts] - points[ends], axis=1))
    totals = np.maximum(1e-6, np.bincount(starts, conductance, minlength=len(vertices)))
    z_pos = np.asarray(z)
    for _ in range(12):
        average = np.zeros_like(weights)
        np.add.at(average, starts, weights[ends] * conductance[:, None])
        average /= totals[:, None]
        active = (z_pos > .41) & (totals > 1e-5) & ((weights[:, driven] + average[:, driven]).sum(axis=1) > .001)
        weights[active] = .58 * weights[active] + .42 * average[active]
    weights /= weights.sum(axis=1, keepdims=True)
    return [dict(zip(keys, row)) for row in weights]


def source_rig(
    root: Path, body: str, budget: int, *, stabilize_grips: bool = False,
) -> tuple[bpy.types.Object, bpy.types.Object, tuple[int, int]]:
    C.reset_scene()
    mesh = kit.import_mesh(root / "prepared" / f"{body}-{budget}.glb")
    if body == "p4":
        remove_staff(mesh)
    mesh.name = body
    bpy.ops.import_scene.gltf(filepath=str(root / "template.glb"))
    arm = next(ob for ob in bpy.data.objects if ob.type == "ARMATURE")
    for other in list(bpy.data.objects):
        if other not in (mesh, arm):
            bpy.data.objects.remove(other, do_unlink=True)
    weights = action_weights(mesh, body)
    bottom = min(v.co.z for v in mesh.data.vertices)
    height = max(v.co.z for v in mesh.data.vertices) - bottom
    grip_vertices = []
    for side, nominal in zip(("R", "L"), PALMS[body]):
        target = Vector(nominal) * height + Vector((0, 0, bottom))
        hand = f"hand_{side}"
        candidates = [v for v in mesh.data.vertices if weights[v.index].get(hand, 0.) > .30]
        nearest = min(candidates, key=lambda v: (v.co - target).length_squared)
        assert (nearest.co - target).length <= .075, (body, side, "grip misses source palm")
        grip_vertices.append(nearest.index)
        if stabilize_grips:
            # The grip must move with the palm even where the scan blends into the cuff.
            for vertex in mesh.data.vertices:
                row = weights[vertex.index]
                distance = (vertex.co - nearest.co).length / (.055 * height)
                if row.get(hand, 0.) > .15 and distance < 1.:
                    keep = distance * distance * (3. - 2. * distance)
                    weights[vertex.index] = {name: weight * keep for name, weight in row.items()}
                    weights[vertex.index][hand] += 1. - keep
    kit.bind(mesh, arm, body, lambda index, _x, _z: weights[index])
    return arm, mesh, (grip_vertices[0], grip_vertices[1])


def socket(hand: bpy.types.PoseBone, contact: np.ndarray, rotation: Matrix | None = None) -> Matrix:
    """Anchor the prop to the same deformed skin point sampled into the VAT."""
    matrix = hand.matrix.copy() if rotation is None else rotation.to_4x4()
    matrix.translation = Vector(contact)
    return matrix


def gltf_socket(matrix: Matrix) -> list[float]:
    godot = CHANGE_BASIS @ matrix.to_3x3() @ CHANGE_BASIS.transposed()
    rotation = godot.to_quaternion()
    return [*C.to_godot(matrix.translation), *(round(x, 6) for x in (*rotation[1:], rotation.w))]


def bake(root: Path, out: Path, body: str, tier: str, budget: int,
         names: tuple[str, ...], capture: bool, *, review: "dict[str, ReviewAction] | None" = None) -> None:
    arm, mesh, (right_grip, left_grip) = source_rig(root, body, budget)
    assert len(arm.pose.bones) == 17 and C.tri_count(mesh) <= budget
    anchor._walk(arm)
    bindings = B._clip_bindings([arm])
    rows, clips, samples, contacts = [], {}, {}, {}
    id = f"s2_{body}{tier}"
    original = json.loads((Path(C.VAT_DIR) / f"{id}_vat.json").read_text())
    for name in (*B.HUMAN, *names):
        authored = name in P.CLIPS and (name in names or name == "carry")
        if name in bindings:
            B._play(bindings["walk" if name == "carry" else name])
            source_start = bindings["walk" if name == "carry" else name][0][1].frame_range[0]
        else:
            arm.animation_data.action = None
            source_start = 0.
        if authored:
            duration = P.CLIPS[name].duration * (1.25 if body == "p4" else 1.0)
            loop = P.CLIPS[name].loop
            count = round(duration * FPS) + (not loop)
        else:
            count = original["clips"][name]["count"]
            loop = True
        clips[name] = {"start": len(rows), "count": count, "loop": loop,
                       "duration": round((count - (not loop)) / FPS, 4)}
        samples[name] = []
        contacts[name] = []
        for frame in range(count):
            source_frame = source_start + frame * C.FPS / FPS
            if authored:
                phase = frame / (count if loop else count - 1)
                source_frame = source_start + phase * C.FPS
            bpy.context.scene.frame_set(int(source_frame), subframe=source_frame % 1)
            direction = P.pose(arm, body, name, phase) if authored else None
            pos = posed_positions(mesh)
            rotation = None
            if direction is not None:
                rotation = (P.tool_basis(direction) if name == "chop"
                            else direction.to_track_quat("Z", "Y").to_matrix())
                goal = P.tool_goal(name, phase, direction)
                goals = []
                if goal is not None:
                    goals.append(("R", right_grip, goal))
                if name == "carry":
                    hip = arm.pose.bones["hips"].head
                    height = hip.z + .253 - (.04 if body == "p4" else 0.)
                    goals.extend((("R", right_grip, Vector((-.23, -.34, height))),
                                  ("L", left_grip, Vector((.23, -.34, height)))))
                for side, index, target in goals:
                    for _ in range(10):
                        correction = target - Vector(pos[index])
                        if correction.length < .001:
                            break
                        P.adjust_grip(arm, side, correction)
                        pos = posed_positions(mesh)
                    contacts[name].append(round((target - Vector(pos[index])).length, 6))
            right = socket(arm.pose.bones["hand_R"], pos[right_grip], rotation)
            left = socket(arm.pose.bones["hand_L"], pos[left_grip])
            if name == "carry":
                lateral = (left.translation - right.translation).normalized()
                depth = Vector((0, 0, 1)).cross(lateral).normalized()
                frame = Matrix((lateral, depth, lateral.cross(depth))).transposed().to_4x4()
                frame.translation = right.translation
                right = frame
            samples[name].append((right, left))
            rows.append(np.stack((pos[:, 0], pos[:, 2], -pos[:, 1], np.ones(len(pos))), 1))
    pos = np.stack(rows)
    assert np.isfinite(pos).all()
    split, n_split = B._static_mesh(id, [mesh], blender_positions(pos[0]))
    C.export_glb(str(out / f"{id}_static.glb"), [split], vertex_colors=True)
    bpy.data.objects.remove(split, do_unlink=True)
    vat = pos.astype("<f2")
    vat.tofile(out / f"{id}_vat.bin")
    original.update(width=pos.shape[1], vertex_count=pos.shape[1], split_vertex_count=n_split,
                    frames=len(pos), clips=clips, bounds_min=pos[:, :, :3].min((0, 1)).round(4).tolist(),
                    bounds_max=pos[:, :, :3].max((0, 1)).round(4).tolist(),
                    sockets={name: [[gltf_socket(right), gltf_socket(left)] for right, left in frames]
                             for name, frames in samples.items()})
    (out / f"{id}-contacts.json").write_text(json.dumps(contacts) + "\n")
    (out / f"{id}_vat.json").write_text(json.dumps(original, indent=1) + "\n")
    print(f"S2_CLIPS_BAKED {id} {pos.shape} {C.tri_count(mesh)} triangles", flush=True)
    if capture:
        from s2clips_capture import capture_body
        captured = tuple(review) if review is not None else (("chop", "carry", "cook") if len(names) == 2
                                                            else (*B.HUMAN, *names))
        capture_body(root, out, mesh, body, captured, clips, samples, vat, review=review)


def posed_positions(mesh: bpy.types.Object) -> np.ndarray:
    """Read the current evaluated pose without resetting the authored bone transforms."""
    evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
    geometry = evaluated.to_mesh()
    positions = np.empty(len(geometry.vertices) * 3, dtype=np.float64)
    geometry.vertices.foreach_get("co", positions)
    evaluated.to_mesh_clear()
    matrix = np.array(evaluated.matrix_world)
    return positions.reshape(-1, 3) @ matrix[:3, :3].T + matrix[:3, 3]


def blender_positions(vat: np.ndarray) -> np.ndarray:
    return vat[:, (0, 2, 1)].astype(np.float32) * (1, -1, 1)


def main() -> None:
    root, out, phase = C.script_args()
    target = Path(out)
    target.mkdir(parents=True, exist_ok=True)
    names = ("chop", "cook") if phase == "review" else tuple(name for name in P.CLIPS if name != "carry")
    for body in BODIES:
        bake(Path(root), target, body, "", 8000, names, True)
        if phase != "review":
            bake(Path(root), target, body, "_mid", 2500, names, False)
    for name in ("axe", "ladle", "logs"):
        C.reset_scene()
        prop = prop_mesh(name)
        C.export_glb(str(target / f"grip_{name}.glb"), [prop])


if __name__ == "__main__":
    main()
