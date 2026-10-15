# Pass 17 fresh independent review — round 2

Source reviewed: `51e1629a059ada0aa6ea9c8434b348d7587cc9aa`, against `adfdecfece79ab019964729319389fd602685fab`.
Runtime producer: `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21`; `git diff 47b72440 51e1629a -- rust` is empty.
Supplied source diff SHA256: `0343dafcc7bb9486a096bfb40e6fda584f24ae12f8f1bafbfc7176d1987f203a` (independently verified).
Authority: complete `docs/revitalize/PASS17_GOAL.md`, especially §8.2 and literal §7.7.

Status: **no new actionable source defect; C technical gate PASS at `21d2b478a81d55ac138f0a60b830e06719ec0af3`; R publication gate PENDING**. Stage A and Stage B measured acceptance remain **FAIL**, for R5 and F1 respectively. The technical gate does not erase either result.

Scope: read-only source, supplied diff, terminal outputs and independent arithmetic over retained raw traces. No other review report, parent reasoning, or parent conversation was used; no agents were spawned. No build, test, probe, Godot, timing workload, checkout, reset or repository edit was performed. Only these external review files are written. Test results below describe inspected terminal evidence, not newly executed tests.

## Findings and limits

1. **R2-01 — measured acceptance failure, high importance, no new code defect.** Stage A R5 fails the original relative budgets: mean +33.0401455885% (limit +15%), p99.9 +339.863713799% (limit +35%). Absolute mean 0.4698833333 ms and decision slope 1.0013160279 pass their separate limits. Evidence: `docs/revitalize/perf/p17-a/perf.json`, `docs/revitalize/probe/p17-a_eval/eval.json`. Preserve the FAIL; do not replace it with the absolute-budget success.
2. **R2-02 — measured acceptance failure, medium importance, no new code defect.** Held-out Stage B F1 is 527/1064 = 0.49530075187969924, below 0.50. The interval crossing 0.50 does not change the point-estimate rule. Independent raw day-60 recount confirms it. Evidence: `docs/revitalize/probe/p17-b_eval/eval.json`; raw `b-evaluation/game_v22/*_family_meta.json`. No evaluation-seed retuning is authorized by the brief.
3. **R2-03 — disclosed methodological deviation, medium importance.** The original Stage B preregistration added sibling co-residence and control life-stage-category filters absent from literal §7.7 (`METRICS.md:3066–3068`). The source at `tools/revitalize/evaluate_pass17_family_pairs.py:22` now removes both restrictions correctly. This is a post-launch correction, not an unchanged preregistered analysis. Original result is preserved byte-for-byte in `original_registered/eval.json` at commit `e353f18f`; corrected source/input/output receipts identify the replay. Both original and corrected F3 pass and both overall Stage B results fail F1. The disclosure must remain in the final report.
4. **R2-04 — C verification resolved; R publication verification pending.** The complete §4.5 C gate is independently verified PASS from terminal raw evidence at `21d2b478a81d55ac138f0a60b830e06719ec0af3`, within this same round. The subsequent R docs-only commit requires a separate full external exact-head gate. No R publication readiness is claimed.

## §8.2 checklist

