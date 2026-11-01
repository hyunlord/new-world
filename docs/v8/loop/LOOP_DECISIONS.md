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

## LOOP-CAL-1: calibration round 1 — "§2.4 scope as specified" (rules 4+6 off)

**Variant.** Caps applied (LOOP-2) AND rules 4 (Production/Bills) and 6 (Teach)
removed from the default set (extended=true, steward_extended=false).

**Seeds 0–7, 48 days, steward on, no player orders.**

| Bar | Passed | Need | Status |
|-----|--------|------|--------|
| stockpile_day1 | 8/8 | 8 | PASS |
| beds_by_day10 | 0/8 | 6 | **FAIL (STOP)** |
| zero_cold_deaths | 8/8 | 7 | PASS |
| alive_day48 | 5/8 | 6 | FAIL |
| wolf_deaths_le1 | 8/8 | 7 | PASS* |
| exact_ledger | 8/8 | 8 | PASS |

\* wolf_deaths evaluated without mux2 observer; combat deaths 3–10 per seed.

**Root cause.** Without rule 4, no axes or blades are produced. Without axes,
tree chopping is too slow for house materials. Without blades, wolf attacks
kill 3–10 villagers per seed. Rule 3 (House) fires 39–89 times but
huts_built=0 in all 8 seeds. Seeds 2, 3, 5 have total extinction.

**Decision.** 0/8 bar triggers the §3 stop rule. Reported to lead.
No held-out run. Variant A (caps only, rules 4+6 in default set) and
variant B (narrow tools sub-rule) to be tested in rounds 2 and 3.

## LOOP-CAL-2: calibration round 2 — variant A (caps only, rules 4+6 default)

**Variant.** Log cap `cap_log_margin: Some(20)` applied. Rules 4 and 6 remain
in the default set (not extended). The `steward_extended` infrastructure exists
but is unused in this variant.

**Seeds 0–7, 48 days, steward on, no player orders.**

| Bar | Need | BASE | Var A |
|-----|------|------|-------|
| stockpile_day1 | 8 | 8/8 | 8/8 |
| zero_cold_deaths | 7 | 8/8 | 8/8 |
| alive_day48 | 6 | 8/8 | 8/8 |
| wolf_deaths_le1 | 7 | 8/8 | 7/8 |
| exact_ledger | 8 | 8/8 | 8/8 |
| logs_on_ground_sum | — | 2470 | **0** |

All bars pass. Log over-production eliminated (2470 → 0 total across 8 seeds).
Survival equivalent to BASE. Minor regression: seed 3 gains 2 combat deaths
(wolf_deaths 7/8 vs BASE 8/8) — within the bar's margin.

## LOOP-CAL-3: calibration round 3 — variant B (narrow tools sub-rule)

**Variant.** Rules 4 (full Production/Bills) and 6 (Teach) marked
`extended: true`. A new rule 9 (`LOOP_STEWARD_TOOLS`) fires by default:
`Bills(recipes: [(HaftAxe, 2), (KnapBlade, 4)], winter_recipe: SplitFirewood,
days: 3)` — tools only + 3-day firewood reserve.

**Seeds 0–7, 48 days, steward on, no player orders.**

| Bar | Need | BASE | Var A | Var B |
|-----|------|------|-------|-------|
| stockpile_day1 | 8 | 8/8 | 8/8 | 8/8 |
| zero_cold_deaths | 7 | 8/8 | 8/8 | 7/8 |
| alive_day48 | 6 | 8/8 | 8/8 | 7/8 |
| wolf_deaths_le1 | 7 | 8/8 | 7/8 | 7/8 |
| exact_ledger | 8 | 8/8 | 8/8 | 8/8 |
| logs_on_ground_sum | — | 2470 | 0 | 22 |

All bars technically pass but with less margin. Seed 3 has total extinction
(10 combat deaths), seed 7 has 3 cold deaths. The reduced firewood reserve
(days: 3 vs 12) and missing Teach lead to more winter exposure deaths.

**Conclusion.** Variant A is strictly better than variant B on survival.
The `steward_extended` setting and narrow-tools sub-rule are available for
future use when the card system replaces automatic production and teaching.

## LOOP-CAL-4: lead decision — instrument defect and variant B′

**Context.** The round-1 beds_by_day10 = 0/8 stop was an instrument defect, not
a regression: BASE is also 0/8 because the bar needs the MUX2 observer
(`--mux2-observe`). The O-2 stop is resolved by measuring correctly; the bar
is not withdrawn.

