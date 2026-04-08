# A-9 Phase 5: Multi-Ruleset Composition (Base + Scenario Merge)

## Section 1: Implementation Intent

### Problem

Currently `DataRegistry` loads exactly ONE `WorldRuleset` via `load_optional_singleton()` — if two RON files exist in `world_rules/`, it errors. The `priority` field on `WorldRuleset` exists but is never used because there's nothing to merge against.

The design vision is a layered composition system:
```
base_rules.ron (priority 0)     — default world constants
  + scenario.ron (priority 10)  — "eternal_winter" overrides
  + oracle.ron (priority 100)   — player divine intervention (future)
  = merged WorldRuleset         — higher priority wins per-field
```

This is the Factorio pattern: Settings → Compile → Runtime. Multiple data layers merge by priority, and the final merged ruleset is what `apply_world_rules()` consumes.

### What this solves

After this feature:
1. `DataRegistry` loads ALL `WorldRuleset` RON files from `world_rules/` (including subdirs)
2. Rulesets are merged by `priority` (higher priority overrides lower)
3. `scenarios/eternal_winter.ron` (priority 10) automatically overrides `base_rules.ron` (priority 0) when present
4. `apply_world_rules()` works unchanged — it receives a single merged `WorldRuleset`
5. Future oracle interventions can inject priority-100 rules without changing architecture

### Merge strategy

For each field type:
- **Option<T> fields** (GlobalConstants, AgentConstants): higher-priority `Some(x)` overrides lower-priority `Some(y)`. `None` = "don't override" (transparent).
- **Vec fields** (resource_modifiers, special_zones, influence_channels): APPEND from all rulesets, sorted by priority. Same-target modifiers: last writer wins (highest priority).
- **Scalar fields** (name): highest-priority ruleset's name wins.

---

## Section 2: What to Build

### Part A: Load multiple rulesets

**File: `rust/crates/sim-data/src/registry.rs`**

Change `world_rules: Option<WorldRuleset>` to `world_rules: Vec<WorldRuleset>`:

```rust
pub struct DataRegistry {
    // ...
    /// World rulesets loaded from all RON files, sorted by priority ascending.
    pub world_rules: Vec<WorldRuleset>,
    // ...
}
```

Change loading from `load_optional_singleton` to a new function that loads from `world_rules/` AND `world_rules/scenarios/` (recursive):

```rust
fn load_all_world_rules(
    dir: &Path,
    errors: &mut Vec<DataLoadError>,
) -> Vec<WorldRuleset> {
    let mut all_rules = Vec::new();

    // Load from base dir
    match load_ron_directory::<WorldRuleset>(dir) {
        Ok(defs) => all_rules.extend(defs),
        Err(mut e) => errors.append(&mut e),
    }

    // Load from subdirectories (scenarios/, oracle/, etc.)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                match load_ron_directory::<WorldRuleset>(&entry.path()) {
                    Ok(defs) => all_rules.extend(defs),
                    Err(mut e) => errors.append(&mut e),
                }
            }
        }
    }

    // Sort by priority ascending (lowest first, highest last = wins on merge)
    all_rules.sort_by_key(|r| r.priority);
    all_rules
}
```

### Part B: Merge function

**File: `rust/crates/sim-data/src/registry.rs`** (or a new `rust/crates/sim-data/src/merge.rs`)

