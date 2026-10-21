## 구현 완료 보고 — V8-PLAY

목표: `docs/v8/play/PLAY_GOAL.md` Part 1–4. 결정 기록 `docs/v8/play/PLAY_DECISIONS.md`(D1–D14), 리뷰 기록 `docs/v8/play/PLAY_REVIEW.md`.
게임 코드(`rust/`, `scripts/`, `scenes/`, `assets/`, `localization/`, `shaders/`)는 한 줄도 바꾸지 않았다.

### 스크린샷

모두 M1(`892edca1`, 지금의 `v8/main`) 게임, 기본 시드 `20261010`, 1920×1080 창(OpenGL), `tools/v8/capture.gd --m1-set`으로 틱을 정확히 맞춰 찍었다.
**목표 문서의 시각과 실제가 다르다:** 게임 시계는 틱 0 = 1일차 08:00이라 틱 360은 14:00, 틱 900은 23:00(밤)이고, 오두막은 틱 766에 완성된다. 그래서 "작업 중" 장면은 오두막 완성 전 통나무를 든 사람이 가장 많은 틱 481(16:01)에 찍고, 틱 900 그대로의 장면은 따로 남겼다(D2).

| # | 파일 | 틱(시각) | 한 줄 설명 | 커밋 |
|---|---|---|---|---|
| P1 | `docs/images/m1_01_start_2x.png` | 360 (14:00) | 시작 캠프, 2배, 명령 없음 | 커밋 |
| P2 | `/Users/rexxa/work/play-data/shots/m1/m1_02_orders_2x.png` | 361 | 명령 직후: 나무 15그루 X, 4×4 비축 구역, 오두막 유령 "통나무 0/12 · 작업 0%" | play-data |
| P3 | `docs/images/m1_03_work_2x.png` | 481 (16:01) | 작업 중: 통나무 묶음을 인 사람 3명, 베는 사람, 베기 표시 | 커밋 |
| P3b | `/Users/rexxa/work/play-data/shots/m1/m1_03b_t900_2x.png` | 900 (23:00) | 목표 문서의 틱 900 그대로: 오두막 완성 뒤의 밤, 비축 구역에 통나무 더미 | play-data |
| P4 | `docs/images/m1_04_hut_done_2x.png` | 826 (21:46) | 오두막 완성 + 60틱, 연대기(C) 켬 — **"오두막이 완성됨" 줄이 없다(버그)** | 커밋 |
| P5 | `/Users/rexxa/work/play-data/shots/m1/m1_05_night_2x.png` | 900 (23:00) | 밤: 오두막 안 1명, 바깥 대기; 잠든 표시 없음 | play-data |
| P6 | `docs/images/m1_06_far_1x.png` | 481 | 1배 원경, 캠프 중앙 — 1배가 최소라 192×192 지도 전체는 안 들어온다(D3) | 커밋 |
| P7 | `docs/images/m1_07_inspector.png` | 481 | 통나무 5개 든 사람 클릭: "지금: 오두막에 통나무 가져가는 중", "들고 있음: 통나무 5"(스크롤한 상태, D4) | 커밋 |
| P7a | `/Users/rexxa/work/play-data/shots/m1/m1_07a_inspector_top.png` | 481 | 같은 인스펙터의 첫 화면(이름, 초상, 상태, 목표) | play-data |
| P8 | `/Users/rexxa/work/play-data/shots/m1/m1_08_why.png` | 481 | 빈 타일 (105, 68)의 "왜?" 패널: 기록 없음, 개발용 표식 "(γ-2-β)" 노출 | play-data |
| P9 | `/Users/rexxa/work/play-data/shots/m1/m1_09_orderbar.png` | 481 | F3 오두막 모드(실제 키·마우스 입력), 나무 위 빨간 유령, "클릭해서 놓으세요" | play-data |
| V1 | `/Users/rexxa/work/play-data/shots/m1/v7_01_near.png` | 1,440 | V7(`v7-final-art`) 최대 확대 8배 | play-data |
| V2 | `/Users/rexxa/work/play-data/shots/m1/v7_02_far.png` | 1,440 | V7 원경 1배(64×64 지도 전체) | play-data |
| 모음 | `docs/images/m1_contact_sheet.png` | — | 위 12장(P7a 제외), 960×540씩 두 장씩, 한국어 설명; 1944×3680, 494 KB | 커밋 |

