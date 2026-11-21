"""Blender worker for the locked MOCAP1 command."""

from dataclasses import asdict
import json
import math
import os
from pathlib import Path
import sys

import bpy
from mathutils import Matrix
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import chars_b as base
import common as C
from mocap_bvh import Motion, read_bvh
from mocap_anatomy import evaluate_anatomy
from mocap_deform import MAX_BODY_STRETCH, DeformationReport, check_deformation
from mocap_kinematics import Check, Track, check_prop, write_track
from mocap_pose import Retarget, constrain, prop_hand, wrist_center
from mocap_skin import PLANTED_SOLE_M, relax_body_skin, relax_joint_skin, stabilize_garment_weights
from mocap_spec import Action, BaseLayer, Take, load_many, select
from mocap_support import evaluate_support
import s2clips as legacy
from s2clips_capture import BODY_SCALE, Clip, ReviewAction, capture_body, fixture
from s2kit import HEIGHT

FPS = 30
REVIEW_PHASES = (0., .20, .40, .60, .80, .97)
Source = tuple[Take, Motion]


def review_for(action: Action, frames: list[dict[str, float]], deformation: DeformationReport, mode: str) -> ReviewAction:
    phases = REVIEW_PHASES
    detail = None
    checks = []
    hero = .60
    if action.target.kind == "swing" and action.contacts:
        hit = action.contacts[0]
        hero = hit
        if action.retime and action.prop is not None and .16 < hit < .88:
            leading = (frame for frame in frames if 0. < frame["phase"] < hit)
            detail = max(leading, key=lambda frame: frame["tip_above_anchor_m"])["phase"]
            hero = detail
            phases = (0., detail, hit, hit + .13 * (1 - hit), hit + .53 * (1 - hit), .96)
            later = (i for i, frame in enumerate(frames) if hit < frame["phase"] <= hit + .22)
            checks = [("garment", deformation.max_body_edge_frame),
                      ("rebound", max(later, key=lambda i: frames[i]["tip_above_anchor_m"]))]
        elif .16 < hit < .88:
            phases = (0., .40 * hit, .68 * hit, hit, hit + .37 * (1 - hit), hit + .78 * (1 - hit))
    if action.target.kind == "reach" and action.contacts:
        hit = action.contacts[0]
        hero = hit
        if not action.loop and .15 < hit < .85:
            phases = (0., hit / 2, hit, hit + (1 - hit) / 3, hit + 2 * (1 - hit) / 3, 1.)
    if action.target.kind == "circle":
        checks.append(("stir", min(range(len(frames)), key=lambda i: frames[i]["head_above_rim_m"])))
    contact = action.contacts[0] if action.contacts else None
    if mode == "strip":
        return ReviewAction(phases, action.prop, action.fixture, action.target.anchor, prop_hand(action),
                            action.target.fixture_radius, hero=hero if contact is None else contact,
                            strip=12, strip_contact=contact)
    return ReviewAction(phases, action.prop, action.fixture, action.target.anchor, prop_hand(action),
                        action.target.fixture_radius, detail, tuple(checks), hero)


