# WorldSim Full Repository Refactor Report

## Implementation Intent

This refactor pass aligned the repository with the real WorldSim authority model
without pretending the migration is more complete than it is. The goal was to:

- map the repository truth
- confirm Rust ECS owns active simulation authority
- confirm RON `DataRegistry` is the authoritative content path
- reduce a few clear pieces of boot/runtime residue safely
- document the remaining shadow/bootstrap debt precisely

## How It Was Implemented

The work was done as a Ralph-style loop:

1. scan the repository and write truth reports
2. compare active boot/runtime/data paths against the target architecture
3. apply only small safe refactors where the authority boundary was already clear
4. rerun verification
5. rescan and update the reports to match repository truth

Safe code refactors in this pass:

- runtime registry deterministic ordering now uses typed `RuntimeSystemId`
  instead of a string-name tie-break
- legacy JSON compatibility loading in `sim-data::load_all()` now clearly logs
  that it is non-authoritative
- dead `Pathfinder` boot residue was removed from `main.gd`
- some live presentation paths were moved off legacy manager references where
  runtime-backed data was already available:
  - `building_renderer`
  - `camera_controller`
  - `minimap_panel`

## What Feature It Adds

This work does not add gameplay. It adds architectural clarity.

After this pass, the repository is easier to reason about because:

- the active simulation authority path is explicitly documented
- the authoritative data path is explicitly documented
- the runtime registry no longer depends on string-key ordering for scheduler determinism
- several live Godot presentation paths depend less on shadow managers
- remaining migration debt is documented instead of hidden

## Verification After Implementation

- `cargo check --workspace`: PASS
- `cargo build -p sim-bridge`: PASS
- `cargo test --workspace`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS
- Godot headless boot: PASS
- `git diff --check`: PASS

Existing non-blocking warnings remain:

- `main.tscn` ext-resource UID fallback warnings
- shutdown-time `ObjectDB/resources still in use` warnings

## Remaining Technical Debt

- Godot still instantiates several legacy/shadow managers in `main.gd`
- some panels still rely on legacy manager graphs
- JSON compatibility loaders still exist for names, personality bootstrap, and legacy tests
- `sim_core::config` still holds many tuning constants that should eventually move into RON/world rules
- performance instrumentation exists, but there is still no dedicated benchmark harness
- the repository is not yet a pure Rust-only UI/render shell

## Next Recommended Refactors

1. remove remaining live legacy manager dependencies from HUD/detail panels
2. narrow JSON compatibility to explicit legacy-only modules/tests
3. migrate more `config.rs` tuning into world-rules RON
4. add a dedicated headless performance benchmark harness
5. continue deleting dead GDScript shadow systems only after their consumers are proven gone

## In-Game Checks (한국어)

- 게임이 정상적으로 부팅되는지 확인한다.
- 초기화 직후 시뮬레이션 tick이 계속 Rust 쪽에서만 진행되는지 확인한다.
- 에이전트가 정상적으로 spawn 되고 움직이는지 확인한다.
- HUD, 미니맵, 건물 렌더러가 runtime 데이터를 정상적으로 읽는지 확인한다.
- 선택 패널과 디버그 패널이 데이터를 보여 주지만 시뮬레이션 상태를 직접 바꾸지 않는지 확인한다.
- 만약 부팅은 되지만 패널 값이 비거나 오래된 값처럼 보이면, 남아 있는 legacy manager 의존 경로를 먼저 의심한다.
