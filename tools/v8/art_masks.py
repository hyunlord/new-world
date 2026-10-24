#!/usr/bin/env python3
"""Build the fixed people skin/hair masks of M-ART2 (goal §3.5: "fix the skin mask so it never covers clothing").

usage: art_masks.py [--check] [--commit COMMIT]

Inputs (all read-only): the 340 route I frames imported under `assets/v8art/people/frames/<look>/<facing>/<pose>.png`
(byte-identical to the art commit), and, straight from the git object store at the pinned art commit, the art line's
masks `art/character/variant_masks/<look>/<facing>/<pose>_{skin,hair}.png` (8-bit L, 255 = selected), the
source-authored envelopes `docs/v8/artpolish2/{ADULT_POSE,CHILD_POSE,IDLE,OWNER}_SOURCE_ENVELOPES.json` (garment,
prop and hair polygons, owner skin supports) and `art/character/variant_anatomy.json` (S/idle face and forearm boxes).

Outputs: `assets/v8art/people/masks/<look>/<facing>/<pose>.png` (RGBA8, same size as the frame; R = skin 0/255,
G = hair 0/255, B = 0, A = 255), `assets/v8art/people/masks/MASKS.json` (method, parameters, colour centres per
look, per-frame hashes, counts and removals) and `assets/v8art/people/masks/skin_regions.json` (the face/arm/leg/torso
boxes per frame and the rule that produced them).

The fix only removes pixels, never adds them. A skin pixel is dropped when (in this order, each pixel counted under
the first rule that drops it): it can never be tinted (alpha <= 127 or L* <= 18, the variant outline policy), it lies
inside a garment or prop polygon, its CIELAB colour is nearest to a clothing centre of its look (among clothing,
trusted-skin and trusted-hair centres), or it lies outside the frame's skin regions (skin_regions.json). A hair pixel
is dropped by the first three rules. Centres per look, over its ten frames: clothing = k-means of the opaque
non-outline pixels inside the garment and prop polygons (eroded by one pixel so antialiased skin at a polygon edge
does not become a clothing colour); trusted skin = the original skin mask inside the S/idle anatomy face and forearm
boxes and the owner skin supports; trusted hair = the original hair mask inside the hair envelopes, outside the hair
exclusions and the polygons. A clothing centre closer than skin_guard_delta_e to a trusted-skin centre is dropped
(tan hide and skin shadows share colours; there the polygons and the regions decide, not colour); for the hair rule
the clothing centres closer than hair_guard_delta_e to a hair centre are also left out (dark fur and dark hair share
colours; the source hair envelope decides). Deterministic: integer polygon rasterisation (Pillow, inclusive),
k-means on the look's unique colours (Lab rounded to 1e-6) with farthest-point initialisation from the darkest colour
and at most kmeans_iterations steps, centres rounded to 1e-6 after every step, PNG written by Pillow with fixed
settings (no ancillary chunks). `--check` writes nothing and exits 1 when any committed output differs from a fresh
build or a stale file sits in the masks folder.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
from art_import import GitReader  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
SOURCE_COMMIT = "c8ed86087c90ed5429716713e930cc5f6a9e1332"
PEOPLE = ROOT / "assets/v8art/people"
FRAMES = PEOPLE / "frames"
OUT = PEOPLE / "masks"
REGIONS_PATH = PEOPLE / "masks" / "skin_regions.json"
MASK_SOURCE = "art/character/variant_masks/{look}/{facing}/{pose}_{region}.png"
ENVELOPE_REPORTS = ("ADULT_POSE", "CHILD_POSE", "IDLE", "OWNER")
ENVELOPE_PATH = "docs/v8/artpolish2/{name}_SOURCE_ENVELOPES.json"
ANATOMY_PATH = "art/character/variant_anatomy.json"
FACINGS = ("S", "E")
POSES = ("idle", "walk_a", "work_a", "carry", "sleep")

PARAMS: dict = {
    "outline_L_max": 18,                # variant_transforms.json outline policy: never tinted
    "outline_alpha_max": 127,
    "clothing_centres": 6,              # k-means centres per look over garment/prop pixels
    "skin_centres": 4,                  # over trusted skin (anatomy boxes, owner skin supports x original mask)
    "hair_centres": 3,                  # over trusted hair (hair envelopes x original hair mask, minus exclusions)
    "kmeans_iterations": 25,
    "garment_sample_erosion": 1,        # clothing samples: polygon interiors only (4-neighbour erosion, px)
    "skin_guard_delta_e": 12,           # a clothing centre this close (CIE76) to a skin centre is ambiguous: dropped
    "hair_guard_delta_e": 8,            # same for hair: dark fur and dark hair share colours; the envelope decides
    "template": {                       # TEMPLATE_V3 canvases and anchors
        "standing_canvas": [80, 112], "foot_line_y": 104, "anchor_x": 40,
        "sleep_canvas": [112, 80], "sleep_lowest_y": 72, "sleep_centre_x": 56,
        "standing_height": {"adult": 56, "child": 40}, "sleep_length": {"adult": 56, "child": 40},
    },
    "head_margin": {"side": 3, "top": 2, "front_extra_E": 3},
    "neck_margin_rows": 2,              # neck rows = deepest owner neck support below the hair box + this
    "hip_margin_rows": 2,
    "torso_core_margin": 1,             # the S torso core is narrowed by this on each side (arm edges vary by pose)
    "sleep_legs_share_permille": 450,   # legs = the last 45 % of the sleeping body length (feet to the right)
    "sleep_arms_share_permille": 300,   # hands tucked by the head: 30 % of the body length after the hair box
    "bare_torso_tops": ["loincloth", "hide_wrap"],
}

RGB_TO_XYZ = np.array([[.4124564, .3575761, .1804375],
                       [.2126729, .7151522, .0721750],
                       [.0193339, .1191920, .9503041]])
WHITE = np.array([.95047, 1., 1.08883])
DELTA = 6 / 29


def rgb_to_lab(rgb: np.ndarray) -> np.ndarray:
    """Encoded sRGB in [0,1] to CIELAB (D65); the formula of the art line's `art/materials/tone.py`."""
    linear = np.where(rgb <= .04045, rgb / 12.92, ((rgb + .055) / 1.055) ** 2.4)
    xyz = (linear @ RGB_TO_XYZ.T) / WHITE
    f = np.where(xyz > DELTA ** 3, np.cbrt(xyz), xyz / (3 * DELTA ** 2) + 4 / 29)
    return np.stack((116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])), axis=-1)


