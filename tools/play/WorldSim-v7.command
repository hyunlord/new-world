#!/bin/bash
# tools/play/WorldSim-v7.command — double-click on macOS to play WorldSim: V7 게임(v7-final-art, 비교용).
# Runs tools/play.sh v7 from the clone this file lives in (also through a ~/Desktop symlink).
# On a non-zero exit the Terminal window stays open until a key is pressed.
src="${BASH_SOURCE[0]}"
if link="$(readlink "$src")"; then
    case "$link" in /*) src="$link" ;; *) src="$(dirname "$src")/$link" ;; esac
fi
here="$(cd "$(dirname "$src")" && pwd -P)" || exit 1
cd "$here/../.." || exit 1
/bin/bash tools/play.sh v7 "$@"
code=$?
if [ $code -ne 0 ]; then
    echo
    echo "WorldSim을 실행하지 못했습니다 (exit $code). 위의 메시지(있으면 로그 경로)를 확인하세요."
    echo "아무 키나 누르면 끝납니다 (그다음 터미널 창은 ⌘W로 닫으세요)."
    read -n 1 -s
fi
exit $code
