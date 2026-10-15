# Part 4 public-shape integration audit

**Result: no prohibited field additions observed in this captured source snapshot.** Exactly two fields were added to existing externally public Rust structs: `AgentRules.routine` and `SocietyRules.family`. This is a bounded integration audit, not a fresh Part 8 review, runtime acceptance, or a complete Part 4 gate.

## Snapshot and scope

- Baseline: `adfdecfece79ab019964729319389fd602685fab`.
- Current HEAD: `44aebafa5acf5668d21418f611d8f0ce3a5d13d3`; Part 4 edits are uncommitted and are bound by the hashes below.
- Capture started: `2026-10-15T09:44:17.005683+00:00`; finished: `2026-10-15T09:44:17.848040+00:00`.
- The coordinator reported trace wiring source frozen before this refreshed capture. All captured files and the full tracked Rust diff were re-read unchanged at the end. A later source fix requires another refresh.
- Compared 38 changed Rust files that already existed at baseline; inventoried 31 added/untracked Rust files separately. All other tracked baseline files are outside the diff, so cannot introduce new fields.
- Examined all 48 existing externally public named-field struct definitions in the changed files, 2 existing public unit structs, and 4 existing restricted-visibility named-field structs. Named fields were compared regardless of whether the field itself is public or private. No existing tuple/multiline-header public structs were omitted by this scan.
- No repository files were written by this audit. No builds, probes, Cargo tests, Godot runs, or commits were performed.

## Allowed additions and private observer state

| Location | Existing type | Change | Classification |
|---|---|---|---|
| `rust/crates/sim-core/src/rules.rs:60` | `AgentRules` | `pub routine: Option<RoutineRule>`; 14 → 15 fields | Allowed by brief §5.2 |
| `rust/crates/sim-core/src/rules.rs:178` | `SocietyRules` | `pub family: Option<FamilyRule>`; 11 → 12 fields | Allowed by brief §5.2 |
| `rust/crates/sim-test/src/probe/agent_trace.rs:105`, `:114`, `:115` | `pub(crate) AgentTracer` | private `routine` and private `family`; 6 → 8 fields | Crate-private observer state, not an external public-struct addition |

`AgentTracer` uses a private `agent_trace` module (`probe/mod.rs:3`); only `ProbeOptions` is re-exported. `routine` is `RoutineTracer`; `family` is `Option<FamilyTracer>`. New family/activity component, view, and tracer types are permitted additions rather than modifications of existing public carriers.

## Explicit protected carriers

| Type | Current definition | Baseline/current field count | Evidence |
|---|---|---|---|
| `Tallies` | `rust/crates/sim-engine/src/lib.rs:67` | 33/33 | Entire source file byte-identical to baseline |
| `SimResources` | `rust/crates/sim-engine/src/lib.rs:228` | 60/60 | Entire source file byte-identical to baseline |
| `FissionRecord` | `rust/crates/sim-engine/src/lib.rs:532` | 14/14 | Entire source file byte-identical to baseline |
| `ProbeArgs` | `rust/crates/sim-test/src/probe/args.rs:6` | 8/8 | No added, removed or visibility-changed fields |
| `ProbeOptions` | `rust/crates/sim-test/src/probe/agent_trace.rs:41` | 1/1 | No added, removed or visibility-changed fields |
| `AgentSnapshotRow` | `rust/crates/sim-bridge/src/ffi/world_node.rs:1416` | 15/15 | No added, removed or visibility-changed fields |
| `AgentDetailRow` | `rust/crates/sim-bridge/src/ffi/world_node.rs:1780` | 9/9 | No added, removed or visibility-changed fields |

Existing Godot dictionary key counts remain **17 snapshot / 9 detail**. Snapshot keys: `ids, xs, ys, states, agent_ids, seek_kinds, target_xs, target_ys, hungers, thirsts, sleeps, carried_foods, carried_woods, target_kinds, life_stages, ages_days, colds`. Detail keys come from `AGENT_DETAIL_DICT_KEYS`: `found, agent_id, x, y, state_tag, hunger, thirst, sleep, target_kind`.
New `get_activity_snapshot` and `get_agent_family` are separate getters. This audit does not reclassify their runtime behavior as verified.
`git diff --name-only BASE -- rust/crates/sim-test/src/worldmodel tools/worldmodel` returned no paths. `sim-engine/src/lib.rs` SHA-256 (identical at baseline and capture): `5181904d4a5c8832b85dda891648a5ce7bc62275d44c354a631f9cae7a4b9826`.