기록: `/Users/rexxa/work/play-data/shots/m1/m1_set.json`(각 사진의 틱, 명령 결과). 같은 스크립트를 두 번 돌려 낮 사진은 픽셀까지 같고, 밤 사진 3장은 화덕 불빛 주변(프레임 시간으로 흔들림)만 다르다. 창 실행 증거: `/Users/rexxa/work/play-data/shots/window/{main,v7}_window_frame89.png`.

### 화면 정직 평가

(`docs/v8/play/SCREEN_REPORT.md` 요약)

잘 보이는 것 3가지
1. 세 명령(베기 X, 비축 구역, 오두막 유령과 "통나무 0/12")이 명령 직후 지도에 바로 나타난다.
2. 일의 종류가 머리 위 그림으로 보인다(도끼 = 베는 중, 통나무 묶음 = 나르는 중).
3. 인스펙터가 "지금: …"과 "들고 있음: 통나무 5"를 정확히 보여 준다(스크롤하면).

어색한 것 3가지
1. **버그 2건:** 연대기(C)가 "오두막이 완성됨"을 버린다 — 시뮬레이션은 HutBuilt(종류 22) 기록을 남기지만 `scripts/ui/panels/chronicle_panel.gd`의 `NOTABLE`에 22가 없다. "왜?" 패널에 개발용 표식 "(γ-2-β)"가 보인다.
2. 2배에서도 화면 대부분이 단조로운 초록 바닥이고 사람·건물이 작아 비어 보이며, 밤에는 거의 깜깜하다. V7(큰 천막·사람)보다 오히려 비어 보인다.
3. 거처(나무 상자 모양)와 오두막(지붕 없는 벽 테두리)이 무엇인지 그림만으로 알기 어렵고, 잠든 사람이 구분되지 않는다. 밤에 오두막 안에는 1명뿐이다.

목업 대비 (`docs/images/v8_target_view_b.png`)

| 요소 | M1 |
|---|---|
| 바닥 팔레트 | 임시 |
| 사람 크기와 그림자 | 있음 |
| 튜닉 색 | 있음 |
| 비축 구역 모양 | 임시 |
| 청사진 유령 | 있음 |
| 오두막 | 임시 |
| 나무 | 임시 |
| 언덕 바위 | 임시 |
| 옆 패널 | 임시 |
| 아래 막대 | 있음 |

### 실행 방법

바탕화면 더블클릭 (처음 한 번, 터미널에):
```bash
ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim.command ~/Desktop/WorldSim.command
ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim-v7.command ~/Desktop/WorldSim-v7.command
```
macOS가 처음 한 번 막으면 바탕화면의 파일을 오른쪽 클릭 → 열기(최근 macOS는 시스템 설정 → 개인정보 보호 및 보안 → "그래도 열기"). "터미널이 '다운로드' 폴더에 접근" 확인 창이 뜨면 허용. 맥 키보드에서는 F1–F5를 **fn을 누른 채로** 누른다(또는 아래 명령 막대 버튼을 클릭).

터미널:
```
usage: tools/play.sh [version] [--seed N] [--speed 1-4] [--no-build] [--headless-check]
  version: main (default, = v8/main), wip, m1, v8-0, v7, or any commit/tag/branch
```
- 지원 버전: `main`(원격 `v8/main` 최신) · `wip`(원격 `v8/wip` 최신) · `m1`(`892edca1`) · `v8-0`(`24e16677`) · `v7`(`v7-final-art`) · 임의의 커밋/태그/브랜치. 버전마다 작업 트리 `/Users/rexxa/work/new-world-play/<버전>/`.
- 로그: `/Users/rexxa/work/play-data/logs/<버전>-<시각>.log`.
- 종료 코드: 0 정상, 1 빌드/가져오기/확인 실패·게임 비정상 종료, 2 Godot 없음, 3 모르는 버전, 4 잘못된 인자.

실제로 창으로 띄워 본 결과 (이 맥, Godot 4.6.stable.official.89cea1439):

