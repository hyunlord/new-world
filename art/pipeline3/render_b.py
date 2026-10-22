# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: Blender --background model.blend --python art/pipeline3/render_b.py -- LOOK STAGE OUTPUT
"""Bounded Blender route-B rendering: one idle look, then explicitly frozen layers."""
import hashlib
import json
import sys
import time
from pathlib import Path
from typing import Final

import bpy
from bpy_extras.object_utils import world_to_camera_view
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from b_render_masks import render_ids
from b_render_scene import camera_for, configure_scene

LAYERS: Final = ('BODY', 'TOP_hide_wrap', 'TOP_fur_cloak', 'HAIR_short',
                 'HAIR_long_tied', 'ITEM_stone_axe', 'CARRY_log_bundle')
FRAMES: Final = {'idle': ('B_idle', 1), 'walk_a': ('B_walk', 1),
                'walk_b': ('B_walk', 13), 'work_a': ('B_two_hand_chop', 1),
                'work_b': ('B_two_hand_chop', 13), 'sleep': ('B_sleep', 1)}


def pose(frame: str) -> None:
    """Select a source action without generating new animation keys."""
    action_name, number = FRAMES[frame]
    for obj in bpy.context.scene.objects:
        if obj.type == 'ARMATURE':
            obj.animation_data_create()
            action = bpy.data.actions[action_name]
            obj.animation_data.action = action
            if action.slots:
                obj.animation_data.action_slot = action.slots[0]
    bpy.context.scene.frame_set(number)
    bpy.context.view_layer.update()


def show(layers: tuple[str, ...]) -> None:
    """Render each layer separately; the lab supplies contact shadows."""
    for obj in bpy.context.scene.objects:
        if obj.type == 'MESH':
            obj.hide_render = obj.get('layer') not in layers


def render(prefix: Path, masks: bool) -> None:
    """Record an untouched beauty render and optional native index passes."""
    scene = bpy.context.scene
    if masks:
        metadata = {'camera_location': list(scene.camera.location),
                    'camera_rotation': list(scene.camera.rotation_euler),
                    'ortho_scale': scene.camera.data.ortho_scale}
        anchor = None
        if prefix.name.startswith('ITEM_'):
            anchor = bpy.data.objects['axe_grip'].matrix_world.translation
            metadata['anchor_source'] = 'axe_grip empty matrix_world.translation'
        if prefix.name.startswith('CARRY_'):
            rig = bpy.data.objects['B_Rig']
            anchor = (rig.matrix_world @ rig.pose.bones['torso'].matrix
                      @ rig.data.bones['torso'].matrix_local.inverted() @ Vector((0, .12, .72)))
            metadata['anchor_source'] = 'authored log attachment bind point(0,.12,.72) transformed by torso pose/rest matrices'
        if anchor is not None:
            point = world_to_camera_view(scene, scene.camera, anchor)
            metadata['anchor_world'] = list(anchor)
            metadata['anchor_game'] = [round(point.x * scene.render.resolution_x / 4),
                                       round((1 - point.y) * scene.render.resolution_y / 4)]
        prefix.with_suffix('.render.json').write_text(json.dumps(metadata, indent=2) + '\n')
    bpy.context.scene.render.filepath = str(prefix.with_suffix('.png'))
    bpy.ops.render.render(write_still=True)
    if masks:
        render_ids(prefix)


def main() -> None:
    """Require an explicit stage and enforce the three-look ledger."""
    args = sys.argv[sys.argv.index('--') + 1:]
    if len(args) != 3 or args[0] not in ('1', '2', '3') or args[1] not in ('look', 'layers'):
        raise SystemExit('Usage after --: LOOK[1..3] STAGE[look|layers] OUTPUT')
    look, stage, output = int(args[0]), args[1], Path(args[2])
    output.mkdir(parents=True, exist_ok=True)
    ledger_path = output / 'render_ledger.json'
    ledger = json.loads(ledger_path.read_text()) if ledger_path.exists() else {'looks': [], 'runs': []}
    if stage == 'look' and look in ledger['looks']:
        raise SystemExit('This look was already rendered; no silent repeat setup.')
    if stage == 'layers':
        freeze = json.loads((output / 'frozen_look.json').read_text())
        if freeze['look'] != look or look not in ledger['looks']:
            raise SystemExit('Full animation requires the reviewed frozen look.')
    started = time.monotonic()
    configure_scene(look)
    pose('idle')
    camera_for('S', False)
    if stage == 'look':
        ledger['looks'].append(look)
        ledger_path.write_text(json.dumps(ledger, indent=2) + '\n')
        show(('BODY', 'TOP_hide_wrap', 'HAIR_short'))
        render(output / f'look{look}_idle_S_4x', False)
    else:
        layer_dir = output / 'raw4x'
        layer_dir.mkdir(exist_ok=True)
        for direction in ('S', 'E', 'N'):
            pose('idle')
            camera_for(direction, False)
            for frame in FRAMES:
                if frame == 'sleep':
                    continue
                pose(frame)
                show(('BODY',))
                render(layer_dir / f'BODY_adult_{direction}_{frame}', True)
            pose('idle')
            for layer in LAYERS[1:]:
                show((layer,))
                render(layer_dir / f'{layer}_adult_{direction}_idle', True)
        pose('sleep')
        camera_for('S', True)
        show(('BODY',))
        render(layer_dir / 'BODY_adult_S_sleep', True)
    ledger['runs'].append({'look': look, 'stage': stage,
                           'elapsed_seconds': time.monotonic() - started,
                           'blend_sha256': hashlib.sha256(Path(bpy.data.filepath).read_bytes()).hexdigest(),
                           'blender': bpy.app.version_string,
                           'credit_cost': 0})
    ledger_path.write_text(json.dumps(ledger, indent=2) + '\n')


if __name__ == '__main__':
    main()
