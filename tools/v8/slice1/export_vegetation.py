"""Run once per Poly Haven source blend with Blender --background.

Pass the destination GLB after --. Keep the original cutout UVs and sample whole
broadleaf cards, since collapsing their triangles erases the leaf silhouettes.
"""

from pathlib import Path
import shutil
import sys

import bpy

SOURCES = {
    "tree_small_02": ("tree_small_02_LOD1", 5.1),
    "fir_sapling_medium": ("fir_sapling_medium_a_LOD1", 5.7),
    "fern_02": ("fern_02_b", 0.55),
}


def card_roots(mesh):
    parents = list(range(len(mesh.vertices)))

    def root(vertex):
        while vertex != parents[vertex]:
            vertex, parents[vertex] = parents[vertex], parents[parents[vertex]]
        return vertex

    for polygon in mesh.polygons:
        if polygon.material_index == 2:
            continue
        first, *others = polygon.vertices
        head = root(first)
        for vertex in others:
            parents[root(vertex)] = head
    return root


def build_mesh(source, model, target_height):
    mesh = source.data
    root = card_roots(mesh) if model == "tree_small_02" else None
    bottom = min(vertex.co.z for vertex in mesh.vertices)
    height = max(vertex.co.z for vertex in mesh.vertices) - bottom
    scale = target_height / height
    vertices, polygons, uvs, slots = [], [], [], []
    vert_indices = {}
    for polygon in mesh.polygons:
        slot = polygon.material_index
        if root is not None and slot != 2:
            key = root(polygon.vertices[0])
            # Broadleaf is a film mesh. Keep complete branch cards and leaves
            # distributed across the crown using a stable hash of each island.
            frequency = 5 if slot == 0 else 13
            if ((key * 2654435761) & 0xFFFFFFFF) % frequency != 0:
                continue
        if model == "fir_sapling_medium":
            coords = mesh.attributes["UVMap"].data
            uv_scale = ((1.2, 0.1), (1.0, 1.0), (0.8, 0.8))[slot]
            points = [coords[i].vector for i in polygon.loop_indices]
        else:
            layer = "UV_map_01" if model == "tree_small_02" and slot == 0 else "UVMap"
            coords = mesh.uv_layers[layer].uv
            uv_scale = (3.0, 0.6) if layer == "UV_map_01" else (1.0, 1.0)
            points = [coords[i].vector for i in polygon.loop_indices]
        uvs.extend((point[0] * uv_scale[0], point[1] * uv_scale[1]) for point in points)
        indices = []
        for original in polygon.vertices:
            if original not in vert_indices:
                co = mesh.vertices[original].co
                vert_indices[original] = len(vertices)
                vertices.append((co.x * scale, co.y * scale, (co.z - bottom) * scale))
            indices.append(vert_indices[original])
        polygons.append(indices)
        slots.append(slot)

    result = bpy.data.meshes.new(model)
    result.from_pydata(vertices, [], polygons)
    result.update()
    uv = result.uv_layers.new(name="UVMap")
    uv.uv.foreach_set("vector", [value for point in uvs for value in point])
    for material in mesh.materials:
        # Keep original names as surface keys, but let Godot own the shader.
        result.materials.append(bpy.data.materials.new(material.name + "_slice"))
    for polygon, slot in zip(result.polygons, slots, strict=True):
        polygon.material_index = slot
        polygon.use_smooth = True
    result.calc_loop_triangles()
    assert 0 < len(result.loop_triangles) < 120_000, (len(result.loop_triangles), [slots.count(slot) for slot in set(slots)])
    assert all(slot in slots for slot in range(len(mesh.materials)))
    print(f"{model}: {len(result.loop_triangles):,} triangles, {target_height:g} tiles tall")
    return result


def main():
    dest = Path(sys.argv[sys.argv.index("--") + 1])
    source_path = Path(bpy.data.filepath)
    model = source_path.stem.removesuffix("_1k")
    name, height = SOURCES[model]
    mesh = build_mesh(bpy.data.objects[name], model, height)
    scene_object = bpy.data.objects.new(model, mesh)
    bpy.context.scene.collection.objects.link(scene_object)
    bpy.ops.object.select_all(action="DESELECT")
    scene_object.select_set(True)
    bpy.context.view_layer.objects.active = scene_object
    dest.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(dest),
        use_selection=True,
        export_format="GLB",
        export_tangents=True,
        export_animations=False,
    )
    assert dest.is_file() and dest.stat().st_size > 0
    texture_dir = dest.parent / "textures"
    texture_dir.mkdir(exist_ok=True)
    for image in (source_path.parent / "textures").iterdir():
        if image.stem.endswith(("_disp_1k", "_mask_1k")):
            continue
        if image.suffix == ".exr":
            # Godot cannot read the source DWA-compressed normal maps.
            data = bpy.data.images.load(str(image), check_existing=False)
            data.colorspace_settings.name = "Non-Color"
            assert data.size[0] > 0, image
            data.file_format = "PNG"
            data.save(filepath=str(texture_dir / image.with_suffix(".png").name), save_copy=True)
        else:
            shutil.copyfile(image, texture_dir / image.name)


if __name__ == "__main__":
    main()
