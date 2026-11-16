"""Check exported living GLBs, their shared skins and all sampled clips."""
import json
import math
import struct
import sys
from pathlib import Path

import bpy

ROOT = Path(__file__).resolve().parents[4]


def main():
    paths = [Path(p) for p in sys.argv[sys.argv.index("--")+1:]] if "--" in sys.argv else sorted((ROOT/"assets/3d/living").rglob("*.glb"))
    assert paths, "No living models selected"
    receipts = []
    templates = {}
    for path in paths:
        binary = path.read_bytes()
        assert binary[:4] == b"glTF"
        document = json.loads(binary[20:20+struct.unpack_from("<I", binary, 12)[0]])
        colours = {"bronze_shoulder_cloth": (.30, .43, .38, 1),
                   "iron_shoulder_cloth": (.40, .22, .16, 1), "elder_silver_hair": (.65, .63, .58, 1)}
        found = set()
        for material in document["materials"]:
            name = material.get("name", "").split(".")[0]
            if name in colours:
                actual = material["pbrMetallicRoughness"]["baseColorFactor"]
                assert max(abs(a-b) for a, b in zip(actual, colours[name])) < 1e-6, (path, name, actual)
                found.add(name)
        if path.parent.name == "people":
            if "elder" in path.stem:
                assert "elder_silver_hair" in found
            for era in ("bronze", "iron"):
                if era in path.stem:
                    assert era+"_shoulder_cloth" in found
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete(use_global=False)
        bpy.ops.import_scene.gltf(filepath=str(path.resolve()))
        objects = list(bpy.context.scene.objects)
        arm = next(o for o in objects if o.type == "ARMATURE")
        meshes = [o for o in objects if o.type == "MESH" and not any(c.name == "glTF_not_exported" for c in o.users_collection)]
        quad = "Root" in arm.data.bones
        expected = {"idle", "walk", "graze", "lie_down"} if quad else {"idle", "walk", "work", "carry", "sleep"}
        names = {t.name for t in arm.animation_data.nla_tracks}
        assert names == expected, (path, names)
        assert len(arm.data.bones) == (12 if quad else 17)
        scale = arm.data.bones["Root"].head_local.z/.65 if quad else arm.data.bones["head"].tail_local.z
        rest = {b.name: (b.parent.name if b.parent else None,
                tuple(b.head_local/scale), tuple(b.tail_local/scale), tuple(b.matrix_local.to_quaternion())) for b in arm.data.bones}
        if quad in templates:
            reference = templates[quad]
            assert rest.keys() == reference.keys()
            for key, (parent, head, tail, rotation) in rest.items():
                assert parent == reference[key][0]
                assert max(abs(a-b) for a, b in zip(head+tail, reference[key][1]+reference[key][2])) < 2e-5, (path, key, "shared rest")
                # q and -q are the same rotation; compare orientation, not component signs.
                assert 1-abs(sum(a*b for a, b in zip(rotation, reference[key][3]))) < 1e-5, (path, key, "shared rotation")
        else:
            templates[quad] = rest
        triangles = sum(len(p.vertices)-2 for o in meshes for p in o.data.polygons)
        assert 1000 <= triangles <= (2500 if quad else 3000), (path, triangles)
        for mesh in meshes:
            if not any(m.type == "ARMATURE" for m in mesh.modifiers):
                continue
            for vertex in mesh.data.vertices:
                values = [g.weight for g in vertex.groups if g.weight > 1e-6]
                assert 1 <= len(values) <= 3 and abs(sum(values)-1) < 1e-5, (path, vertex.index, values)
        for name in sorted(names):
            for obj in objects:
                if obj.animation_data:
                    obj.animation_data.action = None
                    for track in obj.animation_data.nla_tracks:
                        track.mute = track.name != name
            poses = []
            body = max(meshes, key=lambda o: len(o.data.vertices))
            for frame in (1, 5, 9, 13, 17, 21, 25, 29, 33):
                bpy.context.scene.frame_set(frame)
                bpy.context.view_layer.update()
                for mesh in meshes:
                    evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
                    assert all(math.isfinite(c) for vertex in evaluated.data.vertices for c in vertex.co), (path, name, frame)
                evaluated = body.evaluated_get(bpy.context.evaluated_depsgraph_get())
                points = [evaluated.matrix_world @ v.co for v in evaluated.data.vertices]
                assert abs(min(point.z for point in points)) < .003, (path, name, frame, "ground contact")
                poses.append(points)
            assert max((a-b).length for a, b in zip(poses[0], poses[-1])) < 1e-4, (path, name, "loop seam")
        receipts.append({"file": str(path.relative_to(ROOT)), "triangles": triangles,
                         "bones": len(arm.data.bones), "clips": sorted(names), "samples_per_clip": 9})
    destination = Path('/Users/rexxa/work/3d-data/m3d-assets/living/validation.json')
    destination.write_text(json.dumps(receipts, indent=2)+"\n")
    print("M3D_LIVING_VALIDATION_PASS", len(receipts))


if __name__ == "__main__":
    main()
