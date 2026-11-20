"""Render the review poses from the actual quantized VAT rows and socket matrices."""

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
import hashlib
import json
import math
from pathlib import Path
from typing import TypedDict

import bpy
from mathutils import Matrix, Vector
import numpy as np

import common as C
import s2clips as A
import s2clips_pose as P


PROPS = {"chop": "axe", "carry": "logs", "cook": "ladle"}
BODY_SCALE = {"p1": 1., "p2": .95, "p3": .70, "p4": .97}


@dataclass(frozen=True, slots=True)
class ReviewAction:
    """Select review frames and place the prop and its contact fixture."""

    phases: tuple[float, ...]
    prop: str | None
    fixture: str | None
    anchor: tuple[float, float, float] | None = None
    hand: int = 0
    radius: float | None = None


class Clip(TypedDict):
    start: int
    count: int
    loop: bool
    duration: float


def camera() -> bpy.types.Object:
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = scene.render.resolution_y = 260
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.display.render_aa = "8"
    shading = scene.display.shading
    shading.light = "STUDIO"
    shading.color_type = "TEXTURE"
    shading.show_shadows = True
    shading.show_cavity = True
    shading.cavity_type = "BOTH"
    scene.view_settings.view_transform = "Standard"
    scene.view_settings.exposure = .30
    camera_object = bpy.data.objects.new("Review Camera", bpy.data.cameras.new("Review Camera"))
    scene.collection.objects.link(camera_object)
    camera_object.data.type = "ORTHO"
    camera_object.data.ortho_scale = 2.45
    scene.camera = camera_object
    pad = C.Model("Review ground")
    pad.box("earth", (200, 200, .05), loc=(0, 0, -.025))
    pad.build_object()
    return camera_object


def fixture(name: str | None, anchor: tuple[float, float, float] | None = None,
            radius: float | None = None) -> bpy.types.Object | None:
    shape = C.Model("Action setting")
    if name in ("chop", "block"):
        x, y, top = P.CHOP_CONTACT if anchor is None else anchor
        trunk_radius = .162 if radius is None else radius
        shape.rod("trunk", (x, y, 0), (x, y, top), trunk_radius,
                  segs=13, r2=trunk_radius * (.153 / .162), cap_mat="wood")
        for length in (.035, .078, .120):
            length *= trunk_radius / .162
            for i in range(20):
                start, end = (math.tau * (i + offset) / 20 for offset in (0, 1))
                shape.rod("wood_dark", (x + length * math.cos(start), y + length * math.sin(start), top + .0008),
                          (x + length * math.cos(end), y + length * math.sin(end), top + .0008), .002, segs=4)
        return shape.build_object()
    if name not in ("cook", "pot"):
        return None
    centre = P.POT_CENTRE if anchor is None else anchor[:2]
    pot_rim = .335 if anchor is None else anchor[2]
    pot_scale = 1. if radius is None else radius / .180
    raised = max(0., pot_rim - .335)
    if raised > .02:
        shape.rod("earth", (*centre, 0), (*centre, raised + .062), .166 * pot_scale,
                  segs=11, r2=.101 * pot_scale)
        for level in (.30, .74):
            for i in range(8):
                angle = math.tau * (i + level) / 8
                shape.ball("stone", .103, (centre[0] + .118 * pot_scale * math.cos(angle),
                           centre[1] + .118 * pot_scale * math.sin(angle), level * raised),
                           (1.02, .83, max(.49, raised * 2.5)), subdiv=1, jitter=.009)
    else:
        for i in range(7):
            angle = math.tau * i / 7
            shape.ball("stone", .105, (centre[0] + .235 * math.cos(angle),
                       centre[1] + .235 * math.sin(angle), .056), (1., .84, .71), subdiv=1, jitter=.008)
        for x, y in ((.13, 0), (-.11, -.04), (.02, -.13), (-.03, .15)):
            shape.ball("wood_dark", .065, (centre[0] + x, centre[1] + y, .035), (1, .7, .45))
    hearth = shape.build_object()
    rings = ((.085, .048), (.161, .077), (.206, .164), (.210, .235),
             (.198, .297), (.207, .322), (.199, .335), (.180, .326),
             (.177, .293), (.186, .233), (.183, .164), (.145, .103), (.077, .074))
    segments = 24
    points = []
    for radius, z in rings:
        for side in range(segments):
            angle = math.tau * side / segments
            r = radius * pot_scale * (1 + .012 * math.sin(3 * angle))
            points.append((centre[0] + r * math.cos(angle), centre[1] + r * math.sin(angle), z + pot_rim - .335))
    faces = [(row * segments + side, row * segments + (side + 1) % segments,
              (row + 1) * segments + (side + 1) % segments, (row + 1) * segments + side)
             for row in range(len(rings) - 1) for side in range(segments)]
    faces.extend([tuple(reversed(range(segments))), tuple(range(len(points) - segments, len(points)))])
    geometry = bpy.data.meshes.new("Hearth vessel")
    geometry.from_pydata(points, [], faces)
    geometry.materials.append(C.material("clay"))
    pot = bpy.data.objects.new("Hearth vessel", geometry)
    bpy.context.scene.collection.objects.link(pot)
    pot.parent = hearth
    return hearth