| Criterion | Result | Concrete evidence |
|---|---|---|
| I1/I3/I4/I5 identity | Verified from terminal artifacts | Independently compared all four pairs of 32-world summaries: exact equality outside timing, seeds 0–31. Each has only 74 timing differences. All 73 identity receipt references hash-match. `verification/p17-5_b_identity/identity.json`. |
| I2 recorder identity | Verified | Base/final `i2_sha256.txt` inventories are byte-identical: 20 entries, including episode array files. Recorder source diff is empty. |
| v19/v20 pins | Verified terminal evidence; C gate also passed | Both retained pin logs have 3 passed, 0 failed, exit 0. Pass16 frozen file unchanged. v20 pin commit `4f10de75` contains only the new pin file, before any Rust changes from the base; its later diff adds only `routine == None` and `family == None` assertions. Seeds 0–4, 1,440/40,320-tick checks are retained. |
| Existing public fields | Verified source | Comparing existing public struct declarations finds only `AgentRules.routine` and `SocietyRules.family` added (`rules.rs:60,178`). `AgentSnapshotRow`, `AgentDetailRow`, Tallies and FissionRecord shapes are unchanged; detail dictionary remains exactly nine keys. |
| Art compatibility/exhaustive matches | Verified source and prior terminal compile evidence | `COMPAT_PASS17.md` records the pre-Part-1 art Git-object inventory at `46193a03`. Explicit Activity handling exists in decision current-need/resource/consume branches, post-pass, bridge snapshot/detail, fingerprint, deaths and pass12. Legacy exhaustive test helpers add Activity explicitly. No wildcard replaces the target enumeration. |
| Activity entry position | Verified source | `agent_decision.rs:1866–1938`: Gather, then `!no_trips && routine.can_enter`, then settlement migration. `can_enter` requires membership, lineage-key stagger, display clock and no rest. |
| Idle-equivalence | Verified source | Builder candidates and strict/bias/all-idle peer maps use guarded `idle_equivalent`. Exits are evaluated before the state match with the same need-availability, forage/firewood/gather tile selectors and meal predicate. `invite`/ordered replay handles already-processed peers in the same tick. Focused invitation/edge tests cover both AgentId orders, bias-selected nonlonely peers, unavailable work, builders and same-tick observations. |
| Activity lifecycle/determinism | Verified source | Stale Activity removal precedes occupancy. Pending structural changes are sorted by AgentId. Occupancy updates within the tick, one redraw, 30-minute no-place rest, arrival-based duration, timeout, day end, membership exit, and one-tick interruption guard are present in `routine/{mod,lifecycle,places}.rs`. Play has its consuming movement case. |
| No tally field/economy mutation | Verified source | Activity consumption branch only manages lifecycle; counts live in new observer/trace types. No Tallies or FissionRecord field change. |
| Birth gates unchanged | Verified source | The diff in `settlement_system.rs` begins in parent selection after births have already passed the existing gates. Family None preserves the original sorted two-lowest-parent path; qualified mother selection checks both living partners, then updates her last-birth tick and lineage. `pairing.rs:154` plus focused birth gate/dead-partner tests. |
| Pairing/household implementation | Verified source | Pairing priority 144 is registered after Maturation and before beds; None returns before work. Sorted settlement/female/key ordering and kin exclusion. Beds first-fit complete households; oversized groups span shelters. Fission seeds/fills whole nonelder households with bounded individual top-up and trace-only split counts. |
| Manual person arithmetic | Independently verified | Seed17000/person7: interval lengths reproduce all seven lifetime/adult minute totals, 72 lifetime completions/5,642 minutes, and 31 adult completions/2,336 minutes. Detailed example below. |
| Manual family arithmetic | Independently verified | Seed18000/mother11/father9: pair event, five births, child membership and eight of eight shared bed nights reconstructed below. Full cohort F1/F2/F4/F5 also independently recounted. |
| Preregistration/stage ordering | Verified with disclosed F3 deviation | Stage A registration `000d3706` at 08:35:22Z precedes first launch 08:35:38.982376Z; A verdict commit `44aebafa` at 09:20:39Z precedes B Rust commit `47b72440` at 09:48:15Z. B registration `43f22e81` at 10:46:29Z precedes launch 10:46:45.675898Z. Later F3 correction is explicitly post-launch. |
| Seeds/tuning | Verified artifacts/source | A 17000–17031 evaluation versus 17100–17115 tuning; B 18000–18031 evaluation versus 18100–18115 tuning. Changed Rust test sources contain no reserved-range numeric references. Two Python assertions check the allowed seed manifests (18000–18031 and18100–18115) without simulating those seeds. Correction replays retained raw data without another simulation or parameter-tuning round. |
| Verdict arithmetic | Verified source and independent B arithmetic | Original bars and world bootstrap remain intact. F3 >=0.06 and positive lower CI; other point-estimate rules retain their original tests. Stage A R1/R2/R3/R4 pass, R5 fails; B F2–F8 pass, F1 fails. |
| Allowed paths | Verified source | All 587 changed paths from base to source head fall within rust/, tools/revitalize/, docs/revitalize/; no recorder subtree or prohibited directory changes. Existing local ignore edits are outside the committed diff and untouched. |
| Quiet lock | Verified retained records/source | A hold 08:20:08–08:20:23Z, 15.0s, load3.66, uncontended; B hold 10:43:58–10:44:15Z, 16.8s, load3.75, uncontended. More than 10 minutes apart and under 60-minute limit. Command ledgers record owner-allowed checks; helper uses atomic mkdir, owner/pid/purpose/start, stale handling and per-step foreign-lock checks. No new timing performed by this reviewer. |
| Final §4.5 gate | **C PASS; R PENDING** | C terminal evidence independently verified: exact 15 steps exit 0, source bound, all artifact hashes match. Separate R external exact-head gate remains required. |

## Independent trace arithmetic

### One person

