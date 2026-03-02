# Rust Migration Commit Log

## Progress Baseline
- Initial estimated completion: 18%
- Initial remaining: 82%

## Commits

### 0001 - Phase A/C/H runtime bridge scaffold
- Commit: `[rust-r0-101] Add Rust runtime entrypoint and Bus v2 bridge scaffold`
- Completion after commit: 34%
- Remaining after commit: 66%
- Details: [0001-phase-a-c-h-runtime-bridge.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0001-phase-a-c-h-runtime-bridge.md)

### 0002 - Save v2 (.ws2) runtime pipeline
- Commit: `[rust-r0-102] Add ws2 save/load pipeline for Rust runtime`
- Completion after commit: 42%
- Remaining after commit: 58%
- Details: [0002-phase-e-ws2-runtime-save.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0002-phase-e-ws2-runtime-save.md)

### 0003 - Fluent source pipeline + Locale loader
- Commit: `[rust-r0-103] Add Fluent source pipeline and Locale fluent loader`
- Completion after commit: 50%
- Remaining after commit: 50%
- Details: [0003-phase-g-fluent-loader.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0003-phase-g-fluent-loader.md)

### 0004 - Shadow diff reporter + Bus v2 command pipeline
- Commit: `[rust-r0-104] Add shadow reporter and Bus v2 runtime commands`
- Completion after commit: 57%
- Remaining after commit: 43%
- Details: [0004-phase-h-shadow-reporter.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0004-phase-h-shadow-reporter.md)

### 0005 - Domain-based compute backend routing
- Commit: `[rust-r0-105] Add domain-based compute backend modes`
- Completion after commit: 62%
- Remaining after commit: 38%
- Details: [0005-phase-f-domain-compute-modes.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0005-phase-f-domain-compute-modes.md)

### 0006 - Rust runtime system registry bridge
- Commit: `[rust-r0-106] Bridge GDScript system registry into Rust runtime`
- Completion after commit: 68%
- Remaining after commit: 32%
- Details: [0006-phase-b-system-registry-bridge.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0006-phase-b-system-registry-bridge.md)

### 0007 - Pathfinding bridge uses domain compute mode
- Commit: `[rust-r0-107] Route pathfinding bridge via domain compute mode`
- Completion after commit: 72%
- Remaining after commit: 28%
- Details: [0007-phase-f-pathfinding-domain-route.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0007-phase-f-pathfinding-domain-route.md)

### 0008 - Runtime compute-domain command synchronization
- Commit: `[rust-r0-108] Sync compute domain commands into Rust runtime`
- Completion after commit: 76%
- Remaining after commit: 24%
- Details: [0008-phase-f-runtime-compute-sync.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0008-phase-f-runtime-compute-sync.md)

### 0009 - Runtime state events on Bus v2
- Commit: `[rust-r0-109] Route runtime state changes through Bus v2 events`
- Completion after commit: 80%
- Remaining after commit: 20%
- Details: [0009-phase-c-runtime-state-events.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0009-phase-c-runtime-state-events.md)

### 0010 - Rust Fluent runtime bridge
- Commit: `[rust-r0-110] Add Rust Fluent runtime bridge for Locale formatting`
- Completion after commit: 84%
- Remaining after commit: 16%
- Details: [0010-phase-g-rust-fluent-runtime.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0010-phase-g-rust-fluent-runtime.md)

### 0011 - Runtime registry validation + upsert
- Commit: `[rust-r0-111] Validate and upsert runtime system registry`
- Completion after commit: 87%
- Remaining after commit: 13%
- Details: [0011-phase-b-registry-validation.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0011-phase-b-registry-validation.md)

### 0012 - ws2 save/load single-path cutover
- Commit: `[rust-r0-112] Cut over SaveManager to ws2-only runtime path`
- Completion after commit: 90%
- Remaining after commit: 10%
- Details: [0012-phase-e-ws2-cutover.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0012-phase-e-ws2-cutover.md)

### 0013 - Shadow cutover gating metrics
- Commit: `[rust-r0-113] Add shadow cutover approval metrics`
- Completion after commit: 93%
- Remaining after commit: 7%
- Details: [0013-phase-h-shadow-cutover-gating.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0013-phase-h-shadow-cutover-gating.md)

### 0014 - Shadow auto-cutover hook
- Commit: `[rust-r0-114] Add shadow-approved auto cutover hook`
- Completion after commit: 95%
- Remaining after commit: 5%
- Details: [0014-phase-h-shadow-auto-cutover.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0014-phase-h-shadow-auto-cutover.md)

### 0015 - Legacy migration call removal
- Commit: `[rust-r0-115] Remove legacy save migration call path`
- Completion after commit: 96%
- Remaining after commit: 4%
- Details: [0015-phase-e-legacy-call-removal.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0015-phase-e-legacy-call-removal.md)

### 0016 - ws2 runtime-ready guard
- Commit: `[rust-r0-116] Add runtime-initialized guard for ws2 save path`
- Completion after commit: 98%
- Remaining after commit: 2%
- Details: [0016-phase-e-runtime-ready-guard.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0016-phase-e-runtime-ready-guard.md)

### 0017 - Shadow cutover check script
- Commit: `[rust-r0-117] Add shadow cutover readiness check tool`
- Completion after commit: 99%
- Remaining after commit: 1%
- Details: [0017-phase-h-cutover-check-script.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0017-phase-h-cutover-check-script.md)

### 0018 - Headless shadow verification + extension load hardening
- Commit: `[rust-r0-118] Validate headless shadow cutover with runtime registration`
- Completion after commit: 100%
- Remaining after commit: 0%
- Details: [0018-phase-h-headless-shadow-verification.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0018-phase-h-headless-shadow-verification.md)

### 0019 - Headless GDScript parse stabilization
- Commit: `[rust-r0-119] Stabilize GDScript parse path for headless validation`
- Completion after commit: 100%
- Remaining after commit: 0%
- Details: [0019-headless-gdscript-parse-stabilization.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0019-headless-gdscript-parse-stabilization.md)

### 0020 - Runtime registry order determinism
- Commit: `[rust-r0-120] Make runtime registry ordering deterministic`
- Completion after commit: 100%
- Remaining after commit: 0%
- Details: [0020-registry-order-determinism.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0020-registry-order-determinism.md)

