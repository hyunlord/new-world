# M2 decisions

Numbered decisions of the M2 pass (`docs/v8/m2/M2_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m2-data/` (`$DATA`), outside the repository.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `v8/wip` = `e5594e3f` (checked with `git ls-remote` at the start,
  2026-10-22 08:42 KST).
- Choice: base worktree `../new-world-m2-base` detached at `e5594e3f`, with the main clone's `rust/Cargo.lock`
  copied in. Data in `$DATA` = `/Users/rexxa/work/m2-data/`. An owned `caffeinate -i` runs for the session.
  `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`. Start 08:42 KST; the 18-hour target is
  02:42 and the 22-hour hard stop 06:42 on 2026-10-23.
- Evidence: `$DATA/TIMELOG.txt`.

## D2 — Part 0: what the art records rest on

- Context: goal §4 asks for ADR 0018 with the art passes' numbers and "the sustainability condition", and for the
  branches by commit. `v8/art-trial2` is running in a parallel session and is **not on the remote** yet
  (`git ls-remote origin 'refs/heads/v8/*'` lists only `v8/art-trial`, `v8/art-concept`, `v8/main`, `v8/wip`).
- Choice: ADR 0018 cites `v8/art-trial` at `c3dafb07` and `v8/art-concept` at `a5406866` (both read with
  `git show origin/<branch>:<path>`), M1-ART's blind reading from `docs/v8/m1art/ART_REPORT.md` 8.2, and the owner's
  choice of 05 from this goal (§2; no record of it exists on any branch). The sustainability condition is written as
  the concept's own "200-asset consistency" risk (`art/concepts/05_ink_and_tone/CONCEPT.md` on `v8/art-concept`: fix
  the marks per material and the main/secondary line ratio), tested by ART-TRIAL2. ART-TRIAL2 is listed as running,
  with no commit.
- Evidence: the ADR's Source section.

## D3 — Identity base: a fresh capture at `e5594e3f`

- Context: goal §1 wants a base worktree at `e5594e3f`; the baseline world has not changed since M1.5's capture
  (M1.5 gate identity PASS), but a base measured at this pass's own base commit removes any doubt.
- Choice: `tools/v8/identity.sh --capture $DATA/base/bin $DATA/base/identity` with the base worktree's release
  binaries (08:42–08:56). Every identity run of this pass compares with it.
- Evidence: Part 1 code (tree of 09:15): `$DATA/identity/p1` — summary PASS (228,748 values, 0 differences),
  chronicle 64 files, raw samples 64 files, DETERMINISM PASS.

## D4 — Materials: embedded, integerized once, colony only

- Context: goal §5 — the registry is built once at `build_world_with` for the colony, its 15-property `f64`
  struct integerized into `MaterialStats { hardness, density, workability, flammability }` ‰; the baseline does
  not load it. Today nothing loads `data/material/**` at run time (`SimResources.material_registry` is an empty
  `MaterialRegistry::new()` everywhere).
- Choice: `sim_core::colony::materials`: the five files the colony uses (granite, limestone, flint, chert, oak) are
  embedded with `include_str!` and parsed by a new `material::parse_ron_str` (the body of `load_ron`), so the
  colony never reads the disk and the table is identical on every machine. Integerization, once per world:
  hardness = Mohs × 100 (flint 7.0 → 700), density = kg/m³ × 1000 / 25,000 (2,600 → 104), workability and
  flammability × 1000; `round()` then clamp to 0–1000 (no transcendental). The table is `ColonyState.materials`
  (saved with the colony state; a few hundred bytes). `SimResources.material_registry` is left alone (empty in both
  worlds; filling it could reach V7 code that reads it).
- Logs carry `oak` for now: M1 trees have no species, and inventing species would change the colony's worldgen for
  no M2 gain (goal 3.1 "default `oak`").

## D5 — Resource nodes: how many and where

- Context: goal 3.2: "boulders on hill tiles and 2 % of grass tiles with a keyed draw"; "flint outcrops on
  river-bank tiles (within 2 of water), 1 per 40 bank tiles".
