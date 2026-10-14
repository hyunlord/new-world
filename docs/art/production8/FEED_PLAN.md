# Part 1 implementation plan (read-only preparation)

Prepared against the merged working tree during Part 0. This document is a plan, not implementation or passing evidence. Product edits are blocked until the leader completes Part 0 gate and baseline. No simulation behavior changes are needed.

## Current paths and observation hazards

- `scripts/ui/panels/hud_status_panel.gd`: `_ingest_snapshots` observes selection, then food, then society, construction, births, deaths. `_push` appends every line visibly and drops the oldest beyond four. `_prune_old` uses render frames.
- `scripts/core/settlement_selection.gd`: `observe` assigns persistent ascending presentation numbers, including historical fission parents/daughters. `number_for` returns zero for an unknown ID. Observe the complete society snapshot before making any message arguments. Keep raw IDs in metadata, never visible argument slots.
- `scripts/ui/panels/god_hand_panel.gd`: casts synchronously in `cast_kind`, updates `Selection.record_cast(tick, kind, target)`, then notifies effect nodes. This map is keyed by tick/kind and cannot distinguish identical same-tick casts to different villages.
- `get_intervention_state` currently exports only the latest 32 records' ticks, kinds, requested amounts and applied amounts. There is no target ID or absolute identity. Do not infer targets from current selection, row index in a sliding window, or tick/kind.
- `get_society_snapshot` currently has fission IDs, parent IDs, departure/founding/cancel ticks and daughter IDs, but omits `food_taken`.
- `rust/crates/sim-systems/src/runtime/settlement/fission.rs`: departure withdraws `food_taken` from parent; cancellation returns unchanged `group.food` to parent; founding deposits unchanged `group.food` into daughter. No intermediate mutation of `group.food` occurs. Read-only source evidence permits exclusions from `food_taken`; do not edit this simulation source.
- `scripts/test/capture_framings.gd:_advance_to` advances eight ticks before HUD observation. A capture can cross 20:00 without observing that exact boundary; opposite stock movements within an eight-tick interval are irrecoverable from end stocks. Captures must explicitly observe boundary ticks. Prefer per-tick HUD observations in the p8 capture path when exact observed-delta accounting is required.

## Required additive getter dependencies (leader-owned Rust)

Within the allowed `world_node.rs` getters/collectors/tests only:

1. Intervention `log_ids: PackedInt64Array`: original absolute index into `resources.intervention_log`, retained when selecting the last 32 rows. Clear consumer dedup state on reset. This disambiguates same-tick identical casts and survives rolling-window shift.
2. Intervention `log_settlement_ids: PackedInt32Array`: actual `settlement` for the five targeted variants, `-1` for global cold-snap/windfall. Do not fabricate a simulation target for global acts.
3. Society `fission_food_taken: PackedInt64Array`: paired with sorted `fission_ids`. The existing record value supports departure/cancel/founding exclusion.
4. Getter tests assert target IDs and absolute IDs across >32 rows, repeated same-tick equal kinds to distinct villages, and global `-1`; assert food values and sorted fission alignment. Keep old arrays and semantics intact.

Global act feed wording uses the village selected at the cast surface as presentation context, not a simulation target. Record this context keyed by the returned/latest absolute log ID immediately after successful cast (or send it directly to a feed consumer). Preserve `target_scope: global` and raw simulation target `-1` in metadata. If a global cast is observed without UI context, use an explicit recorded unknown/context fallback decision, never silently claim a village was its exclusive target. Food exclusions always use actual getter target and applied units.

## Full feed key to class table

