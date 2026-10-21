#!/bin/bash
# tools/play/WorldSim.command — double-click on macOS to play WorldSim: 지금 게임(v8/main).
# Runs tools/play.sh main from the clone this file lives in (also through a ~/Desktop symlink).
# On a non-zero exit the Terminal window stays open until a key is pressed.
src="${BASH_SOURCE[0]}"
if link="$(readlink "$src")"; then
    case "$link" in /*) src="$link" ;; *) src="$(dirname "$src")/$link" ;; esac
fi
here="$(cd "$(dirname "$src")" && pwd -P)" || exit 1
cd "$here/../.." || exit 1
/bin/bash tools/play.sh main "$@"
code=$?
if [ $code -ne 0 ]; then
    echo
    echo "WorldSim을 실행하지 못했습니다 (exit $code). 위의 메시지와 로그 경로를 확인하세요."
    echo "아무 키나 누르면 창이 닫힙니다."
    read -n 1 -s
fi
exit $code
