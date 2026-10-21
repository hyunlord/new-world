# WorldSim

석기시대의 작은 무리를 이끌어, 사람 하나하나의 손기술과 관계가 쌓여 마을이 되고, 기술이 사람 사이로 퍼지면서 시대가 저절로 넘어가는 **드워프 포트리스식 정착지 시뮬레이션**이다.

- 10명 안팎으로 시작해 출생과 이주로 30~200명까지 자란다. 구조는 수천 명까지 늘릴 수 있게 만든다.
- 플레이어는 파기·베기·짓기·구역·작업 우선순위·제작 주문을 내리고, 사람들은 숙련과 기질에 따라 스스로 일을 고른다. 위협이 오면 림월드식으로 사람을 소집할 수 있다.
- 기술은 사람이 가지고, 사람 사이로 퍼지고, 마지막 전승자가 죽으면 사라진다. 시대는 표지 기술이 얼마나 퍼졌는지로 관측된다.
- 시간: 1배속에서 하루가 2분(1,440틱 ÷ 초당 12틱, M1부터)이고 1~4 키가 1~4배속(초당 12~48틱), P가 일시정지다. 달력은 아직 V7 그대로(계절 7일 × 4 = 28일, 나이는 일 단위)이고, 계절 6일·1년 24일·나이를 해로 세는 달력은 M1.5에서 바꾼다(ADR 0004). 화면은 어두운 흙빛 정면 탑다운, 사람 하나가 타일 한 칸이다.

자세한 방향은 [docs/VISION.md](docs/VISION.md), 진행 상황은 [docs/ROADMAP.md](docs/ROADMAP.md).

> 지금(M1)의 게임은 192×192 세계에 창설자 10명으로 시작하는 첫 정착지다. F1–F5로 지시 모드를 고른다: F1 나무 베기(끌어서 범위), F2 비축 구역, F3 오두막 청사진(3×3, 통나무 12개), F4 구역 지우기, F5 취소. Home/H 키는 캠프로 화면을 옮긴다. 사람들이 나무를 베어 통나무를 나르고 비축 구역에 쌓고 오두막을 짓는다. 사람을 클릭하면 상세 정보와 지금 하는 일, 들고 있는 것이 보인다. 화면의 타일셋은 임시(코드로 그림)다.

**지금 화면:** [docs/images/m1_contact_sheet.png](docs/images/m1_contact_sheet.png) (M1 스크린샷 12장) · 정직한 평가는 [docs/v8/play/SCREEN_REPORT.md](docs/v8/play/SCREEN_REPORT.md).

## 빌드와 실행

필요한 것: Godot 4.6(`Godot.app`; `~/Downloads`나 `/Applications`에 두면 저절로 찾는다), Rust(`cargo`, 버전은 `rust/rust-toolchain.toml`이 정한다), 그리고 이 저장소의 클론(`/Users/rexxa/work/new-world-pass4`).

### 가장 쉬운 방법: 바탕화면의 `WorldSim.command`를 더블클릭

처음 한 번만 터미널에서 바탕화면에 바로가기를 만든다.

```bash
ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim.command ~/Desktop/WorldSim.command
ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim-v7.command ~/Desktop/WorldSim-v7.command
```

- `WorldSim.command` = 지금 게임(`v8/main`), `WorldSim-v7.command` = 비교용 V7 게임(`v7-final-art`).
- 처음 더블클릭하면 macOS가 "확인되지 않은 개발자" 등으로 막을 수 있다. 그때는 바탕화면의 파일을 **오른쪽 클릭 → 열기 → 열기**로 한 번 허용하면 다음부터는 더블클릭으로 열린다.
- 터미널 창이 열리고 준비(아래 1–4단계)를 한 뒤 1920×1080 게임 창이 뜬다. 첫 실행은 빌드와 가져오기 때문에 1분 안팎 걸리고, 다음부터는 몇 초 만에 뜬다. 게임 창을 닫으면 끝난다. 실패하면 터미널 창이 열린 채로 남아 이유와 로그 경로를 보여 준다(아무 키나 누르면 닫힌다).
- 게임 안 조작: F1–F5 지시 모드(F1 베기 · F2 비축 구역 · F3 오두막 · F4 구역 지우기 · F5 취소, 끌거나 클릭해서 지시, Esc/오른쪽 클릭으로 취소), 1–4 속도, P 일시정지, C 연대기, Home/H 캠프로, 마우스 휠 또는 -·= 확대/축소(1–4배), WASD·화살표 이동, 사람이나 땅을 클릭하면 정보 패널.

### 터미널: `tools/play.sh` (`main` / `m1` / `v7` …)

```
usage: tools/play.sh [version] [--seed N] [--speed 1-4] [--no-build] [--headless-check]
  version: main (default, = v8/main), wip, m1, v8-0, v7, or any commit/tag/branch
```

