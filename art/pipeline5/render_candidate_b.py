# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: Blender --background --factory-startup --python art/pipeline5/render_candidate_b.py -- SOURCE_DIRECTORY OUTPUT_DIRECTORY
"""Render one whole UBC figure using TRIAL3b frozen look 3; no masks or clips."""
import hashlib
import json
import math
import sys
import time
from pathlib import Path

import bpy
from mathutils import Matrix

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'pipeline3'))
from render_b import camera_for, configure_scene, render


def main() -> None:
    """Import the supplied CC0 meshes, set a static idle, and retain provenance."""
    source, output = (Path(arg) for arg in sys.argv[sys.argv.index('--') + 1:])
    output.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    body = source / 'Base Characters/Godot - UE/Superhero_Male_FullBody.gltf'
    hair = source / 'Hairstyles/Rigged to Head Bone/glTF (Godot -Unreal)/Hair_Buzzed.gltf'
    bpy.ops.import_scene.gltf(filepath=str(body))
    rig = bpy.data.objects['Armature']
    for name, degrees in (('upperarm_l', 72), ('upperarm_r', -72)):
        bone = rig.pose.bones[name]
        pivot = bone.head.copy()
        bone.matrix = (Matrix.Translation(pivot)
                       @ Matrix.Rotation(math.radians(degrees), 4, 'Y')
                       @ Matrix.Translation(-pivot) @ bone.matrix)
        bpy.context.view_layer.update()
    for obj in bpy.context.scene.objects:
        if obj.type == 'MESH':
            obj['layer'] = 'BODY'
    bpy.ops.import_scene.gltf(filepath=str(hair))
    for obj in bpy.context.scene.objects:
        if obj.type == 'MESH' and obj.get('layer') != 'BODY':
            obj['layer'] = 'HAIR_short'
    for material in bpy.data.materials:
        material.pass_index = 2 if 'Hair' in material.name else 1
    # A static turn supplies the same front-three-quarter presentation as Route I.
    for obj in bpy.context.scene.objects:
        if obj.parent is None:
            obj.rotation_euler.z += math.radians(-20)
    bpy.context.view_layer.update()
    configure_scene(3)
    camera_for('S', False)
    bpy.context.scene.render.resolution_x = 1024
    bpy.context.scene.render.resolution_y = 1536
    # Keep the frozen 2px ink stroke at its original 256x384 relative width.
    bpy.context.scene.render.line_thickness = 4
    bpy.ops.wm.save_as_mainfile(filepath=str(output / 'candidate_b.blend'))
    render(output / 'B', False)
    metadata = {
        'look': 3, 'render_count': 1, 'blender': bpy.app.version_string,
        'elapsed_seconds': time.monotonic() - started,
        'sources': [{'path': str(p.relative_to(source)),
                     'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
                    for p in (body, body.with_suffix('.bin'), hair, hair.with_suffix('.bin'))],
        'pose': 'Static upperarm rotations +72/-72 degrees about rig Y, no keyframes',
        'orientation': 'S camera with whole figure -20 degrees Z',
        'shader': 'Unmodified TRIAL3b configure_scene(3); material skin=1 hair=2',
        'resolution': [1024, 1536], 'line_scale': 4,
        'missing_clothing': True, 'mask_passes': False, 'clips': False,
    }
    (output / 'render_metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')


if __name__ == '__main__':
    main()
