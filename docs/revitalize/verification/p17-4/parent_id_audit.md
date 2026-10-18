# Pass 17 Part 4 parent-ID and system-registration audit

This is a bounded static source audit during parallel Part 4 implementation, not a Part 8 fresh review. Only `/Users/rexxa/work/new-world-pass4` was inspected. No Git commands, builds, tests, recorder execution, or repository edits were performed. This external report is the only authored file. Concurrent implementation can move the cited lines; the final search indexes and capture hashes below preserve the inspected surface.

## Findings

The only pre-existing production slot-zero-specific reader is the lineage derivation in `SettlementSystem::run_births`. Its meaning is “the parent whose lineage key and birth counter generate the child,” so making the mother first deliberately changes that child's keyed identity. No external reader assumes slot zero is the father, male, or lowest AgentId. Temperament inheritance and the existing observer use both parents symmetrically.

The new `Kin` construction intentionally adds a first-slot-as-mother reader. For a real couple, `[mother, father]` must remain in that order even when the mother's ID is higher. For fallback births, the first and second lowest IDs remain nominal mother/father fields as required by the brief; their sex/age must not be used to reject the old fallback selection.

Adding Pairing through `register_rhythm_systems` increases the default registry by one but requires **no changes to existing count/name/order assertions**. Exact counts apply to Phase 2 helpers or private mock engines. Default-runtime assertions check presence, named-system multiplicity, or relative order. This is a source conclusion, not a claim that tests were executed.

The coordinator confirmed that the core `AgentBorn.parent_ids` documentation is corrected and the runtime owner is adding the explicit Maturation < Pairing < Bed registration assertion. These are Part 4 implementation follow-ups, not findings from a fresh final review.

## Direct producer and reader inventory

Contract: [PASS17_GOAL.md:179](/Users/rexxa/work/new-world-pass4/docs/revitalize/PASS17_GOAL.md:179) requires maternal lineage for qualifying couples and old fallback otherwise. [Line 176](/Users/rexxa/work/new-world-pass4/docs/revitalize/PASS17_GOAL.md:176) requires Pairing priority 144 after maturation and no writes/draws without family.

| Exact source/function | Interpretation and action |
|---|---|
| [settlement_system.rs:802](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:802), `SettlementSystem::run_births`, selection at 810–820 | New family branch constructs `[mother, father]`; otherwise the old sorted-member/take-two code applies. `family == None` skips couple lookup. Preserve this branch and never sort the real couple after selection. |
| [pairing.rs:98](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/agent/pairing.rs:98), `birth_parents` | Producer dependency: selects a living reciprocal pair, female adult/non-elder, oldest `last_birth_tick`, then lower mother ID. It does not consume an `AgentBorn` vector. |
| [settlement_system.rs:828](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:828), `run_births` | `parent_ids.first()` finds the first person's mutable Lineage and calls `child`. The selected mother's counter now advances for real couples. Missing first-parent lineage retains the existing settlement/tick-key fallback. Intended semantic change; no external reader repair. |
| [lineage.rs:28](/Users/rexxa/work/new-world-pass4/rust/crates/sim-core/src/components/lineage.rs:28), `Lineage::child` | Hashes first-parent key and that parent's `births`, then increments `births`. This remains a count of children **as first parent**, not both parents' total biological children. Existing docs already state that meaning. |
| [settlement_system.rs:852](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:852), `run_births` | Reads the first two IDs into matching temperament slots and calls `Temperament::child` at 911. Both identities change for a different couple; no maternal/paternal weighting. |
| [temperament.rs:80](/Users/rexxa/work/new-world-pass4/rust/crates/sim-core/src/components/temperament.rs:80), `Temperament::child` | Integer mean of two known parents; identical one-parent handling in either slot. Symmetric for a fixed child key. No change needed. |
| [settlement_system.rs:922](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:922), `run_births` | New reader: `Kin.mother = parent_ids.first()`, `Kin.father = parent_ids.get(1)`. Intentional slot meaning. Fallback labels remain nominal as required. |
| [settlement_system.rs:930](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:930), `run_births` | New `FamilyEvent::Born` copies Kin IDs and explicitly records `fallback: family_parents.is_none()`. Preserve that flag; no parent reordering. |
| [settlement_system.rs:959](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:959), `run_births` | Moves the vector into `AgentBorn`. The separate `parent` field is still the settlement formation **event** ID, not a biological parent. |
| [agent_trace.rs:259](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/src/probe/agent_trace.rs:259), `AgentTracer::after_tick` | Clones parent IDs, fills matching temperament slots for parents alive before the tick (269–273), then recomputes with the child's actual key (274–275). No lowest-ID/father-first assumption. Preserve vector alignment. |
| [agent_trace.rs:380](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/src/probe/agent_trace.rs:380), `AgentTracer::finish` | Exports `parents` and aligned `parent_temperaments` (382). Values/order intentionally change in v22; schema does not. |
| [evaluate_pass16.py:345](/Users/rexxa/work/new-world-pass4/tools/revitalize/evaluate_pass16.py:345), `t4_block` / nested `births` | Reads both exported parent temperaments and takes their integer mean (351–354). Symmetric, no sex/order inference. Historical v19/v20 interpretation remains valid. |
| [harness_p10_gamma_settlement_chronicle.rs:159](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:159), `capture_event_signature` | Ordered vector hash at 176–180 is deliberately order-sensitive. A20 (`harness_p10_gamma_a20_determinism_two_runs`, 1230) compares identical runs, not an old frozen payload. No expected-value rewrite. |
| [harness_p10_gamma_settlement_chronicle.rs:612](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:612), `harness_p10_gamma_a_settlement_chronicle` | Checks vector length two, distinct slots (635), founder membership, and formation-event causal parent. No first-slot role or numeric ordering assertion. Legacy fixture remains unchanged. |
| [harness_p10_alpha_settlement.rs:389](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:389), A18; A19 at 411 | Constructs empty parent vectors; tests accessors and Clone/PartialEq. Variant shape unchanged; no role assumption. |

