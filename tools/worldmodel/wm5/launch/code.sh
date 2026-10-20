#!/bin/bash
# WM-5 (WM-4's code.sh, wm5 paths): a detached worktree of the DGX code clone at a pushed commit (one per commit),
# created under a lock, printed on stdout.
# usage: code.sh SHA
set -u
SHA=$1
D=/home/hyunlord/work/wm1-runs/wm5/_code/${SHA:0:8}
mkdir -p /home/hyunlord/work/wm1-runs/wm5/_code
exec 9>/home/hyunlord/work/wm1-runs/wm5/_code/.lock
flock 9            # concurrent jobs never see a half-created worktree
if [ ! -d "$D" ]; then
  git -C /home/hyunlord/work/new-world-wm1 fetch -q origin && \
  git -C /home/hyunlord/work/new-world-wm1 worktree add -f --detach "$D" "$SHA" >/dev/null 2>&1
fi
[ "$(git -C $D rev-parse HEAD)" = "$(git -C /home/hyunlord/work/new-world-wm1 rev-parse $SHA)" ] || { echo "worktree $D not at $SHA" >&2; exit 9; }
echo "$D"
