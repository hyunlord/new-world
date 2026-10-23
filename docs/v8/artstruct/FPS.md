# ART-STRUCT Crowd FPS

## 상태

**PASS — 실제 창 완료 렌더 FPS 304.108106023482 ≥60.** 2026-10-23 최종 동결 입력으로 단일60초 실행을 완료했다. 기술 성능 판정이며 미관·오너 승인·실제 게임 simulation 성능을 뜻하지 않는다.

| 항목 | 실제 관측 |
|---|---|
| 기간 | 60.001689초 |
| 완료 렌더 / 엔진 카운터 | 18,247 / 18,247 |
| 사람 / 환경 / 벽 방 chunk | 200 / 600 / 3 |
| 실제 STRUCT 환경 종류 | 27 |
| 유효성 | invalid0, 매초60표본 전부200/600/3 |
| 입력 SHA | 423개 시작/종료 및 종료 뒤 재검사 일치 |
| Godot / renderer / GPU | 4.6.stable.official.89cea1439 / Compatibility OpenGL4.1 Metal91.7 / Apple M4 Max |
| vsync / cap | 0 / 0 |
| 실행 종료 | exit0, SCRIPT ERROR/Parse Error/ERROR 없음 |

필수 main asset manifest의 환경만 사용했으며 별도 optional item replacements와 늑대 재시도 시트는 포함하지 않았다. 600환경에는 생성된 문/작업장/바위/나무 등의 스프라이트가 반복된다. 3개 room chunk는 별도 추가 부하다.

## 고정 측정 계약

- 별도 Godot 4.6 art 프로젝트 `art/proto/struct/`; 게임 코드·simulation·GDExtension 없음.
- 1600×900 실제 창, 1×, nearest, vsync off, 프레임 제한0.
- Route I 채택34룩의340프레임에서 사람200명(8Hz 프레임 교체), 실제 STRUCT 환경 sprite600개, 코드 벽 방3개, ground.
- 벽은 `art/compositor/walls.py` 결과를 **방 단위 chunk로 사전 bake**한다. Godot은3개 chunk를 그린다. bake 비용은60초 런타임 FPS에 포함하지 않는다.
- warm-up2초 뒤 `RenderingServer.frame_post_draw` 완료 프레임 수 ÷ 실제60초 이상 구간. `Engine.get_frames_drawn()` 차이와 ±1 이내 대조한다. `_process` 횟수는 FPS가 아니다.
- 매 process에서 창 표시·최소화 여부·그릴 수 있음·viewport1600×900·사람200/환경600/방3의 전체 texture 경계가 viewport 안인지 검사한다. 겹친 sprite의 개별 비가림 픽셀을 보장하는 검사는 아니다.
- 모든 입력 PNG·채택manifest·STRUCT manifest·하네스·벽 bake source SHA를 시작에 기록하고 종료 시 재검사한다.
- 시작/종료 실제 viewport PNG, 매초 검사, 전체 프레임 간격, Godot/OS/GPU/renderer 정보는 외부 원시 디렉터리에 보존한다.
- 통과: 측정 유효 AND 평균 완료 렌더 FPS≥60. 물리 모니터 scanout이나 실제 게임 simulation 성능 인증은 아니다.

## 재현 명령

```bash
bash art/proto/struct/run.sh /Users/rexxa/work/artstruct-data/fps/final
```

기존 measurement가 있으면 덮어쓰지 않는다. 실패 시 새 명시 디렉터리로 재실행하고 모든 시도를 보존한다. 실제 실행은90초 bounded process다.

방 입력 `art/proto/struct/rooms.json`: `rooms` 정확히3개(`path`, `sha256`, `position`), `bake_inputs`는 비어 있지 않은 저장소 상대 경로→SHA 사전이다. 미존재·변경된 입력은 거부한다.

## 현재 검증 증거

- Godot4.6 headless import 및 `--check-only` parse 성공. 로그 `/Users/rexxa/work/artstruct-data/fps/{import,parse}.log`.
- 실제 headless 실행은 exit1로 `STRUCT_CROWD_NOT_VERIFIED real window required` 거부했다. 초기 거부 뒤 `_process`가 빈 배열을 읽던 문제는 `_ready` 첫 줄에서 process를 비활성화하고 정상 준비 완료 후 활성화하여 수정했다. 수정 전/후 로그 보존.
- 최종 실제 창 로그·PNG·measurement는 `/Users/rexxa/work/artstruct-data/fps/final/`에 보존했다. `frame_intervals_us` 개수18,247 및 합계60,001,689µs를 별도 재검사했다.
- `measurement.json` SHA256: `f7a384460316c8eff30e0f82b16e18b4ab99648f581afd5f56d1b6409fa571f6`.
- `final.png` SHA256: `d235d0275243d0846a78691eb4ea8a3cdbe482ebd849fd57716bca451bc43272`. 실제 viewport PNG1600×900은3.43MB이므로 파일 제한에 따라 원시 외부 위치에 보존한다.
- CoreGraphics가 PID16836의 on-screen 창을 확인했다(`os_window.json`, alpha1, on-screen true). Retina OS 창 bounds는800×482 logical points이고 Godot viewport는1600×900px다. 공유상태0 때문에 OS `screencapture`는 `could not create image from window`로 실패했다(`os_capture.log`). 실제 창의 Godot viewport capture는 성공했다. 물리 scanout·완전한 비가림·OS screenshot 획득을 주장하지 않는다.
- 단일 최종60초 실행이며 유리한 반복을 선택하지 않았다. 원시 시작/종료 viewport를 보존한다.
