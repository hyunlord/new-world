#!/usr/bin/env bash
# Generate villager bodies with human-forge (profile worldsim) into OUT (default: ../3d-data/generated/people), each
# with a .provenance.json. Blender runs under the Mac mux2 GPU lock. Never runs at game runtime.
# usage: tools/external/generate_people.sh [OUT] [PRESET...]   presets: villager_man villager_woman (profiles/worldsim/profile.toml)
set -euo pipefail
py() { uv run -q --no-project --python 3.12 python "$@"; }  # tomllib needs Python >= 3.11
here=$(cd "$(dirname "$0")" && pwd)
eng=${ENGINES_DIR:-$here/../../../3d-data/engines}
out=${1:-$here/../../../3d-data/generated/people}; shift || true
presets=${*:-villager_man villager_woman}
hf=$eng/human-forge
export HF_GPU_LOCK=${HF_GPU_LOCK:-"/opt/homebrew/bin/python3 /Users/rexxa/work/new-world-integ1/tools/v8/mux2_gate_lock.py --"}  # the Mac's shared lock file
mkdir -p "$out"
for p in $presets; do
  # preset -> sliders JSON (hair/beard become build flags)
  extra=$("$hf/.venv/bin/python" - "$hf/profiles/worldsim/profile.toml" "$p" "$out/$p.json" <<'PY'
import json, sys, tomllib
pr = tomllib.load(open(sys.argv[1], 'rb'))['presets'][sys.argv[2]]
json.dump({k: v for k, v in pr.items() if k not in ('hair', 'beard')}, open(sys.argv[3], 'w'))
print(' '.join(f'--{k} {pr[k]}' for k in ('hair', 'beard') if k in pr))
PY
)
  cmd="hf blender build $out/$p.json $out $p $extra"
  (cd "$hf" && .venv/bin/python -m human_forge.cli blender build "$out/$p.json" "$out" "$p" $extra)
  py "$here/provenance.py" human-forge "$out/$p.glb" "$out/$p.fbx" --inputs "$out/$p.json" --command "$cmd" --engines-dir "$eng"
done
