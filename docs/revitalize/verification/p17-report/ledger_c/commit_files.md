# Pass17 커밋별 전체 변경 경로 원장

고정 스냅샷 C: `21d2b478a81d55ac138f0a60b830e06719ec0af3`. 최초 검증 기준 `adfdecfece79ab019964729319389fd602685fab`. 생성 UTC 2026-10-15T11:38:12.590052Z.
Pass17 **25개 커밋**, 순변경 **605개 경로**. P16 기준 커밋1개는 작업 수에서 제외한다. 전체 base..C 순변경은 623개 경로다.

[기계 판독 JSON](commit_files.json), SHA256 `87f14b5483bb8030c919759c73c0cbff0b54389329e73bebded9e06428471c08`.

경로 수는 직전 부모 대비 추가/수정/삭제 개수이며 커밋 사이의 중복을 포함한다.

<a id="commit-2c079df265ef7c16baf8ee5bb119591091b5c3eb"></a>

## 2c079df265ef7c16baf8ee5bb119591091b5c3eb

- 구분: P16 기준 이력 — Pass17 작업 제외
- 제목 / 의도: revitalize(p16-6): Pass 16 report (Korean), gate logs (green on adfdecfe), RUNLOG, PR #510 section
- 커밋 UTC: 2026-10-15T02:32:23Z
- 변경 경로: 19개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/2c079df265ef7c16baf8ee5bb119591091b5c3eb)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| A | `docs/revitalize/REPORT_PASS16.md` |
| A | `docs/revitalize/verification/p16_gate/allowed_paths.log` |
| A | `docs/revitalize/verification/p16_gate/bridge_build.log` |
| A | `docs/revitalize/verification/p16_gate/clippy.log` |
| A | `docs/revitalize/verification/p16_gate/clippy_all_targets.log` |
| A | `docs/revitalize/verification/p16_gate/forbidden_math.log` |
| A | `docs/revitalize/verification/p16_gate/gate.txt` |
| A | `docs/revitalize/verification/p16_gate/godot_import.log` |
| A | `docs/revitalize/verification/p16_gate/perf_quick.log` |
| A | `docs/revitalize/verification/p16_gate/prepare_bridge.log` |
| A | `docs/revitalize/verification/p16_gate/prepare_import.log` |
| A | `docs/revitalize/verification/p16_gate/python_unittest.log` |
| A | `docs/revitalize/verification/p16_gate/python_worldmodel.log` |
| A | `docs/revitalize/verification/p16_gate/smoke.log` |
| A | `docs/revitalize/verification/p16_gate/smoke_errors.log` |
| A | `docs/revitalize/verification/p16_gate/strict_check.log` |
| A | `docs/revitalize/verification/p16_gate/test_debug.log` |
| A | `docs/revitalize/verification/p16_gate/test_release.log` |

<a id="commit-e6bde09c5859bf8ba715d240f59092bd7d45eadf"></a>

## e6bde09c5859bf8ba715d240f59092bd7d45eadf

- 구분: Pass17 Part 0
- 제목 / 의도: revitalize(p17-0): preserve the approved routine and family contract before changes
- 커밋 UTC: 2026-10-15T06:01:55Z
- 변경 경로: 1개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/e6bde09c5859bf8ba715d240f59092bd7d45eadf)

| 상태 | 전체 경로 |
|---|---|
| A | `docs/revitalize/PASS17_GOAL.md` |

<a id="commit-4f10de7534a213607e9bcd15e1af6c4f3cc24d68"></a>

## 4f10de7534a213607e9bcd15e1af6c4f3cc24d68

- 구분: Pass17 Part 0
- 제목 / 의도: revitalize(p17-0): freeze v20 behavior before optional rule fields
- 커밋 UTC: 2026-10-15T06:11:25Z
- 변경 경로: 1개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/4f10de7534a213607e9bcd15e1af6c4f3cc24d68)

| 상태 | 전체 경로 |
|---|---|
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_frozen.rs` |

<a id="commit-bc43c8ddf9a536afd8fb2ae38acec9c6d1beebf2"></a>

## bc43c8ddf9a536afd8fb2ae38acec9c6d1beebf2

- 구분: Pass17 Part 0
- 제목 / 의도: revitalize(p17-0): keep routine and family opt-ins absent in every existing world
- 커밋 UTC: 2026-10-15T06:15:50Z
- 변경 경로: 11개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/bc43c8ddf9a536afd8fb2ae38acec9c6d1beebf2)

| 상태 | 전체 경로 |
|---|---|
| M | `rust/crates/sim-core/src/rules.rs` |
| A | `rust/crates/sim-core/src/rules/routine.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass10_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass11_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass12_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass13_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass17_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass6_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass7_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass8_frozen.rs` |
| M | `rust/crates/sim-test/tests/harness_revitalize_pass9_frozen.rs` |

<a id="commit-c47ad7ec688c19995f4856b6c0bfd34aed58591b"></a>

## c47ad7ec688c19995f4856b6c0bfd34aed58591b

- 구분: Pass17 Part 0
- 제목 / 의도: revitalize(p17-0): make unattended evidence collection strict and cooperative
- 커밋 UTC: 2026-10-15T06:34:18Z
- 변경 경로: 19개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/c47ad7ec688c19995f4856b6c0bfd34aed58591b)

| 상태 | 전체 경로 |
|---|---|
| A | `docs/revitalize/COMPAT_PASS17.md` |
| M | `docs/revitalize/DECISIONS.md` |
| A | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/verification/p17-0/clippy.txt` |
| A | `docs/revitalize/verification/p17-0/core_tests.txt` |
| A | `docs/revitalize/verification/p17-0/fields_pins.txt` |
| A | `docs/revitalize/verification/p17-0/identity_comparator.txt` |
| A | `docs/revitalize/verification/p17-0/operations.txt` |
| A | `docs/revitalize/verification/p17-0/operations_initial_failure.txt` |
| A | `docs/revitalize/verification/p17-0/operations_test_source.py` |
| A | `docs/revitalize/verification/p17-0/pins_debug.txt` |
| A | `docs/revitalize/verification/p17-0/pins_release.txt` |
| A | `tools/revitalize/compare_identity_pass17.py` |
| A | `tools/revitalize/gate_pass17.sh` |
| A | `tools/revitalize/identity_p17.sh` |
| A | `tools/revitalize/perf_p17.sh` |
| A | `tools/revitalize/quiet_lock_p17.sh` |
| A | `tools/revitalize/test_compare_identity_pass17.py` |
| A | `tools/revitalize/wait_mac_p17.sh` |