### 0021 - Full Rust tracking baseline (autopilot)
- Commit: `[rust-r0-121] Add full Rust migration tracking baseline`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Details: [0021-full-rust-tracking-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0021-full-rust-tracking-baseline.md)
- Data:
  - `reports/rust-migration/data/gd-inventory.csv`
  - `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `reports/rust-migration/data/tracking-metadata.json`

### 0022 - First Rust runtime system port (stats_recorder)
- Commit: `[rust-r0-122] Add first production Rust runtime system integration`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 2.17%
- Logic implementation remaining after commit: 97.83%
- Details: [0022-first-rust-runtime-system-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0022-first-rust-runtime-system-port.md)

### 0023 - Rust runtime system port (resource_regen_system)
- Commit: `[rust-r0-123] Port resource regen runtime system to Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 4.35%
- Logic implementation remaining after commit: 95.65%
- Details: [0023-resource-regen-rust-runtime-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0023-resource-regen-rust-runtime-port.md)

### 0024 - Rust runtime baseline port (stat_sync_system)
- Commit: `[rust-r0-124] Add stat sync runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 6.52%
- Logic implementation remaining after commit: 93.48%
- Details: [0024-stat-sync-rust-runtime-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0024-stat-sync-rust-runtime-baseline.md)

### 0025 - Rust primary hybrid execution gate
- Commit: `[rust-r0-125] Add rust-primary hybrid execution fallback gate`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 6.52%
- Logic port remaining after commit: 93.48%
- Logic implementation completion after commit: 6.52%
- Logic implementation remaining after commit: 93.48%
- Details: [0025-rust-primary-hybrid-execution-gate.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0025-rust-primary-hybrid-execution-gate.md)

### 0026 - Owner-ready gating safety fix
- Commit: `[rust-r0-126] Enforce owner-ready allowlist for hybrid fallback`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 6.52%
- Logic implementation remaining after commit: 93.48%
- Details: [0026-owner-ready-gating-safety-fix.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0026-owner-ready-gating-safety-fix.md)

### 0027 - Rust runtime system port (upper_needs_system)
- Commit: `[rust-r0-127] Port upper needs runtime system to Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 8.70%
- Logic implementation remaining after commit: 91.30%
- Details: [0027-upper-needs-rust-runtime-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0027-upper-needs-rust-runtime-port.md)

### 0028 - Rust runtime system port (needs_system)
- Commit: `[rust-r0-128] Port needs runtime system to Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 10.87%
- Logic implementation remaining after commit: 89.13%
- Details: [0028-needs-runtime-rust-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0028-needs-runtime-rust-port.md)

### 0029 - Rust runtime baseline port (stress_system)
- Commit: `[rust-r0-129] Add stress runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 13.04%
- Logic implementation remaining after commit: 86.96%
- Details: [0029-stress-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0029-stress-runtime-rust-baseline.md)

### 0030 - Rust runtime baseline port (emotion_system)
- Commit: `[rust-r0-130] Add emotion runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 15.22%
- Logic implementation remaining after commit: 84.78%
- Details: [0030-emotion-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0030-emotion-runtime-rust-baseline.md)

### 0031 - Rust runtime baseline port (stat_threshold_system)
- Commit: `[rust-r0-131] Add stat-threshold runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 17.39%
- Logic implementation remaining after commit: 82.61%
- Details: [0031-stat-threshold-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0031-stat-threshold-runtime-rust-baseline.md)

### 0032 - Rust runtime baseline port (job_assignment_system)
- Commit: `[rust-r0-132] Add job-assignment runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 19.57%
- Logic implementation remaining after commit: 80.43%
- Details: [0032-job-assignment-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0032-job-assignment-runtime-rust-baseline.md)

### 0033 - Rust runtime baseline port (child_stress_processor)
- Commit: `[rust-r0-133] Add child-stress runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 21.74%
- Logic implementation remaining after commit: 78.26%
- Details: [0033-child-stress-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0033-child-stress-runtime-rust-baseline.md)

### 0034 - Rust runtime baseline port (mental_break_system)
- Commit: `[rust-r0-134] Add mental-break runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 23.91%
- Logic implementation remaining after commit: 76.09%
- Details: [0034-mental-break-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0034-mental-break-runtime-rust-baseline.md)

### 0035 - Rust runtime baseline port (occupation_system)
- Commit: `[rust-r0-135] Add occupation runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 26.09%
- Logic implementation remaining after commit: 73.91%
- Details: [0035-occupation-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0035-occupation-runtime-rust-baseline.md)

### 0036 - Rust runtime baseline port (trauma_scar_system)
- Commit: `[rust-r0-136] Add trauma-scar runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 28.26%
- Logic implementation remaining after commit: 71.74%
- Details: [0036-trauma-scar-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0036-trauma-scar-runtime-rust-baseline.md)

### 0037 - Rust runtime baseline port (title_system)
- Commit: `[rust-r0-137] Add title runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 30.43%
- Logic implementation remaining after commit: 69.57%
- Details: [0037-title-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0037-title-runtime-rust-baseline.md)

### 0038 - Rust runtime baseline port (value_system)
- Commit: `[rust-r0-138] Add value runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 32.61%
- Logic implementation remaining after commit: 67.39%
- Details: [0038-value-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0038-value-runtime-rust-baseline.md)

### 0039 - Rust runtime baseline port (network_system)
- Commit: `[rust-r0-139] Add network runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 34.78%
- Logic implementation remaining after commit: 65.22%
- Details: [0039-network-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0039-network-runtime-rust-baseline.md)

### 0040 - Rust runtime baseline port (social_event_system)
- Commit: `[rust-r0-140] Add social-event runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 36.96%
- Logic implementation remaining after commit: 63.04%
- Details: [0040-social-event-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0040-social-event-runtime-rust-baseline.md)

### 0041 - Rust runtime baseline port (building_effect_system)
- Commit: `[rust-r0-141] Add building-effect runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 39.13%
- Logic implementation remaining after commit: 60.87%
- Details: [0041-building-effect-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0041-building-effect-runtime-rust-baseline.md)

### 0042 - Rust runtime baseline port (family_system)
- Commit: `[rust-r0-142] Add family runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 41.30%
- Logic implementation remaining after commit: 58.70%
- Details: [0042-family-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0042-family-runtime-rust-baseline.md)

### 0043 - Rust runtime baseline port (leader_system)
- Commit: `[rust-r0-143] Add leader runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 43.48%
- Logic implementation remaining after commit: 56.52%
- Details: [0043-leader-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0043-leader-runtime-rust-baseline.md)

### 0044 - Rust runtime baseline port (population_system)
- Commit: `[rust-r0-144] Add population runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 45.65%
- Logic implementation remaining after commit: 54.35%
- Details: [0044-population-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0044-population-runtime-rust-baseline.md)

### 0045 - Rust runtime baseline port (migration_system)
- Commit: `[rust-r0-145] Add migration runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 47.83%
- Logic implementation remaining after commit: 52.17%
- Details: [0045-migration-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0045-migration-runtime-rust-baseline.md)

