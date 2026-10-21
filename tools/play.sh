#!/usr/bin/env bash
# tools/play.sh — play any version of WorldSim with one command (V8-PLAY goal 2.1).
#
# usage: tools/play.sh [version] [--seed N] [--speed 1-4] [--no-build] [--headless-check] [-- godot args…]
#   version: main (default, = v8/main), wip, m1, v8-0, v7, or any commit/tag/branch
#
# 1. Godot: $GODOT_BIN if set; else the path in tools/v8/godot_bin.sh; else Spotlight
#    (org.godotengine.godot); else /Applications/Godot.app. None → exit 2.
# 2. The version gets its own worktree under $PLAY_ROOT/<name> (git worktree add --detach from
#    this clone, after a fetch); an existing one is reused and fast-forwarded when the version is
#    a branch. This checkout itself is never used for playing.
# 3. Build sim-bridge in that worktree when a Rust source, Cargo.toml or Cargo.lock is newer than
#    the library the .gdextension makes the Godot editor binary load (macos.debug → debug build).
# 4. Import headless when .godot/ is missing (twice) or older than assets/ scripts/ scenes/ shaders/.
# 5. Launch windowed at 1920x1080 (--seed/--speed only when the version's scripts read them).
# 6. Log to $PLAY_DATA/logs/<version>-<timestamp>.log.
# 7. --headless-check: steps 1-4, then a 300-frame headless run; exit 0 only with 0 SCRIPT ERROR.
# Exit codes: 0 ok, 1 build/import/smoke failed, worktree problem or Godot crashed, 2 Godot not found,
#             3 unknown version, 4 bad arguments.
# Environment: GODOT_BIN, PLAY_ROOT (default /Users/rexxa/work/new-world-play),
#              PLAY_DATA (default /Users/rexxa/work/play-data).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd -P)"
PLAY_ROOT="${PLAY_ROOT:-/Users/rexxa/work/new-world-play}"
PLAY_DATA="${PLAY_DATA:-/Users/rexxa/work/play-data}"
BOUNDED="$REPO/tools/v8/run_bounded.sh"
export PATH="$PATH:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin"

usage() {
    sed -n '4,5p' "$0" | sed 's/^# \{0,1\}//'
}

version="main"
seed=""
speed=""
build=1
headless=0
version_set=0
extra=()
while (( $# > 0 )); do
    case "$1" in
        -h|--help) usage; exit 0 ;;
        --seed)
            [[ "${2:-}" =~ ^[0-9]+$ ]] || { echo "play: --seed는 0 이상의 정수여야 합니다" >&2; usage >&2; exit 4; }
            seed="$2"; shift 2 ;;
        --speed)
            [[ "${2:-}" =~ ^[1-4]$ ]] || { echo "play: --speed는 1-4여야 합니다" >&2; usage >&2; exit 4; }
            speed="$2"; shift 2 ;;
        --no-build) build=0; shift ;;
        --headless-check) headless=1; shift ;;
        --) shift; extra=("$@"); break ;;
        -*) echo "play: 모르는 옵션 $1" >&2; usage >&2; exit 4 ;;
        *)
            (( version_set == 0 )) || { echo "play: 버전은 하나만 줄 수 있습니다 ($version, $1)" >&2; exit 4; }
            version="$1"; version_set=1; shift ;;
    esac
done

# --- 1. Godot -----------------------------------------------------------------------------------
resolve_godot() {
    if [[ -n "${GODOT_BIN:-}" ]]; then
        [[ -x "$GODOT_BIN" ]] && { echo "$GODOT_BIN"; return 0; }
        return 1
    fi
    local candidate app
    candidate="$(sed -n 's/.*GODOT_BIN:-\([^}]*\)}.*/\1/p' "$REPO/tools/v8/godot_bin.sh" 2>/dev/null | head -1)"
    [[ -n "$candidate" && -x "$candidate" ]] && { echo "$candidate"; return 0; }
    if command -v mdfind > /dev/null 2>&1; then
        while IFS= read -r app; do
            [[ -x "$app/Contents/MacOS/Godot" ]] && { echo "$app/Contents/MacOS/Godot"; return 0; }
        done < <(mdfind "kMDItemCFBundleIdentifier == 'org.godotengine.godot'" 2>/dev/null)
    fi
    [[ -x /Applications/Godot.app/Contents/MacOS/Godot ]] && { echo /Applications/Godot.app/Contents/MacOS/Godot; return 0; }
    return 1
}
if ! GODOT="$(resolve_godot)"; then
    echo "play: Godot 4.6을 찾지 못했습니다 — Godot.app을 /Applications 또는 ~/Downloads에 두거나 GODOT_BIN=<.../Godot.app/Contents/MacOS/Godot>로 알려 주세요 (GODOT_BIN=${GODOT_BIN:-없음})" >&2
    exit 2
