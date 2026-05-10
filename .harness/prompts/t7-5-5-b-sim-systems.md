# T7.5.5.B: sim-systems Crate Scaffold

## Implementation Intent

V7 reset 후 sim-systems crate 신설 (empty scaffold).
Phase 2 4 RuntimeSystems (T7.6에서 land 의무)의 home.

근거:
- sim-systems의 본질 = behavior crate (RuntimeSystem implementations)
- T7.5.5.A에서 sim-engine crate (RuntimeSystem trait + SimResources) 신설됨 (1ab45b71)
- sim-systems는 sim-engine + sim-core dependency
- Phase 0 v0.1.3 patch Section 4.3 정확 base

이 작업은 **empty scaffold** — actual systems 구현 X.
T7.6에서 4 RuntimeSystems (BuildingStamp + InfluenceUpdate + AgentSample + Visualization) land 의무.

## What to Build

Path: `rust/crates/sim-systems/`

### Files (3 신설 + 1 수정)

#### 1. `rust/crates/sim-systems/Cargo.toml` (신설)

```toml
[package]
name = "sim-systems"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
hecs = { workspace = true }
serde = { workspace = true }
sim-core = { path = "../sim-core" }
sim-engine = { path = "../sim-engine" }
```

#### 2. `rust/crates/sim-systems/src/lib.rs` (신설)

```rust
//! WorldSim runtime systems.
//!
//! V7 reset 후 sim-systems crate (T7.5.5.B). Phase 2 4 RuntimeSystems
//! (T7.6에서 land 의무)의 home:
//! - InfluenceUpdateSystem (priority 100, Hot/Warm/Cold dispatch)
//! - BuildingStampSystem (priority 90, event-driven)
//! - AgentInfluenceSampleSystem (priority 110, agent ECS query)
//! - InfluenceVisualizationSystem (priority 1000, every 6 ticks)
//!
//! Phase 0 v0.1.3 patch Section 4.3 base.
//!
//! Dependencies:
//! - [`sim_core`] — ECS components, materials, influence, tile data
//! - [`sim_engine`] — `RuntimeSystem` trait + `SimResources` host

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod runtime;
```

#### 3. `rust/crates/sim-systems/src/runtime/mod.rs` (신설)

```rust
//! Runtime systems organization.
//!
//! Phase 2 (T7.6에서 land 의무):
//! - `pub mod influence;` (4 RuntimeSystems)
//!
//! Future phases:
//! - Phase 4 (Agent Core): `pub mod agent;`
//! - Phase 11 (Building Deep): `pub mod building;`
//! - Phase 17~20 (Wildlife/Disasters): 추가
//!
//! 현재 (T7.5.5.B): empty scaffold — actual modules는 후속 phase에서 land.

// T7.6에서 land 의무
// pub mod influence;
```

#### 4. `rust/Cargo.toml` (수정)

```toml
[workspace]
resolver = "2"
members = [
    "crates/sim-core",
    "crates/sim-engine",
    "crates/sim-systems",
]
exclude = []
```

기존 members list에 `"crates/sim-systems"` 추가만. 다른 변경 X.

## How to Implement

trivial scaffold:
1. `mkdir -p rust/crates/sim-systems/src/runtime`
2. `Cargo.toml` 위 내용 정확 적용 (워크스페이스 inheritance + path deps)
3. `src/lib.rs` 정확 적용 (pub mod runtime;)
4. `src/runtime/mod.rs` 정확 적용 (empty placeholder)
5. workspace `Cargo.toml` members 수정
6. `cargo build --workspace` 검증
7. `cargo test --workspace` 검증 (sim-systems 0 tests, regression 0건 의무)
8. `cargo clippy --workspace --all-targets -- -D warnings` 검증

★ **scope 정통**:
- Behavior 0건 (empty modules)
- Public API 노출: `pub mod runtime;` only
- Tests 0건 (T7.6에서 system tests land 의무)
- Documentation comments 의무 (`//!` crate-level + module-level)

## Verification

### Mechanical Gate
- `cargo build --workspace`: 0 errors (3 crates)
- `cargo test --workspace`: regression 0건 (sim-systems 0 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings

### Workspace metadata 검증
```bash
cargo metadata --no-deps --format-version 1 | python3 -c "
import json, sys
data = json.load(sys.stdin)
names = sorted([p['name'] for p in data['packages']])
assert 'sim-core' in names
assert 'sim-engine' in names
assert 'sim-systems' in names
print(f'OK: {names}')
"
```

### sim-systems dependency 검증
```bash
grep -A5 "\[dependencies\]" rust/crates/sim-systems/Cargo.toml
# Expected: hecs, serde, sim-core (path), sim-engine (path)
```

### Public API 검증 (over-exposure 금지)
```bash
grep -n "^pub" rust/crates/sim-systems/src/lib.rs rust/crates/sim-systems/src/runtime/mod.rs
# Expected: only "pub mod runtime;" in lib.rs, nothing in mod.rs
```

### Documentation 검증
```bash
grep -n "^//!" rust/crates/sim-systems/src/lib.rs rust/crates/sim-systems/src/runtime/mod.rs
# Expected: crate-level + module-level doc comments 명시
```

## Localization

No new localization keys (empty scaffold, no UI strings).

## Commit Message

```
T7.5.5.B: sim-systems crate scaffold — Phase 2 RuntimeSystems home

[--quick pipeline] hot/warm tier crate (behavior implementations home)

- rust/crates/sim-systems/Cargo.toml (sim-engine + sim-core deps)
- rust/crates/sim-systems/src/lib.rs (pub mod runtime;)
- rust/crates/sim-systems/src/runtime/mod.rs (empty, T7.6 prep)
- rust/Cargo.toml workspace members 갱신 (+sim-systems)

Empty scaffold:
- behavior 0건 (T7.6에서 4 RuntimeSystems land 의무)
- tests 0건 (T7.6에서 system tests land)
- regression: sim-core + sim-engine tests, all passing

Phase 0 v0.1.3 patch Section 4.3 base.
Refs: 1ab45b71 (T7.5.5.A sim-engine), 733b33a1 (T7.5.5.0 governance v3.3.5)
```