<a id="commit-65ec44c90d49dffa0908c0458f3c8ecec371bd96"></a>

## 65ec44c90d49dffa0908c0458f3c8ecec371bd96

- 구분: Pass17 Part 0
- 제목 / 의도: revitalize(p17-0): bind future identity checks to complete untouched-world baselines
- 커밋 UTC: 2026-10-15T06:36:24Z
- 변경 경로: 14개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/65ec44c90d49dffa0908c0458f3c8ecec371bd96)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/probe/p17-0_base/binaries.sha256` |
| A | `docs/revitalize/probe/p17-0_base/i1_v19/summary.json` |
| A | `docs/revitalize/probe/p17-0_base/i1_vs_p16.txt` |
| A | `docs/revitalize/probe/p17-0_base/i2_sha256.txt` |
| A | `docs/revitalize/probe/p17-0_base/i3_v16/summary.json` |
| A | `docs/revitalize/probe/p17-0_base/i3_vs_p16.txt` |
| A | `docs/revitalize/probe/p17-0_base/i4_v20/summary.json` |
| A | `docs/revitalize/probe/p17-0_base/validation.txt` |
| M | `docs/revitalize/verification/p17-0/core_tests.txt` |
| M | `docs/revitalize/verification/p17-0/fields_pins.txt` |
| M | `docs/revitalize/verification/p17-0/pins_debug.txt` |
| M | `docs/revitalize/verification/p17-0/pins_release.txt` |

<a id="commit-8de4073adbd152ecdf91ab28a1a348085e87a20f"></a>

## 8de4073adbd152ecdf91ab28a1a348085e87a20f

- 구분: Pass17 Part 1
- 제목 / 의도: revitalize(p17-1): give idle residents an observable daily routine
- 커밋 UTC: 2026-10-15T07:06:58Z
- 변경 경로: 69개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/8de4073adbd152ecdf91ab28a1a348085e87a20f)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/ROUTINE.md` |
| A | `docs/revitalize/verification/p17-1/bridge_test.txt` |
| A | `docs/revitalize/verification/p17-1/check.txt` |
| A | `docs/revitalize/verification/p17-1/clippy_final.txt` |
| A | `docs/revitalize/verification/p17-1/clippy_initial_failure.txt` |
| A | `docs/revitalize/verification/p17-1/core_batch_test.txt` |
| A | `docs/revitalize/verification/p17-1/core_clippy.txt` |
| A | `docs/revitalize/verification/p17-1/core_tests.txt` |
| A | `docs/revitalize/verification/p17-1/debug_journal.md` |
| A | `docs/revitalize/verification/p17-1/evaluator_test.txt` |
| A | `docs/revitalize/verification/p17-1/identity_comparator_test.txt` |
| A | `docs/revitalize/verification/p17-1/invitation_corrected_test.txt` |
| A | `docs/revitalize/verification/p17-1/invitation_initial_failure.txt` |
| A | `docs/revitalize/verification/p17-1/lsp.txt` |
| A | `docs/revitalize/verification/p17-1/perf_evaluator_test.txt` |
| A | `docs/revitalize/verification/p17-1/probe_test.txt` |
| A | `docs/revitalize/verification/p17-1/routine_test.txt` |
| A | `docs/revitalize/verification/p17-1/social_control_test.txt` |
| A | `rust/crates/sim-bridge/src/ffi/activity.rs` |
| M | `rust/crates/sim-bridge/src/ffi/mod.rs` |
| M | `rust/crates/sim-bridge/src/ffi/world_node.rs` |
| A | `rust/crates/sim-core/src/components/activity.rs` |
| M | `rust/crates/sim-core/src/components/agent_state.rs` |
| M | `rust/crates/sim-core/src/components/mod.rs` |
| M | `rust/crates/sim-core/src/rng.rs` |
| M | `rust/crates/sim-core/src/rules.rs` |
| M | `rust/crates/sim-core/src/rules/routine.rs` |
| M | `rust/crates/sim-systems/src/runtime/agent/movement.rs` |
| M | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` |
| M | `rust/crates/sim-systems/src/runtime/decision/mod.rs` |
| A | `rust/crates/sim-systems/src/runtime/decision/routine/lifecycle.rs` |
| A | `rust/crates/sim-systems/src/runtime/decision/routine/mod.rs` |
| A | `rust/crates/sim-systems/src/runtime/decision/routine/places.rs` |
| M | `rust/crates/sim-systems/src/worldgen/mod.rs` |
| M | `rust/crates/sim-test/src/perf.rs` |
| M | `rust/crates/sim-test/src/probe/agent_trace.rs` |
| M | `rust/crates/sim-test/src/probe/args.rs` |
| M | `rust/crates/sim-test/src/probe/deaths.rs` |
| M | `rust/crates/sim-test/src/probe/fingerprint.rs` |
| M | `rust/crates/sim-test/src/probe/mod.rs` |
| M | `rust/crates/sim-test/src/probe/pass12.rs` |
| A | `rust/crates/sim-test/src/probe/pass17.rs` |
| A | `rust/crates/sim-test/src/probe/pass17_activity.rs` |
| A | `rust/crates/sim-test/src/probe/pass17_batch_tests.rs` |
| A | `rust/crates/sim-test/src/probe/pass17_tests.rs` |
| M | `rust/crates/sim-test/src/probe/runner.rs` |
| M | `rust/crates/sim-test/src/scenes.rs` |
| M | `rust/crates/sim-test/tests/harness_p5_beta_decision.rs` |
| M | `rust/crates/sim-test/tests/harness_p5_gamma_sleep_daynight_chronicle.rs` |
| M | `rust/crates/sim-test/tests/harness_p6_alpha_construction_components.rs` |
| M | `rust/crates/sim-test/tests/harness_p7_alpha_social_components.rs` |
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_routine.rs` |
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_routine_edges.rs` |
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_routine_invitations.rs` |
| A | `rust/crates/sim-test/tests/pass17_routine_support/mod.rs` |
| A | `tools/revitalize/bridge_smoke_pass17.gd` |
| A | `tools/revitalize/evaluate_pass17.py` |
| A | `tools/revitalize/evaluate_pass17_data.py` |
| A | `tools/revitalize/evaluate_pass17_evidence.py` |
| A | `tools/revitalize/evaluate_pass17_read.py` |
| A | `tools/revitalize/evaluate_pass17_report.py` |
| A | `tools/revitalize/evaluate_pass17_stage_a.py` |
| A | `tools/revitalize/evaluate_pass17_statistics.py` |
| A | `tools/revitalize/evaluate_perf_pass17.py` |
| M | `tools/revitalize/perf_p17.sh` |
| A | `tools/revitalize/test_evaluate_pass17.py` |
| A | `tools/revitalize/test_evaluate_perf_pass17.py` |

