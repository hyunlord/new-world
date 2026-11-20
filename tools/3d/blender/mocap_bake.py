"""Blender worker for the locked MOCAP1 command."""

from dataclasses import asdict
import json
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
from mocap_deform import check_deformation
from mocap_pose import Retarget, constrain, prop_hand, wrist_center
from mocap_skin import relax_joint_skin
from mocap_spec import Action, BaseLayer, Take, load, select
import s2clips as legacy
from s2clips_capture import Clip, ReviewAction, capture_body
from s2kit import HEIGHT

FPS = 30
REVIEW_PHASES = (0., .20, .40, .60, .80, .97)
Source = tuple[Take, Motion]


def review_for(action: Action, frames: list[dict[str, float]]) -> ReviewAction:
    phases = REVIEW_PHASES
    detail = None
    if action.target.kind == "swing" and action.contacts:
        hit = action.contacts[0]
        if action.retime and action.prop is not None and .16 < hit < .88:
            leading = (frame for frame in frames if 0. < frame["phase"] < hit)
            detail = max(leading, key=lambda frame: frame["tip_above_anchor_m"])["phase"]
            phases = (0., detail, hit, hit + .13 * (1 - hit), hit + .53 * (1 - hit), .96)
        elif .16 < hit < .88:
            phases = (0., .40 * hit, .68 * hit, hit, hit + .37 * (1 - hit), hit + .78 * (1 - hit))
    if action.target.kind == "reach" and not action.loop and action.contacts:
        hit = action.contacts[0]
        if .15 < hit < .85:
            phases = (0., hit / 2, hit, hit + (1 - hit) / 3, hit + 2 * (1 - hit) / 3, 1.)
    return ReviewAction(phases, action.prop, action.fixture, action.target.anchor, prop_hand(action),
                        action.target.fixture_radius, detail)


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
         render: bool) -> list[str]:
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
    fitted = False
    for action, (take, motion), lower_body in sources:
        guarded = action.arm_limits.get(body, action.arm_limits.get("default")) is not None
        needs_fitted = guarded or lower_body is not None
        if needs_fitted != fitted:
            fitted = needs_fitted
            arm, mesh, grips = legacy.source_rig(kit, body, 8000, stabilize_grips=True,
                                                before_bind=capture_rest if fitted else None,
                                                align_arm_weights=fitted)
        phases, contacts = timeline(action, body)
        start = len(rows)
        clips[action.key] = Clip(start=start, count=len(phases), loop=action.loop,
                                 duration=(len(phases) - (not action.loop)) / FPS)
        solver = Retarget(arm, motion, action, take, body, lower_body)
        sockets[action.key] = []
        accuracy = []
        contact_wrists = None
        prop_points = None
        if action.prop is not None and action.target.solid:
            tool = legacy.prop_mesh(action.prop)
            prop_points = np.asarray([tuple(vertex.co) for vertex in tool.data.vertices])
            bpy.data.objects.remove(tool, do_unlink=True)
        if action.target.kind == "reach" and action.prop is not None:
            solver.pose(solver.contact)
            contact_wrists = wrist_center(arm, action)
        for phase in phases:
            feet = solver.pose(phase)
            frame = constrain(arm, mesh, grips, action, phase, contact_wrists, prop_points, body)
            measured = {"phase": phase, "foot_R": feet[0], "foot_L": feet[1], **frame.errors}
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
        seam = None
        if action.loop:
            solver.pose(1.)
            end = constrain(arm, mesh, grips, action, 1., contact_wrists, prop_points, body)
            first = rows[start]
            seam = float(np.linalg.norm(end.positions - first, axis=1).max())
        records[action.key] = {"take": take.id, "fps": take.fps, "trim_seconds": action.trim,
                               "contact_frames": contacts, "seam_m": seam, "frames": accuracy}
        if lower_body is not None and action.base_layer is not None:
            records[action.key]["base_layer"] = {"take": lower_body[0].id, "fps": lower_body[0].fps,
                                                  "trim_seconds": action.base_layer.trim}
        if guarded and rest is not None:
            mesh.data.calc_loop_triangles()
            triangles = np.array([tuple(face.vertices) for face in mesh.data.loop_triangles])
            weights = np.zeros((len(rest), 4))
            groups = {mesh.vertex_groups[name].index: column for column, name in enumerate(
                ("upper_arm_R", "lower_arm_R", "upper_arm_L", "lower_arm_L"))}
            for vertex in mesh.data.vertices:
                for group in vertex.groups:
                    if group.group in groups:
                        weights[vertex.index, groups[group.group]] = group.weight
            sampled = np.stack(rows[start:])
            surface = np.concatenate((sampled, end.positions[None])) if action.loop else sampled
            corrected = relax_joint_skin(rest, surface, triangles, weights, grips, loop=action.loop)
            if action.loop:
                records[action.key]["seam_m"] = float(np.linalg.norm(corrected[-1] - corrected[0], axis=1).max())
                corrected = corrected[:-1]
            fixed = np.r_[list(grips), np.flatnonzero(rest[:, 2] <= .10)]
            if not np.array_equal(sampled[:, fixed], corrected[:, fixed]):
                raise RuntimeError(f"{asset} {action.key}: surface correction moved a planted foot or palm")
            full = check_deformation(rest, corrected, triangles, weights, local_edges=True)
            # Axis permutation and sign changes preserve the decoded float16 measurements.
            report = check_deformation(rest, corrected.astype("<f2"), triangles, weights, local_edges=True)
            records[action.key]["deformation"] = asdict(report)
            records[action.key]["deformation"]["before_encoding"] = {
                "max_edge_ratio": full.max_edge_ratio, "min_area_ratio": full.min_area_ratio, "passed": full.passed,
            }
            records[action.key]["deformation"]["max_correction_m"] = float(
                np.linalg.norm(corrected - sampled, axis=2).max())
            rows[start:] = corrected
            print(f"MOCAP_DEFORMATION {body} {action.key} VAT: stretch {report.max_edge_ratio:.3f} "
                  f"at frame {report.max_edge_frame}, area {report.min_area_ratio:.3f} "
                  f"at frame {report.min_area_frame}", flush=True)
            if not full.passed or not report.passed:
                failures.append(f"{body} {action.key}")
                np.savez_compressed(out / f"{asset}-{action.key}-deformation.npz", rest=rest, triangles=triangles,
                                    weights=weights, frames=sampled, corrected=corrected, grips=grips)
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
                    props={action.key: {"mesh": action.prop,
                                        "frames": [legacy.gltf_socket(matrix) for matrix in prop_sockets[action.key]]}
                           for action, _, _ in sources if action.prop is not None})
    vat = vat.astype("<f2")
    vat.tofile(out / f"{asset}_vat.bin")
    (out / f"{asset}_vat.json").write_text(json.dumps(metadata, indent=1) + "\n")
    (out / f"{asset}-contacts.json").write_text(json.dumps(records, indent=1) + "\n")
    if render:
        review = {action.key: review_for(action, records[action.key]["frames"]) for action, _, _ in sources}
        capture_body(kit, out, mesh, body, tuple(review), clips, sockets, vat, review=review, fps=FPS,
                     prop_sockets=prop_sockets)
    return failures