def measure_pose(action: Action, body: str, rest: np.ndarray, posed: np.ndarray, grips: tuple[int, int],
                 frames: list[dict[str, float]], contacts: list[int]) -> list[str]:
    """Measure the clearance visible on the same quantized vertices as the review render."""
    head = rest[:, 2] > .84 * HEIGHT
    scale = BODY_SCALE[body]
    if action.target.kind == "circle":
        for points, frame in zip(posed, frames):
            frame["head_above_rim_m"] = (float(points[head, 2].min()) - action.target.anchor[2]) * scale
        if action.target.rim_clearance and min(frame["head_above_rim_m"] for frame in frames) < action.target.rim_clearance:
            return [f"{body} {action.key}: head clearance at the rim"]
    if not (action.target.face_clearance or action.target.rebound_clearance):
        return []
    hit = contacts[0]
    failures = []
    if action.target.face_clearance:
        apex = max(range(max(1, hit)), key=lambda i: frames[i]["tip_above_anchor_m"])
        face = head & (rest[:, 1] < np.median(rest[head, 1]))
        for side, grip in zip(("R", "L"), grips):
            hand = np.linalg.norm(rest - rest[grip], axis=1) < .055
            distance = float(np.linalg.norm(posed[apex][hand, None] - posed[apex][None, face], axis=2).min()) * scale
            frames[apex][f"face_clearance_{side}_m"] = distance
            if distance < action.target.face_clearance:
                failures.append(f"{body} {action.key}: {side} hand clearance at the apex ({distance:.3f} m)")
    if action.target.rebound_clearance:
        release = frames[hit:hit + round(FPS * .25) + 1]
        rebound = [frame["tip_above_anchor_m"] * scale for frame in release]
        initial, lift_out = max(rebound[:5]), max(rebound)
        minimum = action.target.rebound_clearance
        if initial < .5 * minimum or lift_out < minimum:
            failures.append(f"{body} {action.key}: post-impact tip lift ({initial:.3f} m initial, {lift_out:.3f} m peak)")
    return failures


def timeline(action: Action, body: str) -> tuple[list[float], list[int]]:
    duration = action.duration if action.duration is not None else action.trim[1] - action.trim[0]
    intervals = round(duration * FPS / action.speed[body])
    if not 10 <= intervals <= 450:
        raise ValueError(f"{action.key}: unsupported duration of {intervals} frames")
    count = intervals + (not action.loop)
    phases = [i / intervals for i in range(count)]
    contacts = [min(count - 1, round(phase * intervals)) for phase in action.contacts]
    if len(set(contacts)) != len(contacts):
        raise ValueError(f"{action.key}: contact phases round to the same 30 fps frame")
    for i, time in zip(contacts, action.contacts):
        phases[i] = time if i != 0 or not action.loop else 0.
    return phases, contacts


