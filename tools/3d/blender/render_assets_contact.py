"""Render paginated M3D-ASSETS comparisons at the actual game camera.

blender -b --factory-startup -P tools/3d/blender/render_assets_contact.py -- OUTPUT [--manifest=PATH] [FAMILY_OR_ID ...]
Each row is baseline 1x/2x, candidate 1x/2x; ground-target scale is 64/128 px/m.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: use Blender's bundled Python with the command above (bpy/numpy bundled).
import json
import math
import os
import subprocess
import sys
import traceback
from collections import defaultdict
from pathlib import Path

import bpy
import numpy as np
from bpy_extras.object_utils import world_to_camera_view
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as C

CELL = 512
ROWS = 4
BASE_REF = "a0260ce9a"
REFERENCES = {"station": "hearth", "item": "firewood", "flora": "tree_oak_a",
              "rocks": "boulder_small", "character": "villager"}


def scene() -> tuple[bpy.types.Object, bpy.types.Object]:
    """Set a reusable, flat-colour review stage."""
    C.reset_scene()
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.render.resolution_x = sc.render.resolution_y = CELL
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = "PNG"
    sc.display.shading.color_type = "MATERIAL"
    sc.display.shading.light = "STUDIO"
    sc.display.shading.show_shadows = True
    sc.world = bpy.data.worlds.new("review_world")
    sc.world.color = (0.80, 0.84, 0.88)
    camera = bpy.data.objects.new("review_camera", bpy.data.cameras.new("review_camera"))
    sc.collection.objects.link(camera)
    camera.data.type = "PERSP"
    camera.data.angle = math.radians(30)
    camera.data.clip_end = 500
    sc.camera = camera
    text = bpy.data.curves.new("review_label", "FONT")
    label = bpy.data.objects.new("review_label", text)
    sc.collection.objects.link(label)
    label.parent = camera
    mat = bpy.data.materials.new("review_text")
    mat.diffuse_color = (0.02, 0.02, 0.02, 1)
    text.materials.append(mat)
    return camera, label


def render_cell(path: Path, caption: str, zoom: int) -> np.ndarray:
    """Render in physical metres without per-model fitting or rescaling."""
    camera, label = bpy.context.scene.camera, bpy.data.objects["review_label"]
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    imported = set(bpy.data.objects) - before
    assert any(o.type == "MESH" for o in imported), f"empty model {path}"
    for obj in imported:
        if obj.type == "ARMATURE":
            obj.data.pose_position = "REST"
        for slot in obj.material_slots:
            mat = slot.material
            if mat and mat.node_tree:
                principled = next((n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
                if principled:
                    mat.diffuse_color = principled.inputs["Base Color"].default_value
    textured = any(slot.material and slot.material.node_tree and any(n.type == "TEX_IMAGE" for n in slot.material.node_tree.nodes) for obj in imported for slot in obj.material_slots)
    bpy.context.scene.display.shading.color_type = "TEXTURE" if textured else "MATERIAL"
    pitch = math.radians(50)
    direction = Vector((math.cos(pitch) / math.sqrt(2), -math.cos(pitch) / math.sqrt(2), math.sin(pitch)))
    distance = CELL / (2 * math.tan(math.radians(15)) * 64 * zoom)
    camera.location = direction * distance
    camera.data.shift_y = 0.18
    camera.rotation_euler = (-direction).to_track_quat("-Z", "Y").to_euler()
    bpy.context.view_layer.update()
    origin = world_to_camera_view(bpy.context.scene, camera, Vector((0, 0, 0)))
    metre = world_to_camera_view(bpy.context.scene, camera, Vector((1 / math.sqrt(2), 1 / math.sqrt(2), 0)))
    assert abs((metre.x - origin.x) * CELL - 64 * zoom) < 0.1, "camera scale drift"
    width = 4 * math.tan(math.radians(15))
    label.location = (-width * 0.48, width * (-0.46 + camera.data.shift_y), -2)
    label.data.size = width * 0.024
    label.data.body = f"{caption}\n{zoom}x | {64 * zoom}px/m | FOV30 pitch50"
    tmp = bpy.context.scene.render.filepath
    bpy.context.scene.render.filepath = tmp
    bpy.ops.render.render(write_still=True)
    image = bpy.data.images.load(tmp)
    pixels = np.empty(CELL * CELL * 4, dtype=np.float32)
    image.pixels.foreach_get(pixels)
    bpy.data.images.remove(image)
    os.remove(tmp)
    for obj in imported:
        bpy.data.objects.remove(obj, do_unlink=True)
    return pixels.reshape(CELL, CELL, 4)


def save_page(cells: list[np.ndarray], path: Path) -> None:
    """Save four comparison columns, with top-to-bottom model order."""
    rows = len(cells) // 4
    assert rows > 0 and len(cells) % 4 == 0
    pixels = np.ones((rows * CELL, 4 * CELL, 4), dtype=np.float32)
    for index, cell in enumerate(cells):
        row, col = rows - 1 - index // 4, index % 4
        pixels[row * CELL:(row + 1) * CELL, col * CELL:(col + 1) * CELL] = cell
    image = bpy.data.images.new("review_page", 4 * CELL, rows * CELL, alpha=True)
    image.pixels.foreach_set(pixels.ravel())
    image.filepath_raw = str(path)
    image.file_format = "PNG"
    image.save()
    bpy.data.images.remove(image)


def main() -> None:
    """Render selected current manifest families against pinned M3D-1 references."""
    global CELL
    args = C.script_args()
    assert args, "supply an external output directory"
    out = Path(args[0]).resolve()
    out.mkdir(parents=True, exist_ok=True)
    baseline = json.loads(subprocess.check_output(["git", "show", f"{BASE_REF}:assets/3d/models/manifest.json"], cwd=C.REPO))
    manifest_path = next((arg.split("=", 1)[1] for arg in args[1:] if arg.startswith("--manifest=")), C.MANIFEST)
    filters = [arg for arg in args[1:] if not arg.startswith("--manifest=")]
    with open(manifest_path) as source:
        manifest = json.load(source)
    families = defaultdict(list)
    for model_id, entry in sorted(manifest["models"].items()):
        if not filters or entry["family"] in filters or model_id in filters:
            families[entry["family"]].append((model_id, entry["file"]))
    assert families, "no matching models"
    records = []
    for family, models in sorted(families.items()):
        sizes = [max(entry["height"], *entry["footprint"]) for entry in manifest["models"].values() if entry["family"] == family]
        CELL = max(512, math.ceil(max(sizes) * 1.7) * 128)
        available = [key for key, entry in baseline["models"].items() if entry["family"] == family]
        ref_id = REFERENCES.get(family, available[0] if available else "hearth")
        row_references = {}
        for model_id, _filename in models:
            species = manifest["models"][model_id].get("species", ref_id)
            row_references[model_id] = species if species in baseline["models"] else ref_id
        reference_ids = sorted(set(row_references.values()))
        for key in reference_ids:
            ref = baseline["models"][key]
            CELL = max(CELL, math.ceil(max(ref["height"], *ref["footprint"]) * 1.7) * 128)
        scene()
        bpy.context.scene.render.filepath = str(out / "_cell.png")
        references = {}
        for key in reference_ids:
            ref = baseline["models"][key]
            ref_path = out / f"reference_{key}.glb"
            ref_path.write_bytes(subprocess.check_output(["git", "show", f"{BASE_REF}:assets/3d/{ref['file']}"], cwd=C.REPO))
            references[key] = [render_cell(ref_path, f"M3D-1 {key}", zoom) for zoom in (1, 2)]
        for start in range(0, len(models), ROWS):
            cells = []
            for model_id, filename in models[start:start + ROWS]:
                cells.extend(references[row_references[model_id]])
                cells.extend(render_cell(Path(C.REPO) / "assets/3d" / filename, model_id, zoom) for zoom in (1, 2))
            path = out / f"{family}_{start // ROWS + 1:03}.png"
            save_page(cells, path)
            records.append({"family": family, "sheet": path.name, "models": [m[0] for m in models[start:start + ROWS]], "reference": ref_id, "row_references": {m[0]: row_references[m[0]] for m in models[start:start + ROWS]}})
    (out / "index.json").write_text(json.dumps({"baseline": BASE_REF, "fov": 30, "pitch": 50, "pixels_per_metre": [64, 128], "sheets": records}, indent=2) + "\n")
    print(f"CONTACT PASS: {sum(len(v) for v in families.values())} models, {len(records)} sheets")


if __name__ == "__main__":
    try:
        main()
    except (AssertionError, OSError, KeyError, ValueError, TypeError, RuntimeError, subprocess.CalledProcessError):
        traceback.print_exc()
        sys.exit(1)