<a id="commit-753b51942c20489538faac59c15c5ac45ab48075"></a>

## 753b51942c20489538faac59c15c5ac45ab48075

- 구분: Pass17 Part 1
- 제목 / 의도: revitalize(p17-1): prove routine observations before tuning
- 커밋 UTC: 2026-10-15T07:17:33Z
- 변경 경로: 19개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/753b51942c20489538faac59c15c5ac45ab48075)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/verification/p17-1/api_contract_audit.md` |
| A | `docs/revitalize/verification/p17-1/bridge_build.txt` |
| A | `docs/revitalize/verification/p17-1/bridge_smoke.txt` |
| A | `docs/revitalize/verification/p17-1/evaluator_final.txt` |
| A | `docs/revitalize/verification/p17-1/godot_import.txt` |
| A | `docs/revitalize/verification/p17-1/release_build.txt` |
| A | `docs/revitalize/verification/p17-1/release_pins.txt` |
| A | `docs/revitalize/verification/p17-1/smoke_binaries.sha256` |
| A | `docs/revitalize/verification/p17-1/smoke_raw.sha256` |
| A | `docs/revitalize/verification/p17-1/smoke_runlog.txt` |
| A | `docs/revitalize/verification/p17-1/smoke_runner_initial_failure.txt` |
| A | `docs/revitalize/verification/p17-1/smoke_source_commit.txt` |
| A | `docs/revitalize/verification/p17-1/trace_hand_recompute.md` |
| A | `docs/revitalize/verification/p17-1/v20_trace_identity.txt` |
| A | `docs/revitalize/verification/p17-1/v21_trace_identity.txt` |
| M | `tools/revitalize/evaluate_pass17_read.py` |
| M | `tools/revitalize/test_evaluate_pass17.py` |

<a id="commit-9a0bd85470f4ddbe26d51b8c5062107bf5c1f371"></a>

## 9a0bd85470f4ddbe26d51b8c5062107bf5c1f371

- 구분: Pass17 Part 2
- 제목 / 의도: revitalize(p17-2): freeze routine defaults after all tuning bars pass
- 커밋 UTC: 2026-10-15T07:23:29Z
- 변경 경로: 10개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/9a0bd85470f4ddbe26d51b8c5062107bf5c1f371)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/binaries.sha256` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/eval.json` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/eval.md` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/game_v20/summary.json` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/game_v21/summary.json` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/raw.sha256` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/runlog.txt` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/source_commit.txt` |
| A | `docs/revitalize/verification/p17-1/trace_qa.md` |

<a id="commit-6ad99666deab7acde6cf34d66de20867df639eaa"></a>

## 6ad99666deab7acde6cf34d66de20867df639eaa

- 구분: Pass17 Part 2
- 제목 / 의도: revitalize(p17-2): preserve original identity proof before observer correction
- 커밋 UTC: 2026-10-15T07:45:29Z
- 변경 경로: 25개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/6ad99666deab7acde6cf34d66de20867df639eaa)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/ROUTINE.md` |
| A | `docs/revitalize/probe/p17-2_a_identity/i1_v19/summary.json` |
| A | `docs/revitalize/probe/p17-2_a_identity/i2_sha256.txt` |
| A | `docs/revitalize/probe/p17-2_a_identity/i3_v16/summary.json` |
| A | `docs/revitalize/probe/p17-2_a_identity/i4_v20/summary.json` |
| A | `docs/revitalize/probe/p17-a_tuning/r1/manual_checks.md` |
| A | `docs/revitalize/verification/p17-2_a_identity/I1.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity/I1.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/I2.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity/I2.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/I3.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity/I3.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/I4.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity/I4.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/binaries.sha256` |
| A | `docs/revitalize/verification/p17-2_a_identity/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/identity.json` |
| A | `docs/revitalize/verification/p17-2_a_identity/source_commit.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/source_equivalence.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/v19_pins.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity/v19_pins.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity/v20_pins.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity/v20_pins.txt` |

<a id="commit-50a73474e06d3a9abbb535729e9c7e1e134954b7"></a>

## 50a73474e06d3a9abbb535729e9c7e1e134954b7

- 구분: Pass17 Part 2
- 제목 / 의도: revitalize(p17-2): retain routine transitions when death follows the same decision tick
- 커밋 UTC: 2026-10-15T07:54:19Z
- 변경 경로: 16개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/50a73474e06d3a9abbb535729e9c7e1e134954b7)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/ROUTINE.md` |
| A | `docs/revitalize/verification/p17-2_observer_fix/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-2_observer_fix/clippy.log` |
| A | `docs/revitalize/verification/p17-2_observer_fix/debug_journal.md` |
| A | `docs/revitalize/verification/p17-2_observer_fix/diff_check.log` |
| A | `docs/revitalize/verification/p17-2_observer_fix/green.log` |
| A | `docs/revitalize/verification/p17-2_observer_fix/provenance.log` |
| A | `docs/revitalize/verification/p17-2_observer_fix/red.log` |
| M | `rust/crates/sim-test/src/probe/agent_trace.rs` |
| M | `rust/crates/sim-test/src/probe/mod.rs` |
| M | `rust/crates/sim-test/src/probe/pass17.rs` |
| M | `rust/crates/sim-test/src/probe/pass17_activity.rs` |
| A | `rust/crates/sim-test/src/probe/pass17_death_tests.rs` |
| A | `rust/crates/sim-test/src/probe/routine_observer.rs` |
| M | `rust/crates/sim-test/src/probe/runner.rs` |

<a id="commit-7a6f74b4b0913ff27597de38678bf6f89f48e1a0"></a>

## 7a6f74b4b0913ff27597de38678bf6f89f48e1a0

- 구분: Pass17 Part 2
- 제목 / 의도: revitalize(p17-2): preserve corrected observation evidence before evaluation
- 커밋 UTC: 2026-10-15T08:08:56Z
- 변경 경로: 53개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/7a6f74b4b0913ff27597de38678bf6f89f48e1a0)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/ROUTINE.md` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/binaries.sha256` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/binary_directory.txt` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/game_v20_plain/summary.json` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/game_v20_trace/summary.json` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/game_v21_plain/summary.json` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/game_v21_trace/summary.json` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/raw_sha256.txt` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/runlog.txt` |
| A | `docs/revitalize/probe/p17-a_observerfix_smoke/source_commit.txt` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/binaries.sha256` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/binary_directory.txt` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/death_recovery_handcheck.md` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/eval.json` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/eval.md` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/game_v20/summary.json` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/game_v21/summary.json` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/raw_sha256.txt` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/runlog.txt` |
| A | `docs/revitalize/probe/p17-a_tuning/r1-observerfix/source_commit.txt` |
| A | `docs/revitalize/verification/p17-2_observer_fix/binaries.sha256` |
| A | `docs/revitalize/verification/p17-2_observer_fix/build_RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-2_observer_fix/pins_build.log` |
| A | `docs/revitalize/verification/p17-2_observer_fix/release_build.log` |
| A | `docs/revitalize/verification/p17-2_observer_fix/source_commit.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/binaries.sha256` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/binary_directory.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/game_v20_trace_identity.exit` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/game_v20_trace_identity.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/game_v21_trace_identity.exit` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/game_v21_trace_identity.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/raw_recompute_helper.sha256` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/raw_sha256.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/raw_trace_recompute.txt` |
| A | `docs/revitalize/verification/p17-a_observerfix_smoke/source_commit.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/binaries.sha256` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/binary_directory.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/counter_delta_audit.md` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/game_v20_old_r1_identity.exit` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/game_v20_old_r1_identity.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/game_v20_old_r1_inputs.sha256` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/game_v21_old_r1_identity.exit` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/game_v21_old_r1_identity.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/game_v21_old_r1_inputs.sha256` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/old_r1_binaries.sha256` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/old_r1_source_commit.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/raw_sha256.txt` |
| A | `docs/revitalize/verification/p17-a_tuning_r1_observerfix/source_commit.txt` |

