#!/bin/bash
# WM-3: the DGX code clone detached at a pushed commit and a detached worktree of the DGX code clone at a pushed commit (one per commit), printed on stdout.
# usage: code.sh SHA
set -u
SHA=$1
D=/home/hyunlord/work/wm1-runs/wm3/_code/${SHA:0:8}
if [ ! -d "$D" ]; then
  git -C /home/hyunlord/work/new-world-wm1 fetch -q origin && \
  git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1
fi
git -C /home/hyunlord/work/new-world-wm1 checkout -q --detach "$SHA" 2>/dev/null   # the code clone at the job's commit
[ "$(git -C $D rev-parse HEAD)" = "$(git -C /home/hyunlord/work/new-world-wm1 rev-parse $SHA)" ] || { echo "worktree $D not at $SHA" >&2; exit 9; }
echo "$D"