| Key | Class | Visible eligible | Village argument position (zero-based) |
|---|---|---|---|
| HUD_FEED_DEATH_STARVATION | protected | yes | none |
| HUD_FEED_DEATH_DEHYDRATION | protected | yes | none |
| HUD_FEED_DEATH_COMBAT | protected | yes | none |
| HUD_FEED_DEATH_OLD_AGE | protected | yes | none |
| HUD_FEED_DEATH_EXPOSURE | protected | yes | none |
| HUD_FEED_FISSION_DEPARTED | protected | yes | 1 |
| HUD_FEED_SETTLEMENT_FOUNDED | protected | yes | 0 |
| HUD_FEED_FISSION_CANCELLED | protected | yes | none |
| HUD_FEED_SETTLEMENT_ABANDONED | protected | yes | 0 |
| HUD_FEED_SETTLEMENT_FORMED | protected | yes | 0 |
| HUD_FEED_GOD_CAST_FMT | protected | yes | 0 |
| HUD_FEED_SETTLEMENT_DISSOLVED | protected legacy alias, no current producer | yes if explicitly pushed | 0 |
| HUD_FEED_AGENT_BORN | ordinary | yes | none (population) |
| HUD_FEED_SITE_STARTED | ordinary | yes | none (construction ID) |
| HUD_FEED_SITE_COMPLETED | ordinary | yes | none (construction ID) |
| HUD_FEED_DAY_SUMMARY_FMT | ordinary | yes | 0 |
| HUD_FEED_STOCKPILE_DEPOSIT | routine | never | 0 |
| HUD_FEED_STOCKPILE_CONSUME | routine | never | 0 |
| HUD_FEED_MORE_FMT | presentation counter, not an event | counter only | none |

Store the same explicit mapping in `DESIGN.md` and code. Unknown keys should not silently become protected. Counter creation must not append feed events.

## Observation transaction and food attribution

One ingest call samples a single simulation-clock snapshot and uses it throughout; synchronous getters do not advance simulation. An injected callable or stub world supplies this clock in tests. No render-frame clock and no uppercase day-tick constant in HUD source.

Order:

1. Snapshot time, settlements, society, interventions. Observe selection first.
2. Detect reset/clock rewind and clear all current-world histories/buckets/cursors as existing reset semantics require.
3. Collect unseen cast and fission-transfer identities BEFORE food processing. Emit each event once, including metadata. Compute signed exclusion sums by village for this observation interval.
4. For each village with a prior raw stock, compute `raw_delta = food - previous_food`, then `normal_delta = raw_delta - signed_exclusion`. An unseen historical record whose tick predates the stock baseline must not be subtracted. Do not replace this with resetting the entire stock baseline on cast: ordinary movement in the same interval would be lost.
5. Preserve the old raw-stock threshold aggregation exactly: raw stock change since last-notified stock reaches >=3 or <=-3, then append old deposit/consume key with class routine, visible false and presentation village number. Add raw settlement ID, signed amount, observation tick and attribution/exclusion metadata. An excluded god/fission movement can therefore remain in the raw routine audit log without appearing in summary totals.
6. Track each observed `normal_delta` independently in the current daily bucket: positive contributes stored, negative contributes eaten. Do not derive both from net, and do not wait for the >=3 threshold before counting. Smaller changes must not disappear or move to the next day when aggregation spans 20:00. Preserve a movement audit (metadata list or accessor) of per-observation normal deltas and exclusions so A31 can reproduce sums, even when no >=3 routine record emitted.
7. At an exact 20:00 observation, include the ending interval's movement in the closing bucket, emit at most one summary per village/boundary if `abs(stored-eaten)>=10 OR eaten>=10`, then clear bucket. Determine boundaries from `day_index` and `time_of_day`, with `ticks_per_day` supplied by the snapshot only where conversion is needed. Never derive display day from tick alone; game clock has an offset.
8. In a skipped-boundary interval, record boundary/observed tick and incomplete sampling explicitly. There is no safe way to reconstruct when opposite movements happened; do not distribute one stock delta among missed days or manufacture summaries. Runtime and capture cadence should observe frequently enough to avoid this limitation; capture stepping can stop at boundary.
9. New village: initial stock is baseline, not ordinary deposit. Founding transfer is logged separately; do not subtract a transfer from a village lacking a previous stock (that creates false eating). Retired village: retain identity/history for audit and discard stale stock tracking after final accounting.

Use separate dedup collections for event emission and food exclusions. If the event mark is consumed before food adjustment, ordinary accounting must still receive its signed exclusion. Cancellation and founding have distinct IDs from departure, e.g. `food:depart:<fid>`, `food:cancel:<fid>`, `food:found:<fid>`.

