# Villages that help and villages that quarrel (Pass 20, `game_v25`)

Until Pass 19 a world's one to three villages — a camp and the daughters its splits founded 22–28 tiles away — were kin but never met: nobody carried anything to another village and nothing recorded how villages felt about each other. They touched in one place only: a daughter's foragers can reach some of their parent's bushes, and bush claims are world-wide. `game_v25` = `game_v24` + `SocietyRules::relations` (`RelationsRule`, `WorldRules::V25_RELATIONS`). Every earlier scene has `relations: None` and is unchanged: identity I1–I8 (probe summaries, the recorder's arrays, all pins) is identical on the Part 1 and final builds, and the `--chronicle-out` files are identical in content (their header's kind list grows from 17 to 22 names). The live game starts in `game_v25`.

Fights, raids and deaths between villages are not part of v1.

## The relation

Every unordered pair of living villages has a `RelationState { value, estranged, base }` in `SimResources::settlement_relations.pairs` (lower id first). `RelationsSystem` (priority 149, every tick, after `FissionSystem` at 148, so a daughter's `parent` is already set on its founding tick) adds a missing pair at its **kin base** and drops a pair whose village is gone:

| pair | base | why |
|---|---|---|
| a daughter and its parent | **300** | the daughter's founders grew up at the parent's hearth; most of them have kin there |
| two daughters of one parent | **150** | cousins: shared origin, no shared hearth |
| any other pair | **0** | strangers (no such pair in today's worlds: every village descends from the one camp) |

The value lives on −1,000..=1,000 (every change is clamped).

- **Drift.** Once a day, at the 08:00 decision, `value += (base − value) × 10 / 1000` (1 % of the gap, integer). A grudge fades over months, not days: from −400 to the reconcile line −150 toward a parent base of 300 takes 49 days (almost two years of 28 days).
- **Delivery** (`+40`, once per trip, credited when its first porter delivers): help is remembered; four deliveries undo a quarrel.
- **Forage dispute** (`−10`): a forager of one village picks up at a bush another village's forager is at or walking to (see below).
- **Quarrel** (`−100` more): a dispute that turns into words.
- **Estrangement** (hysteresis, evaluated **only at the 08:00 decision, after the drift**): estranged at value ≤ **−300**, reconciled at ≥ **−150**. The 150-point gap keeps a pair from flapping from one day to the next. Estranged villages send no porters to each other, and each one's foragers skip every bush nearer (Chebyshev) to the other's hearth than to their own — the border is the line of equal distance; a bush exactly on it stays shared.

## Porters

At **08:00** (`decision_minute` 480) each village is measured once:

- **food days** = store food × 1,000,000 / max(1, `need_milli`) in permille of a day, where `need_milli` = ⌊the members' daily food need × 1000⌋ (`members_need_per_day`, 2.4 units per adult); food **keep** = ⌈need × `aid_keep_days_permille` 3,000 / 1,000,000⌉ (three days);
- **wood nights** = store wood / tonight's burn (`tonight_burn`, 13–52 units a night by season and severity); wood keep = 2 nights.

A pair is **in reach** when the walk between the two hearths (Chebyshev tiles × 8 ticks per step) takes at most three quarters of the 08:00–15:00 window (315 ticks: 39 tiles). Every daughter is 22–28 tiles from the camp, so parent–daughter pairs are always in reach; two daughters on opposite sides of the camp (up to ≈ 56 tiles) may not be.

Then the ordered pairs (sender g, receiver r) of non-estranged, in-reach villages are taken in id order, at most one trip per ordered pair per day and at most **2 porters per sender per day**:

1. **Kin aid.** Food when g has ≥ **3,000 ‰** food days and r < **2,000 ‰** (tuned from the brief's 4,000 / 1,500, see below); otherwise wood when g has ≥ 3 nights and r < 1 night. Amount = min(10 × the sender's remaining porter slots, the store above the keep line, 10 × its available carriers); porters = ⌈amount / 10⌉ (a porter carries at most ten units — the inventory cap).
2. **Exchange** (only if the pair's aid sent nothing; both trips or none, using both ordered pairs' slots): g sends food and r sends wood when g has ≥ 3,000 ‰ food days but < 2 wood nights and r has ≥ 4 wood nights but < 2,000 ‰ food days. **With the tuned aid thresholds the exchange's food condition is exactly the kin-aid food condition, which is tried first with the same amount, so exchange cannot happen in `game_v25`.** It never happened in the diagnostic with the brief's thresholds either; the mechanism is kept and tested.

The conditions read the 08:00 measures; the amounts read the store as debited by earlier departures that day, so a village never sends below its keep line.

**Who carries.** Adults (no `Child`, no `Elder`) who are not walking to found a daughter or already carrying, empty-handed, hunger below 30, and Idle or in a routine activity — most reward-dependent first (RD descending, then AgentId): the sociable go. An activity is interrupted (the routine records it as ended early).

**The trip.** At departure the load moves from the sender's store into the porters' hands and each gets `PorterTrip { from, to, kind, amount, stage, depart_tick }`. A porter walks like a founder: `Seeking { Agent(itself) }` with its target at the hearth of its stage. Its needs still come first (a hungry porter eats, then walks on); it hauls nothing home, follows no tether, is never marked `Cold` at its own hearth, never leaves with a fission group, and the stockpile deposit leaves its load alone. Within 5 tiles of the receiver's hearth the load goes into the receiver's store and the porter turns home; at home the marker is removed. A porter not at the receiver 420 ticks (until 15:00) after departing, or whose receiver dissolved, **turns back** with its load (a failed trip) and puts it back in its own store. A porter who dies, or whose village dissolves, **loses** its load. Every trip is booked (`TripRecord`): withdrawn = delivered + returned + lost. Porters stay members of their own village throughout.

Why these numbers: three days is the store's own comfort (the store target is 2.5 days and grows before winter), so a giver keeps what it needs; ten units is what one person can carry; two porters cap what a village gives away in a day and keep most hands at home; 08:00 is after breakfast (the hungry are fed and the night is over), and 15:00 leaves time to walk home before dark.

## Disputes and quarrels

Before the decision loop each tick, the system notes every forager at a bush (picking up) or walking to one, with its village. When a forager picks up at a bush where a forager of another village is (the lowest-AgentId such rival), the two villages **dispute** it (counted, not chronicled) — each pair of foragers at most once a day. A dispute turns into a **quarrel** by a keyed draw (`STREAM_RELATIONS`, subject = the lineage key of the lower-AgentId forager): p = clamp((NS_a + NS_b − HA_a − HA_b + 1000) / 4, 0, 500) in thousandths — bold, novelty-seeking pairs quarrel, cautious ones step back (250 ‰ for two average people). In a quarrel neither picks up that tick.

## What the player sees

- Someone walks out of a village at 08:00 with a load and comes back in the afternoon; the poorer village's store grows (food), or its woodpile (wood).
- Two villages whose foragers keep meeting at the same bushes quarrel; when it gets bad (relation ≤ −300) they stop sending anything and their foraging trips keep to their own side of the land between them, until the relation climbs back to −150. **In the evaluated worlds this did not separate them:** the opportunistic pick-up of food within 8 tiles ignores the border, most quarrels came after the estrangement, the relation stayed near −1,000, and no pair reconciled (from −1,000 the drift alone needs 114 days to reach −150). See *Measured*.
- The chronicle (C key) gets `PorterDeparted`, `PorterDelivered`, `Quarrel`, `VillagesEstranged` and `VillagesReconciled` entries ([CHRONICLE.md](CHRONICLE.md), kinds 17–21). Their wording and the porter's look are the next art pass's.
- The bridge: `get_settlement_relations()` → `{a, b, value, estranged, base}` in key order; `get_porter_snapshot()` → `{ids (entity bits), from, to, kind (0 food / 1 wood), amount, stage (0 out / 1 back)}` by AgentId. Until the art line reads it, a porter shows in the existing snapshots as `Seeking { Agent(itself) }`. `amount` is the load taken at departure: on the way back (`stage` 1) after a delivery the porter's hands are empty, so draw the load only while `stage` is 0 (or after a turn-back, when the load comes home).

## Measured (Pass 20)

`game_v25` vs `game_v24`, seeds 19400–19431, 120 days (19 of the 32 worlds had ≥ 2 villages at once):

- **Porters:** 38 trips, 71 porters, all delivered (no turn-back, no lost load): 634 food and 34 wood moved; 67 porters on kin food aid, 4 on wood, exchange never (see above). 9 of the 19 multi-village worlds saw a delivery (T1's bar is 0.6: **missed**). Two things keep aid rare. The store target keeps every village between about 1.5 and 6.5 food days, so a rich giver and a short receiver seldom coincide (the tuned food-aid condition held on 214 of 2,642 in-reach pair-days); and **estrangement blocked 176 of those 214** — only 38 fell between villages still on speaking terms, and 36 of them became food trips (the other two had nothing above the keep line or nobody free); the remaining 2 of the 38 trips were wood aid.
- **Does aid save lives?** In the villages that received aid, non-old-age deaths were 0.44 fewer per village than in the same region's village under `game_v24` (9 of 9 matched; interval [−1.33, 0.00]). But daughter villages as a whole lost **more** people (26 vs 6 non-old-age deaths over the 19 worlds, T2a **missed**), almost all in one world (19414: a daughter estranged from the camp on day 24 lost 19 to exposure and 5 to starvation against 5 in `game_v24`). Daughters alive at day 118: 24 in both.
- **Conflict:** disputes and quarrels in 13 of 19 multi-village worlds (1,834 disputes, 511 quarrels), 14 estrangements, **no reconciliation**; at day 120, 14 of the 32 living pairs were estranged (0.438 ≤ 0.5, T3 passes). 426 of the 511 quarrels happened after the pair was already estranged: the opportunistic Gather arm (pick up any food within 8 tiles) does not consult the avoid list, so foragers near the border keep meeting. Applying it there (a temporary build, four worlds) cut those quarrels from 52 / 73 / 94 / 74 to 1 / 21 / 7 / 31 — the first fix for the next pass.
- **Health:** alive, non-old-age deaths, shelters, food and wood stores, social rate, F1 and F5 all within Pass 17's bars (T4 passes). **Cost:** +2.7 % (stress), ×1.045 (natural multi-village worlds) (T6 passes).
- **Accounting:** every departure, delivery, quarrel and estrangement has exactly one chronicle entry; every trip balances (T7 passes).


See METRICS.md `## Pass 20` (rules), [diag](probe/p20-2_diag/diag.md) (7.1), [tuning](probe/p20-2_diag/tuning.md) (7.3), [eval](probe/p20-2_eval/eval.md) (T1–T4, T7) and [t6](perf/p20-2/t6.md) (performance).


## Pass 21 — feuds that end and help that arrives (`game_v26`)

Pass 20's mechanics were exact but its balance was wrong: estranged villages never separated (the opportunistic Gather arm ignored the border, so 426 of 511 quarrels came after estrangement), a pair could lose hundreds of points a day and recover at 1 % a day (no reconciliation in 32 worlds), and aid was rare (most qualifying pair-days fell between estranged villages; exchange could not happen). `game_v26` = `game_v25` with `V26_RELATIONS`; no new mechanic, no new kind, component, target or RNG stream. `game_v25` and every older scene are unchanged (identity I1–I9 and all pins). The live game starts in `game_v26`.

### What changed, and why each number

| field | v25 | v26 | why |
|---|---|---|---|
| `avoid_gather` | false | **true** | the Gather arm's food-tile choice (`nearest_gather_tile`), its existence check (`gather_tile_exists`, the routine's exit predicate) and both post-pass re-resolutions skip a tile nearer to an estranged village's hearth than to the own one — the same border test as the forage bush choice. With it every selector that can give a forager a food target respects estrangement. The exit predicate's memo now keys the avoid list's owner (the member's village), so two members of different villages on one tile cannot share an answer |
| `dispute_cap_per_pair_day` | 0 (none) | **3** | a village pair pays `dispute_loss` for at most 3 disputes a display day; further disputes are still counted. A pair whose foragers meet six times a day (measured) no longer loses 60+ points a day from disputes |
| `quarrel_cap_per_pair_day` | 0 (none) | **1** | at most one quarrel per pair and day: once reached, a dispute draws no quarrel and the pickup proceeds (the keyed draw is skipped; it consumes no state) |
| `exchange_first` | false | **true** | the pair's exchange is tried before either village's aid (in whichever orientation meets the exchange condition); with aid first, the tuned aid thresholds shadowed the exchange completely |
| `dispute_loss` | 10 | **5** | with the caps a bad day costs at most 3 × 5 + 1 × 40 = 55 |
| `quarrel_loss` | 100 | **40** | the grid (below) |
| `relation_drift_permille` | 10 | **30** | 3 % of the gap a day: from −300 to −150 toward a parent base of 300 in ≈ 11 days, from −1,000 in ≈ 35 days (≈ 23 from −600). Integer truncation stops the drift within 33 of the base (33 × 30 < 1,000) |
| `aid_giver_days_permille` | 3,000 | **2,500** | the grid (below) |
| `aid_keep_days_permille` | 3,000 | **2,000** | below the giver line, so a giver always has at least half a day's need above its keep line (in v25 the keep line equalled the giver line and the surplus was often 0–10 units) |
| `aid_receiver_days_permille` | 2,000 | 2,000 | the grid kept it |

Ledgers: `SettlementRelations::day_counts` (per pair: costed disputes and quarrels today, cleared with the once-per-day forager set when the display day changes) and the measurement-only `dispute_days` (disputes and quarrels per display day and pair, written under every relations rule, never read by the simulation; `--chronicle-out` writes it as `relations_<seed>.json`). New tallies: `disputes_uncosted`, `quarrels_capped`, `quarrels_while_estranged`.

### The grid

Quarrel loss, drift and the two aid thresholds were not guessed: twelve configurations were pre-registered in the brief and run on 19500–19515 for 120 days ([grid](probe/p21-2_grid/grid.md)). The rule: among configurations with ≤ 0.25 of pairs estranged at day 120, quarrels in ≥ 0.15 of multi-village worlds and (with ≥ 3 estrangements) ≥ 0.5 reconciled, the highest share of multi-village worlds with a delivery, ties to the earlier. Eight qualified; six reached the top share (5 of 7 worlds); the earliest, **G2** (quarrel loss 40, drift 30 ‰, giver 2,500 ‰, receiver 2,000 ‰), was frozen. Slower drift (20 ‰) failed the reconciliation bar; a larger quarrel loss (60) estranged more pairs without reaching more worlds.

### The measured λ against the planner's model

The planner modelled a pair with λ disputes a day (0.5–2), a 0.28 quarrel share and 5 % contact while estranged. The diagnostic ([diag](probe/p21-2_diag/diag.md), 19500–19515) measured per pair and day from the dispute log: **λ is bimodal** — 16 of 20 pairs never disputed; the disputing pairs are camp–daughter pairs whose catchments overlap, at 0.35, 1.5, 6.0 and 6.1 disputes per pair-day (on the evaluation seeds the pooled λ before estrangement is 1.45). Contact while estranged: **3.4×** the earlier rate under `game_v25` (the leak), **0.135×** under `game_v26` (0.20× on the evaluation seeds), not 5 %. The quarrel share of disputes is 0.29 without the cap (as modelled) and 0.11 with it. Replayed with each pair's measured λ and contact, the model predicts 0.23 estrangements per pair against 0.21 measured, and the heavy pairs cycle between feud and reconciliation as it predicted (19509: estranged on days 62, 83, 108, reconciled on days 73, 95, 119). The mechanism holds; the assumed inputs did not.

### What the player sees now

- Two villages that fall out keep apart: foragers no longer cross to the other side of the line between the hearths, whether foraging on purpose or picking up food on the way. Quarrels between estranged villages fall from 81–89 % of all quarrels (`game_v25`, diagnostic and evaluation seeds) to 10–16 %.
- Feuds end: in the evaluation 8 of 21 estrangements were reconciled within the 120 days (13 of 22 on Pass 20's seeds), after 10–15 days in 19 of 21 cases (the other two: 29 and 39 days). Pairs estranged at day 120 are mostly in the recovery band (between −300 and −150) — 7 of the 13.
- Porters are more frequent and carry more: 13 of 19 multi-village worlds saw a delivery (`game_v25` 11), 89 delivered trips (170 porters, 1,510 food and 157 wood), none turned back.
- Exchange still never happened: no village had ≥ 4 nights of wood beside a partner short of both wood and food at once.
- The chronicle now contains `VillagesReconciled` entries ([CHRONICLE.md](CHRONICLE.md)).

### Measured (Pass 21)

`game_v26` against `game_v25` and `game_v24`, seeds 19600–19631, 120 days (19 multi-village worlds), registered rules in METRICS.md `## Pass 21` ([eval](probe/p21-2_eval/eval.md), [u6](perf/p21-2/u6.md), [comparison on 19400–19431](probe/p21-2_compare/compare.md)):

- **U1 aid: PASS** — 13 of 19 multi-village worlds deliver (0.684); failed share 0 of 170 porters.
- **U2 exchange: FAIL** — 0 exchange trips; the exchange condition held on 0 pair-days.
- **U3 conflict that heals: FAIL** — disputes in 0.579 and quarrels in 0.526 of multi-village worlds, quarrels while estranged 51 of 322 (0.158) pass; estranged at day 120 13 of 45 pairs (0.289 > 0.25) and reconciled 8 of 21 (0.381 < 0.5) miss.
- **U4 health: FAIL** on one bar — every Pass 17 R2 rule and F1 / F5 pass; daughter non-old-age deaths per multi-village world `game_v26` − `game_v24` +0.158 [−0.158, 0.474] (12 vs 9; > 0, miss) and − `game_v25` −0.105 [−0.737, 0.368] (12 vs 14, pass); daughters alive at day 118 30 vs 29 (pass).
- **U6 performance: FAIL** on one bar — stress mean +2.2 %, p99.9 +3.6 %, slope 0.986, natural mean ×1.074 pass; natural p99.9 ×1.400 > 1.35 (the statistic is dominated by single 20–174 ms ticks in every rule; per-repeat ratios ×1.02, ×1.49, ×0.52).
- **U5 identity and U7 accounting**: see the report.
