#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
"""Build action specs from the local CMU index, under the shared GPU lock."""

import argparse
import os
from pathlib import Path
import subprocess
import sys

from mocap_spec import BODIES, load_many, select

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
ART = REPO.parent / "art-img-data"
HOME_BREW_PYTHON = Path("/opt/homebrew/bin/python3")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("spec", nargs="+", type=Path)
    parser.add_argument("--index", type=Path, default=ART / "mocap1/index.json")
    parser.add_argument("--kit", type=Path, default=ART / "s2kit")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--blender", default="blender")
    mode_flags = parser.add_mutually_exclusive_group()
    mode_flags.add_argument("--compare", action="store_true", help="also bake CLIPS1 and compare chop and cook")
    mode_flags.add_argument("--bake-only", action="store_true", help="validate the VAT without rendering a review")
    mode_flags.add_argument("--review-slice", action="store_true", help="render only the selected review frames")
    mode_flags.add_argument("--review-strip", action="store_true", help="render the contact and a 12-frame p1 strip")
    parser.add_argument("--body", choices=BODIES, action="append", help="only for a focused bake-only check")
    args = parser.parse_args()
    if args.body and not args.bake_only:
        parser.error("--body requires --bake-only")
    specs = tuple(path.resolve() for path in args.spec)
    index, kit = args.index.resolve(), args.kit.resolve()
    save = (args.out or (ART / "mocap1/builds" / specs[0].stem)).resolve()
    if save.exists():
        parser.error(f"output already exists: {save}; use a fresh --out directory")
    actions = load_many(specs)
    if args.compare and {action.key for action in actions} != {"chop", "cook"}:
        parser.error("CLIPS1 comparison requires the chop and cook pilot keys")
    for action in actions:
        for source in ((action,) if action.base_layer is None else (action, action.base_layer)):
            take = select(source, index)
            print(f"MOCAP_SOURCE {source.key} {take.id} {take.fps:g} Hz {take.url}", flush=True)
    python = str(HOME_BREW_PYTHON) if HOME_BREW_PYTHON.is_file() else sys.executable
    lock = [python, str(REPO / "tools/v8/mux2_gate_lock.py"), "--"]
    worker = [args.blender, "--background", "--factory-startup", "--disable-autoexec",
              "--python-exit-code", "1", "--python", str(HERE / "mocap_bake.py"), "--"]
    bodies = ",".join(args.body or BODIES)
    variant = save / "new" if args.compare else save

    mode = next((key for enabled, key in ((args.bake_only, "bake"), (args.compare, "compare"),
                                         (args.review_slice, "final"), (args.review_strip, "strip")) if enabled),
                "render")
    inherited = os.environ.get("WORLDSIM_MUX2_LANE_FD")
    descriptors = () if inherited is None else (int(inherited),)
    parameters = (specs[0], index, kit, save, bodies, mode, *specs[1:])
    subprocess.run([*lock, *worker, *map(str, parameters)], cwd=REPO, check=True,
                   pass_fds=descriptors)
    if mode in ("bake", "final", "strip"):
        return 0
    review = [python, str(HERE / "mocap_review.py")]
    subprocess.run([*review, "contact", str(variant)], check=True)
    if args.compare:
        subprocess.run([*review, "compare", str(save / "old"), str(variant), str(save)], check=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