## Protected retention and overflow

Replace frame fade with simulation-tick age. Recommended `PROTECTED_RETAIN_TICKS=240`, `NOTIF_FADE_TICKS=480`, cap four visible rows. Preserve source event tick separately from `observed_tick`/retention start: newly observed protected news gets 240 ticks of readability even if snapshot delivery was late. Pausing means the clock does not advance and cannot expire a row.

Keep eligible rows separate from the chosen four. When inserting ordinary news, use free slots or replace expired/unprotected older rows; never displace a protected row under 240 ticks. Protected competition beyond four displays the newest four protected rows (oldest goes first per brief) and counts hidden protected rows in `+N`. Keep their records and metadata; overflow is not deletion. After protection expires a newer line may displace it. Ordinary traffic alone must never flood hidden routine rows into either visible rows or counter.

Expose current `get_visible_feed`, `get_feed_log`, and overflow count/capture metadata. Each event record has class, visibility eligibility, source tick, observed tick, stable event ID, arguments and relevant raw IDs; current visible state must be distinguishable from historical eligibility. Recompute actual visible/hidden protected membership in capture metadata rather than treating a copied boolean at insertion as current truth.

Do not confuse death-source retention (120-tick recent death buffer and its dedup cleanup) with display protection (240 ticks). Keep source eligibility checks; extend only feed display behavior.

## Localization and static pins

Change stockpile wording to Village / 번 마을 and remove famine / 기근. Also update legacy settlement formed/dissolved wording for consistency. Add three Part1 keys exactly as GOAL section11: day summary, more, god cast. Update both Fluent files, compiled en/ko and registry atomically; no missing-key formatting window.

`harness_p14_delta_status_panel.rs` pins frame fade 480; re-pin to simulation-tick fade 480 and protected minimum 240 with the same visible cap four, documenting the behavioral replacement in `docs/art/TEST_LEDGER.md`. `harness_stockpile_feed_2_5c.rs` also pins threshold, tracking/push patterns and localization shape; preserve meaning and only adjust lexical assumptions if structural extraction requires it. `harness_revitalize_screen_contract.rs` forbids the literal `TICKS_PER_DAY` anywhere in HUD text, including comments.

Existing `test_death_exposure.gd` sets `_frame_tick=500` and expects expiry. Replace this with injected clock progression, retain original death identity assertions, and record why in ledger. `test_founding_group.gd` checks the last routine record's args `[2,5]`; keep this assertion intact, optionally add explicit routine/hidden assertions.

## Concrete tests before Part1 claim

`test_feed_priority.gd` uses a stub world/clock:

- 50 >=3 stock changes with mixed signs; every raw routine record logged once, routine invisible, exact normal movement audit and one 20:00 summary.
- One departure and one cast before churn; both visible at +239 ticks despite arbitrary rendered process calls and pause; replacement allowed after +240 only according to queue policy.
- Five or more protected events produce four rows, oldest removed from visible competition and correct `+N`; no dropped audit record.
- Two same-tick same-kind food casts on separate villages and repeated identical casts on one village; each logged once and applied amount (not requested amount) excluded once.
- Fission departure negative share, cancellation positive return and founding daughter baseline; ordinary changes in those same observations still counted.
- Raw deltas below three around 20:00 still sum in the right bucket; equality threshold cases net 10 / eaten10 included, net9 eaten9 excluded.
- Repeat observation at 20:00, pause, initial old histories, clock rewind, registry-free stub snapshots, and >32-cast sliding log window.
- Missing boundary cannot silently pass exact accounting; mark sample coverage and verify capture boundary stepping separately.

`test_feed_wording.gd`: locale en/ko; exercise every village argument slot using raw IDs 7 and42 and expect numbers1 and2, preserving numbers after abandonment; no famine/기근 in routine text; all mapping keys defined in both locales. Construction IDs and population are intentionally not converted to village numbers.

Run existing death/founding tests plus new Godot tests after baseline is green. Leader owns final static Rust tests and full gate. This preparation ran no cargo, Godot, product changes, commits or pushes.