def lab_to_rgb(lab: np.ndarray) -> np.ndarray:
    """CIELAB (D65) to clipped encoded sRGB in [0,1]; inverse of rgb_to_lab (art line `tone.py`)."""
    fy = (lab[..., 0] + 16) / 116
    f = np.stack((fy + lab[..., 1] / 500, fy, fy - lab[..., 2] / 200), axis=-1)
    xyz = np.where(f > DELTA, f ** 3, 3 * DELTA ** 2 * (f - 4 / 29)) * WHITE
    linear = np.clip(xyz @ np.linalg.inv(RGB_TO_XYZ).T, 0, 1)
    return np.where(linear <= .0031308, linear * 12.92, 1.055 * linear ** (1 / 2.4) - .055)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def age_of(look: str) -> str:
    return look.split("_", 1)[0]


def top_of(look: str) -> str:
    for top in ("fur_cloak", "hide_wrap", "loincloth"):
        if f"_{top}_" in look:
            return top
    raise ValueError(f"no top in look {look}")


def raster(shape: tuple[int, int], polygons: list) -> np.ndarray:
    """Union of polygons, inclusive Pillow rasterisation (the envelopes' stated convention)."""
    im = Image.new("L", (shape[1], shape[0]), 0)
    draw = ImageDraw.Draw(im)
    for poly in polygons:
        draw.polygon([(int(x), int(y)) for x, y in poly], fill=255, outline=255)
    return np.array(im) > 0


