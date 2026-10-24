#!/usr/bin/env python3
"""Import the adopted art from a pinned art commit into assets/v8art/ (M-ART2 §3.3).

usage: art_import.py [--from COMMIT] [--out DIR] [--config tools/v8/art_import.json] [--check | --verify]

Reads every file straight from the git object store at the art commit (no checkout needed), copies only the files
named in the config (explicit lists; the people frames and portraits are the cross product of explicit look, facing,
pose and variant lists), keeps the bytes unchanged (PNG stays PNG, JSON stays JSON), and writes
`<out>/manifest.json` (every file: id, role, source path, source commit, SHA-256, size, pixel size, anchor/canvas
from the source manifests) plus `<out>/LICENSE.md`. The 36 logical looks of the appearance grid (age × sex × top ×
hair, in `look_grid` order) are listed with their fallback. Files under the managed directories that are not in the
new list are removed (`*.import` files belong to Godot and are left alone). Re-running on the same commit writes
byte-identical files. `--check` writes nothing and exits 1 when the output differs from what an import would write.
`--verify` (gate step `art_manifest`, no git needed) checks the committed folder against its own manifest: every
listed file exists with its SHA-256, byte size and PNG size, no unlisted file sits in a managed directory, the total
stays within the budget, every texture the material JSON names and every asset id `bindings.json` names resolves,
and the 36 logical looks point at imported frames.
The archive, diagnostics and pipelines stay on the art line.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CONFIG = ROOT / "tools/v8/art_import.json"

LICENSE = """# assets/v8art — licence and origin

Every file in this folder was made in this project: generated on the art line (branches `v8/art-*`) under the
project's own prompts and anchors, normalised by its own pipelines, and imported unchanged by
`tools/v8/art_import.py` from the pinned art commit named in `manifest.json` (`source_commit`). No third-party
asset, font or texture is included. The hand-written data files (`bindings.json`, `render_params.json`) and the
derived people masks (`people/masks/`, from `tools/v8/art_masks.py`) are this repository's own work.

Source commit: `{commit}` (branch `{branch}`).
"""


class GitReader:
    """Reads blobs at one commit through a single `git cat-file --batch` process."""

    def __init__(self, commit: str) -> None:
        self.commit = commit
        self.proc = subprocess.Popen(["git", "-C", str(ROOT), "cat-file", "--batch"], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE)

    def read(self, path: str) -> bytes:
        assert self.proc.stdin is not None and self.proc.stdout is not None
        self.proc.stdin.write(f"{self.commit}:{path}\n".encode())
        self.proc.stdin.flush()
        header = self.proc.stdout.readline().decode().split()
        if len(header) != 3 or header[1] != "blob":
            raise FileNotFoundError(f"{path} is not a file at {self.commit}: {' '.join(header)}")
        size = int(header[2])
        data = self.proc.stdout.read(size)
        self.proc.stdout.read(1)
        return data

    def close(self) -> None:
        if self.proc.stdin:
            self.proc.stdin.close()
        self.proc.wait()


def png_size(data: bytes) -> list[int]:
    """Width and height from the PNG header (IHDR), without decoding."""
    if data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise ValueError("not a PNG")
    return [int.from_bytes(data[16:20], "big"), int.from_bytes(data[20:24], "big")]


def person_luminance_std(data: bytes) -> float:
    """`tone.person_limit` of the art line: Rec.709 luminance std of the person anchor's foreground."""
    from PIL import Image  # only the tone reference needs decoding

    with Image.open(io.BytesIO(data)) as im:
        rgba = im.convert("RGBA")
    values: list[float] = []
    for r, g, b, a in rgba.get_flattened_data() if hasattr(rgba, "get_flattened_data") else rgba.getdata():
        rf, gf, bf = r / 255.0, g / 255.0, b / 255.0
        magenta = rf > .65 and bf > .65 and gf < .4 and abs(rf - bf) < .3
        if a / 255.0 >= .5 and not magenta:
            values.append(0.2126 * rf + 0.7152 * gf + 0.0722 * bf)
    mean = sum(values) / len(values)
    return (sum((v - mean) ** 2 for v in values) / len(values)) ** 0.5


