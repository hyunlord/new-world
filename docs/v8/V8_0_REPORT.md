## 구현 완료 보고 — V8-0

### 구현 의도
신 게임 시절의 코드·문서를 정리하고, 시뮬레이션을 바꾸지 않은 채(32세계 × 120일 identity 동일) V8 정착지 시뮬레이션 개발의 깨끗한 출발점을 만든다.

### 구현 내용
- 태그: `v7-final-sim` = `354e4a13` (태그 객체 `3f1b2072`), `v7-final-art` = `b1ca032f` (태그 객체 `7db29076`). 둘 다 원격에 있다.
- 브랜치: `v8/wip` = 이 보고서를 담은 커밋 (게이트 커밋 `672fffef` 위의 문서 전용 커밋 하나; 해시는 `git ls-remote origin v8/wip v8/main`), `v8/main` = 같은 커밋 (최종 게이트가 16단계 모두 통과한 `672fffef`에서 만들어 푸시하고, 보고서 커밋 뒤 문서 단계(14)만 다시 돌려 fast-forward).
- 삭제 요약 표 (기준 `b1ca032f` → 게이트 커밋 `672fffef`; 코드 줄 = `.rs`/`.gd` 줄 수):

  | 영역 | 전 | 후 | 비고 |
  |---|---|---|---|
  | sim-test | 202 파일, 3.6 MiB, 86,529줄 | 110 파일, 2.8 MiB, 70,903줄 | 세계 모델 기록기, ablation 장면, 패스 전용 스캐폴딩, 버전 차이 테스트 삭제; `harness_*` 108개 → 도메인 파일 68개 |
  | sim-systems | 54 파일, 18,246줄 | 59 파일, 19,829줄 | 개입 시스템 삭제, 결정 모듈 분할(+5 파일); 줄 증가는 `cargo fmt`, 모듈 머리말, 기준값으로 옮긴 단위 테스트 |
  | sim-core | 170 파일, 13,994줄 | 179 파일, 13,634줄 | `game_v8…v26` 사슬 → `rules/baseline.rs`(`rules/mod.rs` 1,821 → 819줄), 죽은 규칙 필드 7개 삭제, V7 덤프 8개 추가 |
  | sim-bridge | 15 파일, 134.9 MiB, 6,356줄 | 14 파일, 0.2 MiB, 6,475줄 | 135 MB 아트 커널 픽스처 삭제, 개입 메서드 2개 삭제, `world_node.rs` 4,364줄 → 6개 파일 |
  | sim-engine | 4 파일, 1,490줄 | 3 파일, 1,442줄 | `intervention.rs` 삭제 |
  | scripts/ui | 53 파일, 8,717줄 | 8 파일, 1,686줄 | 오버레이·배지·신의 손·상태 패널 삭제; 31개는 `scripts/core`, `scripts/world`, `scripts/ui/hud`로 이동 (scripts 전체 278 파일 43,342줄 → 39 파일 6,721줄) |
  | scripts/test | 222 파일, 33,947줄 | 0 | V7·아트 라인 Godot 테스트 코드 |
  | tools | 895 파일, 6.5 MiB | 25 파일, 0.1 MiB | V7 하네스, revitalize, 아트 파이프라인, 세계 모델 도구 → `tools/v8/` |
  | docs | 17,280 파일, 2,362.7 MiB | 27 파일, 0.6 MiB | 패스 기록·증거·보드 삭제; 삭제 목록은 `docs/archive/INDEX.md` |
  | localization 키 수 | 5,287 | 156 | Fluent 원본 삭제, 카테고리 JSON이 원본 (`localization/{ko,en}/{ui,buildings}.json`) |
  | assets | 888 파일, 15.0 MiB | 198 파일, 12.9 MiB | 참조되지 않는 스프라이트·오디오·타일셋·아트 원본 삭제 (ASK 174개는 유지) |
  | shaders / scenes | 11 / 3 | 3 / 1 | |
  | 저장소 트리 전체 | 20,299 파일, 2,533 MiB | 714 파일, 18.5 MiB | 지운 것은 모두 `v7-final-art`에 있다 |

