# ADR 0027 — The storyteller and seasonal pressure

Status: accepted (M5, 2026-10-23; drafted in Part 0, final in Part 7 with the amendments below).

## Context
VISION §7 items 5–6: pressure arrives at times the system chooses (the god's hand of V7 is gone, ADR 0010), and it
must be felt through the existing loops — firewood, foraging, the body. Before M5 the only weather is the calendar's
winter severity draw (`firewood::winter_severity`, mild / normal / harsh burn per hour).

## Decision
- `StorytellerSystem` (priority 170, interval 1,440, the first tick of a day) under `ColonyRule::story`
  (`StoryRule`; `None` in the baseline and in the M1–M4 colony rules). State `ColonyState.story`: the next
  check day, the last major day, the history `(day, kind)`, the planned and active pressures.
- Each season is planned on its first day with keyed draws (`STREAM_STORY`, subject = the year·4 + season):
  one **major** on a day drawn uniformly in the season's days 2–5 (0-based 1–4), its kind by season (winter ColdSnap
  600 / Beasts 400; summer Drought 600 / Beasts 400; spring Beasts 700 / none 300; autumn Beasts 700 / Illness 300),
  and in autumn and winter one **minor** Illness. A kind not seen for a year (ColdSnap) or three seasons (Beasts) is
  *due* and replaces the draw where its season and mercy allow (so every two-year world sees both).
- Mercy and gaps: no ColdSnap in the first winter's first two days; no Drought or Beasts before day 12; no Beasts
  within 6 days of the last Beasts nor in winter's last two days; no Illness before day 18; majors at least 6 days
  apart (a draw inside the gap moves to the gap's end if the season still has the day, else the season has none).
- Intensity scales with wealth: wolves = 2 + items on stockpile cells / 150, at most 4; a ColdSnap or Drought lasts one
  day more when the colony has 20 adults or more. Durations: ColdSnap 2–4 days, Drought 4–6, Beasts until the pack is
  dead or gone (2 days at most), Illness 3 days a person.
- Effects (each read where the behaviour lives, never in the storyteller):
  - **ColdSnap** (`FireSystem`): the night's burn ×2; a person marked cold outside a roofed room takes ×2 exposure
    damage; a roofed sleeper is sheltered only while the settlement's fire is lit (the M3 roof alone no longer
    suffices); outdoor work takes ×1,300 ‰ ticks (not for fur-cloak wearers).
  - **Drought** (`ResourceRegenSystem`, `ThirstDecaySystem`): forage regrowth ×400 ‰, thirst decay ×1,200 ‰; river
    and clay tiles unchanged.
  - **Beasts** (`AnimalSystem`, ADR 0028): a wolf pack spawns at the reachable map-edge tile nearest the largest
    forest and walks to the camp.
  - **Illness** (droppable): one adult `Sick { until }` for 3 days (work ×1,500 ‰, sleep need ×1,500 ‰), 20 % a night
    to a person sleeping in the same bedroom; never fatal by itself.
- Chronicle `PressureBegan { kind }` and `PressureEnded { kind, cost }` (deaths, items lost, work days lost); the
  HUD banner reads the active pressures; the causal event `PressureCause` (a cold hour, a wolf bite, an illness onset)
  makes "why?" on the person name the pressure.

## Alternatives considered
- A random daily chance per kind: no guarantee a two-year world meets both pressures; clustered events.
- A storyteller that reads the colony's weakness (RimWorld's adaptive director): out of scope; intensity scales with
  wealth only, as the goal says.

## Consequences
- Every pressure is a pure function of `(seed, day, colony wealth)`; a world without the rule never draws.
- New tallies and the probe's `m5` block measure each pressure's cost.

## Amendments (as built, M5 DECISIONS D7, D9, D14)
- Due rules: a cold snap is due after 24 days without one, beasts after 18 (`StoryRule::cold_due_days`,
  `beasts_due_days`); the soak's `--story no-beasts` turns beasts off (`StoryRule::beasts`).
- The firewood line's numbers are shared by the probe and the bridge: `story::firewood_have` (settlement wood +
  feedable firewood) and `story::firewood_day_need` (one night's burn at the snap's rate).
- Held-out result, final (`docs/v8/m5/M5_EVALUATION.md`): F1 32/32 (the schedule holds on every world), F2a 26/32
  (the snap bites), F2b 30/32 (two worlds lost most of their people in their first snap, with illness at the same
  time).

## Amendment M5.5 — no stacked pressure (as built; M5.5 goal §3.1.1, DECISIONS M5.5 D9)
- A minor pressure (illness) never begins while a major is active or within 2 days after a major ended: a planned
  illness whose day falls in that window waits for the first day out of it inside its season, else it is dropped
  (`Storyteller::skipped`).
- A major never begins while 2 or more people are sick: it waits day by day inside its season's window (the mercy
  and gap rules still apply), else it is dropped.
- Reason: M5's held-out seeds 804 and 814 lost 9 and 6 people when a cold snap met an illness on zero firewood; the
  owner decided pressures do not stack.

- As built: a held-back pressure waits a day at a time (`story::held_back`): an illness until its season's last day, a
  major until its window's last day; past that it is dropped (`skipped`). Seeds 0–7 × 48 days: 14 illnesses, none during
  or within 2 days after a major (M5's schedule had stacked 15).
## Date
2026-10-23

## Source
docs/v8/m5/M5_GOAL.md §3 and §4; docs/v8/m5/M5_DECISIONS.md
