# Pass 17 pre-implementation compatibility audit

Audit date: 2026-10-15. Scope: PASS17_GOAL sections 2, 3, 5, 6 and 7.

The inspected art ref was `origin/codex/art-production-2026-10-wip`, resolved
with `git rev-parse` to **`46193a0301d335fc04dcfd6460e59cc0f1370d08`**.
Every art source reference below means that exact Git object, not the live art
checkout. This audit only used `git grep` and `git show` in the simulation
checkout; it did not access or modify the art checkout, build, test, or launch
Godot. The simulation HEAD read for the Stage B parent audit was
`e6bde09c5859bf8ba715d240f59092bd7d45eadf`; its simulation base is Pass 16
`adfdecfece79ab019964729319389fd602685fab`.

This is the required **before-Part-1 inventory**, not evidence that the later
implementation has passed compatibility or identity gates. Recheck the final
diff against the prohibitions below after implementation.

## Method and reproducible searches

Graft located the current-code definitions and bridge round-trip test first.
The exhaustive art inventory then used Git object reads, as the brief requires:

```sh
git rev-parse origin/codex/art-production-2026-10-wip
git grep -n -E 'TargetKind|AgentState' origin/codex/art-production-2026-10-wip -- '*.rs'
git grep -n -E '(AgentRules|SocietyRules|AgentSnapshotRow|AgentDetailRow|Tallies)[[:space:]]*\{' origin/codex/art-production-2026-10-wip -- '*.rs'
git grep -n -E 'TargetKind::\*|AgentState::\*|TargetKind as|AgentState as' origin/codex/art-production-2026-10-wip -- '*.rs'
git grep -n 'parent_ids' origin/codex/art-production-2026-10-wip -- '*.rs' '*.py' '*.gd'
git grep -n -E 'parent_ids\[0\]|parent_ids\.first|parent_ids\.get|parents\[0\]|parents\.first' HEAD -- '*.rs' '*.py' '*.gd'
```

The alias/glob-import search returned no hits. All Rust files returned by the
first search were inspected for `match` expressions; comments and string
literals were excluded from the match inventory. Type mentions in constructors,
queries, `if let`, and `matches!` are not exhaustive matches. Source-text test
strings such as `"match target {"` in the P7 alpha harness are not Rust matches.
Those tests are separately called out below because their constraints matter.

## Exhaustive target matches: 14 sites

All sites in this table enumerate the seven existing `TargetKind` variants
without a catch-all. Add an explicit `Activity` arm to each applicable source
when integrating Pass 17; do not replace the enumeration with a wildcard.
Paths are relative to `rust/crates/`.

| Art file and line | Meaning / required integration |
|---|---|
| `sim-bridge/src/ffi/world_node.rs:1622` | Snapshot target encoding; add code 8. |
| `sim-bridge/src/ffi/world_node.rs:1929` | Detail target encoding; add code 8. |
| `sim-systems/src/runtime/decision/agent_decision.rs:1705` | Current target need during critical preemption; Activity has no resource-need value. |
| `sim-systems/src/runtime/decision/agent_decision.rs:1826` | `has_resource`; Activity arrival means standing on the selected place. |
| `sim-systems/src/runtime/decision/agent_decision.rs:1967` | Consume dispatch; Activity must stay until its activity exit, without economy effects. |
| `sim-test/src/probe/deaths.rs:42` | Human-readable target name; add Activity. |
| `sim-test/src/probe/pass12.rs:62` | Per-agent task label; add Activity. |
| `sim-test/tests/harness_p5_beta_decision.rs:996` | Target-name helper in old decision tests. |
| `sim-test/tests/harness_p5_gamma_sleep_daynight_chronicle.rs:846` | First need-seek time; Activity belongs in the explicitly irrelevant-target group. |
| `sim-test/tests/harness_p5_gamma_sleep_daynight_chronicle.rs:899` | Need-seek counts; same explicitly irrelevant group. |
| `sim-test/tests/harness_p5_gamma_sleep_daynight_chronicle.rs:949` | Need-consumption flags; same explicitly irrelevant group. |
| `sim-test/tests/harness_p6_alpha_construction_components.rs:298` | Exhaustiveness test helper; add Activity. |
| `sim-test/tests/harness_p7_alpha_social_components.rs:495` | Exhaustiveness test helper; add Activity. |
| `sim-test/tests/harness_p7_alpha_social_components.rs:608` | Target-name helper; add Activity. |