## Existing public structs in changed Rust files

| Current definition | Type | Fields before → now | Added fields |
|---|---|---|---|
| `rust/crates/sim-bridge/src/ffi/world_node.rs:151` | `WorldSimNode` | 5 → 5 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:690` | `CausalEventView` | 16 → 16 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:1416` | `AgentSnapshotRow` | 15 → 15 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:1780` | `AgentDetailRow` | 9 → 9 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:1972` | `ConstructionSnapshotRow` | 5 → 5 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:2060` | `ResourceSnapshotRow` | 5 → 5 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:2254` | `RecentDeathRow` | 4 → 4 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:2353` | `SettlementSnapshotRow` | 9 → 9 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:2536` | `RelationshipSnapshotRow` | 4 → 4 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:2729` | `TimeSnapshot` | 8 → 8 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:2802` | `InterventionReply` | 3 → 3 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:3123` | `WorldInfoSnapshot` | 6 → 6 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:3163` | `TerrainSnapshot` | 4 → 4 | none |
| `rust/crates/sim-bridge/src/ffi/world_node.rs:3195` | `BuildingSnapshotRow` | 4 → 4 | none |
| `rust/crates/sim-core/src/rng.rs:164` | `NoiseFork` | 2 → 2 | none |
| `rust/crates/sim-core/src/rules.rs:9` | `WorldRules` | 4 → 4 | none |
| `rust/crates/sim-core/src/rules.rs:21` | `AgentRules` | 14 → 15 | `routine` (pub) |
| `rust/crates/sim-core/src/rules.rs:68` | `TemperamentRule` | 7 → 7 | none |
| `rust/crates/sim-core/src/rules.rs:168` | `SocialSeekRule` | 2 → 2 | none |
| `rust/crates/sim-core/src/rules.rs:176` | `SocietyRules` | 11 → 12 | `family` (pub) |
| `rust/crates/sim-core/src/rules.rs:226` | `ShelterPlanningRule` | 9 → 9 | none |
| `rust/crates/sim-core/src/rules.rs:264` | `SustainRule` | 4 → 4 | none |
| `rust/crates/sim-core/src/rules.rs:283` | `NightRestRule` | 5 → 5 | none |
| `rust/crates/sim-core/src/rules.rs:297` | `EveningGatherRule` | 3 → 3 | none |
| `rust/crates/sim-core/src/rules.rs:307` | `LifeStageRule` | 5 → 5 | none |
| `rust/crates/sim-core/src/rules.rs:321` | `ForageRule` | 5 → 5 | none |
| `rust/crates/sim-core/src/rules.rs:335` | `StockTargetRule` | 3 → 3 | none |
| `rust/crates/sim-core/src/rules.rs:345` | `AgingRule` | 8 → 8 | none |
| `rust/crates/sim-core/src/rules.rs:366` | `MealRule` | 6 → 6 | none |
| `rust/crates/sim-core/src/rules.rs:382` | `FirewoodRule` | 34 → 34 | none |
| `rust/crates/sim-core/src/rules.rs:484` | `ExposureRule` | 4 → 4 | none |
| `rust/crates/sim-core/src/rules.rs:500` | `FissionRule` | 16 → 16 | none |
| `rust/crates/sim-core/src/rules.rs:563` | `FissionPressure` | 3 → 3 | none |
| `rust/crates/sim-core/src/rules.rs:575` | `FissionHazard` | 8 → 8 | none |
| `rust/crates/sim-core/src/rules.rs:605` | `SeasonRule` | 2 → 2 | none |
| `rust/crates/sim-systems/src/runtime/agent/beds.rs:9` | `BedAssignmentSystem` | 1 → 1 | none |
| `rust/crates/sim-systems/src/runtime/agent/movement.rs:68` | `MovementRng` | 1 → 1 | none |
| `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs:169` | `NeedAvailability` | 3 → 3 | none |
| `rust/crates/sim-systems/src/runtime/settlement/fission.rs:134` | `FissionSystem` | 5 → 5 | none |
| `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:128` | `SettlementSystem` | 10 → 10 | none |
| `rust/crates/sim-systems/src/worldgen/mod.rs:11` | `WorldGenConfig` | 18 → 18 | none |
| `rust/crates/sim-test/src/perf.rs:64` | `RunSpec` | 7 → 7 | none |
| `rust/crates/sim-test/src/perf.rs:195` | `SlowTick` | 4 → 4 | none |
| `rust/crates/sim-test/src/perf.rs:208` | `SlowestTicks` | 1 → 1 | none |
| `rust/crates/sim-test/src/perf.rs:269` | `RunOutput` | 2 → 2 | none |
| `rust/crates/sim-test/src/perf.rs:644` | `PerfArgs` | 9 → 9 | none |
| `rust/crates/sim-test/src/probe/agent_trace.rs:41` | `ProbeOptions` | 1 → 1 | none |
| `rust/crates/sim-test/src/probe/args.rs:6` | `ProbeArgs` | 8 → 8 | none |