- 구조 변경:
  - 규칙: `game_vNN` 사슬 → `WorldRules::baseline(seed)` / `WorldGenConfig::baseline(seed)` 하나 (V7 `game_v26`과 같은 평면 리터럴, variety 추첨만 호출). `baseline_matches_v7_final` 두 테스트가 V7 덤프(시드 0, 1, 7, 42)와 비교.
  - 분할: `sim-bridge/src/ffi/world_node.rs` → `node.rs`, `engine_build.rs`, `snapshot.rs`, `reads_agent.rs`, `reads_settlement.rs`, `causal.rs`; `sim-systems/.../decision/agent_decision.rs` → `system.rs`, `targets.rs`, `needs.rs`, `forage.rs`, `memory_bias.rs`, `wood.rs`. 둘 다 순수 이동(항목 169개, 75개 그대로).
  - 폴더: `scripts/core`, `scripts/world`, `scripts/ui/hud`, `scripts/ui/panels` (`.gd`와 `.uid`를 함께 이동, `res://` 경로 30개 이동·26개 파일 수정).
  - 테스트: `tests/harness_*.rs` 108개 → `agents_*`, `settlement_*`, `family_*`, `relations_*`, `chronicle_*`, `engine_*`, `determinism.rs` 68개.
  - 이름 변경: 패스·버전 접미사 제거 — probe 모듈 16개, Rust 테스트 이름 9개, GDScript 파일 6개와 클래스·메서드·환경변수, 아틀라스 3개 (전부 DECISIONS D24).
  - 죽은 규칙 필드 7개 삭제 (D25): `capacity_weighted`, `winter_prep` 2개, `rain_wet_burn`, `reset_on_split`, `avoid_gather`, `exchange_first`.
  - 머리말: 모든 `src/**/*.rs`에 `//!` (시스템 28개는 Priority·Interval·Gate·Reads·Writes·Events), 모든 `.gd`에 목적 + 브리지 메서드 주석.
- 문서: `README.md`, `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`, `docs/ARCHITECTURE.md`(생성되는 시스템 표), `docs/decisions/0001`~`0011`, `docs/archive/INDEX.md`, `CHANGELOG.md`, `AGENTS.md`(78줄), `CLAUDE.md`(`@AGENTS.md` + 8줄), `docs/v8/` 기록(GOAL, DIRECTION, DECISIONS D1–D33, INVENTORY, BASE_NUMBERS, 이 보고서).
- 스킬 (규칙 수 / 교훈 수, 교훈마다 커밋 또는 `v7-final-art` 경로 출처):
  - `worldsim-architecture` 26 / 27
  - `worldsim-godot` 36 / 48
  - `worldsim-testing` 23 / 29
  - `worldsim-pitfalls` 16 / 51
  - `worldsim-docs-upkeep` 6 / 3
  - 일반 스킬 `systematic-debugging`, `verification-before-completion`은 그대로 옮김. `.agents/skills/<이름>`은 모두 상대 심볼릭 링크(git mode 120000).

### 구현 방법
1. **고정과 기준 데이터 (Part 0).** 두 태그를 밀고, 기준 커밋 `b1ca032f`의 분리 작업트리에서 identity 데이터(32세계 × 120일 probe 요약, 원시 표본 64개, 연대기 64개, 결정론), 규칙·월드젠 덤프, 브리지 API 목록, 성능, 스모크 수, 화면을 떴다.
2. **인벤토리 (Part 1).** `tools/v8/inventory.py`가 `v7-final-art`와 현재 추적 파일의 합집합 모든 경로에 판정(KEEP / DELETE / MOVE / ASK / TRIAGE)과 근거를 붙인다. GDScript 도달성은 `tools/v8/gd_reach.py`(`res://`, preload/load, class_name, ext_resource, JSON 경로, 파일 계열), 현지화 키는 `locale_keys.py`, 라이브 시스템은 ablation으로 확인했다. 게이트 15단계가 모든 추적 경로가 행으로 덮였는지 검사한다.
3. **수확 → 삭제 (Part 2).** 지우기 전에 역사·교훈·구조를 `docs/v8/harvest/`에 옮겨 적고(→ HISTORY, 스킬; Part 5 끝에 삭제, `83a4f2dd`에 남음), 영역별로 한 커밋씩 지웠다. Rust 묶음마다 빌드·테스트·clippy·identity, GDScript 묶음마다 가져오기·스모크.
4. **구조 정리 (Part 3).** 단계마다 identity(요약 228,748값, 연대기 64, 원시 64, 결정론)와 release 테스트, clippy, `quick` 지문을 확인한 뒤 커밋·푸시했다. 분할 두 개는 항목 집합과 정규화된 줄 집합 비교로 순수 이동임을 따로 확인했다.
5. **문서·스킬 (Part 4–5)**, **게이트·독립 검토·보고 (Part 6).** 서브에이전트(`general-purpose`)는 범위를 정해 맡기고 커밋은 리드가 경로를 지정해 했다.

