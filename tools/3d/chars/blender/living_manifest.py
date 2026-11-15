"""Describe the exported skinned living GLBs for renderer integration."""
import hashlib
import json
from pathlib import Path

import bpy

ROOT = Path(__file__).resolve().parents[4]


def main():
    base = ROOT/"assets/3d"
    people = json.loads((base/"living/people_manifest.json").read_text())
    entries = {}
    for path in sorted((base/"living").rglob("*.glb")):
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete(use_global=False)
        bpy.ops.import_scene.gltf(filepath=str(path))
        meshes = [o for o in bpy.context.scene.objects if o.type == "MESH" and not any(c.name == "glTF_not_exported" for c in o.users_collection)]
        arm = next(o for o in bpy.context.scene.objects if o.type == "ARMATURE")
        body = [o for o in meshes if not o.name.startswith("CarryLoad")]
        points = [o.matrix_world@v.co for o in body for v in o.data.vertices]
        low = [min(v[i] for v in points) for i in range(3)]
        high = [max(v[i] for v in points) for i in range(3)]
        key = path.stem
        entry = people.get(key, {})
        entry.update({"file": str(path.relative_to(base)), "family": "character",
            "height": high[2]-low[2], "footprint": [high[0]-low[0], high[1]-low[1]],
            "triangles": sum(len(p.vertices)-2 for o in meshes for p in o.data.polygons),
            "materials": sorted({m.name for o in meshes for m in o.data.materials}),
            "bones": [b.name for b in arm.data.bones], "sockets": {},
            "animations": sorted(t.name for t in arm.animation_data.nla_tracks),
            "integration": "skinned_glb_animation_player; no VAT baked",
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
        if key not in people:
            receipt = json.loads((path.parent/"receipt.json").read_text())
            entry["species"] = "human" if key in ("man", "woman", "child") else ("wild_goat" if key == "goat" else key)
            entry["provenance"] = receipt.get("pipeline", "TRELLIS.2 set B + M3D-2b corrected rig")
            if "raw_sha256" in receipt:
                entry["raw_sha256"] = receipt["raw_sha256"]
            else:
                entry["source_commit"] = "f0b91f999e96c9095e32b9d649a835b40e37f6e7"
        entries["living_"+key] = entry
    assert len(entries) >= 23
    (base/"living/manifest.json").write_text(json.dumps({"models": entries}, indent=2)+"\n")
    print("M3D_LIVING_MANIFEST_PASS", len(entries))


if __name__ == "__main__":
    main()
