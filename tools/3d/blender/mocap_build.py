#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
"""Build one action spec from the local CMU index, under the shared GPU lock."""

import argparse
import os
from pathlib import Path
import subprocess
import sys

from mocap_spec import BODIES, load, select

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
ART = REPO.parent / "art-img-data"
HOME_BREW_PYTHON = Path("/opt/homebrew/bin/python3")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("spec", type=Path)
    parser.add_argument("--index", type=Path, default=ART / "mocap1/index.json")
    parser.add_argument("--kit", type=Path, default=ART / "s2kit")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--blender", default="blender")
    parser.add_argument("--compare", action="store_true", help="also bake CLIPS1 and compare chop and cook")
    parser.add_argument("--bake-only", action="store_true", help="validate the VAT without rendering a review")
    parser.add_argument("--body", choices=BODIES, action="append", help="only for a focused bake-only check")
    args = parser.parse_args()
    if args.compare and args.bake_only or args.body and not args.bake_only:
        parser.error("--compare needs captures, and --body requires --bake-only")
    paths = (args.spec.resolve(), args.index.resolve(), args.kit.resolve())
    save = (args.out or (ART / "mocap1/builds" / args.spec.stem)).resolve()
    if save.exists():
        parser.error(f"output already exists: {save}; use a fresh --out directory")
    actions = load(paths[0])
    if args.compare and {action.key for action in actions} != {"chop", "cook"}:
        parser.error("CLIPS1 comparison requires the chop and cook pilot keys")
    for action in actions:
        take = select(action, paths[1])
        print(f"MOCAP_SOURCE {action.key} {take.id} {take.fps:g} Hz {take.url}", flush=True)
    python = str(HOME_BREW_PYTHON) if HOME_BREW_PYTHON.is_file() else sys.executable
    lock = [python, str(REPO / "tools/v8/mux2_gate_lock.py"), "--"]
    worker = [args.blender, "--background", "--factory-startup", "--disable-autoexec",
              "--python-exit-code", "1", "--python", str(HERE / "mocap_bake.py"), "--"]
    bodies = ",".join(args.body or BODIES)
    variant = save / "new" if args.compare else save

    mode = "bake" if args.bake_only else ("compare" if args.compare else "render")
    inherited = os.environ.get("WORLDSIM_MUX2_LANE_FD")
    descriptors = () if inherited is None else (int(inherited),)
    subprocess.run([*lock, *worker, *map(str, paths), str(save), bodies, mode], cwd=REPO, check=True,
                   pass_fds=descriptors)
    if args.bake_only:
        return 0
    review = [python, str(HERE / "mocap_review.py")]
    subprocess.run([*review, "contact", str(variant)], check=True)
    if args.compare:
        subprocess.run([*review, "compare", str(save / "old"), str(variant), str(save)], check=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