### 기능 설명
- **사라진 것:** 신의 손 패널과 개입 7종(브리지 `apply_intervention`, `get_intervention_state`), 마을 소식 패널, 활동 배지·툴팁, 마을 이름표, 마을 관계선, 운반꾼 경로선, 탐색·죽음·욕구 막대·활동 궤적 표시, 안절부절 막대, 창설 무리 표시, 식량 장부, 줌 LOD. 게임 밖에서는 V7 문서·증거·도구·테스트, 연구 코드(세계 모델은 `codex/worldmodel-2026-10`), 패스 전용 장면과 규칙 버전 사슬.
- **남은 것:** 같은 세계(같은 시드면 사람 수·마을·건물이 같다), 사람·건물·나무, 상단 막대, 연대기 패널(C), 사람을 클릭하면 상세 정보, 빈 타일을 클릭하면 원인('왜?') 패널, 속도 조절.
- **사용자 관점 변화:** 화면이 깨끗해졌다(후보 화면 `$DATA/cand/shots/*_t1440.png` vs 기준 `$DATA/base/shots/`: 상단 막대 숫자 — 2일차 08:00, 인구 26, 거처 4, 식량 15, 장작 31, 열매 46 — 가 같고, 신의 손 패널·마을 소식·활동 배지·마을 이름표만 없다). 시뮬레이션 결과는 같다.

### 변경된 파일 목록
| 해시 | 메시지 | 영역 |
|---|---|---|
| `da359860` | v8-0(p0): add V8-0 goal and V8 direction documents unchanged | Part 0 고정·기준 데이터 |
| `217c122f` | v8-0(p0): pin v7-final rules/worldgen dumps, bridge API list and identity/perf/capture tooling | Part 0 고정·기준 데이터 |
| `501867ff` | v8-0(p0): record base numbers (perf, smoke counts) and the V8 smoke | Part 0 고정·기준 데이터 |
| `83a4f2dd` | v8-0(p1): inventory with verdicts for every path, harvest drafts, reachability and key tools | Part 1 인벤토리·수확 |
| `93ce74e1` | v8-0(p2): delete V7 docs, evidence, stats, harness state, hooks, harness addon and agents (7.1) | Part 2 삭제 |
| `53da844f` | v8-0(p2): delete V7 tool trees and one-off scripts; keep strict check and run_bounded in tools/v8 | Part 2 삭제 |
| `19baad71` | v8-0(p2): replace the committed .claude/settings.json (V7 harness hooks) with the neutral {} (7.6) | Part 2 삭제 |
| `e6b49757` | v8-0(p2): delete research code and pass scaffolding from sim-test (7.7) | Part 2 삭제 |
| `7875bb37` | v8-0(p2): remove the god hand (7.2) | Part 2 삭제 |
| `24615913` | v8-0(p2): delete the V7 and art-line Godot test code (scripts/test, scenes/test) | Part 2 삭제 |
| `2488b820` | v8-0(p2): keep only the V8 screen skeleton in the UI (7.3) | Part 2 삭제 |
| `152b65c8` | v8-0(p2): keep only the localization keys the kept UI shows (7.4) | Part 2 삭제 |
| `b1e428e0` | v8-0(p2): smoke and capture open the inspector, causal and chronicle panels | Part 2 삭제 |
| `f7c04f21` | v8-0(p2): delete unreferenced assets and shaders (7.5) | Part 2 삭제 |
| `a306d548` | v8-0(p2): delete the 135 MB art-kernel fixtures and the three tests that read them | Part 2 삭제 |
| `a9f35c73` | v8-0(p3): cargo fmt --all (8.8, its own commit at the start of Part 3) | Part 3 구조 |
| `892118b5` | v8-0(p3): one baseline instead of the game_vNN chain — add and switch (8.1a) | Part 3 구조 |
| `5364e936` | v8-0(p3): delete the game_vNN chain and port the tests to the baseline (8.1b) | Part 3 구조 |
| `4c30013a` | v8-0(p3): remove pass and version suffixes (8.2) | Part 3 구조 |
| `a92d6b80` | v8-0(p3): remove seven dead rule fields (8.3) | Part 3 구조 |
| `2701bd36` | v8-0(p3): tests by domain (8.4) | Part 3 구조 |
| `8fe572c9` | v8-0(p3): split the bridge node by concern (8.5) | Part 3 구조 |
| `3ddd228f` | v8-0(p3): GDScript layout (8.7) | Part 3 구조 |
| `93ecada4` | v8-0(p3): split the decision module and add module headers (8.6, 8.8) | Part 3 구조 |
| `347b0fd4` | v8-0(p3): GDScript header comments (8.8) | Part 3 구조 |
| `0b5e8d6e` | v8-0(p4): repository docs | Part 4 문서 |
| `cdcca88e` | v8-0(p5): agent instructions and skills | Part 5 스킬 |
| `4a1a2035` | v8-0(p6): the 16-step gate; final deletion counts | Part 6 게이트·검토 |
| `9ff8b4ce` | v8-0(p6): fixes from the independent diff review | Part 6 게이트·검토 |
| `672fffef` | v8-0(p6): docs fixes from the independent docs review | Part 6 게이트·검토 |
| (이 커밋) | v8-0(p6): V8-0 report | Part 6 보고서 (문서만; `v8/main`을 여기로 fast-forward) |

