# ADR 0057: Generic gate evidence binds products and native world occurrences

> Retired before publication, 2026-11-04: D-S0-2 stops this execution/observer design. Published legacy gate restored; only small contract tests retained. Follow-up #685. Historical decisions below are evidence, not an active implementation requirement.

Status: implementation in progress; runtime qualification and publication pending.

## Context

Pass-specific configuration selected historical binaries, fixture subsets and
acceptance branches. Matching output alone did not establish the product source,
complete workload or ordinary-world safety. M6b Stage0 requires a generic gate
against the candidate's published-main merge base, with independent self and
deliberate-defect proofs. Stage −1 published BASE0 is
`2f6993009ddb600e9249c28f9768a1a5252c8821`.

## Decision

Keep product identity distinct from harness identity. A publication uses its
candidate as harness; self-proof runs two separately built BASE products under
the new harness. Proof receipts never authorize publication. Freeze source,
effective inputs, build outputs, actual child exits and complete occurrence
coverage. Ordinary worlds retain actual initial census, realized terminal tick,
native knowledge capability and full loss history, including every reset/load.
Loaded history requires a hash-bound save prefix; an empty UI page is not proof.

Observer code is opt-in and excluded from simulation resources. It issues no
commands and does not change ticks, timers or test workloads. BASE observer
compatibility overlays are explicit derived sources with reviewed preimages,
separate independent builds and required behavior/performance qualification.
Default-off and runtime neutrality are claims requiring execution, not comments.

Use the existing committed pairwise normalizer. Preserve unknown keys and native
state; presentation-only exclusions remain explicit. Intentional fixture-change
declarations require a post-BASE ancestor commit and reason and cannot waive
ordinary survival or KnowledgeLost. Mandatory workload contracts independently
pin existing seeds, duration, input scripts and instance coverage, so editing
matching registry fields cannot quietly weaken a test. Additive fixtures remain
possible.

Keep injected F10 separate: authenticate the exact script, resolved targets and
accepted native commands, validate the original loss/door oracle and complete
pre-injection safety. A committed companion removes only that kill entry and is
an ordinary world. Never use an intentional fault to exempt other losses.

Performance uses fresh merge-base binaries, interleaved rounds, native child
admission journals and recomputed C3 ceilings. A named CHANGELOG justification
can extend a specific ratio to1.15, never a ceiling. Stall eligibility is
provisional until authenticated capture and every other required step pass.
Preserve raw strict failures and the applied waiver as distinct facts.

## Alternatives considered

- Retain per-pass gate modes: rejected because the tested baseline and fixture
  coverage would continue to depend on historical pass configuration.
- Trust a source label or a matching summary: rejected because neither binds
  the executable, effective inputs, child process or complete native history.
- Omit discarded or loaded worlds: rejected because their loss history and
  founding population still affect the survival contract.
- Exempt BASE losses in the self-proof: rejected because Stage0 explicitly
  requires zero KnowledgeLost; a behavior-preserving implementation cannot
  silently change that bar.

## Consequences

Native sidecars and process/build manifests add tooling complexity, but remove
implicit trust in a filename, echoed PASS or caller-supplied clean flag. Historical
logs remain readable; they do not select the current BASE.

The published BASE ordinary CORE steward-OFF seed0×20days has KnowledgeLost
`clay_digging` at28560. Stage −1 correctly preserved it. The new strict self-proof
must reject that loss; shortening the fixture, adding orders or weakening safety
would violate the owner contract. Generic implementation can proceed, but no
Stage0 successful self-gate or publication is claimed. Stage1–3 held-out seeds
remain unconsumed. Resolve any actual proof failure through recorded evidence
and the authorized stage boundaries.

## Date

2026-11-02

## Source

`docs/v8/m6b/M6B_GOAL.md` Stage0; owner D-minus1;
`docs/v8/m6b/M6B_DECISIONS.md`; `docs/v8/m6b/M6B_REPORT.md`.
Development proof and unresolved safety evidence are retained externally under
`/Users/rexxa/work/m6b-data/stage0/`; none substitutes for an exact-commit full gate.
