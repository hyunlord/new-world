## Implementation Intent

WS-EXP-001 already recorded raw influence-driven steering events, but those raw per-tick events were still too noisy for players or developers to read as world history. This ticket adds the missing middle layer: deterministic summarization that compresses related low-level events into bounded, explainable chronicle entries.

The design stays lightweight on purpose. It does not generate freeform narrative text and it does not change simulation outcomes. Instead it clusters recent chronicle events, filters them by significance, stores only bounded summaries, and exposes those summaries through the bridge so the UI can render readable history without guessing.

## How It Was Implemented

The chronicle model was extended in [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-engine/src/chronicle.rs](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-engine/src/chronicle.rs). I added typed `ChronicleCluster`, `ChronicleSummary`, and `ChronicleTimeline` structures. `ChronicleTimeline` is a bounded `VecDeque` with append, recent query, per-entity query, and pruning behavior capped by `CHRONICLE_TIMELINE_MAX_ENTRIES`.

Periodic summarization was added in [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-systems/src/runtime/record.rs](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-systems/src/runtime/record.rs). `ChronicleRuntimeSystem` now runs summarization every `CHRONICLE_SUMMARY_INTERVAL_TICKS`, reads recent raw events from `ChronicleLog`, builds social gathering group clusters, clusters per-entity events by temporal proximity and cause, scores clusters, rejects low-significance noise, and appends accepted summaries to the bounded timeline. Debug logs were added for cluster creation, rejection, summary generation, and timeline pruning.

The simulation resource layer was updated in [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-engine/src/engine.rs](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-engine/src/engine.rs) so `SimResources` owns `chronicle_timeline` alongside the existing raw `chronicle_log`.

Bridge exposure was implemented in [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-bridge/src/lib.rs](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-bridge/src/lib.rs) and [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-bridge/src/runtime_queries.rs](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/rust/crates/sim-bridge/src/runtime_queries.rs). The bridge now exposes a typed runtime getter for recent chronicle timeline entries and includes per-entity recent summaries in entity detail. Bootstrap reset also clears the timeline so old summaries do not leak across worlds.

The UI side was switched to consume runtime summaries in [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/scripts/core/simulation/sim_bridge.gd) and [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/scripts/ui/panels/chronicle_panel.gd](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/scripts/ui/panels/chronicle_panel.gd). The chronicle panel now prefers runtime summaries when the runtime is initialized and falls back to the old legacy source only when runtime chronicle data is unavailable. New locale keys for food, danger, warmth, and social summary templates and filters were added in [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/localization/en/ui.json](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/localization/en/ui.json) and [/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/localization/ko/ui.json](/Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization/localization/ko/ui.json).

## What Feature It Adds

WorldSim now has a readable history layer instead of only raw steering traces. Repeated low-level actions like moving toward food, fleeing danger, seeking shelter, or clustering socially can be compressed into stable chronicle summaries. That makes influence-driven behavior observable and gives later narrative systems a bounded, structured event timeline to build on.

It also gives the existing UI an authoritative runtime history source. Instead of relying only on legacy chronicle code paths, the panel can now show Rust-owned summaries that explain why an agent moved or why a group formed.

## Verification After Implementation

Commands run:

- `cd rust && cargo check --workspace` — PASS
- `cd rust && cargo test -p sim-systems runtime::record::tests -- --nocapture` — PASS
- `cd rust && cargo test -p sim-bridge chronicle -- --nocapture` — PASS
- `cd rust && cargo test --workspace` — PASS
- `cd rust && cargo clippy --workspace -- -D warnings` — PASS
- `cd rust && cargo build -p sim-bridge` — PASS
- `git diff --check` — PASS
- `'/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot' --headless --path /Users/rexxa/github/new-world-wt/codex-ws-exp-002-chronicle-summarization --quit` — PASS

Key behavioral evidence:

- low-significance chronicle clusters are rejected
- danger-driven chronicle events collapse into a readable escape summary
- social chronicle events from multiple nearby entities collapse into a single group summary
- timeline storage remains bounded and prunes oldest entries correctly
- runtime entity detail and runtime chronicle timeline both expose summary data through sim-bridge
- Godot boot still succeeds with the chronicle panel switched to runtime-backed summaries

## In-Game Checks (한국어)

- 에이전트 이동 이유가 Chronicle 패널이나 상세 정보에 실제로 기록되는지 확인한다.
- 음식 때문에 이동한 경우 `Food` 원인의 Chronicle summary가 생성되는지 본다.
- 위험 때문에 도망간 경우 `Danger` 원인의 Chronicle summary가 생성되는지 본다.
- 캠프파이어 주변의 여러 에이전트 움직임이 각각 따로 찍히지 않고 하나의 `Social` gathering 사건으로 묶이는지 확인한다.
- 추위 때문에 shelter 쪽으로 이동한 경우 `Warmth` 관련 summary가 남는지 본다.
- 단순 랜덤 이동이나 약한 흔들림은 Chronicle에 과하게 쌓이지 않는지 확인한다.
- Chronicle 패널이 시간 순서대로 안정적으로 보이고, 오래된 항목이 무한정 늘어나지 않는지 확인한다.
- 시뮬레이션이 계속 진행될 때 Chronicle이 성능을 눈에 띄게 떨어뜨리지 않는지 확인한다.

## Remaining Risks

- 현재 summary 문장은 locale key + params 구조까지만 구현되어 있고, richer narrative phrasing이나 timeline UX polish는 아직 없다.
- significance model은 첫 baseline이다. 실제 장시간 플레이에서는 threshold나 social cluster scoring을 더 다듬을 여지가 있다.
- 현재 chronicle summarization은 steering 기반 영향 사건 중심이다. settlement milestone, births, deaths, long-form world events 같은 broader summary coverage는 후속 티켓이 필요하다.
- runtime chronicle panel은 runtime 초기화 시 summary path를 쓰고, runtime이 없을 때만 legacy fallback을 유지한다. 완전한 legacy chronicle 제거는 별도 migration 범위다.
