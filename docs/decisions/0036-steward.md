# ADR 0036 — Optional ordinary-command steward

Status: accepted for M-UX Stage A implementation; evaluation remains a separate evidence gate.

## Context

M-UX A8 requires optional colony assistance without resource grants, hidden authority or replay divergence. Existing player commands, knowledge gates and construction jobs are the authoritative effects.

## Decision

The colony assistant is an optional command producer at priority 125, every 360 ticks. Stable registration puts it after JobAssignSystem (124) and AgentDecisionSystem (125), before job execution (127). It reads the same physical state and uses the same command validation, reservations and jobs as player orders. It never writes resource grants, accepts migrants or drafts people.

Version-one `sim-core/data/colony/steward.ron` defines ordered conditions, actions, reserve days, content choices and localized labels. IDs are unique and fit the chronicle cause byte. Registry validation rejects unknown content and unsupported schema versions. Mechanical action variants are code; content choices stay data. Stockpile `6 + adults/2` is a target **area**, rounded up to the smallest square; this avoids interpreting “cells square” as 121 cells for ten adults. A house contains beds for living people, with its door on the hearth-facing border; construction waits while another building is underway. Food demand uses the established `need_milli` conversion of actual member hunger, not a fixed adult count. Fuel demand derives from booked-night seasonal severity and burn hours; inventory includes the hearth's stored wood. Winter bill reserve days remain data.

Housing/Food restrict exclusive housing/hunting actions. Stockpiling, fuel, production and teaching are shared support in either scope. Default simulation factories and probes remain OFF; the game bridge explicitly records the new-game ON policy. `--steward on` is the only probe default override. The player can record `SetSteward { on, scope }` at any time.

## Provenance and precedence

Existing command records retain their positional layout. Appended `Command::Steward { rule, command }` wraps an ordinary command; exported JSON retains the inner `cmd` with explicit `source: "Steward"`, `rule`, and `phase: "after_assignment"`. Accepted actions update saved provenance, cooldowns and a monotonically numbered StewardAction chronicle event. Renderers use that sequence for toasts.

A canceled steward target vetoes that target and its owning rule for two days. Thus the planner cannot immediately relocate a canceled building or zone. Player-authored bill settings are protected indefinitely; the assistant may replace the economy's untouched automatic fuel bill but never a player bill. Existing marks, blueprints, rooms and assignments are not canceled by the assistant. All selection ties are distance, y, x or stable agent IDs.

Recorded playback suppresses planning and queues source commands to the same after-assignment phase. Ordinary player commands still apply before the tick. Lifecycle metadata is derived from those recorded actions in both paths, and tests compare complete v7 save bytes after playback, including command results and chronicle records.

## Save migration

Format 7 appends StewardState after the frozen format-6 body fields. `SimResources.steward` uses serde `skip`: it is absent from the resource wire format and defaults when decoding. The explicit v7 body extension restores the setting and provenance. A separate LegacyBody decoder reads the exact v6 body, defaults to OFF, and inserts the stateless StewardSystem into the registered system-list position before installation. Existing enum variants remain in their original order; new variants are appended. Any future change to the v6 resource/component shape still requires a frozen versioned decoder; serde defaults cannot repair positional bincode changes.

Real pre-change v6 fixture verification is an explicit ignored integration test invoked with `MUX_V6_FIXTURES`. It loads both captured files, checks OFF and preserved commands, resaves v7, and compares two continued engines byte-for-byte. The coordinator retains the actual fixture bytes outside git.

## Validation boundary

Scoped tests cover data-only rule identity, invalid data, OFF defaults, ordinary source logs, scope, two-day cancellation, full-state replay and v7 continuation. The coordinator owns baseline/OFF identity, calibrated evaluation, held-out evaluation, full gates and UI acceptance. Opt-in probe `seeds[].mux` measurements report physical completion, first-winter coverage and deaths, all-tick cold-snap usable-fuel zeros, population and accepted rule counts. Default probe summary shape is unchanged. No target threshold is inferred from an incomplete seasonal window.

## Review corrections

Player protection is now a read-only pre-apply proposal: removal targets are captured while they still exist, then vetoes/cooldowns commit only if the ordinary command succeeds. A rejected bill or failed cancellation cannot change assistant policy. Regression tests include the complete two-day boundary and expiry, plus positive/negative fixtures for all six planners.

The fire's existing active-pressure multiplication was extracted verbatim into `fire::effective_burn_per_hour`; FireSystem calls it with the same hourly value and retains its other pressure handling. Steward reuses this authoritative mechanic instead of adding a pressure-content branch. This narrow extraction is behavior-neutral within the A8 behavior commit; final OFF fixture comparison remains required. Existing fire/firewood tests pass. It is not a change to wolf mechanics or an evaluation-threshold adjustment.

## Preregistered round-two candidate (D15)

After round one failed the original survival/fuel bars, D15 retains all six command families and the missing-bed trigger. The first room adds data-defined service cells and space for the existing stockpile, preferring legal templates enclosing storage. Existing zones, buildings and orders remain intact. Where terrain prevents enclosure the planner chooses a legal fallback; it does not dig, relocate storage or override placement validation. Workshops prefer work/output tiles on storage or within the established hearth-pile reach, then roof and distance.

The D15 addendum permits ordinary tree-chop preparation for a valid future house template. Its read-only preview ignores only the tree that would be harvested; all other placement checks remain. Preparation respects knowledge, existing marks, vetoes and the data-defined protected camp radius. The planner waits for harvesting before issuing ordinary blueprints, which still pass the original validator. No automatic refuge orders, drafting or wildlife changes were introduced. Extra construction cost and terrain-limited enclosure remain risks against the unchanged day-10 and survival bars.