### 0046 - Rust runtime baseline port (age_system)
- Commit: `[rust-r0-146] Add age runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 50.00%
- Logic implementation remaining after commit: 50.00%
- Details: [0046-age-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0046-age-runtime-rust-baseline.md)

### 0047 - Rust runtime baseline port (trait_violation_system)
- Commit: `[rust-r0-147] Add trait-violation runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 52.17%
- Logic implementation remaining after commit: 47.83%
- Details: [0047-trait-violation-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0047-trait-violation-runtime-rust-baseline.md)

### 0048 - Rust runtime baseline port (mortality_system)
- Commit: `[rust-r0-148] Add mortality runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 54.35%
- Logic implementation remaining after commit: 45.65%
- Details: [0048-mortality-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0048-mortality-runtime-rust-baseline.md)

### 0049 - Rust runtime baseline port (reputation_system)
- Commit: `[rust-r0-149] Add reputation runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 56.52%
- Logic implementation remaining after commit: 43.48%
- Details: [0049-reputation-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0049-reputation-runtime-rust-baseline.md)

### 0050 - Rust runtime baseline port (contagion_system)
- Commit: `[rust-r0-150] Add contagion runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 58.70%
- Logic implementation remaining after commit: 41.30%
- Details: [0050-contagion-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0050-contagion-runtime-rust-baseline.md)

### 0051 - Rust runtime baseline port (job_satisfaction_system)
- Commit: `[rust-r0-151] Add job-satisfaction runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 60.87%
- Logic implementation remaining after commit: 39.13%
- Details: [0051-job-satisfaction-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0051-job-satisfaction-runtime-rust-baseline.md)

### 0052 - Rust runtime baseline port (morale_system)
- Commit: `[rust-r0-152] Add morale runtime baseline system in Rust`
- Infra completion after commit: 100%
- Infra remaining after commit: 0%
- Logic port completion after commit: 0.0%
- Logic port remaining after commit: 100.0%
- Logic implementation completion after commit: 63.04%
- Logic implementation remaining after commit: 36.96%
- Details: [0052-morale-runtime-rust-baseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0052-morale-runtime-rust-baseline.md)

### 0053 - Runtime no-op baseline removal + strict state-write rebaseline
- Commit: `[rust-r0-153] Remove no-op runtime baselines and rebaseline strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 6.52% (3/46)
- Actual state-write coverage (strict): 6.52% (3/46)
- Owner transfer coverage (`exec_owner=rust`): 0.00% (0/46)
- Remaining (strict state-write basis): 93.48%
- Details: [0053-runtime-noop-removal-and-strict-rebaseline.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0053-runtime-noop-removal-and-strict-rebaseline.md)

### 0054 - sim-bridge module split + GPU placeholder hardening
- Commit: `[rust-r0-154] Split sim-bridge modules and harden non-path GPU placeholders`
- Reported implementation coverage (legacy `rust_runtime_impl`): 6.52% (3/46)
- Actual state-write coverage (strict): 6.52% (3/46)
- Owner transfer coverage (`exec_owner=rust`): 0.00% (0/46)
- Remaining (strict state-write basis): 93.48%
- Details: [0054-sim-bridge-module-split-and-gpu-placeholder-hardening.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0054-sim-bridge-module-split-and-gpu-placeholder-hardening.md)

### 0055 - stress runtime active-write port (Phase 5 start)
- Commit: `[rust-r0-155] Port stress runtime to active-write and re-enable strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 8.70% (4/46)
- Actual state-write coverage (strict): 8.70% (4/46)
- Owner transfer coverage (`exec_owner=rust`): 0.00% (0/46)
- Remaining (strict state-write basis): 91.30%
- Details: [0055-stress-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0055-stress-runtime-active-write-port.md)

### 0056 - emotion runtime active-write port
- Commit: `[rust-r0-156] Port emotion runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 10.87% (5/46)
- Actual state-write coverage (strict): 10.87% (5/46)
- Owner transfer coverage (`exec_owner=rust`): 0.00% (0/46)
- Remaining (strict state-write basis): 89.13%
- Details: [0056-emotion-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0056-emotion-runtime-active-write-port.md)

### 0057 - reputation runtime active-write port
- Commit: `[rust-r0-157] Port reputation runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 13.04% (6/46)
- Actual state-write coverage (strict): 13.04% (6/46)
- Owner transfer coverage (`exec_owner=rust`): 0.00% (0/46)
- Remaining (strict state-write basis): 86.96%
- Details: [0057-reputation-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0057-reputation-runtime-active-write-port.md)

### 0058 - social-event runtime active-write port
- Commit: `[rust-r0-158] Port social-event runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 15.22% (7/46)
- Actual state-write coverage (strict): 15.22% (7/46)
- Owner transfer coverage (`exec_owner=rust`): 0.00% (0/46)
- Remaining (strict state-write basis): 84.78%
- Details: [0058-social-event-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0058-social-event-runtime-active-write-port.md)

### 0059 - morale runtime active-write port
- Commit: `[rust-r0-159] Port morale runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 17.39% (8/46)
- Actual state-write coverage (strict): 17.39% (8/46)
- Owner transfer coverage (`exec_owner=rust`): 0.00% (0/46)
- Remaining (strict state-write basis): 82.61%
- Details: [0059-morale-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0059-morale-runtime-active-write-port.md)

### 0060 - owner transfer allowlist phase 1
- Commit: `[rust-r0-160] Enable owner-ready allowlist for initial Rust runtime systems`
- Reported implementation coverage (legacy `rust_runtime_impl`): 17.39% (8/46)
- Actual state-write coverage (strict): 17.39% (8/46)
- Owner transfer coverage (`exec_owner=rust`): 6.52% (3/46)
- Remaining (strict state-write basis): 82.61%
- Details: [0060-owner-transfer-allowlist-phase1.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0060-owner-transfer-allowlist-phase1.md)

### 0061 - owner transfer allowlist phase 2
- Commit: `[rust-r0-161] Expand owner-ready allowlist with stress and emotion systems`
- Reported implementation coverage (legacy `rust_runtime_impl`): 17.39% (8/46)
- Actual state-write coverage (strict): 17.39% (8/46)
- Owner transfer coverage (`exec_owner=rust`): 10.87% (5/46)
- Remaining (strict state-write basis): 82.61%
- Details: [0061-owner-transfer-allowlist-phase2.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0061-owner-transfer-allowlist-phase2.md)

### 0062 - owner transfer allowlist phase 3
- Commit: `[rust-r0-162] Expand owner-ready allowlist with reputation social-event and morale`
- Reported implementation coverage (legacy `rust_runtime_impl`): 17.39% (8/46)
- Actual state-write coverage (strict): 17.39% (8/46)
- Owner transfer coverage (`exec_owner=rust`): 17.39% (8/46)
- Remaining (strict state-write basis): 82.61%
- Details: [0062-owner-transfer-allowlist-phase3.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0062-owner-transfer-allowlist-phase3.md)