<a id="commit-09eb5c2671d86334683b2273115635c34580dcea"></a>

## 09eb5c2671d86334683b2273115635c34580dcea

- 구분: Pass17 Part 2
- 제목 / 의도: revitalize(p17-2): retain final identity proof and measured timing regressions
- 커밋 UTC: 2026-10-15T08:31:55Z
- 변경 경로: 75개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/09eb5c2671d86334683b2273115635c34580dcea)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/PERF_BUDGET.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| M | `docs/revitalize/ROUTINE.md` |
| A | `docs/revitalize/perf/p17-a/perf.json` |
| A | `docs/revitalize/perf/p17-a/perf.md` |
| A | `docs/revitalize/perf/p17-a/raw/RUNLOG.txt` |
| A | `docs/revitalize/perf/p17-a/raw/a/r1/runs.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r1/stress_stress_v20_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r1/stress_stress_v20_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/a/r1/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r1/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/a/r2/runs.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r2/stress_stress_v20_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r2/stress_stress_v20_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/a/r2/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r2/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/a/r3/runs.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r3/stress_stress_v20_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r3/stress_stress_v20_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/a/r3/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/a/r3/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v20/runs.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v20/stress_stress_v20_s15100_n100_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v20/stress_stress_v20_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v21/runs.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/commands.jsonl` |
| A | `docs/revitalize/perf/p17-a/raw/provenance.json` |
| A | `docs/revitalize/perf/p17-a/raw/quick_base/quick_stress_v19_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/quick_base/quick_stress_v19_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/quick_base/runs.json` |
| A | `docs/revitalize/perf/p17-a/raw/quick_final/quick_stress_v19_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-a/raw/quick_final/quick_stress_v19_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-a/raw/quick_final/runs.json` |
| A | `docs/revitalize/perf/p17-a/raw/quiet.log` |
| A | `docs/revitalize/perf/p17-a/raw/runner.exit` |
| A | `docs/revitalize/perf/p17-a/raw/runs.log` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/binaries.sha256` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/binary_directory.txt` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/i1_v19/summary.json` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/i2_sha256.txt` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/i3_v16/summary.json` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/i4_v20/summary.json` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/raw_sha256.txt` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/runlog.txt` |
| A | `docs/revitalize/probe/p17-2_a_identity_observerfix/source_commit.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I1.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I1.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I2.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I2.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I3.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I3.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I4.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/I4.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/binaries.sha256` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/binary_directory.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/identity.json` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/raw_sha256.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/source_commit.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/v19_pins.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/v19_pins.txt` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/v20_pins.exit` |
| A | `docs/revitalize/verification/p17-2_a_identity_observerfix/v20_pins.txt` |

<a id="commit-000d3706c6463dd19cd6b1efce20ae60a7acaa9f"></a>

## 000d3706c6463dd19cd6b1efce20ae60a7acaa9f

- 구분: Pass17 Part 3
- 제목 / 의도: revitalize(p17-3): freeze held-out rules before observing evaluation seeds
- 커밋 UTC: 2026-10-15T08:35:22Z
- 변경 경로: 2개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/000d3706c6463dd19cd6b1efce20ae60a7acaa9f)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/METRICS.md` |

<a id="commit-44aebafa5acf5668d21418f611d8f0ce3a5d13d3"></a>

## 44aebafa5acf5668d21418f611d8f0ce3a5d13d3

- 구분: Pass17 Part 3
- 제목 / 의도: revitalize(p17-3): preserve held-out routine verdicts before family changes
- 커밋 UTC: 2026-10-15T09:20:39Z
- 변경 경로: 29개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/44aebafa5acf5668d21418f611d8f0ce3a5d13d3)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/METRICS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| M | `docs/revitalize/ROUTINE.md` |
| A | `docs/revitalize/perf/p17-a/interpretation.md` |
| A | `docs/revitalize/perf/p17-a/raw_copy_verification.txt` |
| A | `docs/revitalize/probe/p17-a_eval/binaries.sha256` |
| A | `docs/revitalize/probe/p17-a_eval/binary_directory.txt` |
| A | `docs/revitalize/probe/p17-a_eval/eval.json` |
| A | `docs/revitalize/probe/p17-a_eval/eval.md` |
| A | `docs/revitalize/probe/p17-a_eval/game_v20/summary.json` |
| A | `docs/revitalize/probe/p17-a_eval/game_v21/summary.json` |
| A | `docs/revitalize/probe/p17-a_eval/i5_v21/summary.json` |
| A | `docs/revitalize/probe/p17-a_eval/preregistration.json` |
| A | `docs/revitalize/probe/p17-a_eval/preregistration_remote_heads.txt` |
| A | `docs/revitalize/probe/p17-a_eval/raw_sha256.txt` |
| A | `docs/revitalize/probe/p17-a_eval/runlog.txt` |
| A | `docs/revitalize/probe/p17-a_eval/source_commit.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/binaries.sha256` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/binary_directory.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/completion_verified.json` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/preregistration.json` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/raw_sha256.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/raw_trace_recompute.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/raw_trace_recompute_helper.sha256` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/source_commit.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/stage_a_retained_binaries.json` |

