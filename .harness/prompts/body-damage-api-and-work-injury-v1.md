# Body Damage API + Work Injury v1

## Feature Description

Extends the existing BodyHealth system (A-11) with:

1. **Damage API** on `BodyHealth` in sim-core/components/body_health.rs:
   - `InjurySpec { part_idx: u8, severity: u8, flags: PartFlags, bleed_rate: u8 }`
   - `InjuryReport { part_idx: u8, hp_after: u8, vital_destroyed: bool }`
   - `BodyHealth::apply_injury(spec: InjurySpec) -> InjuryReport`
     - part_idx 255 → `find_random_minor_part_index()` (first non-vital, non-disabled, hp>0)
     - Applies saturating_sub(severity), ORs flags, maxes bleed_rate
     - Calls recalculate_aggregates(); promotes LOD from Aggregate → Standard if needed
   - Both types re-exported from sim-core/components/mod.rs

2. **`EffectPrimitive::DamagePart`** variant in sim-core/src/effect.rs:
   - `DamagePart { part_idx: u8, severity: u8, flags_bits: u8, bleed_rate: u8 }`
   - Integrates with the A-3 effect queue pipeline

3. **`EffectApplySystem::apply_damage_part`** handler in sim-systems/src/runtime/effect_apply.rs:
   - Resolves entity, calls health.apply_injury(InjurySpec{...}), pushes causal log entry
   - Effect key: `"part_{part_idx}_hp_{hp_after}"`, magnitude = f64::from(severity)
   - Handles DamagePart arm in the run() match block

4. **Work injury triggers** in sim-systems/src/runtime/world.rs:
   - Forage completion: 1% chance (WORK_INJURY_FORAGE_CHANCE), severity 5–15 (WORK_INJURY_FORAGE_SEVERITY_MIN/MAX)
   - Craft completion: 1.5% chance (WORK_INJURY_CRAFT_CHANCE), severity 10–30 (WORK_INJURY_CRAFT_SEVERITY_MIN/MAX)
   - Both use deterministic hash (entity_id ⊕ tick LCG) — NOT shared RNG — to avoid poisoning
     the RNG sequence for unrelated stochastic decisions
   - All 7 new config constants added to sim-core/src/config.rs
   - Part index 255 (random minor part), flags_bits=0x01 (BLEEDING), bleed_rate=WORK_INJURY_BLEED_RATE

5. **7 harness tests** in `mod harness_body_damage_api` (top-level #[cfg(test)] module in sim-test):
   - D1: apply_injury_basic — severity applied, hp_after correct, vital_destroyed false
   - D2: apply_injury_random_part — part_idx 255 selects a non-vital minor part
   - D3: vital_part_destroyed_flag — vital part zeroed sets vital_destroyed=true
   - D4: effect_primitive_damage_part_via_queue — DamagePart via EffectQueue reduces HP
   - D5: work_injury_causal_log_records_kind — causal log contains forage_injury entry after 10k ticks
   - D6: hp_rate_tolerance — work injury rate within 0.5%–3% over 10k forage events
   - D7: body_health_lod_promotes_on_injury — LOD promotes from Aggregate to Standard on first injury

## Files Changed

- `rust/crates/sim-core/src/components/body_health.rs` — apply_injury, InjurySpec, InjuryReport
- `rust/crates/sim-core/src/components/mod.rs` — re-exports InjurySpec, InjuryReport
- `rust/crates/sim-core/src/effect.rs` — DamagePart variant
- `rust/crates/sim-core/src/config.rs` — 7 WORK_INJURY_* constants
- `rust/crates/sim-systems/src/runtime/effect_apply.rs` — apply_damage_part handler
- `rust/crates/sim-systems/src/runtime/world.rs` — forage + craft work injury triggers
- `rust/crates/sim-test/src/main.rs` — harness_body_damage_api module (D1–D7)
