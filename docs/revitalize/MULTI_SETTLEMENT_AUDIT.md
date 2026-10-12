# Multi-settlement audit (Pass 11, Part 0)

Base: `8cca0334` (Pass 10 final docs; code = `14372ea2`). Today every generated
world holds one settlement, so every assumption of "the" settlement is
untested. This audit lists each place that (a) takes the first / lowest /
only settlement, (b) uses the first Hearth or camp centre as "the" centre,
(c) aggregates over settlements in a way that breaks with two, (d) assumes
resources lie around one camp, (e) assumes every member is at its own fire,
(f) re-forms a settlement at an orphaned hearth, (g) ties tethers, builders,
plans or haul-home to "home", (h) takes per-seed values from the first seed
or assumes one settlement slot.

Method: a read-only sweep of `rust/crates/sim-*`, `tools/revitalize`,
`tools/worldmodel` (a fresh `general-purpose` subagent, then checked by hand
on the items v17 acts on). Line numbers are those of the base unless marked
≈ (a few were read while Part 1 edits were in the working tree and may be off
by a few lines). "v17 action" says what Part 2 does: **fix (v17)** behind a
v17-only flag or a v17-only code path (v8–v16 keep their behaviour bit for
bit), or **harmless** with the reason.

## How membership works (the fact everything below depends on)

A member who walks away stays a member until it dies or the exclusivity pass
moves it. `sync_membership` (settlement_system.rs:258-304) adds every live
agent within radius 5 of a formation tile; `resolve_exclusivity` (:323-379)
keeps a double member in the Manhattan-nearest settlement. So a walking group
stays on the parent's roster until the daughter forms, and any member that
later comes within 5 of the other camp (wood near it, water near it) would
switch villages.

## (a) First, lowest or only settlement

| file:line | what it does | with two settlements | v17 action |
|---|---|---|---|
| sim-systems/runtime/intervention.rs:48-49 | `home_settlement` = lowest settlement id | "home" is always the parent; rain soak (L70) and gifts never reach the daughter | Harmless, because it is the documented meaning. Add an explicit target (id or "newest") for schedules and the UI. |
| worldmodel/schedule.rs:89; tools/worldmodel/make_dataset.py:80,149,161 | `"home"` resolves to the lowest id | all dataset interventions hit the parent only | Harmless for v16 datasets. Document it for v17. |
| sim-bridge/src/ffi/world_node.rs:2724-2731 | FFI intervention defaults to `home` when id < 0 | the UI acts on the parent unless an id is given | Harmless; the UI needs a settlement picker. |
| sim-systems/runtime/settlement/settlement_system.rs:600-616 | SettlementReason history goes to the lowest-id settlement with capacity | the chronicle can go to the wrong village | Harmless (display only). |
| sim-systems/runtime/combat/combat_system.rs:161-173 | `settlement_link` = lowest sid involved | — | Harmless (history only). |
| settlement_system.rs:770-775 | birth parents = two lowest-id members | correct per settlement | Harmless. |
| sim-systems/runtime/decision/agent_decision.rs:679-689, 711-719, 739-745 | `member_of` / `meal_camp`: first settlement in id order wins | only matters during double membership, which the one-settlement check removes | Harmless. |
| sim-systems/runtime/agent/movement.rs:189-205; probe/metrics.rs:58-64; probe/pass5.rs:132-141; probe/deaths_observer.rs:49-60; worldmodel/export.rs:185-195 | `homes`: first settlement in id order | same | Harmless. |

## (b) The first Hearth or the camp centre as "the" centre

| file:line | what it does | with two settlements | v17 action |
|---|---|---|---|
| sim-systems/worldgen/camp.rs:238-245 | `deadwood()` centres on the first Hearth found in `building_kinds` | only one hearth exists at worldgen | Harmless today. Region deadwood needs its own function and centre, not a reuse. |
| camp.rs:83, 175, 200 | `berries`/`berry_candidates` 6..=16 around camp `c` | same | Regions need their own candidate generator. Run it after berries and deadwood with its own salt so the v16 world stays identical. |
| sim-systems/worldgen/terrain.rs:108 | berry repair takes donor Forest from anywhere more than 16 tiles from the camp | the donors sit exactly in wild-region territory (22–28), so the repair thins region forest | Fix in v17: exclude region bands from donors, or generate regions after the repair and validate them. |
| probe/long_run.rs:97-102; probe/pass6.rs:45-48 | "near hearth" = own formation tile is a Hearth and agent within 3 | correct per settlement | Harmless. The denominator problem is in (c). |
| settlement_system.rs:392-413, 432 | hearth-anchored formation scan: every Hearth tile is a candidate; needs ≥3 agents and ≥2 registry buildings within 5 | the daughter forms on its own once Hearth + 1 building + 3 agents are present. The Hearth must be in `building_registry` (stamped from `building_event_queue`), not just in `building_kinds`. | Fix in v17: FissionSystem must stamp the Hearth through the queue, and may need to own the formation instead of relying on the scan. |

