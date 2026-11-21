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
from stations import stone_ring


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
    detail: float | None = None
    checks: tuple[tuple[str, int], ...] = ()
    hero: float = .60
    strip: int = 0
    strip_contact: float | None = None


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
    ground = pad.build_object()
    # Fixture palette overrides share material names. Keep the stage colour independent.
    ground.data.materials[0] = ground.data.materials[0].copy()
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
    if name == "seat_log" and anchor is not None:
        x, y, height = anchor
        diameter = .40 if radius is None else min(.45, 2 * radius)
        axis = height - diameter / 2
        shape.rod("trunk", (x - .39, y, axis), (x + .39, y, axis), diameter / 2,
                  segs=13, r2=.94 * diameter / 2, cap_mat="wood")
        for dx in (-.27, .27):
            shape.rod("wood_dark", (x + dx, y - .12, .025), (x + dx, y + .12, .025), .03, segs=8)
        return shape.build_object()
    if name == "workbench" and anchor is not None:
        x, y, height = anchor
        for dx in (-.20, .20):
            for dy in (-.09, .09):
                shape.rod("trunk", (x + dx, y + dy, 0), (x + dx, y + dy, height - .035), .04, segs=7)
        for dy in (-.112, 0, .112):
            shape.box("wood", (.58, .108, .075), loc=(x, y + dy, height - .0375))
        return shape.build_object()
    if name == "stake" and anchor is not None and radius is not None:
        x, y, top = anchor
        shoulder = top - .06
        shape.rod("trunk", (x, y, 0), (x, y, shoulder), 1.18 * radius, segs=9, r2=1.12 * radius)
        shape.rod("wood", (x, y, shoulder), (x, y, top), 1.12 * radius, segs=9, r2=radius)
        return shape.build_object()
    if name in ("soil", "plant_patch") and anchor is not None:
        x, y, top = anchor
        patch = .25 if radius is None else radius
        soil_top = top if name == "soil" else 0.
        shape.rod("earth", (x, y, soil_top - .018), (x, y, soil_top + .003), patch, segs=11, cap_mat="earth")
        if name == "plant_patch":
            for dx, dy in ((-.12, -.05), (.08, .08), (.13, -.09), (-.02, .10), (0, 0)):
                for sign in (-1, 1):
                    shape.rod("thatch", (x + dx, y + dy, soil_top),
                              (x + dx + sign * .021, y + dy + .024, top), .009, segs=3, r2=.001)
        return shape.build_object()
    if name not in ("cook", "pot", "hearth"):
        return None
    centre = P.POT_CENTRE if anchor is None else anchor[:2]
    pot_rim = .335 if anchor is None else anchor[2]
    pot_scale = 1. if radius is None else radius / .199
    raised = max(0., pot_rim - .335)
    if name == "hearth":
        shape.colors["stone"] = (.29, .245, .19)
        shape.colors["earth"] = (.24, .22, .21)
        stone_ring(shape, 9, .182 * pot_scale, .110, scale=(1., .86, .79))
        shape.cyl("earth", .168 * pot_scale, .025, segs=9)
        shape.ball("glow", .106, (0, 0, .031), (1, 1, .28), subdiv=1)
        for angle in (.35, 2.55, 4.65):
            shape.rod("wood_dark", (.135 * math.cos(angle), .135 * math.sin(angle), .03), (0, 0, .06),
                      .022, segs=5)
            shape.cyl("glow", .049, .115, loc=(.053 * math.cos(angle), .053 * math.sin(angle), .032),
                      segs=5, r2=0.)
    elif raised > .02:
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
    if name == "hearth":
        hearth.matrix_world = Matrix.Translation((*centre, 0.))
    rings = ((.085, .048), (.161, .077), (.206, .164), (.210, .235),
             (.198, .297), (.207, .322), (.199, .335), (.180, .326),
             (.177, .293), (.186, .233), (.183, .164), (.145, .103), (.077, .074))
    segments = 24
    points = []
    for radius, z in rings:
        for side in range(segments):
            angle = math.tau * side / segments
            r = radius * pot_scale * (1 + .012 * math.sin(3 * angle))
            x, y = (0., 0.) if name == "hearth" else centre
            points.append((x + r * math.cos(angle), y + r * math.sin(angle), z + pot_rim - .335))
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
                 prop_sockets: Mapping[str, Sequence[Matrix]] | None = None,
                 selected_only: bool = False) -> None:
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
        detail = min(count - 1, round(end * action.detail)) if action is not None and action.detail is not None else None
        checks = dict(action.checks) if action is not None else {}
        hero = None if action is None else min(count - 1, round(end * action.hero))
        if detail is not None:
            review_clips[name]["detail"] = detail
        if checks:
            review_clips[name]["checks"] = checks
        if hero is not None:
            review_clips[name]["hero"] = hero
        hand = 0 if action is None else action.hand
        if context is not None:
            context.matrix_world = stage_scale @ context.matrix_world
        standard = set(picks) if hero is None or not selected_only else {hero}
        if action is not None and action.strip and body == "p1":
            if not 3 <= action.strip <= count:
                raise ValueError(f"{name} needs between three and {count} strip frames")  # noqa: GENERIC_ERR_OK
            strip = [round((count - 1) * i / (action.strip - 1)) for i in range(action.strip)]
            if action.strip_contact is not None:
                contact = min(count - 1, round(end * action.strip_contact))
                if contact not in strip:
                    nearest = min(range(1, len(strip) - 1), key=lambda i: abs(strip[i] - contact))
                    strip[nearest] = contact
                    strip.sort()
            review_clips[name]["strip"] = strip
            standard.update(strip)
        selected = sorted(standard | set(checks.values()) | ({detail} if detail is not None else set()))
        for frame in (range(count) if body == "p1" and not selected_only else selected):
            positions = A.blender_positions(vat[clip["start"] + frame])
            geometry.vertices.foreach_set("co", positions.ravel())
            geometry.update()
            if prop is not None:
                transform = prop_sockets[name][frame] if prop_sockets is not None else sockets[name][frame][hand]
                prop.matrix_world = stage_scale @ transform
            for view, direction in sides.items():
                focal = Vector((0, -.29, .98))
                camera_object.location = focal + 6 * direction.normalized()
                camera_object.rotation_euler = (-direction).to_track_quat("-Z", "Y").to_euler()
                if frame in standard or (body == "p1" and not selected_only and view == "quarter"):
                    scene.render.filepath = str(directory / f"{name}-{body}-{view}-{frame:03d}.png")
                    bpy.ops.render.render(write_still=True)
                if frame == detail:
                    low, high = scale * np.percentile(positions[:, 2], [65, 100])
                    if prop is not None:
                        high = max(high, max((prop.matrix_world @ vertex.co).z for vertex in prop.data.vertices))
                    focal = Vector((0, -.11, (low + high) / 2))
                    camera_object.location = focal + 6 * direction.normalized()
                    camera_object.data.ortho_scale = max(1.18, 1.2 * (high - low))
                    scene.render.resolution_x = scene.render.resolution_y = 440
                    scene.render.filepath = str(directory / f"detail-{name}-{body}-{view}-{frame:03d}.png")
                    bpy.ops.render.render(write_still=True)
                    camera_object.data.ortho_scale = 2.45
                    scene.render.resolution_x = scene.render.resolution_y = 260
                for check, sample in checks.items():
                    if sample != frame:
                        continue
                    anchor = Vector(action.anchor) * scale
                    if check == "rebound":
                        centre = anchor + Vector((0., .05, .18)) * scale
                        width = max(.65, .92 * scale)
                    elif check == "stir":
                        centre = Vector((anchor.x * .5, anchor.y * .6, .51 * scale))
                        width = 1.32 * scale
                    else:
                        centre = Vector((0., -.04, .65 * scale))
                        width = 1.06 * scale
                    camera_object.location = centre + 6 * direction.normalized()
                    camera_object.data.ortho_scale = width
                    scene.render.resolution_x = scene.render.resolution_y = 440
                    scene.render.filepath = str(directory / f"detail-{check}-{name}-{body}-{view}-{frame:03d}.png")
                    bpy.ops.render.render(write_still=True)
                    camera_object.data.ortho_scale = 2.45
                    scene.render.resolution_x = scene.render.resolution_y = 260
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
