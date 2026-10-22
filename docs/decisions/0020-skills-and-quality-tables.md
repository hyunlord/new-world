# ADR 0020 — Skills and quality tables

## Context
M2's promise (ROADMAP): "부싯돌을 주문하면 돌깎이가 만들고, 숙련도에 따라 품질이 다르다." Skills must change how fast work
goes and how good crafted items are, deterministically, in integer arithmetic (ADR 0016 C4), and the baseline must
never see them.

## Decision
- **Skills per craft skill**: `SkillKind { Knap, Woodwork, Chop, Quarry, Build }`; the recipe names its skill (knap
  blade → Knap; haft axe, split firewood → Woodwork). `Skills` is an ECS component added at a person's first trained
  work; levels 0–20.
- **XP = work ticks of finished jobs** (crafts, chops, quarry batches; a builder 1 per tick at a site). A cancelled job
  trains nothing.
- **Tables, not formulas at run time** (`sim_core::colony::skills`):
  - `XP_TABLE[L] = 15·L² + 15·L` — level 1 at 30 XP, 5 at 450, 10 at 1,650, 20 at 6,300;
  - `SPEED_PERMILLE[L]` = 1400 − 80·L up to level 5, then 1000 − 20·(L − 5) (1400 → 1000 → 700); applied to crafts,
    chopping and quarrying (building speed stays V7's);
  - quality = `clamp(30 + 3·level + bonus + material + temperament, 0, 100)`, `bonus` from `QUALITY_BONUS_TABLE`
    (−10 … +15, triangular weights `27 − |2v − 5|`, mean +2.5) drawn with one keyed draw on
    `(seed, STREAM_QUALITY, agent, tick, bill)`; `material` = `(hardness − 50) / 5` for blades (flint +4);
  - `AXE_PERMILLE[bucket] = 600 − 15·bucket`: a held stone axe of quality bucket b multiplies chop work ticks.
- **Temperament, two effects only**: persistence ≥ 600 ‰ +2 quality; novelty seeking ≥ 600 ‰ −2 craft ticks.
- **Tools**: a chopper without a tool picks up the best stockpiled axe within 24 tiles on the way; an axe lasts 200
  chops, then becomes one stone.

## Alternatives considered
- Skills keyed by work type (goal 3.7's `BTreeMap<WorkType, Skill>`): one "Craft" skill would mix knapping and
  woodworking, while the recipes and the inspector ("돌 다듬기 7") name specific crafts.
- Exponential XP curves or float multipliers: not bit-reproducible guarantees; a 21-entry integer table is.
- Quality from a uniform bonus: a triangular table keeps most items near the skill's mean and the extremes rare.

## Consequences
- A level-0 knapper's blade averages about 36, a level-10 blade about 66 (unit test over 200 keyed draws: difference
  30 ≥ 25). A novice's axe cuts chop time to 55.5 %.
- In the M2-U evaluation script every blade is made on day 1, so the crafter's blade quality cannot rise between day
  3 and day 20 (E7 fails; M2 evaluation) — skills grow with steady work, which the script does not give.
- The UI shows levels as tiers: 초보 0–4, 견습 5–9, 숙련 10–14, 달인 15–20.

## Date
2026-10-22

## Source
docs/v8/m2/M2_GOAL.md §3.6–3.7, §7; docs/v8/m2/M2_DECISIONS.md D9–D11, D13; rust/crates/sim-core/src/colony/skills.rs
