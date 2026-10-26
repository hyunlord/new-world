# M-CORE decisions

Status: implementation in progress; no acceptance or publication claim.

**Current performance rule: D10 supersedes the original ≤4 rule and D2's
historical admission requirement.**

## D1 — Published base and isolated ownership

Fetched origin on 2026-10-26. `origin/v8/main` resolved to
`9855bd082ad88a4c49296410f206f148d98c4df9`, containing
`docs/v8/mux2/MUX2_REPORT.md`. Created `/Users/rexxa/work/new-world-core-dev`
on `v8/core` from that SHA. The first commit, `001f236c3`, preserves the
owner's complete goal. Source changes and runs use this worktree only.

The release probe/perf build completed successfully before implementation.
Frozen executables, dependency lock, SHA256 manifest, build head and empty
Rust source diff against BASE are in `/Users/rexxa/work/core-data/base-bin`.
The build head differs from BASE only by the goal document.

## D2 — Measurement admission and evidence

The first observed one-minute load was 30.91; after the base build it was
67.20. Neither meets the owner's <=4 condition. No current decision share
is claimed; the historical 80% figure is not today's measurement.
Functional captures can run under load, but performance comparisons require
an admitted session without concurrent builds/probes from this task.

Use seed15100 colony stress scenes n200/n1000/n2000, 1440 warmup and 2880
measured ticks, three interleaved rounds. Decision mean is total_ms*1000/2880;
the reported share is the arithmetic mean decision cost divided by the
arithmetic mean tick cost. Final same-session comparison must measure both
frozen old and candidate executables and retain per-run load and survival.

Planner update (2026-10-26): concurrent M-UX2.1 Godot FPS runs and builds
occupied this Mac approximately 17:30–18:40 KST; the planner has stopped
that work until CORE publishes. All arm elapsed times from the initial
captures are explicitly **INVALID**, including values retained in the
inventory and external aggregate. Counts and identity remain functional
evidence. No share `s` or quiet performance number was accepted. Remeasure
both current per-system share and arm costs under the original load-≤4
condition; the update waives no goal requirement. Later observed load was
still 46.95, so the stopped planner work is not itself quiet-host evidence.

## D3 — Preserve behaviour until the single switch

Capture baseline identity, the nine OFF fixtures from `mux_safety.sh`, and
the two-seed24-day steward-ON fixture from the frozen source before edits.
Every pre-switch implementation commit must compare these outputs exactly.
Arm counters and shadow results live outside serialized simulation state:
they must not allocate causal IDs, evict tile events, consume RNG streams or
change fingerprints. Missing fixture coverage is a failure, not a zero.

Colony rules inherit relation, memory and routine settings from baseline.
V7 labels are insufficient evidence of unreachability. Port/replace/drop
verdicts require the requested execution counts and explicit replacements.

## D4 — World registration and save ordering

Concrete system constructors belong in sim-systems; sim-engine owns scheduler
metadata. Preserve this dependency direction. Current save loading registers
systems before installing saved resources (`save/mod.rs`); world-dependent
registration must receive the saved world rules first. New persistent mind
state requires explicit binary save version/migration handling. Append causal
event variants rather than renumbering existing variants. Verify active
behaviour, hysteresis and the bounded choice ring across save/load.

## D5 — Baseline boundary and existing wildlife ownership

Do not move shared need/clock/consumption helpers under baseline: colony jobs,
mood, wildlife, movement, resource regeneration and settlement systems use
them. The isolation check must reject indirect re-exports too.

DraftSystem (121) and FleeSystem (123) run before the decision slot (125).
The new behaviour registry must have explicit ownership of their continuation
and choice recording, without replaying their side effects. Shadow includes
these skipped old-seam ticks in its classification and coverage accounting.

## D6 — Evaluation arithmetic and held-out separation

Freeze policy after at most four logged calibration rounds on seeds0–7.
Held-out seeds are1232–1263 inclusive (`1232..1264`),48days, steward/defence ON,
without player orders. Compare BASE on those same seeds. Historical MUX2
measured values remain the acceptance reference: stockpile, houses/beds,
cold deaths and survival >=31/32; wolf deaths <=1 in >=29/32; ledger32/32.
Firewood: report exposed/32 and no-depletion count >=exposed-1. Do not reuse
the old scorer's fixed seed set or fixed32 cold denominator.

## D7 — Story-yield ancestry

Define depth as parent edges, not record count. Use durable chronicle cause
references because the tile ring evicts after eight events. Report missing
ancestors and cycles separately; do not manufacture chains from UI records.
The observer and definition must be identical for BASE and M-CORE.