<a id="commit-47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21"></a>

## 47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21

- 구분: Pass17 Part 4
- 제목 / 의도: revitalize(p17-4): let families preserve parentage and households
- 커밋 UTC: 2026-10-15T09:48:15Z
- 변경 경로: 69개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| A | `docs/revitalize/FAMILY.md` |
| A | `docs/revitalize/verification/p17-4/README.md` |
| A | `docs/revitalize/verification/p17-4/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-4/bridge_build.txt` |
| A | `docs/revitalize/verification/p17-4/core_clippy.txt` |
| A | `docs/revitalize/verification/p17-4/core_complete_tests.txt` |
| A | `docs/revitalize/verification/p17-4/family_tests_initial.txt` |
| A | `docs/revitalize/verification/p17-4/godot_family_smoke.txt` |
| A | `docs/revitalize/verification/p17-4/godot_import.txt` |
| A | `docs/revitalize/verification/p17-4/parent_id_audit.md` |
| A | `docs/revitalize/verification/p17-4/partial_receipt.json` |
| A | `docs/revitalize/verification/p17-4/probe_clippy_initial.txt` |
| A | `docs/revitalize/verification/p17-4/probe_tests_initial.txt` |
| A | `docs/revitalize/verification/p17-4/public_shape_audit.md` |
| A | `docs/revitalize/verification/p17-4/python_tests.txt` |
| A | `docs/revitalize/verification/p17-4/runtime_bridge_clippy_final.txt` |
| A | `docs/revitalize/verification/p17-4/runtime_bridge_clippy_initial.txt` |
| A | `docs/revitalize/verification/p17-4/runtime_bridge_clippy_retry.txt` |
| A | `docs/revitalize/verification/p17-4/runtime_bridge_unit_tests.txt` |
| A | `docs/revitalize/verification/p17-4/workspace_clippy.txt` |
| A | `docs/revitalize/verification/p17-a_eval_observerfix/cohort_audit.md` |
| A | `rust/crates/sim-bridge/src/ffi/family.rs` |
| M | `rust/crates/sim-bridge/src/ffi/mod.rs` |
| M | `rust/crates/sim-bridge/src/ffi/world_node.rs` |
| M | `rust/crates/sim-core/src/causal/event.rs` |
| A | `rust/crates/sim-core/src/components/family.rs` |
| A | `rust/crates/sim-core/src/components/family_events.rs` |
| M | `rust/crates/sim-core/src/components/mod.rs` |
| M | `rust/crates/sim-core/src/rng.rs` |
| M | `rust/crates/sim-core/src/rules.rs` |
| M | `rust/crates/sim-core/src/rules/routine.rs` |
| M | `rust/crates/sim-systems/src/lib.rs` |
| M | `rust/crates/sim-systems/src/runtime/agent/beds.rs` |
| A | `rust/crates/sim-systems/src/runtime/agent/households.rs` |
| M | `rust/crates/sim-systems/src/runtime/agent/mod.rs` |
| A | `rust/crates/sim-systems/src/runtime/agent/pairing.rs` |
| M | `rust/crates/sim-systems/src/runtime/settlement/fission.rs` |
| M | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` |
| M | `rust/crates/sim-systems/src/worldgen/camp.rs` |
| M | `rust/crates/sim-systems/src/worldgen/mod.rs` |
| M | `rust/crates/sim-test/src/perf.rs` |
| M | `rust/crates/sim-test/src/probe/agent_trace.rs` |
| M | `rust/crates/sim-test/src/probe/args.rs` |
| A | `rust/crates/sim-test/src/probe/family_events.rs` |
| A | `rust/crates/sim-test/src/probe/family_samples.rs` |
| A | `rust/crates/sim-test/src/probe/family_trace.rs` |
| A | `rust/crates/sim-test/src/probe/family_trace_tests.rs` |
| M | `rust/crates/sim-test/src/probe/mod.rs` |
| M | `rust/crates/sim-test/src/probe/runner.rs` |
| M | `rust/crates/sim-test/src/scenes.rs` |
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_family.rs` |
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_family_beds.rs` |
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_family_births.rs` |
| A | `rust/crates/sim-test/tests/harness_revitalize_pass17_family_fission.rs` |
| A | `rust/crates/sim-test/tests/pass17_family_support/mod.rs` |
| M | `tools/revitalize/bridge_smoke_pass17.gd` |
| M | `tools/revitalize/evaluate_pass17.py` |
| M | `tools/revitalize/evaluate_pass17_evidence.py` |
| A | `tools/revitalize/evaluate_pass17_family_data.py` |
| A | `tools/revitalize/evaluate_pass17_family_pairs.py` |
| A | `tools/revitalize/evaluate_pass17_family_read.py` |
| A | `tools/revitalize/evaluate_pass17_family_report.py` |
| A | `tools/revitalize/evaluate_pass17_family_rows.py` |
| M | `tools/revitalize/evaluate_pass17_report.py` |
| A | `tools/revitalize/evaluate_pass17_stage_b.py` |
| A | `tools/revitalize/test_evaluate_pass17_family_fixtures.py` |
| A | `tools/revitalize/test_evaluate_pass17_family_read.py` |
| A | `tools/revitalize/test_evaluate_pass17_stage_b.py` |

<a id="commit-7082258c3c6e5bcfe07f69558bd3ba82451c13e7"></a>

## 7082258c3c6e5bcfe07f69558bd3ba82451c13e7

