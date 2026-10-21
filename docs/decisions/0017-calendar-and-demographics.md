# ADR 0017 — The colony calendar: 6-day seasons, a 24-day year, ages in years, births by a fertility rate

## Context
ADR 0004 fixed the tick pacing (12 ticks/s at 1×, a 2-minute game day) and left the calendar to M1.5. The colony
inherited V7's calendar and its demography: 7-day seasons, ages counted in days (a founder "lived 54 days"),
childhood of 4 days, and `BirthRule::Provisioned`, which gives a settlement a newborn every 720 ticks whenever it is
fed — 10 founders became 29 people in 10 days on every seed (M1 evaluation). A long colony game needs years that
pass at a readable pace and a population that grows slowly, mostly from migrants (ADR 0006), not from a birth
cooldown.

## Decision
- **One calendar rule.** `WorldRules.calendar: CalendarRule { ticks_per_day, days_per_season, seasons_per_year }`
  in both worlds; every derived length is a method (`ticks_per_year`, `years_to_ticks`, `days_to_ticks`,
  `whole_years`, `date_of_day`), never a literal. The baseline keeps 7-day seasons (bit-identical, identity gate);
  the colony has **6-day seasons and a 24-day year** (34,560 ticks; at 1× a year is 48 minutes). Season readers take
  the day count from the calendar; `SeasonRule::days_per_season` must equal it.
- **Ages in years**, all from `Life::born_tick` (closed form, nothing increments): the colony's elders at 50,
  lifespans 62 ± 8 years (the mean of three keyed uniforms, clamped to 40–85), founders 16–40 with at least 10
  years left, children until 15, infants until 3 (a flag derived from the age; infants and children keep the V7
  child behaviour: no foraging, no jobs, tethered at the hearth). The UI shows `3년 차 · 봄 2일째` and ages as
  "세".
- **Births by a fertility rate** (`BirthRule::Fertility`, colony only; the baseline keeps `Provisioned`).
  `FertilitySystem` (priority 150, daily at 12:00) gives each woman of 16–44 with a living partner, not pregnant and
  48 days past her last birth, in a fed settlement (members' mean hunger and thirst ≤ 60), a keyed daily conception
  chance of 33 ‰ (key: lineage, day). A pregnancy (`Pregnant { conceived_tick, due_tick, father }`) lasts 18 days
  (9 of 12 "months" of a 24-day year); the birth goes through the settlement's existing birth path (the child is
  spawned at the hearth with both parents in `Kin`, so it belongs to her household), taking up to 10 food.

## The demographic reasoning
With 18 days of gestation and 48 days after a birth, a woman can be pregnant again on average every
18 + 49 + (1 − p)/p days; at p = 33 ‰ that is 96 days = 4.0 colony years, i.e. **~0.25 births per woman-year**.
A colony of 10 founders has about four fertile, paired women, so about one birth a year: roughly **3 % growth a
year** without migration, and no child-mortality model yet (M5). Growth from 10 towards 100 people therefore depends
on migrants, as ADR 0006 intends; births alone add one or two people a year and keep the colony from dying out.
The rate the goal wrote (12 ‰) gives 0.16 a year once the 66 non-eligible days are counted, below the 0.25 the goal
names; M1.5 DECISIONS D6 records the change to 33 ‰. The system runs at midday because at 06:00 the members' mean
thirst is at its overnight peak and the fed gate would fail on half the mornings (D7).

## Alternatives considered
- Keep the provisioned birth with a long cooldown: still a settlement-level birth at a fixed pace, unrelated to
  who could be a mother; no pregnancy, no household.
- A 4-day season (16-day year): years pass faster but a season is too short to read on screen; 6 days keeps a
  season at 12 minutes of play.
- Stochastic lifespans with a mortality curve: needs child mortality and illness (M5) to mean anything.
- A conception draw per tick: per-tick polling of every woman; one daily draw is the same model at 1/1,440 of the work.

## Consequences
- Every duration in the rules is audited against the calendar (M1.5 DECISIONS D5): values meaning "a season" or
  "a winter" come from it (the colony's autumn now stores a whole 6-day winter of firewood), values meaning a day
  stay.
- `RuntimeSystem::tick_offset` lets a daily system run at a clock time; the default (0) keeps every other system.
- The colony's demographic bars (M1.5 D1–D5) are measured over 3 colony years without orders and a 17-year run for
  coming of age.
- Child mortality, illness and migrants are M5; the population after 3 years is 11–15 on the calibration seeds.

## Date
2026-10-21

## Source
docs/v8/m1_5/M1_5_GOAL.md §5; docs/v8/m1_5/M1_5_DECISIONS.md D4–D9; docs/decisions/0004-compressed-year.md; docs/decisions/0006-neighbours-as-knowledge-channel.md
