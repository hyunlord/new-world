#!/bin/bash
# sync.sh: mirror the perf2 worktree to the DGX (no .git, target, caches).
rsync -a --delete --exclude .git --exclude rust/target --exclude graft --exclude .omc --exclude .godot --exclude '.claude/settings*' \
  /Users/rexxa/work/new-world-perf2/ hyunlord@100.70.109.50:worldsim-3d/perf2/repo/
