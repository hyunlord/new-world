"""Clean new TRELLIS species with set-B atlas, quadruped skeleton and clips."""
import hashlib
import json
import sys
from pathlib import Path

import bmesh
import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
import prepare
from motion import clips
from rig import skeleton
from weights import fit, heat

HEIGHTS = {"wolf": .8, "deer": 1.1, "wild_sheep": .75, "wild_boar": .9, "reindeer": 1.1}


def ground_fragments(mesh):
    """Remove verified disconnected remesh slivers below the belly, retaining the body."""
    topology = bmesh.new()
    topology.from_mesh(mesh.data)
    pending = set(topology.verts)
    height = max(v.co.z for v in pending)
    discard = []
    while pending:
        component = {pending.pop()}
        queue = list(component)
        while queue:
            for edge in queue.pop().link_edges:
                for vertex in edge.verts:
                    if vertex in pending:
                        pending.remove(vertex)
                        component.add(vertex)
                        queue.append(vertex)
        extent = max(max(v.co[i] for v in component)-min(v.co[i] for v in component) for i in range(3))
        if len(component) <= 8 and max(v.co.z for v in component) < .3*height and extent < .3*height:
            discard.extend(component)
    bmesh.ops.delete(topology, geom=discard, context="VERTS")
    topology.to_mesh(mesh.data)
    topology.free()
    return len(discard)


def main():
    source, destination, species = sys.argv[sys.argv.index("--")+1:]
    height = HEIGHTS[species]
    out = Path(destination).resolve()
    assert not out.exists(), "Preserve previous attempts"
    out.mkdir(parents=True)
    # Reuse the pipeline's quadruped cleanup category, with species shoulder height.
    prepare.HEIGHTS["goat"] = height
    mesh = prepare.cleanup(Path(source).resolve(), "goat", True, True)
    mesh.name = species
    removed = ground_fragments(mesh)
    prepare.atlas(mesh, out)
    arm = skeleton(height, True)
    fit(arm, height)
    heat(mesh, arm)
    names = clips(arm, mesh)
    assert len(arm.data.bones) == 12 and names == ["idle", "walk", "graze", "lie_down"]
    for obj in (mesh, arm):
        obj.select_set(True)
    arm.animation_data.action = None
    for track in arm.animation_data.nla_tracks:
        track.mute = False
    bpy.ops.export_scene.gltf(filepath=str(out/f"{species}.glb"), export_format="GLB", use_selection=True,
        export_animation_mode="NLA_TRACKS", export_force_sampling=True, export_optimize_animation_size=False,
        export_optimize_animation_keep_anim_object=True)
    receipt = {"species": species, "ground_fragment_vertices_removed": removed,
        "raw_sha256": hashlib.sha256(Path(source).read_bytes()).hexdigest(), "seed": 0,
        "trellis_revision": "75fbf0183001ed9876c8dbb35de6b68552ee08bd", "source": str(Path(source).resolve()),
        "triangles": sum(len(p.vertices)-2 for p in mesh.data.polygons),
        "shoulder_height_m": height, "height_landmark_xyz": list(mesh["height_landmark_xyz"]),
        "bones": [b.name for b in arm.data.bones], "animations": names,
        "texture": "512x512 embedded atlas", "pipeline": "TRELLIS.2 -> set-B remesh/atlas -> fitted shared quadruped -> shared clips",
        "front": "Godot +Z / Blender -Y"}
    assert 1000 <= receipt["triangles"] <= 2500
    (out/"receipt.json").write_text(json.dumps(receipt, indent=2)+"\n")
    print("M3D_EXTEND_ANIMAL_PASS", species)


if __name__ == "__main__":
    main()