### Identity 결과
- 방법: `tools/v8/identity.sh` — probe `--scene game_v26`(후보에서는 `baseline`의 별칭) 시드 0..32, 120일, 시각 표본 8개, 120틱마다 원시 표본. 요약은 시간·`git_sha`를 뺀 모든 값 비교(한쪽에만 있는 경로 0개), 연대기는 `git_sha` 외 모든 필드, 원시 표본은 `git_sha`와 의도적으로 지운 0값 키 5개(개입 관련)를 뺀 sha256, 결정론 `--check-determinism` 시드 0–3.
- 단계별 통과 기록:

  | 단계 | 요약 | 연대기 | 원시 표본 | 결정론 | release 테스트 |
  |---|---|---|---|---|---|
  | 7.7 sim-test 삭제 | 231,180값 동일 | 64 동일 | 64 동일 | PASS | 1829/0 |
  | 7.2 신의 손 | 228,748값 동일 (0값 개입 경로 2,432개는 제거 키로 제외; 첫 비교는 비교 도구가 경로 구성요소를 보지 않아 FAIL → 도구 수정 후 같은 데이터로 PASS) | 64 동일 | 64 동일 | PASS | 1820/1 → 수정 후 통과 |
  | 8.8 fmt | 228,748 동일 | 64 | 64 | PASS | 1818/0 |
  | 8.1a 기준값 추가·전환 | 동일 | 64 | 64 | PASS | (5개 실패 → 8.1b에서 이식) |
  | 8.1b 사슬 삭제 | 동일 | 64 | 64 | PASS | 1751/0 |
  | 8.2 이름 | 동일 | 64 | 64 | PASS | 1750/0 |
  | 8.3 죽은 필드 | 동일 | 64 | 64 | PASS | 1750/0 |
  | 8.4 테스트 도메인 | 동일 | 64 | 64 | PASS | 1751/0 |
  | 8.5 브리지 분할 | 동일 | 64 | 64 | PASS | 1751/0 |
  | 8.6·8.8 결정 분할·머리말 | 동일 | 64 | 64 | PASS | 1751/0 |
  | 최종 게이트 (`672fffef`) | 228,748값 동일, 한쪽 경로 0 | 64 동일 (19,683 항목) | 64 동일 | PASS (시드 0–3) | release 1751/0, debug 1751/0 |

- 성능 `quick` 지문: 8.1 이후 모든 단계에서 `d4d9b81677e1ebcc` (= 기준의 `stress --rules game_v26`).

### 성능
`tools/v8/perf.sh` (게이트 12단계, 보고용). 기준과 후보 모두 같은 Mac(Apple M4 Max)에서 다른 작업과 함께 잰 값(경합 있음).

| 측정 | 기준 (`b1ca032f`) | 후보 (`672fffef`) | 비율 |
|---|---|---|---|
| natural 평균 틱 (Pass 21 방식: `natural --scene game_v26 --days 120`, 시드 19600/19603/19605/19606, 3회 중앙값) | 0.2773 ms | 0.2697 ms | ×0.973 |
| natural p99 | 0.611 ms | 0.572 ms | ×0.937 |
| natural p99.9 | 1.875 ms | 0.743 ms | ×0.396 (3회 반복의 꼬리값은 잡음이 크다 — 개선 주장 아님) |
| `quick` 평균 (기준선 stress, 시드 15100, 500명, 64×64, 먹이 공급, 1일) | 0.4105 ms (`stress --rules game_v26`) | 0.4121 ms | ×1.004 |
| `quick` p99 | 0.750 ms | 1.071 ms | ×1.428 (1일 1회 측정의 꼬리값; 평균은 같음) |