The seven harness sites are additional compile dependencies beyond the brief's
production-code survey. They must not be missed because the old scene tests do
not instantiate activities.

## Exhaustive state matches: 9 sites

`AgentState` retains its three variants. A nested new target does not make these
matches non-exhaustive, but generic Seeking/Consuming arms can hide new behavior.

| Art file and line (under `rust/crates/`) | Existing behavior / caution |
|---|---|
| `sim-bridge/src/ffi/world_node.rs:1587` | Snapshot state encoding; place Activity-specific state 4 before generic Seeking/Consuming. |
| `sim-bridge/src/ffi/world_node.rs:1921` | Detail state encoding; same ordering. |
| `sim-core/src/components/agent_state.rs:100` | `AgentState::target`; generic forwarding already supports Activity. |
| `sim-systems/src/runtime/decision/agent_decision.rs:1043` | Main decision FSM; preemption to Idle must happen before this match, then use its existing Idle arm. |
| `sim-test/src/probe/deaths.rs:37` | Extract state prefix and inner target; no new state variant needed. |
| `sim-test/src/probe/fingerprint.rs:19` | State-tag encoding; add Activity-specific 4, preserving old-scene outputs. |
| `sim-test/tests/harness_p6_alpha_construction_components.rs:372` | Three-variant exhaustiveness helper; remains valid. |
| `sim-test/tests/harness_p7_alpha_social_components.rs:632` | Three-variant state-name helper; remains valid. |
| `sim-test/tests/harness_p7_delta_social_ui.rs:80` | Legacy state mapping helper; locks legacy scenes, not a fourth AgentState variant. |

## Remaining matches with targeted patterns / fallbacks

These sites were included in the inventory. Their wildcard, binding, or guarded
arms already cover additional targets, so compilation alone cannot detect a
wrong Activity outcome.

| Art file (under `rust/crates/`) | Match line(s) | Observed meaning |
|---|---|---|
| `sim-bridge/src/ffi/world_node.rs` | 1597 | `seek_kind` only encodes Food/Water/Sleep; other seeks map to 0. Keep this old field unchanged. |
| `sim-systems/src/runtime/decision/agent_decision.rs` | 1699 | Need value from Food/Water/Sleep; other target returns None. |
| `sim-systems/src/runtime/decision/agent_decision.rs` | 2185 | SeekTarget post-pass: wildcard at 2348 clears the target. Add an explicit Seeking Activity arm or activity walkers freeze. |
| `sim-systems/src/runtime/decision/agent_decision.rs` | 2265 | Resource-seek selector nested inside the post-pass. Its wildcard chooses food; Activity must not enter this resource-only branch. |
| `sim-systems/src/runtime/resource_regen/mod.rs` | 528 | Resource cleanup handles only Seeking Food/Water/Sleep and continues for other states. |
| `sim-test/src/probe/deaths.rs` | 270 | Social-partner existence only for Agent target; otherwise None. |
| `sim-test/tests/harness_p10_gamma_settlement_migration.rs` | 588, 681, 728 | Expect a specific Agent seek and panic for any other state. Legacy fixtures remain unchanged. |
| `sim-test/tests/harness_p7_alpha_social_components.rs` | 580 | Agent-target serde assertion; fallback panics. |
| `sim-test/tests/harness_p7_alpha_social_components.rs` | 794, 991 | Specific social state assertions; fallback panics. |
| `sim-test/tests/harness_p7_beta_social_system.rs` | 2223 | Permitted old social states; otherwise panic. |
| `sim-test/tests/harness_p7_gamma_social_chronicle.rs` | 321, 343, 390, 402, 917 | Social seek/consume filters with fallback. |
| `sim-test/tests/harness_s16_delta_need_stagger.rs` | 297, 339 | Resource-only consume tracking; other need target yields NaN. |

Three other matches mention the enums only as their result, not as a matched
type: `world_node.rs:3693` (round-trip fixture `Option<TargetKind>` construction),
`agent_decision.rs:1053` (founding/need tuple), and
`harness_p10_gamma_settlement_migration.rs:1265` (integer-to-target fixture).
The round-trip fixture's target list at `world_node.rs:3687` needs Activity so
that the new code is exercised.

