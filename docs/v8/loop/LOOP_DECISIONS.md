# M-LOOP decisions

Citing: M-LOOP brief 2026-11-01.

## LOOP-1: 304 logs on day 5 — root cause

**Context.** Owner playtest seed 0 day 5 showed 304 stacked logs
(§2.4: "저렇게 나무 쌓여 있는 게 정상인가? 할 게 저거밖에 없어서?").

**Diagnosis.** Rule 2 (`MUX_STEWARD_RULE_2`, `WoodDeficit → Wood`) has no cap
on total log production.  Its demand formula is:

    need = days(12) × daily_fuel + building_need − (have + marked)

Three factors compound:

1. **building_need** includes undelivered materials from ALL active blueprints:
   the house rule (rule 3) places ~25 wall/door/bed blueprints each needing logs,
   creating ~60–80 log demand in one tick.  The bills/production rule (rule 4)
   adds workshop blueprints needing more.

2. **batch: 32** allows designating up to 32 trees per firing.  Each tree yields
   ~3 logs → up to 96 logs per firing.

3. **cooldown: 360** (6 game-hours) allows refiring ~4× per day.  The house rule
   also independently clears trees for its footprint, adding logs as a side
   effect.

Combined: the wood rule fires ~3–5× in 5 days, designating 100+ trees.  Logs
accumulate faster than people can haul them.  Rule 4 (production) and rule 3
(house clearing) contribute additional logs.  With 10 villagers, there is
insufficient labour to consume logs at the rate they are produced, leading to
the observed 304-log pile.

**Confirmation.** Probe seed 0, steward on, 5 days:

| Tick | Rule | Trees designated |
|------|------|-----------------|
| 0 | 2 | 32 (full batch) |
| 360 | 2 | 32 (full batch) |
| 720 | 2 | 31 |
| 1080 | 2 | 1 |
| 1440 | 2 | 3 |

Total: 99 trees felled, 390 logs created, 84 consumed, **306 on the ground at
day 5**.  Rule 2 is the sole source of tree designations.  The first three
firings (ticks 0–720, the first 12 game-hours) designate 95 trees at full
batch because `12 × daily_fuel + building_need` far exceeds available supply.
After tick 720, `have` catches up as logs land on the ground from completed
chops, and designations drop to single digits.

## LOOP-2: steward cap design (§2.4)

**Predicted effect (written before measuring):**

With caps applied:
- Fewer idle-chop hours because the wood rule stops designating once the cap is
  reached.
- Logs at day 24 ≤ target × 1.5 in ≥ 28/32 worlds.
- Survival rows (stockpile day 1, beds by day 10, cold deaths 0, alive day 48,
  wolf deaths ≤ 1, ledger) unchanged vs BASE within one world, because caps are
  set above survival minimums.
- The "집사가 더 많이" setting (off by default) restores rules 4 and 6, so
  existing behaviour is still reachable.

**Cap formulas (data in `steward.ron`):**

| Resource | Cap | Rationale |
|----------|-----|-----------|
| Logs | planned builds' log need + 20 | 20 = one batch margin for queueing |
| Firewood | winter need × 1.25 | The M6a arithmetic (#674) |
| Food | stop proactive harvest when usable stores + reserved production ≥ target; build a roofed store before harvesting into the open | Per #674 |

No rule designates or bills beyond its target.

## LOOP-3: "집사가 더 많이" setting

**Name:** `steward_extended` (bridge command / setting)
**Default:** off
**Effect:** enables steward rules 4 (Production/Bills) and 6 (Teach).
**Exposed as:** a bridge command `cmd_set_steward_extended(bool)` that the UI
agent can wire to a 설정 toggle.

## LOOP-4: text scale range (§1.2a)

**Context.** M-LOOP brief §1.2 — "today's look is effectively larger — measure
and say what it was". The existing settings_store offered 1.0 / 1.25 / 1.5.
At 1920×1080, the 1.0× baseline already appears large because the topbar,
panels, and lesson cards use 1.0× which Godot scales at the viewport's native
DPI. The effective visual scale was approximately 1.0× at 1920×1080 — i.e. no
multiplier was in effect, but the font sizes and panel sizes were designed at
a scale suitable for 1080p.

**Decision.** Replace the scale ladder with 0.85 / 1.0 / 1.15 / 1.25. The
old 1.5× value is removed (it covered too much screen at 1080p). 0.85× gives
a denser view for experienced players. Default stays 1.0.

## LOOP-5: card state is presentation only (§2.2)

**Context.** Decision cards map simulation events to player options. They must
not affect the simulation hash or save files.

**Decision.** Card state (open/answered/expired) is tracked in the GDScript
card_panel.gd only — it is not saved, serialised, or hashed. After a save/load,
cards are rebuilt from the events of the last in-game day. This means a card
answered just before saving may reappear after loading; the limit is stated in
the report. Answering a card sends ordinary bridge commands that enter the
command log with source Player, so replay remains valid.

## LOOP-6: chronicle curation split (§2.5)

**Context.** The owner's playtest complaint #1: "연대기는 너무 단순하고 반복적인
내용들만 많은데?" The chronicle_feed.json kind_visibility (schema 1) allowed
per-kind on/off but had no concept of "notable" vs "routine".

**Decision.** Schema 2 adds `notable_kinds` and `routine_kinds` arrays.
Notable kinds are shown by default: Born, Died, Paired, CameOfAge,
BecameElder, Widowed, Quarrel, RoomFormed, KnowledgeDiscovered/AtRisk/Lost,
PressureBegan/Ended, Fight, Breakdown, Drafted, WolfKilled, MigrantsArrived/
Joined/Declined, TraderArrived, Traded, Eureka, StewardDefence. Routine kinds
(Hunted, FoodOperation, StewardAction, PorterDeparted/Delivered,
HungerFoodChain, MindChoice) are behind the "전부" toggle. The "사람별" filter
(existing M5 feature) is preserved.

## LOOP-7: chapter data structure (§2.3)

**Context.** The goals.json flat list does not express year-based progression.

**Decision.** chapters.json replaces the goal grouping with an ordered list of
5 chapters. Each chapter has conditions checked against existing bridge-read
facts. Chapter 4 ("첫 밭과 천 옷") is locked with text "M6b에서 열립니다".
Chapter 5 is the long-term horizon (era ≥ 600‰, population ≥ 40). The
goals.json file is kept for backward compatibility with learning_facts.gd.