- Choice: a boulder on **25 % of hill tiles** and 2 % of plain tiles (one keyed draw per tile,
  `STREAM_BOULDER`), never on the camp clearing, a building, a water source, a bush, a deadwood spot or a tree.
  "On hill tiles" read as "on every hill tile" would cover the hills with boulders; a quarter keeps them a
  feature you choose from. Outcrops: `bank / 40` where `bank` counts every non-water, non-mountain tile within 2
  of a river tile; they are placed on the free bank tiles in a keyed shuffle (`STREAM_OUTCROP`). Counting only
  free bank tiles first gave 10 outcrops on seed 0 instead of 23 (most bank tiles carry a tree), so the count
  uses all bank tiles. Amounts: keyed per tile (slot 1). Stone material: granite on hills, limestone elsewhere.
- Evidence: `colony_items.rs::resource_nodes_follow_the_placement_rule` (seeds 0 and 3: boulder shares within
  20–30 % of hills and 1.2–2.8 % of plains; outcrops exactly bank / 40).

## D6 — Item specs: kind, material, quality bucket

- Choice: `ItemStack` and `Carry` gain `quality` and `material`; `ItemSpec { kind, material, quality }` is what
  `put_items` and `take_items` move. A stack accepts a spec with the same kind, material and quality bucket
  (quality / 10); its quality is the count-weighted mean of what merged in, which stays inside the bucket. Raw
  items are quality 50. Per-kind stack maxima and carry loads are the goal's table (`ItemKind::default_stack_max`,
  `default_carry`; logs keep `ColonyRule::stack_max_logs`/`carry_logs`, so M1's numbers are one source).
- Haul: a loose stack goes to the cell nearest it that takes its spec; `JobAssignSystem` memoises "no cell takes
  this spec" per spec and skips such stacks for the rest of the tick (the memo's predicate only grows stricter
  within a tick, so the M1.5 D15 argument still holds).

## D7 — Sites need up to two materials; workshops in the colony state

