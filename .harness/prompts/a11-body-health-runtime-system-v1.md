# A-11: BodyHealth Runtime System v1

## Feature Description

Implements HealthRuntimeSystem for the WorldSim simulation.
The BodyHealth component (85 PartState entries, HealthLod, PartFlags, PART_VITAL constants)
was already present at ~90%. This feature adds:

1. 4 config constants in sim-core/config.rs:
   - BLEED_HP_DRAIN = 1 (u8)
   - INFECTION_DAMAGE_THRESHOLD = 80 (u8)
   - INFECTION_HP_DRAIN = 1 (u8)
   - HEALTH_AGGREGATE_DEATH_THRESHOLD = 0.05 (f64)

2. HealthRuntimeSystem (Cold-tier, priority 110, tick_interval 30):
   - LOD dispatch: Aggregate → 1 threshold check only; Simplified/Standard/Full → process_parts()
   - process_parts(): bleeding drain (BLEED_HP_DRAIN/tick, self-resolve at hp=0),
     infection sev accumulation (saturating_add 1/tick, HP drain when sev >= threshold),
     vital part hp=0 → vital_dead = true
   - Two-phase run(): Phase 1 collects to_die via query_mut, Phase 2 marks age.alive=false
     + emits SimEvent::Death { cause: "body_damage" } + increments stats_total_deaths

3. Module wiring in sim-systems/src/runtime/mod.rs

4. Registration in sim-bridge runtime_system.rs (HealthRuntime = 67),
   DEFAULT_RUNTIME_SYSTEMS array size: 63 → 64

5. EXPECTED_SYSTEM_COUNT update: 66 → 67 in sim-test/src/main.rs

6. 7 harness tests (see below)

## Crate: sim-systems, sim-bridge, sim-core, sim-test

## Harness Tests (7)

1. harness_a11_health_runtime_registered — registry check, 64 entries, HealthRuntime present
2. harness_a11_vital_part_destruction_triggers_death — vital part hp→0 (Full LOD) → age.alive=false
3. harness_a11_destroyed_part_clears_bleeding — BLEEDING cleared when hp→0
4. harness_a11_lod_aggregate_skips_per_part — Aggregate LOD ignores per-part BLEEDING, hp unchanged
5. harness_a11_infection_progresses_over_time — infection_sev increases each tick (Full LOD)
6. harness_a11_bleeding_drains_hp — BLEEDING flag → hp decreases (Full LOD)
7. harness_a11_default_health_pristine_after_ticks — no flags set → hp unchanged, agent alive
