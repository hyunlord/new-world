#!/usr/bin/env bash
# Serialize GPU items; stop only the inspected owned container on heat or exit.
set -euo pipefail
cd "$HOME/worldsim-3d"
if [ "$#" -eq 0 ]; then echo 'usage: generate-safe.sh COMMAND [ARG...]' >&2; exit 2; fi
mkdir -p logs tmp
exec 9>generation.lock
flock -n 9 || { echo 'Another generation owns generation.lock' >&2; exit 3; }
while :; do
  temperature=$(timeout 5 nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits)
  [[ "$temperature" =~ ^[0-9]+$ ]] || { echo 'Invalid GPU temperature' >&2; exit 4; }
  load=$(cut -d ' ' -f 1 /proc/loadavg)
  [[ "$load" =~ ^[0-9]+([.][0-9]+)?$ ]] || { echo 'Invalid load' >&2; exit 4; }
  printf '%s admission temperature=%s load=%s\n' "$(date -Iseconds)" "$temperature" "$load"
  if awk -v t="$temperature" -v l="$load" 'BEGIN { exit !(t < 70 && l < 8) }'; then break; fi
  sleep 60
done
started=$(date +%s)
name="worldsim-m3d2-item-$$"
cidfile="$PWD/logs/$name.cid"
pid=""
thermal=0
stop_owned() {
  pid=${pid:-$(jobs -pr)}
  while [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null && [ ! -s "$cidfile" ]; do sleep 1; done
  if [ -s "$cidfile" ]; then
    cid=$(cat "$cidfile")
    # A removed --rm container is finished even if the client has already exited.
    identity=$(docker inspect --format '{{.Name}}|{{.Config.WorkingDir}}|{{range .Mounts}}{{if eq .Destination "/workspace"}}{{.Source}}{{end}}{{end}}|{{.State.Running}}' "$cid" 2>/dev/null) || return 0
    case "$identity" in
      "/$name|/workspace|$PWD|true") docker stop --time 10 "$cid" ;;
      "/$name|/workspace|$PWD|false") ;;
      *) echo 'Container identity mismatch; refusing signal' >&2; return 5 ;;
    esac
  fi
}
cleanup() {
  exit_status=$?
  stop_owned
  if [ -n "$pid" ]; then wait "$pid" || true; fi
  sleep 30
  if [ "$thermal" -eq 1 ]; then
    while :; do
      temperature=$(timeout 5 nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits) || return 4
      [[ "$temperature" =~ ^[0-9]+$ ]] || return 4
      [ "$temperature" -lt 70 ] && break
      sleep 60
    done
  fi
  printf '%s finished status=%s elapsed_seconds=%s\n' "$(date -Iseconds)" "$exit_status" "$(( $(date +%s) - started ))"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
docker run --rm --read-only --name "$name" --cidfile "$cidfile" --gpus all \
  --user "$(id -u):$(id -g)" -v "$PWD:/workspace" -v "$PWD/tmp:/tmp" -w /workspace \
  -e HOME=/workspace -e HF_HOME=/workspace/cache/huggingface \
  -e TORCH_HOME=/workspace/cache/torch -e TORCH_EXTENSIONS_DIR=/workspace/cache/torch_extensions \
  -e CUDA_CACHE_PATH=/workspace/cache/cuda -e TMPDIR=/workspace/tmp \
  -e PYTHONPATH=/workspace/TRELLIS.2 -e ATTN_BACKEND=sdpa \
  -e PATH=/workspace/venv/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  nvcr.io/nvidia/pytorch:25.11-py3@sha256:417cbf33f87b5378849df37983552cd1f8bc8b62fe1ceabe004de816a55dff21 "$@" &
pid=$!
while kill -0 "$pid" 2>/dev/null; do
  temperature=$(timeout 5 nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits) || temperature=80
  [[ "$temperature" =~ ^[0-9]+$ ]] || temperature=80
  printf '%s running temperature=%s\n' "$(date -Iseconds)" "$temperature"
  if [ "$temperature" -ge 80 ]; then thermal=1; stop_owned; break; fi
  sleep 2
done
status=0
wait "$pid" || status=$?
[ "$thermal" -eq 0 ] || status=80
exit "$status"
