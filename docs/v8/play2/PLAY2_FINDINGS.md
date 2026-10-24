# PLAY2 조사 기록

조사 시작: 2026-10-25 00:54 KST. 기준 `f0b759fe29d1f8c16f4ae6a994807036933b50df`.

## 플레이 루트

`/Users/rexxa/work/new-world-play/`의 항목은 아래 세 개뿐이다. 모두 이 클론에 등록된 detached worktree이며 초기 `git status --short`는 비었다. 삭제하지 않았다.

| 항목 | 최초 HEAD | 최초 용량 |
|---|---|---|
| main | d796fcacd23d03959e2bd1046f69581a9b1fc637 | 3.5G |
| m1 | 892edca1d31b26e535a3e9b62b37d62b99bb1130 | 1.4G |
| v7 | b1ca032f8fa84cee87d68a185e471b8e2c2bddce | 4.3G |

## 최초 실행

- `tools/play.sh main --headless-check`: exit 0, SCRIPT ERROR 0줄. main HEAD를 f0b759fe2로 갱신. debug 빌드 3초, import 1회 4초. 기존 실행기에 fetch/checkout 개별 타이머가 없어 해당 시간은 미측정. 로그 `/Users/rexxa/work/play-data/logs/main-20261025-005656.log`.
- Godot: `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`, 4.6.stable.official.89cea1439. `.gdextension`의 macos.debug는 `rust/target/debug/libsim_bridge.dylib`를 읽는다.
- 창 실행: `tools/play.sh main -- --quit-after 2400`, 1920×1080, Metal 4.0 Forward+, Apple M4 Max. exit 0. 빌드/import 모두 생략. 로그 `/Users/rexxa/work/play-data/logs/main-20261025-005722.log`.
- OS 화면 캡처 `screencapture`는 `could not create image from display`로 실패. System Events 조회도 -600으로 실패. 이 결과를 화면 확인 성공으로 취급하지 않는다. 아래 실제 viewport 캡처와 입력 하네스로 별도 검증했다.

## 실행기 격리 회귀 시험

`tools/play/test_launcher.sh`는 임시 Git 저장소와 가짜 Godot/cargo로 실행기의 분기를 검증한다. 실제 게임 구동 증거와 구별한다.

- 수정 전 26개 확인 중 14개 실패. 원본 로그 `/Users/rexxa/work/play-data/play2/regression-before.txt`.
- 같은 HEAD의 tracked/untracked 수정이 있어도 실행했다. 다른 클론의 `.git`도 수용했다. 원격과 갈라진 HEAD를 옮겼다. import/런타임의 `ERROR:` + exit 0도 통과했다.
- 네트워크 변경 없이 fetch 실패만 주입한 시험에서는 마지막 fetch 상태를 사용하며 경고했다. 최초 빌드에 필요한 패키지까지 오프라인으로 제공한다는 뜻은 아니다.

## 바탕화면

`~/Desktop/WorldSim.command`는 존재하며 `/Users/rexxa/work/new-world-pass4/tools/play/WorldSim.command`를 가리킨다. 이 경로에서 실행하거나 수정하지 말라는 최신 지시와 충돌하므로 `open`하지 않았다. Desktop에는 아무것도 쓰지 않았다. 새 실행기 연결은 오너용 정확한 한 줄 명령으로 제공한다.

## 범위와 증거

게임 소스와 아트는 수정하지 않는다. UI 조작은 프로그램으로 입력한 창 실행과 사람이 손으로 확인한 결과를 구별한다. 전체 키 근거는 KEYS.md. 전체 gate와 원격 게시 상태는 PLAY2_REPORT.md에서 최종 기록한다.

## 최종 창 입력 확인

두 번째 창 입력 시험은 79개 확인, 실패 0, 47.988초였다. 로그 `/Users/rexxa/work/play-data/logs/main-20261025-010419-94413.log`, JSON `/Users/rexxa/work/play-data/shots/play2/gui2/gui-check.json`. `launch-10s.png`, `launch-30s.png`에 실제 viewport를 캡처했다. 직접 열어 1× 새 아트, 초록 나무, 인물 초상, ¼× 마커와 F12 벽 윗면 밝기 변화를 확인했다.