fi

# --- 2. Version → worktree ----------------------------------------------------------------------
git -C "$REPO" fetch -q origin 2>/dev/null || echo "play: git fetch 실패 — 마지막으로 받아 둔 원격 상태를 씁니다" >&2
branch=""
case "$version" in
    main) target="origin/v8/main"; branch="$target"; name="main" ;;
    wip) target="origin/v8/wip"; branch="$target"; name="wip" ;;
    m1) target="892edca1"; name="m1" ;;
    v8-0) target="24e16677"; name="v8-0" ;;
    v7) target="v7-final-art"; name="v7" ;;
    *)
        name="$(printf '%s' "$version" | tr '/: ' '---')"
        target="$version"
        if git -C "$REPO" rev-parse -q --verify "refs/remotes/origin/$version^{commit}" > /dev/null 2>&1; then
            target="origin/$version"; branch="$target"
        fi ;;
esac
commit="$(git -C "$REPO" rev-parse -q --verify "$target^{commit}" 2>/dev/null)" || commit=""
if [[ -z "$commit" ]]; then
    echo "play: 모르는 버전 '$version' — main, wip, m1, v8-0, v7 또는 커밋/태그/브랜치를 주세요" >&2
    exit 3
fi
short="$(git -C "$REPO" rev-parse --short "$commit")"

mkdir -p "$PLAY_DATA/logs" "$PLAY_ROOT" || { echo "play: $PLAY_DATA 또는 ${PLAY_ROOT}를 만들 수 없습니다" >&2; exit 1; }
LOG="$PLAY_DATA/logs/$name-$(date '+%Y%m%d-%H%M%S').log"
: > "$LOG"
say() { echo "$*" | tee -a "$LOG"; }
say "play: 로그 $LOG"
{ echo "[play] version $version → $target = $commit"; echo "[play] godot $GODOT"; } >> "$LOG"

WT="$PLAY_ROOT/$name"
fresh=0
if [[ -e "$WT/.git" ]]; then
    head="$(git -C "$WT" rev-parse HEAD 2>/dev/null)"
    if [[ "$head" != "$commit" ]]; then
        if [[ -n "$branch" ]] && ! git -C "$WT" merge-base --is-ancestor "$head" "$commit" 2>/dev/null; then
            say "play: ${WT}의 ${head}는 ${branch}의 조상이 아닙니다 — 그대로 ${commit}으로 옮깁니다"
        fi
        git -C "$WT" checkout -q --detach "$commit" >> "$LOG" 2>&1 || { say "play: ${WT}를 ${short}로 옮기지 못했습니다 (로컬 수정이 있나요?)"; exit 1; }
        say "play: 작업 트리 $WT → $short"
    fi
elif [[ -e "$WT" ]]; then
    say "play: ${WT}가 있지만 git 작업 트리가 아닙니다 — 지우거나 PLAY_ROOT를 바꿔 주세요"
    exit 1
else
    git -C "$REPO" worktree add -q --detach "$WT" "$commit" >> "$LOG" 2>&1 || { say "play: git worktree add 실패 ($WT)"; exit 1; }
    fresh=1
    say "play: 새 작업 트리 $WT ($short)"
fi
# rust/Cargo.lock is gitignored: give a new worktree this clone's lock so it does not resolve newer crates.
if [[ ! -f "$WT/rust/Cargo.lock" && -f "$REPO/rust/Cargo.lock" ]]; then
    cp "$REPO/rust/Cargo.lock" "$WT/rust/Cargo.lock"
fi
cd "$WT" || exit 1

