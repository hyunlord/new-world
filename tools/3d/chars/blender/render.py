"""Render actual textured meshes, four views and every animation into a contact sheet."""
import json
import math
from pathlib import Path

import bpy
from mathutils import Vector


def capture(mesh: bpy.types.Object, arm: bpy.types.Object, out: Path) -> None:
    """Render 30-degree FOV, 50-degree pitch; game-size references are 32/64 px."""
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 8
    scene.render.resolution_x = scene.render.resolution_y = 256
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.world = bpy.data.worlds.new("Grey studio")
    scene.world.use_nodes = True
    scene.world.node_tree.nodes.get("Background").inputs[0].default_value = (.65, .65, .65, 1)
    scene.world.node_tree.nodes.get("Background").inputs[1].default_value = .8
    light = bpy.data.objects.new("Softbox", bpy.data.lights.new("Softbox", "AREA"))
    scene.collection.objects.link(light)
    light.location = (3, -4, 6)
    light.data.energy, light.data.size = 500, 5
    camera = bpy.data.objects.new("Camera", bpy.data.cameras.new("Camera"))
    scene.collection.objects.link(camera)
    scene.camera = camera
    camera.data.angle = math.radians(30)
    label = bpy.data.objects.new("Caption", bpy.data.curves.new("Caption", "FONT"))
    scene.collection.objects.link(label)
    label.parent = camera
    label.location = (-.25, -.245, -1)
    label.data.size = .022
    ink = bpy.data.materials.new("Caption ink")
    ink.use_nodes = True
    shader = ink.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = (.015, .015, .015, 1)
    label.data.materials.append(ink)
    height = max(v.co.z for v in mesh.data.vertices)
    centre = Vector((0, 0, height*.5))
    distance = max(height*3.2, 3)
    paths = []
    actions = [(track.name, strip.action) for track in arm.animation_data.nla_tracks for strip in track.strips]
    views = [(f"view_{angle}", angle, None) for angle in (0, 90, 180, 270)]
    views += [(name, 35, action) for name, action in actions]
    for name, angle, action in views:
        label.data.body = name
        arm.animation_data.action = action
        if action is None:
            for bone in arm.pose.bones:
                bone.rotation_euler = (0, 0, 0)
                bone.location = (0, 0, 0)
        scene.frame_set(8)
        yaw, pitch = math.radians(angle), math.radians(50)
        offset = Vector((math.sin(yaw)*math.cos(pitch), -math.cos(yaw)*math.cos(pitch), math.sin(pitch)))
        camera.location = centre+offset*distance
        camera.rotation_euler = (-offset).to_track_quat("-Z", "Y").to_euler()
        path = out/f"{name}.png"
        scene.render.filepath = str(path)
        bpy.ops.render.render(write_still=True)
        paths.append(path)
        evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
        assert all(math.isfinite(c) for vertex in evaluated.data.vertices for c in vertex.co), name
    arm.animation_data.action = None
    # Blender image buffers avoid a Pillow dependency in Blender's embedded Python.
    images = [bpy.data.images.load(str(path), check_existing=False) for path in paths]
    for size in (32, 64):
        image = images[0].copy()
        image.scale(size, size)
        image.filepath_raw = str(out/f"game_{size}px.png")
        image.file_format = "PNG"
        image.save()
        label.data.body = f"game {size}px ({size//32}x)"
        mesh.hide_render = True
        scene.render.filepath = str(out/f"game_{size}px_cell.png")
        bpy.ops.render.render(write_still=True)
        mesh.hide_render = False
        cell = bpy.data.images.load(scene.render.filepath, check_existing=False)
        pixels = list(cell.pixels[:])
        source = list(image.pixels[:])
        for row in range(size):
            start = (((256-size)//2+row)*256+(256-size)//2)*4
            pixels[start:start+size*4] = source[row*size*4:(row+1)*size*4]
        cell.pixels[:] = pixels
        cell.save()
        images.append(cell)
        paths.append(Path(scene.render.filepath))
    placeholder = out/"placeholder.glb"
    if placeholder.is_file():
        before = set(scene.objects)
        bpy.ops.import_scene.gltf(filepath=str(placeholder))
        imported = set(scene.objects)-before
        for obj in imported:
            if obj.parent not in imported:
                obj.scale *= height/1.72
        mesh.hide_render = True
        label.data.body = "M3D-1 placeholder (height matched)"
        pitch = math.radians(50)
        offset = Vector((0, -math.cos(pitch), math.sin(pitch)))
        camera.location = centre+offset*distance
        camera.rotation_euler = (-offset).to_track_quat("-Z", "Y").to_euler()
        path = out/"placeholder_comparison.png"
        scene.render.filepath = str(path)
        bpy.ops.render.render(write_still=True)
        images.append(bpy.data.images.load(str(path), check_existing=False))
        paths.append(path)
        for obj in imported:
            obj.hide_render = True
        mesh.hide_render = False
    cols = 4
    rows = math.ceil(len(images)/cols)
    sheet = bpy.data.images.new("contact_sheet", width=cols*256, height=rows*256, alpha=True)
    pixels = [.85, .85, .85, 1]*(cols*rows*256*256)
    for index, image in enumerate(images):
        source = list(image.pixels[:])
        x, y = (index % cols)*256, (rows-1-index//cols)*256
        for row in range(256):
            start = ((y+row)*cols*256+x)*4
            pixels[start:start+256*4] = source[row*256*4:(row+1)*256*4]
    sheet.pixels[:] = pixels
    sheet.filepath_raw = str(out/"contact_sheet.png")
    sheet.file_format = "PNG"
    sheet.save()
    (out/"contact_sheet_layout.json").write_text(json.dumps({"columns": cols, "cell_px": 256,
        "rows_top_to_bottom": [[path.stem for path in paths[i:i+cols]] for i in range(0, len(paths), cols)]}, indent=2)+"\n")
