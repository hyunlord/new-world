# ADR 0019 — Items, bills and the economic loop

## Context
After M1 and M1.5 the colony had one item (`Log`), three orders, one blueprint and jobs that replace leisure. The V7
material data (`sim-core/data/material/**`, 105 definitions with 15 `f64` properties) was connected to no run time.
M2 (`docs/v8/m2/M2_GOAL.md`) adds the loop every later milestone builds on: resource nodes → gathering jobs → items in
stockpiles → craft orders at workshops → tools that change work → skills that change speed and quality — and takes
the first piece out of the 3,490-line decision tick (ADR 0016 C5): firewood.

## Decision
- **Items carry kind, material and quality** (`ItemSpec`). Seven kinds (log, firewood, stone, flint, flint blade,
  stone axe, hide). A stack holds one bucket: kind, material and quality / 10; its quality is the count-weighted mean
  of what merged in. Raw materials are quality 50.
- **One ledger per kind, every tick.** `created = on the ground + carried + held as tools + delivered to sites + on
  workshop shelves + consumed`, asserted by `jobs::invariants::check` (tests, probe, debug builds). A craft is a
  **transform**: its inputs are logged consumed (`craft_inputs`) when it starts and its output created (`crafted`)
  when it ends; firewood fed to a hearth is consumed as `burned`.
- **Inputs are consumed when a craft starts.** A craft cancelled after it started loses its inputs (counted as
  `crafts_lost`). The alternative — inputs held on the bench until the end and returned on a cancel — needs a second
  shelf state and a return path in every cancel reason; the M2 rule keeps one transition. To keep the loss rare, a
  craft at work yields only to a critical need (M2 DECISIONS D13): 0 crafts lost on the calibration seeds.
- **Resource nodes** (boulders, flint outcrops) are placed at colony worldgen with keyed draws and exhausted by
  `Quarry` jobs in batches of 4; no regrowth.
- **Workshops and bills.** A finished workshop blueprint becomes a `Workshop` in the colony state (not an ECS entity,
  not a dwelling) with a work tile, an input shelf and an ordered bill list. `BillSystem` (123, hourly) activates the
  first runnable bill; the job layer supplies its inputs (`Supply`, the shelf form of `Deliver`) and crafts it
  (`Craft`). `Maintain(n)` counts what is on its way to the stock (D12). Bill commands are logged and replayed.
- **Work priorities** (0 never, 1 highest … 4 lowest; default 3) are honoured in passes by level across the colony
  (D14): a person with priority 1 gets first pick of that work.
- **Materials are integerized once.** The colony embeds the five material files it uses (`include_str!`) and turns
  each into `MaterialStats { hardness, density, workability, flammability }` in permille when the world is built
  (`round`, no transcendental). Run-time code reads only those integers; the baseline never builds them.
- **Firewood through jobs; food stays abstract.** In the colony the decision system's firewood arms are off (one
  line: its firewood rule is `None` when the economy is on). Firewood is items, made by the wood bench's bill or
  gathered from deadwood as a `Split` job; `HearthFeedSystem` moves tonight's burn from items into the settlement's
  abstract `stockpile[Wood]`, one unit per item, so `FireSystem` is unchanged. Food stays an abstract settlement stock
  this milestone: foraging, meals and the food economy are spread over the decision tick, the stockpile systems and
  the routine, and moving them is a larger extraction than M2 can verify (goal 8.3 was conditional and not attempted).

## Alternatives considered
- Items as ECS entities (one entity per stack): every haul and merge becomes a structural change; the ledger and the
  overlay would iterate archetypes instead of a sorted map. Ground stacks stay a `BTreeMap` by tile.
- Inputs consumed at the end of a craft: no loss on cancel, but the inputs would sit reserved in two states (shelf,
  in work) and every cancel reason would need a return path.
- Loading the material files from disk at run time: the Godot build would depend on a data path; embedding is
  deterministic and needs no I/O.
- Firewood fed by agents carrying it to the hearth (no abstract stock): would change `FireSystem` and the baseline's
  settlement model; the feed keeps the fire's contract.

## Consequences
- Save format 2 (item specs, nodes, workshops and bills, two-line material needs, `Skills`, `WorkPriorities`, `Tool`).
- The M2 colony's decision tick no longer plans wood; its share of the N = 1,000 tick fell 82.8 % → 78.9 % (the
  evaluation's bar of −10 points is not met: the wood arm was not the decision tick's cost in the fed benchmark world;
  M-PERF continues the extraction).
- New bridge reads and commands (`docs/v8/snapshot_schema.md` §Economy, `docs/v8/bridge_api_m2.txt`).

## Date
2026-10-22

## Source
docs/v8/m2/M2_GOAL.md §3, §5–§8; docs/v8/m2/M2_DECISIONS.md D4–D8, D12–D17; docs/decisions/0013-jobs-replace-leisure-never-survival.md; docs/decisions/0016-simulation-contract.md