def main() -> None:
    if not os.environ.get("WORLDSIM_MUX2_LANE_FD"):
        raise RuntimeError("Run this Blender worker through mocap_build.py and the shared GPU lock")
    spec, index, kit_path, out_path, bodies, mode = C.script_args()
    kit, out = Path(kit_path), Path(out_path)
    out.mkdir(parents=True, exist_ok=False)
    destination = out
    if mode == "compare":
        destination = out / "new"
        destination.mkdir()
    actions = load(Path(spec))
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
    failures = [error for body in bodies.split(",")
                for error in bake(kit, destination, body, sources, render=mode != "bake")]
    if failures:
        raise RuntimeError(f"Shoulder/elbow deformation failed for {', '.join(failures)}")
    for name in sorted({action.prop for action in actions if action.prop is not None}):
        C.reset_scene()
        C.export_glb(str(destination / f"grip_{name}.glb"), [legacy.prop_mesh(name)])
    if mode == "compare":
        original = out / "old"
        original.mkdir()
        review = {"chop": ReviewAction((0., .18, .35, .61, .76, .92), "axe", "block"),
                  "cook": ReviewAction(REVIEW_PHASES, "ladle", "pot")}
        for body in bodies.split(","):
            legacy.bake(kit, original, body, "", 8000, tuple(review), True, review=review)


if __name__ == "__main__":
    main()
