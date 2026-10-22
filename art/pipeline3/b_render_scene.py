# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by Blender's render_b.py.
"""Blender scene setup for the three preregistered ink looks."""
import json
import math
from pathlib import Path
from typing import Final

import bpy
from mathutils import Vector

PALETTE: Final = Path(__file__).resolve().parents[1] / 'style05/palette_05.json'


def linear(value: float) -> float:
    """Convert display sRGB to Blender scene-linear color."""
    return value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4


def configure_materials(look: int) -> None:
    """Use normal-derived constant bands, without quantizing rendered pixels."""
    palette = json.loads(PALETTE.read_text())['rgb']
    swatches = {1: (21, 14, 4), 2: (2, 3, 7), 3: (14, 0, 6),
                4: (11, 8, 5), 5: (22, 23, 2)}
    for material in bpy.data.materials:
        if material.pass_index not in swatches:
            continue
        material.use_nodes = True
        nodes = material.node_tree.nodes
        links = material.node_tree.links
        nodes.clear()
        output = nodes.new('ShaderNodeOutputMaterial')
        if look == 1:
            shader = nodes.new('ShaderNodeBsdfDiffuse')
            shader.inputs['Color'].default_value = (0.46, 0.43, 0.38, 1)
            links.new(shader.outputs[0], output.inputs['Surface'])
            continue
        geometry = nodes.new('ShaderNodeNewGeometry')
        dot = nodes.new('ShaderNodeVectorMath')
        dot.operation = 'DOT_PRODUCT'
        dot.inputs[1].default_value = (-0.45, -0.55, 0.70)
        ramp = nodes.new('ShaderNodeValToRGB')
        ramp.color_ramp.interpolation = 'CONSTANT'
        ramp.color_ramp.elements.new(0.3)
        for element, index, position in zip(ramp.color_ramp.elements,
                                           reversed(swatches[material.pass_index]),
                                           (0.0, 0.3, 0.65), strict=True):
            element.position = position
            element.color = (*[linear(c / 255) for c in palette[index]], 1)
        emission = nodes.new('ShaderNodeEmission')
        links.new(geometry.outputs['Normal'], dot.inputs[0])
        links.new(dot.outputs['Value'], ramp.inputs[0])
        links.new(ramp.outputs['Color'], emission.inputs['Color'])
        links.new(emission.outputs[0], output.inputs['Surface'])


def configure_scene(look: int) -> None:
    """Set native Freestyle strokes and optional compositor paper grain."""
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = 12
    scene.cycles.use_denoising = False
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'
    scene.render.image_settings.color_depth = '8'
    scene.render.resolution_percentage = 100
    scene.view_settings.view_transform = 'Standard'
    scene.view_settings.look = 'None'
    scene.view_settings.exposure = 0
    scene.view_settings.gamma = 1
    scene.render.use_freestyle = True
    scene.render.line_thickness = 1
    fs = bpy.context.view_layer.freestyle_settings
    fs.mode = 'EDITOR'
    fs.crease_angle = math.radians(125)
    fs.use_material_boundaries = True
    line = fs.linesets[0] if len(fs.linesets) else fs.linesets.new('Ink')
    if line.linestyle is None:
        line.linestyle = bpy.data.linestyles.new('B_Ink')
    line.select_silhouette = True
    line.select_border = True
    line.select_crease = True
    line.select_material_boundary = True
    line.linestyle.color = tuple(linear(c / 255) for c in (20, 18, 16))
    line.linestyle.thickness = 2
    if scene.world is None:
        scene.world = bpy.data.worlds.new('B_World')
    scene.world.color = (0.35, 0.35, 0.35)
    light = bpy.data.lights.new('B_Key', 'AREA')
    light.energy = 250
    obj = bpy.data.objects.new('B_Key', light)
    scene.collection.objects.link(obj)
    obj.location = (-3, -4, 6)
    obj.rotation_euler = (-obj.location).to_track_quat('-Z', 'Y').to_euler()
    configure_materials(look)
    scene.use_nodes = True
    nodes = scene.node_tree.nodes
    nodes.clear()
    render = nodes.new('CompositorNodeRLayers')
    output = nodes.new('CompositorNodeComposite')
    image = render.outputs['Image']
    if look == 3:
        texture = bpy.data.textures.new('B_Paper_8percent', type='CLOUDS')
        texture.noise_scale = 0.015
        texture.noise_depth = 1
        paper = nodes.new('CompositorNodeTexture')
        paper.texture = texture
        mix = nodes.new('CompositorNodeMixRGB')
        mix.blend_type = 'MULTIPLY'
        mix.inputs[0].default_value = 0.08
        scene.node_tree.links.new(image, mix.inputs[1])
        scene.node_tree.links.new(paper.outputs['Value'], mix.inputs[2])
        alpha = nodes.new('CompositorNodeSetAlpha')
        scene.node_tree.links.new(mix.outputs[0], alpha.inputs['Image'])
        scene.node_tree.links.new(render.outputs['Alpha'], alpha.inputs['Alpha'])
        image = alpha.outputs[0]
    scene.node_tree.links.new(image, output.inputs['Image'])


def camera_for(direction: str, sleeping: bool) -> None:
    """Frame projected body geometry at 56 pixels and upright foot edge y88."""
    scene = bpy.context.scene
    width, height = (96, 64) if sleeping else (64, 96)
    scene.render.resolution_x, scene.render.resolution_y = width * 4, height * 4
    camera = scene.camera
    if camera is None:
        camera = bpy.data.objects.new('B_Camera', bpy.data.cameras.new('B_Camera'))
        scene.collection.objects.link(camera)
        scene.camera = camera
    angle = {'S': 0, 'E': -math.pi / 2, 'N': math.pi}[direction]
    offset = Vector((6 * math.sin(angle), -6 * math.cos(angle), 6 * math.tan(math.radians(20))))
    camera.rotation_euler = (-offset).to_track_quat('-Z', 'Y').to_euler()
    rotation = camera.rotation_euler.to_matrix()
    right, up = rotation @ Vector((1, 0, 0)), rotation @ Vector((0, 1, 0))
    points = []
    depsgraph = bpy.context.evaluated_depsgraph_get()
    for obj in scene.objects:
        if obj.type == 'MESH' and obj.get('layer') == 'BODY':
            evaluated = obj.evaluated_get(depsgraph)
            mesh = evaluated.to_mesh()
            points.extend(evaluated.matrix_world @ v.co for v in mesh.vertices)
            evaluated.to_mesh_clear()
    xs, ys = [p.dot(right) for p in points], [p.dot(up) for p in points]
    camera.data.type = 'ORTHO'
    if sleeping:
        # Blender ortho_scale follows the longer image dimension.
        camera.data.ortho_scale = (max(xs) - min(xs)) * width / 56
        scale_y = camera.data.ortho_scale * height / width
        center_y = (min(ys) + max(ys)) / 2 + (36 - height / 2) * scale_y / height
    else:
        camera.data.ortho_scale = (max(ys) - min(ys)) * height / 56
        center_y = min(ys) + (88 - height / 2) * camera.data.ortho_scale / height
    center = right * ((min(xs) + max(xs)) / 2) + up * center_y
    camera.location = center + offset
    bpy.context.view_layer.update()