### 0063 - value runtime active-write port
- Commit: `[rust-r0-163] Port value runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 19.57% (9/46)
- Actual state-write coverage (strict): 19.57% (9/46)
- Owner transfer coverage (`exec_owner=rust`): 17.39% (8/46)
- Remaining (strict state-write basis): 80.43%
- Details: [0063-value-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0063-value-runtime-active-write-port.md)

### 0064 - job-satisfaction runtime active-write port
- Commit: `[rust-r0-164] Port job-satisfaction runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 21.74% (10/46)
- Actual state-write coverage (strict): 21.74% (10/46)
- Owner transfer coverage (`exec_owner=rust`): 17.39% (8/46)
- Remaining (strict state-write basis): 78.26%
- Details: [0064-job-satisfaction-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0064-job-satisfaction-runtime-active-write-port.md)

### 0065 - network runtime active-write port
- Commit: `[rust-r0-165] Port network runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 23.91% (11/46)
- Actual state-write coverage (strict): 23.91% (11/46)
- Owner transfer coverage (`exec_owner=rust`): 17.39% (8/46)
- Remaining (strict state-write basis): 76.09%
- Details: [0065-network-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0065-network-runtime-active-write-port.md)

### 0066 - occupation runtime active-write port
- Commit: `[rust-r0-166] Port occupation runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 26.09% (12/46)
- Actual state-write coverage (strict): 26.09% (12/46)
- Owner transfer coverage (`exec_owner=rust`): 17.39% (8/46)
- Remaining (strict state-write basis): 73.91%
- Details: [0066-occupation-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0066-occupation-runtime-active-write-port.md)

### 0067 - owner transfer allowlist phase 4
- Commit: `[rust-r0-167] Expand owner-ready allowlist with value and job-satisfaction systems`
- Reported implementation coverage (legacy `rust_runtime_impl`): 26.09% (12/46)
- Actual state-write coverage (strict): 26.09% (12/46)
- Owner transfer coverage (`exec_owner=rust`): 21.74% (10/46)
- Remaining (strict state-write basis): 73.91%
- Details: [0067-owner-transfer-allowlist-phase4.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0067-owner-transfer-allowlist-phase4.md)

### 0068 - contagion runtime active-write port
- Commit: `[rust-r0-168] Port contagion runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 28.26% (13/46)
- Actual state-write coverage (strict): 28.26% (13/46)
- Owner transfer coverage (`exec_owner=rust`): 21.74% (10/46)
- Remaining (strict state-write basis): 71.74%
- Details: [0068-contagion-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0068-contagion-runtime-active-write-port.md)

### 0069 - owner transfer allowlist phase 5
- Commit: `[rust-r0-169] Expand owner-ready allowlist with network and occupation systems`
- Reported implementation coverage (legacy `rust_runtime_impl`): 28.26% (13/46)
- Actual state-write coverage (strict): 28.26% (13/46)
- Owner transfer coverage (`exec_owner=rust`): 26.09% (12/46)
- Remaining (strict state-write basis): 71.74%
- Details: [0069-owner-transfer-allowlist-phase5.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0069-owner-transfer-allowlist-phase5.md)

### 0070 - age runtime active-write port
- Commit: `[rust-r0-170] Port age runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 30.43% (14/46)
- Actual state-write coverage (strict): 30.43% (14/46)
- Owner transfer coverage (`exec_owner=rust`): 26.09% (12/46)
- Remaining (strict state-write basis): 69.57%
- Details: [0070-age-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0070-age-runtime-active-write-port.md)

### 0071 - owner transfer allowlist phase 6
- Commit: `[rust-r0-171] Expand owner-ready allowlist with age system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 30.43% (14/46)
- Actual state-write coverage (strict): 30.43% (14/46)
- Owner transfer coverage (`exec_owner=rust`): 28.26% (13/46)
- Remaining (strict state-write basis): 69.57%
- Details: [0071-owner-transfer-allowlist-phase6.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0071-owner-transfer-allowlist-phase6.md)

### 0072 - owner transfer allowlist phase 7
- Commit: `[rust-r0-172] Expand owner-ready allowlist with contagion system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 30.43% (14/46)
- Actual state-write coverage (strict): 30.43% (14/46)
- Owner transfer coverage (`exec_owner=rust`): 30.43% (14/46)
- Remaining (strict state-write basis): 69.57%
- Details: [0072-owner-transfer-allowlist-phase7.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0072-owner-transfer-allowlist-phase7.md)

### 0073 - job-assignment runtime active-write port
- Commit: `[rust-r0-173] Port job-assignment runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 32.61% (15/46)
- Actual state-write coverage (strict): 32.61% (15/46)
- Owner transfer coverage (`exec_owner=rust`): 30.43% (14/46)
- Remaining (strict state-write basis): 67.39%
- Details: [0073-job-assignment-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0073-job-assignment-runtime-active-write-port.md)

### 0074 - owner transfer allowlist phase 8
- Commit: `[rust-r0-174] Expand owner-ready allowlist with job-assignment system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 32.61% (15/46)
- Actual state-write coverage (strict): 32.61% (15/46)
- Owner transfer coverage (`exec_owner=rust`): 32.61% (15/46)
- Remaining (strict state-write basis): 67.39%
- Details: [0074-owner-transfer-allowlist-phase8.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0074-owner-transfer-allowlist-phase8.md)

### 0075 - mortality runtime active-write port
- Commit: `[rust-r0-175] Port mortality runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 34.78% (16/46)
- Actual state-write coverage (strict): 34.78% (16/46)
- Owner transfer coverage (`exec_owner=rust`): 32.61% (15/46)
- Remaining (strict state-write basis): 65.22%
- Details: [0075-mortality-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0075-mortality-runtime-active-write-port.md)

### 0076 - owner transfer allowlist phase 9
- Commit: `[rust-r0-176] Expand owner-ready allowlist with mortality system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 34.78% (16/46)
- Actual state-write coverage (strict): 34.78% (16/46)
- Owner transfer coverage (`exec_owner=rust`): 34.78% (16/46)
- Remaining (strict state-write basis): 65.22%
- Details: [0076-owner-transfer-allowlist-phase9.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0076-owner-transfer-allowlist-phase9.md)

### 0077 - mental-break runtime active-write port
- Commit: `[rust-r0-177] Port mental-break runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 36.96% (17/46)
- Actual state-write coverage (strict): 36.96% (17/46)
- Owner transfer coverage (`exec_owner=rust`): 34.78% (16/46)
- Remaining (strict state-write basis): 63.04%
- Details: [0077-mental-break-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0077-mental-break-runtime-active-write-port.md)

### 0078 - owner transfer allowlist phase 10
- Commit: `[rust-r0-178] Expand owner-ready allowlist with mental-break system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 36.96% (17/46)
- Actual state-write coverage (strict): 36.96% (17/46)
- Owner transfer coverage (`exec_owner=rust`): 36.96% (17/46)
- Remaining (strict state-write basis): 63.04%
- Details: [0078-owner-transfer-allowlist-phase10.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0078-owner-transfer-allowlist-phase10.md)