def plan(config: dict, git: GitReader) -> tuple[list[dict], dict]:
    """Every file to write (dest relative to out, bytes, manifest row) and the manifest's extra sections."""
    rows: list[dict] = []
    people = config["people"]

    def add(role: str, source: str, dest: str, ident: str, meta: dict | None = None) -> None:
        data = git.read(source)
        row = {"id": ident, "role": role, "path": dest, "source": source,
               "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
        if dest.endswith(".png"):
            row["size"] = png_size(data)
        if meta:
            row["meta"] = meta
        rows.append({"dest": dest, "data": data, "row": row})

    sleep = set(people["sleep_poses"])
    for look in people["looks"]:
        for facing in people["facings"]:
            for pose in people["poses"]:
                geometry = people["sleep" if pose in sleep else "upright"]
                src = people["frame_source"].format(look=look, facing=facing, pose=pose)
                add("people_frame", src, people["frame_dest"].format(look=look, facing=facing, pose=pose),
                    f"{look}/{facing}/{pose}", {"canvas": geometry["canvas"], "anchor": geometry["anchor"]})
        for variant in people["variants"]:
            add("portrait", people["portrait_source"].format(look=look, variant=variant),
                people["portrait_dest"].format(look=look, variant=variant), f"{look}/{variant}")
    for entry in people["data"]:
        add("people_data", entry["source"], entry["dest"], entry["id"])

    metas = config["meta_sources"]
    structures = {a["id"]: a for a in json.loads(git.read(metas["structure"]))["assets"]}
    items_doc = json.loads(git.read(metas["item"]))
    items = {a["id"]: a for a in items_doc["assets"]}
    for entry in config["files"]:
        role, ident = entry["role"], entry.get("id", Path(entry["dest"]).stem)
        meta: dict | None = None
        if role == "structure":
            s = structures[ident]
            meta = {"anchor": [round(v) for v in s["anchor"]], "footprint": [round(v) for v in s["footprint"]],
                    "family": s["family"]}
        elif role == "rock_face":
            s = structures[ident]
            meta = {"anchor": [round(v) for v in s["anchor"]], "family": s["family"]}
        elif role == "ford_stone":
            sidecar = json.loads(git.read(metas["ford_stone"].format(n=ident.rsplit("_", 1)[1])))
            meta = {"anchor": sidecar["anchor"]}
        elif role == "plant":
            sidecar = json.loads(git.read(entry["source"][:-4] + ".json"))
            meta = {"anchor": [round(v) for v in sidecar["anchor_point"]]}
        elif role == "item":
            base = items.get(ident)
            if base is not None:
                meta = {"canvas": base["canvas_px"], "anchor": [round(v) for v in base["anchor_px"]]}
        add(role, entry["source"], entry["dest"], ident, meta)

    flora = config.get("flora")
    if flora:
        for entry in flora["data"]:
            add("flora_data", entry["source"], entry["dest"], entry["id"])
        for entry in flora.get("fixtures", []):
            add("flora_fixture", entry["source"], entry["dest"], entry["id"])
        catalogue = json.loads(git.read(flora["catalogue"]))
        frames: dict[str, dict] = {}
        masks: set[str] = set()

        def frame(f: dict) -> None:
            frames.setdefault(f["path"], {"canvas": f["canvas"], "anchor": f["anchor"]})
            masks.add(f["canopy_mask"])

        for species in catalogue["species"].values():
            for size in species["sizes"].values():
                for f in size["variants"] + size["bare"]:
                    frame(f)
            if species.get("stump"):
                frame(species["stump"])

        def dest_of(source: str) -> str:
            if not source.startswith(flora["source_prefix"]):
                raise ValueError(f"flora path outside {flora['source_prefix']}: {source}")
            return flora["dest_prefix"] + source[len(flora["source_prefix"]):]

        for source, meta in sorted(frames.items()):
            add("flora_frame", source, dest_of(source), Path(source).stem, meta)
        for source in sorted(masks):
            add("flora_mask", source, dest_of(source), Path(source).stem + "_mask")
        for source in sorted(set(catalogue["map_canopies"].values())):
            add("flora_texture", source, dest_of(source), Path(source).stem)

    animals = config["animals"]
    frames = {f["image"]: f for f in json.loads(git.read(animals["meta_source"]))["frames"]}
    for species in animals["species"]:
        for facing in animals["facings"]:
            for pose in animals["poses"][species]:
                src = animals["source"].format(species=species, facing=facing, pose=pose)
                f = frames[src]
                add("animal_frame", src, animals["dest"].format(species=species, facing=facing, pose=pose),
                    f"{species}/{facing}/{pose}", {"canvas": f["canvas_px"], "anchor": f["anchor_px"]})

    grid = config["look_grid"]
    present = set(people["looks"])
    looks = []
    for age in grid["ages"]:
        for sex in grid["sexes"]:
            for top in grid["tops"]:
                for hair in grid["hairs"]:
                    ident = f"{age}_{sex}_{top}_{hair}"
                    source = grid["fallbacks"].get(ident, ident)
                    if source not in present:
                        raise ValueError(f"look {ident} has no imported source ({source})")
                    looks.append({"index": len(looks), "id": ident, "frames": source, "fallback": source != ident,
                                  "age": age, "sex": sex, "top": top, "hair": hair})
    tone_source = metas["tone_reference"]
    tone_bytes = git.read(tone_source)
    extra = {
        "looks": looks,
        "people": {"facings": people["facings"], "poses": people["poses"], "variants": people["variants"],
                   "upright": people["upright"], "sleep": people["sleep"], "sleep_poses": people["sleep_poses"],
                   "frame_path": people["frame_dest"], "portrait_path": people["portrait_dest"]},
        "animals": {"species": animals["species"], "facings": animals["facings"], "poses": animals["poses"],
                    "frame_path": animals["dest"]},
        "tone_reference": {"source": tone_source, "sha256": hashlib.sha256(tone_bytes).hexdigest(),
                           "person_luminance_std": round(person_luminance_std(tone_bytes), 6)},
    }
    rows.sort(key=lambda r: r["dest"])
    dests = [r["dest"] for r in rows]
    if len(set(dests)) != len(dests):
        raise ValueError("two files map to the same destination")
    return rows, extra


def render(config: dict, commit: str) -> dict[str, bytes]:
    """Every output file (relative path → bytes)."""
    git = GitReader(commit)
    try:
        rows, extra = plan(config, git)
    finally:
        git.close()
    files = {r["dest"]: r["data"] for r in rows}
    manifest = {"schema_version": 1, "source_commit": commit, "source_branch": config["source_branch"],
                "generated_by": "tools/v8/art_import.py", "config": "tools/v8/art_import.json",
                **extra, "files": [r["row"] for r in rows]}
    total = sum(len(d) for d in files.values())
    manifest["total_bytes"] = total
    files["manifest.json"] = (json.dumps(manifest, indent=1, sort_keys=True, ensure_ascii=False) + "\n").encode()
    files["LICENSE.md"] = LICENSE.format(commit=commit, branch=config["source_branch"]).encode()
    return files


def managed(out: Path, config: dict) -> set[str]:
    found: set[str] = set()
    for d in config["managed_dirs"]:
        base = out / d
        if base.is_dir():
            for p in base.rglob("*"):
                if p.is_file() and not p.name.endswith(".import") and not p.relative_to(out).as_posix().startswith(
                        "people/masks/"):
                    found.add(p.relative_to(out).as_posix())
    return found


def binding_ids(node: object) -> set[str]:
    """Every asset id a bindings.json value names (keys that hold manifest ids)."""
    keys = {"ground", "structure", "unlit", "firing", "depleted", "working", "asset", "many", "spoiled", "large",
            "small", "stump", "face", "water", "snow", "item", "fallback"}
    found: set[str] = set()
    if isinstance(node, dict):
        for k, v in node.items():
            if k in keys and isinstance(v, str):
                found.add(v)
            elif k in ("assets", "ford_stones") and isinstance(v, list):
                found.update(x for x in v if isinstance(x, str))
            else:
                found |= binding_ids(v)
    elif isinstance(node, list):
        for v in node:
            found |= binding_ids(v)
    return found


def verify(out: Path, config: dict) -> list[str]:
    """Problems of the committed folder against its manifest (empty = pass)."""
    problems: list[str] = []
    manifest = json.loads((out / "manifest.json").read_text())
    listed = set()
    total = 0
    for row in manifest["files"]:
        p = out / row["path"]
        listed.add(row["path"])
        if not p.is_file():
            problems.append(f"missing {row['path']}")
            continue
        data = p.read_bytes()
        total += len(data)
        if hashlib.sha256(data).hexdigest() != row["sha256"] or len(data) != row["bytes"]:
            problems.append(f"hash or size differs: {row['path']}")
        if "size" in row and png_size(data) != row["size"]:
            problems.append(f"pixel size differs: {row['path']}")
    if total != manifest["total_bytes"]:
        problems.append(f"total {total} != manifest {manifest['total_bytes']}")
    if total > config["budget_bytes"]:
        problems.append(f"total {total} over the budget {config['budget_bytes']}")
    for p in sorted(managed(out, config) - listed):
        problems.append(f"unlisted file in a managed directory: {p}")
    sources = {row["source"]: row["path"] for row in manifest["files"]}
    styles = json.loads((out / "materials/construction_styles.json").read_text())
    for style in styles["styles"]:
        for role, path in style["textures"].items():
            if path not in sources:
                problems.append(f"style {style['id']} {role} texture not imported: {path}")
    ids = {row["id"] for row in manifest["files"]}
    for ident in sorted(binding_ids(json.loads((out / "bindings.json").read_text())) - ids):
        problems.append(f"bindings.json names an asset the manifest lacks: {ident}")
    frames = {row["id"].split("/")[0] for row in manifest["files"] if row["role"] == "people_frame"}
    if len(manifest["looks"]) != 36:
        problems.append(f"{len(manifest['looks'])} logical looks, expected 36")
    for look in manifest["looks"]:
        if look["frames"] not in frames:
            problems.append(f"look {look['id']} points at {look['frames']} which has no frames")
    flora = config.get("flora")
    if flora:
        catalogue = json.loads((out / flora["data"][0]["dest"]).read_text())
        named: set[str] = set(catalogue["map_canopies"].values())
        for species in catalogue["species"].values():
            for size in species["sizes"].values():
                for f in size["variants"] + size["bare"]:
                    named.update((f["path"], f["canopy_mask"]))
            if species.get("stump"):
                named.update((species["stump"]["path"], species["stump"]["canopy_mask"]))
        for path in sorted(named - set(sources)):
            problems.append(f"flora catalogue names a file the import lacks: {path}")
        bound = json.loads((out / "bindings.json").read_text()).get("trees", {}).get("species", [])
        for row in bound:
            if row.get("species") not in catalogue["species"]:
                problems.append(f"bindings.json trees.species names an unknown species: {row.get('species')}")
        if not bound:
            problems.append("bindings.json trees.species is empty")
    params = json.loads((out / "render_params.json").read_text())
    if params.get("schema_version") != 1:
        problems.append("render_params.json schema_version is not 1")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--config", default=str(DEFAULT_CONFIG))
    parser.add_argument("--from", dest="commit")
    parser.add_argument("--out")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    config = json.loads(Path(args.config).read_text())
    commit = args.commit or config["source_commit"]
    out = Path(args.out) if args.out else ROOT / config["out"]
    if args.verify:
        problems = verify(out, config)
        for p in problems[:40]:
            print(f"art_manifest: {p}")
        manifest = json.loads((out / "manifest.json").read_text())
        print(f"art_manifest: {'FAIL' if problems else 'PASS'} ({len(manifest['files'])} files, "
              f"{manifest['total_bytes']} bytes, {len(problems)} problems)")
        return 1 if problems else 0
    files = render(config, commit)
    total = sum(len(d) for p, d in files.items() if p not in ("manifest.json", "LICENSE.md"))
    if total > config["budget_bytes"]:
        print(f"art_import: {total} bytes over the budget {config['budget_bytes']}", file=sys.stderr)
        return 1
    stale = managed(out, config) - set(files)
    if args.check:
        bad = [p for p, d in files.items() if not (out / p).is_file() or (out / p).read_bytes() != d]
        for p in sorted(bad)[:20]:
            print(f"differs: {p}")
        for p in sorted(stale)[:20]:
            print(f"stale: {p}")
        print(f"art_import --check: {len(files)} files, {len(bad)} differ, {len(stale)} stale")
        return 1 if bad or stale else 0
    for p, d in files.items():
        target = out / p
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.is_file() or target.read_bytes() != d:
            target.write_bytes(d)
    for p in stale:
        (out / p).unlink()
    print(f"art_import: {len(files)} files, {total} bytes of art from {commit[:8]} into {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