def box_mask(shape: tuple[int, int], boxes: list[list[int]]) -> np.ndarray:
    """Union of [left, top, right, bottom) boxes."""
    m = np.zeros(shape, bool)
    for left, top, right, bottom in boxes:
        m[max(top, 0):max(bottom, 0), max(left, 0):max(right, 0)] = True
    return m


def erode(m: np.ndarray, n: int) -> np.ndarray:
    """4-neighbour erosion, `n` steps; pixels on the canvas border count as outside."""
    for _ in range(n):
        p = np.pad(m, 1)
        m = m & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
    return m


def bbox(m: np.ndarray) -> list[int]:
    """[left, top, right, bottom) of the set pixels."""
    ys, xs = np.nonzero(m)
    return [int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1]


def png_bytes(arr: np.ndarray) -> bytes:
    buf = io.BytesIO()
    Image.fromarray(arr, "RGBA").save(buf, format="PNG", optimize=False, compress_level=9)
    return buf.getvalue()


def read_png(data: bytes, mode: str) -> np.ndarray:
    with Image.open(io.BytesIO(data)) as im:
        return np.array(im.convert(mode))


# ---------------------------------------------------------------- sources

def load_sources(commit: str = SOURCE_COMMIT) -> dict:
    """Everything the build reads: frames (repo), original masks, envelopes and anatomy (git at `commit`)."""
    git = GitReader(commit)
    try:
        envelopes: dict = {}
        report_hashes: dict = {}
        for name in ENVELOPE_REPORTS:
            path = ENVELOPE_PATH.format(name=name)
            data = git.read(path)
            report_hashes[path] = sha256(data)
            for fr in json.loads(data)["frames"]:
                _, _, _, look, facing, file = fr["source"].split("/")
                key = (look, facing, file[:-4])
                if key in envelopes:
                    raise ValueError(f"two envelope records for {key}")
                envelopes[key] = dict(fr, report=name)
        anatomy_data = git.read(ANATOMY_PATH)
        report_hashes[ANATOMY_PATH] = sha256(anatomy_data)
        anatomy = {fr["look"]: fr for fr in json.loads(anatomy_data)["frames"]}
        frames: dict = {}
        looks = sorted(p.name for p in FRAMES.iterdir() if p.is_dir())
        for look in looks:
            for facing in FACINGS:
                for pose in POSES:
                    key = (look, facing, pose)
                    frame_bytes = (FRAMES / look / facing / f"{pose}.png").read_bytes()
                    rec = {"frame": read_png(frame_bytes, "RGBA"), "frame_sha256": sha256(frame_bytes)}
                    for region in ("skin", "hair"):
                        data = git.read(MASK_SOURCE.format(look=look, facing=facing, pose=pose, region=region))
                        rec[f"{region}_source_sha256"] = sha256(data)
                        rec[f"{region}0"] = read_png(data, "L") > 127
                    if key not in envelopes:
                        raise ValueError(f"no source envelope for {key}")
                    rec["env"] = envelopes[key]
                    frames[key] = rec
    finally:
        git.close()
        if git.proc.stdout:
            git.proc.stdout.close()
    return {"commit": commit, "frames": frames, "anatomy": anatomy, "report_hashes": report_hashes}


# ---------------------------------------------------------------- geometry

def polygons(env: dict, *fields: str) -> list:
    out: list = []
    for f in fields:
        out.extend(env.get(f) or [])
    return out


def torso_cores(src: dict) -> dict:
    """Per age, the S-facing torso core [left, right): between the left arms' rightmost and the right arms' leftmost
    column over every look of that age, from the anatomy forearm boxes and the owner arm skin supports, narrowed by
    torso_core_margin."""
    left: dict = {}
    right: dict = {}
    for look, fr in src["anatomy"].items():
        a = age_of(look)
        left[a] = max(left.get(a, 0), fr["forearms"][0][2])
        right[a] = min(right.get(a, 999), fr["forearms"][1][0])
    for (look, _, _), rec in src["frames"].items():
        env = rec["env"]
        for poly, role in zip(env.get("skin_supports") or [], env.get("skin_support_roles") or []):
            xs = [v[0] for v in poly]
            if role.startswith("image_left_arm"):
                left[age_of(look)] = max(left[age_of(look)], max(xs) + 1)
            elif role.startswith("image_right_arm"):
                right[age_of(look)] = min(right[age_of(look)], min(xs))
    m = PARAMS["torso_core_margin"]
    return {a: [left[a] + m, right[a] - m] for a in sorted(left)}