- 구분: Pass17 Part 4
- 제목 / 의도: Bind family behavior claims to actual runtime traces
- 커밋 UTC: 2026-10-15T09:59:22Z
- 변경 경로: 46개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/7082258c3c6e5bcfe07f69558bd3ba82451c13e7)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/probe/p17-4_smoke/game_v21_plain/summary.json` |
| A | `docs/revitalize/probe/p17-4_smoke/game_v21_trace/summary.json` |
| A | `docs/revitalize/probe/p17-4_smoke/game_v22_day60/summary.json` |
| A | `docs/revitalize/probe/p17-4_smoke/game_v22_plain/summary.json` |
| A | `docs/revitalize/probe/p17-4_smoke/game_v22_trace/summary.json` |
| A | `docs/revitalize/verification/p17-4_smoke/README.md` |
| A | `docs/revitalize/verification/p17-4_smoke/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/binaries.sha256` |
| A | `docs/revitalize/verification/p17-4_smoke/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/build/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/build/binaries.sha256` |
| A | `docs/revitalize/verification/p17-4_smoke/build/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/build/bins.log` |
| A | `docs/revitalize/verification/p17-4_smoke/build/commands.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/build/launch_receipt.json` |
| A | `docs/revitalize/verification/p17-4_smoke/build/pins.log` |
| A | `docs/revitalize/verification/p17-4_smoke/build/postbuild_receipt.json` |
| A | `docs/revitalize/verification/p17-4_smoke/build/runner.exit` |
| A | `docs/revitalize/verification/p17-4_smoke/build/source_commit.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/completion_verified.json` |
| A | `docs/revitalize/verification/p17-4_smoke/family_recompute_day60.json` |
| A | `docs/revitalize/verification/p17-4_smoke/family_recompute_day60.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/family_recompute_short.json` |
| A | `docs/revitalize/verification/p17-4_smoke/family_recompute_short.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/family_trace_recompute.py` |
| A | `docs/revitalize/verification/p17-4_smoke/game_v21_plain.log` |
| A | `docs/revitalize/verification/p17-4_smoke/game_v21_trace.log` |
| A | `docs/revitalize/verification/p17-4_smoke/game_v21_trace_identity.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/game_v22_day60.log` |
| A | `docs/revitalize/verification/p17-4_smoke/game_v22_plain.log` |
| A | `docs/revitalize/verification/p17-4_smoke/game_v22_trace.log` |
| A | `docs/revitalize/verification/p17-4_smoke/game_v22_trace_identity.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/loader_verification.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/part4_actual_evaluator_qa.json` |
| A | `docs/revitalize/verification/p17-4_smoke/part4_actual_evaluator_qa.md` |
| A | `docs/revitalize/verification/p17-4_smoke/part4_smoke.sh` |
| A | `docs/revitalize/verification/p17-4_smoke/raw_sha256.json` |
| A | `docs/revitalize/verification/p17-4_smoke/routine_v21_recomputed.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/routine_v22_day60_recomputed.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/routine_v22_recomputed.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/runner.exit` |
| A | `docs/revitalize/verification/p17-4_smoke/source_commit.txt` |
| A | `docs/revitalize/verification/p17-4_smoke/trace_recompute_corrected.py` |
| A | `docs/revitalize/verification/p17-4_smoke/trace_recompute_stage_b.py` |

<a id="commit-c7ec0b5a53d46958ecb9562a9e2b38efe9a687ee"></a>

## c7ec0b5a53d46958ecb9562a9e2b38efe9a687ee

- 구분: Pass17 Part 5
- 제목 / 의도: revitalize(p17-5): freeze family rules without concealing split failures
- 커밋 UTC: 2026-10-15T10:08:25Z
- 변경 경로: 28개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/c7ec0b5a53d46958ecb9562a9e2b38efe9a687ee)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/FAMILY.md` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/binaries.sha256` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/binary_directory.txt` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/eval.json` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/eval.md` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/game_v21/summary.json` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/game_v22/summary.json` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/launch_receipt.json` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/raw_sha256.txt` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/runlog.txt` |
| A | `docs/revitalize/probe/p17-b_tuning/r1/source_commit.txt` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/binaries.sha256` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/binary_directory.txt` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/commands.txt` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/completion_verified.json` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/evaluate.exit` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/evaluate.log` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/final_binary_copy.json` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/game_v21.exit` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/game_v21.log` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/game_v22.exit` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/game_v22.log` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/launch_receipt.json` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/raw_sha256.txt` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/source_commit.txt` |

<a id="commit-f0410d398d72045b69e621d0d244395d85286a21"></a>

## f0410d398d72045b69e621d0d244395d85286a21

- 구분: Pass17 Part 5
- 제목 / 의도: revitalize(p17-5): explain family splits without weakening their bar
- 커밋 UTC: 2026-10-15T10:17:55Z
- 변경 경로: 10개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/f0410d398d72045b69e621d0d244395d85286a21)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/README.md` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/b_tuning_family_audit.exit` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/b_tuning_family_audit.json` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/b_tuning_family_audit.log` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/b_tuning_family_audit.md` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/b_tuning_family_audit.sha256` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/b_tuning_fission_diagnostic.json` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/b_tuning_fission_diagnostic.md` |
| A | `docs/revitalize/verification/p17-b_tuning_r1/family_trace_audit_runner.py` |

<a id="commit-d2cde5e9d50d2650ec5f9b2745b636116295a754"></a>

## d2cde5e9d50d2650ec5f9b2745b636116295a754

- 구분: Pass17 Part 5
- 제목 / 의도: revitalize(p17-5): bind family evaluation to complete identity and timing proofs
- 커밋 UTC: 2026-10-15T10:45:43Z
- 변경 경로: 85개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/d2cde5e9d50d2650ec5f9b2745b636116295a754)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/perf/p17-b/launch_receipt.json` |
| A | `docs/revitalize/perf/p17-b/perf.json` |
| A | `docs/revitalize/perf/p17-b/perf.md` |
| A | `docs/revitalize/perf/p17-b/raw/RUNLOG.txt` |
| A | `docs/revitalize/perf/p17-b/raw/a/r1/runs.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r1/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r1/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/a/r1/stress_stress_v22_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r1/stress_stress_v22_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/a/r2/runs.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r2/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r2/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/a/r2/stress_stress_v22_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r2/stress_stress_v22_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/a/r3/runs.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r3/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r3/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/a/r3/stress_stress_v22_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/a/r3/stress_stress_v22_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v21/runs.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v22/runs.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v22/stress_stress_v22_s15100_n100_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/b_game_v22/stress_stress_v22_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/commands.jsonl` |
| A | `docs/revitalize/perf/p17-b/raw/launch_receipt.json` |
| A | `docs/revitalize/perf/p17-b/raw/provenance.json` |
| A | `docs/revitalize/perf/p17-b/raw/quick_base/quick_stress_v19_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/quick_base/quick_stress_v19_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/quick_base/runs.json` |
| A | `docs/revitalize/perf/p17-b/raw/quick_final/quick_stress_v19_s15100_n500_side64_fed.json` |
| A | `docs/revitalize/perf/p17-b/raw/quick_final/quick_stress_v19_s15100_n500_side64_fed.tick_us.bin` |
| A | `docs/revitalize/perf/p17-b/raw/quick_final/runs.json` |
| A | `docs/revitalize/perf/p17-b/raw/quiet.log` |
| A | `docs/revitalize/perf/p17-b/raw/runner.exit` |
| A | `docs/revitalize/perf/p17-b/raw/runs.log` |
| A | `docs/revitalize/perf/p17-b/raw_copy_verification.txt` |
| A | `docs/revitalize/probe/p17-5_b_identity/binaries.sha256` |
| A | `docs/revitalize/probe/p17-5_b_identity/binary_directory.txt` |
| A | `docs/revitalize/probe/p17-5_b_identity/i1_v19/summary.json` |
| A | `docs/revitalize/probe/p17-5_b_identity/i2_sha256.txt` |
| A | `docs/revitalize/probe/p17-5_b_identity/i3_v16/summary.json` |
| A | `docs/revitalize/probe/p17-5_b_identity/i4_v20/summary.json` |
| A | `docs/revitalize/probe/p17-5_b_identity/i5_v21/summary.json` |
| A | `docs/revitalize/probe/p17-5_b_identity/launch_receipt.json` |
| A | `docs/revitalize/probe/p17-5_b_identity/raw_sha256.txt` |
| A | `docs/revitalize/probe/p17-5_b_identity/runlog.txt` |
| A | `docs/revitalize/probe/p17-5_b_identity/source_commit.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/I1.exit` |
| A | `docs/revitalize/verification/p17-5_b_identity/I1.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/I2.exit` |
| A | `docs/revitalize/verification/p17-5_b_identity/I2.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/I3.exit` |
| A | `docs/revitalize/verification/p17-5_b_identity/I3.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/I4.exit` |
| A | `docs/revitalize/verification/p17-5_b_identity/I4.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/I5.exit` |
| A | `docs/revitalize/verification/p17-5_b_identity/I5.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/README.md` |
| A | `docs/revitalize/verification/p17-5_b_identity/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/binaries.sha256` |
| A | `docs/revitalize/verification/p17-5_b_identity/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/binary_directory.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/completion_verified.json` |
| A | `docs/revitalize/verification/p17-5_b_identity/identity.json` |
| A | `docs/revitalize/verification/p17-5_b_identity/launch_receipt.json` |
| A | `docs/revitalize/verification/p17-5_b_identity/partial_audit.json` |
| A | `docs/revitalize/verification/p17-5_b_identity/partial_audit.md` |
| A | `docs/revitalize/verification/p17-5_b_identity/raw_sha256.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/source_commit.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/v19_pins.exit` |
| A | `docs/revitalize/verification/p17-5_b_identity/v19_pins.txt` |
| A | `docs/revitalize/verification/p17-5_b_identity/v20_pins.exit` |
| A | `docs/revitalize/verification/p17-5_b_identity/v20_pins.txt` |