def bake(kit: Path, out: Path, body: str, sources: tuple[tuple[Action, Source, Source | None], ...],
         mode: str) -> list[str]:
    def capture_rest(source: bpy.types.Object) -> None:
        nonlocal rest
        rest = np.array([tuple(vertex.co) for vertex in source.data.vertices])
        rest[:, 2] -= rest[:, 2].min()
        rest *= HEIGHT / rest[:, 2].max()

    rest = None
    arm, mesh, grips = legacy.source_rig(kit, body, 8000, stabilize_grips=True)
    legacy.anchor._walk(arm)
    bind = base._clip_bindings([arm])
    asset = f"s2_{body}"
    metadata = json.loads((Path(C.VAT_DIR) / f"{asset}_vat.json").read_text())
    rows, sockets, records = [], {}, {}
    prop_sockets: dict[str, list[Matrix]] = {}
    clips: dict[str, Clip] = {}
    review = {}
    failures = []
    overrides = {action.key for action, _, _ in sources}
    for name in (name for name in base.HUMAN if name not in overrides):
        base._play(bind[name])
        start = bind[name][0][1].frame_range[0]
        previous = metadata["clips"][name]
        count = round(previous["duration"] * FPS)
        clips[name] = Clip(start=len(rows), count=count, loop=True, duration=count / FPS)
        sockets[name] = []
        for i in range(count):
            frame = start + i * C.FPS / FPS
            bpy.context.scene.frame_set(int(frame), subframe=frame % 1.)
            positions = legacy.posed_positions(mesh)
            rows.append(positions)
            sockets[name].append(tuple(legacy.socket(arm.pose.bones[f"hand_{side}"], positions[index])
                                       for side, index in zip(("R", "L"), grips)))
    arm.animation_data.action = None
    fitted = None
    for action, (take, motion), lower_body in sources:
        guarded = action.arm_limits.get(body, action.arm_limits.get("default")) is not None
        needs_fitted = guarded or lower_body is not None
        if needs_fitted != fitted:
            fitted = needs_fitted
            arm, mesh, grips = legacy.source_rig(kit, body, 8000, stabilize_grips=True,
                                                before_bind=capture_rest, align_arm_weights=fitted,
                                                weight_adjust=stabilize_garment_weights)
            mesh.data.calc_loop_triangles()
            triangles = np.array([tuple(face.vertices) for face in mesh.data.loop_triangles])
            # Staff removal can leave unrendered vertices below the boots.
            surface_indices = np.unique(triangles)
            sole_indices = surface_indices[rest[surface_indices, 2] <= PLANTED_SOLE_M]
            shoes = {}
            for side in ("R", "L"):
                foot = mesh.vertex_groups[f"foot_{side}"].index
                shoes[side] = [int(index) for index in sole_indices if any(
                    group.group == foot and group.weight > .5 for group in mesh.data.vertices[int(index)].groups)]
                if not shoes[side]:
                    raise RuntimeError(f"{asset}: no rendered sole samples for foot {side}")
            rest_soles = {side: np.array([tuple(mesh.data.vertices[i].co) for i in indices])
                          for side, indices in shoes.items()}
        phases, contacts = timeline(action, body)
        start = len(rows)
        solver = Retarget(arm, motion, action, take, body, lower_body, rest_soles)
        clips[action.key] = Clip(start=start, count=len(phases), loop=action.loop,
                                 duration=(len(phases) - (not action.loop)) / FPS)
        if action.in_place:
            clips[action.key]["stride_m"] = solver.stride.length
        sockets[action.key] = []
        accuracy = []
        bone_names = tuple(bone.name for bone in arm.pose.bones)
        parents = tuple(-1 if bone.parent is None else bone_names.index(bone.parent.name) for bone in arm.pose.bones)
        matrices = []
        contact_wrists = None
        prop_points = None
        prop_extent = None
        if action.prop is not None:
            tool = legacy.prop_mesh(action.prop)
            points = np.asarray([tuple(vertex.co) for vertex in tool.data.vertices])
            prop_extent = float(np.ptp(points, axis=0).max())
            if action.target.solid:
                prop_points = points
            bpy.data.objects.remove(tool, do_unlink=True)
        if action.target.kind == "reach" and action.prop is not None:
            solver.pose(solver.contact)
            contact_wrists = wrist_center(arm, action)

        def constrain_frame(phase: float):
            return solver.constrain(phase, shoes, lambda: constrain(
                arm, mesh, grips, action, phase, contact_wrists, prop_points, body,
            ))

        for phase in phases:
            feet = solver.pose(phase)
            frame = constrain_frame(phase)
            matrices.append([tuple(tuple(row) for row in bone.matrix) for bone in arm.pose.bones])
            measured = {"phase": phase, "foot_R": feet[0], "foot_L": feet[1], **frame.errors}
            if action.torso_lean:
                bones = arm.pose.bones
                hip = bones["hips"].head
                measured["torso_lean_deg"] = math.degrees((bones["neck"].head - hip).angle(C.Z))
                measured["hip_x_m"], measured["hip_y_m"] = hip.x, hip.y
            if action.target.kind == "circle":
                neck, head = (arm.pose.bones[name].head.z for name in ("neck", "head"))
                measured["hand_below_chin_m"] = (neck + head) / 2 - frame.sockets[prop_hand(action)].translation.z
            if frame.prop is not None and action.target.kind == "swing":
                tip = frame.prop @ C.Vector(action.target.tip)
                measured["tip_above_anchor_m"] = tip.z - action.target.anchor[2]
            accuracy.append(measured)
            sockets[action.key].append(frame.sockets)
            if frame.prop is not None:
                prop_sockets.setdefault(action.key, []).append(frame.prop)
            rows.append(frame.positions)
        track = Track(bone_names, parents,
                      np.array([tuple(tuple(row) for row in bone.bone.matrix_local) for bone in arm.pose.bones]),
                      np.array(matrices), np.array([bone.bone.length for bone in arm.pose.bones]))
        write_track(out / "retarget" / body / f"{action.key}.npz", track, np.asarray(phases), FPS, BODY_SCALE[body])
        seam = None
        if action.loop:
            solver.pose(1.)
            end = constrain_frame(1.)
            first = rows[start]
            seam = float(np.linalg.norm(end.positions - first, axis=1).max())
        records[action.key] = {"take": take.id, "fps": take.fps, "trim_seconds": action.trim,
                               "contact_frames": contacts, "seam_m": seam, "frames": accuracy}
        if lower_body is not None and action.base_layer is not None:
            records[action.key]["base_layer"] = {"take": lower_body[0].id, "fps": lower_body[0].fps,
                                                  "trim_seconds": action.base_layer.trim}
        if rest is not None:
            weights = np.zeros((len(rest), 4))
            groups = {mesh.vertex_groups[name].index: column for column, name in enumerate(
                ("upper_arm_R", "lower_arm_R", "upper_arm_L", "lower_arm_L"))}
            for vertex in mesh.data.vertices:
                for group in vertex.groups:
                    if group.group in groups:
                        weights[vertex.index, groups[group.group]] = group.weight
            sampled = np.stack(rows[start:])
            surface = np.concatenate((sampled, end.positions[None])) if action.loop else sampled
            corrected = relax_joint_skin(rest, surface, triangles, weights, grips, loop=action.loop) if guarded else surface
            corrected = relax_body_skin(rest, corrected, triangles, grips, loop=action.loop,
                                        arm_weights=weights if guarded else None)
            if action.loop:
                records[action.key]["seam_m"] = float(np.linalg.norm(corrected[-1] - corrected[0], axis=1).max())
                corrected = corrected[:-1]
            fixed = np.r_[list(grips), np.flatnonzero(rest[:, 2] <= PLANTED_SOLE_M)]
            if not np.array_equal(sampled[:, fixed], corrected[:, fixed]):
                raise RuntimeError(f"{asset} {action.key}: surface correction moved a planted foot or palm")
            full = check_deformation(rest, corrected, triangles, weights, local_edges=guarded, check_joints=guarded)
            # Axis permutation and sign changes preserve the decoded float16 measurements.
            decoded = corrected.astype("<f2").astype(float)
            report = check_deformation(rest, decoded, triangles, weights, local_edges=guarded, check_joints=guarded)
            records[action.key]["deformation"] = asdict(report)
            records[action.key]["deformation"]["before_encoding"] = {
                "max_edge_ratio": full.max_edge_ratio, "min_area_ratio": full.min_area_ratio,
                "max_body_edge_ratio": full.max_body_edge_ratio, "max_body_edge_frame": full.max_body_edge_frame,
                "passed": full.passed,
            }
            records[action.key]["deformation"]["max_correction_m"] = float(
                np.linalg.norm(corrected - sampled, axis=2).max())
            rows[start:] = corrected
            print(f"MOCAP_DEFORMATION {body} {action.key} whole body {report.max_body_edge_ratio:.3f}x "
                  f"at frame {report.max_body_edge_frame} (limit 3.0x)", flush=True)
            if not full.passed or not report.passed:
                failures.append(f"{body} {action.key}")
                np.savez_compressed(out / f"{asset}-{action.key}-deformation.npz", rest=rest, triangles=triangles,
                                    weights=weights, frames=sampled, corrected=corrected, grips=grips)
            failures.extend(measure_pose(action, body, rest, decoded, grips, accuracy, contacts))
            soles = decoded[:, sole_indices]
            scale = BODY_SCALE[body]
            planted = {side: decoded[:, indices] * scale for side, indices in shoes.items()}
            times = np.asarray(phases)
            limits = action.plausibility
            worst_stretch = max((full, report), key=lambda value: value.max_body_edge_ratio)
            stretch = worst_stretch.max_body_edge_ratio
            checks = {
                **evaluate_support(times, track.names, track.heads * scale, track.tails * scale, planted, limits,
                                   foot_lock=action.foot_lock, loop=action.loop,
                                   stride=np.asarray(solver.stride) * scale),
                **evaluate_anatomy(track, body, scale, times, limits, rest_vertices=rest,
                                   posed_vertices=decoded, grips=grips),
                "garment_stretch": Check(stretch, worst_stretch.max_body_edge_frame, MAX_BODY_STRETCH,
                                         stretch <= MAX_BODY_STRETCH,
                                         "decoded float16" if worst_stretch is report else "before encoding"),
                "prop_ratio": check_prop(None if prop_extent is None else prop_extent * scale,
                                         HEIGHT * scale, limits.prop_ratio),
            }
            records[action.key]["plausibility"] = {name: asdict(result) for name, result in checks.items()}
            for name, result in checks.items():
                if not result.passed:
                    label = (f"{body} {action.key}: {name} frame {result.frame}, "
                             f"value {result.value}, limit {result.limit}, {result.detail}")
                    failures.append(label)
                    print(f"MOCAP_PLAUSIBILITY_FAIL {label}", flush=True)
            if action.target.release:
                placed = prop_sockets[action.key][contacts[0]:]
                drift = max((pose.translation - C.Vector(action.target.anchor)).length for pose in placed)
                records[action.key]["release_drift_m"] = drift * BODY_SCALE[body]
                if drift > .001:
                    failures.append(f"{body} {action.key}: released prop drift ({drift:.3f} m)")
                if action.foot_lock:
                    slide = float(np.linalg.norm(soles - soles[0], axis=2).max()) * BODY_SCALE[body]
                    records[action.key]["sole_slide_m"] = slide
                    if slide > .01:
                        failures.append(f"{body} {action.key}: planted sole slide ({slide:.3f} m)")
            if action.torso_lean:
                lean_phases, caps = zip(*action.torso_lean)
                for frame, sole in zip(accuracy, soles):
                    hip = np.array([frame["hip_x_m"], frame["hip_y_m"]])
                    ground = sole[:, :2]
                    excess = np.maximum(ground.min(axis=0) - hip, hip - ground.max(axis=0)).clip(0., None)
                    frame["hip_outside_stance_m"] = float(np.linalg.norm(excess)) * BODY_SCALE[body]
                if any(frame["torso_lean_deg"] > np.interp(frame["phase"], lean_phases, caps) + .05
                       for frame in accuracy):
                    failures.append(f"{body} {action.key}: torso exceeds the authored lean")
                if max(frame["hip_outside_stance_m"] for frame in accuracy) > .005:
                    failures.append(f"{body} {action.key}: hips move outside the planted feet")
            if mode != "bake":
                review[action.key] = review_for(action, accuracy, report, mode)
        print(f"MOCAP_BAKED {body} {action.key} {take.id} {len(phases)} frames", flush=True)
    vat = np.stack(rows)
    vat = np.stack((vat[:, :, 0], vat[:, :, 2], -vat[:, :, 1], np.ones(vat.shape[:2])), axis=2)
    if not np.isfinite(vat).all() or len(arm.pose.bones) != 17 or C.tri_count(mesh) > 8000:
        raise RuntimeError(f"{asset}: invalid mesh, bounds, or rig")
    split, width = base._static_mesh(asset, [mesh], legacy.blender_positions(vat[0]))
    C.export_glb(str(out / f"{asset}_static.glb"), [split], vertex_colors=True)
    bpy.data.objects.remove(split, do_unlink=True)
    metadata.update(width=vat.shape[1], vertex_count=vat.shape[1], split_vertex_count=width,
                    fps=FPS, frames=len(vat), clips=clips,
                    bounds_min=vat[:, :, :3].min((0, 1)).round(4).tolist(),
                    bounds_max=vat[:, :, :3].max((0, 1)).round(4).tolist(),
                    sockets={name: [[legacy.gltf_socket(matrix) for matrix in pair] for pair in frames]
                             for name, frames in sockets.items()},
                    fixtures={action.key: f"fixture_{action.key}" for action, _, _ in sources
                              if action.fixture is not None},
                    props={action.key: {"mesh": action.prop, "frames": [
                        legacy.gltf_socket(matrix) + [action.target.kind != "reach" or (
                            i <= records[action.key]["contact_frames"][0] if action.reverse or action.target.release
                            else i >= records[action.key]["contact_frames"][0]
                        )]
                        for i, matrix in enumerate(prop_sockets[action.key])
                    ]} for action, _, _ in sources if action.prop is not None})
    vat = vat.astype("<f2")
    vat.tofile(out / f"{asset}_vat.bin")
    (out / f"{asset}_vat.json").write_text(json.dumps(metadata, indent=1) + "\n")
    (out / f"{asset}-contacts.json").write_text(json.dumps(records, indent=1) + "\n")
    if mode != "bake" and not failures:
        capture_body(kit, out, mesh, body, tuple(review), clips, sockets, vat, review=review, fps=FPS,
                     prop_sockets=prop_sockets, selected_only=mode in ("final", "strip"))
    return failures