Other `AgentBorn` matches do not inspect the vector:

- `sim-core/src/causal/event.rs` accessor arms (`id`, `parent`, `tick`, `channel`, at 619/650/671/694) read event metadata. `parent` is a causal event ID.
- `sim-systems/src/runtime/memory/memory_system.rs:248`, `classify_event`, ignores births.
- `sim-bridge/src/ffi/world_node.rs:1026`, `from_event`, exports ID/causal-parent/agent/tick and ignores `parent_ids` via `..`.
- `harness_newborn_mobile.rs`, `collect_run` (319) and `harness_newborn_birth_logic_preserved` (697), count newborn agent/tick events.
- `harness_p10_beta_settlement_system.rs`, `count_agent_born` (135), community-history test (1087), and causal-parent test (1321), count births or inspect the formation-event chain.
- `harness_p10_gamma_settlement_migration.rs:193` and `harness_s16_zeta_social_freeze_fix.rs:138`, `count_agent_born`, count the event kind.
- `harness_p3_alpha_event_recording.rs` tests at 203/331/454 read event ticks; classifiers in `harness_p6_alpha_construction_components.rs:392` and `harness_p6_beta_construction_system.rs:164`/652 read the event kind.

Every exact `parent_ids` / `AgentBorn` occurrence, including comments and metadata-only matches, is preserved in the final index below.

## Downstream effects of the maternal key

| Source/function | Intended effect |
|---|---|
| `settlement_system.rs:905`; `sim-core/src/components/hardiness.rs:27`, `Hardiness::draw` | Child hardiness can differ. |
| `settlement_system.rs:911`; `components/temperament.rs:57`, `Temperament::founder` | Child fresh temperament changes with the key as well as the selected parents' mean; its RD-dependent social rate follows. |
| `settlement_system.rs:944`; `runtime/agent/aging.rs:88`, `keyed_newborn_life` | Lifespan can differ. Birth tick is still supplied by the existing gate execution. |
| `runtime/agent/movement.rs:378`, `AgentMovementSystem::tick` (also 229–232, 290–305) | Keyed pause/dx/dy and child-play draws use that child key. |
| `runtime/decision/routine/mod.rs:119`, `RoutineTick::new`, `start` (148), `draw` (204); `routine/places.rs:105`, `choose` | Routine type/duration draws and place rank consume the key. |
| `runtime/settlement/fission.rs:152`, `roster`, and `FissionSystem::check` at 572/584 | Existing individual departure rank uses the lineage key before separate household grouping. |
| `sim-core/src/components/family.rs:17`, `Sex::from_lineage`; caller `settlement_system.rs:920` | New v22 sex identity uses the maternal-derived child key. |

These identity changes may alter later population and birth-gate inputs. Preserving the gate algorithm is not a promise that v21 and v22 will share every future birth tick. The v15 lineage test in `harness_revitalize_pass9_rng.rs:63` and v16 test in `harness_revitalize_pass10_forks.rs:70` check a first-parent lineage closure, not lowest-ID or paternal semantics. Keep their expected values and old-world pins.

Recommended family regression: mother ID greater than father ID, with a third still-lower-ID settlement member; assert maternal next-child key/counter and agreement among AgentBorn/Kin/trace. Also cover no qualifying couple, dead partner before evening cleanup, and disabled family. This report does not execute those tests.

## Recorder inspected read only

The recorder is `rust/crates/sim-test/src/worldmodel`; there is no indexed `sim-recorder` crate. [export.rs:113](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/src/worldmodel/export.rs:113), `take_row`, never reads `AgentBorn`/`parent_ids`. It collects actual Lineage keys at 201 and exports `agent_keys.npy` at 393–397. Their meaning at 454 is lineage identity for fork-arm matching, not parent ID or sex. A real-couple v22 birth can intentionally change those keys; no recorder schema or interpretation repair is indicated.

The recorder exports lifespan/hardiness at 250/252 and snapshot state/target tags at 236/238. Those can change indirectly through a new lineage. [worldmodel/digest.rs:27](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/src/worldmodel/digest.rs:27), `state_digest`, and [probe/fingerprint.rs:33](/Users/rexxa/work/new-world-pass4/rust/crates/sim-test/src/probe/fingerprint.rs:33), `fingerprint`, do not hash parent vectors or registry names directly. No recorder file was modified.

## Registry mechanics and assertion inventory

[SimEngine::register_system, lib.rs:874](/Users/rexxa/work/new-world-pass4/rust/crates/sim-engine/src/lib.rs:874) appends then stable-sorts by priority. The new [register_rhythm_systems, sim-systems/lib.rs:326](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/lib.rs:326) registers Maturation, Pairing, BedAssignment, Aging in that order. Their relevant priorities are 144, 144, 145. Thus Pairing follows maturation on ticks where maturation is due and precedes beds. [pairing.rs:17](/Users/rexxa/work/new-world-pass4/rust/crates/sim-systems/src/runtime/agent/pairing.rs:17) returns for absent family before clock work, queries, allocation, draws or writes.

