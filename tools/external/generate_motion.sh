#!/usr/bin/env bash
# Export motion from a motion-engine run (profile worldsim) to BVH + glTF + VAT in OUT, each with a .provenance.json.
# usage: tools/external/generate_motion.sh XML QPOS.npz NAME [OUT]
# XML/QPOS come from a motion-engine run (e.g. me train single ... -> rollout.npz); generation never runs at game runtime.
set -euo pipefail
py() { uv run -q --no-project --python 3.12 python "$@"; }  # tomllib needs Python >= 3.11
here=$(cd "$(dirname "$0")" && pwd)
eng=${ENGINES_DIR:-$here/../../../3d-data/engines}
xml=$1; qpos=$2; name=$3; out=${4:-$here/../../../3d-data/generated/motion}
me=$eng/motion-engine
mkdir -p "$out"
"$me/.venv/bin/me" export bvh --xml "$xml" --qpos "$qpos" --out "$out/$name.bvh"
"$me/.venv/bin/me" export gltf --xml "$xml" --qpos "$qpos" --out "$out/$name.glb"
"$me/.venv/bin/me" export vat --xml "$xml" --qpos "$qpos" --out "$out/$name-vat"
py "$here/provenance.py" motion-engine "$out/$name.bvh" "$out/$name.glb" "$out/$name-vat/positions.png" \
  --inputs "$xml" "$qpos" --command "me export {bvh,gltf,vat} --xml $xml --qpos $qpos" --engines-dir "$eng"
