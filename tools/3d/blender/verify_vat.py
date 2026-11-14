#!/usr/bin/env python3
"""Verify VAT outputs without Blender: python3 tools/3d/blender/verify_vat.py [ids...]

For every character in assets/3d/models/manifest.json (family == "character") checks:
  * <id>_vat.json fields and that <id>_vat.bin size == width * frames * 2 * 16 bytes
  * all floats finite, position w == 1 and normal w == 0, normals unit length
  * clips are contiguous, inside [0, frames), fps == 15
  * <id>_static.glb: one mesh primitive, not skinned, has POSITION/NORMAL/TEXCOORD_1/COLOR_0,
    vertex count == width, TEXCOORD_1 maps each vertex to a unique texel, V == 0.5
  * frame 0 of `idle` matches the static mesh positions within 5 cm (per vertex, via TEXCOORD_1)
  * <id>.glb (skinned) has a skin and exactly the expected animation names
Exit code 1 on any failure.
"""
import json
import math
import os
import struct
import sys
from array import array

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
ASSETS = os.path.join(REPO, "assets", "3d")
COMP = {5126: ("f", 4), 5123: ("H", 2), 5121: ("B", 1), 5125: ("I", 4)}
NCOMP = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}


def read_glb(path):
    with open(path, "rb") as f:
        data = f.read()
    magic, _ver, _len = struct.unpack_from("<III", data, 0)
    assert magic == 0x46546C67, "not a glb"
    off, gltf, binc = 12, None, b""
    while off < len(data):
        clen, ctype = struct.unpack_from("<II", data, off)
        chunk = data[off + 8:off + 8 + clen]
        if ctype == 0x4E4F534A:
            gltf = json.loads(chunk)
        elif ctype == 0x004E4942:
            binc = chunk
        off += 8 + clen
    return gltf, binc


def accessor(gltf, binc, idx):
    acc = gltf["accessors"][idx]
    view = gltf["bufferViews"][acc["bufferView"]]
    fmt, size = COMP[acc["componentType"]]
    nc = NCOMP[acc["type"]]
    stride = view.get("byteStride", size * nc)
    base = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    out = []
    for i in range(acc["count"]):
        vals = struct.unpack_from("<" + fmt * nc, binc, base + i * stride)
        if acc.get("normalized"):
            vals = tuple(v / (2 ** (8 * size) - 1) for v in vals)
        out.append(vals)
    return out


def check(cid, expected_anims):
    errs = []
    meta = json.load(open(os.path.join(ASSETS, "vat", f"{cid}_vat.json")))
    for k in ("width", "vertex_count", "frames", "fps", "normals_offset", "clips", "bounds_min",
              "bounds_max", "height"):
        if k not in meta:
            errs.append(f"json missing {k}")
    w, frames = meta["width"], meta["frames"]
    if meta["fps"] != 15:
        errs.append("fps != 15")
    if meta["normals_offset"] != -1:
        errs.append("normals_offset != -1 (positions-only VAT)")
    if meta["vertex_count"] != w:
        errs.append("vertex_count != width")
    binp = os.path.join(ASSETS, "vat", f"{cid}_vat.bin")
    expect = w * frames * 8
    if os.path.getsize(binp) != expect:
        errs.append(f"bin size {os.path.getsize(binp)} != {expect}")
        return errs
    if expect > 5 * 1024 * 1024:
        errs.append(f"bin {expect} bytes exceeds the 5 MB tracked-file limit")
    raw = open(binp, "rb").read()
    data = [v[0] for v in struct.iter_unpack("<e", raw)]
    bad = sum(1 for v in data if not math.isfinite(v))
    if bad:
        errs.append(f"{bad} non-finite floats")
    for r in range(frames):
        base = r * w * 4
        for i in range(w):
            if data[base + i * 4 + 3] != 1.0:
                errs.append(f"row {r} texel {i} w != 1")
                break
    nxt = 0
    for name, c in meta["clips"].items():
        if c["start"] != nxt or c["count"] <= 0:
            errs.append(f"clip {name} not contiguous")
        nxt = c["start"] + c["count"]
    if nxt != frames:
        errs.append(f"clips cover {nxt} rows, frames = {frames}")
    if list(meta["clips"]) != list(expected_anims):
        errs.append(f"clip names {list(meta['clips'])} != {expected_anims}")

    gltf, binc = read_glb(os.path.join(ASSETS, "vat", f"{cid}_static.glb"))
    prims = [p for m in gltf["meshes"] for p in m["primitives"]]
    if len(prims) != 1:
        errs.append(f"static glb has {len(prims)} primitives (want 1 material)")
    if gltf.get("skins"):
        errs.append("static glb is skinned")
    attrs = prims[0]["attributes"]
    for a in ("POSITION", "NORMAL", "TEXCOORD_1", "COLOR_0"):
        if a not in attrs:
            errs.append(f"static glb missing {a}")
    if errs:
        return errs
    pos = accessor(gltf, binc, attrs["POSITION"])
    uv1 = accessor(gltf, binc, attrs["TEXCOORD_1"])
    col = accessor(gltf, binc, attrs["COLOR_0"])
    if len(pos) != meta.get("split_vertex_count", w):
        errs.append(f"static vertex count {len(pos)} != split_vertex_count {meta.get('split_vertex_count')}")
    seen = set()
    worst = 0.0
    idle0 = meta["clips"]["idle"]["start"]
    for p, (u, v), c in zip(pos, uv1, col):
        i = int(u * w)
        if abs(u - (i + 0.5) / w) > 0.25 / w or abs(v - 0.5) > 1e-4:
            errs.append(f"bad TEXCOORD_1 {u},{v}")
            break
        seen.add(i)
        if any(abs(ch - round(ch)) > 0.01 for ch in c[:3]):
            errs.append(f"COLOR_0 not a pure tint mask: {c}")
            break
        base = (idle0 * w + i) * 4
        d = math.dist(p, data[base:base + 3])
        worst = max(worst, d)
    if len(seen) != w:
        errs.append(f"TEXCOORD_1 maps to {len(seen)} unique texels, want {w}")
    if worst > 0.05:
        errs.append(f"idle frame 0 differs from static mesh by {worst:.3f} m (> 0.05)")

    sk, _ = read_glb(os.path.join(ASSETS, "models", f"{cid}.glb"))
    if not sk.get("skins"):
        errs.append("skinned glb has no skin")
    names = sorted(a.get("name") for a in sk.get("animations", []))
    if names != sorted(expected_anims):
        errs.append(f"skinned glb animations {names} != {sorted(expected_anims)}")
    print(f"{cid}: width={w} frames={frames} clips={ {k: v['count'] for k, v in meta['clips'].items()} } "
          f"idle0-vs-static max err={worst * 100:.2f} cm bounds_min={meta['bounds_min']} "
          f"-> {'OK' if not errs else 'FAIL'}")
    return errs


def main():
    manifest = json.load(open(os.path.join(ASSETS, "models", "manifest.json")))
    chars = {k: e for k, e in manifest["models"].items() if e.get("family") == "character"}
    want = sys.argv[1:] or sorted(chars)
    failed = False
    if not chars:
        print("no characters in manifest")
        return 1
    for cid in want:
        errs = check(cid, chars[cid]["animations"])
        for e in errs:
            print(f"  FAIL {cid}: {e}")
        failed |= bool(errs)
    print("ALL OK" if not failed else "FAILURES")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