`harness_p7_alpha_social_components.rs:937` onward additionally scans
`agent_decision.rs` source text and rejects wildcard `_ =>` arms in every
`match target { ... }`. Preserve that source-level invariant when adding Activity.

## Rule literals: 22 whole-struct literals, 2 update literals

`AgentRules` is declared at art `sim-core/src/rules.rs:18`, and `SocietyRules`
at `:63`. These are declarations, not literals. The exhaustive Git search found
the following whole-struct construction sites. Pass 17 adds only
`routine: None` and `family: None` respectively; preserve all other values.

| File under `rust/crates/` | AgentRules whole literal | SocietyRules whole literal |
|---|---:|---:|
| `sim-core/src/rules.rs` | 499, 1030, 1388 | 514, 1048, 1406 |
| `sim-test/tests/harness_revitalize_pass6_frozen.rs` | 19 | 46 |
| `sim-test/tests/harness_revitalize_pass7_frozen.rs` | 19 | 46 |
| `sim-test/tests/harness_revitalize_pass8_frozen.rs` | 19 | 46 |
| `sim-test/tests/harness_revitalize_pass9_frozen.rs` | 22 | 49 |
| `sim-test/tests/harness_revitalize_pass10_frozen.rs` | 22 | 49 |
| `sim-test/tests/harness_revitalize_pass11_frozen.rs` | 23 | 50 |
| `sim-test/tests/harness_revitalize_pass12_frozen.rs` | 23 | 57 |
| `sim-test/tests/harness_revitalize_pass13_frozen.rs` | 36 | 70 |

The two additional `SocietyRules {` hits are update literals, not whole ones:
`harness_revitalize_pass12_frozen.rs:220` and
`harness_revitalize_pass13_frozen.rs:224` set `fission: None` and inherit the rest
from `..literal.society`. They need no additional explicit field if the whole
literal above has the correct `family: None`.

The art `AgentRules` predates Pass 16's temperament additions. Do not replace a
current simulation literal with the older art literal during conflict resolution.

## Public schema boundaries: no new Pass 17 fields

Only the two rules fields above may extend existing public structs. Activity
and family data use new types and separate getters. In particular:

| Existing schema | Art declaration/construction sites | Fixed shape |
|---|---|---|
| `AgentSnapshotRow` | `sim-bridge/src/ffi/world_node.rs:1464`, whole literal `:1639` | 15 fields listed below. |
| `AgentDetailRow` | `world_node.rs:1825`, `Self` default literal `:1850`, whole literal `:1941` | 9 fields; dictionary exactly 9 keys. |
| `Tallies` | `sim-engine/src/lib.rs:67`; whole literal in `world_node.rs:3301`; `sim-test/tests/harness_revitalize_tallies.rs:54`, `:127`, `:153` | No activity, family or fission-couple counters. Probe trace owns these. |

Other grep hits `impl Default for AgentDetailRow` (`:1848`), return signatures
for `collect_agent_detail` (`:1900`) and `collect_tallies` (`:3080`), and
`harness_s16_epsilon_state_viz.rs:61` are not construction sites.

Snapshot fields: `entity_bits, x, y, state_tag, agent_id, seek_kind, target_x,
target_y, hunger, thirst, sleep, carried_food, carried_wood, target_kind,
life_stage`. Its ordinary dictionary at `world_node.rs:1779` through `:1793`
has corresponding arrays `ids, xs, ys, states, agent_ids, seek_kinds, target_xs,
target_ys, hungers, thirsts, sleeps, carried_foods, carried_woods, target_kinds,
life_stages`. The art getter already adds `ages_days` (`:463`) and `colds`
(`:466`) separately. Preserve these art-owned extras in a later merge; they do
not authorize any new Pass 17 snapshot key.

Detail fields and dictionary keys: `found, agent_id, x, y, state_tag, hunger,
thirst, sleep, target_kind`. The exact-key constant is
`world_node.rs:1874`; `agent_detail_to_dict` at `:1958` uses that constant.
`harness_p14_gamma_click_inspector.rs:478` through `:506` verifies the nine-key
contract, with further checks at `:660` through `:682`.

Art Tallies has 33 fields. The five art-only additions after the shared
`exclusivity_moves` are `wet_burn_wood`, `area_floor_wood`,
`winter_wood_lost_settled`, `area_floor_placed`, `area_floor_lost_at_max`.
Preserve those existing art fields when eventually merging. Pass 17 must add
none, and must not add couple-split fields to `FissionRecord` either.