| 항목 | 결과 | 증거 |
|---|---|---|
| 기본 1×·64px 타일·Route I 사람·초록 나무 | 확인 | 기본 줌 assertion, 10초/30초 창 캡처, View.TILE 소스 |
| Space 일시정지 | 확인 | 정지 전후 틱 동일 |
| 1–4 속도 | 확인 | 카메라 실제 배속 조회 |
| -/= 및 휠의 ¼×·½×·1×·2× | 확인 | 각 단계 실제 줌·HUD 문자열·캡처 |
| 1× 미만 인물 마커 | 확인 | 실제 LOD 상태·¼× 캡처 |
| 사람 클릭·128px 초상 | 확인 | 실제 클릭, 선택 ID·초상 경로·크기, 캡처 |
| C/T/N/P/R | 확인 | 패널/방 오버레이 열림·닫힘과 캡처 |
| D 징집·해제 | 확인 | 선택 인물의 실제 draft 표시 |
| F5/F9 | 확인 | 저장 생성, 틱 진행 뒤 저장 시점 복원; 임시 저장 제거 |
| F12 | 확인 | 벽 건설 후 top_value 0.80→0.15, 토스트·매개변수 버전·벽 crop 픽셀 변경, 직접 이미지 확인 |
| 설정 원복 | 확인 | 원본 SHA-256 657226e8a1e745fe3c4509c41cca36103745e79f6e360a92930a74ebf6f9d83b와 일치, main 작업 트리 clean |
| 물리 키보드·fn·IME·Finder 더블클릭 | 미검증 | OS 자동화 사용 불가; 합성 입력과 구별 |

최초 시험에서는 F12 토스트/설정 반영은 성공했으나 벽을 만들지 못해 1개 실패했다. 벌목 반경 7칸이 숲에 닿지 않은 시험 준비 문제였다. 두 번째는 ¼×에서 반경 18칸의 235그루 지정, 비축 구역과 벽 청사진 수락을 각각 확인한 후 벽을 만들었다. 최초 실패 기록은 `/Users/rexxa/work/play-data/shots/play2/gui/`에 보존했다.

## 문서·전달 검증

`bash tools/v8/check_docs.sh`는 `docs: PASS`. `bash -n` 통과, shellcheck는 설치되지 않아 미실행. `.command`는 셸에서 `--update-only`로 직접 실행하여 정상 종료했고, 로그 `/Users/rexxa/work/play-data/logs/main-20261025-010351-90597.log`에 fetch 1초, debug 증분 빌드 0초, import 4초가 기록됐다. Finder의 실행 확인과는 구별한다.

추가 회귀: import 오류 뒤 40,000줄이 오면 `grep -q`의 조기 종료와 `pipefail` 때문에 오류를 놓쳤다. 수정 전 44개 중 2개 실패(`/Users/rexxa/work/play-data/play2/regression-long-before.txt`), 전체 스트림을 읽도록 보완했다. 최초 gate는 이 결함 발견 후 자체 검증 프로세스만 중단했으며 완료 판정하지 않는다. 최종 전체 실행 경로는 `/Users/rexxa/work/play-data/gate-play2-final/`이다.

최종 회귀 44개/실패 0, 전체 gate 34단계 exit 0 및 status 0(72d2db1f, dirty 0). 원격 게시 후 오너 실행기는 main을 같은 SHA로 갱신했고 헤드리스·창 실행 exit 0, 두 체크아웃의 tools/play.sh 바이트 일치를 확인했다. 02:29 헤드리스 로그에는 fetch/작업 트리/증분 빌드 각각 0초, import 생략이 기록됐고 02:30 창 실행의 fetch는 1초였다. 원본 로그와 최종 원격 확인은 PLAY2_REPORT.md에서 찾을 수 있다. 최종 독립 소스 재검토에서도 앞선 창 오류 누락 지적이 해결됐고 추가 중대 문제는 발견되지 않았다.
