## Implementation Intent

Chronicle 요약은 이미 있었지만, 현재 구조는 의미 있는 사건과 반복적인 사건을 같은 timeline에 계속 누적하는 쪽에 가까웠다. 이 티켓에서는 Chronicle을 단순 압축 로그가 아니라 플레이어 주의력을 관리하는 편집 레이어로 만들기 위해, visible/background/recall attention budget과 runtime-only authority 정리를 넣었다.

핵심 목표는 두 가지였다. 첫째, visible Chronicle을 최대 5개로 제한하면서 `CRITICAL`, `MAJOR`, `NOTABLE`를 분리해 중요한 사건이 먼저 보이게 하는 것. 둘째, UI가 더 이상 legacy `ChronicleSystem`을 조용히 읽지 않고 Rust runtime timeline만 authoritative source로 읽도록 만드는 것이다.

## How It Was Implemented

- `rust/crates/sim-engine/src/chronicle.rs`
  - `ChronicleSignificanceCategory`를 추가해 `Ignore`, `Minor`, `Notable`, `Major`, `Critical`로 요약 중요도를 분류했다.
  - `ChronicleSummary`에 `category` 필드를 추가했다.
  - `ChronicleTimeline`을 단일 `VecDeque`에서 `visible_queue`, `background_queue`, `recall_queue` 3개 queue로 바꿨다.
  - `route_summary()`로 attention budget 규칙을 구현했다.
    - `Critical`은 항상 visible
    - `Major`는 visible slot이 남을 때만 visible, 아니면 recall
    - `Notable`은 background
    - `Minor`/`Ignore`는 drop
  - `promote_background_if_starved()`를 추가해 일정 시간 visible surface가 비면 background의 최고 significance 항목을 visible로 올리도록 했다.
  - `recent_family_count()`를 추가해 같은 `(event_type, cause)` family 반복 여부를 추적했다.

- `rust/crates/sim-core/src/config.rs`
  - `CHRONICLE_VISIBLE_MAX_ENTRIES = 5`
  - `CHRONICLE_RECALL_MAX_ENTRIES`
  - `CHRONICLE_SUMMARY_NOTABLE_THRESHOLD`
  - `CHRONICLE_SUMMARY_CRITICAL_THRESHOLD`
  - `CHRONICLE_REPEAT_SUPPRESSION_WINDOW_TICKS`
  - `CHRONICLE_REPEAT_SUPPRESSION_STEP`
  - `CHRONICLE_VISIBLE_STARVATION_TICKS`
  를 추가해 attention budget과 suppression 정책을 config 상수로 뺐다.

- `rust/crates/sim-systems/src/runtime/record.rs`
  - cluster score에서 기존 raw significance에 recent family repeat penalty를 적용하도록 `adjusted_cluster_significance()`를 추가했다.
  - `chronicle_category()`로 score를 attention category로 변환했다.
  - 기존처럼 `score >= MAJOR`만 timeline에 넣는 대신, `Notable`까지 summary를 만들고 routing 단계에서 background로 보내도록 바꿨다.
  - 요약 append를 `append_chronicle_summary(resources, summary, surfaced_tick)`로 바꾸고, 실제 queue 결과에 따라
    - `[Chronicle] summary_surfaced`
    - `[Chronicle] summary_backgrounded`
    - `[Chronicle] summary_recalled`
    - `[Chronicle] summary_suppressed`
    - `[Chronicle] timeline_pruned`
    로그를 남기도록 했다.
  - summarization pass 끝에서 anti-starvation promotion을 실행하도록 연결했다.

- `rust/crates/sim-bridge/src/lib.rs`
  - `chronicle_summary_to_dict()`에 `category_id`를 추가해서 UI/debug 소비자가 summary tier를 읽을 수 있게 했다.
  - `runtime_get_chronicle_timeline()`은 이제 active visible queue만 노출한다. background/recall은 runtime 안에 남고, UI의 authoritative main timeline source는 visible queue다.

- `scripts/ui/panels/chronicle_panel.gd`
  - runtime chronicle만 authoritative source로 사용하도록 바꿨다.
  - legacy `ChronicleSystem` silent fallback을 제거했다.
  - runtime이 없는 경우에는 빈 배열을 반환하고, `[Chronicle] runtime timeline unavailable; legacy ChronicleSystem fallback is disabled` warning을 한 번만 남기도록 했다.
  - filter options도 runtime chronicle 기준으로 고정했다.

