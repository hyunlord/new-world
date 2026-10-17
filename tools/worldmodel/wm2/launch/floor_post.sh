#!/bin/bash
# WM-2 Part 1 (GOAL.md 6.4-6.5) after record_floor.sh: per-world symlink directories of every floor family, the prefix
# check (every member equals none_r0 over rows <= row(s)), replicate 0 against unforked recordings of two worlds per
# scene (game_v19: v2/v19/_unforked; game_v16: the control itself), and v2_floor_manifest.json.
# usage: floor_post.sh CODE_SHA
set -u
SHA=$1
W=/home/hyunlord/work/wm1-runs/wm2
V=/home/hyunlord/work/worldsim-wm-data/v2
D=$W/_code/${SHA:0:8}
[ -d "$D" ] || { git -C /home/hyunlord/work/new-world-wm1 fetch -q origin; git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1; }
source /home/hyunlord/work/wm1-venv/bin/activate
cd $D
O=$W/floor_post; mkdir -p $O
for sc in v19 v16; do
  for s in 20040 56040 99240; do
    python3 tools/worldmodel/wm2/families_v2.py link --flat $V/$sc/floor_s$s/flat --fam $V/$sc/floor_s$s/fam > $O/link_${sc}_s$s.txt || exit 1
    python3 tools/worldmodel/wm2/families_v2.py check --fam $V/$sc/floor_s$s/fam --out $O/check_${sc}_s$s.json > $O/check_${sc}_s$s.txt || exit 2
  done
done
for s in 20040 56040 99240; do
  python3 tools/worldmodel/wm2/families_v2.py r0 --floor-fam $V/v19/floor_s$s/fam --ref $V/v19/_unforked --seeds 40480 40481 --out $O/r0_v19_s$s.json > /dev/null || exit 3
  python3 tools/worldmodel/wm2/families_v2.py r0 --floor-fam $V/v16/floor_s$s/fam --ref $V/v16 --seeds 42480 42481 --out $O/r0_v16_s$s.json > /dev/null || exit 3
done
python3 tools/worldmodel/wm2/families_v2.py manifest --part floor --git-sha 26b9af1f4b1552359ad812efcd38b10449ca7175 --out $O/v2_floor_manifest.json > $O/manifest.txt || exit 4
echo ok