def hip_rows(src: dict) -> dict:
    """Per age, the first row of the legs region on standing canvases: the highest owner leg skin support (S/idle;
    TEMPLATE_V3 gives walk/work/carry the S-idle scale and foot line) minus `hip_margin_rows`."""
    top: dict = {}
    for (look, _, _), rec in src["frames"].items():
        env = rec["env"]
        for poly, role in zip(env.get("skin_supports") or [], env.get("skin_support_roles") or []):
            if "_leg_" in role:
                a = age_of(look)
                top[a] = min(top.get(a, 999), min(v[1] for v in poly))
    return {a: top[a] - PARAMS["hip_margin_rows"] for a in sorted(top)}


def neck_rows(src: dict) -> dict:
    """Per age, how far skin reaches below the hair box: the deepest owner face/neck skin support below its frame's
    hair-envelope box, plus `neck_margin_rows`."""
    rows: dict = {}
    for (look, _, _), rec in src["frames"].items():
        env = rec["env"]
        supports = [p for p, role in zip(env.get("skin_supports") or [], env.get("skin_support_roles") or [])
                    if "neck" in role or "face" in role]
        if supports:
            hair_bottom = bbox(raster(rec["skin0"].shape, polygons(env, "hair_envelopes", "facial_hair_envelopes")))[3]
            reach = max(max(v[1] for v in p) + 1 for p in supports) - hair_bottom
            rows[age_of(look)] = max(rows.get(age_of(look), 0), reach)
    return {a: rows[a] + PARAMS["neck_margin_rows"] for a in sorted(rows)}