```rust
/// Merges multiple rulesets into one, applying priority-based overrides.
/// Rulesets must be sorted by priority ascending (lowest first).
pub fn merge_world_rules(rulesets: &[WorldRuleset]) -> Option<WorldRuleset> {
    if rulesets.is_empty() {
        return None;
    }

    let mut merged = WorldRuleset {
        name: String::new(),
        priority: 0,
        resource_modifiers: Vec::new(),
        special_zones: Vec::new(),
        special_resources: Vec::new(),
        agent_constants: None,
        influence_channels: Vec::new(),
        global_constants: None,
    };

    for ruleset in rulesets {
        // Name: last (highest priority) wins
        merged.name = ruleset.name.clone();
        merged.priority = ruleset.priority;

        // Vec fields: append (duplicates resolved by last-writer-wins on same target)
        for modifier in &ruleset.resource_modifiers {
            // Remove existing modifier with same target, then add
            merged.resource_modifiers.retain(|m| m.target != modifier.target);
            merged.resource_modifiers.push(modifier.clone());
        }

        // Special zones: append (no dedup — different rulesets can add different zones)
        merged.special_zones.extend(ruleset.special_zones.clone());

        // Special resources: dedup by name
        for resource in &ruleset.special_resources {
            merged.special_resources.retain(|r| r.name != resource.name);
            merged.special_resources.push(resource.clone());
        }

        // Influence channels: last writer wins per channel
        for channel_rule in &ruleset.influence_channels {
            merged.influence_channels.retain(|c| c.channel != channel_rule.channel);
            merged.influence_channels.push(channel_rule.clone());
        }

        // GlobalConstants: merge field-by-field (Some overwrites Some, None is transparent)
        merged.global_constants = merge_global_constants(
            merged.global_constants.as_ref(),
            ruleset.global_constants.as_ref(),
        );

        // AgentConstants: same field-by-field merge
        merged.agent_constants = merge_agent_constants(
            merged.agent_constants.as_ref(),
            ruleset.agent_constants.as_ref(),
        );
    }

    Some(merged)
}

fn merge_global_constants(
    base: Option<&GlobalConstants>,
    overlay: Option<&GlobalConstants>,
) -> Option<GlobalConstants> {
    let overlay = match overlay {
        Some(o) => o,
        None => return base.cloned(),
    };
    let base = base.cloned().unwrap_or(GlobalConstants {
        season_mode: None,
        hunger_decay_mul: None,
        warmth_decay_mul: None,
        food_regen_mul: None,
        wood_regen_mul: None,
        farming_enabled: None,
        temperature_bias: None,
        disaster_frequency_mul: None,
    });
    Some(GlobalConstants {
        season_mode: overlay.season_mode.clone().or(base.season_mode),
        hunger_decay_mul: overlay.hunger_decay_mul.or(base.hunger_decay_mul),
        warmth_decay_mul: overlay.warmth_decay_mul.or(base.warmth_decay_mul),
        food_regen_mul: overlay.food_regen_mul.or(base.food_regen_mul),
        wood_regen_mul: overlay.wood_regen_mul.or(base.wood_regen_mul),
        farming_enabled: overlay.farming_enabled.or(base.farming_enabled),
        temperature_bias: overlay.temperature_bias.or(base.temperature_bias),
        disaster_frequency_mul: overlay.disaster_frequency_mul.or(base.disaster_frequency_mul),
    })
}

fn merge_agent_constants(
    base: Option<&AgentConstants>,
    overlay: Option<&AgentConstants>,
) -> Option<AgentConstants> {
    let overlay = match overlay {
        Some(o) => o,
        None => return base.cloned(),
    };
    let base = base.cloned().unwrap_or(AgentConstants {
        mortality_mul: None,
        skill_xp_mul: None,
        body_potential_mul: None,
        fertility_mul: None,
        lifespan_mul: None,
        move_speed_mul: None,
    });
    Some(AgentConstants {
        mortality_mul: overlay.mortality_mul.or(base.mortality_mul),
        skill_xp_mul: overlay.skill_xp_mul.or(base.skill_xp_mul),
        body_potential_mul: overlay.body_potential_mul.or(base.body_potential_mul),
        fertility_mul: overlay.fertility_mul.or(base.fertility_mul),
        lifespan_mul: overlay.lifespan_mul.or(base.lifespan_mul),
        move_speed_mul: overlay.move_speed_mul.or(base.move_speed_mul),
    })
}
```

### Part C: Update world_rules_ref() to return merged

**File: `rust/crates/sim-data/src/registry.rs`**

Change `world_rules_ref()` to merge on access:

```rust
/// Returns the merged world ruleset (all loaded rulesets merged by priority).
pub fn merged_world_rules(&self) -> Option<WorldRuleset> {
    merge_world_rules(&self.world_rules)
}
```

### Part D: Update apply_world_rules() caller

**File: `rust/crates/sim-engine/src/engine.rs`**

Change from:
```rust
let rules = self.data_registry.as_ref()
    .and_then(|registry| registry.world_rules_ref())
    .cloned();
```

To:
```rust
let rules = self.data_registry.as_ref()
    .and_then(|registry| registry.merged_world_rules());
```

Log which rulesets were merged:
```rust
if let Some(registry) = &self.data_registry {
    let count = registry.world_rules.len();
    let names: Vec<&str> = registry.world_rules.iter().map(|r| r.name.as_str()).collect();
    info!("[WorldRules] merging {} ruleset(s): {:?}", count, names);
}
```

### Part E: Fix all compilation breaks

All code that uses `registry.world_rules` (singular Option) must be updated to handle `Vec<WorldRuleset>`. Search:

```bash
grep -rn "world_rules\b" rust/crates/ | grep -v test | grep -v target | grep -v "\.ron"
```

Common patterns:
- `registry.world_rules.as_ref()` → `registry.merged_world_rules()`
- `registry.world_rules.is_some()` → `!registry.world_rules.is_empty()`

---

## Section 3: How to Implement

### Key design decision: when to merge

Option A: Merge on every access (`merged_world_rules()` recomputes each time)
Option B: Merge once at load time, cache result

**Choose Option B** — merge once in `DataRegistry::load_from_directory()` and store as a cached `Option<WorldRuleset>`. Merging is cheap but there's no reason to repeat it.

```rust
pub struct DataRegistry {
    /// Raw loaded rulesets (for diagnostics/debugging).
    pub world_rules_raw: Vec<WorldRuleset>,
    /// Merged ruleset (cached, computed once at load time).
    pub world_rules: Option<WorldRuleset>,
}
```

This keeps the existing `world_rules: Option<WorldRuleset>` field type — `apply_world_rules()` and all downstream code works unchanged. The `_raw` field is new and only for debug.