- 버전: `main`(기본값, 원격 `v8/main` 최신) · `wip`(원격 `v8/wip` 최신) · `m1`(`892edca1`) · `v8-0`(`24e16677`) · `v7`(태그 `v7-final-art`) · 그 밖의 커밋/태그/브랜치.
- 하는 일: ① Godot 찾기(`GODOT_BIN` → `tools/v8/godot_bin.sh`의 경로 → Spotlight → `/Applications/Godot.app`) ② 버전마다 따로 작업 트리 `/Users/rexxa/work/new-world-play/<버전>/`을 만들거나 다시 쓰기(이 클론 자체로는 실행하지 않는다) ③ Rust가 바뀌었을 때만 `cargo build -p sim-bridge`(Godot 편집기 실행 파일이 debug 라이브러리를 읽으므로 debug로 빌드; 아니면 "빌드 생략") ④ 필요할 때만 가져오기(import; 새 작업 트리는 두 번) ⑤ 게임 창 실행. 실행 전에 버전·커밋·빌드·가져오기·시드·속도를 한 줄로 보여 준다.
- `--seed`, `--speed`: 지금 어느 버전도 명령줄에서 읽지 않는다. 주면 "이 버전은 --seed를 지원하지 않습니다"라고 알리고 무시한다(게임은 기본 시드 `20261010`, 1배속으로 시작; 속도는 게임 안에서 1–4 키).
- `--headless-check`: 창을 띄우지 않고 준비 후 300프레임만 돌려 오류(`SCRIPT ERROR`)가 0줄인지 확인한다.
- 로그: `/Users/rexxa/work/play-data/logs/<버전>-<시각>.log` (시작할 때 경로를 출력).
- 종료 코드: 0 정상, 1 빌드/가져오기/확인 실패, 2 Godot 없음, 3 모르는 버전, 4 잘못된 인자.

### 수동으로

```bash
cd rust && cargo build -p sim-bridge && cd ..                 # Godot이 읽는 libsim_bridge 빌드
"$GODOT_BIN" --headless --path . --import --editor --quit     # 가져오기: 새로 받은 체크아웃에서는 두 번(두 번째 결과로 판단), 에셋이 바뀐 뒤에는 한 번
"$GODOT_BIN" --path .                                         # 게임 실행
```

`GODOT_BIN`의 기본값은 `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`(`source tools/v8/godot_bin.sh`). Rust를 고친 뒤에는 Godot을 띄우기 전에 반드시 `cargo build -p sim-bridge`를 다시 한다 (`cargo test`는 Godot이 읽는 라이브러리를 다시 만들지 않는다).

## 게이트

```bash
bash tools/v8/gate.sh /path/to/log_dir      # 21단계: fmt, 빌드, 가져오기, 테스트, clippy, 결정론, identity, 문서, M1 테스트, 정착지 결정론, 재생, UID, 실행 확인 …
```

`v8/main`은 게이트가 모두 통과한 커밋으로만 옮긴다. 21단계 각각의 설명은 `.claude/skills/worldsim-testing/SKILL.md`(절 C)와 `tools/v8/gate.sh` 머리말에 있다.

## 문서 지도

| 문서 | 내용 |
|---|---|
| [docs/VISION.md](docs/VISION.md) | 게임의 목적과 방향 (오너 결정 사항) |
| [docs/ROADMAP.md](docs/ROADMAP.md) | M0~M6 단계와 현재 상태 |
| [docs/HISTORY.md](docs/HISTORY.md) | V7 이전부터 V8까지의 역사, 지운 것의 기록 |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 크레이트·폴더 지도, 데이터 흐름, 시스템 표, 브리지 API (영문) |
| [docs/decisions/](docs/decisions/) | 설계 결정 기록 ADR (영문) |
| [docs/archive/INDEX.md](docs/archive/INDEX.md) | 지운 문서·코드가 어느 태그에 있는지 |
| [CHANGELOG.md](CHANGELOG.md) | 패스별 변경 (영문) |
| [AGENTS.md](AGENTS.md) | 코딩 에이전트 지침 (영문); `CLAUDE.md`는 이것을 가리킨다 |
| `.claude/skills/` | 작업 요령과 교훈: 구조, Godot, 함정, 테스트, 문서 갱신 |
| `docs/v8/` | V8-0 패스의 목표, 방향 원본, 결정, 인벤토리, 보고서; `docs/v8/m1/`은 M1 패스의 같은 기록 |

## 브랜치 규칙

1. 작업은 `v8/wip`에 커밋한다. `v8/main`은 게이트가 통과한 커밋으로 fast-forward만 한다.
2. `lead/main`, `main`, `codex/*`에는 푸시하지 않는다. force-push, 이력 재작성, 원격 브랜치·태그 삭제는 하지 않는다.
3. V7 줄기와 연구는 태그 `v7-final-sim`, `v7-final-art`와 `codex/worldmodel-2026-10`에 고정되어 있다.

---

**English.** WorldSim is a Dwarf Fortress–style stone-age settlement sim: about ten founders grow into a village of 30–200 people; the player gives orders (dig, chop, build, zones, work priorities, craft orders) and people choose their own work; technology is owned by individuals, spreads between them and is lost when its last bearer dies, and eras are observed from how widely marker technologies have spread. The simulation is Rust (hecs ECS, deterministic, integer math) behind a Godot 4.6 GDExtension bridge; GDScript only draws and reads. Build the bridge with `cd rust && cargo build -p sim-bridge`, import with `Godot --headless --path . --import --editor --quit` (twice on a fresh checkout; judge the second run), run with `Godot --path .`, and check a change with `bash tools/v8/gate.sh <log_dir>`. Agents start at [AGENTS.md](AGENTS.md); the structure is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