| | 빌드(새 작업 트리) | 가져오기(새 작업 트리, 2회) | 두 번째부터 | 창이 뜨고 첫 프레임까지(두 번째부터) |
|---|---|---|---|---|
| `main` (= M1 `892edca1`) | 20초 (debug) | 6초 | 빌드 생략 · 가져오기 생략 | 4.9초 (명령 시작 → 첫 "Project FPS" 줄, `git fetch` 포함), 120 FPS |
| `v7` (`b1ca032f`) | 21초 (debug) | 37초 | 빌드 생략 · 가져오기 생략 | 4.8초, 117–120 FPS |

두 버전 모두 1920×1080 창이 떴다(Godot Movie Maker로 실제 창 프레임 저장, D12). 게임이 스스로 끝난 실행(`--quit-after 90`)은 `play.sh` 0, 창을 강제로 닫은 실행(SIGTERM)은 1로 끝났다. 이 세션에서는 `screencapture`가 화면을 못 찍어("could not create image from display") 그 방법을 썼다.

### 변경된 파일 목록

- `03d5abb6` v8-play(p1): `tools/v8/capture.gd`(`--m1-set`), `tools/v8/contact_sheet.py`, `tools/v8/test_contact_sheet.py`, `docs/images/m1_contact_sheet.png`, `docs/images/m1_01_start_2x.png`, `m1_03_work_2x.png`, `m1_04_hut_done_2x.png`, `m1_06_far_1x.png`, `m1_07_inspector.png`, `docs/v8/play/PLAY_GOAL.md`, `PLAY_DECISIONS.md`, `SCREEN_REPORT.md`, `contact_sheet_m1.json`.
- `e34d63a4` v8-play(p2-p3): `tools/play.sh`, `tools/play/WorldSim.command`, `tools/play/WorldSim-v7.command`(실행 권한), `tools/v8/test_play.sh`, `tools/v8/gate.sh`(21단계 `play_check`), `tools/v8/check_docs.sh`(모음 사진 확인), `README.md`, `AGENTS.md`, `CHANGELOG.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md`, `docs/v8/play/PLAY_DECISIONS.md`, 스킬 4개(`worldsim-docs-upkeep`, `worldsim-godot`, `worldsim-pitfalls`, `worldsim-testing`).
- `761fd5e2` v8-play(p4): 리뷰 두 차례 뒤 `README.md`, `tools/play.sh`, 두 `.command`, `docs/v8/play/PLAY_REVIEW.md`. (게이트 통과 커밋)
- (보고서 커밋, 해시는 최종 메시지) v8-play(p4): 이 보고서 `docs/v8/play/PLAY_REPORT.md`, `PLAY_DECISIONS.md` D14, `docs/ROADMAP.md` 행.

### 제한사항

- **게임 코드를 못 고쳐서 남긴 요청:** (1) `scripts/ui/panels/chronicle_panel.gd`의 `NOTABLE`에 22(HutBuilt)를 넣어 "오두막이 완성됨"이 연대기에 뜨게 하기; (2) "왜?" 패널 안내문의 "(γ-2-β)" 지우기(`localization/{ko,en}/ui.json`의 `UI_CAUSAL_PANEL_PLACEHOLDER`, 이어서 `localization/compiled/` 재생성); (3) 오두막 진행 글씨에 배경 넣기, 비축 구역 틴트 진하게, 잠든 사람 표시 — 화면 평가에서 본 것.
- **`--seed`/`--speed`:** 지금 어느 버전(M1, V8-0, V7)의 게임 스크립트도 명령줄을 읽지 않는다. `play.sh`는 "이 버전은 --seed를 지원하지 않습니다 (무시)"라고 알리고 무시한다(값이 잘못되면 종료 코드 4 — 목표 문서에 없는 코드, D11). 게임은 늘 기본 시드 `20261010`, 1배속으로 시작한다.
- **빌드 프로필:** 목표 문서는 release 빌드를 기본으로 적었지만, 플레이에 쓰는 Godot 편집기 실행 파일은 `.gdextension`의 `macos.debug`(debug 라이브러리)를 읽어서 모든 버전에서 debug로 빌드한다(D9).
- **P6 "지도 전체":** 1배가 가장 작은 배율이라 192×192 지도 전체를 한 화면에 담을 수 없다(D3).
- **목표 문서의 틱:** P3/P6–P9는 틱 900이 아니라 측정한 작업 틱 481에서 찍었다(D2). P7의 문구는 "통나무 나르는 중"이 아니라 "오두막에 통나무 가져가는 중"이었다(그 사람의 실제 일).
- **창 확인 방식:** `screencapture`가 이 세션에서 거부되어 Godot의 `--write-movie`로 실제 창 프레임을 저장했다(D12). `.command`는 Finder 더블클릭이 아니라 바탕화면식 심볼릭 링크(절대·상대 경로)를 bash로 실행해 시험했다; Finder 더블클릭과 그때의 macOS 확인 창(Gatekeeper·폴더 접근)은 이 세션에서 시험할 수 없었다 — README에 대응 방법만 적었다.
- 한글 입력 소스에서 글자 키가 안 먹을 수 있다는 README의 안내는 이 맥에서 확인하지 않은 조건부 안내다.
- 게이트 21단계(`play_check`)는 원격 `v8/main`·`m1`·`v7` 작업 트리를 쓰므로 인터넷(`git fetch`)과 첫 실행 때 약 100초가 필요하다.