## (c) Aggregation over settlements

| file:line | what it does | with two settlements | v17 action |
|---|---|---|---|
| probe/pass5.rs:47-48 | `fire_lit` = **all** settlements lit | a dark daughter hearth makes the world "unlit" | Fix in v17: also emit per-settlement or `fire_lit_any`. |
| probe/pass10.rs:85-91 | seed-winter `dark_nights` from that all-lit flag; `members_at_start` = `settled_agents` (all members) | a daughter dark night counts for the whole seed; the H3 tables mix both camps | Fix in v17 (per-settlement winter rows). |
| tools/revitalize/fork_metrics.py:176 | `fire` = **max** over settlements | means "any lit", the opposite of the probe; daughter dark nights disappear | Fix in v17: pick one rule (min or member-weighted) shared by probe and tools. |
| tools/worldmodel/bench_v1.py:91 (and :89-92), bench_v0.py:68-71, tools/revitalize/diversity_p10.py:116-117 (via bench_v1) | food/wood/members summed; fire = max | sums are fine; the meaning of fire changes and shifts the world-model V vector | Fix in v17: add a min or per-settlement column; keep v16 behaviour unchanged. |
| fork_metrics.py:177 | food store summed | — | Harmless. |
| tools/revitalize/evaluate_pass10.py:127-144 | H2 `one_settlement`: fails when any day has settlements ≠ 1 | every successful fission seed fails | Fix in v17: 1 ≤ settlements ≤ `max_settlements`, chosen by scene. |
| evaluate_pass10.py:71-74, 291-293 | H5 checks evening-near-hearth and home-distance against the v15 baseline | walkers drag both down | Fix in v17: exclude agents in transit, or set v17 thresholds. |
| probe/metrics.rs:143-144 | `home_distance_mean` measured to own formation tile | walkers are still parent members, so it jumps by ~25 | Fix in v17: exclude agents in transit. |
| probe/long_run.rs:97-102 + probe/summary.rs:362-365 | `evening_near_hearth_share` = near own hearth / **all** idle adults | walkers sit in the denominator | Fix in v17 (same exclusion). |
| probe/summary.rs:357-360 | night-in-shelter / alive | walkers sleep in the open | Harmless (real effect). Note it in the report. |
| probe/long_run.rs:44-56 | `food_supply_per_day` = union of catchments, each bush counted once | correct; jumps when the daughter forms | Harmless. |
| probe/metrics.rs:167 | `shelters_completed` = global shelter count − initial | the daughter's founding shelters count as "completed" | Fix in v17 if they are stamped directly. |
| probe/metrics.rs:168; probe/pass4.rs:28-31; probe/pass5.rs:37-39 | stockpile, stock target, wood target summed | consistent sums | Harmless. |
| probe/runner.rs:264 | owned-housing overflow = alive − sum of owned capacity | — | Harmless. |
| probe/deaths_observer.rs:49-60 / probe/deaths.rs:290-292 | home distance at death to own formation tile | walkers who die show about 25 | Harmless (true value). |
| worldmodel/export.rs:158-176, 341-347 | `settlements.npy` with S slots in first-seen order, NaN before a settlement exists; agent `settlement` index | already handles S > 1 | Harmless. `wood_target` is the same for every slot (world-level). |
| worldmodel/export.rs:203-206 | agent → settlement index | correct | Harmless. |
| sim-bridge/src/ffi/world_node.rs:2300-2350 | settlement snapshot centroid of live members | the parent centroid drifts along the walk route | Harmless (the marker uses the formation tile). |
| world_node.rs:2428-2432 | the same wood target for every row | — | Harmless (burn is per hearth). |

## (d) Resources assumed to lie around one camp

