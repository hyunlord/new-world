#!/usr/bin/env bash
# Exercise launcher safety against isolated Git fixtures and fake Godot/cargo.
# No game binaries, network, Desktop paths, or owner play worktrees are used.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
SOURCE="${1:-$ROOT/tools/play.sh}"
SANDBOX="$(mktemp -d "${TMPDIR:-/tmp}/worldsim-launcher-test.XXXXXX")"
trap 'rm -rf "$SANDBOX"' EXIT
REAL_GIT="$(command -v git)"
export REAL_GIT
mkdir -p "$SANDBOX/bin"
cat > "$SANDBOX/bin/git" <<'GIT'
#!/usr/bin/env bash
for arg in "$@"; do
    if [[ "$arg" == fetch && "${FAKE_OFFLINE:-0}" == 1 ]]; then exit 128; fi
done
exec "$REAL_GIT" "$@"
GIT
cat > "$SANDBOX/bin/Godot" <<'GODOT'
#!/usr/bin/env bash
if [[ "${1:-}" == --version ]]; then echo 4.6.stable; exit 0; fi
printf '%s\n' "$*" >> "$FAKE_CALLS"
case " $* " in
    *' --import '*)
        mkdir -p .godot
        [[ "${FAKE_ERROR:-}" != import ]] || echo 'ERROR: injected import failure'
        if [[ "${FAKE_ERROR:-}" == long_import ]]; then
            awk 'BEGIN { print "ERROR: injected long import failure"; for (i = 0; i < 40000; i++) print "filler" }'
        fi ;;
    *) [[ "${FAKE_ERROR:-}" != headless ]] || echo 'ERROR: injected runtime failure' ;;
esac
exit 0
GODOT
cat > "$SANDBOX/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$FAKE_CARGO_CALLS"
mkdir -p target/debug
: > target/debug/libsim_bridge.dylib
CARGO
chmod +x "$SANDBOX/bin/git" "$SANDBOX/bin/Godot" "$SANDBOX/bin/cargo"
export PATH="$SANDBOX/bin:$PATH"
export GODOT_BIN="$SANDBOX/bin/Godot"
failures=0
checks=0

