#!/usr/bin/env bash
# Invoke through with_window_lock.sh; raw evidence stays outside committed docs.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
PLAN="${1:-art}"
DOCS="${2:-docs/art/production/shots}"
case "$PLAN" in baseline|final|art|art_long|calibration) ;; *) echo 'Usage: capture_art.sh baseline|final|art|art_long|calibration <docs_dir>' >&2; exit 2;; esac
case "$DOCS" in docs/art/*|"$ROOT"/docs/art/*) ;; *) echo 'Output must be under docs/art/.' >&2; exit 2;; esac
[[ -d /Users/rexxa/work/.worldsim-godot-window.lock ]] || { echo 'Run capture_art.sh through with_window_lock.sh.' >&2; exit 2; }
[[ "$(uname -s)" == Darwin && "$(launchctl managername)" == Aqua ]] || { echo 'Capture requires the Mac Aqua session.' >&2; exit 77; }
GODOT_BIN="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
export WORLDSIM_GIT_SHA="$(git rev-parse HEAD)"
export WORLDSIM_GIT_DIRTY="$(git status --porcelain | wc -l | tr -d ' ')"
EVIDENCE="${CAPTURE_EVIDENCE:-$ROOT/.harness/evidence/art-production}"
out="$EVIDENCE/shots/$PLAN"
mkdir -p "$out" "$DOCS"
if [[ "${CAPTURE_SKIP_BUILD:-0}" != 1 ]]; then (cd rust && cargo build -p sim-bridge); fi
bash tools/art_pipeline/run_bounded.sh 300 "$GODOT_BIN" --headless --path . --import --editor --quit
bash tools/art_pipeline/run_bounded.sh 1100 "$GODOT_BIN" --path . --rendering-driver opengl3 --resolution 1920x1080 \
    --script res://scripts/test/capture_framings.gd -- --out "$out" --plan "$PLAN"
python3 - "$out" "$DOCS" <<'PY'
import json
import gzip
import hashlib
from pathlib import Path
import shutil
import sys
from PIL import Image
raw, docs = map(Path, sys.argv[1:])
manifest = json.loads((raw / 'manifest.json').read_text())
full = {'a_d2_1200_F2', 'a_d2_2300_F2N', 'a_d2_1200_F1'}
for capture in manifest['captures']:
    source = raw / capture['file']
    image = Image.open(source).convert('RGB')
    # Raw framebuffer remains lossless at its original Retina resolution.
    standard = image.resize((1920, 1080), Image.Resampling.NEAREST)
    standard.save(raw / (source.stem + '.1920.png'))
    capture['raw_file'] = str(source.resolve())
    capture['raw_sha256'] = hashlib.sha256(source.read_bytes()).hexdigest()
    entity_source = raw / capture['entities']
    capture['raw_entities_file'] = str(entity_source.resolve())
    capture['raw_entities_sha256'] = hashlib.sha256(entity_source.read_bytes()).hexdigest()
    capture['standard_file'] = str((raw / (source.stem + '.1920.png')).resolve())
    if source.stem.startswith(('a_strip_', 'a_evening_')):
        capture['published_file'] = '../village_noon.gif' if source.stem.startswith('a_strip_') else '../village_evening.gif'
        capture['entities'] = str(entity_source.resolve())
        capture['published_frame_index'] = int(source.stem.rsplit('_', 1)[1])
        capture['published_width'], capture['published_height'] = 960, 540
        continue
    if source.stem in full:
        standard.save(docs / source.name)
        capture['published_file'] = source.name
        capture['published_width'], capture['published_height'] = 1920, 1080
    else:
        target = source.stem + '.jpg'
        standard.resize((960, 540), Image.Resampling.NEAREST).save(docs / target, quality=85)
        capture['published_file'] = target
        capture['published_width'], capture['published_height'] = 960, 540
    shutil.copy2(raw / capture['entities'], docs / capture['entities'])
for extra in ['frame_times.json', 'strip.json', 'evening.json', 'forest_evidence.json']:
    if (raw / extra).exists():
        shutil.copy2(raw / extra, docs / extra)
for cycle in raw.glob('*.cold_cycle.json'):
    shutil.copy2(cycle, docs / cycle.name)
published = json.loads(json.dumps(manifest))
for metadata in docs.glob('*.json'):
    if metadata.name == 'manifest.json' or metadata.name.startswith('manifest_'):
        continue
    compact = (json.dumps(json.loads(metadata.read_text()), ensure_ascii=False, separators=(',', ':')) + '\n').encode()
    metadata.with_suffix('.json.gz').write_bytes(gzip.compress(compact, mtime=0))
    metadata.unlink()
for capture in published['captures']:
    if (docs / (capture['entities'] + '.gz')).exists():
        capture['entities'] += '.gz'
(docs / ('manifest_' + manifest['plan'] + '.json')).write_text(json.dumps(published, indent=2) + '\n')
combined = published
if manifest['plan'] in {'art', 'art_long'}:
    sibling_plan = 'art_long' if manifest['plan'] == 'art' else 'art'
    sibling_path = docs / ('manifest_' + sibling_plan + '.json')
    if sibling_path.exists():
        sibling = json.loads(sibling_path.read_text())
        if sibling['git_sha'] != published['git_sha']:
            raise ValueError('Cannot combine capture plans from different commits')
        combined = dict(published)
        combined['plan'] = 'art+art_long'
        combined['captures'] = (published['captures'] + sibling['captures'] if manifest['plan'] == 'art' else sibling['captures'] + published['captures'])
        combined['plan_manifests'] = ['manifest_art.json', 'manifest_art_long.json']
(docs / 'manifest.json').write_text(json.dumps(combined, indent=2) + '\n')
(raw / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(f"Published {len(manifest['captures'])} captures; raw framebuffers: {raw}")
PY