| file:line | what it does | with two settlements | v17 action |
|---|---|---|---|
| sim-systems/runtime/resource_regen/mod.rs:194-216, called at 278 | the winter deadwood fall is a **world total**, handed out round-robin over **all** `wood_source_max` spots in key order | region spots soak up the parent's winter fall even **before** fission and in `game_v17_nofission` (1–3 regions × 6–12 spots against ~20 camp spots) | **Fix in v17:** a per-region accumulator, scaling, or limiting it to spots within `wood_radius` of a settlement. |
| sim-systems/runtime/intervention.rs:113-140 | windfall goes one unit per spot per pass, in (x,y) key order, over all spots | low-x regions get the windfall first; the parent gets less | Fix in v17: limit to the target settlement's `wood_radius`. |
| resource_regen/mod.rs:269 (`bury_wood`), regen_wood :158-174, food `regen_channel` | per spot | — | Harmless. |
| sim-systems/runtime/settlement/food_economy.rs:14-24, 120-133 | supply = bushes within `forage_radius` (16) of own formation tile | centres 22–28 apart, so the two catchments overlap. Both birth gates (settlement_system.rs:731) and both winter-prep targets count the shared bushes | Fix in v17: give each bush to its nearest settlement or weight it 1/k. |
| agent_decision.rs:105, 153 (`choose_bush` / `choose_spot` radius around `home`) | forage 16 / wood 18 from the formation tile | region resources less than 16/18 from the parent hearth get stripped before fission. Daughter members may also gather parent wood at distance 5 and get pulled into the parent (see g8). | Decision: put region resources more than 18 tiles from the parent hearth, or accept the overlap. |
| camp.rs:30-44 (shore water), :45-81 (camp picked at river distance 3) | water is the river only | a region 22–28 away may be 20+ tiles from water. Thirst seeks the nearest shore tile (agent_decision.rs:2150), which can sit inside the parent's 5-tile radius and absorb the member (g8). | **Fix in v17:** require shore water within N tiles of a region, at a tile more than 5 from the parent hearth. |
| sim-systems/runtime/settlement/firewood.rs:270-288 `wood_target` | a world-level target set by season | the same target for a small daughter | Harmless, because burn is per hearth. |

## (e) Every member assumed at its own fire

| file:line | what it does | with two settlements | v17 action |
|---|---|---|---|
| sim-systems/runtime/settlement/fire.rs:106-115 | `lit` = settlement had a full hour of wood; no position or Hearth check | walkers are warmed by the parent fire 25 tiles away | **Fix in v17:** decide whether warmth depends on position, and how agents in transit are treated. |
| fire.rs:126-131 | cold set = **all** members of an unlit settlement, wherever they are | a new daughter with 0 wood is unlit on its first winter night, so every founder gets the Cold marker and exposure damage; non-members are never Cold (free immunity for detached walkers) | **Fix in v17:** provision the daughter's wood at formation (`stock_share`), and give agents in transit a defined exposure rule. |
| intervention.rs:63-78 | rain + soak on the targeted settlement | — | Harmless. |

## (f) Re-formation at an orphaned hearth

| file:line | what it does | with two settlements | v17 action |
|---|---|---|---|
| settlement_system.rs:906-914 | dissolves only when 0 members **and** no buildings | buildings never go away, so a parent emptied by deaths becomes a permanent zombie | Harmless for formation. |
| settlement_system.rs:258-297 | sync_membership adds every live agent within 5 of the formation tile, zombies included | a daughter member passing the zombie gets added; the one-settlement check (:323-372, nearest tile) can move it into the empty village | Fix in v17: skip adding to settlements with no members, or require the agent to be idle at that camp. |
| settlement_system.rs:392-432 | any Hearth tile with no settlement, ≥3 agents and ≥2 buildings forms one | if a fission fails (`settle_timeout_days`), the orphan hearth + shelter re-forms when any 3 agents wander by | **Fix in v17:** on timeout, remove the Hearth and sites, or only form at hearths flagged by FissionSystem. |
| settlement_system.rs:497-501 | at formation, unowned buildings within radius+3 are given to the new settlement (Owned housing, v9+) | only works if the founding shelters are already `BuildingKind::Shelter` | See g3. |

## (g) Tethers, builders, plans and haul-home tied to "home"