### 0079 - trait-violation runtime active-write port
- Commit: `[rust-r0-179] Port trait-violation runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 39.13% (18/46)
- Actual state-write coverage (strict): 39.13% (18/46)
- Owner transfer coverage (`exec_owner=rust`): 36.96% (17/46)
- Remaining (strict state-write basis): 60.87%
- Details: [0079-trait-violation-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0079-trait-violation-runtime-active-write-port.md)

### 0080 - owner transfer allowlist phase 11
- Commit: `[rust-r0-180] Expand owner-ready allowlist with trait-violation system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 39.13% (18/46)
- Actual state-write coverage (strict): 39.13% (18/46)
- Owner transfer coverage (`exec_owner=rust`): 39.13% (18/46)
- Remaining (strict state-write basis): 60.87%
- Details: [0080-owner-transfer-allowlist-phase11.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0080-owner-transfer-allowlist-phase11.md)

### 0081 - economic-tendency runtime active-write port
- Commit: `[rust-r0-181] Port economic-tendency runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 41.30% (19/46)
- Actual state-write coverage (strict): 41.30% (19/46)
- Owner transfer coverage (`exec_owner=rust`): 39.13% (18/46)
- Remaining (strict state-write basis): 58.70%
- Details: [0081-economic-tendency-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0081-economic-tendency-runtime-active-write-port.md)

### 0082 - owner transfer allowlist phase 12
- Commit: `[rust-r0-182] Expand owner-ready allowlist with economic-tendency system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 41.30% (19/46)
- Actual state-write coverage (strict): 41.30% (19/46)
- Owner transfer coverage (`exec_owner=rust`): 41.30% (19/46)
- Remaining (strict state-write basis): 58.70%
- Details: [0082-owner-transfer-allowlist-phase12.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0082-owner-transfer-allowlist-phase12.md)

### 0083 - intelligence runtime active-write port
- Commit: `[rust-r0-183] Port intelligence runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 43.48% (20/46)
- Actual state-write coverage (strict): 43.48% (20/46)
- Owner transfer coverage (`exec_owner=rust`): 41.30% (19/46)
- Remaining (strict state-write basis): 56.52%
- Details: [0083-intelligence-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0083-intelligence-runtime-active-write-port.md)

### 0084 - owner transfer allowlist phase 13
- Commit: `[rust-r0-184] Expand owner-ready allowlist with intelligence system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 43.48% (20/46)
- Actual state-write coverage (strict): 43.48% (20/46)
- Owner transfer coverage (`exec_owner=rust`): 43.48% (20/46)
- Remaining (strict state-write basis): 56.52%
- Details: [0084-owner-transfer-allowlist-phase13.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0084-owner-transfer-allowlist-phase13.md)

### 0085 - memory runtime active-write port
- Commit: `[rust-r0-185] Port memory runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 45.65% (21/46)
- Actual state-write coverage (strict): 45.65% (21/46)
- Owner transfer coverage (`exec_owner=rust`): 43.48% (20/46)
- Remaining (strict state-write basis): 54.35%
- Details: [0085-memory-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0085-memory-runtime-active-write-port.md)

### 0086 - owner transfer allowlist phase 14
- Commit: `[rust-r0-186] Expand owner-ready allowlist with memory system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 45.65% (21/46)
- Actual state-write coverage (strict): 45.65% (21/46)
- Owner transfer coverage (`exec_owner=rust`): 45.65% (21/46)
- Remaining (strict state-write basis): 54.35%
- Details: [0086-owner-transfer-allowlist-phase14.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0086-owner-transfer-allowlist-phase14.md)

### 0087 - trauma-scar runtime active-write port
- Commit: `[rust-r0-187] Port trauma-scar runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 47.83% (22/46)
- Actual state-write coverage (strict): 47.83% (22/46)
- Owner transfer coverage (`exec_owner=rust`): 45.65% (21/46)
- Remaining (strict state-write basis): 52.17%
- Details: [0087-trauma-scar-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0087-trauma-scar-runtime-active-write-port.md)

### 0088 - owner transfer allowlist phase 15
- Commit: `[rust-r0-188] Expand owner-ready allowlist with trauma-scar system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 47.83% (22/46)
- Actual state-write coverage (strict): 47.83% (22/46)
- Owner transfer coverage (`exec_owner=rust`): 47.83% (22/46)
- Remaining (strict state-write basis): 52.17%
- Details: [0088-owner-transfer-allowlist-phase15.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0088-owner-transfer-allowlist-phase15.md)

### 0089 - coping runtime active-write port
- Commit: `[rust-r0-189] Port coping runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 50.00% (23/46)
- Actual state-write coverage (strict): 50.00% (23/46)
- Owner transfer coverage (`exec_owner=rust`): 47.83% (22/46)
- Remaining (strict state-write basis): 50.00%
- Details: [0089-coping-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0089-coping-runtime-active-write-port.md)

### 0090 - owner transfer allowlist phase 16
- Commit: `[rust-r0-190] Expand owner-ready allowlist with coping system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 50.00% (23/46)
- Actual state-write coverage (strict): 50.00% (23/46)
- Owner transfer coverage (`exec_owner=rust`): 50.00% (23/46)
- Remaining (strict state-write basis): 50.00%
- Details: [0090-owner-transfer-allowlist-phase16.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0090-owner-transfer-allowlist-phase16.md)

### 0091 - child-stress-processor runtime active-write port
- Commit: `[rust-r0-191] Port child-stress-processor runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 52.17% (24/46)
- Actual state-write coverage (strict): 52.17% (24/46)
- Owner transfer coverage (`exec_owner=rust`): 50.00% (23/46)
- Remaining (strict state-write basis): 47.83%
- Details: [0091-child-stress-processor-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0091-child-stress-processor-runtime-active-write-port.md)

### 0092 - owner transfer allowlist phase 17
- Commit: `[rust-r0-192] Expand owner-ready allowlist with child-stress-processor system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 52.17% (24/46)
- Actual state-write coverage (strict): 52.17% (24/46)
- Owner transfer coverage (`exec_owner=rust`): 52.17% (24/46)
- Remaining (strict state-write basis): 47.83%
- Details: [0092-owner-transfer-allowlist-phase17.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0092-owner-transfer-allowlist-phase17.md)

