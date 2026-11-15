"""Fit shared B joints and normalize anatomical heat weights."""
import bpy
from mathutils import Vector


def fit(arm: bpy.types.Object, height: float) -> None:
    """Fit one normalized B human template to measured chunky-body joint centers."""
    if "hips" not in arm.data.bones:
        height = arm.data.bones["Root"].head_local.z/.65
        specs = {name: (tuple(b.head_local/height), tuple(b.tail_local/height))
                 for name, b in arm.data.bones.items()}
        specs["Tail"] = ((0, .60, .78), (0, .85, .35))
        for side, sign in (("L", 1), ("R", -1)):
            for end, y in (("Front", -.20), ("Back", .40)):
                specs[f"{end}Upper.{side}"] = ((sign*.17, y, .65), (sign*.17, y, .30))
                specs[f"{end}Lower.{side}"] = ((sign*.17, y, .30), (sign*.17, y, .03))
        apply_specs(arm, specs, height)
        return
    specs = {"hips": ((0, 0, .36), (0, 0, .44)),
             "spine": ((0, 0, .44), (0, 0, .61)),
             "chest": ((0, 0, .61), (0, 0, .71)),
             "neck": ((0, 0, .71), (0, 0, .78)),
             "head": ((0, 0, .78), (0, 0, 1))}
    for side, sign in (("L", 1), ("R", -1)):
        for part, start, end in (("upper_arm", (.21, 0, .66), (.27, -.005, .52)),
                                 ("lower_arm", (.27, -.005, .52), (.305, -.015, .40)),
                                 ("hand", (.305, -.015, .40), (.32, -.025, .34)),
                                 ("upper_leg", (.105, 0, .34), (.12, 0, .18)),
                                 ("lower_leg", (.12, 0, .18), (.13, 0, .045)),
                                 ("foot", (.13, 0, .045), (.13, -.08, .025))):
            specs[f"{part}_{side}"] = ((sign*start[0], start[1], start[2]), (sign*end[0], end[1], end[2]))
    apply_specs(arm, specs, height)


def apply_specs(arm: bpy.types.Object, specs: dict, height: float) -> None:
    """Apply a common family rest template without changing vertex positions."""
    bpy.ops.object.select_all(action="DESELECT")
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    for name, (start, end) in specs.items():
        bone = arm.data.edit_bones.get(name) or arm.data.edit_bones.new(name)
        if name == "Tail":
            bone.parent = arm.data.edit_bones["Root"]
        bone.head, bone.tail = Vector(start)*height, Vector(end)*height
        bone.align_roll(Vector((0, -1, 0)))
    bpy.ops.object.mode_set(mode="OBJECT")


def blend(value: float, low: float, high: float) -> float:
    """Smoothly blend spatial anatomical regions without a hard seam."""
    t = max(0, min(1, (value-low)/(high-low)))
    return t*t*(3-2*t)


def heat(mesh: bpy.types.Object, arm: bpy.types.Object) -> None:
    """Use fitted-rig heat diffusion, retaining three normalized side-local groups."""
    mesh.vertex_groups.clear()
    bpy.ops.object.select_all(action="DESELECT")
    mesh.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    rows = []
    height = max(v.co.z for v in mesh.data.vertices)
    oversized = False
    if "head" in mesh.vertex_groups:
        group = mesh.vertex_groups["head"].index
        oversized = min(v.co.z for v in mesh.data.vertices
                        if any(g.group == group and g.weight > .7 for g in v.groups))/height < .74
    for vertex in mesh.data.vertices:
        suffix = ("_R", ".R") if vertex.co.x >= 0 else ("_L", ".L")
        # Reconstructed quadruped legs can cross the centre plane; heat owns anatomical side.
        row = [(mesh.vertex_groups[g.group].name, g.weight) for g in vertex.groups
               if ("Root" in arm.data.bones or not mesh.vertex_groups[g.group].name.endswith(suffix)) and g.weight > 1e-5]
        if oversized:
            side = "L" if vertex.co.x >= 0 else "R"
            chain = [f"{part}_{side}" for part in ("upper_arm", "lower_arm", "hand")]
            total_arm = sum(value for name, value in row if name in chain)
            if total_arm > .5:
                z = vertex.co.z/height
                hand = 1-blend(z, .36, .41)
                upper = blend(z, .49, .55)
                row = [(name, value) for name, value in row if name not in chain]
                row += list(zip(chain, (total_arm*(1-hand)*upper,
                                        total_arm*(1-hand)*(1-upper), total_arm*hand)))
        if "Root" in arm.data.bones and vertex.co.z < .25*(arm.data.bones["Root"].head_local.z/.65):
            row = [(name, value) for name, value in row if name not in ("Root", "Neck", "Head", "Tail")]
        if "Root" in arm.data.bones:
            shoulder = arm.data.bones["Root"].head_local.z/.65
            tail_weight = sum(value for name, value in row if name == "Tail")
            row = [(name, value) for name, value in row if name != "Tail"]
            if tail_weight:
                values = dict(row)
                values["Root"] = values.get("Root", 0)+tail_weight
                row = list(values.items())
            if vertex.co.y > .60*shoulder and vertex.co.z > .25*shoulder:
                tail = blend(vertex.co.y/shoulder, .60, .65)*(1-blend(abs(vertex.co.x)/shoulder, .07, .11))
                row = [("Root", 1-tail), ("Tail", tail)]
        row = sorted(row, key=lambda pair: pair[1], reverse=True)[:3]
        total = sum(value for _, value in row)
        assert total > 1e-6, ("Heat left unweighted vertex", vertex.index, tuple(vertex.co), [(mesh.vertex_groups[g.group].name, g.weight) for g in vertex.groups])
        rows.append([(name, value/total) for name, value in row])
    mesh.vertex_groups.clear()
    groups = {bone.name: mesh.vertex_groups.new(name=bone.name) for bone in arm.data.bones}
    for index, row in enumerate(rows):
        for name, value in row:
            groups[name].add([index], value, "REPLACE")