| # | file:line | what it does | walking group / daughter | v17 action |
|---|---|---|---|---|
| g1 | movement.rs:330-345 | idle/evening/child/elder tethers step toward `home` | a walker that goes Idle is dragged back to the parent | **Fix in v17:** walkers need a Seeking state, or are exempt from tethers. |
| g2 | movement.rs:278-300 (`hauls_home`, 117-133) | a loaded idle member walks straight to its formation tile | provisions carried by walkers trigger a haul back to the parent; `loaded` (agent_decision.rs:944-949) also blocks all trips | **Fix in v17:** move provisions stockpile-to-stockpile at formation, not via inventories. |
| g3 | sim-systems/runtime/construction/construction_system.rs:206-212 | a site becomes `Shelter` + sleep tile + owner **only** if `site_owner` has it | founder-placed shelter sites with no owner finish as untyped buildings: no beds, no capacity | **Fix in v17:** pending-daughter ownership, or stamp founding shelters directly the way worldgen does (camp.rs:84-106). |
| g4 | agent_decision.rs:641-653 + ~991 | under planning only assigned builders build; assignment goes per `site_owner` settlement to the nearest idle members, with **no distance cap** | founding sites with no owner never get builders. Parent sites can pull members from far away. | **Fix in v17** (with g3); add a distance cap. |
| g5 | sim-systems/runtime/agent/beds.rs:29-75; agent_decision.rs:2146-2156 | beds assigned to owned shelters per roster; Sleep seeks `bed_of` | walkers keep their parent beds and walk back at night until the daughter forms | **Fix in v17:** drop the bed when the walk starts, sleep on the ground or near the site. |
| g6 | agent_decision.rs:2229-2235 | the post-pass catch-all clears the `SeekTarget` for any unhandled Seeking kind; movement freezes a Seeking agent with no target | a new "migrate/found" target kind freezes the walkers | **Fix in v17:** add an explicit post-pass arm. Also extend the target_kind codes (world_node.rs:1495-1497, 1802-1806) and the recorder schema. |
| g7 | agent_decision.rs:1488-1570, 1651-1700 | non-members are pulled to the **nearest member** of any settlement with capacity | if walkers are dropped from the parent, they get pulled back to the nearest parent member | **Fix in v17:** exclude agents in transit from the migration arm. |
| g8 | settlement_system.rs:258-297, 323-372 | add everyone within 5, then keep the nearest formation tile | the daughter forms correctly (founders move to it). But any member who goes within 5 of the other camp (wood at distance 5, water near the parent) switches villages. | Fix in v17: keep membership sticky unless the agent stays; at least exempt agents with a settlement. |
| g9 | agent_decision.rs:679-745 | forage, wood and meal plans use `home` = formation tile | walkers forage and eat at the parent until reassigned | Harmless once the walk state skips these arms. |
| g10 | sim-systems/runtime/settlement/stockpile_deposit.rs:108 / stockpile_consume.rs:145; agent_decision.rs:975-981 | deposits and meals only within the radius of the agent's own formation tile | the daughter starts with no stock: no meals, no fire | Fix in v17: initial stock transfer (`stock_share`). |
| g11 | sim-systems/runtime/settlement/shelter_planning.rs:52-110; housing.rs:28-35 | per settlement, ring 2–5 around the formation tile, Owned count | — | Harmless. |
| g12 | settlement_system.rs:706-745 | birth gate: own capacity, stock and catchment supply | walkers still count in the parent's `member_agents` | Harmless; overlap is covered in (d). |
| g13 | agent_decision.rs:1006-1020; sim-systems/runtime/survival/mod.rs:86-92 | social pairs on the same tile; death roster cleanup | — | Harmless. |

## (h) First-seed values and S=1 assumptions in the recorder

| file:line | what it does | v17 action |
|---|---|---|
| probe/summary.rs:168 | `rules` block built from `args.seeds.start` only; v16/v17 rules depend on the variety | Fix in v17: if FissionRule depends on the seed, move it to `seed_info`. Otherwise harmless. |
| probe/pass10.rs:56-62 → worldmodel/export.rs:467-468 | `variety_json` exports only the 8 v16 slots | Fix in v17: add `wild_regions`, `region_bushes`, `region_deadwood` only for v17 scenes (v16 uses `without_regions`). |
| probe/pass10.rs:40-50 (`seed_info`) | deadwood counts cover camp spots only | Fix in v17: add region counts. |
| probe/summary.rs:62 | `settlements_daily` | Harmless (feeds the new H2). |
| worldmodel/export.rs | S taken from first-seen order | Harmless. |


## v17 decisions taken from this audit (implemented in Part 2)