<a id="commit-43f22e81f120a755de33712b02a62469c67ab840"></a>

## 43f22e81f120a755de33712b02a62469c67ab840

- 구분: Pass17 Part 6
- 제목 / 의도: revitalize(p17-6): freeze family criteria before opening reserved worlds
- 커밋 UTC: 2026-10-15T10:46:29Z
- 변경 경로: 3개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/43f22e81f120a755de33712b02a62469c67ab840)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/METRICS.md` |
| A | `docs/revitalize/verification/p17-b_prereg/evaluator_sources.json` |

<a id="commit-32e2a009966d6d429a62282897241510400dbc45"></a>

## 32e2a009966d6d429a62282897241510400dbc45

- 구분: Pass17 Part 6
- 제목 / 의도: revitalize(p17-6): preserve the F3 discrepancy before any held-out analysis result
- 커밋 UTC: 2026-10-15T10:55:59Z
- 변경 경로: 7개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/32e2a009966d6d429a62282897241510400dbc45)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/perf/p17-b/independent_audit.json` |
| A | `docs/revitalize/perf/p17-b/independent_audit.md` |
| A | `docs/revitalize/verification/p17-5_b_identity/final_audit.json` |
| A | `docs/revitalize/verification/p17-5_b_identity/final_audit.md` |
| A | `docs/revitalize/verification/p17-b_prereg/f3_review_notice.md` |

<a id="commit-9a06b11167984116071c444e3540479f74059539"></a>

## 9a06b11167984116071c444e3540479f74059539

