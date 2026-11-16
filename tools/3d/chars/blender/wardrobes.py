"""Export era, age and sex attachments on the validated set-B shared skeletons."""
import json
from pathlib import Path
from typing import NamedTuple

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[4]
ERAS = {"neolithic": ((.48, .35, .18, 1), .0),
        "bronze": ((.55, .30, .10, 1), .7),
        "iron": ((.30, .32, .34, 1), .8)}


class Attachment(NamedTuple):
    name: str
    dimensions: tuple[float, float, float]
    centre: Vector


def attachment(spec, arm, material):
    """A rigid chest attachment follows the same shared animation tracks."""
    bpy.ops.mesh.primitive_cube_add(size=1, location=spec.centre)
    obj = bpy.context.object
    obj.name = spec.name
    obj.dimensions = spec.dimensions
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    group = obj.vertex_groups.new(name="chest")
    group.add(list(range(len(obj.data.vertices))), 1, "REPLACE")
    modifier = obj.modifiers.new("Shared skeleton", "ARMATURE")
    modifier.object = arm
    obj.data.materials.append(material)
    return obj


def main():
    out = ROOT/"assets/3d/living/people"
    out.mkdir(exist_ok=True)
    entries = {}
    for age in ("child", "adult", "elder"):
        for sex in ("M", "F"):
            subject = "child" if age == "child" else ("woman" if sex == "F" else "man")
            for era, (colour, metallic) in ERAS.items():
                bpy.ops.object.select_all(action="SELECT")
                bpy.ops.object.delete(use_global=False)
                bpy.ops.import_scene.gltf(filepath=str(ROOT/f"assets/3d/living/{subject}/{subject}.glb"))
                arm = next(o for o in bpy.context.scene.objects if o.type == "ARMATURE")
                material = bpy.data.materials.new(f"{era}_wardrobe")
                material.diffuse_color = colour
                material.use_nodes = True
                shader = material.node_tree.nodes.get("Principled BSDF")
                shader.inputs["Base Color"].default_value = colour
                shader.inputs["Metallic"].default_value = metallic
                chest = arm.data.bones["chest"].head_local
                height = 1.2 if age == "child" else 1.7
                # Metal pins and a fibre chest band keep the original atlas and body untouched.
                size = {"neolithic": (.19, .025, .035), "bronze": (.045, .028, .065),
                        "iron": (.065, .035, .045)}[era]
                attachment(Attachment(f"{era}_fastening", tuple(v*height for v in size),
                           chest+Vector((0, -.105*height, .015*height))), arm, material)
                if era != "neolithic":
                    cloth = bpy.data.materials.new(f"{era}_shoulder_cloth")
                    cloth.diffuse_color = (.30, .43, .38, 1) if era == "bronze" else (.40, .22, .16, 1)
                    cloth.use_nodes = True
                    cloth.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = cloth.diffuse_color
                    bpy.ops.mesh.primitive_cone_add(vertices=8, radius1=.24*height,
                        radius2=.19*height, depth=.14*height, location=(0, 0, .65*height))
                    wrap = bpy.context.object
                    wrap.name = f"{era}_shoulder_wrap"
                    wrap.scale.y = .72
                    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
                    wrap.vertex_groups.new(name="chest").add(list(range(len(wrap.data.vertices))), 1, "REPLACE")
                    wrap.modifiers.new("Shared skeleton", "ARMATURE").object = arm
                    wrap.data.materials.append(cloth)
                if age == "elder":
                    hair = bpy.data.materials.new("elder_silver_hair")
                    hair.diffuse_color = (.65, .63, .58, 1)
                    hair.use_nodes = True
                    hair.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = hair.diffuse_color
                    head = arm.data.bones["head"].tail_local
                    obj = attachment(Attachment("elder_hair", (.25*height, .21*height, .09*height),
                                     head+Vector((0, .015*height, -.025*height))), arm, hair)
                    obj.vertex_groups.clear()
                    obj.vertex_groups.new(name="head").add(list(range(len(obj.data.vertices))), 1, "REPLACE")
                ident = f"person_{age}_{sex}_{era}"
                bpy.ops.object.select_all(action="SELECT")
                bpy.ops.export_scene.gltf(filepath=str(out/f"{ident}.glb"), export_format="GLB",
                    export_animation_mode="NLA_TRACKS", export_force_sampling=True,
                    export_optimize_animation_size=False, export_optimize_animation_keep_anim_object=True)
                entries[ident] = {"file": f"living/people/{ident}.glb", "family": "character",
                    "source_subject": subject, "age": age, "sex": sex, "era": era,
                    "wardrobe": "fibre chest fastening" if era == "neolithic" else f"{era} garment pin and shoulder wrap",
                    "animations": ["idle", "walk", "work", "carry", "sleep"],
                    "provenance": "TRELLIS.2 set B + M3D-2b rig + rigid wardrobe attachments"}
    (ROOT/"assets/3d/living/people_manifest.json").write_text(json.dumps(entries, indent=2)+"\n")
    assert len(entries) == 18
    print("M3D_WARDROBES_PASS", len(entries))


if __name__ == "__main__":
    main()