### 0093 - movement runtime active-write port
- Commit: `[rust-r0-193] Port movement runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 54.35% (25/46)
- Actual state-write coverage (strict): 54.35% (25/46)
- Owner transfer coverage (`exec_owner=rust`): 52.17% (24/46)
- Remaining (strict state-write basis): 45.65%
- Details: [0093-movement-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0093-movement-runtime-active-write-port.md)

### 0094 - owner transfer allowlist phase 18
- Commit: `[rust-r0-194] Expand owner-ready allowlist with movement system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 54.35% (25/46)
- Actual state-write coverage (strict): 54.35% (25/46)
- Owner transfer coverage (`exec_owner=rust`): 54.35% (25/46)
- Remaining (strict state-write basis): 45.65%
- Details: [0094-owner-transfer-allowlist-phase18.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0094-owner-transfer-allowlist-phase18.md)

### 0095 - childcare runtime active-write port
- Commit: `[rust-r0-195] Port childcare runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 56.52% (26/46)
- Actual state-write coverage (strict): 56.52% (26/46)
- Owner transfer coverage (`exec_owner=rust`): 54.35% (25/46)
- Remaining (strict state-write basis): 43.48%
- Details: [0095-childcare-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0095-childcare-runtime-active-write-port.md)

### 0096 - owner transfer allowlist phase 19
- Commit: `[rust-r0-196] Expand owner-ready allowlist with childcare system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 56.52% (26/46)
- Actual state-write coverage (strict): 56.52% (26/46)
- Owner transfer coverage (`exec_owner=rust`): 56.52% (26/46)
- Remaining (strict state-write basis): 43.48%
- Details: [0096-owner-transfer-allowlist-phase19.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0096-owner-transfer-allowlist-phase19.md)

### 0097 - leader runtime active-write port
- Commit: `[rust-r0-197] Port leader runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 58.70% (27/46)
- Actual state-write coverage (strict): 58.70% (27/46)
- Owner transfer coverage (`exec_owner=rust`): 56.52% (26/46)
- Remaining (strict state-write basis): 41.30%
- Details: [0097-leader-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0097-leader-runtime-active-write-port.md)

### 0098 - owner transfer allowlist phase 20
- Commit: `[rust-r0-198] Expand owner-ready allowlist with leader system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 58.70% (27/46)
- Actual state-write coverage (strict): 58.70% (27/46)
- Owner transfer coverage (`exec_owner=rust`): 58.70% (27/46)
- Remaining (strict state-write basis): 41.30%
- Details: [0098-owner-transfer-allowlist-phase20.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0098-owner-transfer-allowlist-phase20.md)

### 0099 - title runtime active-write port
- Commit: `[rust-r0-199] Port title runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 60.87% (28/46)
- Actual state-write coverage (strict): 60.87% (28/46)
- Owner transfer coverage (`exec_owner=rust`): 58.70% (27/46)
- Remaining (strict state-write basis): 39.13%
- Details: [0099-title-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0099-title-runtime-active-write-port.md)

### 0100 - owner transfer allowlist phase 21
- Commit: `[rust-r0-200] Expand owner-ready allowlist with title system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 60.87% (28/46)
- Actual state-write coverage (strict): 60.87% (28/46)
- Owner transfer coverage (`exec_owner=rust`): 60.87% (28/46)
- Remaining (strict state-write basis): 39.13%
- Details: [0100-owner-transfer-allowlist-phase21.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0100-owner-transfer-allowlist-phase21.md)

### 0101 - stratification-monitor runtime active-write port
- Commit: `[rust-r0-201] Port stratification-monitor runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 63.04% (29/46)
- Actual state-write coverage (strict): 63.04% (29/46)
- Owner transfer coverage (`exec_owner=rust`): 60.87% (28/46)
- Remaining (strict state-write basis): 36.96%
- Details: [0101-stratification-monitor-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0101-stratification-monitor-runtime-active-write-port.md)

### 0102 - owner transfer allowlist phase 22
- Commit: `[rust-r0-202] Expand owner-ready allowlist with stratification monitor`
- Reported implementation coverage (legacy `rust_runtime_impl`): 63.04% (29/46)
- Actual state-write coverage (strict): 63.04% (29/46)
- Owner transfer coverage (`exec_owner=rust`): 63.04% (29/46)
- Remaining (strict state-write basis): 36.96%
- Details: [0102-owner-transfer-allowlist-phase22.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0102-owner-transfer-allowlist-phase22.md)

### 0103 - tension runtime active-write port
- Commit: `[rust-r0-203] Port tension runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 65.22% (30/46)
- Actual state-write coverage (strict): 65.22% (30/46)
- Owner transfer coverage (`exec_owner=rust`): 63.04% (29/46)
- Remaining (strict state-write basis): 34.78%
- Details: [0103-tension-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0103-tension-runtime-active-write-port.md)

### 0104 - owner transfer allowlist phase 23
- Commit: `[rust-r0-204] Expand owner-ready allowlist with tension system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 65.22% (30/46)
- Actual state-write coverage (strict): 65.22% (30/46)
- Owner transfer coverage (`exec_owner=rust`): 65.22% (30/46)
- Remaining (strict state-write basis): 34.78%
- Details: [0104-owner-transfer-allowlist-phase23.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0104-owner-transfer-allowlist-phase23.md)

### 0105 - building-effect runtime active-write port
- Commit: `[rust-r0-205] Port building-effect runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 67.39% (31/46)
- Actual state-write coverage (strict): 67.39% (31/46)
- Owner transfer coverage (`exec_owner=rust`): 65.22% (30/46)
- Remaining (strict state-write basis): 32.61%
- Details: [0105-building-effect-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0105-building-effect-runtime-active-write-port.md)

### 0106 - owner transfer allowlist phase 24
- Commit: `[rust-r0-206] Expand owner-ready allowlist with building-effect system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 67.39% (31/46)
- Actual state-write coverage (strict): 67.39% (31/46)
- Owner transfer coverage (`exec_owner=rust`): 67.39% (31/46)
- Remaining (strict state-write basis): 32.61%
- Details: [0106-owner-transfer-allowlist-phase24.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0106-owner-transfer-allowlist-phase24.md)

### 0107 - migration runtime active-write port
- Commit: `[rust-r0-207] Port migration runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 69.57% (32/46)
- Actual state-write coverage (strict): 69.57% (32/46)
- Owner transfer coverage (`exec_owner=rust`): 67.39% (31/46)
- Remaining (strict state-write basis): 30.43%
- Details: [0107-migration-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0107-migration-runtime-active-write-port.md)

