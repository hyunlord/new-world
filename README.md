# WorldSim

석기시대의 작은 무리를 이끌어, 사람 하나하나의 손기술과 관계가 쌓여 마을이 되고, 기술이 사람 사이로 퍼지면서 시대가 저절로 넘어가는 **드워프 포트리스식 정착지 시뮬레이션**이다.

- 10명 안팎으로 시작해 출생과 이주로 30~200명까지 자란다. 구조는 수천 명까지 늘릴 수 있게 만든다.
- 플레이어는 파기·베기·짓기·구역·작업 우선순위·제작 주문을 내리고, 사람들은 숙련과 기질에 따라 스스로 일을 고른다. 위협이 오면 림월드식으로 사람을 소집할 수 있다.
- 기술은 사람이 가지고, 사람 사이로 퍼지고, 마지막 전승자가 죽으면 사라진다. 시대는 표지 기술이 얼마나 퍼졌는지로 관측된다.
- 시간: 1배속에서 하루가 2분(1,440틱 ÷ 초당 12틱, M1부터)이고 1~4 키가 1~4배속(초당 12~48틱), P가 일시정지다. 달력은 아직 V7 그대로(계절 7일 × 4 = 28일, 나이는 일 단위)이고, 계절 6일·1년 24일·나이를 해로 세는 달력은 M1.5에서 바꾼다(ADR 0004). 화면은 어두운 흙빛 정면 탑다운, 사람 하나가 타일 한 칸이다.

자세한 방향은 [docs/VISION.md](docs/VISION.md), 진행 상황은 [docs/ROADMAP.md](docs/ROADMAP.md).

> 지금(M1)의 게임은 192×192 세계에 창설자 10명으로 시작하는 첫 정착지다. F1–F5로 지시 모드를 고른다: F1 나무 베기(끌어서 범위), F2 비축 구역, F3 오두막 청사진(3×3, 통나무 12개), F4 구역 지우기, F5 취소. Home/H 키는 캠프로 화면을 옮긴다. 사람들이 나무를 베어 통나무를 나르고 비축 구역에 쌓고 오두막을 짓는다. 사람을 클릭하면 상세 정보와 지금 하는 일, 들고 있는 것이 보인다. 화면의 타일셋은 임시(코드로 그림)다.

**지금 화면:** [docs/images/m1art_contact_sheet.png](docs/images/m1art_contact_sheet.png) (M1-ART: 같은 장면의 이전/이후 7쌍 + 목업 비교; 지금 `v8/main` = M1-ART) · M1 화면 평가는 [docs/v8/play/SCREEN_REPORT.md](docs/v8/play/SCREEN_REPORT.md), 새 화면 보고는 [docs/v8/m1art/ART_REPORT.md](docs/v8/m1art/ART_REPORT.md).

## 빌드와 실행

필요한 것 (처음 한 번):
- **Godot 4.6** — `Godot.app` 이름 그대로 `/Users/rexxa/Downloads`나 `/Applications`에 둔다(저절로 찾는다; 다른 곳이면 Spotlight로 찾는다). 내려받은 Godot.app은 처음 한 번 Finder에서 더블클릭해 열어 두면(macOS 확인 창 → 열기) 이후 막히지 않는다.
- **Rust** — 터미널에서 `cargo --version`이 안 되면 https://rustup.rs 의 한 줄 설치 명령을 터미널에 붙여 넣는다(`rust/rust-toolchain.toml`이 정한 버전을 저절로 받는다). 맥에 Apple 명령줄 도구가 없으면 `xcode-select --install`도 한 번(터미널에서 `git`이 되면 이미 있다).
- **이 저장소의 클론** — `/Users/rexxa/work/new-world-pass4`, 브랜치 `v8/wip`(또는 `v8/main`; 실행기 `tools/play/`가 있는 브랜치). 실행기 자체를 최신으로 하려면 가끔 `git -C /Users/rexxa/work/new-world-pass4 pull --ff-only`. 첫 실행 때는 인터넷이 필요하다(`git fetch`, Rust 패키지 내려받기). 버전마다 작업 트리와 빌드가 따로 생겨 디스크를 지금 게임 약 1.4 GB, V7 약 4.3 GB 쓴다.

### 가장 쉬운 방법: 바탕화면의 `WorldSim.command`를 더블클릭

처음 한 번만 터미널에서 바탕화면에 바로가기를 만든다.

```bash
ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim.command ~/Desktop/WorldSim.command
ln -sf /Users/rexxa/work/new-world-pass4/tools/play/WorldSim-v7.command ~/Desktop/WorldSim-v7.command
```