Existing public unit structs are unchanged: `AgentMovementSystem` in `rust/crates/sim-systems/src/runtime/agent/movement.rs`; `AgentDecisionSystem` in `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`.

Restricted-visibility existing structs:

- `rust/crates/sim-test/src/probe/agent_trace.rs:105`: `pub(crate) AgentTracer` 6 → 8; additions: routine (private), family (private).
- `rust/crates/sim-test/src/probe/deaths.rs:116`: `pub(super) Census` 7 → 7; additions: none.
- `rust/crates/sim-test/src/probe/deaths.rs:191`: `pub(super) DeathTracker` 4 → 4; additions: none.
- `rust/crates/sim-test/src/probe/runner.rs:22`: `pub(super) RunResult` 20 → 20; additions: none.

## Reproduction commands and current short controls

The audit used these read-only commands from `/Users/rexxa/work/new-world-pass4`:

```bash
git rev-parse HEAD
git diff --name-status adfdecfe -- rust
git diff --name-only adfdecfe -- '*.rs'
git ls-files --others --exclude-standard rust
git diff --unified=3 adfdecfe -- rust/crates/sim-core/src/rules.rs rust/crates/sim-test/src/probe/agent_trace.rs rust/crates/sim-test/src/probe/args.rs rust/crates/sim-test/src/probe/runner.rs
git diff --name-only adfdecfe -- rust/crates/sim-test/src/worldmodel tools/worldmodel rust/crates/sim-engine/src/lib.rs
```

A read-only Python scan loaded each baseline file with `git show BASE:PATH`, enumerated anchored public/restricted `struct` definitions, compared every named field in matching definitions, checked unchanged public unit structs, and rejected unhandled public headers. The complete changed-file inventory and SHA-256 hashes below bind the scan. Existing definitions absent from the changed-file list are unchanged by the full diff.

These exact test selections exist in the current source; **they were not executed by this audit**. They are short controls after compilation, not long-run identity evidence. Run sequentially only when the coordinator schedules Cargo work:

```bash
cd /Users/rexxa/work/new-world-pass4/rust
CARGO_BUILD_JOBS=6 cargo test -p sim-core --lib rules::routine::tests::v21_changes_only_the_optional_routine_rule -- --exact --test-threads=1
CARGO_BUILD_JOBS=6 cargo test -p sim-test --lib probe::pass17::death_tests::observer_is_absent_from_old_worlds_and_checks_tick_and_single_drain -- --exact --test-threads=1
CARGO_BUILD_JOBS=6 cargo test -p sim-test --test harness_revitalize_pass17_frozen harness_game_v20_rules_pinned_field_by_field -- --exact --test-threads=1
CARGO_BUILD_JOBS=6 cargo test -p sim-test --test harness_revitalize_pass16_frozen harness_game_v19_rules_pinned_field_by_field -- --exact --test-threads=1
CARGO_BUILD_JOBS=6 cargo test -p sim-test --test harness_revitalize_pass17_family family_none_is_inert_and_dead_partner_is_cleared_only_at_pair_minute -- --exact --test-threads=1
```