The disabled system still adds a dispatch and a profiling row. `new_tick_profile` (`sim-engine/src/lib.rs:911`) sizes its arrays from the actual registry; `tick_profiled` (903) rebuilds stale-size profiles. `sim-test/src/perf.rs:282` sizes buckets dynamically; `tools/revitalize/perf_p15.py:192`/198 reads each run's own names. Old evidence should not be rewritten to add a row retroactively.

| Exact test file/function | Assertion and impact |
|---|---|
| `rust/crates/sim-systems/tests/integration.rs:232` `harness_register_phase2_count`; `:244` `harness_register_phase2_order` | 4 and exact four-name list, Phase 2 helper only. Keep unchanged. |
| `rust/crates/sim-test/tests/harness_phase2.rs:350` `harness_register_phase2_count`; `:362` `harness_register_phase2_order` | Same partial helper; keep 4/exact list. |
| `rust/crates/sim-test/tests/harness_phase2_substantial.rs:810` `harness_substantial_register_phase2_twice_no_panic` | 8 after two Phase 2 calls. Keep 8. |
| `rust/crates/sim-engine/src/lib.rs:1078` `test_engine_new`; `:1106` `test_register_and_run_single_system`; `:1123` `test_priority_sort_lower_first` | Bare count 0, mock count 1, exact low/mid/high names. No default registration; unchanged. |
| `rust/crates/sim-engine/src/lib.rs:1148` `tick_profile_counts_due_ticks_and_every_whole_tick`; `:1177` `tick_profile_with_the_wrong_system_count_is_rebuilt` | Private two-/one-mock lists, dynamic names and call counts. Unchanged. Neither proves equal-priority tie order. |
| `rust/crates/sim-engine/tests/integration.rs:17` `test_zero_systems_baseline`; `:88` `test_multiple_systems_priority_order` | Bare 0/empty and three custom mocks. Keep 0/3 and exact names. |
| `rust/crates/sim-test/src/probe/pass17_death_tests.rs:330` `observer_is_absent_from_old_worlds_and_checks_tick_and_single_drain` | Count 0 after failed observer install on a bare engine. Default registry absent; unchanged. |
| `rust/crates/sim-test/tests/harness_p5_beta_decision.rs:171` `harness_p5_beta_agent_decision_system_executes_between_movement_and_decay` | Relative order on a partial Stage 1 helper, no rhythm systems. Unchanged. |
| `rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs:57` `harness_p6_beta_a1_construction_system_metadata_and_registry` | One Construction, Sleep < Construction < Viz on partial helper. Unchanged. |
| `rust/crates/sim-test/tests/harness_p7_beta_social_system.rs:233` `harness_p7_beta_a1b_system_registered_in_default_runtime_systems`; `:2552` `harness_p7_beta_a21_hot_tier_production_schedule` | One SocialInteraction/one SocialDecay; Construction < SocialInteraction < SocialDecay < Viz. New name does not change these assertions. |
| `rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs:264` `harness_p8_beta_a1b_system_registered_in_default_runtime` | Contains MemorySystem; unchanged. |
| `rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:156` `harness_p10_beta_a1_settlement_runs_after_construction_and_building_stamp`; `:1643` `harness_p10_beta_edge_settlement_system_in_default_runtime` | Relative early ordering/contains Settlement. Unchanged. |
| `rust/crates/sim-test/tests/harness_resource_scarcity.rs:1291` `harness_scarcity_a17_regen_system_registered_in_runtime_schedule` | Contains ResourceRegenSystem; unchanged. |
| `rust/crates/sim-test/tests/harness_starvation_death.rs:758` `harness_starvation_a17_population_stability` (804) | Contains StarvationSystem; name assertion unchanged. Behavioral population checks still require actual runtime verification. |
| `rust/crates/sim-test/src/perf.rs:811` `profiled_and_plain_game_v19_reach_the_same_fingerprint` | Dynamic profile names equal plain names (824); all interval-1 systems have 3,000 calls (826). Both engines gain Pairing; its interval-1 no-op should meet that call count. No literal total to update. |
| `rust/crates/sim-systems/src/runtime/agent/maturation.rs:55` `due_children_restore_rates_without_resetting_needs` (73) | Maturation metadata remains (144,60). New equal-priority ordering needs its own regression. |

## Documentation and verification boundary

The corrected `sim-core/src/causal/event.rs:521` now describes mother/father with old fallback. At inspection, `run_births` comments at 806–809 and 846–847 still unconditionally described lowest-ID parents / real couples as future work; the production registry list at `sim-systems/src/lib.rs:285` and rhythm helper doc at 325 still omitted Pairing. Implementation owners should align those comments while preserving behavior.

Graft exhaustive searches ran first; source grep then covered all Rust/GDScript/Python/shell files beneath `rust/crates`, `tools`, and `scripts` with ignore rules disabled and generated targets/virtualenvs excluded. Parent aliases and the real recorder module were inspected separately. Python filesystem `Path.parents[1]` matches are unrelated. Initial Graft parent/event scan: 106 hits across 17 files, 408 indexed files searched.

The first recorder scope guess and digest filename guess were absent and corrected to `sim-test/src/worldmodel` and `worldmodel/digest.rs`. A family-harness glob had no files yet during concurrent work. One external-report generation attempt used Ruby `filter_map`, unavailable on this machine; it failed before writing the file and was replaced. None of these lookup/tool errors was classified as a test result.

No old assertion rewrite is indicated. New family/order regressions, old-world identity proofs, measured performance, and any later code changes remain subject to the coordinator's verification. This report approves no Stage B acceptance gate.