Raw files: `/Users/rexxa/work/p17-data/a-evaluation-observerfix/game_v21/game_v21_s17000_{routine,people,agents}.jsonl`.
Person7 temperament P=504 gives configured duration `90 + trunc(60*(504-500)/500) = 90` minutes. For the HearthCircle chosen at 338, arrival 448, until 538, interruption 481, consuming minutes are `481-448=33`; walking 110 minutes are excluded and completion=false. For TendFire chosen 1866, arrival 1952, until 2042, day-end exit 2040, `2040-1952=88` minutes are completed because exit(6) includes day end.

Sum every emitted consuming interval's `end_tick-start_tick`, separately by recorded life stage:

| Kind | Lifetime | Adult |
|---|---:|---:|
| TendFire | 818 | 818 |
| Craft | 615 | 615 |
| Repair | 441 | 441 |
| HearthCircle | 2136 | 762 |
| Explore | 359 | 359 |
| Play | 0 | 0 |
| SitByFire | 2476 | 0 |

These equal the retained people/agents rows exactly. Lifetime starts `[27,17,15,50,11,0,35]`; 72 completed activities total 5642 minutes. Adult completed 31/minutes 2336 yield R3-P input `2336/31=75.35483870967742`; adult activity total 2995 yields Explore share `359/2995=0.11986644407345576` and HearthCircle share `762/2995=0.254424040066778`. Adult exposure 30840 ticks =21.4166666667 days qualifies for the ten-day cohort.

### One family

Raw files: `/Users/rexxa/work/p17-data/b-evaluation/game_v22/game_v22_s18000_family_{events,hours,people}.jsonl`.
Mother11/father9 pair mutually at tick 7920, elapsed day 6, display 20:00. Their nonfallback children are 32 at 8640,33 at9360,35 at10800,37 at12240,40 at14400. At tick 9360, both parents and child-stage32/33 have household_key 9 and bed `[33,28]`. Across the eight 20:00 samples from 7920 through 18000 where this living mutual couple appears, both are bedded and share the shelter: F4 contribution 8/8. All five listed births precede day 20 and therefore contribute 0/0 to the post-day 20 F2 exposure; they remain in the all-birth table and ancestry. Children 32 and 33 have matching known ordered parents and therefore form a full-sibling pair regardless of later residence.

### Entire Stage B cohort

An independent inline reader, without importing the evaluator, reconstructed the pair sets from lifetime parents and the union of non-null settlement memberships, then recomputed all four-axis integer moments and 2,000 whole-world bootstrap samples with fresh `default_rng(0)`.

| Metric | Independent raw result | Canonical match |
|---|---:|---|
| F1 | 527/1064 =0.49530075187969924 | Exact |
| F2 | 2969/3070 =0.967100977198697 | Exact |
| F4 | 38875/39024 =0.9961818368183681 | Exact |
| F5 | 18/234 =0.07692307692307693 | Exact |
| F3 sibling unordered pairs | 7054 | All 32 pair-ID sets exact |
| F3 unrelated unordered pairs | 107049 | All 32 pair-ID sets exact |
| F3 difference | 0.10492328730233096 | Exact |
| F3 95% CI | [0.08279428448915345,0.1260843651055091] | Lower differs only at floating-roundoff1e-17 |

Sibling coefficients in NS/HA/RD/P order: `[0.13528580157833128,0.11063095103956115,0.12387966738925653,0.10346482218515539]`; unrelated coefficients: `[0.022791281822751766,0.0010408720190552452,0.020276839497900972,0.009459099643272576]`.
There are 271 sibling pairs without any shared hourly non-null-settlement sample; these remain included with null reference context, as the literal brief requires. Examples: seed18000 pairs46/52,46/56,46/62; seed18002 pairs38/41,38/44. Control construction does not depend on sibling stage categories.

## Corrected-analysis provenance

- Original evaluation JSON SHA256 `80ab9b5c24c4da35df65547fba0e4e17bf68f42a949f1ca12ab396ba45f7a43c`; current bytes equal commit `e353f18f` exactly.
- Corrected evaluation JSON SHA256 `a671f614e22d78fbc187494c38eb7f0b4ba92f51bf8e9358e4f89886709a0945`; terminal 11:25:09.134950Z; evaluator exit 1 reflects F1 FAIL, not missing evidence.
- Corrected tuning JSON SHA256 `5599d9248ffed652ebd86bb994161b346c90682740e643aff88caac51a857786`; original tuning retained separately.
- Rehashed all 706 corrected evaluation receipt entries (690 raw inputs,14 source files,2 outputs) and 369 tuning entries (353 inputs,14 source files,2 outputs): zero mismatches.
- Inspected preserved regression evidence: 9 expected RED failures, 9 GREEN tests, 45 Pass17 evaluator tests, and report helper fixtures. No reviewer test rerun was performed.

## Final gate supplement