### 0108 - owner transfer allowlist phase 25
- Commit: `[rust-r0-208] Expand owner-ready allowlist with migration system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 69.57% (32/46)
- Actual state-write coverage (strict): 69.57% (32/46)
- Owner transfer coverage (`exec_owner=rust`): 69.57% (32/46)
- Remaining (strict state-write basis): 30.43%
- Details: [0108-owner-transfer-allowlist-phase25.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0108-owner-transfer-allowlist-phase25.md)

### 0109 - population runtime active-write port
- Commit: `[rust-r0-209] Port population runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 71.74% (33/46)
- Actual state-write coverage (strict): 71.74% (33/46)
- Owner transfer coverage (`exec_owner=rust`): 69.57% (32/46)
- Remaining (strict state-write basis): 28.26%
- Details: [0109-population-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0109-population-runtime-active-write-port.md)

### 0110 - owner transfer allowlist phase 26
- Commit: `[rust-r0-210] Expand owner-ready allowlist with population system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 71.74% (33/46)
- Actual state-write coverage (strict): 71.74% (33/46)
- Owner transfer coverage (`exec_owner=rust`): 71.74% (33/46)
- Remaining (strict state-write basis): 28.26%
- Details: [0110-owner-transfer-allowlist-phase26.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0110-owner-transfer-allowlist-phase26.md)

### 0111 - tech-utilization runtime active-write port
- Commit: `[rust-r0-211] Port tech-utilization runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 73.91% (34/46)
- Actual state-write coverage (strict): 73.91% (34/46)
- Owner transfer coverage (`exec_owner=rust`): 71.74% (33/46)
- Remaining (strict state-write basis): 26.09%
- Details: [0111-tech-utilization-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0111-tech-utilization-runtime-active-write-port.md)

### 0112 - owner transfer allowlist phase 27
- Commit: `[rust-r0-212] Expand owner-ready allowlist with tech-utilization system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 73.91% (34/46)
- Actual state-write coverage (strict): 73.91% (34/46)
- Owner transfer coverage (`exec_owner=rust`): 73.91% (34/46)
- Remaining (strict state-write basis): 26.09%
- Details: [0112-owner-transfer-allowlist-phase27.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0112-owner-transfer-allowlist-phase27.md)

### 0113 - tech-maintenance runtime active-write port
- Commit: `[rust-r0-213] Port tech-maintenance runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 76.09% (35/46)
- Actual state-write coverage (strict): 76.09% (35/46)
- Owner transfer coverage (`exec_owner=rust`): 73.91% (34/46)
- Remaining (strict state-write basis): 23.91%
- Details: [0113-tech-maintenance-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0113-tech-maintenance-runtime-active-write-port.md)

### 0114 - owner transfer allowlist phase 28
- Commit: `[rust-r0-214] Expand owner-ready allowlist with tech-maintenance system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 76.09% (35/46)
- Actual state-write coverage (strict): 76.09% (35/46)
- Owner transfer coverage (`exec_owner=rust`): 76.09% (35/46)
- Remaining (strict state-write basis): 23.91%
- Details: [0114-owner-transfer-allowlist-phase28.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0114-owner-transfer-allowlist-phase28.md)

### 0115 - tech-discovery runtime active-write port
- Commit: `[rust-r0-215] Port tech-discovery runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 78.26% (36/46)
- Actual state-write coverage (strict): 78.26% (36/46)
- Owner transfer coverage (`exec_owner=rust`): 76.09% (35/46)
- Remaining (strict state-write basis): 21.74%
- Details: [0115-tech-discovery-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0115-tech-discovery-runtime-active-write-port.md)

### 0116 - owner transfer allowlist phase 29
- Commit: `[rust-r0-216] Expand owner-ready allowlist with tech-discovery system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 78.26% (36/46)
- Actual state-write coverage (strict): 78.26% (36/46)
- Owner transfer coverage (`exec_owner=rust`): 78.26% (36/46)
- Remaining (strict state-write basis): 21.74%
- Details: [0116-owner-transfer-allowlist-phase29.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0116-owner-transfer-allowlist-phase29.md)

### 0117 - tech-propagation runtime active-write port
- Commit: `[rust-r0-217] Port tech-propagation runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 80.43% (37/46)
- Actual state-write coverage (strict): 80.43% (37/46)
- Owner transfer coverage (`exec_owner=rust`): 78.26% (36/46)
- Remaining (strict state-write basis): 19.57%
- Details: [0117-tech-propagation-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0117-tech-propagation-runtime-active-write-port.md)

### 0118 - owner transfer allowlist phase 30
- Commit: `[rust-r0-218] Expand owner-ready allowlist with tech-propagation system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 80.43% (37/46)
- Actual state-write coverage (strict): 80.43% (37/46)
- Owner transfer coverage (`exec_owner=rust`): 80.43% (37/46)
- Remaining (strict state-write basis): 19.57%
- Details: [0118-owner-transfer-allowlist-phase30.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0118-owner-transfer-allowlist-phase30.md)

### 0119 - gathering runtime active-write port
- Commit: `[rust-r0-219] Port gathering runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 82.61% (38/46)
- Actual state-write coverage (strict): 82.61% (38/46)
- Owner transfer coverage (`exec_owner=rust`): 80.43% (37/46)
- Remaining (strict state-write basis): 17.39%
- Details: [0119-gathering-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0119-gathering-runtime-active-write-port.md)

### 0120 - owner transfer allowlist phase 31
- Commit: `[rust-r0-220] Expand owner-ready allowlist with gathering system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 82.61% (38/46)
- Actual state-write coverage (strict): 82.61% (38/46)
- Owner transfer coverage (`exec_owner=rust`): 82.61% (38/46)
- Remaining (strict state-write basis): 17.39%
- Details: [0120-owner-transfer-allowlist-phase31.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0120-owner-transfer-allowlist-phase31.md)

### 0121 - construction runtime active-write port
- Commit: `[rust-r0-221] Port construction runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 84.78% (39/46)
- Actual state-write coverage (strict): 84.78% (39/46)
- Owner transfer coverage (`exec_owner=rust`): 82.61% (38/46)
- Remaining (strict state-write basis): 15.22%
- Details: [0121-construction-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0121-construction-runtime-active-write-port.md)

### 0122 - owner transfer allowlist phase 32
- Commit: `[rust-r0-222] Expand owner-ready allowlist with construction system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 84.78% (39/46)
- Actual state-write coverage (strict): 84.78% (39/46)
- Owner transfer coverage (`exec_owner=rust`): 84.78% (39/46)
- Remaining (strict state-write basis): 15.22%
- Details: [0122-owner-transfer-allowlist-phase32.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0122-owner-transfer-allowlist-phase32.md)

### 0123 - family runtime active-write port
- Commit: `[rust-r0-223] Port family runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 86.96% (40/46)
- Actual state-write coverage (strict): 86.96% (40/46)
- Owner transfer coverage (`exec_owner=rust`): 84.78% (39/46)
- Remaining (strict state-write basis): 13.04%
- Details: [0123-family-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0123-family-runtime-active-write-port.md)

