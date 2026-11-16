"""Animals: one parametric quadruped generator driven by the SPECIES table + procedural actions +
VAT bake. Run alone: blender -b --factory-startup -P animals.py [-- wolf deer ...]

Blender coords: front = -Y, left = +X. Lengths in metres. Angles in radians.
  S shoulder height, L shoulder-to-hip length, G body half-depth, W body width factor,
  neck = (length, angle from vertical towards the front, radius), head = (length, radius, pitch down),
  ear = (length, radius, outward tilt), tail = (length, radius, droop, bushiness),
  feet = "hoof" | "paw", extras = antlers/goat_horns/curl_horns/long_horns/tusks/bristles/beard/wool.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
from common import X, Y, Z, wave  # noqa: E402

SPECIES = {
    "wolf": dict(S=0.8, L=0.72, G=0.19, W=0.8, leg_r=0.038, neck=(0.3, 0.95, 0.12),
                 head=(0.3, 0.105, 0.2), ear=(0.12, 0.045, 0.25), tail=(0.45, 0.055, 0.95, 1.7),
                 feet="paw", extras={},
                 colors={"fur": (0.42, 0.40, 0.38), "fur_light": (0.78, 0.75, 0.70)}),
    "deer": dict(S=1.1, L=0.75, G=0.21, W=0.78, leg_r=0.033, neck=(0.45, 0.45, 0.085),
                 head=(0.28, 0.085, 0.55), ear=(0.14, 0.05, 0.9), tail=(0.12, 0.045, 1.2, 1.0),
                 feet="hoof", extras={"antlers": 1.0},
                 colors={"fur": (0.56, 0.36, 0.20), "fur_light": (0.88, 0.82, 0.72)}),
    "wild_goat": dict(S=0.75, L=0.6, G=0.2, W=0.85, leg_r=0.033, neck=(0.3, 0.5, 0.08),
                      head=(0.24, 0.08, 0.6), ear=(0.09, 0.035, 1.2), tail=(0.1, 0.03, -0.3, 1.0),
                      feet="hoof", extras={"goat_horns": 1.0, "beard": 1.0},
                      colors={"fur": (0.50, 0.42, 0.32), "fur_light": (0.82, 0.78, 0.68)}),
    "wild_sheep": dict(S=0.75, L=0.62, G=0.25, W=0.95, leg_r=0.032, neck=(0.25, 0.6, 0.1),
                       head=(0.22, 0.08, 0.6), ear=(0.08, 0.035, 1.5), tail=(0.12, 0.05, 1.3, 1.0),
                       feet="hoof", extras={"curl_horns": 1.0, "wool": 1.0},
                       colors={"fur": (0.30, 0.24, 0.20), "fur_light": (0.88, 0.85, 0.78)}),
    "wild_boar": dict(S=0.9, L=0.72, G=0.3, W=0.78, leg_r=0.042, neck=(0.18, 1.25, 0.21),
                      head=(0.42, 0.15, 0.45), ear=(0.11, 0.05, 0.5), tail=(0.22, 0.02, 1.3, 1.0),
                      feet="hoof", extras={"tusks": 1.0, "bristles": 1.0},
                      colors={"fur": (0.28, 0.22, 0.18), "fur_light": (0.45, 0.38, 0.30)}),
    "reindeer": dict(S=1.1, L=0.85, G=0.26, W=0.85, leg_r=0.042, neck=(0.42, 0.6, 0.11),
                     head=(0.32, 0.1, 0.55), ear=(0.1, 0.04, 1.0), tail=(0.1, 0.05, 1.2, 1.0),
                     feet="hoof", extras={"antlers": 1.4, "palmate": 1.0},
                     colors={"fur": (0.47, 0.41, 0.35), "fur_light": (0.86, 0.83, 0.77)}),
    "aurochs": dict(S=1.7, L=1.25, G=0.44, W=0.82, leg_r=0.068, neck=(0.38, 1.05, 0.28),
                    head=(0.5, 0.17, 0.75), ear=(0.13, 0.06, 1.5), tail=(0.75, 0.04, 1.45, 1.0),
                    feet="hoof", extras={"long_horns": 1.0},
                    colors={"fur": (0.17, 0.11, 0.08), "fur_light": (0.78, 0.68, 0.52)}),
}

# clip -> (duration s, loop)
CLIPS = {"idle": (2.0, True), "walk": (1.0, True), "run": (0.6, True), "graze": (2.4, True),
         "rest": (3.0, True), "attack": (1.0, False)}


def v3(*a):
    return tuple(float(x) for x in a)


def add(a, b, k=1.0):
    return tuple(x + y * k for x, y in zip(a, b))


class Rig:
    """Derived joint positions for one species."""

    def __init__(self, sp):
        S, L, G = sp["S"], sp["L"], sp["G"]
        self.zc = S - G * 0.85
        self.yf, self.yb = -L / 2, L / 2
        self.lx = G * sp["W"] * 0.55
        self.zk = S * 0.42
        nl, na, _ = sp["neck"]
        self.neck0 = v3(0, -L / 2 - G * 0.25, self.zc + G * 0.35)
        self.neck1 = add(self.neck0, (0, -math.sin(na), math.cos(na)), nl)
        hl, _, hp = sp["head"]
        self.hdir = (0, -math.cos(hp), -math.sin(hp))
        self.head1 = add(self.neck1, self.hdir, hl)
        tl, _, droop, _ = sp["tail"]
        self.tail0 = v3(0, L / 2 + G * 0.65, self.zc + G * 0.45)
        self.tail1 = add(self.tail0, (0, math.cos(droop), -math.sin(droop)), tl)


def bones(rig):
    out = [("hips", (0, rig.yb, rig.zc), (0, 0, rig.zc), None),
           ("chest", (0, 0, rig.zc), (0, rig.yf, rig.zc), "hips"),
           ("neck", rig.neck0, rig.neck1, "chest"),
           ("head", rig.neck1, rig.head1, "neck"),
           ("tail", rig.tail0, rig.tail1, "hips")]
    for end, y, parent in (("front", rig.yf, "chest"), ("back", rig.yb, "hips")):
        for side, s in (("L", 1), ("R", -1)):
            x = s * rig.lx
            out += [(f"{end}_upper_{side}", (x, y, rig.zc), (x, y, rig.zk), parent),
                    (f"{end}_lower_{side}", (x, y, rig.zk), (x, y, 0.02), f"{end}_upper_{side}")]
    return out


def body_mesh(cid, sp, rig):
    m = C.Model(cid)
    m.colors.update(sp["colors"])
    S, L, G, W, lr = sp["S"], sp["L"], sp["G"], sp["W"], sp["leg_r"]
    ex = sp["extras"]
    coat = "fur_light" if ex.get("wool") else "fur"
    # body: rear and front barrels + pale belly
    m.ball(coat, 1.0, loc=(0, L * 0.2, rig.zc), scale=(G * W, L * 0.45 + G * 0.3, G), subdiv=2,
           jitter=0.06 if ex.get("wool") else 0.02, bone="hips")
    hump = 1.12 if "long_horns" in ex or "tusks" in ex else 1.06
    m.ball(coat, 1.0, loc=(0, -L * 0.2, rig.zc + G * 0.05), scale=(G * W * 1.05, L * 0.45 + G * 0.3,
           G * hump), subdiv=2, jitter=0.06 if ex.get("wool") else 0.02, bone="chest")
    if not ex.get("wool"):
        m.ball("fur_light", 1.0, loc=(0, 0.0, rig.zc - G * 0.5),
               scale=(G * W * 0.75, L * 0.45, G * 0.5), subdiv=1, bone="hips")
    # neck + head
    nl, na, nr = sp["neck"]
    m.tube(coat, [add(rig.neck0, (0, 0.06, -0.06)), rig.neck1], [nr * 1.25, nr * 0.9], segs=7,
           bone="neck")
    hl, hr, _ = sp["head"]
    m.ball("fur", hr * 1.12, loc=add(rig.neck1, rig.hdir, hr * 0.3), scale=(1, 1.1, 1), subdiv=1,
           bone="head")
    m.rod("fur", add(rig.neck1, rig.hdir, hr * 0.3), rig.head1, hr * 0.85, r2=hr * 0.5, segs=7,
          bone="head")
    m.ball("fur_light" if not ex.get("tusks") else "horn", hr * 0.38, loc=rig.head1,
           scale=(1.1, 0.6, 0.9), subdiv=1, bone="head")  # muzzle / nose tip
    el, er, et = sp["ear"]
    for s in (1, -1):
        base = add(rig.neck1, (s * hr * 0.55, 0.02, hr * 0.55))
        tip = add(base, (s * math.sin(et), 0.25, math.cos(et)), el)
        m.rod("fur", base, tip, er, r2=0.0, segs=4, bone="head")
    # legs
    for end, y in (("front", rig.yf), ("back", rig.yb)):
        thigh = 1.9 if end == "back" else 1.5
        for side, s in (("L", 1), ("R", -1)):
            x = s * rig.lx
            m.rod(coat if ex.get("wool") and end == "back" else "fur", (x, y, rig.zc - G * 0.2),
                  (x, y, rig.zk), lr * thigh, r2=lr * 1.05, segs=6, bone=f"{end}_upper_{side}")
            m.rod("fur", (x, y, rig.zk), (x, y, 0.07), lr, r2=lr * 0.8, segs=6,
                  bone=f"{end}_lower_{side}")
            if sp["feet"] == "hoof":
                m.cyl("horn", lr * 0.95, 0.07, loc=(x, y - 0.01, 0.0), segs=6, r2=lr * 0.8,
                      bone=f"{end}_lower_{side}")
            else:
                m.ball("fur", lr * 1.3, loc=(x, y - lr * 0.6, lr * 0.9), scale=(1, 1.4, 0.7),
                       subdiv=1, bone=f"{end}_lower_{side}")
    # tail (bushy tails swell in the middle)
    tl, tr, _, bushy = sp["tail"]
    mid = add(rig.tail0, tuple(b - a for a, b in zip(rig.tail0, rig.tail1)), 0.5)
    m.tube("fur", [rig.tail0, mid, rig.tail1], [tr, tr * bushy, 0.0], segs=6, bone="tail")
    extras(m, sp, rig)
    return m


def extras(m, sp, rig):
    ex = sp["extras"]
    hr = sp["head"][1]
    top = add(rig.neck1, (0, 0.0, hr * 0.85))
    for s in (1, -1):
        if "antlers" in ex:
            k = ex["antlers"]
            b0 = add(top, (s * hr * 0.4, 0.01, 0))
            pts = [b0, add(b0, (s * 0.1, 0.06, 0.2), k), add(b0, (s * 0.2, 0.12, 0.38), k),
                   add(b0, (s * 0.24, 0.08, 0.55), k)]
            m.tube("horn", pts, [0.022 * k, 0.019 * k, 0.015 * k, 0.0], segs=5, bone="head")
            for i, (p, d) in enumerate(((pts[1], (s * 0.02, -0.12, 0.12)),
                                        (pts[2], (s * 0.03, -0.1, 0.16)))):
                m.rod("horn", p, add(p, d, k), 0.014 * k, r2=0.0, segs=4, bone="head")
            if "palmate" in ex:  # reindeer: forward brow tine + a crown of short tines
                m.rod("horn", add(b0, (s * 0.03, 0.0, 0.05), k), add(b0, (s * 0.05, -0.17, 0.12), k),
                      0.016 * k, r2=0.0, segs=4, bone="head")
                for d in ((s * 0.06, -0.06, 0.08), (s * 0.02, 0.07, 0.07)):
                    m.rod("horn", pts[3], add(pts[3], d, k), 0.012 * k, r2=0.0, segs=4,
                          bone="head")
        if "goat_horns" in ex:
            b0 = add(top, (s * hr * 0.35, 0.02, -0.01))
            m.tube("horn", [b0, add(b0, (s * 0.04, 0.06, 0.13)), add(b0, (s * 0.08, 0.2, 0.2)),
                            add(b0, (s * 0.1, 0.32, 0.14))], [0.03, 0.025, 0.016, 0.0], segs=5,
                   bone="head")
        if "curl_horns" in ex:
            c = add(rig.neck1, (s * (hr + 0.03), 0.04, hr * 0.15))
            pts, radii = [], []
            for i in range(7):
                t = i / 6
                a = math.pi * 0.6 + t * math.pi * 1.6
                r = 0.1 * (1 - 0.45 * t)
                pts.append(add(c, (s * 0.06 * t, math.cos(a) * r, math.sin(a) * r)))
                radii.append(0.04 * (1 - 0.75 * t))
            m.tube("horn", pts, radii, segs=5, bone="head")
        if "long_horns" in ex:
            b0 = add(top, (s * hr * 0.6, 0.02, -0.03))
            m.tube("horn", [b0, add(b0, (s * 0.22, 0.02, 0.02)), add(b0, (s * 0.38, -0.12, 0.14)),
                            add(b0, (s * 0.4, -0.25, 0.34))], [0.055, 0.045, 0.03, 0.0], segs=6,
                   bone="head")
        if "tusks" in ex:
            b0 = add(rig.head1, rig.hdir, -0.1)
            b0 = add(b0, (s * hr * 0.45, 0, -hr * 0.2))
            m.tube("horn", [b0, add(b0, (s * 0.03, -0.04, 0.03)), add(b0, (s * 0.04, -0.03, 0.09))],
                   [0.018, 0.014, 0.0], segs=4, bone="head")
    if "beard" in ex:
        chin = add(rig.head1, rig.hdir, -0.08)
        m.rod("fur_light", add(chin, (0, 0, -0.03)), add(chin, (0, 0.03, -0.17)), 0.03, r2=0.0,
              segs=5, bone="head")
    if "bristles" in ex:
        for i in range(7):
            f = i / 6
            y = -sp["L"] * 0.55 + f * sp["L"] * 0.85
            z = rig.zc + sp["G"] * (1.12 - 0.25 * f)
            m.rod("fur", (0, y, z - 0.04), (0, y + 0.06, z + 0.09 - 0.04 * f), 0.035, r2=0.0, segs=4,
                  bone="chest" if y < 0 else "hips")
    if "wool" in ex:
        for i in range(6):
            a = 2 * math.pi * i / 6
            m.ball("fur_light", sp["G"] * 0.5, loc=(math.cos(a) * sp["G"] * 0.55,
                   (i - 2.5) * sp["L"] * 0.15, rig.zc + sp["G"] * 0.45 + 0.03 * math.sin(a)),
                   subdiv=1, jitter=0.12, bone="hips" if i >= 3 else "chest")


# ---------------------------------------------------------------- actions (s = loop time 0..1)

GRAZE_HEAD = 1.35  # head pitch below horizontal while grazing (muzzle down, horns/antlers up-front)


def graze_angle(sp, rig):
    """Neck pitch that brings the muzzle to the ground while the head itself points down at
    GRAZE_HEAD (front legs straight, chest dipped 0.08 rad)."""
    hl = sp["head"][0]
    d = tuple(b - a for a, b in zip(rig.neck0, rig.neck1))

    def tip_z(th):
        nz = d[1] * math.sin(th) + d[2] * math.cos(th)
        return rig.neck0[2] - 0.08 * abs(rig.neck0[1]) + nz - hl * math.sin(GRAZE_HEAD)
    th = 0.0
    while th < 2.0 and tip_z(th) > 0.05:
        th += 0.02
    return th


def legs(p, s, amp, phases, knee=0.6):
    for (end, side), ph in phases.items():
        sw = wave(s, ph)
        fwd = max(0.0, math.cos(2 * math.pi * (s + ph)))
        p[f"{end}_upper_{side}"] = [(X, -amp * sw)]
        p[f"{end}_lower_{side}"] = [(X, (knee if end == "front" else -knee * 0.6) * amp * fwd)]


def make_actions(sp, rig):
    G = sp["G"]
    gz = graze_angle(sp, rig)
    horned = any(k in sp["extras"] for k in ("antlers", "goat_horns", "curl_horns", "long_horns",
                                              "tusks"))

    def idle(s):
        b = 0.5 - 0.5 * math.cos(2 * math.pi * s)
        return {"neck": [(Z, 0.15 * wave(s)), (X, -0.05 * b)], "head": [(X, 0.08 * wave(s, 0, 2) * b)],
                "tail": [(Z, 0.3 * wave(s, 0, 2))], "chest": [(X, 0.01 * b)]}

    def walk(s):
        p = {}
        legs(p, s, 0.4, {("back", "L"): 0.0, ("front", "L"): 0.25, ("back", "R"): 0.5,
                         ("front", "R"): 0.75})
        p["hips"] = {"rot": [(Z, 0.03 * wave(s))], "loc": (0, 0, 0.015 * math.cos(4 * math.pi * s))}
        p["neck"] = [(X, 0.06 * wave(s, 0, 2))]
        p["tail"] = [(Z, 0.2 * wave(s))]
        return p

    def run(s):
        p = {}
        legs(p, s, 0.75, {("front", "L"): 0.0, ("front", "R"): 0.08, ("back", "L"): 0.5,
                          ("back", "R"): 0.58}, knee=0.9)
        p["hips"] = {"rot": [(X, 0.12 * wave(s, 0.25))], "loc": (0, 0, 0.05 * wave(s, 0.25) + 0.02)}
        p["chest"] = [(X, -0.1 * wave(s, 0.25))]
        p["neck"] = [(X, 0.25 + 0.12 * wave(s))]
        p["tail"] = [(X, -0.4)]
        return p

    def graze(s):
        chew = 0.04 * wave(s, 0, 4)
        hz = GRAZE_HEAD - sp["head"][2] - gz  # counter-rotate so the head ends pointing down
        p = {"neck": [(X, gz), (Z, 0.12 * wave(s))], "head": [(X, hz + chew)],
             "tail": [(Z, 0.25 * wave(s, 0, 2))]}
        step = max(0.0, wave(s, 0.0)) ** 4  # one slow shuffle step per loop
        p["front_upper_L"] = [(X, -0.25 * step)]
        p["front_lower_L"] = [(X, 0.4 * step)]
        p["chest"] = [(X, 0.08)]
        p["hips"] = {"rot": [], "loc": (0, 0, -0.01)}
        return p

    def rest(s):
        br = wave(s)
        drop = rig.zc - G * 1.0
        p = {"hips": {"rot": [(Y, 0.12)], "loc": (0, 0, -drop)}, "chest": [(X, -0.03 * br * 0.5)],
             "neck": [(X, -0.15), (Z, 0.12 * wave(s))], "head": [(X, 0.2)], "tail": [(Y, 0.3)]}
        for side in ("L", "R"):
            p[f"front_upper_{side}"] = [(X, 1.35)]
            p[f"front_lower_{side}"] = [(X, -2.6)]
            p[f"back_upper_{side}"] = [(X, -1.25), (Y, 0.3 if side == "L" else -0.3)]
            p[f"back_lower_{side}"] = [(X, 2.4)]
        return p

    def attack(s):
        # wind up (0..0.35), lunge (0.35..0.55), recover (0.55..1)
        if s < 0.35:
            k = s / 0.35
            fwd, down = -0.12 * k, 0.4 * k
        elif s < 0.55:
            k = (s - 0.35) / 0.2
            fwd, down = -0.12 + 0.5 * k, 0.4 + 0.3 * k
        else:
            k = (s - 0.55) / 0.45
            k = k * k * (3 - 2 * k)
            fwd, down = 0.38 * (1 - k), 0.7 * (1 - k)
        p = {"hips": {"rot": [(X, -0.08 * down)], "loc": (0, -fwd * sp["L"], -0.03 * down)}}
        if horned:  # head down, horns first
            p["neck"] = [(X, down * min(gz, 1.1))]
            p["head"] = [(X, 0.3 * down)]
        else:  # predator lunge: neck forward, head up, front legs reach
            p["neck"] = [(X, 0.5 * down)]
            p["head"] = [(X, -0.35 * down)]
        reach = max(0.0, fwd) * 1.6
        for side in ("L", "R"):
            p[f"front_upper_{side}"] = [(X, -reach)]
            p[f"front_lower_{side}"] = [(X, 0.5 * reach)]
            p[f"back_upper_{side}"] = [(X, 0.4 * reach)]
        p["tail"] = [(X, -0.5 * down)]
        return p

    return {"idle": idle, "walk": walk, "run": run, "graze": graze, "rest": rest, "attack": attack}


def build(entries, only=None):
    for cid, sp in SPECIES.items():
        if only and cid not in only:
            continue
        C.reset_scene()
        rig = Rig(sp)
        mesh = body_mesh(cid, sp, rig).build_object(cid)
        arm = C.make_armature(cid, bones(rig))
        acts = make_actions(sp, rig)
        clips = {n: (d, lp, C.add_action(arm, n, d, acts[n])) for n, (d, lp) in CLIPS.items()}
        e = C.export_character(cid, mesh, arm, clips, entries,
                               {"shoulder_height": sp["S"], "species": cid})
        if not 300 <= e["triangles"] <= 900:
            print(f"WARNING {cid} triangle budget 300-900 violated: {e['triangles']}")
    return entries


if __name__ == "__main__":
    C.reset_scene()
    C.update_manifest(build({}, C.script_args() or None))