Coverage boundaries: the first restores `routine=None` and compares all rules to v20 (`rules/routine.rs:223`); the observer test verifies no observer registration in default no-routine rules (`probe/pass17_death_tests.rs:330`); the v20 pin test explicitly checks `routine=None` and `family=None` for five v20/v20_inert rule seeds (`harness_revitalize_pass17_frozen.rs:33`, `:73`, `:225`); the v19 selection checks its established rule fields for v19/v19_nofission (`harness_revitalize_pass16_frozen.rs:280`) but predates explicit routine/family assertions; the family test covers the disabled family path (`harness_revitalize_pass17_family.rs:34`). None alone establishes full old-scene identity.
Do not call `harness_v20_fingerprints_unchanged` or `harness_v20_inert_fingerprints_unchanged` short controls: their shared checker advances each of five seeds to 40,320 ticks (28 days; frozen harness lines 331–378).

## Captured Rust input manifest

Tracked Rust diff SHA-256 (`git diff --no-ext-diff adfdecfece79ab019964729319389fd602685fab -- '*.rs'`): `a7a7937a3ede33d80319718869d3d1644c349300d235a5c49f1c66f92d148d5c`.

```text
c8218a717ce9eb8f9c984d7a172fce567735d47a0e56f85cbc575eac8a57d0aa  rust/crates/sim-bridge/src/ffi/activity.rs
bb1d81324bbca935a652d6a3290d76d8f8517858cc3ab0bfde668c362f6f21ed  rust/crates/sim-bridge/src/ffi/family.rs
3b4ba9287963ce99e4ba678dd956c85c8c1b07be3dccd7ae006ab9b6cf2ff987  rust/crates/sim-bridge/src/ffi/mod.rs
92f2182b6c5f27117586ebd8a2166019d957942444b690aeed805f8b4fe14ba9  rust/crates/sim-bridge/src/ffi/world_node.rs
bd873c6744131486a8701d7543d47875884a467515304567d6bc7258b3fed09d  rust/crates/sim-core/src/causal/event.rs
7e837c3cbc208d6fc17783362e623ce5e8b3c40d8478be4a835a8d340094c412  rust/crates/sim-core/src/components/activity.rs
959e8b5790fe9ed4ac164213b06e142d6080d34c75505ebde7238abc70f7c999  rust/crates/sim-core/src/components/agent_state.rs
38b94d108d39f199c924cf826e16d7cffe7d20b4d03888c9e05678c804baba00  rust/crates/sim-core/src/components/family.rs
8696a6e1c63bf51c3145118ae3ed9b91b8a41802e4eaf31c4473902ef2ae55bf  rust/crates/sim-core/src/components/family_events.rs
fec9bdb8a9af1a14199480a41b5adf92014680d3f66ec736560eb04961e7e192  rust/crates/sim-core/src/components/mod.rs
c2d1a56375c307cfa1eb79b770ef4f7e0ff127660902b2a19b4c73a89d064949  rust/crates/sim-core/src/rng.rs
82ff51f2e12e31a879a711bc89739e75a4df43d1ab5ffc053c75370f963eee54  rust/crates/sim-core/src/rules.rs
05dfc32597f810f12f453ff643c9948587787bc48d5b02d81f0365baf1fd534e  rust/crates/sim-core/src/rules/routine.rs
a2aa1880759cf7cf12bc63d6b540f5fbe9e360706823f5f0f45d205af5201d25  rust/crates/sim-systems/src/lib.rs
3ad0e6231f258b5213dfa16e3590cf0242d36410b3dd1bfe3c7d8db3ccfecd26  rust/crates/sim-systems/src/runtime/agent/beds.rs
25441955ec67aa6ca63f73a283615e0a9d0e9720c7dbcaa61ece3efe5d9de766  rust/crates/sim-systems/src/runtime/agent/households.rs
b079429b2301c9f4e41290ecd9916dd1b20d4710af263fc56d3bbaaceab6e6cc  rust/crates/sim-systems/src/runtime/agent/mod.rs
9980e7eeb3145cabf7794226e2048100aec04cf7be1ec0f443472ffe75371ca3  rust/crates/sim-systems/src/runtime/agent/movement.rs
b42d84fc8f23a1e0c0d25e2300366e6385eb8fc6f5e6739965955523bfc598b4  rust/crates/sim-systems/src/runtime/agent/pairing.rs
667c88c82cf182ca5f0ca0d6781d8c0c1c7526a04a4e6d8b3dd26ab3a907b424  rust/crates/sim-systems/src/runtime/decision/agent_decision.rs
7ccaaf49aeecc6a48b5dfa9473aa713e7ca3c3c72b47d85c8cd06d8909b33550  rust/crates/sim-systems/src/runtime/decision/mod.rs
996b1bd28980e50a704a53a49043935b3aafadd3848e62297632c0b14528a77a  rust/crates/sim-systems/src/runtime/decision/routine/lifecycle.rs
225a7621b01f0d189b39a1f4ecee8c44a98957904b798c70e4866a01f8c5102c  rust/crates/sim-systems/src/runtime/decision/routine/mod.rs
ae433779fcb57261218209d3b28011bf69c25422af341086addbb142220e9b11  rust/crates/sim-systems/src/runtime/decision/routine/places.rs
c845f983bbcc0c8aacb4073b73c3492cfd75b265f6e6b24829d117fd5f8acd0c  rust/crates/sim-systems/src/runtime/settlement/fission.rs
0cd553bae1cd58da27babbab0e981e01c47661f815bd7da32cdc8780b42fc73f  rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs
520c492f0bd8c1e7f61bf9ee3a646620cfd487406d14c1e60c1e296bf7a53355  rust/crates/sim-systems/src/worldgen/camp.rs
be9595995a06c5ee329caa4686f79f2013d781172cfd30ccbbb7874ce66321be  rust/crates/sim-systems/src/worldgen/mod.rs
4843870afbe2181f72f66d5db9bc21c8ab32de315232e8b177fde3482bcc7351  rust/crates/sim-test/src/perf.rs
118950ce0e4d2bcd912f3c73c85510776662f6592c762073fbbc88f8fe47feb0  rust/crates/sim-test/src/probe/agent_trace.rs
422d1c11bec1f833972ed7f296bd6c75d0093e80d8065f2a013ffd33306899d3  rust/crates/sim-test/src/probe/args.rs
edc9a2a6be21aba2e8eb808ff872ebc319fbb05459ad83e2c56502b2e71fe3cc  rust/crates/sim-test/src/probe/deaths.rs
18f69ac447ead5ca330a9c40abb68f3b60154b3814f5c1fc98f3cf11f09527ef  rust/crates/sim-test/src/probe/family_events.rs
ce749225356fed33f5520f00963b99a8e55429be10592869243716f93010dfd8  rust/crates/sim-test/src/probe/family_samples.rs
dcc5bf92348c8d36fd3b2ddb482805fcad69b304b84840a12cd829187d145eb3  rust/crates/sim-test/src/probe/family_trace.rs
169f302776f2569fb0284dc148802f3d115ad829bd1d3c0401fab9ccfe3e66ed  rust/crates/sim-test/src/probe/family_trace_tests.rs
215464257b5d6ddde5d5d09eb88d017d132c8beeec79237417d30361ff6ac685  rust/crates/sim-test/src/probe/fingerprint.rs
4f2a655e7fdbd534a0ddfed53d71d4726e1b83a9677c37b329a7f74634681287  rust/crates/sim-test/src/probe/mod.rs
700bbbb6218c74c2d5a20a1e79c3cc38c060aa9f6ab30f52e771d352d68e687b  rust/crates/sim-test/src/probe/pass12.rs
357f6e1004516a30187ab2c418c2ad9c720939a2a419770e427aff6721c4450a  rust/crates/sim-test/src/probe/pass17.rs
9c824ef4bfe1025d47cc2f7b99fd534686eacdd633fbfe253d82c44dc18609c8  rust/crates/sim-test/src/probe/pass17_activity.rs
c275086dc096205249149bd21238ea12e05ffb1dd0fb1414905aab859d2260e8  rust/crates/sim-test/src/probe/pass17_batch_tests.rs
765d911c55198cf06ce01aac8ad5c448f3e091624e4fa1fc3398191bdf629e11  rust/crates/sim-test/src/probe/pass17_death_tests.rs
10a84f7d6feb0335b3409d8da1b5a3e08f10b9886344656ee7c080e2d505cc2e  rust/crates/sim-test/src/probe/pass17_tests.rs
8eeb28c7c51777dbd9bd38cdeec2017dd4c31d223e77fc8dff1ebbb35282091a  rust/crates/sim-test/src/probe/routine_observer.rs
6acc68a817bc21a849e7b98dcdf7353e5e03fef62920eb56c854f61fdf608035  rust/crates/sim-test/src/probe/runner.rs
09515ff3d7ceb5744fcda772e3b1cccba224ef173002708d0de7e633e9d0e98e  rust/crates/sim-test/src/scenes.rs
67a91882f016ec53f98cde9fd01fe884da7d05f1fbc07cf0fac82bbe6a97704f  rust/crates/sim-test/tests/harness_p5_beta_decision.rs
d0af6f58dae0646a3de59e0917944402909f8fb96bf1d2e04ab619e6a36d37c3  rust/crates/sim-test/tests/harness_p5_gamma_sleep_daynight_chronicle.rs
c7db8dae1697bd47781eecc71f75a4b0b768b8bb36fb88341364c825844f921c  rust/crates/sim-test/tests/harness_p6_alpha_construction_components.rs
568a0057d546825488a76b034c97a46ac975f70bd9323d591d41224863282075  rust/crates/sim-test/tests/harness_p7_alpha_social_components.rs
339ff562c5884527733f53bdd196991304f2184d5a7a5aa315c04000d3291c46  rust/crates/sim-test/tests/harness_revitalize_pass10_frozen.rs
ea49775d5bd5802272670c9d5e2babd9428be3ba797b0fb03cf68e5a3b77179d  rust/crates/sim-test/tests/harness_revitalize_pass11_frozen.rs
3b915c1db50ad0ce2fa329062d03bfb8a3342ad46b47806dbb7d3e29675c2b10  rust/crates/sim-test/tests/harness_revitalize_pass12_frozen.rs
aab425d60db361fcfada7351c7fa98fc4e914d1d152e9635016bb12ea56d4312  rust/crates/sim-test/tests/harness_revitalize_pass13_frozen.rs
2ffa260873054f357b6dbb403be322213329000502e6d124b7dd9f8299e71645  rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs
cf508586fd823aa4f7d34149077317fc4e436032891e2380dbaa390ab0e5b7ba  rust/crates/sim-test/tests/harness_revitalize_pass17_family_beds.rs
f06b1c728a26fa84ffc4e0c7ade689b9747a75bfed103e54680496bd4160e5e0  rust/crates/sim-test/tests/harness_revitalize_pass17_family_births.rs
f6468ce8f512403aab379a3ab8378aa0eddeda9eb69b45be579aa659d72b6b22  rust/crates/sim-test/tests/harness_revitalize_pass17_family_fission.rs
913e6f289340aa81620dbb3a6cde0af0ec8e38108793db573d4d0d3d3b6f2536  rust/crates/sim-test/tests/harness_revitalize_pass17_frozen.rs
7c2b3866e4cd7006ccaf828a11bda94d0dd438faf9b5824a7636fc8164f2c80c  rust/crates/sim-test/tests/harness_revitalize_pass17_routine.rs
4526674aac935833a3cb8d6c0b5e738f5a206d260780113bd7fa9b7515543a3e  rust/crates/sim-test/tests/harness_revitalize_pass17_routine_edges.rs
42e210867217ba9f27587caf3f48e3e19d04e6889a213e71d353925a72895da3  rust/crates/sim-test/tests/harness_revitalize_pass17_routine_invitations.rs
2b80a5d34daf290be52316b3572004f2cf04c59ebec60c023bee2f1726923515  rust/crates/sim-test/tests/harness_revitalize_pass6_frozen.rs
ecbc2f44452a7cf47a902d11ce4735ebab21187c7a289a2402ed8b4209495bf6  rust/crates/sim-test/tests/harness_revitalize_pass7_frozen.rs
c97e89bb349750a45d49cd70052b441965d0799fcc3e50f2f1af4a58d243f0fb  rust/crates/sim-test/tests/harness_revitalize_pass8_frozen.rs
e8de337e720750e14d3e5db64be5cc2f906667636fb0b8357a8d5d4d4144117c  rust/crates/sim-test/tests/harness_revitalize_pass9_frozen.rs
60894ef85e15a72671a003f173f9d1d10ffb5a8ba9244c5120337dc19932774a  rust/crates/sim-test/tests/pass17_family_support/mod.rs
54891cecbccfb4b0481f1ca26533a47528b3add89f4829ae8b795497ccecfef0  rust/crates/sim-test/tests/pass17_routine_support/mod.rs
```

Graft discovery for this audit: one call, estimated 25,123 tokens / $0.01 saved.