**Decision.** The brief's scope ("rules 4 and 6 leave the default") is binding
where it doesn't hurt survival. Variant B's rule 9 lowered firewood reserve to
3 days, causing seed 7's cold deaths and seed 3's extinction. Build variant B′:
B with the wood/firewood rule exactly as in BASE plus the cap (firewood ≤ winter
need × 1.25), and rule 9 covers tools only (axes/blades, no winter_recipe
beyond what rule 2 handles). Bills and teach are extended-only. Rerun A and B′
on 0–7 with `--mux2-observe`. Choose B′ if every bar is within one world of
BASE; otherwise choose A and record why.

## LOOP-CAL-5: calibration round 5 — A-80 and B′-80 (with MUX2)

Increased `cap_log_margin` from 20 to 80 to cover the initial house-material
burst (at tick 0 the Wood rule fires before House places blueprints, so
`building_need=0`; margin must front-load one house worth of logs).

| Bar | Need | BASE | A-80 | B′-80 |
|-----|------|------|------|-------|
| stockpile_day1 | 8 | 8/8 | 8/8 | 8/8 |
| walled_beds_day10 | 6 | 8/8 | 8/8 | 8/8 |
| zero_cold_deaths | 7 | 8/8 | 8/8 | 8/8 |
| alive_day48 | 6 | 8/8 | 8/8 | 8/8 |
| wolf_deaths_le1 | 7 | 8/8 | 8/8 | 7/8 |
| exact_ledger | 8 | 8/8 | 8/8 | 8/8 |
| logs_on_ground_sum | — | 2470 | 56 | 46 |

Both within one world of BASE on every bar. B′-80 is brief-compliant (rules
4+6 extended, narrow tools rule 9 default). **Chosen: B′-80.**

B′-80 held-out (1392–1423) result: walled_beds_day10 29/32 (need 30, BASE 30).
Within one world but below threshold. Per LOOP-CAL-4: choose A, record why.

## LOOP-D-steward: owner decision — grace window fallback

**Context.** Owner decision 2026-11-01 13:45.

**Decision.**
1. Tools that survival needs count as survival. B′ is approved as the target.
2. Unattended runs must stay intact. Cards are presentation-only. The sim
   provides fallback after a grace window of ≤ 1 in-game day:
   - **knowledge risk:** the steward issues the teach order itself after the
     grace window, if no player teach order addresses that knowledge.
   - **tools low:** the steward adds the bill after the same window, if no
     player bill exists.
   Grace window is data in `steward.ron` (`grace_ticks: Some(1440)`).
3. The soak, story, neighbours and steward fixtures (no player) must show
   0 KnowledgeLost. Add a 0-KnowledgeLost assertion. List changed fixtures.
4. B's 3-day firewood reserve stays reverted.

## LOOP-CAL-6: held-out 1392–1423 result (B′-80, BASE-relative)

B′-80 held-out judged by the BASE-relative rule (within one world of BASE):

| Bar | Need | BASE | B′-80 | diff | verdict |
|-----|------|------|-------|------|---------|
| stockpile_day1 | 32 | 32/32 | 32/32 | 0 | PASS |
| walled_beds_day10 | 30 | 30/32 | 29/32 | 1 | PASS (29 ≥ 30−1) |
| zero_cold_deaths | 28 | 32/32 | 32/32 | 0 | PASS |
| alive_day48 | 30 | 31/32 | 32/32 | −1 | PASS |
| wolf_deaths_le1 | 28 | 29/32 | 28/32 | 1 | PASS (28 ≥ 29−1) |
| exact_ledger | 32 | 32/32 | 32/32 | 0 | PASS |
| logs_on_ground | — | 7336 | 138 | | 98% reduction |
| snap_fw_zero | — | 0 | 0 | | PASS |

All bars pass under the BASE-relative rule. p1b was a misreading: it applied
the absolute threshold instead of the "within one world of BASE" criterion.

**cap_log_margin=80 rationale:** At tick 0, rule 2 (Wood) fires before rule 3
(House) places blueprints. With building_need=0, the cap target is just the
margin. A margin of 20 (the brief's value) produced only ~5 trees, insufficient
for the house-material burst. 80 covers one house (~25 walls × 3 logs = 75).

## LOOP-CAL-7: lead decision — cap floor and fresh held-out

**Decision (2026-11-01).** The cap floor: `max(building_need + 80,
days × daily_fuel)` ensures the early-game fuel reserve matches BASE even
before blueprints exist. Candidate = B′-grace + floor. Calibrate on 0–7,
freeze, then held-out on fresh seeds 1424–1455 (not used previously).
Record that 1392–1423 was used for A-80 and B′-80.

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
