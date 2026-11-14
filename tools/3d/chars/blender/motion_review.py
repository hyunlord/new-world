"""Review eight samples per exported clip at native game2x and close range.

blender -b --factory-startup -P motion_review.py -- INPUT.glb NEW_OUTPUT
The square game camera is an exact central crop of the 1080px vertical-FOV view.
"""
import hashlib
import json
import math
import sys
from pathlib import Path

import bpy
import numpy as np
from bpy_extras.object_utils import world_to_camera_view
from mathutils import Vector


def select_clip(objects: list[bpy.types.Object], name: str) -> None:
    """Activate the corresponding exported NLA track on every animated object."""
    for obj in objects:
        data = obj.animation_data
        if data is None:
            continue
        data.action = None
        data.use_nla = True
        for track in data.nla_tracks:
            track.mute = track.name != name


def pixels(path: Path) -> np.ndarray:
    image = bpy.data.images.load(str(path), check_existing=False)
    values = np.empty(len(image.pixels), dtype=np.float32)
    image.pixels.foreach_get(values)
    result = values.reshape(image.size[1], image.size[0], 4).copy()
    bpy.data.images.remove(image)
    return result


def save_pixels(path: Path, values: np.ndarray) -> None:
    image = bpy.data.images.new(path.stem, width=values.shape[1], height=values.shape[0])
    image.pixels.foreach_set(values.ravel())
    image.filepath_raw = str(path)
    image.file_format = "PNG"
    image.save()
    bpy.data.images.remove(image)


def point_camera(camera: bpy.types.Object, centre: Vector, offset: Vector) -> None:
    camera.location = centre + offset
    camera.rotation_euler = (-offset).to_track_quat("-Z", "Y").to_euler()
    bpy.context.view_layer.update()