### 0124 - owner transfer allowlist phase 33
- Commit: `[rust-r0-224] Expand owner-ready allowlist with family system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 86.96% (40/46)
- Actual state-write coverage (strict): 86.96% (40/46)
- Owner transfer coverage (`exec_owner=rust`): 86.96% (40/46)
- Remaining (strict state-write basis): 13.04%
- Details: [0124-owner-transfer-allowlist-phase33.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0124-owner-transfer-allowlist-phase33.md)

### 0125 - intergenerational runtime active-write port
- Commit: `[rust-r0-225] Port intergenerational runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 89.13% (41/46)
- Actual state-write coverage (strict): 89.13% (41/46)
- Owner transfer coverage (`exec_owner=rust`): 86.96% (40/46)
- Remaining (strict state-write basis): 10.87%
- Details: [0125-intergenerational-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0125-intergenerational-runtime-active-write-port.md)

### 0126 - owner transfer allowlist phase 34
- Commit: `[rust-r0-226] Expand owner-ready allowlist with intergenerational system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 89.13% (41/46)
- Actual state-write coverage (strict): 89.13% (41/46)
- Owner transfer coverage (`exec_owner=rust`): 89.13% (41/46)
- Remaining (strict state-write basis): 10.87%
- Details: [0126-owner-transfer-allowlist-phase34.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0126-owner-transfer-allowlist-phase34.md)

### 0127 - parenting runtime active-write port
- Commit: `[rust-r0-227] Port parenting runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 91.30% (42/46)
- Actual state-write coverage (strict): 91.30% (42/46)
- Owner transfer coverage (`exec_owner=rust`): 89.13% (41/46)
- Remaining (strict state-write basis): 8.70%
- Details: [0127-parenting-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0127-parenting-runtime-active-write-port.md)

### 0128 - owner transfer allowlist phase 35
- Commit: `[rust-r0-228] Expand owner-ready allowlist with parenting system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 91.30% (42/46)
- Actual state-write coverage (strict): 91.30% (42/46)
- Owner transfer coverage (`exec_owner=rust`): 91.30% (42/46)
- Remaining (strict state-write basis): 8.70%
- Details: [0128-owner-transfer-allowlist-phase35.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0128-owner-transfer-allowlist-phase35.md)

### 0129 - stats recorder runtime active-write port
- Commit: `[rust-r0-229] Port stats recorder runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 93.48% (43/46)
- Actual state-write coverage (strict): 93.48% (43/46)
- Owner transfer coverage (`exec_owner=rust`): 91.30% (42/46)
- Remaining (strict state-write basis): 6.52%
- Details: [0129-stats-recorder-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0129-stats-recorder-runtime-active-write-port.md)

### 0130 - owner transfer allowlist phase 36
- Commit: `[rust-r0-230] Expand owner-ready allowlist with stats recorder`
- Reported implementation coverage (legacy `rust_runtime_impl`): 93.48% (43/46)
- Actual state-write coverage (strict): 93.48% (43/46)
- Owner transfer coverage (`exec_owner=rust`): 93.48% (43/46)
- Remaining (strict state-write basis): 6.52%
- Details: [0130-owner-transfer-allowlist-phase36.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0130-owner-transfer-allowlist-phase36.md)

### 0131 - stat sync runtime active-write port
- Commit: `[rust-r0-231] Port stat sync runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 95.65% (44/46)
- Actual state-write coverage (strict): 95.65% (44/46)
- Owner transfer coverage (`exec_owner=rust`): 93.48% (43/46)
- Remaining (strict state-write basis): 4.35%
- Details: [0131-stat-sync-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0131-stat-sync-runtime-active-write-port.md)

### 0132 - stat threshold runtime active-write port
- Commit: `[rust-r0-232] Port stat threshold runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 97.83% (45/46)
- Actual state-write coverage (strict): 97.83% (45/46)
- Owner transfer coverage (`exec_owner=rust`): 93.48% (43/46)
- Remaining (strict state-write basis): 2.17%
- Details: [0132-stat-threshold-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0132-stat-threshold-runtime-active-write-port.md)

### 0133 - behavior runtime active-write port
- Commit: `[rust-r0-233] Port behavior runtime to active-write and update strict tracking`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 93.48% (43/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0133-behavior-runtime-active-write-port.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0133-behavior-runtime-active-write-port.md)

### 0134 - owner transfer allowlist phase 37
- Commit: `[rust-r0-234] Expand owner-ready allowlist with behavior system`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 95.65% (44/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0134-owner-transfer-allowlist-phase37.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0134-owner-transfer-allowlist-phase37.md)

### 0135 - owner transfer allowlist phase 38
- Commit: `[rust-r0-235] Expand owner-ready allowlist with stat sync and threshold`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 100.00% (46/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0135-owner-transfer-allowlist-phase38.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0135-owner-transfer-allowlist-phase38.md)

### 0136 - runtime default cutover to rust primary
- Commit: `[rust-r0-236] Switch default simulation runtime mode to rust primary`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 100.00% (46/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0136-runtime-default-cutover-rust-primary.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0136-runtime-default-cutover-rust-primary.md)

### 0137 - bus v2 event payload expansion
- Commit: `[rust-r0-237] Expand Bus v2 event mapping and payload coverage`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 100.00% (46/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0137-bus-v2-event-payload-expansion.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0137-bus-v2-event-payload-expansion.md)

### 0138 - locale fluent rust indexing
- Commit: `[rust-r0-238] Prefer Rust Fluent formatting when building locale key index`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 100.00% (46/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0138-locale-fluent-rust-indexing.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0138-locale-fluent-rust-indexing.md)

### 0139 - gpu domain exposure pathfinding only
- Commit: `[rust-r0-239] Restrict compute domain exposure to pathfinding only`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 100.00% (46/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0139-gpu-domain-exposure-pathfinding-only.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0139-gpu-domain-exposure-pathfinding-only.md)

### 0140 - disable gpu placeholder resolution
- Commit: `[rust-r0-240] Disable pathfinding GPU placeholder resolution`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 100.00% (46/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0140-disable-gpu-placeholder-resolution.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0140-disable-gpu-placeholder-resolution.md)

### 0141 - save manager ws2 only cleanup
- Commit: `[rust-r0-241] Simplify SaveManager to ws2-only runtime path`
- Reported implementation coverage (legacy `rust_runtime_impl`): 100.00% (46/46)
- Actual state-write coverage (strict): 100.00% (46/46)
- Owner transfer coverage (`exec_owner=rust`): 100.00% (46/46)
- Remaining (strict state-write basis): 0.00%
- Details: [0141-save-manager-ws2-only-cleanup.md](/Users/rexxa/github/new-world-wt/lead/reports/rust-migration/0141-save-manager-ws2-only-cleanup.md)
