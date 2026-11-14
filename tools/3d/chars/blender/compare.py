"""Native-pixel A/B/placeholder comparison at the M3D-1 game camera.

blender -b --factory-startup -P tools/3d/chars/blender/compare.py -- A.glb B.glb PLACEHOLDER.glb SUBJECT OUTPUT
All three inputs remain read-only. Full viewport renders are retained alongside crops.
"""
import hashlib
import json
import math
import sys
from pathlib import Path

import bpy
from bpy_extras.object_utils import world_to_camera_view
from mathutils import Vector

PARAMS = Path("/Users/rexxa/work/new-world-3d/assets/3d/params3d.json")


def main() -> None:
    """Render identical camera/light conditions without resizing comparison pixels."""
    a, b, placeholder, subject, destination = sys.argv[sys.argv.index("--")+1:]
    out = Path(destination).resolve()
    assert not out.exists(), "Comparison output directory must be new"
    out.mkdir(parents=True)
    params = json.loads(PARAMS.read_text())
    camera_params = params["camera"]
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 12
    scene.render.resolution_x, scene.render.resolution_y = 1920, 1080
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.world = bpy.data.worlds.new("Game comparison ambient")
    scene.world.use_nodes = True
    background = scene.world.node_tree.nodes.get("Background")
    background.inputs["Color"].default_value = (.70, .76, .86, 1)
    background.inputs["Strength"].default_value = .32
    sun = bpy.data.objects.new("Sun", bpy.data.lights.new("Sun", "SUN"))
    scene.collection.objects.link(sun)
    sun.data.energy = 1.5
    sun.rotation_euler = (math.radians(25), math.radians(-30), math.radians(-25))
    camera = bpy.data.objects.new("Game camera", bpy.data.cameras.new("Game camera"))
    scene.collection.objects.link(camera)
    scene.camera = camera
    camera.data.sensor_fit = "VERTICAL"
    camera.data.angle = math.radians(camera_params["fov_deg"])
    camera.data.clip_end = 900
    pitch = math.radians(camera_params["pitch_deg"])
    bpy.ops.mesh.primitive_plane_add(size=200)
    floor = bpy.context.object
    floor.name = "Comparison ground"
    ground = bpy.data.materials.new("Ground")
    ground.diffuse_color = (.34, .47, .19, 1)
    ground.use_nodes = True
    ground.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = ground.diffuse_color
    floor.data.materials.append(ground)
    # Captions occupy a separate strip: no text is resampled with the native game pixels.
    caption = bpy.data.objects.new("Caption", bpy.data.curves.new("Caption", "FONT"))
    scene.collection.objects.link(caption)
    caption.parent = camera
    caption.location = (-.40, -.25, -1)
    caption.data.size = .011
    ink = bpy.data.materials.new("Caption ink")
    ink.use_nodes = True
    ink.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = (.01, .01, .01, 1)
    caption.data.materials.append(ink)
    models = []
    input_receipts = []
    for name, path in (("A", a), ("B", b), ("M3D-1 placeholder", placeholder)):
        before = set(scene.objects)
        source = Path(path).resolve()
        bpy.ops.import_scene.gltf(filepath=str(source))
        objects = set(scene.objects)-before
        for obj in objects:
            if obj.name.startswith("CarryLoad"):
                obj.animation_data_clear()
                obj.scale = (0, 0, 0)
            if obj.type == "ARMATURE":
                obj.animation_data_clear()
                for bone in obj.pose.bones:
                    bone.matrix_basis.identity()
            obj.hide_render = True
            if subject == "child" and name == "M3D-1 placeholder" and obj.parent not in objects:
                obj.scale *= params["people"]["child_scale"]
        models.append((name, objects))
        input_receipts.append({"label": name, "path": str(source), "sha256": hashlib.sha256(source.read_bytes()).hexdigest()})
    cells = []
    measures = []
    for zoom in (1, 2):
        distance = 1080/(2*camera_params["px_per_tile_at_1x"]*zoom*math.tan(camera.data.angle/2))
        camera.location = (0, -math.cos(pitch)*distance, math.sin(pitch)*distance)
        camera.rotation_euler = (-camera.location).to_track_quat("-Z", "Y").to_euler()
        bpy.context.view_layer.update()
        origin = world_to_camera_view(scene, camera, Vector((0, 0, 0)))
        metre = world_to_camera_view(scene, camera, Vector((1, 0, 0)))
        projected = (metre.x-origin.x)*1920
        assert abs(projected-64*zoom) < .01, (zoom, projected)
        measures.append({"zoom": zoom, "distance_m": distance, "projected_ground_metre_px": projected})
        scene.render.filepath = str(out/f"ground_{zoom}x.png")
        caption.hide_render = True
        bpy.context.view_layer.update()
        bpy.ops.render.render(write_still=True)
        background_pixels = list(bpy.data.images.load(scene.render.filepath, check_existing=False).pixels[:])
        for label, objects in models:
            for obj in objects:
                obj.hide_render = False
            caption.hide_render = True
            bpy.context.view_layer.update()
            path = out/f"{label.split()[0]}_{zoom}x_viewport.png"
            scene.render.filepath = str(path)
            bpy.ops.render.render(write_still=True)
            image = bpy.data.images.load(str(path), check_existing=False)
            data = list(image.pixels[:])
            changed = sum(max(abs(data[i+c]-background_pixels[i+c]) for c in range(3)) > .08
                          for i in range(0, len(data), 4))
            assert changed > 20, f"Blank model render: {label}/{zoom}x ({changed} foreground pixels)"
            for obj in objects:
                if obj.type != "MESH" or any(collection.hide_render for collection in obj.users_collection):
                    continue
                evaluated = obj.evaluated_get(bpy.context.evaluated_depsgraph_get())
                for vertex in evaluated.data.vertices:
                    screen = world_to_camera_view(scene, camera, evaluated.matrix_world @ vertex.co)
                    assert 704 < screen.x*1920 < 1216 and 360 < screen.y*1080 < 872, (label, zoom, "crop clips model")
            crop = []
            # Fixed crop centred on the world origin: feet at row180, room for2x animal widths.
            for y in range(360, 872):
                crop.extend(data[(y*1920+704)*4:(y*1920+1216)*4])
            cell = bpy.data.images.new(f"{label}_{zoom}x", width=512, height=512)
            cell.pixels[:] = crop
            cell.filepath_raw = str(out/f"{label.split()[0]}_{zoom}x.png")
            cell.file_format = "PNG"
            cell.save()
            cells.append(cell)
            for obj in objects:
                obj.hide_render = True
    sheet = bpy.data.images.new("A_B_placeholder", width=1536, height=1104)
    pixels = [.18, .18, .18, 1]*(1536*1104)
    for index, cell in enumerate(cells):
        x, y = (index % 3)*512, (1-index//3)*552+40
        source = list(cell.pixels[:])
        for row in range(512):
            start = ((y+row)*1536+x)*4
            pixels[start:start+2048] = source[row*2048:(row+1)*2048]
    # Use Blender's vector font once per cell to create an independently rendered caption strip.
    floor.hide_render = True
    caption.hide_render = False
    for index, (zoom, label) in enumerate((z, name) for z in (1, 2) for name, _ in models):
        caption.data.body = f"{subject} / {label} / {zoom}x"
        scene.render.filepath = str(out/f"caption_{index}.png")
        bpy.ops.render.render(write_still=True)
        rendered = bpy.data.images.load(scene.render.filepath, check_existing=False)
        source = list(rendered.pixels[:])
        x, y = (index % 3)*512, (1-index//3)*552
        for row in range(40):
            start = ((y+row)*1536+x)*4
            pixels[start:start+2048] = source[((row+26)*1920+152)*4:((row+26)*1920+664)*4]
    sheet.pixels[:] = pixels
    sheet.filepath_raw = str(out/"comparison_sheet.png")
    sheet.file_format = "PNG"
    sheet.save()
    receipt = {"camera_params": str(PARAMS), "params_sha256": hashlib.sha256(PARAMS.read_bytes()).hexdigest(),
               "viewport": [1920, 1080], "crop": [704, 360, 512, 512], "measurements": measures,
               "inputs": input_receipts, "columns": ["A", "B", "M3D-1 placeholder"], "rows": ["1x", "2x"],
               "child_placeholder_scale": params["people"]["child_scale"] if subject == "child" else 1,
               "qualification": "Camera projection matched; Blender lighting approximates Godot, not renderer parity"}
    (out/"comparison_receipt.json").write_text(json.dumps(receipt, indent=2)+"\n")
    print("M3D2_COMPARISON_DONE", subject)


if __name__ == "__main__":
    main()