## Alternatives considered

- Direct resource or finished-room mutation: rejected because it bypasses ordinary labor, knowledge, inventories and replay.
- Enable all simulation factories: rejected because OFF evaluation and historical behavior must remain available.
- Increase detached-room size again after round two: rejected because larger bedrooms still provided no stockpile coverage.

## Consequences

Source orders and saved policy are inspectable and replayable. Terrain and labor can prevent assistance from achieving evaluation targets; no target is waived. Format-6 positional types must stay frozen until an explicit later migration replaces them.

## Date

2026-10-25.

## Source

[M-UX goal, A8](../v8/mux/MUX_GOAL.md), [simulation contract](0016-simulation-contract.md), [extensibility contract](0033-extensibility-contract.md). Calibration evidence and preregistrations are retained by the coordinator outside git under `/Users/rexxa/work/mux-data/calibration/`.

## Preregistered terrain-aware candidate (D19)

Round two produced eight larger bedrooms but no covered stockpile cells; combat deaths stayed at 73. D19 reuses existing natural room boundaries instead of issuing illegal construction on hills, water or mountains. Candidate envelopes compensate blocked cells only until the unchanged bed, stock and service-cell demand fits, bounded by the existing room maximum. The existing pure room detector must find a single connected preview containing the intended stock cells and bed capacity, with a legal door on the hearth-facing side and reachable exterior. All effects remain ordinary chopping and blueprint commands; no terrain, room, wildlife or drafting state is directly changed.

Opt-in probe diagnostics record first/final physical layouts and pre-tick context for people lost while hunting threats exist. They live only in the probe observer, are not saved gameplay state, and do not appear in OFF summaries. They distinguish observed context from a causal conclusion: a job before a death does not by itself prove that the job caused the death.

The D19 aspect-ratio addendum searches the square-derived width and its immediate smaller/larger widths; alternate-width origins are restricted to hearth/stock anchors. Ranking prefers stock coverage and then hearth inclusion at unchanged usable demand. With an interior hearth, the door is the nearest legal perimeter opening. Candidate room detection uses the bounded local envelope, not a full-map flood fill per candidate. Multiple disconnected detected rooms are rejected.

Production derives prospective wall/bed cells from the same pure house preview, avoiding them for new workshop footprints and work tiles while tree preparation completes. This ephemeral constraint survives normal preparation cooldowns and support-workshop construction; existing orders are not canceled and no saved state is added. The actual house still waits for other construction. Target vetoes and scope remain authoritative.

Threat-loss diagnostics also retain the observer's pre-tick context when a flight begins, because by the death tick FleeSystem has already canceled the original job. Both snapshots are observational; neither changes decisions, saves or OFF output.

## Preregistered candidate clearance and compact envelopes (D26)

Round three still roofed no stockpile cells. Tick-zero counterfactuals separated transient people from fixed geometry: actual bed/service capacity and person clearance exposed shared candidates in four seeds; allowing later near-anchor heights exposed legal area-40 envelopes in three more. One seed retained a fixed node obstruction. These observations establish candidate feasibility, not survival or a Flee defect: reachable roofs may fail the existing person-versus-threat distance test.

D26 counts actual eligible bed/service cells using a prefix table, preserving demand and the existing room-area cap. Only near-stock/hearth origins enumerate additional heights; global fallback retains one sufficient height. Within the existing coverage preference tier, area precedes distance so a legal compact envelope wins over an unnecessarily large one. The local room detector and fixed-obstacle checks remain authoritative.

Pure geometry previews omit transient people, then check every intended ordinary blueprint with the real person-aware placement validator before issuance. A temporary obstruction returns no blueprint commands while retaining the same ephemeral prospective wall/bed reservations for support-workshop planning. Tree preparation remains ordinary validated chopping, and final construction still waits for harvest and clearance. No saved fields, resource grants, terrain mutation, new rule families, threshold tuning, drafting or wildlife mechanics change. Waiting may delay the original day-10 housing bar; static feasibility does not waive that risk or any evaluation bar.

## Preregistered House completion role (D27)

Round four physically roofed seven stockpiles and all hearth/workshop work tiles, but shared rooms classified Workshop and all eight worlds recorded zero bedroom sleepers. Existing bed assignment ranks legacy shelters ahead of bedrolls outside Bedrooms. House now has an optional data-defined `completion_role` (serde default None, configured Bedroom), completing its own finished bed enclosure through ordinary SetRoomRole even after physical bed demand is met. Accepted House provenance is required for every finished bed and every built boundary wall/door; natural terrain boundaries remain allowed, and foreign overrides prevent completion. Later player replacements at previously owned tiles do not inherit historical ownership. The existing six families/order, scope, cooldown and replay phase remain unchanged.

Any valid raw player SetRoomRole in the saved command log permanently suppresses automatic role completion, including Auto while OFF and idempotent Nothing. Unknown-room failures do not suppress it. This deliberately conservative rule remains safe after room-ID changes without adding saved geometry history; it can decline completion of an unrelated later room. No saved wire shape changes.

Bedroom assignment, sleeping and mood benefits also enable existing shared-bedroom illness transmission. Relabeling Workshop loses its900-permille crafting-time benefit. These are accepted policy tradeoffs, not cosmetic changes. Physical roofs, terrain, Flee, wolves, resources, drafting and all evaluation bars remain unchanged. Ordinary construction plus actual BedAssignmentSystem, source replay, data-only alternate role, ownership rejection, scope/cooldown and player Auto save/load regressions verify the implementation; they do not establish survival acceptance.