## What Feature It Adds

Chronicle은 이제 “모든 summary를 그냥 쌓는 패널”이 아니라, 실제로 중요한 사건만 전면에 드러내는 attention budget 시스템이 된다.

- 동시에 보이는 active Chronicle은 최대 5개로 제한된다.
- danger 같은 높은 significance summary는 active visible queue를 우선 차지한다.
- 중간 중요도 summary는 background에 남고, 너무 오래 화면에 아무 일도 없으면 자동으로 한 건이 visible로 승격된다.
- 반복되는 동일 family 사건은 repeat penalty를 받아 surface priority가 내려간다.
- UI는 더 이상 legacy chronicle source를 조용히 섞지 않고, Rust runtime chronicle만 읽는다.

## Verification After Implementation

- `cd rust && cargo check --workspace` — PASS
- `cd rust && cargo test -p sim-engine chronicle -- --nocapture` — PASS
- `cd rust && cargo test -p sim-systems runtime::record::tests -- --nocapture` — PASS
- `cd rust && cargo test --workspace` — PASS
- `cd rust && cargo clippy --workspace -- -D warnings` — PASS
- `cd rust && cargo build -p sim-bridge` — PASS
- `if rg -n "\\btr\\(" scripts/ui -g '*.gd'; then ...` — PASS
- `git diff --check` — PASS
- `Godot --headless --path /Users/rexxa/github/new-world-wt/codex-ws-exp-006-attention-budget --quit` — PASS

추가 확인한 behavior evidence:

- visible queue는 `CHRONICLE_VISIBLE_MAX_ENTRIES`를 넘지 않는다.
- visible이 가득 찬 상태에서 `MAJOR`는 recall queue로 가고, `CRITICAL`은 visible을 유지하면서 기존 visible을 recall로 밀어낸다.
- repeated family는 repeat penalty를 받아 `MAJOR -> NOTABLE`로 내려갈 수 있다.
- starved 상태에서는 background summary가 visible로 승격된다.
- Chronicle panel은 runtime이 없을 때 legacy source를 읽지 않고 warning만 남긴다.

## Remaining Risks

- repeat suppression은 현재 `(event_type, cause)` family 기준이다. 더 정교한 “같은 장소/같은 actor/같은 상황” 중복 판단은 이후 significance engine 단계에서 확장해야 한다.
- background/recall queue는 runtime에 존재하지만, 지금 UI는 visible queue만 전면적으로 보여 준다. background/recall inspection UI는 아직 없다.
- anti-starvation은 “최근 visible surface가 없을 때 background 최고 score를 승격”하는 최소 규칙이다. 더 세밀한 editorial policy는 이후 ticket이 맡아야 한다.
- `chronicle_panel.gd`는 authority cleanup을 위해 legacy fallback을 끊었지만, richer runtime-unavailable UX는 아직 없다.

## In-Game Checks (한국어)

다음 항목을 실제 게임에서 확인해야 한다.

1. Chronicle에 동시에 너무 많은 사건이 표시되지 않는지 확인  
   화면에 보이는 active Chronicle이 과도하게 쌓이지 않고, 최대 5개 수준으로 유지되는지 본다.

2. 중요한 사건이 항상 화면에 보이는지 확인  
   위험 탈출 같은 강한 사건이 생기면 덜 중요한 사건보다 먼저 visible Chronicle에 올라오는지 본다.

3. 사건이 없는 시간이 길어지면 자동으로 의미 있는 사건이 올라오는지 확인  
   한동안 visible Chronicle이 비어 있으면 background에 있던 의미 있는 사건이 다시 surface되는지 본다.

4. 같은 종류의 사건이 반복될 때 Chronicle에 계속 올라오지 않는지 확인  
   예를 들어 비슷한 food attraction 사건이 연속으로 나올 때 전부 visible에 반복해서 올라오지 않는지 본다.

5. runtime Chronicle과 UI 표시 내용이 동일한지 확인  
   panel에 보이는 Chronicle이 legacy `ChronicleSystem`이 아니라 Rust runtime timeline에서 온 내용인지 확인한다.

6. runtime이 없을 때 legacy fallback을 조용히 타지 않는지 확인  
   chronicle panel이 비어 있더라도 예전 legacy path를 silently 섞지 않고 warning만 남기는지 본다.
