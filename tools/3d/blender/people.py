"""Villager: low-poly rigid-skinned human with procedural actions + VAT bake.
Run alone: blender -b --factory-startup -P people.py
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
from common import X, Y, Z, wave  # noqa: E402

# Body proportions (metres, Blender coords, front = -Y, character's left = +X)
BODY = dict(hip_z=0.92, spine_z=1.02, chest_z=1.22, neck_z=1.42, head_z=1.53, top_z=1.72,
            shoulder_x=0.19, hip_x=0.095, knee_z=0.5, ankle_z=0.08, elbow_z=1.14, wrist_z=0.9,
            arm_out=0.035, limb_segs=8)

# clip -> (duration seconds, loop)
CLIPS = {"idle": (2.0, True), "walk": (1.0, True), "work": (1.2, True), "carry": (1.0, True),
         "sleep": (3.0, True)}


def bones(b):
    sx, hx = b["shoulder_x"], b["hip_x"]
    out = [("hips", (0, 0, b["hip_z"]), (0, 0, b["spine_z"]), None),
           ("spine", (0, 0, b["spine_z"]), (0, 0, b["chest_z"]), "hips"),
           ("chest", (0, 0, b["chest_z"]), (0, 0, b["neck_z"]), "spine"),
           ("neck", (0, 0, b["neck_z"]), (0, 0, b["head_z"]), "chest"),
           ("head", (0, 0, b["head_z"]), (0, 0, b["top_z"]), "neck")]
    for side, s in (("L", 1), ("R", -1)):
        sh = (s * sx, 0, b["neck_z"] - 0.03)
        el = (s * (sx + b["arm_out"]), 0, b["elbow_z"])
        wr = (s * (sx + b["arm_out"] * 1.5), 0, b["wrist_z"])
        out += [(f"upper_arm_{side}", sh, el, "chest"),
                (f"lower_arm_{side}", el, wr, f"upper_arm_{side}"),
                (f"hand_{side}", wr, (wr[0], 0, wr[2] - 0.1), f"lower_arm_{side}"),
                (f"upper_leg_{side}", (s * hx, 0, b["hip_z"]), (s * hx, 0, b["knee_z"]), "hips"),
                (f"lower_leg_{side}", (s * hx, 0, b["knee_z"]), (s * hx, 0, b["ankle_z"]),
                 f"upper_leg_{side}"),
                (f"foot_{side}", (s * hx, 0, b["ankle_z"]), (s * hx, -0.14, 0.03),
                 f"lower_leg_{side}")]
    return out


def body_mesh(b):
    m = C.Model("villager")
    seg = b["limb_segs"]
    # torso: leather hip wrap, cloth tunic (abdomen + chest/shoulders)
    m.lathe("leather", [(0.15, 0.0), (0.165, 0.1), (0.15, 0.22)], loc=(0, 0, 0.8), segs=8,
            scale=(1, 0.75, 1), bone="hips")
    m.lathe("cloth", [(0.15, 0.0), (0.155, 0.12), (0.17, 0.24)], loc=(0, 0, 0.99), segs=8,
            scale=(1, 0.72, 1), bone="spine")
    m.lathe("cloth", [(0.17, 0.0), (0.2, 0.12), (0.2, 0.18), (0.11, 0.25)], loc=(0, 0, 1.2),
            segs=8, scale=(1, 0.66, 1), bone="chest")
    # neck, head, face cues, hair
    m.cyl("skin", 0.05, 0.14, loc=(0, 0, 1.41), segs=6, bone="neck")
    m.ball("skin", 0.115, loc=(0, -0.01, 1.625), scale=(0.9, 1.0, 1.1), subdiv=2, bone="head")
    m.box("skin", (0.035, 0.04, 0.05), loc=(0, -0.122, 1.61), taper=0.6, bone="head")  # nose
    for s in (1, -1):
        m.box("hair", (0.03, 0.02, 0.022), loc=(s * 0.042, -0.108, 1.648), bone="head")  # eyes
    m.ball("hair", 0.12, loc=(0, 0.02, 1.672), scale=(0.97, 1.0, 0.9), subdiv=2, bone="head")
    m.box("hair", (0.16, 0.07, 0.2), loc=(0, 0.085, 1.56), taper=0.8, bone="head")
    # limbs
    for side, s in (("L", 1), ("R", -1)):
        sx, ao = b["shoulder_x"], b["arm_out"]
        sh = (s * sx, 0, b["neck_z"] - 0.03)
        el = (s * (sx + ao), 0, b["elbow_z"])
        wr = (s * (sx + ao * 1.5), 0, b["wrist_z"])
        m.ball("cloth", 0.07, loc=sh, scale=(1, 0.9, 0.9), subdiv=1, bone=f"upper_arm_{side}")
        m.rod("cloth", sh, el, 0.055, r2=0.047, segs=seg, bone=f"upper_arm_{side}")
        m.rod("skin", el, wr, 0.045, r2=0.037, segs=seg, bone=f"lower_arm_{side}")
        m.ball("skin", 0.048, loc=(wr[0], -0.005, wr[2] - 0.05), scale=(0.7, 1.0, 1.2), subdiv=1,
               bone=f"hand_{side}")
        hx = s * b["hip_x"]
        m.rod("skin", (hx, 0, b["hip_z"] - 0.05), (hx, 0, b["knee_z"]), 0.072, r2=0.06, segs=seg,
              bone=f"upper_leg_{side}")
        m.rod("leather", (hx, 0, b["knee_z"]), (hx, 0, b["ankle_z"]), 0.058, r2=0.045, segs=seg,
              bone=f"lower_leg_{side}")
        m.box("leather", (0.095, 0.21, 0.075), loc=(hx, -0.045, 0.0375), taper=0.85,
              bone=f"foot_{side}")
    return m


# ---------------------------------------------------------------- actions (s = loop time 0..1)

def idle(s):
    b = 0.5 - 0.5 * math.cos(2 * math.pi * s)  # 0 at s=0 so frame 0 == rest pose
    return {"chest": [(X, -0.025 * b)], "head": [(Z, 0.12 * wave(s)), (X, 0.04 * b)],
            "upper_arm_L": [(Y, -0.03 * b)], "upper_arm_R": [(Y, 0.03 * b)],
            "hips": {"rot": [], "loc": (0, 0, -0.008 * b)}}


def gait(s, amp=0.45, arms=True):
    p = {}
    for side, ph in (("L", 0.0), ("R", 0.5)):
        sw = wave(s, ph)
        swing_fwd = max(0.0, math.cos(2 * math.pi * (s + ph)))
        p[f"upper_leg_{side}"] = [(X, -amp * sw)]
        p[f"lower_leg_{side}"] = [(X, 0.9 * amp * swing_fwd + 0.05)]
        p[f"foot_{side}"] = [(X, -0.3 * amp * swing_fwd)]
        if arms:
            p[f"upper_arm_{side}"] = [(X, 0.8 * amp * sw)]
            p[f"lower_arm_{side}"] = [(X, -0.25 - 0.2 * max(0.0, -sw))]
    p["hips"] = {"rot": [(Z, 0.06 * wave(s))], "loc": (0, 0, -0.025 + 0.025 * math.cos(4 * math.pi * s))}
    p["chest"] = [(Z, -0.1 * wave(s)), (X, 0.04)]
    return p


def walk(s):
    return gait(s)


def carry(s):
    p = gait(s, amp=0.35, arms=False)
    for side, sg in (("L", 1), ("R", -1)):
        p[f"upper_arm_{side}"] = [(Y, sg * 0.25), (X, -0.75)]
        p[f"lower_arm_{side}"] = [(X, -0.9)]
        p[f"hand_{side}"] = [(X, -0.2)]
    p["chest"] = [(X, -0.06)]
    return p


def ease(x):
    return x * x * (3 - 2 * x)


def work(s):
    # raise slowly (0..0.6), strike fast (0.6..0.72), recover (0.72..1)
    if s < 0.6:
        a = -0.6 + (-2.0) * ease(s / 0.6)
    elif s < 0.72:
        a = -2.6 + 2.1 * ease((s - 0.6) / 0.12)
    else:
        a = -0.5 - 0.1 * ease((s - 0.72) / 0.28)
    lean = 0.25 - 0.35 * (-(a + 0.5) / 2.1)  # lean back when raised, forward on the strike
    p = {"chest": [(X, lean * 0.6)], "spine": [(X, lean * 0.4)], "head": [(X, -lean * 0.4)]}
    for side, sg in (("L", 1), ("R", -1)):
        p[f"upper_arm_{side}"] = [(Y, sg * 0.3), (X, a)]
        p[f"lower_arm_{side}"] = [(X, -0.35)]
        p[f"upper_leg_{side}"] = [(X, -0.2), (Y, sg * 0.08)]
        p[f"lower_leg_{side}"] = [(X, 0.35)]
        p[f"foot_{side}"] = [(X, -0.15)]
    p["hips"] = {"rot": [], "loc": (0, 0, -0.045)}
    return p


def sleep(s):
    br = wave(s)
    p = {"hips": {"rot": [(X, -0.15), (Y, math.pi / 2)], "loc": (0, 0, -(0.92 - 0.25))},
         "chest": [(X, 0.12 + 0.025 * br)], "spine": [(X, 0.1)], "head": [(X, 0.25)]}
    for side in ("L", "R"):
        p[f"upper_leg_{side}"] = [(X, -1.2)]
        p[f"lower_leg_{side}"] = [(X, 1.6)]
        p[f"upper_arm_{side}"] = [(X, -1.1)]
        p[f"lower_arm_{side}"] = [(X, -0.9)]
    p["upper_arm_L"] = [(Y, 0.2), (X, -1.4)]
    return p


ACTIONS = {"idle": idle, "walk": walk, "work": work, "carry": carry, "sleep": sleep}


def build(entries):
    m = body_mesh(BODY)
    mesh = m.build_object("villager")
    arm = C.make_armature("villager", bones(BODY))
    clips = {}
    for name, (dur, loop) in CLIPS.items():
        clips[name] = (dur, loop, C.add_action(arm, name, dur, ACTIONS[name]))
    e = C.export_character("villager", mesh, arm, clips, entries)
    if not 500 <= e["triangles"] <= 1000:
        print("WARNING villager triangle budget 500-1000 violated:", e["triangles"])
    return entries


if __name__ == "__main__":
    C.reset_scene()
    C.update_manifest(build({}))