- 지문: `quick` = `d4d9b81677e1ebcc` = 기준 `stress --rules game_v26`; natural 지문도 기준과 같음.
- WARN: 없음 (평균이 +10 %를 넘는 항목 없음). 예비 게이트(`4a1a2035`)에서도 natural ×0.945, quick ×0.955였다.
- `quick` 파일 이름: `quick_stress_baseline_s15100_n500_side64_fed.json` (기준은 `quick_stress_v19_…`).

### 문서 검토 결과
독립 검토자(새 `general-purpose` 에이전트, README·`docs/`(`docs/v8/` 제외)·스킬만 읽음) 2회:

| # | 질문 | 1회 | 2회 | 결과 |
|---|---|---|---|---|
| 1 | 관계 시스템 위치·우선순위 | 정답 | — | 정답 (`relations.rs`, 149) |
| 2 | 시스템 추가 | 정답 (작은 차이 3개) | — | 정답, 차이 수정 |
| 3 | 브리지 읽기 추가 | 부분 (API 목록 갱신 방법 모순) | 부분 (ARCHITECTURE 쪽 단계 누락) | 2회 모두 부분 → 2회 뒤 수정 (ARCHITECTURE §6을 스킬과 같은 단계로 보완), 재질문은 하지 않음(2회 제한) — 확인되지 않은 수정 |
| 4 | 푸시할 브랜치 | 정답 | — | 정답 |
| 5 | 게이트 | 정답 | — | 정답 |
| 6 | 신의 손 행방 | 정답 | — | 정답 (ADR 0010) |
| 7 | 시간 척도 | 부분 (지금 빌드 값 없음) | 부분 (목표의 틱 정의 없음) | 2회 모두 부분 → 2회 뒤 수정 (지금 48초/일·28일/년, 목표 2분/일·24일/년, 목표의 틱 정의는 미정이라고 명시), 재질문은 하지 않음 — 확인되지 않은 수정 |
| 8 | 연구 코드 위치 | 정답 | — | 정답 |
| 9 | 패스 후 할 일 | 정답 | — | 정답 |
| 10 | Godot 교훈 기록 위치 | 정답 | — | 정답 |

모순 10건(시간 척도 표기, 덤프 경로, API 목록, 가져오기 횟수, Godot 쪽 명령 목록, 등록 함수, 스킬 수, 공유 파일의 시스템 표, 깨진 교훈 라벨, 존재하지 않는 경로 표기)은 모두 고쳤다(D32). 변경 검토자(인벤토리 + `git diff --stat b1ca032f..HEAD`)의 발견 F1–F4와 ASK 판단은 D31: F1(사람 상세의 마을 줄·마을 링크·선택 윤곽이 조용히 죽어 있던 회귀)은 고치고 기준과 같은 값임을 확인했다.

### 확인된 제한사항
- **M1에서 교체할 의존 파일 (7.3 유지 목록이 아니지만 남긴 것, D20·D31):** `scripts/world/`의 `agent_activity.gd`, `activity_icons.gd`(+ `assets/art/production/U1/04.png`), `porter_presentation.gd`, `offscreen_marker.gd`, `art_hash.gd`, `art_season_tint.gd`, `art_sprite_library.gd`, `camp_seats.gd`, `carry_load_layer.gd`, `cold_effects.gd`(+ `cold_audit.json`), `morning_chill.gd`, `agent_frame_choices.gd`, `agent_pose_moments.gd`, `agent_selection_index.gd`, `shelter_cold.gd`, `wood_visuals.gd`, `nature_layout.gd`, `declump_offsets.gd`, `walker_offsets.gd`, `nature_renderer.gd`·`nature_canopy.gd`(M1 타일 층이 대체); `scripts/core/settlement_selection.gd`, `scripts/core/sim_reads.gd`; `scripts/ui/chronicle_text.gd`, `person_names.gd`, `person_read_cache.gd`, `panels/art_ui.gd`. 이 파일들 안에는 지운 UI만 쓰던 죽은 경로가 있다(D31 F2: 레이어·캐스트 헬퍼, 타입 래퍼, 성능 후보 setter, 활동 배지 경로).
- **드롭한 단계:** 없음 (8.3·8.5·8.6·8.7 모두 수행). 8.3은 조건 셋을 다 만족하고 probe가 출력하지 않는 필드 7개만 지웠고, probe가 출력하거나 테스트가 반대 값을 쓰는 필드는 남겼다(목록 D25, `docs/v8/p3/8_3_candidates.md`). 8.6은 `tick` 함수 하나가 약 2,900줄이라 함수 밖 항목만 옮겼다.
- **1,200줄 초과 `src` 파일:**

  | 파일 | 줄 |
  |---|---|
  | `rust/crates/sim-systems/src/runtime/decision/system.rs` | 3,462 |
  | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | 1,508 |
  | `rust/crates/sim-bridge/src/ffi/snapshot.rs` | 1,415 |
  | `rust/crates/sim-test/src/perf.rs` | 1,351 |
  | `rust/crates/sim-engine/src/lib.rs` | 1,303 |