### Backward compatibility

- `base_rules.ron` alone → loads 1 ruleset, "merges" to itself → identical behavior
- `base_rules.ron` + `scenarios/eternal_winter.ron` → loads 2, merges with eternal_winter overriding base → new behavior
- No scenario files → loads 1, works as before

### Scenario activation

Currently `scenarios/eternal_winter.ron` is always loaded if present. For now this is fine — to "activate" a scenario, the player places the RON file in `world_rules/scenarios/`. To deactivate, they remove it. Future improvement: an `enabled: bool` field or a CLI/UI selector.

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | load_all_world_rules + merge function | sim-data/src/registry.rs | Rust | 🟢 DISPATCH | — |
| T2 | Update engine to use merged rules | sim-engine/src/engine.rs | Rust | 🟢 DISPATCH | T1 |
| T3 | Fix all world_rules references | multiple .rs files | Rust | 🟢 DISPATCH | T1 |
| T4 | Harness test | sim-test/src/main.rs | Rust | 🟢 DISPATCH | T1, T2 |

**Dispatch ratio**: 4/4 = 100% ✓

---

## Section 5: Localization Checklist

No new localization keys.

---

## Section 6: Verification & Harness

### Gate command

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

### Harness test

```rust
#[test]
fn harness_world_rules_merge_base_only() {
    // When only base_rules.ron exists (no scenario), behavior is identical to before
    let engine = make_stage1_engine(42, 20);
    let resources = engine.resources();

    // Type A: defaults must be unchanged
    assert!((resources.hunger_decay_rate - config::HUNGER_DECAY_RATE).abs() < 1e-9);
    assert!(resources.farming_enabled);
    assert_eq!(resources.season_mode, "default");
}
```

### Unit tests for merge logic

```rust
#[test]
fn merge_two_rulesets_higher_priority_wins() {
    let base = WorldRuleset {
        name: "base".into(),
        priority: 0,
        global_constants: Some(GlobalConstants {
            hunger_decay_mul: Some(1.0),
            farming_enabled: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };
    let scenario = WorldRuleset {
        name: "winter".into(),
        priority: 10,
        global_constants: Some(GlobalConstants {
            hunger_decay_mul: Some(1.5),
            farming_enabled: Some(false),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merge_world_rules(&[base, scenario]).unwrap();
    assert_eq!(merged.name, "winter"); // highest priority name
    let gc = merged.global_constants.unwrap();
    assert_eq!(gc.hunger_decay_mul, Some(1.5)); // scenario overrides base
    assert_eq!(gc.farming_enabled, Some(false)); // scenario overrides base
}

#[test]
fn merge_none_is_transparent() {
    let base = WorldRuleset {
        name: "base".into(),
        priority: 0,
        global_constants: Some(GlobalConstants {
            hunger_decay_mul: Some(1.0),
            food_regen_mul: Some(0.8),
            ..Default::default()
        }),
        ..Default::default()
    };
    let scenario = WorldRuleset {
        name: "winter".into(),
        priority: 10,
        global_constants: Some(GlobalConstants {
            hunger_decay_mul: Some(1.5),
            // food_regen_mul: None → transparent, base value preserved
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merge_world_rules(&[base, scenario]).unwrap();
    let gc = merged.global_constants.unwrap();
    assert_eq!(gc.hunger_decay_mul, Some(1.5)); // overridden
    assert_eq!(gc.food_regen_mul, Some(0.8)); // preserved from base
}
```

---

## Section 7: 인게임 확인사항

1. **base_rules만 있을 때**: 기존과 동일한 동작 (회귀 없음).
2. **scenarios/eternal_winter.ron 활성화 시**: 콘솔에 `[WorldRules] merging 2 ruleset(s): ["BaseRules", "EternalWinter"]` 출력.
3. **eternal_winter 값 적용 확인**: hunger_decay가 config 기본값 × 1.3인지.
4. **기존 harness 전부 통과**.

### 구현 후 정리 보고

```
## 구현 완료 보고

### 구현 의도
단일 WorldRuleset → 다중 레이어 합성 (base + scenario). priority 기반 merge.

### 구현 내용
load_all_world_rules() — 재귀 디렉토리 로딩. merge_world_rules() — priority 기반 필드별 합성.
world_rules_raw (디버그용) + world_rules (머지 결과 캐시).

### 구현 방법
Option 필드: overlay.or(base). Vec 필드: same-target dedup + append. Scalar: last writer wins.

### 변경된 파일 목록
(커밋별 나열)

### 확인된 제한사항
시나리오 활성화는 파일 존재 여부로만 제어 — UI 선택기는 별도 구현 필요.
merge는 로드 시 1회 — 런타임 ruleset 교체는 지원 안 함 (Oracle Phase에서 별도 구현).

### Harness 결과
harness_world_rules_merge_base_only: (결과)
기존 65+개: (전부 통과 확인)
```

---

## Execution

```bash
bash tools/harness/harness_pipeline.sh a9-ruleset-merge .harness/prompts/a9-ruleset-merge.md --full
```