- `WorldSim.command` = 지금 게임(GitHub의 `v8/main` 최신), `WorldSim-v7.command` = 비교용 V7 게임(`v7-final-art`).
- macOS가 처음 한 번 막을 수 있다. "열 수 없음/확인되지 않은 개발자"가 뜨면 바탕화면의 파일을 **오른쪽 클릭 → 열기 → 열기**; 최근 macOS에서 그래도 막히면 **시스템 설정 → 개인정보 보호 및 보안 → 맨 아래 "그래도 열기"**. "터미널이 '다운로드' 폴더에 접근하려고 합니다"가 뜨면 **허용**을 누른다(거부하면 Godot을 못 찾아 "Godot 4.6을 찾지 못했습니다"로 끝난다; 그때는 Godot.app을 `/Applications`로 옮기면 된다).
- 터미널 창이 열리고 `play:`로 시작하는 줄이 나온 뒤 1920×1080 게임 창이 뜬다. 이 맥에서 잰 첫 실행(새 작업 트리: 빌드 + 가져오기 두 번)은 지금 게임 약 30초, V7 약 1분이었다(Rust 패키지를 처음 받는 맥은 몇 분). 빌드 중에는 한동안 아무것도 안 나와도 정상이다. 다음부터는 빌드와 가져오기를 건너뛰어("빌드 생략", "가져오기 생략") 약 5초 만에 뜬다. `v8/main`이 새로 올라오면 그때 한 번 바뀐 만큼 다시 빌드·가져오기를 한다.
- 맥북 화면은 대부분 1920×1080보다 작아서 창이 화면보다 크게 뜬다. 창 제목 줄을 더블클릭하면 화면에 맞춰진다(게임 화면은 창 크기에 맞게 늘거나 준다). 게임은 1일차 08:00, 2배 확대, 캠프가 가운데인 화면으로 시작한다.
- 게임 창을 닫으면(또는 ⌘Q) 끝나고, 남은 터미널 창은 ⌘W로 닫는다. 게임 중에 터미널 창을 닫으면 게임도 꺼진다. 실패하면 터미널 창에 이유가(빌드·가져오기 단계면 로그 경로도) 남고, 아무 키나 누르면 스크립트가 끝난다.

**처음 할 일 (명령을 내리기 전에는 사람들이 먹을 것을 모으고 쉴 뿐, 나무를 베거나 집을 짓지 않는다).** 1배속에서 하루가 2분이라 08:00에 시작하면 약 1분 뒤(21시)에 밤이 된다. 먼저 **P**로 멈추고 명령을 내린 뒤 다시 **P**로 풀면 낮에 오두막이 서는 것을 볼 수 있다(멈춘 동안에도 명령은 바로 들어간다).
1. **F1**(베기)을 누르고 캠프 근처 숲을 마우스로 끌어 덮는다 → 나무에 노란 X.
2. **F2**(비축 구역)를 누르고 빈 땅을 4×4쯤 끈다 → 말뚝 박힌 다진 흙 바닥.
3. **F3**(오두막)을 누르고 초록 유령이 뜨는 곳을 클릭한다(빨강 = 놓을 수 없음) → 파란 청사진과 "통나무 0/12".
4. 사람들이 나무를 베어 통나무를 나르고 오두막을 짓는다. 1배속에서 30–40초쯤 지나면 오두막이 선다(스크린샷의 시드에서 놓은 뒤 405틱 ≈ 34초). 밤에는 화면이 많이 어두워진다.

**키보드 주의:** 글자 키(P, C, H, WASD)가 안 먹으면 입력 소스를 한글에서 ABC로 바꿔 본다(Ctrl+Space 또는 Caps Lock). 맥북·애플 키보드에서 F1–F5는 밝기·Mission Control 같은 시스템 키라서 **fn을 누른 채로** 눌러야 게임에 들어간다(또는 시스템 설정 → 키보드 → 키보드 단축키 → 기능 키 → "F1, F2 등의 키를 표준 기능 키로 사용" 켜기). 명령 막대의 버튼을 마우스로 눌러도 된다. Home 키가 없으면 **H**.

게임 안 조작: F1–F5 지시 모드(F1 베기: 끈 범위의 나무에 표시 · F2 비축 구역: 끈 범위를 구역으로 · F3 오두막: 클릭한 곳에 청사진 · F4 구역 지우기: 끈 범위의 비축 구역 칸을 지움 · F5 취소: 끈 범위의 베기 표시와 청사진을 지움; Esc/오른쪽 클릭으로 모드 끄기), 숫자 1–4 속도, P 일시정지, C 연대기, H(또는 Home) 캠프로, 마우스 휠 또는 -·= 확대/축소(1–4배 확대), WASD·화살표 화면 이동, 사람을 클릭하면 정보 패널(지금 하는 일, 들고 있는 것), 빈 땅을 클릭하면 "왜?" 패널.