- 구분: Pass17 Part 6
- 제목 / 의도: revitalize(p17-6): bind the sibling-population correction before held-out analysis
- 커밋 UTC: 2026-10-15T11:08:46Z
- 변경 경로: 2개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/9a06b11167984116071c444e3540479f74059539)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/verification/p17-b_prereg/f3_review_notice.md` |

<a id="commit-e353f18f5b523b4198fd785b44c5ad962b15576d"></a>

## e353f18f5b523b4198fd785b44c5ad962b15576d

- 구분: Pass17 Part 6
- 제목 / 의도: revitalize(p17-6): preserve the held-out failure before correcting F3 populations
- 커밋 UTC: 2026-10-15T11:22:21Z
- 변경 경로: 35개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/e353f18f5b523b4198fd785b44c5ad962b15576d)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/METRICS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| A | `docs/revitalize/probe/p17-b_eval/binaries.sha256` |
| A | `docs/revitalize/probe/p17-b_eval/binary_directory.txt` |
| A | `docs/revitalize/probe/p17-b_eval/eval.json` |
| A | `docs/revitalize/probe/p17-b_eval/eval.md` |
| A | `docs/revitalize/probe/p17-b_eval/game_v21/summary.json` |
| A | `docs/revitalize/probe/p17-b_eval/game_v22/summary.json` |
| A | `docs/revitalize/probe/p17-b_eval/launch_receipt.json` |
| A | `docs/revitalize/probe/p17-b_eval/original_registered/README.md` |
| A | `docs/revitalize/probe/p17-b_eval/original_registered/archive_receipt.json` |
| A | `docs/revitalize/probe/p17-b_eval/original_registered/eval.json` |
| A | `docs/revitalize/probe/p17-b_eval/original_registered/eval.md` |
| A | `docs/revitalize/probe/p17-b_eval/original_registered/launch_receipt.json` |
| A | `docs/revitalize/probe/p17-b_eval/original_registered/preregistration.json` |
| A | `docs/revitalize/probe/p17-b_eval/preregistration.json` |
| A | `docs/revitalize/probe/p17-b_eval/raw_sha256.txt` |
| A | `docs/revitalize/probe/p17-b_eval/runlog.txt` |
| A | `docs/revitalize/probe/p17-b_eval/source_commit.txt` |
| A | `docs/revitalize/verification/p17-b_eval/RUNLOG.txt` |
| A | `docs/revitalize/verification/p17-b_eval/binaries.sha256` |
| A | `docs/revitalize/verification/p17-b_eval/binaries_verified.txt` |
| A | `docs/revitalize/verification/p17-b_eval/binary_directory.txt` |
| A | `docs/revitalize/verification/p17-b_eval/fission_diagnostic.json` |
| A | `docs/revitalize/verification/p17-b_eval/fission_diagnostic.md` |
| A | `docs/revitalize/verification/p17-b_eval/fission_diagnostic_root_verified.json` |
| A | `docs/revitalize/verification/p17-b_eval/launch_receipt.json` |
| A | `docs/revitalize/verification/p17-b_eval/original_completion_verified.json` |
| A | `docs/revitalize/verification/p17-b_eval/preregistration.json` |
| A | `docs/revitalize/verification/p17-b_eval/raw_sha256.txt` |
| A | `docs/revitalize/verification/p17-b_eval/routine_recompute.txt` |
| A | `docs/revitalize/verification/p17-b_eval/source_commit.txt` |
| A | `docs/revitalize/verification/p17-b_eval/trace_recompute.py` |
| A | `docs/revitalize/verification/p17-b_eval/verify_original_completion.py` |

<a id="commit-51e1629a059ada0aa6ea9c8434b348d7587cc9aa"></a>

## 51e1629a059ada0aa6ea9c8434b348d7587cc9aa

- 구분: Pass17 Part 6
- 제목 / 의도: revitalize(p17-6): measure the full sibling population required by the brief
- 커밋 UTC: 2026-10-15T11:23:48Z
- 변경 경로: 16개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/51e1629a059ada0aa6ea9c8434b348d7587cc9aa)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/FAMILY.md` |
| A | `docs/revitalize/verification/p17-f3_correction/README.md` |
| A | `docs/revitalize/verification/p17-f3_correction/report_helper_fixtures.exit` |
| A | `docs/revitalize/verification/p17-f3_correction/report_helper_fixtures.log` |
| A | `docs/revitalize/verification/p17-f3_correction/source_check.json` |
| A | `docs/revitalize/verification/p17-f3_correction/tests_green.exit` |
| A | `docs/revitalize/verification/p17-f3_correction/tests_green.log` |
| A | `docs/revitalize/verification/p17-f3_correction/tests_pass17.exit` |
| A | `docs/revitalize/verification/p17-f3_correction/tests_pass17.log` |
| A | `docs/revitalize/verification/p17-f3_correction/tests_red.exit` |
| A | `docs/revitalize/verification/p17-f3_correction/tests_red.log` |
| M | `tools/revitalize/evaluate_pass17_family_data.py` |
| M | `tools/revitalize/evaluate_pass17_family_pairs.py` |
| M | `tools/revitalize/evaluate_pass17_family_report.py` |
| M | `tools/revitalize/test_evaluate_pass17_stage_b.py` |

<a id="commit-21d2b478a81d55ac138f0a60b830e06719ec0af3"></a>

## 21d2b478a81d55ac138f0a60b830e06719ec0af3

- 구분: Pass17 Part 6
- 제목 / 의도: revitalize(p17-6): preserve literal family estimates without hiding failed bars
- 커밋 UTC: 2026-10-15T11:33:03Z
- 변경 경로: 41개
- [GitHub 커밋](https://github.com/hyunlord/new-world/commit/21d2b478a81d55ac138f0a60b830e06719ec0af3)

| 상태 | 전체 경로 |
|---|---|
| M | `docs/revitalize/DECISIONS.md` |
| M | `docs/revitalize/METRICS.md` |
| M | `docs/revitalize/PLAN_PASS17.md` |
| M | `docs/revitalize/probe/p17-b_eval/eval.json` |
| M | `docs/revitalize/probe/p17-b_eval/eval.md` |
| A | `docs/revitalize/probe/p17-b_tuning/r1-f3-corrected/eval.json` |
| A | `docs/revitalize/probe/p17-b_tuning/r1-f3-corrected/eval.md` |
| A | `docs/revitalize/verification/p17-b_eval/family_audit.exit` |
| A | `docs/revitalize/verification/p17-b_eval/family_audit.json` |
| A | `docs/revitalize/verification/p17-b_eval/family_audit.log` |
| A | `docs/revitalize/verification/p17-b_eval/family_audit.md` |
| A | `docs/revitalize/verification/p17-b_eval/family_audit.sha256` |
| A | `docs/revitalize/verification/p17-b_eval/family_audit_root_verified.json` |
| A | `docs/revitalize/verification/p17-f3_correction/artifact_hashes.json` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_comparison.json` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_independent_f3.exit` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_independent_f3.json` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_independent_f3.log` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_independent_f3.md` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_independent_f3.sha256` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_replay.json` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluation_replay.log` |
| A | `docs/revitalize/verification/p17-f3_correction/evaluator_sources.json` |
| A | `docs/revitalize/verification/p17-f3_correction/f3_literal_audit.py` |
| A | `docs/revitalize/verification/p17-f3_correction/f3_literal_audit_usage.md` |
| A | `docs/revitalize/verification/p17-f3_correction/f3_population_correction.patch` |
| A | `docs/revitalize/verification/p17-f3_correction/f3_population_correction_manifest.json` |
| A | `docs/revitalize/verification/p17-f3_correction/f3_population_correction_strategy.md` |
| A | `docs/revitalize/verification/p17-f3_correction/independent_f3_root_verified.json` |
| A | `docs/revitalize/verification/p17-f3_correction/run_f3_replay.py` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_comparison.json` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_independent_f3.exit` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_independent_f3.json` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_independent_f3.log` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_independent_f3.md` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_independent_f3.sha256` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_replay.json` |
| A | `docs/revitalize/verification/p17-f3_correction/tuning_replay.log` |
| A | `docs/revitalize/verification/p17-f3_correction/verify_f3_replay.py` |
| A | `docs/revitalize/verification/p17-review/round1.json` |
| A | `docs/revitalize/verification/p17-review/round1.md` |