| # | item | v17 behaviour |
|---|---|---|
| 1 | the walk (g1, g6, g7) | founders keep `AgentState::Seeking { Agent(leader) }`, the post-pass gives every founder (leader included) `SeekTarget = site`; the social gate and chase timeout skip `Founding`; urgent needs pre-empt the walk but keep the marker; the next Idle decision re-enters the walk before every work/forage/firewood/meal/social/building/combat arm. No new `TargetKind`. |
| 2 | tethers and haul-home (g1, g2) | `movement.rs` skips the tethers and the haul-home step for `Founding` agents. Provisions travel as a pending store held by `FissionSystem` (`stock_share`), never in inventories. |
| 3 | builders, plans, meals (g4, g9, g10) | `Founding` agents are excluded from builder assignment, forage/firewood plans and the meal wait; `StockpileConsumeSystem` does not serve them. |
| 4 | beds (g5) | kept. A group leaves at 06:00 and walks ~25 tiles (≈ 200 ticks at one step per 8 ticks), so it normally arrives before noon; a group that is still walking at night sleeps in its parent's beds and walks on in the morning (accepted, reported). |
| 5 | founding buildings (b, g3, g4) | `FissionSystem` stamps the Hearth and two Shelters directly as `camp.rs::populate` does (`building_kinds`, `building_event_queue`, sleep tiles), so they are complete, typed buildings; the formation scan then forms the daughter and gives it the shelters (Owned housing, radius + 3). |
| 6 | membership after founding (g8, f) | under `FissionRule` the proximity sync only admits agents that belong to no settlement (sticky membership); `FissionSystem` moves the whole group from parent to daughter explicitly on the founding tick. Count of exclusivity re-assignments reported in METRICS. |
| 7 | fire (e) | `FireSystem` never marks a `Founding` agent `Cold` for its parent's hearth (it is not at that fire); a founded member is marked at the daughter's hearth, which has the pending wood. Position-aware warmth for ordinary members is out of scope (members away from the hearth at night are rare: night rest). |
| 8 | winter deadwood fall and windfall (d) | under v17 (`FirewoodRule::regional_wood`) the winter fall is split into a budget per area (camp area + each region) proportional to its spot count, each with its own round-robin cursor; windfall fills the home settlement's nearest spots first. v8–v16 keep the world-total round robin and key-order windfall. |
| 9 | orphaned hearths and zombies (f) | `FissionSystem` keeps `retired_hearths` (formation scan skips them): the hearth of a cancelled founding, of a merged remnant and of a settlement dissolved with 0 members. A settlement that reaches 0 members is dissolved explicitly although its buildings stay. |
| 10 | probe, recorder and tools (c, h) | probe schema 11 adds per-seed fission, settlement and exposure-by-stage blocks, `settlements_alive` and `fire_lit_home`; the existing aggregate keys keep their v16 meaning. Recorder schema 5 slots keep their index; `evaluate_pass11.py` replaces H2's one-settlement clause; `fork_metrics.py --home-slot-fire` reads the home settlement's slot (12.1); `bench_v1.py` untouched. **Not done (review round 1, M3), accepted and reported:** (a) no per-settlement dark-night winter rows — `seed_winters` still counts a night dark when any settlement's fire is out (`pass5.rs` all-lit flag), so in v17 a dark daughter makes a dark night of the world in the reported X3 clauses; (b) `home_distance_mean` and evening-near-hearth still include agents in transit (founders walk a few hours once per fission, so the effect on v17's reported J6 numbers is small); (c) `shelters_completed` counts a daughter's two stamped shelters; (d) no builder distance cap (g4) — a parent's sites are built by its own members, and founders are excluded. |
| 11 | catchment overlap (d) | **changed in Part 3** (review round 2, L4): under `FissionRule` a bush inside the catchments of n settlements with members counts 1/n in the capacity-weighted supply (round 1, `food_economy::weighted_source_count`), and region bushes and deadwood lie beyond the camp's forage and wood radii (round 3, `camp::regions`), so the two birth gates no longer plan the same bushes twice. |
| 12 | water (d) | regions are placed by the goal's rule (no water condition); daughters drink at the nearest water tile, which may be the river near the parent. Sticky membership (6) keeps a thirsty daughter member from switching villages; the distance from each region centre to water is reported in the worldgen counts. |
| 13 | interventions (a) | unchanged: `"home"` = the lowest settlement id (the original village), as the goal says. |