지금 화면은 M1-ART 그림이다: 덩어리진 숲, 통나무 벽과 잠자리가 보이는 오두막, 가죽 지붕 움막, 다진 흙 비축 구역과 커지는 통나무 더미, 테두리가 있는 사람(숲 속·밤에는 밝은 윤곽). V8-PLAY의 버그 2건(연대기에 "오두막이 완성됨"이 안 나옴, "왜?" 패널의 개발 표식)은 고쳤다. 사람 하나가 타일 한 칸인 것은 그대로다.

### 터미널: `tools/play.sh` (`main` / `m1` / `v7` …)

```
usage: tools/play.sh [version] [--seed N] [--speed 1-4] [--no-build] [--headless-check]
  version: main (default, = v8/main), wip, m1, v8-0, v7, or any commit/tag/branch
```

- 예: `cd /Users/rexxa/work/new-world-pass4 && tools/play.sh` (= 바탕화면 `WorldSim.command`), `tools/play.sh v7`, `tools/play.sh m1`.
- 버전: `main`(기본값, 원격 `v8/main` 최신) · `wip`(원격 `v8/wip` 최신) · `m1`(`892edca1`) · `v8-0`(`24e16677`) · `v7`(태그 `v7-final-art`) · 그 밖의 커밋/태그/브랜치.
- 하는 일: ① Godot 찾기(`GODOT_BIN`이 있으면 그것만 → `tools/v8/godot_bin.sh`의 경로(`~/Downloads/Godot.app`) → Spotlight → `/Applications/Godot.app`; 4.6이 아니면 경고) ② `git fetch` 뒤 버전마다 따로 작업 트리 `/Users/rexxa/work/new-world-play/<버전>/`을 만들거나 그 버전으로 옮기기(이 클론의 작업 파일로는 실행하지 않는다; 작업 트리는 이 클론의 `.git`에 묶여 있으니 클론을 옮기거나 지우지 않는다) ③ Rust가 바뀌었을 때만 `cargo build -p sim-bridge` — `.gdextension`이 Godot 편집기 실행 파일에 읽히는 라이브러리(`macos.debug`, 지금은 모든 버전이 debug)를 가리키므로 그 프로필로 빌드; 아니면 "빌드 생략" ④ 필요할 때만 가져오기(import; 새 작업 트리는 두 번) ⑤ 게임 창 실행. 실행 전에 버전·커밋·빌드·가져오기·시드·속도를 한 줄로 보여 준다.
- `--seed`, `--speed`: 그 버전의 `scripts/`가 이 인자를 읽을 때만 넘긴다. 지금(M1, V7 포함) 어느 버전도 읽지 않아서 "이 버전은 --seed를 지원하지 않습니다 (무시)"라고 알리고 무시한다(게임은 기본 시드 `20261010`, 1배속으로 시작; 속도는 게임 안에서 1–4 키). 값이 잘못되면(예: `--speed 7`) 종료 코드 4.
- `--no-build`: 빌드 단계를 건너뛴다(라이브러리가 아직 없으면 종료 코드 1).
- `--headless-check`: 창을 띄우지 않고 준비 후 300프레임만 돌려 오류(`SCRIPT ERROR`)가 0줄인지 확인한다.
- `-- <Godot 인자>`: 그 뒤는 Godot에 그대로 넘긴다(예: `tools/play.sh main -- --print-fps`).
- 로그: `/Users/rexxa/work/play-data/logs/<버전>-<시각>.log` (버전을 알아낸 뒤 만들고 경로를 출력; 지우지 않으니 가끔 비운다).
- 종료 코드: 0 정상, 1 빌드/가져오기/확인 실패·작업 트리 문제·게임 비정상 종료, 2 Godot 없음, 3 모르는 버전(인터넷이 끊겨 원격 브랜치를 못 받은 경우 포함), 4 잘못된 인자. 작업 트리가 꼬이면("옮기지 못했습니다", "git worktree add 실패") `rm -rf /Users/rexxa/work/new-world-play/<버전>` 다음 `git -C /Users/rexxa/work/new-world-pass4 worktree prune`을 하고 다시 실행하면 새로 만든다.

### 수동으로

지금 클론(체크아웃된 브랜치와 고친 파일 그대로)을 실행한다. 다른 버전은 위의 `tools/play.sh`를 쓴다.

```bash
source tools/v8/godot_bin.sh                                  # GODOT_BIN = /Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot (이미 있으면 그대로)
cd rust && cargo build -p sim-bridge && cd ..                 # Godot이 읽는 libsim_bridge 빌드
"$GODOT_BIN" --headless --path . --import --editor --quit     # 가져오기: 새로 받은 체크아웃에서는 두 번(두 번째 결과로 판단), 에셋이 바뀐 뒤에는 한 번
"$GODOT_BIN" --path . --resolution 1920x1080                  # 게임 실행
```

Rust를 고친 뒤에는 Godot을 띄우기 전에 반드시 `cargo build -p sim-bridge`를 다시 한다 (`cargo test`는 Godot이 읽는 라이브러리를 다시 만들지 않는다).

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
