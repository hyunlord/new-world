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