# --- 3. Build -----------------------------------------------------------------------------------
lib_path="$(sed -n 's/^macos\.debug *= *"res:\/\/\(.*\)"/\1/p' sim_bridge.gdextension 2>/dev/null | head -1)"
case "$lib_path" in
    */target/debug/*) profile=debug; cargo_flags=() ;;
    */target/release/*) profile=release; cargo_flags=(--release) ;;
    *) say "play: sim_bridge.gdextension에서 macos.debug 라이브러리를 읽지 못했습니다"; exit 1 ;;
esac
build_state="생략"
build_secs=0
needs_build=0
if [[ ! -f "$lib_path" ]]; then
    needs_build=1
elif [[ -n "$(find rust -path rust/target -prune -o -type f \( -name '*.rs' -o -name Cargo.toml -o -name Cargo.lock \) -newer "$lib_path" -print 2>/dev/null | head -1)" ]]; then
    needs_build=1
fi
if (( build == 0 )); then
    build_state="생략(--no-build)"
    [[ -f "$lib_path" ]] || { say "play: --no-build인데 ${lib_path}가 없습니다"; exit 1; }
elif (( needs_build == 1 )); then
    [[ "$profile" == debug ]] && say "play: Godot 편집기 실행 파일은 .gdextension의 macos.debug($lib_path)를 읽으므로 release가 아니라 debug로 빌드합니다"
    say "play: cargo build -p sim-bridge ${cargo_flags[*]+${cargo_flags[*]}} …"
    started=$(date +%s)
    (cd rust && cargo build -p sim-bridge ${cargo_flags[@]+"${cargo_flags[@]}"}) >> "$LOG" 2>&1 || { say "play: 빌드 실패 — $LOG"; exit 1; }
    build_secs=$(( $(date +%s) - started ))
    build_state="완료(${profile}, ${build_secs}초)"
else
    say "play: 빌드 생략 — ${lib_path}가 Rust 소스보다 새것입니다"
fi

# --- 4. Import ----------------------------------------------------------------------------------
STAMP=".godot/play_import.stamp"
import_state="생략"
runs=0
if [[ ! -d .godot ]]; then
    runs=2
elif [[ ! -f "$STAMP" ]]; then
    runs=1
else
    dirs=()
    for d in assets scripts scenes shaders; do [[ -d "$d" ]] && dirs+=("$d"); done
    if [[ -n "$(find ${dirs[@]+"${dirs[@]}"} project.godot sim_bridge.gdextension -newer "$STAMP" -print 2>/dev/null | head -1)" ]]; then
        runs=1
    fi
fi
if (( runs > 0 )); then
    started=$(date +%s)
    for (( i = 1; i <= runs; i++ )); do
        say "play: 가져오기(import) $i/$runs …"
        out="$(bash "$BOUNDED" 600 "$GODOT" --headless --path . --import --editor --quit 2>&1)"
        code=$?
        printf '%s\n' "$out" >> "$LOG"
    done
    # A fresh tree's first import logs font errors; only the last run counts.
    if (( code != 0 )) || printf '%s\n' "$out" | grep -q 'SCRIPT ERROR'; then
        say "play: 가져오기 실패(exit ${code}, SCRIPT ERROR $(printf '%s\n' "$out" | grep -c 'SCRIPT ERROR')줄) — ${LOG}"
        exit 1
    fi
    touch "$STAMP"
    import_state="완료(${runs}회, $(( $(date +%s) - started ))초)"
else
    say "play: 가져오기 생략 — .godot/가 최신입니다"
fi

# --- 5./7. Launch or headless check -------------------------------------------------------------
reads_arg() { grep -rqs -e "$1" --include='*.gd' --exclude-dir=test scripts; }
godot_user=()
if [[ -n "$seed" ]]; then
    if reads_arg '"--seed"'; then godot_user+=(--seed "$seed"); else say "play: 이 버전은 --seed를 지원하지 않습니다 (무시)"; seed="$seed(무시)"; fi
fi
if [[ -n "$speed" ]]; then
    if reads_arg '"--speed"'; then godot_user+=(--speed "$speed"); else say "play: 이 버전은 --speed를 지원하지 않습니다 (무시)"; speed="$speed(무시)"; fi
fi
if (( headless == 1 )); then
    args=(--headless --path . --quit-after 300)
else
    args=(--path . --resolution 1920x1080)
fi
args+=(${extra[@]+"${extra[@]}"})
(( ${#godot_user[@]} > 0 )) && args+=(-- "${godot_user[@]}")

say "play: 버전 $version · 커밋 $short · 빌드 $build_state · 가져오기 $import_state · 시드 ${seed:-기본} · 속도 ${speed:-기본}"
if (( headless == 1 )); then
    say "play: 헤드리스 확인 — 300프레임"
    out="$(bash "$BOUNDED" 600 "$GODOT" "${args[@]}" 2>&1)"
    code=$?
    printf '%s\n' "$out" >> "$LOG"
    errors=$(printf '%s\n' "$out" | grep -c 'SCRIPT ERROR')
    say "play: 헤드리스 확인 exit $code, SCRIPT ERROR ${errors}줄"
    (( code == 0 && errors == 0 )) && { say "play: 헤드리스 확인 통과"; exit 0; }
    say "play: 헤드리스 확인 실패 — $LOG"
    exit 1
fi
say "play: 게임 창을 엽니다 (창을 닫으면 끝납니다)"
echo "[play] launch $(date '+%H:%M:%S') $GODOT ${args[*]}" >> "$LOG"
"$GODOT" "${args[@]}" >> "$LOG" 2>&1
code=$?
say "play: 게임 종료 (exit $code)"
(( code == 0 )) || exit 1
exit 0