def frame_regions(key: tuple, rec: dict, cores: dict, hips: dict, necks: dict) -> dict:
    """The boxes a skin pixel of this frame may lie in (canvas coordinates, [left, top, right, bottom))."""
    look, facing, pose = key
    a = age_of(look)
    frame = rec["frame"]
    h, w = frame.shape[:2]
    env = rec["env"]
    hm = PARAMS["head_margin"]
    props = raster((h, w), polygons(env, "prop_exclusions"))
    fl, ft, fr, fb = bbox((frame[..., 3] >= 16) & ~props)
    hl, ht, hr, hb = bbox(raster((h, w), polygons(env, "hair_envelopes", "facial_hair_envelopes")))
    regions: dict = {}
    if pose == "sleep":
        length = PARAMS["template"]["sleep_length"][a]
        neck = necks[a]
        regions["head"] = [hl - hm["side"], ht - hm["side"], hr + hm["side"] + neck, hb + hm["side"]]
        arms_right = hr + length * PARAMS["sleep_arms_share_permille"] // 1000
        regions["arms"] = [[hr, ft, arms_right, fb]]
        regions["legs"] = [fr - length * PARAMS["sleep_legs_share_permille"] // 1000, ft, fr, fb]
    else:
        hip = hips[a]
        front = hm["front_extra_E"] if facing == "E" else 0
        regions["head"] = [hl - hm["side"], ht - hm["top"], hr + hm["side"] + front, hb + necks[a]]
        # overhead and shoulder band: every figure row down to the head box's bottom (raised hands, a log held
        # behind the head, an axe swung over it)
        arms = [[fl, ft, fr, regions["head"][3]]]
        if facing == "S":
            core_l, core_r = cores[a]
            arms.append([fl, ft, core_l, hip])
            arms.append([core_r, ft, fr, hip])
        else:
            arms.append([hl - hm["side"], ft, fr, hip])  # E: the arms hang in front of the back of the head
        regions["arms"] = arms
        regions["legs"] = [fl, hip, fr, max(fb, PARAMS["template"]["foot_line_y"] + 1)]
    if top_of(look) in PARAMS["bare_torso_tops"]:
        regions["torso"] = [fl, ft, fr, fb]
    clamp = lambda b: [max(b[0], 0), max(b[1], 0), min(b[2], w), min(b[3], h)]  # noqa: E731
    return {k: ([clamp(b) for b in v] if k == "arms" else clamp(v)) for k, v in regions.items()}


def region_boxes(regions: dict) -> list[list[int]]:
    out: list[list[int]] = []
    for k in ("head", "arms", "legs", "torso"):
        if k in regions:
            out.extend(regions[k] if k == "arms" else [regions[k]])
    return out


# ---------------------------------------------------------------- colour centres

def unique_colours(lab_rows: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """Unique Lab rows (lexicographic order) and their counts."""
    if len(lab_rows) == 0:
        return np.zeros((0, 3)), np.zeros(0, np.int64)
    uniq, counts = np.unique(lab_rows, axis=0, return_counts=True)
    return uniq, counts


def kmeans(points: np.ndarray, weights: np.ndarray, k: int) -> tuple[np.ndarray, np.ndarray]:
    """Weighted k-means, deterministic: farthest-point init from the darkest colour (lowest L*, then a*, b*),
    `kmeans_iterations` Lloyd steps, centres rounded to 1e-6. Returns centres and their pixel counts."""
    k = min(k, len(points))
    order = np.lexsort((points[:, 2], points[:, 1], points[:, 0]))
    centres = [points[order[0]]]
    dist = np.linalg.norm(points - centres[0], axis=1)
    while len(centres) < k:
        idx = int(np.argmax(dist))  # first maximum: ties broken by the lexicographic order of np.unique
        centres.append(points[idx])
        dist = np.minimum(dist, np.linalg.norm(points - points[idx], axis=1))
    c = np.round(np.array(centres, dtype=np.float64), 6)
    for _ in range(PARAMS["kmeans_iterations"]):
        assign = np.argmin(np.linalg.norm(points[:, None, :] - c[None, :, :], axis=2), axis=1)
        new = c.copy()
        for j in range(k):
            sel = assign == j
            if sel.any():
                new[j] = (points[sel] * weights[sel, None]).sum(axis=0) / weights[sel].sum()
        new = np.round(new, 6)
        if np.array_equal(new, c):
            break
        c = new
    assign = np.argmin(np.linalg.norm(points[:, None, :] - c[None, :, :], axis=2), axis=1)
    counts = np.array([int(weights[assign == j].sum()) for j in range(k)], np.int64)
    return c, counts


def eligible(frame: np.ndarray, lab: np.ndarray) -> np.ndarray:
    """Pixels a variant may tint: alpha >= 128 and L* > 18 (variant_transforms.json outline policy)."""
    return (frame[..., 3] > PARAMS["outline_alpha_max"]) & (lab[..., 0] > PARAMS["outline_L_max"])


def look_centres(look: str, src: dict, labs: dict) -> dict:
    """Clothing, trusted-skin and trusted-hair centres of one look, from all ten frames."""
    cloth: list = []
    skin: list = []
    hair: list = []
    for facing in FACINGS:
        for pose in POSES:
            rec = src["frames"][(look, facing, pose)]
            env = rec["env"]
            lab = labs[(look, facing, pose)]
            shape = lab.shape[:2]
            ok = eligible(rec["frame"], lab)
            garment = raster(shape, polygons(env, "garment_exclusions", "prop_exclusions"))
            cloth.append(lab[ok & erode(garment, PARAMS["garment_sample_erosion"])])
            hair_env = raster(shape, polygons(env, "hair_envelopes", "facial_hair_envelopes"))
            hair_env &= ~raster(shape, polygons(env, "hair_exclusions"))
            hair.append(lab[ok & hair_env & rec["hair0"] & ~garment])
            trusted = raster(shape, polygons(env, "skin_supports"))
            if (facing, pose) == ("S", "idle"):
                an = src["anatomy"][look]
                trusted |= box_mask(shape, [an["face"]] + an["forearms"])
            skin.append(lab[ok & trusted & rec["skin0"] & ~garment])
    out: dict = {}
    for name, rows, k in (("clothing", cloth, PARAMS["clothing_centres"]), ("skin", skin, PARAMS["skin_centres"]),
                          ("hair", hair, PARAMS["hair_centres"])):
        pts, w = unique_colours(np.round(np.concatenate(rows), 6))
        if len(pts) == 0:
            raise ValueError(f"{look}: no {name} pixels to cluster")
        centres, counts = kmeans(pts, w, k)
        out[name] = {"centres": centres, "counts": counts, "dropped": [], "pixels": int(w.sum())}
    cl = out["clothing"]
    gap = np.linalg.norm(cl["centres"][:, None, :] - out["skin"]["centres"][None, :, :], axis=2).min(axis=1)
    keep = gap >= PARAMS["skin_guard_delta_e"]
    cl["dropped"] = [[round(float(x), 3) for x in c] for c in cl["centres"][~keep]]
    cl["centres"], cl["counts"] = cl["centres"][keep], cl["counts"][keep]
    gap = np.linalg.norm(cl["centres"][:, None, :] - out["hair"]["centres"][None, :, :], axis=2).min(axis=1)
    cl["hair_rule"] = gap >= PARAMS["hair_guard_delta_e"]
    return out


def nearest_kind(lab: np.ndarray, centres: dict, for_hair: bool = False) -> np.ndarray:
    """0 = clothing, 1 = skin, 2 = hair: the kind of the nearest centre (ties: clothing first). For the hair rule the
    clothing centres within hair_guard_delta_e of a hair centre are left out."""
    cloth = centres["clothing"]["centres"]
    if for_hair:
        cloth = cloth[centres["clothing"]["hair_rule"]]
    allc = np.concatenate([cloth, centres["skin"]["centres"], centres["hair"]["centres"]])
    kinds = np.concatenate([np.full(len(cloth), 0), np.full(len(centres["skin"]["centres"]), 1),
                            np.full(len(centres["hair"]["centres"]), 2)])
    flat = lab.reshape(-1, 3)
    d = np.linalg.norm(flat[:, None, :] - allc[None, :, :], axis=2)
    return kinds[np.argmin(d, axis=1)].reshape(lab.shape[:2])


# ---------------------------------------------------------------- build

def frame_lab(frame: np.ndarray) -> np.ndarray:
    return np.round(rgb_to_lab(frame[..., :3].astype(np.float64) / 255), 6)


def fix_frame(rec: dict, lab: np.ndarray, centres: dict, regions: dict) -> tuple[np.ndarray, np.ndarray, dict]:
    shape = lab.shape[:2]
    env = rec["env"]
    ok = eligible(rec["frame"], lab)
    garment = raster(shape, polygons(env, "garment_exclusions", "prop_exclusions"))
    clothing = nearest_kind(lab, centres) == 0
    clothing_for_hair = nearest_kind(lab, centres, for_hair=True) == 0
    inside = box_mask(shape, region_boxes(regions))
    removed: dict = {}
    out = {}
    for region, rules in (("skin", (("outline", ~ok), ("polygon", garment), ("colour", clothing),
                                    ("region", ~inside))),
                          ("hair", (("outline", ~ok), ("polygon", garment), ("colour", clothing_for_hair)))):
        m = rec[f"{region}0"].copy()
        removed[region] = {}
        for name, drop in rules:
            hit = m & drop
            removed[region][name] = int(hit.sum())
            m &= ~hit
        out[region] = m
    return out["skin"], out["hair"], removed


def build(src: dict) -> tuple[dict[str, bytes], dict, dict]:
    """Every output file (path relative to the repo -> bytes), plus the MASKS.json and regions documents."""
    frames = src["frames"]
    labs = {k: frame_lab(rec["frame"]) for k, rec in frames.items()}
    cores = torso_cores(src)
    hips = hip_rows(src)
    necks = neck_rows(src)
    looks = sorted({k[0] for k in frames})
    centres = {look: look_centres(look, src, labs) for look in looks}
    files: dict[str, bytes] = {}
    rows = []
    region_rows = {}
    totals = {"skin_before": 0, "skin_after": 0, "hair_before": 0, "hair_after": 0,
              "skin_removed": {"outline": 0, "polygon": 0, "colour": 0, "region": 0},
              "hair_removed": {"outline": 0, "polygon": 0, "colour": 0}}
    for key in sorted(frames):
        look, facing, pose = key
        rec = frames[key]
        regions = frame_regions(key, rec, cores, hips, necks)
        region_rows[f"{look}/{facing}/{pose}"] = regions
        skin, hair, removed = fix_frame(rec, labs[key], centres[look], regions)
        h, w = skin.shape
        rgba = np.zeros((h, w, 4), np.uint8)
        rgba[..., 0] = skin * 255
        rgba[..., 1] = hair * 255
        rgba[..., 3] = 255
        data = png_bytes(rgba)
        rel = f"assets/v8art/people/masks/{look}/{facing}/{pose}.png"
        files[rel] = data
        row = {"look": look, "facing": facing, "pose": pose, "size": [w, h], "frame_sha256": rec["frame_sha256"],
               "skin_source": MASK_SOURCE.format(look=look, facing=facing, pose=pose, region="skin"),
               "skin_source_sha256": rec["skin_source_sha256"],
               "hair_source_sha256": rec["hair_source_sha256"], "output": rel, "output_sha256": sha256(data),
               "skin_before": int(rec["skin0"].sum()), "skin_after": int(skin.sum()),
               "hair_before": int(rec["hair0"].sum()), "hair_after": int(hair.sum()),
               "skin_removed": removed["skin"], "hair_removed": removed["hair"],
               "regions": sorted(regions), "envelope_report": rec["env"]["report"]}
        rows.append(row)
        for k in ("skin_before", "skin_after", "hair_before", "hair_after"):
            totals[k] += row[k]
        for region in ("skin", "hair"):
            for name, n in removed[region].items():
                totals[f"{region}_removed"][name] += n
    totals["frames"] = len(rows)
    totals["frames_losing_skin"] = sum(1 for r in rows if r["skin_after"] < r["skin_before"])
    method = " ".join(__doc__.split("\n\n")[4].split())
    masks_doc = {
        "schema_version": 1,
        "source_commit": src["commit"],
        "generator": "tools/v8/art_masks.py",
        "method": method,
        "channels": {"R": "skin 0/255", "G": "hair 0/255", "B": "0", "A": "255"},
        "parameters": PARAMS,
        "source_reports": src["report_hashes"],
        "regions": "assets/v8art/people/masks/skin_regions.json",
        "centres": {look: {name: {"lab": [[round(float(x), 3) for x in c] for c in v["centres"]],
                                  "pixels": [int(n) for n in v["counts"]], "dropped_near_skin": v["dropped"],
                                  "sample_pixels": v["pixels"]}
                           for name, v in centres[look].items()} for look in looks},
        "totals": totals,
        "frames": rows,
    }
    regions_doc = {
        "schema_version": 1,
        "source_commit": src["commit"],
        "generator": "tools/v8/art_masks.py",
        "convention": "canvas pixels, boxes [left, top, right, bottom) with right/bottom exclusive",
        "method": (
            "Derived from source-authored geometry and TEMPLATE_V3, never from the masks. Standing frames: head = the "
            "bounding box of the frame's hair (and facial hair) envelopes, widened by head_margin.side, raised by "
            "head_margin.top, widened on the front side by head_margin.front_extra_E for E, and extended down by the "
            "age's neck rows (the deepest owner face/neck skin support below its hair box, plus neck_margin_rows); "
            "legs = from the age's hip row to the foot line across the figure (figure = alpha >= 16 outside prop "
            "polygons); hip row = the highest owner leg skin support of that age (S/idle) minus hip_margin_rows "
            "(TEMPLATE_V3 gives walk/work/carry the S/idle scale and foot line); arms = the shoulder band (every "
            "figure row down to the head box's bottom: raised hands, a log held behind the head, an axe overhead) "
            "plus, for S, the figure left and right of the age's torso core from the figure top to the hip row "
            "(core = from the rightmost column of any left forearm/arm to the leftmost column of any right one over "
            "the anatomy forearm boxes and owner arm skin supports of every look of the age, narrowed by "
            "torso_core_margin), for E, the figure from the head box's left edge rightwards down to the hip row (the "
            "arms hang in front of the back of the head; the cloak hangs behind it). Sleep frames (head left after "
            "TEMPLATE_V3's rotation rule): head = hair box widened by head_margin.side and by the neck rows to the "
            "right; arms = from the hair box's right edge for sleep_arms_share of the template body length; legs = "
            "the last sleep_legs_share of the template body length ending at the figure's right edge. torso = the "
            "whole figure box, only for tops in bare_torso_tops (loincloth: bare chest or a chest band; hide wrap: a "
            "one-shoulder wrap leaving the other shoulder and upper chest bare). Fur cloak looks get no torso region: "
            "their tunic and cloak cover it."),
        "parameters": {k: PARAMS[k] for k in ("template", "head_margin", "neck_margin_rows", "hip_margin_rows",
                                              "torso_core_margin", "sleep_legs_share_permille",
                                              "sleep_arms_share_permille", "bare_torso_tops")},
        "derived": {"torso_core_S": cores, "hip_row": hips, "neck_rows": necks},
        "frames": region_rows,
    }
    files["assets/v8art/people/masks/MASKS.json"] = (json.dumps(masks_doc, indent=1) + "\n").encode()
    files["assets/v8art/people/masks/skin_regions.json"] = (json.dumps(regions_doc, indent=1) + "\n").encode()
    return files, masks_doc, regions_doc


def stale_outputs(files: dict[str, bytes], root: Path = ROOT) -> list[str]:
    """Files under the masks folder that a build would not write (Godot's `*.import` files excepted)."""
    out = root / OUT.relative_to(ROOT)
    if not out.is_dir():
        return []
    found = {p.relative_to(root).as_posix() for p in out.rglob("*") if p.is_file() and not p.name.endswith(".import")}
    return sorted(found - set(files))


def differences(files: dict[str, bytes], root: Path = ROOT) -> tuple[list[str], list[str]]:
    """(paths missing or different under `root`, stale paths): the `--check` verdict."""
    bad = [p for p, data in files.items() if not (root / p).is_file() or (root / p).read_bytes() != data]
    return bad, stale_outputs(files, root)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--check", action="store_true", help="write nothing; exit 1 if the outputs are stale")
    parser.add_argument("--commit", default=SOURCE_COMMIT)
    args = parser.parse_args()
    files, doc, _ = build(load_sources(args.commit))
    if args.check:
        bad, stale = differences(files)
        for p in bad[:20]:
            print(f"differs: {p}")
        for p in stale[:20]:
            print(f"stale: {p}")
        print(f"art_masks --check: {len(files)} files, {len(bad)} differ, {len(stale)} stale")
        return 1 if bad or stale else 0
    for p, data in files.items():
        dest = ROOT / p
        dest.parent.mkdir(parents=True, exist_ok=True)
        if not dest.is_file() or dest.read_bytes() != data:
            dest.write_bytes(data)
    for p in stale_outputs(files):
        (ROOT / p).unlink()
    t = doc["totals"]
    print(f"art_masks: {t['frames']} frames; skin {t['skin_before']} -> {t['skin_after']} "
          f"(removed {t['skin_removed']}); hair {t['hair_before']} -> {t['hair_after']} "
          f"(removed {t['hair_removed']}); {t['frames_losing_skin']} frames lost skin pixels")
    return 0


if __name__ == "__main__":
    sys.exit(main())
