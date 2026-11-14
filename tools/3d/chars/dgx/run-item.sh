#!/usr/bin/env bash
# Run one authorized A/B subject attempt; safe wrapper owns GPU admission/cooldown.
set -euo pipefail
[ "$#" -eq 3 ] || { echo 'usage: run-item.sh A|B SUBJECT 1|2|3' >&2; exit 2; }
case "$1" in A|B) ;; *) exit 2 ;; esac
case "$2" in man|woman|child|goat|aurochs) ;; *) exit 2 ;; esac
case "$3" in 1|2|3) ;; *) exit 2 ;; esac
cd "$HOME/worldsim-3d"
set_name=$1
subject=$2
attempt=$3
source="/workspace/concepts/$subject.png"
if [ "$set_name" = B ]; then source="/workspace/concepts/B/$subject.png"; fi
rgba="/workspace/rgba/$set_name/$subject.png"
output="/workspace/raw/$set_name/$subject-attempt$attempt.glb"
bash generate-safe.sh bash -c '
  if [ ! -s "$2" ]; then python /workspace/prepare_rgba.py "$1" "$2"; fi
  python /workspace/infer.py "$2" "$3"
' -- "$source" "$rgba" "$output"