The initial development instrument incorrectly excluded an entire path when
its older ancestor was missing, even after three verified edges. Schema 2
retains the verified prefix; a dangling reference adds no edge, and detected
cycles remain excluded and separately counted. Depth counts record edges,
not presentation-only pressure/modifier label nodes. Distinct chains are
distinct observed kind paths, excluding IDs and unresolved suffixes. A root
and a missing-ancestor path with the same verified kinds are one observed
kind path. Annual windows are half-open; partial years are not extrapolated.
Twelve focused tests pass, including depth-two/depth-three missing-ancestor
boundaries, cycles and ordinary-world fingerprint preservation. Development
schema-1 smoke evidence remains preserved and is not the reference metric.

## D8 — Arm observation scope and count correction

The off-by-default `perf_arms` feature records 247 registered keys. An early
source extraction counted 219 because its lowercase-only expression omitted
28 uppercase `.L` call-site keys; no source change explains that difference.
The runtime ON export independently confirms 247 keys. Timers include nested
work and instrumentation overhead; never sum them into a system cost or use
them as the quiet uninstrumented performance result. Count-only observations
have zero timing and distinguish evaluations, successful candidates, cascade
commits, continuations and cardinalities. Routine replay evaluations are not
additional unique person-ticks. Drivers discard setup and determinism-twin
observations and retain explicit zero rows.

Before the story observer correction, frozen default and feature binaries
both matched BASE's steward ON two-seed 24-day summary byte for byte, with
determinism PASS. The feature eight-seed 48-day ON summary also matched a
fresh BASE control exactly; ledger and reservation violations were zero.
Both initial binaries also pass the full baseline and nine OFF comparisons:
228,748 shared simulation values with zero differences and zero one-sided
paths, 64 chronicle files equal in content, equal raw sample digests and
determinism PASS. The strict arm aggregator validates all 11 fixture groups,
66 worlds, 3,824,640 world-ticks and 247 scope rows. The corrected final
observer binary also passes the same full comparison and the ON 24-day
fixture. Its schema-2 story command matches the ordinary probe fingerprints
on two 25-day worlds, including one full and one partial calendar year.
Raw provenance and receipts live in
`/Users/rexxa/work/core-data`; no held-out seed has been run.

## D9 — Separate CORE outcome scorer

`tools/v8/core_score.py` preserves the published MUX2 scorer and validates
CORE's exact seed set, complete observer coverage, physical housing and
fatal-predator provenance. It binds summaries and receipts to the scorer,
source and binary hashes, and validates matched BASE/new-brain controls.
Freeze chronology remains an explicit pre-run attestation, not a property
proved by equal hashes. Historical BASE outcomes are descriptive controls;
the new brain is tested against D6's registered thresholds.

Receipts bind the imported validator file as well as the scorer itself;
mutation and mismatch controls reject stale bindings. Independent literal
tests lock the six fixed thresholds to 31/31/31/31/29/32.

Unexposed firewood rows are null, neither pass nor fail. The denominator is
the exposed count, the required count is `max(0, exposed - 1)`, and zero
exposure is labelled `NOT_EXPOSED`. Ten scorer tests and synthetic CLI
controls pass. Outcome-row success does not claim shadow, why coverage,
performance, UI or full CORE acceptance.

## D10 — Planner corrects admission to the standing ledger rule

Authority: [planner correction, 2026-10-26 22:00 KST](CORE_PLANNER_CORRECTION_2026-10-26.md).
The original goal is retained verbatim in commit `001f236c3`; the working goal now incorporates this correction. The operative rule for §1.1 and §5
is now the [performance ledger](../../perf/PERF_LEDGER.md), with its M4
[§3.6 provenance](../m4/M4_GOAL.md): every repeat must start at one-minute
load **strictly below 8**; record all 1/5/15-minute values before each repeat.
A repeat starting at ≥8 is contended/INVALID, retained and never used as a
comparison base. A post-run load is diagnostic, not an added acceptance
condition. No extra ≤4 or sustained-minute gate applies.

Every old/new comparison interleaves both frozen binaries in the same
session. Use the within-session ratios for the mind and total-tick bars;
the initial old-brain-only share establishes the design baseline but cannot
replace that later matched comparison. Keep previous failed admission logs
and invalid arm timings; they lack the required per-repeat admission proof
and are not retroactively promoted by this correction.

If three recorded windows 15 minutes apart remain at ≥8, proceed with
inventory, mind, shadow and held-out evaluation without timing, and complete
timing before the gate. This explicit fallback supersedes the former
measure-first impasse only after its three-window evidence exists. The
remaining scope, calibration ceiling, held-out freeze, identity, publication
and isolation requirements are unchanged. The planner states M-UX2.1 is
paused until CORE publishes; actual host admission is still measured.
