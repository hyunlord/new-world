# M-CORE decisions

Status: implementation in progress; no acceptance or publication claim.

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
