# ADR 0030 — Mood, quarrels and chronicle causes

Status: draft (M5 Part 0, 2026-10-23); final in M5 Part 7.

## Context
V7's third signature is the person detail, the chronicle and "why?". Temperament, family and relations existed but
nothing in the colony drove them; people never quarrelled. The goal (3.4) asks that "who quarrelled with whom and why"
can be followed.

## Decision
- **Mood** (`ColonyRule::mood`, `MoodRule`; `MoodSystem` 172, every 60 ticks): `Mood { value: 0..=1000 }` on adults and
  children of 10 years or more. Each interval the value moves 1/8 of the way to a target = 500 + Σ modifiers (slept in a
  roofed bedroom +80, slept outside −120, fed +40 / hungry −150, thirsty −100, cold last night −120, quarrelled today
  −100, taught or learned today +60, discovered something +150 for 3 days, a kin died −250 for 6 days, sick −80, worked
  a gated craft as its practitioner today +40, drafted a day or more −60, a pressure active −40). Harm avoidance scales
  the negative modifiers ×(800 + ha / 5) ‰ and reward dependence the social ones.
- **Thresholds**: below 250 *tantrum risk* (state word 우울): quarrel chance ×3, work ×1,200 ‰ ticks; below 100
  *breakdown*: the person refuses jobs for a day and wanders (chronicle `Breakdown` with the three largest modifiers);
  750 or more 흡족: work ×900 ‰.
- **Quarrels** (`QuarrelSystem` 173, every 30 ticks; the V7 `SocialInteractionSystem` has no quarrel and stays
  untouched): two people within 2 tiles, not kin-close (a kin base ≥ 300: parent, child, sibling, partner), with a
  person-pair relation below 0 or either mood below 250, quarrel with chance
  `2 + max(0, 250 − min mood) / 25 + max(0, −relation) / 100` ‰ (×3 under tantrum risk, at most 40 ‰), keyed on
  `STREAM_QUARREL`. A quarrel: relation −60 × NS / 500 of the initiator (the lower mood), hostility +0.1, mood −100 for
  both, chronicle `Quarrel { a, b, cause }` with the initiator's largest negative modifier as the cause (hunger, cold,
  sleep, grief, pressure, relation). Hostility ≥ 0.6 and both moods below 250 → one fight through `CombatSystem`
  (damage 4, one resolved hit; chronicle `Fight`).
- **Person-pair relations** (`ColonyState.bonds`): value −1,000..1,000 that drifts back to its kin base by 10 a day; the
  kin base is 500 for kin-close pairs, else 0. (V7's `RelationState` is a village relation; the colony has one
  village, so a person-pair relation is new.)
- **Chronicle causes**: a side table `Chronicle::causes` maps an entry to a `CauseRef` — another entry, a pressure or a
  mood modifier (with the entry that started it when there is one) — so the panel can follow "→ 왜?" chains. New kinds
  29–37: `PressureBegan`, `PressureEnded`, `Hunted`, `Fight`, `Breakdown`, `Drafted`, `WolfKilled`, `Sick`,
  `Recovered`; `Quarrel` (19) is reused with cause = 1 + the M5 cause code (0 stays V7's forager quarrel). A year
  summary entry (droppable 9.5) and the person's 이야기 tab read the same table.

## Alternatives considered
- Mood as a sum recomputed on read: modifiers need memory of the night (sleep, cold) and of events (grief, discovery).
- A cause field on every entry: changes the baseline chronicle's entry size, which identity pins (D6).

## Consequences
- Mood, its modifier days and the person-pair relations add ≤ 24 B a person (goal §4).
- The decision system is not touched by mood: breakdown and tantrum act through the job layer's assignment and work
  ticks.

## Date
2026-10-23

## Source
docs/v8/m5/M5_GOAL.md §3 and §4; docs/v8/m5/M5_DECISIONS.md