def main() -> None:
    if not os.environ.get("WORLDSIM_MUX2_LANE_FD"):
        raise RuntimeError("Run this Blender worker through mocap_build.py and the shared GPU lock")
    spec, index, kit_path, out_path, bodies, mode, *extra_specs = C.script_args()
    kit, out = Path(kit_path), Path(out_path)
    out.mkdir(parents=True, exist_ok=False)
    destination = out
    if mode == "compare":
        destination = out / "new"
        destination.mkdir()
    actions = load_many(tuple(map(Path, (spec, *extra_specs))))
    motions: dict[str, Motion] = {}

    def resolve(source: Action | BaseLayer) -> Source:
        take = select(source, Path(index))
        if take.id not in motions:
            motions[take.id] = read_bvh(take.path)
        return take, motions[take.id]

    sources = tuple((action, resolve(action), None if action.base_layer is None else resolve(action.base_layer))
                    for action in actions)
    if any(action.key == "idle" for action in actions):
        raise ValueError("The idle clip provides the static reference mesh")
    failures = [error for body in bodies.split(",") for error in bake(kit, destination, body, sources, mode)]
    if failures:
        raise RuntimeError(f"Pose checks failed for {', '.join(failures)}")
    for name in sorted({action.prop for action in actions if action.prop is not None}):
        C.reset_scene()
        C.export_glb(str(destination / f"grip_{name}.glb"), [legacy.prop_mesh(name)])
    for action in actions:
        if action.fixture is not None:
            C.reset_scene()
            setting = fixture(action.fixture, action.target.anchor, action.target.fixture_radius)
            if setting is None:
                raise ValueError(f"{action.key}: unknown fixture {action.fixture!r}")
            objects = [setting, *setting.children_recursive]
            if action.fixture == "hearth":
                # Runtime already draws the gameplay hearth. Export its matching vessel for the stir socket.
                vessel, = setting.children
                pose = setting.matrix_world @ vessel.matrix_basis
                vessel.parent = None
                vessel.matrix_world = pose
                objects = [vessel]
            C.export_glb(str(destination / f"fixture_{action.key}.glb"), objects, vertex_colors=True)
    if mode == "compare":
        original = out / "old"
        original.mkdir()
        review = {"chop": ReviewAction((0., .18, .35, .61, .76, .92), "axe", "block"),
                  "cook": ReviewAction(REVIEW_PHASES, "ladle", "pot")}
        for body in bodies.split(","):
            legacy.bake(kit, original, body, "", 8000, tuple(review), True, review=review)


if __name__ == "__main__":
    main()