def capture_body(root: Path, out: Path, mesh: bpy.types.Object, body: str, names: tuple[str, ...],
                 clips: Mapping[str, Clip], sockets: Mapping[str, Sequence[tuple[Matrix, Matrix]]],
                 vat: np.ndarray, *, review: Mapping[str, ReviewAction] | None = None,
                 fps: int = A.FPS, asset_id: str | None = None,
                 prop_sockets: Mapping[str, Sequence[Matrix]] | None = None) -> None:
    mesh.hide_render = True
    geometry = mesh.data.copy()
    subject = bpy.data.objects.new(body + " VAT review", geometry)
    bpy.context.scene.collection.objects.link(subject)
    scale = BODY_SCALE[body] if review is not None else 1.
    subject.scale = (scale, scale, scale)
    stage_scale = Matrix.Scale(scale, 4)
    atlas = bpy.data.images.load(str(root / f"s2_{body}_atlas.png"))
    material = C.plain_material("Review atlas")
    texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    texture.image = atlas
    material.node_tree.nodes.active = texture
    material.node_tree.links.new(texture.outputs["Color"],
                                 material.node_tree.nodes.get("Principled BSDF").inputs["Base Color"])
    geometry.materials.clear()
    geometry.materials.append(material)
    camera_object = camera()
    directory = out / "frames"
    directory.mkdir(exist_ok=True)
    scene = bpy.context.scene
    sides = {"side": Vector((-1, -.10, .10)), "quarter": Vector((-.72, -1, .33))}
    review_clips = {}
    for name in names:
        clip = clips[name]
        count = clip["count"]
        action = review[name] if review is not None else None
        if action is not None:
            end = count if clip["loop"] else count - 1
            if count < 6 or len(action.phases) != 6 or not 0 <= min(action.phases) <= max(action.phases) <= 1:
                raise ValueError(f"{name} needs six review phases between zero and one")
            rounded = [min(count - 1, round(end * phase)) for phase in action.phases]
            chosen = set(rounded)
            for preferred in sorted(chosen):
                for _ in range(rounded.count(preferred) - 1):
                    chosen.add(min(set(range(count)) - chosen, key=lambda row: (abs(row - preferred), row)))
            picks = sorted(chosen)
            review_clips[name] = {**clip, "picks": picks}
        elif name == "chop":
            picks = [round(count * phase) for phase in (0., .26, .61, .81)]
        elif clip["loop"]:
            picks = [round(count * phase / 4) for phase in range(4)]
        else:
            picks = [round((count - 1) * phase / 3) for phase in range(4)]
        prop_name = action.prop if action is not None else PROPS.get(name)
        prop = A.prop_mesh(prop_name) if prop_name is not None else None
        context = fixture(name) if action is None else fixture(action.fixture, action.anchor, action.radius)
        hand = 0 if action is None else action.hand
        if context is not None:
            context.matrix_world = stage_scale @ context.matrix_world
        for frame in (range(count) if body == "p1" else picks):
            geometry.vertices.foreach_set("co", A.blender_positions(vat[clip["start"] + frame]).ravel())
            geometry.update()
            if prop is not None:
                transform = prop_sockets[name][frame] if prop_sockets is not None else sockets[name][frame][hand]
                prop.matrix_world = stage_scale @ transform
            for view, direction in sides.items():
                if frame not in picks and view == "side":
                    continue
                focal = Vector((0, -.29, .98))
                camera_object.location = focal + 6 * direction.normalized()
                camera_object.rotation_euler = (-direction).to_track_quat("-Z", "Y").to_euler()
                scene.render.filepath = str(directory / f"{name}-{body}-{view}-{frame:03d}.png")
                bpy.ops.render.render(write_still=True)
        for subject_mesh in (prop, context):
            if subject_mesh is not None:
                for child in list(subject_mesh.children):
                    bpy.data.objects.remove(child, do_unlink=True)
                bpy.data.objects.remove(subject_mesh, do_unlink=True)
    if review is not None:
        source = asset_id or f"s2_{body}"
        vat_sha256 = hashlib.sha256(vat.tobytes()).hexdigest()
        (directory / f"review-{body}.json").write_text(json.dumps({
            "body": body, "fps": fps, "asset": source, "vat_sha256": vat_sha256, "clips": review_clips,
        }, indent=2) + "\n")
