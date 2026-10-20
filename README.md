# WorldSim

석기시대의 작은 무리를 이끌어, 사람 하나하나의 손기술과 관계가 쌓여 마을이 되고, 기술이 사람 사이로 퍼지면서 시대가 저절로 넘어가는 **드워프 포트리스식 정착지 시뮬레이션**이다.

- 10명 안팎으로 시작해 출생과 이주로 30~200명까지 자란다. 구조는 수천 명까지 늘릴 수 있게 만든다.
- 플레이어는 파기·베기·짓기·구역·작업 우선순위·제작 주문을 내리고, 사람들은 숙련과 기질에 따라 스스로 일을 고른다. 위협이 오면 림월드식으로 사람을 소집할 수 있다.
- 기술은 사람이 가지고, 사람 사이로 퍼지고, 마지막 전승자가 죽으면 사라진다. 시대는 표지 기술이 얼마나 퍼졌는지로 관측된다.
- 목표 시간 척도(이후 패스에서 바꿈, 아직 일정 없음): 하루는 1배속에서 약 2분, 1년은 계절 6일 × 4 = 24일이다. 지금(V8-0) 빌드는 V7 달력 그대로라 하루가 1배속 48초(1,440틱 ÷ 초당 30틱), 1년이 계절 7일 × 4 = 28일이다. 화면은 어두운 흙빛 정면 탑다운, 사람 하나가 타일 한 칸이다.

자세한 방향은 [docs/VISION.md](docs/VISION.md), 진행 상황은 [docs/ROADMAP.md](docs/ROADMAP.md).

> 지금(V8-0)의 게임은 V7 시뮬레이션을 그대로 돌리는 화면 골격이다: 세계, 사람, 건물, 상단 막대, 연대기 패널, 사람을 클릭하면 상세 정보와 '왜?' 패널. 새 플레이는 M1부터 들어간다.

## 빌드와 실행

필요한 것: Rust 1.93 (`rust/rust-toolchain.toml`), Godot 4.6 (macOS 기본 경로 `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`, 다른 곳이면 `GODOT_BIN`을 지정).

```bash
cd rust && cargo build -p sim-bridge && cd ..                 # Godot이 읽는 libsim_bridge 빌드
"$GODOT_BIN" --headless --path . --import --editor --quit     # 가져오기: 새로 받은 체크아웃에서는 두 번(두 번째 결과로 판단), 에셋이 바뀐 뒤에는 한 번
"$GODOT_BIN" --path .                                         # 게임 실행
```

Rust를 고친 뒤에는 Godot을 띄우기 전에 반드시 `cargo build -p sim-bridge`를 다시 한다 (`cargo test`는 Godot이 읽는 라이브러리를 다시 만들지 않는다).

## 게이트

```bash
bash tools/v8/gate.sh /path/to/log_dir      # 16단계: fmt, 빌드, 가져오기, 테스트, clippy, 결정론, identity, 문서 …
```

`v8/main`은 게이트가 모두 통과한 커밋으로만 옮긴다. 16단계 각각의 설명은 `.claude/skills/worldsim-testing/SKILL.md`(절 C)와 `tools/v8/gate.sh` 머리말에 있다.

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
| `docs/v8/` | V8-0 패스의 목표, 방향 원본, 결정, 인벤토리, 보고서 |

## 브랜치 규칙

1. 작업은 `v8/wip`에 커밋한다. `v8/main`은 게이트가 통과한 커밋으로 fast-forward만 한다.
2. `lead/main`, `main`, `codex/*`에는 푸시하지 않는다. force-push, 이력 재작성, 원격 브랜치·태그 삭제는 하지 않는다.
3. V7 줄기와 연구는 태그 `v7-final-sim`, `v7-final-art`와 `codex/worldmodel-2026-10`에 고정되어 있다.

---

**English.** WorldSim is a Dwarf Fortress–style stone-age settlement sim: about ten founders grow into a village of 30–200 people; the player gives orders (dig, chop, build, zones, work priorities, craft orders) and people choose their own work; technology is owned by individuals, spreads between them and is lost when its last bearer dies, and eras are observed from how widely marker technologies have spread. The simulation is Rust (hecs ECS, deterministic, integer math) behind a Godot 4.6 GDExtension bridge; GDScript only draws and reads. Build the bridge with `cd rust && cargo build -p sim-bridge`, import with `Godot --headless --path . --import --editor --quit` (twice on a fresh checkout; judge the second run), run with `Godot --path .`, and check a change with `bash tools/v8/gate.sh <log_dir>`. Agents start at [AGENTS.md](AGENTS.md); the structure is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