This is a terminal-evidence supplement to round 2, not a third review. **The complete C technical gate is PASS at `21d2b478a81d55ac138f0a60b830e06719ec0af3`. The subsequent R documentation/publication commit still requires its own full external exact-head gate and remains PENDING.** C's result does not establish R's result.

Source binding: independently compared C Rust to runtime producer `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21` and C `tools/revitalize` to reviewed source `51e1629a059ada0aa6ea9c8434b348d7587cc9aa`: both diffs are empty. The external wrapper's before/after snapshots both name C, with no staged, unstaged, untracked or discoverable ignored source under its explicit `rust/` and `tools/` scope. Those snapshots exclude `rust/target/`, `__pycache__`, and ignored files without a listed source suffix; they are endpoint observations, not continuous monitoring or a whole-repository clean-tree claim. Documentation staging after C is outside that scope. The gate script's before/after hash matches its committed C blob: `81f30f2e0b70c53ea745628886c1379273810493f200d573321530ae30ba5b65` (4453 bytes).

Provenance: independently rehashed all 18 receipt artifacts and checked their byte sizes, then verified all 19 manifest entries (the 18 artifacts plus receipt): zero mismatches. Receipt SHA256 is `15006bc94791f886cb199ac1c6cfe5b047fe188ce7021602c582e52668e042cb`; manifest SHA256 is `117e9e28cda2a27ee83cc1294ac2cf6c0e6c4300b061ee40c85733d2a09a9dce`. The receipt records gate exit 0, no wrapper signal, no validation errors, and terminal timestamps. The wrapper spans 11:33:37.996124–12:38:03.459129Z (3865.463005 seconds); the child gate spans 11:33:38.913896–12:38:02.001723Z (3863.087827 seconds). `gate.txt` independently shows C and terminal status 0 at 21:38:01 KST.

Raw `gate.txt` has exactly these 15 ordered step exits, each zero: `prepare_bridge`, `prepare_import`, `test_debug`, `test_release`, `clippy`, `clippy_all_targets`, `forbidden_math`, `bridge_build`, `godot_import`, `strict_check`, `smoke`, `python_unittest`, `python_worldmodel`, `perf_quick`, `allowed_paths`.

| Raw evidence | Independently verified result |
|---|---|
| `test_debug.log` | 159 result blocks, every block `ok`; 2241 passed, 0 failed, 0 ignored. |
| `test_release.log` | 159 result blocks, every block `ok`; 2241 passed, 0 failed, 0 ignored. |
| `python_unittest.log` | Ran 246, executed 246, skipped 0, OK. |
| `python_worldmodel.log` | Ran 49, executed 25, skipped 24, OK (skipped=24). Skipped checks are not counted as passed. |
| `strict_check.log` | 17 GDScript files clean; 14 FFI calls match. |
| `smoke.log` / `smoke_errors.log` | Headless renderer/camera ready; errors file is empty (0 bytes). This is not human visual acceptance. |
| `allowed_paths.log` | Exact match to independently obtained base-to-C list: 623 paths, all in allowed roots, no recorder subtree. The earlier 587-path count belongs to source51, not C. |

The 24 worldmodel skips follow the committed gate's explicit `WM_BIN=/nonexistent PROBE_BIN=/nonexistent`. Static inspection of class decorators and test methods accounts for all 24: `ExportDeterminismTests` 9, `FamilyTests` 3 and `SchemaThreeTests` 11 are skipped for `recorder not built: /nonexistent`; `HiddenColumnTests` 1 is skipped for `binaries not built: /nonexistent, /nonexistent`. The 25 executed checks are the offline Metric (8), Schedule (3), BenchV1 (7) and Loader (7) tests. The raw nonverbose log gives the aggregate skip count; the class-specific reasons are established by the source and gate environment, not a verbose runtime reason log.

Quiet-lock absence proof is bounded: `p17_gate/RUNLOG.txt` does not exist, `launcher.log` is empty, and no `wait_failed` row exists. The committed gate calls `wait_mac_p17.sh` before the gate, each step, and the quick-performance executable. Inspection shows that the helper creates/appends RUNLOG only on owned-lock permission, a foreign-lock wait, or stale-lock removal. Thus no such event was recorded by these guards; this does not prove continuous lock absence or an idle Mac. C holds no timing lock and its quick-performance smoke is explicitly untimed validation. Its mean/p99.9/max (0.3805746377/14.573/23.369 ms) do not replace the separately retained R5/F8 measurements.

No new source defect was found. R2-04's missing C terminal evidence is resolved by these artifacts. R5 FAIL, F1 FAIL, the post-launch F3 methodology disclosure, and human visual limitations remain unchanged. The R full external exact-head publication gate is still pending.
