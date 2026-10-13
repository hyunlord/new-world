# 사망 원인 합성 캘리브레이션

`scripts/test/art/capture_deaths_p7.gd`는 실제 `death_viz_renderer.gd`, `hud_topbar.gd`, `hud_status_panel.gd`를 그대로 인스턴스화한다. `SnapshotWorld`가 tick100의 다섯 사망 원인과 원인별 tally1만 제공한다. 실제 시뮬레이션을 만들거나 수정하지 않는다.

1920×1080 PNG의 상단과 창 제목에 `SYNTHETIC CALIBRATION / NOT NATURAL SIMULATION EVIDENCE`를 표시한다. 실제 크기의 마커 5개, 원인별 색상 코드, 실제 상단 합계5, 실제 알림 최대4줄, 별도 진단용 전체 feed log5줄을 함께 보여준다. 전체 로그 목록은 `get_feed_log()`의 실제 결과를 Locale로 번역한 표시이다.

출력은 `--out` 디렉터리의 `calibration_deaths_p7.png`와 `.json`이다. JSON에는 `natural=false`, `simulation_instantiated=false`, PNG SHA256, 실행 시 전달한 source SHA, 생산 스크립트 경로, 실제 `_draw()`가 기록한 마커, 실제 표시 알림·전체 로그·합계가 들어간다. 원인 순서·마커5·표시4·로그5·합계5·viewport 크기를 검사하여 실패 시 exit1로 종료한다. headless 실행은 거부하며 headless `--check-only`만 허용한다.

첫 파싱 검사에서 스크립트 상수의 `resource_path` 조회가 지원되지 않아 오류가 났다. Godot 종료 코드는 0이었지만 SCRIPT ERROR를 확인하여 실패로 판정했다 (`death-calibration-parse-attempt1.log`). metadata의 출처를 명시적 resource 경로로 바꾸고 재검사한다. 창 실행·화면 검증은 리더의 window lock 실행 후 별도로 기록한다.

수정 후 재검사: exit0, SCRIPT ERROR 없음 (`death-calibration-parse.log`). `git diff --check` 통과.

리더 실행 명령(저장소 루트, `launchctl asuser` 배경 세션 내부의 lock):

```bash
WORLDSIM_GIT_SHA=$(git rev-parse HEAD) bash tools/art_pipeline/with_window_lock.sh 90 /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path . --script scripts/test/art/capture_deaths_p7.gd -- --out /Users/rexxa/work/new-world-art-prod/docs/art/production7/calibration
```

이 화면은 다섯 색상·HUD 연결의 통제된 증거이다. 자연 시나리오 사망 빈도, A26 자연 프레임 검사, 실제 시뮬레이션에서 다섯 원인이 관측되었다는 증거로 사용하지 않는다.
