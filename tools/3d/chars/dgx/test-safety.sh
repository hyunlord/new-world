#!/usr/bin/env bash
# Mock telemetry/container lifecycle; never connects to Docker or the GPU.
set -euo pipefail
script=$(cd "$(dirname "$0")" && pwd)/generate-safe.sh
sandbox=$(mktemp -d)
trap 'rm -rf "$sandbox"' EXIT
mkdir -p "$sandbox/bin" "$sandbox/worldsim-3d"
cat > "$sandbox/bin/nvidia-smi" <<'MOCK'
#!/bin/bash
count=0
[ ! -f "$MOCK_ROOT/count" ] || count=$(cat "$MOCK_ROOT/count")
echo $((count + 1)) > "$MOCK_ROOT/count"
case "$MOCK_CASE:$count" in
  wait:0) echo 70 ;;
  thermal:0|failed:0|hung:0|hup:0) echo 60 ;;
  failed:1) exit 1 ;;
  thermal:1) echo 80 ;;
  *) echo 60 ;;
esac
MOCK
cat > "$sandbox/bin/cut" <<'MOCK'
#!/bin/bash
echo 1.0
MOCK
cat > "$sandbox/bin/docker" <<'MOCK'
#!/bin/bash
echo "$*" >> "$MOCK_ROOT/docker.log"
if [ "$1" = run ]; then
  while [ "$1" != --cidfile ]; do shift; done
  echo owned-test-cid > "$2"
  basename "$2" .cid > "$MOCK_ROOT/name"
  if [ "$MOCK_CASE" = hup ]; then kill -HUP "$PPID"; fi
  if [ "$MOCK_CASE" = orphan ]; then exit 0; fi
  /bin/sleep 0.2
  touch "$MOCK_ROOT/removed"
elif [ "$1" = inspect ]; then
  [ ! -f "$MOCK_ROOT/removed" ] || exit 1
  echo "/$(cat "$MOCK_ROOT/name")|/workspace|$PWD|true"
elif [ "$1" = stop ]; then
  touch "$MOCK_ROOT/removed"
fi
MOCK
cat > "$sandbox/bin/sleep" <<'MOCK'
#!/bin/bash
echo "$*" >> "$MOCK_ROOT/sleep.log"
/bin/sleep 0.01
MOCK
cat > "$sandbox/bin/flock" <<'MOCK'
#!/bin/bash
exit 0
MOCK
cat > "$sandbox/bin/timeout" <<'MOCK'
#!/bin/bash
shift
if [ "$MOCK_CASE" = hung ] && [ -f "$MOCK_ROOT/count" ] && [ ! -f "$MOCK_ROOT/timed-out" ]; then touch "$MOCK_ROOT/timed-out"; exit 124; fi
exec "$@"
MOCK
chmod +x "$sandbox/bin/"*
sed 's|cd "$HOME/worldsim-3d"|cd "$MOCK_ROOT/worldsim-3d"|' "$script" > "$sandbox/generate-safe.sh"
script="$sandbox/generate-safe.sh"
export MOCK_ROOT="$sandbox" PATH="$sandbox/bin:$PATH"
for scenario in wait thermal failed hung hup orphan; do
  export MOCK_CASE="$scenario"
  rm -f "$sandbox/count" "$sandbox/docker.log" "$sandbox/sleep.log" "$sandbox/timed-out" "$sandbox/removed"
  status=0
  bash "$script" true > "$sandbox/output" || status=$?
  if [ "$scenario" = wait ]; then
    [ "$status" -eq 0 ]
    grep -qx 60 "$sandbox/sleep.log"
  else
    if [ "$scenario" = hup ]; then [ "$status" -eq 129 ]; elif [ "$scenario" = orphan ]; then [ "$status" -eq 0 ]; else [ "$status" -eq 80 ]; fi
    grep -q '^stop --time 10 owned-test-cid$' "$sandbox/docker.log"
  fi
  grep -qx 30 "$sandbox/sleep.log"
done
echo 'PASS: admission waits, thermal stop uses exact CID, cooldown occurs'
