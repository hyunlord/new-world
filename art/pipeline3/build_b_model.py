# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: Blender -b --python art/pipeline3/build_b_model.py
# bpy is provided by Blender; no external Python packages or paid services.
"""Build TRIAL3B's first CC0 rig candidate and record reproducible evidence."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Final

import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from b_model_clips import clips
from b_model_source import reshape_source
from b_model_wardrobe import props, wardrobe

ROOT: Final = Path("/Users/rexxa/work/arttrial3-data/b")
SOURCE: Final = ROOT / "sources/kenney-mini/Models/GLB format/character-male-a.glb"
OUT: Final = ROOT / "model"


def main() -> None:
    """Construct real Blender assets and the measurements required by the renderer."""
    OUT.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(SOURCE))
    rig = bpy.data.objects["character-male-a"]
    rig.name = "B_Rig"
    rig.animation_data_clear()
    for bone in rig.pose.bones:
        bone.matrix_basis.identity()
    for obj in list(bpy.data.objects):
        if obj.name not in ("B_Rig", "body-mesh", "head-mesh"):
            bpy.data.objects.remove(obj, do_unlink=True)
    original = {
        name: {
            "vertices": len(bpy.data.objects[name].data.vertices),
            "polygons": len(bpy.data.objects[name].data.polygons),
        }
        for name in ("body-mesh", "head-mesh")
    }
    body = reshape_source(rig)
    wardrobe(rig)
    actions = clips(rig)
    props(rig)
    bpy.context.view_layer.update()
    points = []
    for obj in body:
        evaluated = obj.evaluated_get(bpy.context.evaluated_depsgraph_get())
        points.extend(
            [evaluated.matrix_world @ vertex.co for vertex in evaluated.data.vertices]
        )
    bounds = [[min(p[i] for p in points), max(p[i] for p in points)] for i in range(3)]
    record = {
        "source_url": "https://kenney.nl/assets/mini-characters",
        "license": "CC0 1.0",
        "license_url": "https://creativecommons.org/publicdomain/zero/1.0/",
        "license_sha256": hashlib.sha256(
            (ROOT / "sources/kenney-mini/License.txt").read_bytes()
        ).hexdigest(),
        "base_attempt": 1,
        "credits": 0,
        "source_path": str(SOURCE),
        "source_sha256": hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
        "source_geometry": original,
        "original_total_height": 0.67132497,
        "original_head_height": 0.32807517,
        "original_head_ratio": 0.32807517 / 0.67132497,
        "adapted_rest_height": 1.0,
        "adapted_head_height": 0.30,
        "adapted_head_ratio": 0.30,
        "idle_body_bounds_xyz": bounds,
        "coordinate_system": {"up": "+Z", "forward": "-Y", "sole_z": 0},
        "bone_names": [bone.name for bone in rig.data.bones],
        "clips": [
            {
                "action": name,
                "distinct_keyframe_times": times,
                "authored_in_blender": True,
            }
            for name, times in actions
        ],
        "frame_map": {
            "idle": ["B_idle", 1],
            "walk_a": ["B_walk", 1],
            "walk_b": ["B_walk", 13],
            "work_a": ["B_two_hand_chop", 1],
            "work_b": ["B_two_hand_chop", 13],
            "sleep": ["B_sleep", 1],
        },
        "anatomical_parts": [
            {
                "object": obj.name,
                "id": obj["anatomical_id"],
                "source_mesh": obj["source_mesh"],
                "retained_polygons": obj["source_polygon_count"],
            }
            for obj in body
        ],
        "edits": [
            "Source body polygons retained and reshaped, atlas replaced with neutral skin",
            "Source dark hair and blue-grey glasses polygons removed from head; brown skin preserved",
            "Seven source bones retained; bind coordinates adapted to sculpted proportions",
            "New primitive loincloth, hide wrap, fur cloak, two hair variants, stone axe with grip empty, three-log bundle",
            "Source animation library replaced with five authored pose clips, at most five distinct key times each",
        ],
        "limitations": [
            "Source body includes sculpted clothing surfaces recolored as skin; residual seam geometry is possible",
            "Seven-bone rigid-limb source has no elbow/knee articulation",
            "Projected hand, foot, limb and head thresholds require renderer checker; world dimensions are not proof",
        ],
    }
    (OUT / "model_evidence.json").write_text(json.dumps(record, indent=2) + "\n")
    bpy.context.scene["model_evidence"] = str(OUT / "model_evidence.json")
    bpy.context.scene["source_sha256"] = record["source_sha256"]
    bpy.context.scene.frame_start = 1
    bpy.context.scene.frame_end = 25
    bpy.context.scene.render.fps = 12
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT / "model.blend"))
    bpy.ops.export_scene.gltf(
        filepath=str(OUT / "model.glb"),
        export_format="GLB",
        export_animations=True,
        export_animation_mode="ACTIONS",
        export_extras=True,
    )
    print(
        json.dumps(
            {
                "model": str(OUT / "model.blend"),
                "bounds": bounds,
                "actions": actions,
                "bones": record["bone_names"],
            }
        )
    )


if __name__ == "__main__":
    main()