- **크기 예외:** `assets/fonts/galmuri/Galmuri11.ttf`(5.4 MB)가 "5 MB 넘는 추적 파일 금지" 규칙의 유일한 예외(D27).
- **기타:** 테스트 수는 `sim-test/tests`에 여전히 7만 줄이 넘는다(V7 단계별 하네스를 도메인으로 묶기만 함). 현재 달력은 V7 그대로(하루 1,440틱 = 1배속 48초, 28일/년)이고 ADR 0004의 압축 연도는 아직 일정이 없다. 작업 디렉터리에 추적되지 않는 이전 잔여물(`docs/revitalize/`, `addons/harness/`, `.omc/`)이 남아 있다(커밋되지 않음, 오너가 지워도 된다).

### 오너 결정 필요
- **ASK 모듈 (권고):**

  | 항목 | 권고 |
  |---|---|
  | `rust/crates/sim-systems/src/runtime/combat/**` (CombatSystem, 우선순위 137) | 유지. 120일 4세계에서 결과를 바꾸지 않음. M5에서 짐승 위협·소집이 새 전투 모델을 쓰면 그때 삭제(원인 패널의 전투 분기, 키 5개, 전투 테스트 포함) |
  | `shaders/palette_swap.gdshader`, `assets/sprites/palette_lut.png`, `assets/sprites/agent_base.png` | 유지(VISION §5 "팔레트 교체 셰이더"). §5가 이 LUT 묶음인지 지금 쓰는 `villager_ramp.gdshader`인지 오너가 정하면 M1이 다른 쪽을 지움 |
  | `assets/sprites/buildings/**`, `assets/sprites/furniture/**` (174개) | M1/M2 아트에서 새 크기 재사용 여부를 검토하고 안 쓰는 것은 삭제 (VISION §6.3) |
  | (해결됨) `nature_renderer.gd`, `nature_canopy.gd` | 살아 있음 → KEEP, M1 타일 층이 대체 |