## Capture metadata

- UTC capture: `2026-10-15T09:39:35Z`.
- Parent/event index: 108 hits across 17 files.
- Registration index: 81 hits across 21 files.
- Graft estimate: 412,456 input tokens saved; $0.20 reported subtotal plus two calls each below $0.01 (approximately $0.21).
- Static source references checked for existing files and line bounds; no build/test executed.

### Source SHA256 at capture

These identify mutable files when the report was saved, not Git commits.

| Source path | SHA256 |
|---|---|
| `docs/revitalize/PASS17_GOAL.md` | `e6583e3fbe36b21731b67965e66faa146f282cacffbfeb155d725f34a29fd879` |
| `rust/crates/sim-bridge/src/ffi/world_node.rs` | `92f2182b6c5f27117586ebd8a2166019d957942444b690aeed805f8b4fe14ba9` |
| `rust/crates/sim-core/src/causal/event.rs` | `bd873c6744131486a8701d7543d47875884a467515304567d6bc7258b3fed09d` |
| `rust/crates/sim-core/src/components/lineage.rs` | `490cc9e1585aab5dbf879450d571400bcb0c27df2f6c3f38d07a11ba014a7902` |
| `rust/crates/sim-core/src/components/settlement.rs` | `d0efca624bbdf6172f6416507e5817ed0d29e6011fa38fe2beba4582d1d713c5` |
| `rust/crates/sim-core/src/components/temperament.rs` | `12fb59a9befac6daabafdb1dbeb1f1d8de9d4c0df32e7c9366f9df33954d26cb` |
| `rust/crates/sim-engine/src/lib.rs` | `5181904d4a5c8832b85dda891648a5ce7bc62275d44c354a631f9cae7a4b9826` |
| `rust/crates/sim-engine/tests/integration.rs` | `9ab447a8a9e15cdce7ff3c0c5dd34bdaecf55e82084638f9a9cf4bc48e325851` |
| `rust/crates/sim-systems/src/lib.rs` | `a2aa1880759cf7cf12bc63d6b540f5fbe9e360706823f5f0f45d205af5201d25` |
| `rust/crates/sim-systems/src/runtime/agent/maturation.rs` | `6282a26984524bf5f2a20c9f46237a912f1d69a179ab7226f34899ee4d77d8da` |
| `rust/crates/sim-systems/src/runtime/agent/mod.rs` | `b079429b2301c9f4e41290ecd9916dd1b20d4710af263fc56d3bbaaceab6e6cc` |
| `rust/crates/sim-systems/src/runtime/agent/pairing.rs` | `b42d84fc8f23a1e0c0d25e2300366e6385eb8fc6f5e6739965955523bfc598b4` |
| `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` | `ff6e99e4069cba92d8a9618fd2341c0824c4e0740d2a5e3df4aae63c8d2102bc` |
| `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs` | `f69a34e9378d1f38479c296eb0be2edb7a2dc8f17df3cad431745f22281ab2e9` |
| `rust/crates/sim-systems/src/runtime/settlement/mod.rs` | `c9ed8b7e3a5078bd5a3cbaba3a381bf198458418599a6d1378b04c4652e725ef` |
| `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | `0cd553bae1cd58da27babbab0e981e01c47661f815bd7da32cdc8780b42fc73f` |
| `rust/crates/sim-systems/tests/integration.rs` | `b8f8f1152dc49f66ee45f86008f7ba83bb6e81733bf051413800d9fcbdbee528` |
| `rust/crates/sim-test/src/perf.rs` | `4843870afbe2181f72f66d5db9bc21c8ab32de315232e8b177fde3482bcc7351` |
| `rust/crates/sim-test/src/probe/agent_trace.rs` | `118950ce0e4d2bcd912f3c73c85510776662f6592c762073fbbc88f8fe47feb0` |
| `rust/crates/sim-test/src/probe/fingerprint.rs` | `215464257b5d6ddde5d5d09eb88d017d132c8beeec79237417d30361ff6ac685` |
| `rust/crates/sim-test/src/probe/pass17_death_tests.rs` | `765d911c55198cf06ce01aac8ad5c448f3e091624e4fa1fc3398191bdf629e11` |
| `rust/crates/sim-test/src/worldmodel/digest.rs` | `3e3e3b873a08577a15cc2c2696dcb09fbb1c82265d97a3175a16fa1e2cae16cd` |
| `rust/crates/sim-test/src/worldmodel/export.rs` | `0aad62b3d3df1ca76f092dfeb5b850cec76c39f62c7b82e07fcb33b4323d44f2` |
| `rust/crates/sim-test/tests/harness_newborn_mobile.rs` | `aa9d3a771d0c7ca3596d2d55f725075587af9d37bdaded5dc8b709c14d768a47` |
| `rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs` | `a9e2c6da1b42bac2990918997b96aa0aa9f2b87827ae0e1832bb944f8bfc189b` |
| `rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs` | `511d230f599e6f1e75d61a7954fbbe571ea0028a59b95cf02e2942908634915b` |
| `rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs` | `3cc5906954fcfb4bc622db2034bcefa4e7a7c7ff9625dc0d50c9a2905872d25d` |
| `rust/crates/sim-test/tests/harness_p10_gamma_settlement_migration.rs` | `e76bf00f1a5df21109126a32105c5c5d212a565f9344b65474b80c9dc1ae071d` |
| `rust/crates/sim-test/tests/harness_p3_alpha_event_recording.rs` | `83c8bcf8a876d2cf5a231d4eb6e3d1844adcc6476cdf96b755d0121065a26976` |
| `rust/crates/sim-test/tests/harness_p5_beta_decision.rs` | `67a91882f016ec53f98cde9fd01fe884da7d05f1fbc07cf0fac82bbe6a97704f` |
| `rust/crates/sim-test/tests/harness_p6_alpha_construction_components.rs` | `c7db8dae1697bd47781eecc71f75a4b0b768b8bb36fb88341364c825844f921c` |
| `rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs` | `d6e80338fe439de2bcbc86b3f9b4f1b02a44fe0ab1deefc1f4259c13fc09558b` |
| `rust/crates/sim-test/tests/harness_p7_beta_social_system.rs` | `eb99928ff5d94a3fb177a6cab75a51d74a91b9f3b0264fe595175cdc2bbfb583` |
| `rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs` | `bd345fe416c812cce8822a98899b6d18a20071dc2f144cc448ef2f00c14cceb9` |
| `rust/crates/sim-test/tests/harness_phase2.rs` | `dedb0bcab19ba4db2a476860f29a8bc4a7cec15da47f67c52d93e49d9fb8ebdc` |
| `rust/crates/sim-test/tests/harness_phase2_substantial.rs` | `86e0c3065adfa111b0ce398f2a92f351a30220d0d9f9f8601d83266c39d060cf` |
| `rust/crates/sim-test/tests/harness_resource_scarcity.rs` | `9cc740d0262b7e59059229525bab177d5a1c93115053ed15d84efe5c402ff7e6` |
| `rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs` | `c23f478e4d44e9647fc982e1d87ba2e98425bd18bfd54042de4b79c7abeede47` |
| `rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs` | `b6b57843a876d8d3e41e9641510b8d73c8840226437c4b1e8495f1b3486c9176` |
| `rust/crates/sim-test/tests/harness_starvation_death.rs` | `762e59b7e44ccb4077a55a13ce1e52bc3b41fe06880ff03700f1ef36e99a8ee2` |
| `tools/revitalize/evaluate_pass16.py` | `25426db59ffc111653a03f40107e4ab0ef6b0d33e286284d34201565588b9ec9` |
| `tools/revitalize/perf_p15.py` | `046b32851bff1e755bef494ae1c905a58dea71ea8016705bd8a209c71c820add` |

## Exact parent/event occurrence index

```text
rust/crates/sim-bridge/src/ffi/world_node.rs:1029:            CausalEvent::AgentBorn { id, parent, agent, tick, .. } => Self {
rust/crates/sim-test/src/probe/agent_trace.rs:7://! computed here from the lineage key and the `AgentBorn` parents, never
rust/crates/sim-test/src/probe/agent_trace.rs:110:    /// Newborns whose `AgentBorn` event was not in their tile's ring.
rust/crates/sim-test/src/probe/agent_trace.rs:263:                // A newborn: its parents from the AgentBorn event at its tile.
rust/crates/sim-test/src/probe/agent_trace.rs:270:                            CausalEvent::AgentBorn { agent: born, parent_ids, .. } if *born == id => Some(parent_ids.clone()),
rust/crates/sim-core/src/causal/event.rs:509:    /// Chain note: `AgentBorn` is a causal root for the agent's lifetime —
rust/crates/sim-core/src/causal/event.rs:513:    AgentBorn {
rust/crates/sim-core/src/causal/event.rs:524:        parent_ids: Vec<AgentId>,
rust/crates/sim-core/src/causal/event.rs:586:    /// Mirrors the `AgentBorn` field shape. A causal leaf for the agent's
rust/crates/sim-core/src/causal/event.rs:619:            | CausalEvent::AgentBorn { id, .. }
rust/crates/sim-core/src/causal/event.rs:650:            | CausalEvent::AgentBorn { parent, .. }
rust/crates/sim-core/src/causal/event.rs:671:            | CausalEvent::AgentBorn { tick, .. }
rust/crates/sim-core/src/causal/event.rs:694:            | CausalEvent::AgentBorn { .. }
tools/revitalize/evaluate_pass16.py:523:    lines.append("- Would-be (v19) temperaments use the AgentBorn parents alive at the end of the previous tick; in v20 the observer's computation is compared with the component (`component_mismatches`).")
rust/crates/sim-core/src/components/settlement.rs:78:    /// `CausalEvent::AgentBorn` with this settlement's id).
rust/crates/sim-systems/src/runtime/settlement/mod.rs:11://!     `CombatCompleted` / `AgentBorn` event ids to each settlement's
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:43://!    `AgentBorn` is appended directly at birth-emit time (step 7).
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:48://!    member's position. Emit `CausalEvent::AgentBorn` parented to the
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:49://!    settlement's `SettlementFormed` event id. Append the AgentBorn id
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:133:    /// `SettlementDissolved` and `AgentBorn` events.
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:135:    /// Tick at which the last `AgentBorn` fired for each settlement.
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:808:            let parent_ids: Vec<AgentId> = if let Some((_, mother, father)) = family_parents {
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:826:                let parent = parent_ids.first().and_then(|first| {
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:849:                        for (slot, id) in parent_ids.iter().take(2).enumerate() {
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:919:                        mother: parent_ids.first().copied(),
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:920:                        father: parent_ids.get(1).copied(),
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:956:                CausalEvent::AgentBorn {
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:960:                    parent_ids,
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:975:                // Append the AgentBorn id to history once (1:1 with the
rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:1104:        // run_births appends AgentBorn directly to history.
rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs:177:            CausalEvent::AgentBorn { .. } => "agent_born",
rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs:715:            CausalEvent::AgentBorn { .. } => "agent_born",
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:136:    count_kind(e, |ev| matches!(ev, CausalEvent::AgentBorn { .. }))
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:547:        "AgentBorn must not fire before BIRTH_COOLDOWN_TICKS"
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:570:        "AgentBorn must fire after BIRTH_COOLDOWN_TICKS elapsed"
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1084:// Type A — BuildingPlaced, CombatCompleted, AgentBorn, SettlementReason.
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1101:    // lifetime) as the lookup surface. Other classes (AgentBorn,
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1119:                    CausalEvent::AgentBorn { .. } => {
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1145:    // (b) AgentBorn — fires after BIRTH_COOLDOWN.
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1152:    // AgentBorn event is still in the tile ring buffer. The born agent has
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1161:            if let Some(CausalEvent::AgentBorn { agent, .. }) =
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1168:        found.expect("AgentBorn event must be findable immediately after birth tick")
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1245:        "AgentBorn must be routed to community history"
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1317:// A23: AgentBorn parent chains to SettlementFormed event id.
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1336:        collect_events(&e, |ev| matches!(ev, CausalEvent::AgentBorn { .. }));
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1339:        if let CausalEvent::AgentBorn { parent, .. } = ev {
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1344:                "AgentBorn.parent must equal SettlementFormed.id"
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:35://!   A18 : `CausalEvent::AgentBorn { id: 99, parent: Some(10), agent: 1,
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:38://!   A19 : `AgentBorn` Clone + PartialEq (`ev.clone() == ev`).
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:387:// ─── A18: AgentBorn { id: 99, parent: Some(10), agent: 1, tick: 42 } ────
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:391:    let ev = CausalEvent::AgentBorn {
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:395:        parent_ids: Vec::new(),
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:405:        "channel() must be None (AgentBorn is not influence-channel-bound)"
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:409:// ─── A19: AgentBorn Clone + PartialEq ────────────────────────────────────
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:412:    let ev = CausalEvent::AgentBorn {
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:416:        parent_ids: Vec::new(),
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:423:        "cloned AgentBorn must compare equal to original via derive(PartialEq)"
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:459:    // still wired through the new `CausalEvent::AgentBorn` exhaustive
rust/crates/sim-test/tests/harness_p10_alpha_settlement.rs:496:    // wired through the new AgentBorn variant's match arms.
rust/crates/sim-systems/src/runtime/memory/memory_system.rs:248:        | CausalEvent::AgentBorn { .. }
rust/crates/sim-test/tests/harness_p10_gamma_settlement_migration.rs:198:            if matches!(ev, CausalEvent::AgentBorn { .. }) {
rust/crates/sim-test/tests/harness_p10_gamma_settlement_migration.rs:1098:        println!("[p10-γ A13a] {births} AgentBorn after cooldown — births preserved ✓");
rust/crates/sim-test/tests/harness_p6_alpha_construction_components.rs:405:            CausalEvent::AgentBorn { .. } => "agent_born",
rust/crates/sim-test/tests/harness_newborn_mobile.rs:95:    /// RAW number of `AgentBorn` causal events accumulated tick-by-tick (each
rust/crates/sim-test/tests/harness_newborn_mobile.rs:99:    /// Distinct agent ids that appeared in an `AgentBorn` event (SECONDARY
rust/crates/sim-test/tests/harness_newborn_mobile.rs:242:/// agent and the accumulated `AgentBorn` event set. Per-tick sampling (every
rust/crates/sim-test/tests/harness_newborn_mobile.rs:315:        // Accumulate AgentBorn events emitted on THIS tick, before the per-tile
rust/crates/sim-test/tests/harness_newborn_mobile.rs:319:                if let CausalEvent::AgentBorn { agent, tick, .. } = ev {
rust/crates/sim-test/tests/harness_newborn_mobile.rs:699:    // radius / community-history / parent-chain). (a) accumulated AgentBorn
rust/crates/sim-test/tests/harness_newborn_mobile.rs:709:    // (a) AgentBorn correspondence.
rust/crates/sim-test/tests/harness_newborn_mobile.rs:713:        "[newborn A9] AgentBorn raw events = {born_events}, distinct born ids = {distinct_born}, \
rust/crates/sim-test/tests/harness_newborn_mobile.rs:718:        "A9(a): raw AgentBorn event count ({born_events}) must equal distinct newborn ids \
rust/crates/sim-test/tests/harness_newborn_mobile.rs:723:        "A9(a'): distinct AgentBorn ids ({distinct_born}) must equal the raw event count \
rust/crates/sim-test/tests/harness_p3_alpha_event_recording.rs:255:            | CausalEvent::AgentBorn { tick, .. }
rust/crates/sim-test/tests/harness_p3_alpha_event_recording.rs:366:            | CausalEvent::AgentBorn { tick, .. }
rust/crates/sim-test/tests/harness_p3_alpha_event_recording.rs:490:            | CausalEvent::AgentBorn { tick, .. }
rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs:137:/// Count `AgentBorn` causal events across all tile ring buffers.
rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs:142:            if matches!(ev, CausalEvent::AgentBorn { .. }) {
rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs:579:    // buildings, run past BIRTH_COOLDOWN_TICKS, assert ≥1 AgentBorn.
rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs:594:    println!("[S16-ζ A9] {births} AgentBorn after cooldown — settlement dynamics preserved ✓");
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:40://!   Tick BIRTH_COOLDOWN_TICKS   = 200 — AgentBorn fires (A9, A10, A11, A12, A12b)
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:176:                CausalEvent::AgentBorn { agent, parent_ids, .. } => {
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:178:                    for p in parent_ids {
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:551:    // A8 — No AgentBorn before cooldown elapses.
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:553:        let n_born = count_kind(&e, |ev| matches!(ev, CausalEvent::AgentBorn { .. }));
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:556:            "A8: cumulative AgentBorn count must be 0 across ticks 1..={} (got {n_born})",
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:559:        println!("[A8] no AgentBorn before tick {} ✓", BIRTH_COOLDOWN_TICKS);
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:603:    // A9 — AgentBorn fires exactly once at tick BIRTH_COOLDOWN_TICKS.
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:604:    let born_events = collect_events(&e, |ev| matches!(ev, CausalEvent::AgentBorn { .. }));
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:608:        "A9: exactly 1 AgentBorn must fire at tick {}; got {}",
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:612:    let (born_id, born_agent_id, born_tick, born_parent, born_parent_ids) = match &born_events[0] {
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:613:        CausalEvent::AgentBorn {
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:614:            id, agent, tick, parent, parent_ids,
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:615:        } => (*id, *agent, *tick, *parent, parent_ids.clone()),
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:620:        "A9: AgentBorn.tick = {born_tick}, expected {BIRTH_COOLDOWN_TICKS}"
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:623:        "[A9] AgentBorn id={born_id} agent={born_agent_id} tick={born_tick} ✓"
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:626:    // A10 — Newborn parent chain: parent_ids.len() == 2 AND both in founder set
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:629:        born_parent_ids.len(),
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:631:        "A10: AgentBorn.parent_ids.len() = {}, expected 2",
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:632:        born_parent_ids.len()
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:635:        born_parent_ids[0], born_parent_ids[1],
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:636:        "A10: parent_ids must be distinct"
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:639:    for pid in &born_parent_ids {
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:648:        "A10: AgentBorn.parent = {born_parent:?}, expected Some({formed_id})"
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:651:        "[A10] AgentBorn.parent_ids={born_parent_ids:?} ⊆ founder_set, distinct, parent==Some({formed_id}) ✓"
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:687:    // A12b — AgentBorn routed to community_history.
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:692:            "A12b: AgentBorn id={born_id} not in community_history; history={:?}",
rust/crates/sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:695:        println!("[A12b] AgentBorn id={born_id} routed to community_history ✓");
```

## Exact registration occurrence index

```text
rust/crates/sim-engine/src/lib.rs:880:    pub fn system_count(&self) -> usize {
rust/crates/sim-engine/src/lib.rs:881:        self.systems.len()
rust/crates/sim-engine/src/lib.rs:886:    pub fn system_names(&self) -> Vec<&str> {
rust/crates/sim-engine/src/lib.rs:903:        if profile.systems.len() != self.systems.len() || profile.last_tick_ns.len() != self.systems.len() {
rust/crates/sim-engine/src/lib.rs:923:            last_tick_ns: vec![0; self.systems.len()],
rust/crates/sim-engine/src/lib.rs:1082:        assert_eq!(engine.system_count(), 0);
rust/crates/sim-engine/src/lib.rs:1115:        assert_eq!(engine.system_count(), 1);
rust/crates/sim-engine/src/lib.rs:1125:        // Register out of priority order; system_names() must reflect sort.
rust/crates/sim-engine/src/lib.rs:1144:        assert_eq!(engine.system_names(), vec!["low", "mid", "high"]);
rust/crates/sim-engine/src/lib.rs:1157:        assert_eq!(names, engine.system_names());
rust/crates/sim-engine/src/lib.rs:1182:        assert_eq!(profile.systems.len(), 1);
rust/crates/sim-engine/tests/integration.rs:19:    assert_eq!(engine.system_count(), 0);
rust/crates/sim-engine/tests/integration.rs:21:    assert!(engine.system_names().is_empty());
rust/crates/sim-engine/tests/integration.rs:127:    assert_eq!(engine.system_count(), 3);
rust/crates/sim-engine/tests/integration.rs:128:    assert_eq!(engine.system_names(), vec!["low", "mid", "high"]);
tools/revitalize/perf_p15.py:192:def system_names(run):
tools/revitalize/perf_p15.py:202:        names = system_names(run)
tools/revitalize/perf_p15.py:328:    names = system_names(next(iter((natural or fed).values())))
rust/crates/sim-test/src/probe/pass17_death_tests.rs:333:    assert_eq!(engine.system_count(), 0);
rust/crates/sim-systems/src/lib.rs:285:/// - 144  MaturationSystem (Revitalize Pass 3, interval 60)
rust/crates/sim-systems/src/lib.rs:327:    engine.register_system(Box::new(runtime::agent::MaturationSystem));
rust/crates/sim-systems/src/lib.rs:328:    engine.register_system(Box::new(runtime::agent::PairingSystem));
rust/crates/sim-systems/src/runtime/resource_regen/mod.rs:17://! extra unbounded entry in `system_names()`).
rust/crates/sim-systems/src/runtime/resource_regen/mod.rs:468:/// (unbounded) entry appears in `system_names()`, which no harness asserts a
rust/crates/sim-systems/tests/integration.rs:230:/// ticks: 0 | components_read: SimEngine::system_count()
rust/crates/sim-systems/tests/integration.rs:233:    // metric: engine.system_count(), threshold: == 4
rust/crates/sim-systems/tests/integration.rs:236:    assert_eq!(engine.system_count(), 4);
rust/crates/sim-systems/tests/integration.rs:241:/// Type A: system_names() must reflect priority sort: 90 < 100 < 110 < 1000
rust/crates/sim-systems/tests/integration.rs:242:/// ticks: 0 | components_read: SimEngine::system_names()
rust/crates/sim-systems/tests/integration.rs:245:    // metric: engine.system_names() in execution order
rust/crates/sim-systems/tests/integration.rs:250:        engine.system_names(),
rust/crates/sim-systems/src/runtime/agent/maturation.rs:8:pub struct MaturationSystem;
rust/crates/sim-systems/src/runtime/agent/maturation.rs:10:impl RuntimeSystem for MaturationSystem {
rust/crates/sim-systems/src/runtime/agent/maturation.rs:12:        "MaturationSystem"
rust/crates/sim-systems/src/runtime/agent/maturation.rs:72:        let mut system = MaturationSystem;
rust/crates/sim-systems/src/runtime/agent/maturation.rs:103:        MaturationSystem.tick(&mut world, &mut resources);
rust/crates/sim-systems/src/runtime/agent/mod.rs:24:pub use maturation::MaturationSystem;
rust/crates/sim-systems/src/runtime/agent/mod.rs:25:pub use pairing::PairingSystem;
rust/crates/sim-test/src/perf.rs:228:    let mut order: Vec<usize> = (0..profile.systems.len()).collect();
rust/crates/sim-test/src/perf.rs:283:    let systems = profile.systems.len();
rust/crates/sim-test/src/perf.rs:824:        assert_eq!(names, plain.system_names());
rust/crates/sim-systems/src/runtime/agent/pairing.rs:10:pub struct PairingSystem;
rust/crates/sim-systems/src/runtime/agent/pairing.rs:12:impl RuntimeSystem for PairingSystem {
rust/crates/sim-systems/src/runtime/agent/pairing.rs:14:        "PairingSystem"
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:159:    // SettlementSystem (138). Verify the ordering via `system_names()`.
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:162:    let names = e.system_names();
rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs:1646:    let names = e.system_names();
rust/crates/sim-test/tests/harness_p7_beta_social_system.rs:237:        .system_names()
rust/crates/sim-test/tests/harness_p7_beta_social_system.rs:2555:    let names = engine.system_names();
rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs:66:    //     uses (register_construction_systems / engine.system_names()).
rust/crates/sim-test/tests/harness_p6_beta_construction_system.rs:73:    let names = engine.system_names();
rust/crates/sim-test/tests/harness_phase2.rs:348:/// ticks: 0 | components_read: SimEngine::system_count()
rust/crates/sim-test/tests/harness_phase2.rs:351:    // metric: engine.system_count(), threshold: == 4
rust/crates/sim-test/tests/harness_phase2.rs:354:    assert_eq!(engine.system_count(), 4);
rust/crates/sim-test/tests/harness_phase2.rs:359:/// Type A: system_names() must reflect priority sort: 90 < 100 < 110 < 1000
rust/crates/sim-test/tests/harness_phase2.rs:360:/// ticks: 0 | components_read: SimEngine::system_names()
rust/crates/sim-test/tests/harness_phase2.rs:363:    // metric: engine.system_names() in execution order
rust/crates/sim-test/tests/harness_phase2.rs:368:        engine.system_names(),
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:5:use sim_systems::runtime::agent::{PairingSystem, households::households};
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:20:    PairingSystem.tick(&mut e.world, &mut e.resources);
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:24:    PairingSystem.tick(&mut e.world, &mut e.resources);
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:44:    PairingSystem.tick(&mut e.world, &mut e.resources);
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:49:    PairingSystem.tick(&mut e.world, &mut e.resources);
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:52:    PairingSystem.tick(&mut e.world, &mut e.resources);
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:76:    PairingSystem.tick(&mut e.world, &mut e.resources);
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:105:    let names = e.system_names();
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:106:    assert_eq!(names.iter().filter(|name| **name == "PairingSystem").count(), 1);
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:108:    assert!(at("MaturationSystem") < at("PairingSystem"));
rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs:109:    assert!(at("PairingSystem") < at("BedAssignmentSystem"));
rust/crates/sim-test/tests/harness_p5_beta_decision.rs:163:// Type A — A5a (observable priority ordering via system_names()) AND
rust/crates/sim-test/tests/harness_p5_beta_decision.rs:172:    // ── A5a: observable priority ordering via system_names() ─────────
rust/crates/sim-test/tests/harness_p5_beta_decision.rs:174:    let names = engine.system_names();
rust/crates/sim-test/tests/harness_phase2_substantial.rs:800:/// Type C: engine.system_count() == 8 after register_phase2_systems called twice.
rust/crates/sim-test/tests/harness_phase2_substantial.rs:808:/// components_read: SimEngine.system_count()
rust/crates/sim-test/tests/harness_phase2_substantial.rs:817:        engine.system_count(),
rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs:267:        engine.system_names().contains(&"MemorySystem"),
rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs:269:        engine.system_names()
rust/crates/sim-test/tests/harness_resource_scarcity.rs:1296:        e.system_names().contains(&"ResourceRegenSystem"),
rust/crates/sim-test/tests/harness_resource_scarcity.rs:1298:        e.system_names()
rust/crates/sim-test/tests/harness_starvation_death.rs:804:        e.system_names().contains(&"StarvationSystem"),
rust/crates/sim-test/tests/harness_starvation_death.rs:806:        e.system_names()
```