- Context: the wood bench needs logs and stone (goal 3.3); `MaterialNeeds` held one kind.
- Choice: `MaterialNeeds { lines: [NeedLine; 2] }` (an unused line has `needed == 0`; it stays `Copy`); `Deliver`
  carries the item kind in `Job::item`; in-transit amounts are per `(site, kind)`. The bridge's `site_needed` and
  `site_delivered` columns report totals. Each blueprint's footprint, materials and work come from
  `BlueprintKind::plan` (the hut's from the colony rule). A finished workshop is a `Workshop` in
  `ColonyState::workshops` (id order), not an ECS entity and not a dwelling (`building_kinds`, sleep tiles and
  owners are untouched); its footprint is `Occupant::Workshop(id)`.

## D8 — The probe script waits for workshops

- Context: the M2-U script adds bills at workshops that do not exist when it is resolved (tick 360).
- Choice: a script entry may carry `"after": "workshop:<kind>"`; it is applied at the first tick at or after its
  `tick` when a finished workshop of that kind exists. New resolvers: `designate_quarry`/`nearest_nodes`,
  `place_blueprint`/`south_of_hearth` for any blueprint kind (the hut rule; a workshop's work tile is kept clear of
  later placements of the same tick), `add_bill`/`workshop_of_kind`, `set_work_priority`/`adult_rank`. The
  recorded command log carries the actual ticks, so the replay needs no conditions.

## D9 — Skills are per craft skill, not per work type

- Context: goal 3.7 writes `Skills: BTreeMap<WorkType, Skill>`, while the recipe table (3.4) names the skills
  `Knap` and `Woodwork` and the inspector line (9.4) reads "돌 다듬기 7".
- Choice: `SkillKind { Knap, Woodwork, Chop, Quarry, Build }` — the recipe's skill for crafts, one skill each for
  chopping, quarrying and building. `Skills { entries: [Skill; 5] }` (a fixed array indexed by `SkillKind::code`, so
  the component stays `Copy`; the goal's map in another shape). Added to a person at the first work that trains a
  skill; the baseline never has it. XP += the work ticks of a finished job (craft, chop, quarry; a builder gains
  1 XP per tick at a site); a job cancelled before it finishes trains nothing.
- Speed applies to crafts, chopping and quarrying. Building speed stays `ConstructionSystem`'s (V7 code, one progress
  per builder per tick); only its XP is counted. Gathering deadwood (Part 4) trains nothing.

## D10 — The tables (ADR 0020)

- `XP_TABLE[L] = 15·L² + 15·L` (level 1 at 30 XP, 5 at 450, 10 at 1,650, 20 at 6,300): a level-0 knapper's blade
  takes 63 ticks, so the first blades lift a novice by a level or two and mastery takes about 6,300 work ticks
  (4.4 days of nonstop work at one skill).
- `SPEED_PERMILLE`: 1400 − 80·L to level 5, then 1000 − 20·(L − 5) (goal: 1400 at 0, 1000 at 5, 700 at 20).
- Quality: `clamp(30 + 3·level + bonus + material + temperament, 0, 100)`; `bonus` from `QUALITY_BONUS_TABLE`, the
  values −10 … +15 with triangular weights `27 − |2v − 5|` (mean +2.5), one keyed draw on
  `(seed, STREAM_QUALITY, agent, tick, bill)`; material: `(hardness − 50) / 5` for blades, hardness on the 0–100
  Mohs × 10 scale (flint +4). A level-0 blade averages about 36, a level-10 blade about 66.
- `AXE_PERMILLE[bucket] = 600 − 15·bucket` (600 at bucket 0, 450 at 10): a novice's axe (quality 30–39) cuts chop
  time to 55.5 %.

## D11 — Temperament: two effects

- Persistence (p) ≥ 600 ‰ adds 2 to a craft's quality; novelty seeking (ns) ≥ 600 ‰ takes 2 ticks off a craft. Both
  are small on purpose (one bonus step, one tick in thirty); no other temperament effect in M2.

## D12 — `Maintain(n)` counts what is on its way to the stock

- Context: goal 3.5 counts "the stockpiled count of the output kind (across stockpile zones, unreserved)". With that
  count a `Maintain(10)` firewood bill overshot to 24 in `colony_workshops.rs`: firewood made at the bench waits at
  the work tile for a hauler, and each tick before the hauler stocks it another craft starts.
- Choice: the count is every item of the kind on the ground (stocked or waiting), in a hauler's hands, and the
  outputs of crafts in progress (`bills::maintain_count`); `BillSystem` and `JobAssignSystem` (before handing out a
  craft, updated as crafts are taken in the tick) use the same count. With it the test stays within 10 ± a carry
  (low 6 after a burn of 4, high ≤ 20).

## D13 — A craft at work finishes before yielding (calibration round 1)

- Context: inputs are consumed when a craft starts (goal §6, ADR 0019). Round 0 (M2-U, seeds 0–7, 20 days,
  `$DATA/try/p3a`): 1–3 crafts per seed were cancelled by an urgent need after they started, so their flint was
  lost; seeds 5 and 7 made only 4 and 5 of 6 blades because their outcrop held 6 and 7 flint.
- Choice: under the economy a crafter in the working stage yields only to a critical hunger or thirst (75, M1.5's
  `critical_preemption`), as M1.5 lets a near load finish its leg (`jobs::seam`). Round 1 (`$DATA/calib/r1`): 0
  crafts lost, 6 blades and 2 axes on all 8 seeds.

## D14 — Priorities: passes by level across the colony

- Context: goal 3.6 — "`JobAssignSystem` picks the lowest priority number first, then this order, then nearest".
  Applied per person only, a priority-1 crafter has no precedence over a priority-3 crafter who happens to be
  served first (`AgentId` order).
- Choice: the assignment runs in passes by level: every available person (AgentId order) tries their level-1 work
  types, then everyone their level-2 types, … level 4; within a pass, the person's types in `WorkType` order, then
  the nearest target. Level 0 = never. Without the economy (M1 rules) one pass over every type (M1's order).
- Measured (round 2, `$DATA/calib/r2`, `$DATA/try/e4b`): the Chop = 0 adult took 0 chop jobs on 8/8 seeds; the
  Craft = 1 adult made 0–3 of the 8 knap/haft crafts. Every craft of the M2-U script happens within hours on day 1,
  and the priority-1 adult is often busy or tending a need at that moment; precedence only helps when the person is
  free. Kept: it is the rule the goal states; E4 is reported as measured.

## D15 — Firewood through jobs: what replaced the decision arm

- Context: goal 3.8 / §8. The colony's firewood reached the hearth through the decision system's `GatherWood` arm
  (wood plans, claims, trips, inventory wood, the deposit system).
- Choice: one line in `decision/system.rs` — the decision's local firewood rule is
  `jobs::hearth::decision_firewood(resources)`, `None` when the economy and the firewood rule are both on (the M2
  colony). Every firewood branch of the decision tick reads that local: the wood plans, the urgent window, the
  claims, the `GatherWood` arm, the firewood exit of leisure, the loaded-haul check and the re-targeting of an emptied
  spot; the pickup arm is reached only by a `GatherWood` target, which no colony agent can get. The baseline keeps
  `rules.society.firewood` (identity PASS). `git diff --numstat` on `decision/system.rs`: +2 −1.
- In the colony, firewood is items: the wood bench's firewood bill (1 log → 4 firewood) and, when the colony is
  short and no firewood bill can run (no bench, or no logs), `JobKind::Gather` (`WorkType::Split`): walk to the
  nearest untaken deadwood spot, 30 ticks per batch of 2 firewood from the spot's wood units, until the hands hold a
  carry (10) or the spot is empty, then to the nearest stockpile cell with room, or to the hearth pile. "Short" =
  `max(hearth need, every firewood Maintain target) − (settlement wood + firewood items + a carry per gather job)`,
  the hearth need being the decision's former target (`wood_target`, rounded up; it includes the autumn's winter
  store). The goal says the fallback runs "before a bench exists"; it also runs when a bench has no logs, so a
  colony that stops chopping does not freeze.
- `HearthFeedSystem` (146, hourly, before `FireSystem` 147): each settlement below tonight's burn takes firewood
  stacks on stockpile cells or within 2 of its hearth, nearest first, one wood unit per item, logged `burned`.
  Tonight's burn (not the whole store) moves into the abstract stock; the rest stays visible as items.
- Evidence: `colony_firewood.rs` (no `GatherWood` target in 3 colony days; feed conservation; the baseline still
  gathers); calibration `$DATA/calib/r3` (M2-U 20 days, 8 seeds: 0 deaths, firewood in stock ≥ 28 every day from day 2)
  and `$DATA/calib/e6_r3` (3 years without orders, 8 seeds: 0 deaths of any cause).

## D16 — The hearth's auto-bill

- Choice: a finished wood bench starts with one bill flagged `auto`: split firewood, `Maintain(firewood_days ×
  burn_per_day)` = `winter_base_days` (4) × winter burn (3/h) × 13 burn hours = 156. A player's firewood bill at that
  bench replaces it (owner check 4 expects "장작 유지 40" to hold near 40, which a standing 156 would not), and player
  bills are inserted before auto bills, so the auto-bill never starves a player's bill of its turn.

## D17 — E8: the decision system's share did not fall by 10 points

- Context: E8 wants the decision system's share of the N = 1,000 tick to fall by ≥ 10 points against the M1.5 ledger
  (82.7 %).
- Measured (interleaved `bench-one n1000`, base `e5594e3f` vs candidate, same session, 1-min load ~12):
  decision 1,475 → 1,489 µs per tick; share 82.8 % → 78.9 %; tick mean 1,782 → 1,886 µs (+5.8 %). The share fell 3.9
  points, mostly because `JobAssignSystem` grew (16 → 82 µs: the priority passes and the firewood shortfall count),
  not because the decision tick got cheaper: in the fed benchmark world the wood arm was never its cost.
- A first candidate built the deadwood index every tick (1,600 spots at N = 1,000): `JobAssignSystem` 280 µs, tick
  +9 %, decision share 71.4 % — a "pass" that came from a slower system. The index is now built at most once per
  tick and only when the colony is short of firewood.
- Not done: 8.3 (foraging as a job) is allowed only if every bar is green by hour 14; E2, E4 and E7 are not (D18), so
  it was not attempted. E8 is reported as a fail.
- Official measurement (the ledger session, `$DATA/bench/`, base `e5594e3f` and M2 `f6a89e72` interleaved, 3 rounds,
  load 2.9–3.9): decision 82.7 % → 80.0 % of the N = 1,000 tick (1,430 → 1,371 µs, −4 %); `JobAssignSystem`
  15 → 62 µs; tick mean 1,726 → 1,712 µs. E8 FAIL (−2.7 points). The development figures above (decision 1,475 →
  1,489 µs, tick +5.8 %) came from a loaded session (1-min load ~12) at an earlier commit and are superseded. The M1.5
  ledger's 82.7 % is the share of its median-tick round (median of its three shares: 82.9 %). The numbers review
  recommends barring the decision tick's own µs in a later pass: a share also falls when another system grows.

## D18 — How the evaluation bars are measured (registered before the official run)

- Seeds 300–331 (not used by any earlier pass: `git grep` finds them only in the goal), the M2-U script
  `tools/v8/m2_u_script.json` (goal §10; bills are applied once their workshop stands, D8), 20 days,
  `--check-determinism`; and 72 days (3 colony years) without orders. Scorer: `docs/v8/m2/m2_score.py`.
- E1: the 6th blade crafted by day 10 (tick ≤ 14,400). E2: 2 axes crafted by day 15, and `chops_with_axe ≥ 1`
  by day 20. E3: firewood on stockpile cells at the start of each day 8–20 ≥ 20. E4: the Chop = 0 adult took 0
  chop jobs, and the Craft = 1 adult finished ≥ 50 % of the crafts of work type Craft (knap blade, haft axe; split
  firewood is work type Split). E5: 0 ticks with a ledger or reservation violation. E6: 0 starvation, dehydration
  or exposure deaths in the 20-day run; 0 exposure deaths in the 3-year run (≥ 30/32; with no orders there is no
  bench, so the firewood comes from the gather fallback, not an auto-bill). E7: the Craft = 1 adult's blade quality
  at day 20 minus at day 3, as the mean of 200 keyed draws of the quality formula at the adult's Knap level on each
  day (the same draws on both days, so the difference is 3 × the level difference); the mean quality of the blades
  the adult actually made before and after day 3 is reported beside it.

## D19 — Firewood lying away from a hearth stopped the gathering (found by the evaluation)

- Context: the first official run (`$DATA/eval`, commit `d05391cc`) had E6b at 29/32: seeds 300, 320 and 324 lost
  1–2 people to cold in their third year. Their settlement wood was 0 at ~26 dawns in 3 years while 43–137 firewood
  items lay on the ground and never moved (a temporary probe showed them scattered 10–30 tiles from the hearth:
  loads dropped where a gatherer was released by a need).
- Cause: the gather shortfall counted every firewood item on the ground as held, but the hearth feeds only from
  stockpile cells and its own pile (within 2 tiles), and without a stockpile no hauler moves loose firewood. The
  stray items blocked gathering for good.
- Fix: the shortfall counts only firewood a hearth can use (stockpile cells, hearth piles, loads in haulers' hands,
  crafts under way; `hearth::feedable_firewood`), and a gatherer with no stockpile cell drops at the settlement's
  formation tile (the tile the feed measures from; `hearth::hearth_tiles`). Calibration seeds 0–7, 3 years: dawns
  with an empty hearth 0 (was ~26 per seed); seed 300: 0 deaths. The stray items stay on the ground (no hauling to a
  hearth pile in M2; a limitation).
- The evaluation was then rerun in full from the fixed commit; seeds 300, 320, 324 are no longer blind for E6b, and
  the report says so.

## D20 — UI: keys and panels (Part 5, by a subagent; report `$DATA/ui/UI_REPORT.md`)

- **P = the priority table, Space = pause.** The goal puts the table on P, which paused the game since V7; pause
  moved to Space (and the Pause key). README updated.
- F3 opens a build menu (1 hut, 2 knapping spot, 3 wood bench, 4 drying rack; the order tool's mode is now `BUILD`
  with a kind, code 3 as the old hut mode); F7 is the quarry rectangle. A finished workshop opens its bill panel on
  click: a new bill is "만들기 5"; −/+ step 1 for make, 5 for maintain; auto-bills show "(자동)".
- Item names carry their material through a per-kind format key ("부싯돌 날"); ground stacks show kind, count and
  quality on hover only (the overlay has no item material column), keeping the 16-px ground calm.
- 14 temporary sprites in a new shelf of the code-drawn sheet (`tools/v8/gen_tiles.py`); M-ART2 replaces them.
- 90 new localization keys and 5 changed values (`docs/v8/m2/M2_LOCALE.md`).

## D21 — Code review dispositions (`$DATA/review/CODE_REVIEW.md`, round 1 on `d05391cc`)

No MAJOR finding. MINOR findings and what was done:
- m1 (a player's firewood bill dropped the auto-bill's working craft and its log): fixed — the replaced auto-bill's
  craft jobs are re-pointed to the new bill (same recipe). Test `a_players_firewood_bill_takes_over_the_auto_bills_craft`.
- m2 (a cancelled site returned its materials without their material): fixed — a need line remembers its first
  delivery's material (`NeedLine::material`, `deliver_spec`). Test `a_cancelled_site_returns_materials_with_their_material`.
- m3 (the decision's food "loaded" check went off in the colony with the firewood rule): fixed — that check reads
  the rule itself again (`decision/system.rs` one line changed; the pass's diff of that file is +3 −2, net +1); M1.5's food
  behaviour is back in the colony.
- m4, m6 (workshops and priorities accepted under M1 rules): fixed — `NotYet` without the economy. Test
  `without_the_economy_workshops_and_priorities_are_refused`.
- m5 (a blueprint could cover an unfinished workshop's work tile): fixed — `ColonyState::site_work_tiles`, kept
  by place, cancel and completion. Test `a_blueprint_may_not_cover_a_pending_workshops_work_tile`.
- m7 (craft protection depended on the margin field): fixed — checked before the margin rule's early return.
- NIT 10 (blueprints over resource nodes): fixed (a node blocks a footprint). The reviewer's remark on the
  uncommitted D19 edit: the hearth tile list is now cached once per tick.
- Kept, recorded: NIT 1 (quality keyed by `AgentId`, as the goal states; deterministic and saved), NIT 2 (`JobKind::
  work` is display-only), NIT 3 (`Job::amount` holds a chop's or quarry's work ticks), NIT 4 (gather may be sent
  while a runnable bench is busy; at most one carry over per tick), NIT 5 (one craft's inputs per supply trip), NIT 6
  (workshops are V7 `building_registry` entries), NIT 7 (`MoveBill` can move a player bill below an auto-bill),
  NIT 8 (a priority set to 0 does not cancel a current job; an adult's component keeps quarry 3 as an elder), NIT 9
  (axes are kept until death or wear), NIT 11 (`f64` on grandfathered `wood_target` inputs), NIT 12–14 (derived
  `Skill.level` saved, registry order, unpruned measurement tallies).

## D22 — A finished workshop is not a V7 building (found by the UI screen checks)

- Context: the screen-check run drew a hearth on the knapping spot and on the bench (`$DATA/ui/UI_REPORT.md`):
  `ConstructionSystem` put every finished colony site into `building_registry` with a `BuildingPlaced` event, and a
  registry entry without a `building_kinds` entry reads as kind 0 (hearth) in `get_building_snapshot`. The registry
  also feeds `SettlementSystem`'s member buildings and the routine's occupied tiles.
- Choice: a workshop completion skips the registry entry and the `BuildingPlaced` event (it keeps
  `ConstructionCompleted`); huts keep M1's path. Test `a_finished_workshop_is_not_in_the_building_registry`. The
  building layer's GDScript guard (skip rows inside a workshop footprint) stays as a second line of defence.
- The evaluation was rerun from the fixed commit.