- **원격 브랜치 정리 제안** (아무것도 지우지 않았다; "포함" = 그 태그의 조상):

  | 브랜치 | 마지막 커밋 | 날짜 | `v7-final-art` 포함 | `v7-final-sim` 포함 | 권고 |
  |---|---|---|---|---|---|
  | `archive/pre-v7-reset` | `71d5fa6e` chore(archive): preserve mega-day untrac | 2026-05-04 | no | no | 유지 (V7 리셋 전 보관본) |
  | `art-p13b-ui` | `a5771f1f` art-p13b(p1): enable walker only after e | 2026-10-19 | yes | no | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/a3-effect-queue-runtime` | `9bbf1c40` [t-000] add effect queue runtime system | 2026-03-13 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/a6-building-grid-hardening` | `e77af441` [t-000] add data-driven building constru | 2026-03-13 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/a8-temperament-ron` | `d602b416` [t-000] wire temperament pipeline to RON | 2026-03-13 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/a9-world-rules-runtime` | `58e5a760` [t-000] wire world-rules runtime (channe | 2026-03-13 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/art-direction-2026-10` | `b5ead380` Separate recovered visual strengths from | 2026-10-10 | yes | no | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/art-production-2026-10` | `b1ca032f` art-p13b(p5): publish verified evidence  | 2026-10-20 | yes | no | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/art-production-2026-10-wip` | `b1ca032f` art-p13b(p5): publish verified evidence  | 2026-10-20 | yes | no | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/inventory-foundation` | `2eb9e3cc` [t-000] add inventory foundation (item t | 2026-03-13 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/item-store-inventory-ui` | `06500496` [t-000] expose item-store inventory to e | 2026-03-14 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/lead-main-to-main-check` | `89afdf5d` Merge remote-tracking branch 'origin/lea | 2026-03-14 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/probe-runtime-verification-overlay` | `d5026640` [t-000] add probe runtime verification o | 2026-03-10 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/probe-survival-need-indicators-pass` | `e7f8f638` [t-000] add probe survival need indicato | 2026-03-11 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/refactor-core-foundations` | `96f2f068` [t-000] scaffold core simulation foundat | 2026-03-11 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/refactor-sim-authority-cutover` | `8236f8ec` [t-000] tighten final simulation authori | 2026-03-11 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/refactor-ws-full-cleanup` | `6d02e602` [t-000] complete ws full cleanup audit p | 2026-03-11 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/refactor-ws-ref-004a` | `10003064` [t-000] secure boot authority audit | 2026-03-11 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/refactor-ws-ref-004b` | `61d82511` [t-000] finalize typed runtime registry | 2026-03-11 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/revitalize-2026-10` | `354e4a13` revitalize(p21-3): REPORT_PASS21.md (Kor | 2026-10-19 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/revitalize-2026-10-wip` | `79513cef` revitalize(p21-3): final gate record (35 | 2026-10-19 | no | no | 삭제 가능 검토 (`354e4a13` 이후 게이트 기록 커밋만 있음; 확인 후 삭제) |
  | `codex/root-markdown-cleanup-hold-20260312` | `7b0b760d` [t-000] fix stale root markdown referenc | 2026-03-12 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/worldmodel-2026-10` | `58cf8611` worldmodel(wm6-7): review round 2 and re | 2026-10-20 | no | no | 유지 (세계 모델 연구가 계속되는 곳) |
  | `codex/worldmodel-2026-10-wip` | `f096e13a` worldmodel(wm6-7): REPORT.md wording (KS | 2026-10-20 | no | no | 유지 (세계 모델 연구가 계속되는 곳) |
  | `codex/ws-chr-mig-006-publish` | `fce7458b` [t-000] add chronicle snapshot family | 2026-03-12 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/ws-exp-001-chronicle-hook` | `784aae05` [t-000] add chronicle hook baseline | 2026-03-12 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/ws-exp-002-chronicle-summarization` | `5a0d6112` [t-000] add chronicle summarization engi | 2026-03-12 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/ws-exp-006-attention-budget` | `5e43920d` [t-000] add chronicle attention budget | 2026-03-12 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `codex/ws-feat-003-shelter-warmth-bias` | `fe87c919` [t-000] add shelter warmth bias vertical | 2026-03-11 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/ws-feat-004-social-gathering` | `45dff367` [t-000] add social gathering vertical sl | 2026-03-12 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/ws-ref-005-influence-runtime` | `3928c7bd` [t-000] implement influence grid runtime | 2026-03-11 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `codex/ws-sys-001-steering-layer` | `f869ad33` [t-000] unify influence steering layer | 2026-03-12 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `lead/main` | `d409e9b9` harness: fix cross-run stale-residue con | 2026-06-24 | yes | yes | 유지 (V7 줄기; 규칙상 건드리지 않음) |
  | `main` | `ca6df234` Merge pull request #509 from hyunlord/le | 2026-05-04 | no | no | 유지 (V7 줄기; 규칙상 건드리지 않음) |
  | `t/2002-integration-bugfix` | `cfef9d39` fix: integration bugfix — death tracking | 2026-02-16 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `t/2003-urgent-fixes` | `e3a1ae9d` fix: parent_ids crash, death counter, re | 2026-02-16 | yes | yes | 삭제 가능 (태그에 포함되어 잃는 것 없음) |
  | `t/2005-population-balance` | `7a184234` feat: population balance overhaul with a | 2026-02-16 | no | no | 오너 검토 (태그에 없는 2026-02~03 실험; 필요 없으면 삭제) |
  | `v8/wip` | 이 보고서 커밋 | 2026-10-20 | no | no | 유지 (V8 작업 브랜치) |
  | `v8/main` | 이 보고서 커밋 (게이트 커밋 `672fffef`에서 생성) | 2026-10-20 | no | no | 유지 (V8 기준 브랜치, 게이트 통과 커밋으로만 fast-forward) |

- **저장소 용량:** 목표 문서 기준 팩 5.25 GiB(이 클론의 `git count-objects` 팩은 718 MiB — 다른 클론이 기준인 것으로 보임). 트리는 2,533 MiB → 18.5 MiB가 됐지만 이력 때문에 클론 크기는 그대로다. 선택지(아무것도 하지 않았다):
  - (a) 저장소를 그대로 두고 앞으로 큰 파일만 막는다 (게이트 14단계가 5 MB 초과 추적 파일을 막음). 비용 0, 클론은 계속 큼.
  - (b) `v8/main` 트리로 새 저장소를 시작하고 지금 저장소는 보관한다. 클론이 작아지고 이력은 옛 저장소와 태그로 찾는다.
  - (c) 이력 재작성(filter-repo로 큰 경로 제거). 모든 해시가 바뀌고 force-push가 필요해 규칙상 오너만 할 수 있다.
  - 권고: (b) 또는 (a). (c)는 태그·브랜치 해시를 모두 바꾸므로 권하지 않는다.
- **`.gitignore` 제안** (로컬 `.gitignore` 수정은 커밋하지 않았다):
  - `docs/**/*.png`, `docs/**/*.jpg` 무시, 단 `!docs/images/**` (증거 이미지가 다시 쌓이지 않게).
  - 죽은 예외 3줄 삭제: `!scripts/ui/panels/oracle_panel.gd.uid`, `!scripts/ui/panels/band_detail_panel.gd.uid`, `!scripts/ui/panels/civilization_detail_panel.gd.uid` (파일이 없음).
  - `*.uid` 정책: Godot 4.4+는 `.uid` 커밋을 권장한다. 지금은 무시되어 체크아웃마다 UID가 새로 생긴다. 커밋으로 바꿀지 결정 필요(바꾸면 `.uid` 38개를 추가 커밋).
  - 로컬 도구 상태: `/graft/`, `.omc/`(이미 있음), `addons/harness/`.

### Gate 결과
`tools/v8/gate.sh` — 커밋 `672fffef`, 실행 중 작업트리 변경 없음(dirty 0), 2026-10-20 17:52–18:41 KST. 로그: `/Users/rexxa/work/v8-0-data/gate/final/` (단계별 `<step>.log`, 요약 `gate.txt`).

| # | 단계 | 결과 | 시간 | 내용 |
|---|---|---|---|---|
| 1 | `fmt` | 통과 | 0s | `cargo fmt --all -- --check` |
| 2 | `bridge_build` | 통과 | 7s | `cargo build -p sim-bridge` |
| 3 | `godot_import` | 통과 | 5s | 헤드리스 가져오기 2회 |
| 4 | `test_debug` | 통과 | 1611s | debug 테스트 1751/0 |
| 5 | `test_release` | 통과 | 101s | release 테스트 1751/0 |
| 6 | `clippy` | 통과 | 0s | `--all-targets -D warnings` |
| 7 | `forbidden_math` | 통과 | 0s | sim-core/systems/engine 부동소수 초월함수 0건 |
| 8 | `gdscript_strict` | 통과 | 9s | 38개 파일, FFI 호출 27개 일치 |
| 9 | `smoke` | 통과 | 14s | 900프레임, SCRIPT ERROR 0 / ERROR 0 / 키 경고 0 / 원시 키 0 (기준 0/0/0/0) |
| 10 | `identity` | 통과 | 497s | 요약 228,748값·연대기 64·원시 64 동일 |
| 11 | `determinism` | 통과 | 131s | 시드 0–3 PASS |
| 12 | `perf` | 통과 | 559s | WARN 없음 (보고용) |
| 13 | `bridge_api` | 통과 | 0s | 37 = 기준 39 − 개입 2 |
| 14 | `docs` | 통과 | 1s | `check_docs.sh` PASS |
| 15 | `inventory_closure` | 통과 | 0s | 추적 경로 714개 모두 행 있음 |
| 16 | `python_unittest` | 통과 | 0s | 9개 통과 |

보고서 커밋에서는 문서 단계(14, `bash tools/v8/check_docs.sh`)만 다시 돌렸다(바뀐 파일은 이 보고서·CHANGELOG·HISTORY뿐, 모두 문서): `docs: PASS` (로그 `/Users/rexxa/work/v8-0-data/gate/report_docs.log`). 예비 게이트(`4a1a2035`, 17:00–17:51)도 16단계 모두 통과했다(`/Users/rexxa/work/v8-0-data/gate/pre1/`).
