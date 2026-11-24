#!/usr/bin/env bash
# Check out every engine pinned in engines.toml at its commit and give it a Python 3.12 venv.
# usage: tools/external/fetch_engines.sh [ENGINES_DIR]   (default: ../3d-data/engines, outside the repo)
set -euo pipefail
py() { uv run -q --no-project --python 3.12 python "$@"; }  # tomllib needs Python >= 3.11
here=$(cd "$(dirname "$0")" && pwd)
dst=${1:-$here/../../../3d-data/engines}
mkdir -p "$dst"
py - "$here/engines.toml" <<'PY' | while read -r name repo commit; do
import sys, tomllib
for name, e in tomllib.load(open(sys.argv[1], 'rb')).items():
    print(name, e['repo'], e['commit'])
PY
  if [ ! -d "$dst/$name/.git" ]; then git clone -q "$repo" "$dst/$name"; fi
  git -C "$dst/$name" fetch -q origin
  git -C "$dst/$name" -c advice.detachedHead=false checkout -q "$commit"
  if [ ! -x "$dst/$name/.venv/bin/python" ]; then
    (cd "$dst/$name" && uv venv -q --python 3.12 .venv && uv pip install -q --python .venv/bin/python -e .)
  fi
  echo "$name @ $(git -C "$dst/$name" rev-parse --short HEAD) -> $dst/$name"
done
