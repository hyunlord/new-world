# ENV1 실제 창 Crowd FPS

**PASS:** 최종 1× 340.984428377771 FPS, ½× LOD 263.182634332511 FPS.
각각 실제 `RenderingServer.frame_post_draw` 완료 프레임을 60초 이상 관측했다.
독립 아트 하네스 결과이며 실제 시뮬레이션 성능이나 M-ART2 통합 통과가 아니다.

| 배율 | 완료 프레임 | 경과 시간 | FPS | 사람 / 환경 / 고유 환경 입력 | 유효성 |
|---|---:|---:|---:|---|---|
| 1× | 20,460 | 60.002740s | 340.984428377771 | 200 / 400 / 75 | PASS |
| ½× LOD | 15,792 | 60.003959s | 263.182634332511 | 200 / 400 / 75 | PASS |

Godot 4.6 stable `89cea1439`, Compatibility, OpenGL 4.1 Metal, Apple M4 Max,
1600×900 viewport, nearest filtering, vsync off, frame cap 0, warm-up 2초.
200명은 채택된 34룩·340프레임을 읽어 8Hz로 순환한다. ½×에서는 10px 상의색
marker와 활동 glyph로 전환한다. Round1 도끼14×14·통나무14×8은 합성기의 같은 기하 그리기 함수에서 PNG로 출력해 사용하고 입력 해시에도 포함했다. 환경은 동결 manifest에 명시된 실제 자산만
읽으며 진단 점수표·보드 파일은 입력에서 제외한다. 두 실행 모두 invalid frame 0,
동일 입력 해시 유지, Engine 완료 프레임 대조 통과, 400개 환경/200개 사람 노드와
가시 viewport 경계 검사를 통과했다. Process FPS는 `NOT_MEASURED`다.

[원시 측정 해시·모든 입력 해시·설정·실행 이력](fps/results.json),
[1× 시작](fps/1x_start.png), [1× 종료](fps/1x_final.png),
[½× 시작](fps/half_start.png), [½× 종료](fps/half_final.png).
게시 이미지는 1.5MB 제한 때문에 전체 화면을 960×540으로 기하 축소했다.
1600×900 원본과 프레임 간격 원시 자료는
`/Users/rexxa/work/artenv1-data/crowd/serial_r1_{1x,half}/`에 그대로 보존했다.

macOS CoreGraphics는 두 Godot PID의 실제 on-screen 창과 alpha 1을 확인했다.
Retina 창 좌표는 800×482 point(장식 포함), 실제 viewport는 1600×900 pixel이다.
OS 창 공유 상태가 0이고 `screencapture`가 `could not create image from window`로
실패하여 OS 합성 화면 캡처는 `UNAVAILABLE`다. 게시 화면은 headless가 아닌
해당 실제 창의 Godot viewport 캡처다. 물리 패널 scanout·가림 없음까지 측정한
증거로 승격하지 않는다.

## 모든 실행과 수정

1. 첫 1×: 421.141233243328 FPS / 60.001249s, 유효 측정. 원시 자료 `crowd/1x/` 보존.
2. 첫 ½×: 점수표 PNG가 갱신되는 동안 디렉터리의 모든 PNG를 읽는 사전 단계에서
   `ERR_FILE_CORRUPT`로 종료했다. 60초 계측은 시작되지 않았으며 `PREFLIGHT_FAILED`다.
3. 입력 선택을 manifest 경로·해시 검증으로 고쳤다. 진단 그림이 부하 자산으로
   읽히던 원인을 제거했으며 그림·성능 기준은 변경하지 않았다.
4. Round0 최종 하네스는1×419.86659465176·½×299.005065270134 FPS였다. 기존 표식의 결과로 보존했다.
5. Round1 표식으로325.058610630767·229.189980578016 FPS를 측정했으나 첫 프로세스 종료 확인 전에 다음 창을 시작해 약2초 warm-up이 겹쳤다. 내부 유효성은 통과했지만 순차 실행 계약은 불확실하므로 두 결과를 그대로 보존했다.
6. 이전 프로세스 `wait()` 종료를 확인한 뒤 다음 창을 시작하는 동일 실행기로 두 배율을 다시 측정했다. 위 표는 이 최종 순차 쌍이며, 가장 빠른 결과를 고르지 않았다. 창·명령·PID·실제 시작/종료 UTC는 results.json에 있다.

실행 명령(저장 경로의 배율만 바꿈):

```bash
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto/env1 -- --zoom 1 --seconds 60 --out /Users/rexxa/work/artenv1-data/crowd/serial_r1_1x
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --path art/proto/env1 -- --zoom 0.5 --seconds 60 --out /Users/rexxa/work/artenv1-data/crowd/serial_r1_half
```

최종 순차 쌍에서는 동시 Godot/합성 행렬 실행이 없었다. 공유 호스트에서 짧은 CPU 전용 합성기 검증·일반 도구 실행은 있었으며,
특정 FPS만 골라 반복하지 않았다. 프레임 완료율과 시각적 가독성은 별도 판정이다.