## Art presentation at the pinned ref

`scripts/ui/agent_activity.gd:14` through `:42` only understands states 0–3
and targets 1–7. A stationary activity with the new state 4 resolves to `idle`.
If it is moving, the `moving` condition at line 38 still resolves to walk/carry;
therefore “state 4 always renders idle” would be too broad. State 3 with unknown
target falls through tile-derived actions and finally `eat`, so activities
must use state 4 in both snapshot and detail collectors.

`scripts/ui/panels/agent_inspector_panel.gd:151` and `:152` clamp values to the
last label. With its arrays at `:34` and `:39`, state 4 displays the existing
Working label and target 8 displays the existing Wood label. This is a known
presentation limitation until the art line consumes the new activity getter;
the simulation pass must not edit those scripts.

## Stage B: first-parent meaning and other parent arrays

The literal `parent_ids[0]`/`.first()` search finds one production consumer of
the first agent parent, plus a distinctness assertion:

| Source | Existing meaning | Stage B consequence |
|---|---|---|
| Art and simulation `sim-systems/src/runtime/settlement/settlement_system.rs:822` | `parent_ids.first()` finds that agent's mutable `Lineage` and calls `child` at `:826`, advancing its birth count. The old list at `:806` is sorted and takes two lowest ids. | Under family mode the first element must be mother. Lineage key and downstream identity draws intentionally follow the mother; retain old selection and mutation verbatim under `family == None`. |
| Simulation same file `:841` | Reads the first two ids into temperament slots 0 and 1. | Preserve `[mother, father]` ordering in the event and in the inherited temperament inputs. The computation uses both parents; this is not a male/female assumption. |
| Simulation `sim-test/src/probe/agent_trace.rs:250` and `:260` | Copies parents from AgentBorn, then iterates the first two to reconstruct the inherited temperament. | Keep event order and explicit fallback marker; do not infer biological mother from a legacy/fallback list alone. |
| Both refs `sim-test/tests/harness_p10_gamma_settlement_chronicle.rs:635` | Compares first and second parent for inequality, then checks both are founder ids. | No sex or maternal meaning; old fixtures remain valid. |

Other `AgentBorn.parent_ids` consumers are the event declaration at
`sim-core/src/causal/event.rs:525`, the event emission (art settlement system
`:905`, simulation `:933`), P10 alpha fixtures at `:395` and `:416`, and the
P10 gamma chronicle event extraction/iteration at `:176`, `:178`, `:612`–`:615`,
`:629` and `:639`. None assigns a sex meaning to index zero. The grep found no
other production read of the first agent parent.

The art branch has a **different** array also named `parent_ids`: settlement
ancestry. `world_node.rs:2493` declares it, `:2532` pushes
`settlement.parent.map_or(-1, ...)`, and `:2583` publishes it. Its consumers in
`scripts/core/settlement_selection.gd:62,78`,
`scripts/ui/panels/hud_status_panel.gd:442`, capture scripts, and art pipeline
tools operate on settlement ids. In particular,
`scripts/test/art/test_society_snapshot_p7.gd:32` checks
`snapshot.parent_ids[0] == -1` for an original settlement. This is not an agent
mother/father read and must remain unchanged.

## Integration checklist for the implementing agent

1. Keep all old scene rules `routine == None` and `family == None`; use the new
   v20 pins and I1–I5, not this source audit, to establish identity.
2. Handle all 14 target matches and the state-encoding sites; preserve the
   post-pass Activity SeekTarget and explicitly exercise code 8/state 4 in the
   bridge round-trip test.
3. Add only the two permitted existing-struct fields, preserve frozen literal
   values, and keep all statistics in trace types rather than Tallies or
   FissionRecord.
4. Preserve nine detail keys and all existing snapshot keys. Add
   `get_activity_snapshot` and `get_agent_family` independently.
5. On the art line's later merge, retain its extra snapshot arrays and Tallies
   fields, while resolving its older rule definitions forward to simulation
   Pass 16/17. Do not copy older art definitions over current ones.
6. Under family mode put the mother first, verify both partners are alive at
   the birth tick, and mark old-rule fallback births explicitly. Do not reuse
   settlement `parent_ids` for agent kinship.

No code changes or runtime compatibility verdict are claimed by this audit.
