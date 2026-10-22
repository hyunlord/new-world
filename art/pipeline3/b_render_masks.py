# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by Blender render_b.py.
"""Native unfiltered material/object ID passes, separate from visible artwork."""
from pathlib import Path

import bpy


def render_ids(prefix: Path) -> None:
    """Save IndexMA and anatomical IndexOB without changing mesh or beauty pixels."""
    scene = bpy.context.scene
    view = bpy.context.view_layer
    view.use_pass_material_index = True
    view.use_pass_object_index = True
    for obj in scene.objects:
        if obj.type == 'MESH':
            obj.pass_index = int(obj.get('anatomical_id', 10))
    previous = scene.node_tree
    # Preserve the complete beauty compositor across the numeric-only render.
    saved = [(node, node.mute) for node in previous.nodes]
    composite = next(node for node in previous.nodes if node.type == 'COMPOSITE')
    original_input = composite.inputs['Image'].links[0].from_socket
    render = next(node for node in previous.nodes if node.type == 'R_LAYERS')
    added = []
    for socket, suffix in (('IndexMA', '_material'), ('IndexOB', '_anatomy')):
        divide = previous.nodes.new('CompositorNodeMath')
        divide.operation = 'DIVIDE'
        divide.inputs[1].default_value = 255
        output = previous.nodes.new('CompositorNodeOutputFile')
        output.base_path = str(prefix.parent)
        output.file_slots[0].path = prefix.name + suffix + '_'
        output.format.file_format = 'PNG'
        output.format.color_mode = 'BW'
        output.format.color_depth = '8'
        previous.links.new(render.outputs[socket], divide.inputs[0])
        previous.links.new(divide.outputs[0], output.inputs[0])
        added.extend((divide, output))
    scene.view_settings.view_transform = 'Raw'
    scene.render.use_freestyle = False
    bpy.ops.render.render()
    for node in added:
        previous.nodes.remove(node)
    previous.links.new(original_input, composite.inputs['Image'])
    for node, muted in saved:
        node.mute = muted
    scene.view_settings.view_transform = 'Standard'
    scene.render.use_freestyle = True
    for suffix in ('_material', '_anatomy'):
        files = sorted(prefix.parent.glob(prefix.name + suffix + '_*.png'))
        if len(files) != 1:
            raise RuntimeError(f'Expected one native index pass for {prefix}{suffix}: {files}')
        files[0].rename(prefix.with_name(prefix.name + suffix + '.png'))
