#!/usr/bin/env python3
"""Write <asset>.provenance.json next to a generated asset: engine, version, commit, inputs, command, licence, time.

usage: provenance.py ENGINE ASSET [ASSET...] --inputs IN [IN...] --command "CMD" [--engines-dir DIR]
The engine's version and commit come from engines.toml and are checked against the checkout that produced the asset.
"""
import argparse
import datetime
import hashlib
import json
import subprocess
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent


def sha256(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("engine")
    ap.add_argument("assets", nargs="+")
    ap.add_argument("--inputs", nargs="*", default=[])
    ap.add_argument("--command", required=True)
    ap.add_argument("--engines-dir", default=str(HERE.parents[2] / "3d-data" / "engines"))
    a = ap.parse_args()
    pin = tomllib.load(open(HERE / "engines.toml", "rb"))[a.engine]
    checkout = Path(a.engines_dir) / a.engine
    head = subprocess.run(["git", "-C", str(checkout), "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    if head != pin["commit"]:
        raise SystemExit(f"{a.engine} checkout is at {head}, engines.toml pins {pin['commit']}: run fetch_engines.sh")
    for asset in a.assets:
        rec = {"engine": a.engine, "repo": pin["repo"], "tag": pin["tag"], "commit": head, "licence": pin["licence"],
               "profile": pin.get("profile"), "command": a.command,
               "inputs": [{"path": i, "sha256": sha256(i)} if Path(i).is_file() else {"value": i} for i in a.inputs],
               "output_sha256": sha256(asset), "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")}
        Path(str(asset) + ".provenance.json").write_text(json.dumps(rec, indent=1) + "\n")
        print("provenance", asset)


if __name__ == "__main__":
    main()