def main() -> None:
    source_text, output_text = sys.argv[sys.argv.index("--") + 1:]
    source, out = Path(source_text).resolve(), Path(output_text).resolve()
    assert source.is_file() and not out.exists(), "Require an input and fresh output"
    out.mkdir(parents=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    bpy.ops.import_scene.gltf(filepath=str(source))
    objects = list(bpy.context.scene.objects)
    arm = next(obj for obj in objects if obj.type == "ARMATURE")
    meshes = [obj for obj in objects if obj.type == "MESH" and not obj.name.startswith("CarryLoad")
              and not obj.hide_render and not any(collection.hide_render for collection in obj.users_collection)]
    assert meshes and all(not any(collection.name == "glTF_not_exported" for collection in obj.users_collection)
                          for obj in meshes), "Exclude importer bone-display helper geometry"
    torso_vertices, body_vertices, rest_edges = [], [], []
    for obj in meshes:
        rest = [obj.matrix_world @ vertex.co for vertex in obj.data.vertices]
        height = max(point.z for point in rest)
        torso_vertices.append([index for index, point in enumerate(rest)
                               if .42 * height < point.z < .70 * height and abs(point.x) < .16 * height])
        body_groups = {group.index for group in obj.vertex_groups if group.name in ("hips", "spine", "chest", "Root")}
        body_vertices.append([vertex.index for vertex in obj.data.vertices
                              if rest[vertex.index].z > .25 * height
                              and sum(group.weight for group in vertex.groups if group.group in body_groups) >= .5])
        rest_edges.append([(edge.vertices[0], edge.vertices[1], (rest[edge.vertices[0]] - rest[edge.vertices[1]]).length)
                           for edge in obj.data.edges if (rest[edge.vertices[0]] - rest[edge.vertices[1]]).length > .005 * height])
    clips = [(track.name, track.strips[0].frame_start, track.strips[0].frame_end)
             for track in arm.animation_data.nla_tracks]
    assert len(clips) in (4, 5), clips
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = scene.render.resolution_y = 512
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.film_transparent = False
    scene.world = bpy.data.worlds.new("Review ambient")
    scene.world.use_nodes = True
    scene.world.node_tree.nodes.get("Background").inputs[0].default_value = (.42, .47, .50, 1)
    scene.world.node_tree.nodes.get("Background").inputs[1].default_value = .8
    for name, angle, energy in (("Key", (25, -30, -25), 2.5), ("Fill", (40, 25, 145), 1.0)):
        light = bpy.data.objects.new(name, bpy.data.lights.new(name, "SUN"))
        scene.collection.objects.link(light)
        light.rotation_euler = tuple(math.radians(value) for value in angle)
        light.data.energy = energy
        light.data.angle = .2
    bpy.ops.mesh.primitive_plane_add(size=200)
    floor = bpy.context.object
    floor.name = "Review ground"
    material = bpy.data.materials.new("Review ground")
    material.diffuse_color = (.34, .47, .19, 1)
    material.use_nodes = True
    material.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = material.diffuse_color
    floor.data.materials.append(material)
    camera = bpy.data.objects.new("Review camera", bpy.data.cameras.new("Review camera"))
    scene.collection.objects.link(camera)
    scene.camera = camera
    camera.data.sensor_fit = "VERTICAL"
    camera.data.clip_end = 300
    pitch = math.radians(50)
    distance = 1080 / (256 * math.tan(math.radians(15)))
    game_offset = Vector((0, -math.cos(pitch) * distance, math.sin(pitch) * distance))
    game_fov = 2 * math.atan(math.tan(math.radians(15)) * 512 / 1080)
    camera.data.angle = game_fov
    point_camera(camera, Vector((0, 0, 0)), game_offset)
    a = world_to_camera_view(scene, camera, Vector((0, 0, 0)))
    b = world_to_camera_view(scene, camera, Vector((1, 0, 0)))
    density = (b.x - a.x) * 512
    assert abs(density - 128) < .01, density
    records = []
    for name, start, end in clips:
        select_clip(objects, name)
        bounds, ground_samples = [], []
        frames = [start + (end - start) * index / 8 for index in range(8)]
        for frame in frames:
            scene.frame_set(int(frame), subframe=frame % 1)
            bpy.context.view_layer.update()
            points = [obj.evaluated_get(bpy.context.evaluated_depsgraph_get()).matrix_world @ vertex.co
                      for obj in meshes for vertex in obj.evaluated_get(bpy.context.evaluated_depsgraph_get()).data.vertices]
            assert all(math.isfinite(value) for point in points for value in point), (name, frame)
            bounds.extend(points)
            torso_z = [
                (obj.evaluated_get(bpy.context.evaluated_depsgraph_get()).matrix_world
                 @ obj.evaluated_get(bpy.context.evaluated_depsgraph_get()).data.vertices[index].co).z
                for obj, indices in zip(meshes, torso_vertices) for index in indices]
            body_z = [(obj.evaluated_get(bpy.context.evaluated_depsgraph_get()).matrix_world
                       @ obj.evaluated_get(bpy.context.evaluated_depsgraph_get()).data.vertices[index].co).z
                      for obj, indices in zip(meshes, body_vertices) for index in indices]
            stretches = []
            for obj, edges in zip(meshes, rest_edges):
                evaluated = obj.evaluated_get(bpy.context.evaluated_depsgraph_get())
                posed = [evaluated.matrix_world @ vertex.co for vertex in evaluated.data.vertices]
                stretches.extend((posed[a] - posed[b]).length / length for a, b, length in edges)
            ground_samples.append({"frame": frame, "minimum_z_m": min(point.z for point in points),
                                   "vertices_within_1cm_ground": sum(abs(point.z) <= .01 for point in points),
                                   "central_rest_torso_minimum_z_m": min(torso_z) if torso_z else None,
                                   "body_weighted_minimum_z_m": min(body_z) if body_z else None,
                                   "maximum_edge_stretch": max(stretches),
                                   "edges_stretched_over_3x": sum(ratio > 3 for ratio in stretches)})
        low = Vector(tuple(min(point[axis] for point in bounds) for axis in range(3)))
        high = Vector(tuple(max(point[axis] for point in bounds) for axis in range(3)))
        centre = (low + high) / 2
        close_direction = Vector((.12, -.97, .20)).normalized()
        close_rotation = (-close_direction).to_track_quat("-Z", "Y")
        relative = [close_rotation.inverted() @ (point - centre) for point in bounds]
        close_distance = max(max(abs(point.x), abs(point.y)) / (math.tan(math.radians(15)) * .78) + point.z
                             for point in relative)
        game_images, close_images, shoulder_images, side_images = [], [], [], []
        for index, frame in enumerate(frames):
            scene.frame_set(int(frame), subframe=frame % 1)
            camera.data.angle = game_fov
            point_camera(camera, Vector((0, 0, 0)), game_offset)
            game_path = out / f"{name}_{index:02d}_game2x.png"
            scene.render.filepath = str(game_path)
            bpy.ops.render.render(write_still=True)
            game_images.append(pixels(game_path))
            camera.data.angle = math.radians(30)
            point_camera(camera, centre, close_direction * close_distance)
            close_path = out / f"{name}_{index:02d}_close.png"
            scene.render.filepath = str(close_path)
            bpy.ops.render.render(write_still=True)
            image = pixels(close_path)
            close_images.append(image)
            if name == "work":
                crops = []
                for side in ("L", "R"):
                    joints = [arm.matrix_world @ arm.pose.bones[f"{bone}_{side}"].head
                              for bone in ("upper_arm", "lower_arm", "hand")]
                    projected = [world_to_camera_view(scene, camera, point) for point in joints]
                    x = max(0, min(384, round(sum(point.x for point in projected) / 3 * 512) - 64))
                    y = max(0, min(384, round(sum(point.y for point in projected) / 3 * 512) - 64))
                    crop = np.repeat(np.repeat(image[y:y + 128, x:x + 128], 4, axis=0), 4, axis=1)
                    save_pixels(out / f"work_{index:02d}_{side}_shoulder4x.png", crop)
                    crops.append(crop)
                shoulder_images.append(np.concatenate(crops, axis=0))
            if name in ("work", "sleep", "lie_down", "graze"):
                side_direction = Vector((.96, -.20, .20)).normalized()
                if name == "sleep":
                    side_direction = Vector((.08, -.995, .06)).normalized()
                if name in ("lie_down", "graze"):
                    side_direction = Vector((.995, -.06, .06)).normalized()
                side_rotation = (-side_direction).to_track_quat("-Z", "Y")
                side_points = [side_rotation.inverted() @ (point - centre) for point in bounds]
                side_distance = max(max(abs(point.x), abs(point.y)) / (math.tan(math.radians(15)) * .78) + point.z
                                    for point in side_points)
                point_camera(camera, centre, side_direction * side_distance)
                side_path = out / f"{name}_{index:02d}_side.png"
                scene.render.filepath = str(side_path)
                bpy.ops.render.render(write_still=True)
                side_images.append(pixels(side_path))
            records.append({"clip": name, "sample": index, "frame": frame,
                            "game": game_path.name, "close": close_path.name})
        sheet = np.concatenate((np.concatenate(game_images, axis=1), np.concatenate(close_images, axis=1)), axis=0)
        save_pixels(out / f"{name}_review.png", sheet)
        if shoulder_images:
            save_pixels(out / "work_shoulders4x_review.png", np.concatenate(shoulder_images, axis=1))
        if side_images:
            save_pixels(out / f"{name}_side_review.png", np.concatenate(side_images, axis=1))
        (out / f"{name}_ground.json").write_text(json.dumps(ground_samples, indent=2) + "\n")
    receipt = {"input": str(source), "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
               "native_game_px_per_ground_metre": density, "game_view_crop_px": 512,
               "game_pitch_deg": 50, "original_vertical_fov_deg": 30,
               "crop_vertical_fov_deg": math.degrees(game_fov), "game_distance_m": distance,
               "rows_top_to_bottom": ["close", "native game2x"],
               "columns": "8 samples at phase index/8, indexed0..7 left to right",
               "shoulder_crop": "128px of close view around shoulder/elbow/hand centroid, nearest4x; topR bottomL",
               "central_rest_torso_mask": "Rest z42–70% crown and absX<16% crown; a diagnostic central band, not all contact skin",
               "qualification": "Numeric finite check is not a self-intersection or deformation quality verdict",
               "samples": records}
    (out / "review_receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print("M3D2B_MOTION_REVIEW_DONE", source.stem, len(records))


if __name__ == "__main__":
    main()