fixture() {
    local label="$1"
    BASE="$SANDBOX/$label"
    REPO="$BASE/repo"
    PLAY_ROOT="$BASE/play"
    PLAY_DATA="$BASE/data"
    FAKE_CALLS="$BASE/calls"
    FAKE_CARGO_CALLS="$BASE/cargo-calls"
    export PLAY_ROOT PLAY_DATA FAKE_CALLS FAKE_CARGO_CALLS
    unset FAKE_OFFLINE FAKE_ERROR
    mkdir -p "$REPO/tools/v8" "$REPO/rust" "$REPO/scripts" "$REPO/assets"
    cp "$SOURCE" "$REPO/tools/play.sh"
    cp "$ROOT/tools/v8/run_bounded.sh" "$REPO/tools/v8/run_bounded.sh"
    printf '.godot/\nrust/target/\n' > "$REPO/.gitignore"
    printf 'macos.debug = "res://rust/target/debug/libsim_bridge.dylib"\n' > "$REPO/sim_bridge.gdextension"
    printf 'fixture\n' > "$REPO/README.md"
    : > "$REPO/project.godot"
    : > "$REPO/rust/Cargo.toml"
    printf '(value: 1)\n' > "$REPO/rust/data.ron"
    git init -q "$REPO"
    git -C "$REPO" config user.name 'Launcher test'
    git -C "$REPO" config user.email launcher-test@example.invalid
    git -C "$REPO" add tools .gitignore sim_bridge.gdextension README.md project.godot rust/Cargo.toml rust/data.ron
    git -C "$REPO" commit -qm fixture
    git -C "$REPO" branch -M v8/main
    git clone -q --bare "$REPO" "$BASE/origin.git"
    git -C "$REPO" remote add origin "$BASE/origin.git"
    git -C "$REPO" fetch -q origin
    mkdir -p "$PLAY_ROOT"
    git -C "$REPO" worktree add -q --detach "$PLAY_ROOT/main" HEAD
    BEFORE="$(git -C "$PLAY_ROOT/main" rev-parse HEAD)"
    : > "$FAKE_CALLS"
    : > "$FAKE_CARGO_CALLS"
}
run_play() {
    set +e
    /bin/bash "$REPO/tools/play.sh" "$@" > "$BASE/output" 2>&1
    CODE=$?
    set -e
}
check() {
    local label="$1"; shift
    checks=$((checks + 1))
    if "$@"; then
        echo "PASS $label"
    else
        echo "FAIL $label (launcher exit $CODE)"
        sed -n '1,20p' "$BASE/output"
        failures=$((failures + 1))
    fi
}
unchanged_head() { [[ "$(git -C "$PLAY_ROOT/main" rev-parse HEAD)" == "$BEFORE" ]]; }
no_engine() { [[ ! -s "$FAKE_CALLS" ]]; }
only_imports() { [[ -s "$FAKE_CALLS" ]] && ! grep -v -- '--import' "$FAKE_CALLS"; }
offline_logged() { grep -q 'fetch 실패' "$PLAY_DATA"/logs/*.log; }

fixture clean
run_play main --headless-check
check 'clean registered worktree succeeds' test "$CODE" -eq 0
check 'clean reuse preserves HEAD' unchanged_head
check 'clean run reaches engine' test -s "$FAKE_CALLS"

fixture update_only
run_play main --update-only
check 'update-only succeeds' test "$CODE" -eq 0
check 'update-only builds bridge' test -f "$PLAY_ROOT/main/rust/target/debug/libsim_bridge.dylib"
check 'update-only imports resources' test -f "$PLAY_ROOT/main/.godot/play_import.stamp"
check 'update-only never launches game' only_imports

fixture incompatible_modes
run_play main --headless-check --update-only
check 'incompatible modes rejected as arguments' test "$CODE" -eq 4
check 'incompatible modes never launch engine' no_engine

fixture changed_data
run_play main --update-only
check 'data-change fixture prepares successfully' test "$CODE" -eq 0
: > "$FAKE_CARGO_CALLS"
sleep 1
printf '(value: 2)\n' > "$REPO/rust/data.ron"
git -C "$REPO" add rust/data.ron
git -C "$REPO" commit -qm 'Change embedded data only'
git -C "$REPO" push -q origin v8/main
run_play main --update-only
check 'data-only update prepares successfully' test "$CODE" -eq 0
check 'changed embedded data invokes cargo' test -s "$FAKE_CARGO_CALLS"
check 'data-only update reaches new commit' test "$(git -C "$PLAY_ROOT/main" rev-parse HEAD)" = "$(git -C "$REPO" rev-parse HEAD)"

fixture deleted_data
run_play main --update-only
check 'data-deletion fixture prepares successfully' test "$CODE" -eq 0
: > "$FAKE_CARGO_CALLS"
git -C "$REPO" rm -q rust/data.ron
git -C "$REPO" commit -qm 'Remove embedded data only'
git -C "$REPO" push -q origin v8/main
run_play main --update-only
check 'data-deletion update prepares successfully' test "$CODE" -eq 0
check 'deleted embedded data invokes cargo' test -s "$FAKE_CARGO_CALLS"
check 'data-deletion reaches new commit' test "$(git -C "$PLAY_ROOT/main" rev-parse HEAD)" = "$(git -C "$REPO" rev-parse HEAD)"

fixture tracked
printf 'owner edit\n' >> "$PLAY_ROOT/main/README.md"
cp "$PLAY_ROOT/main/README.md" "$BASE/expected"
run_play main --headless-check
check 'tracked dirty worktree refused at unchanged HEAD' test "$CODE" -eq 1
check 'tracked dirty bytes preserved' cmp -s "$BASE/expected" "$PLAY_ROOT/main/README.md"
check 'tracked dirty HEAD preserved' unchanged_head
check 'tracked dirty never launches' no_engine

fixture untracked
printf 'owner file\n' > "$PLAY_ROOT/main/owner-note.txt"
cp "$PLAY_ROOT/main/owner-note.txt" "$BASE/expected"
run_play main --headless-check
check 'untracked dirty worktree refused' test "$CODE" -eq 1
check 'untracked bytes preserved' cmp -s "$BASE/expected" "$PLAY_ROOT/main/owner-note.txt"
check 'untracked dirty never launches' no_engine

fixture offline
export FAKE_OFFLINE=1
run_play main --headless-check
check 'offline reuses last fetched version' test "$CODE" -eq 0
check 'offline warns about fetch failure' grep -q 'fetch 실패' "$BASE/output"
check 'offline warning persisted in logfile' offline_logged
check 'offline HEAD preserved' unchanged_head

fixture foreign
# A clone can have exactly the requested commit but is not this repo's worktree.
git -C "$REPO" worktree remove "$PLAY_ROOT/main"
git clone -q "$BASE/origin.git" "$PLAY_ROOT/main"
run_play main --headless-check
check 'foreign Git directory refused even at same commit' test "$CODE" -eq 1
check 'foreign clone HEAD preserved' unchanged_head
check 'foreign clone never launches' no_engine

fixture divergence
printf 'local commit\n' >> "$PLAY_ROOT/main/README.md"
git -C "$PLAY_ROOT/main" add README.md
git -C "$PLAY_ROOT/main" commit -qm 'local play commit'
BEFORE="$(git -C "$PLAY_ROOT/main" rev-parse HEAD)"
run_play main --headless-check
check 'non-fast-forward branch update refused' test "$CODE" -eq 1
check 'non-fast-forward local HEAD preserved' unchanged_head
check 'non-fast-forward never launches' no_engine

fixture import_error
export FAKE_ERROR=import
run_play main --headless-check
check 'import ERROR with exit zero is failure' test "$CODE" -eq 1
check 'failed import does not write success stamp' test ! -e "$PLAY_ROOT/main/.godot/play_import.stamp"

fixture long_import_error
export FAKE_ERROR=long_import
run_play main --headless-check
check 'early import ERROR in long output is failure' test "$CODE" -eq 1
check 'long failed import does not write success stamp' test ! -e "$PLAY_ROOT/main/.godot/play_import.stamp"

fixture runtime_error
export FAKE_ERROR=headless
run_play main --headless-check
check 'runtime ERROR with exit zero is failure' test "$CODE" -eq 1

fixture window_error
export FAKE_ERROR=headless
run_play main
check 'windowed ERROR with exit zero is failure' test "$CODE" -eq 1

for name in . ..; do
    fixture "invalid-${#name}"
    run_play "$name" --headless-check
    check "unsafe name $name rejected as argument" test "$CODE" -eq 4
    check "unsafe name $name never launches" no_engine
done
printf 'LAUNCHER_TEST_DONE checks=%s failures=%s\n' "$checks" "$failures"
(( failures == 0 ))