### Gate 결과

`bash tools/v8/gate.sh /Users/rexxa/work/play-data/gate/761fd5e2` — 커밋 `761fd5e2`(head 기록 일치, dirty 0), 11:11:32–12:07:06 KST, **status 0 (21단계 모두 통과)**. 요약 `/Users/rexxa/work/play-data/gate/761fd5e2/gate.txt`, 단계별 로그는 같은 폴더의 `<단계>.log`.

| # | 단계 | 결과 | 시간 |
|---|---|---|---|
| 1 | fmt | 통과 | 1초 |
| 2 | bridge_build | 통과 | 0초 |
| 3 | godot_import | 통과 | 7초 |
| 4 | test_debug | 통과 — 1,782 통과 / 0 실패 / 0 무시 | 1,987초 |
| 5 | test_release | 통과 — 1,782 / 0 / 0 | 117초 |
| 6 | clippy | 통과 | 0초 |
| 7 | forbidden_math | 통과 | 0초 |
| 8 | gdscript_strict | 통과 | 6초 |
| 9 | smoke | 통과 — script_errors 0, errors 0, locale_warnings 0, raw_keys 0 | 15초 |
| 10 | identity | 통과 — summary 228,748값 차이 0, chronicle 64파일 같음, raw samples 64파일 같음, determinism PASS | 498초 |
| 11 | determinism | 통과 — DETERMINISM PASS | 139초 |
| 12 | perf (보고만) | OK — natural ×0.908, quick ×0.889 (+10 % 넘는 평균 없음) | 530초 |
| 13 | bridge_api | 통과 | 0초 |
| 14 | docs | 통과 (`docs: PASS`, 모음 사진·README 링크 확인 포함) | 1초 |
| 15 | inventory_closure | 통과 | 0초 |
| 16 | python_unittest | 통과 (`test_contact_sheet.py` 4개 포함) | 1초 |
| 17 | m1_tests | 통과 — 36 / 0 | 16초 |
| 18 | colony_determinism | 통과 — DETERMINISM PASS | 3초 |
| 19 | replay | 통과 | 1초 |
| 20 | uid_check | 통과 | 0초 |
| 21 | play_check (새 단계) | 통과 — 6/6: 모르는 버전 3, Godot 없음 2, 잘못된 인자 4, `--headless-check` main·m1·v7 0 | 12초 |

이 보고서 커밋은 게이트 커밋 뒤에 보고서·결정 D14·ROADMAP 행만 더한다(D14). 그 커밋에서 `check_docs.sh`(docs: PASS)와 `test_play.sh --quick`(3/3)을 다시 돌렸다.

git ls-remote 결과

보고서 커밋을 `v8/wip`에 푸시하고 `v8/main`을 그 커밋으로 fast-forward한 뒤의 `git ls-remote origin v8/wip v8/main` 결과는 최종 메시지에 있다(두 줄 모두 이 보고서 커밋이어야 한다).
