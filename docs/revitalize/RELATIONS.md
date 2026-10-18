# Villages that help and villages that quarrel (Pass 20, `game_v25`)

Until Pass 19 a world's one to three villages — a camp and the daughters its splits founded 22–28 tiles away — were kin but never met: nobody carried anything to another village and nothing recorded how villages felt about each other. They touched in one place only: a daughter's foragers can reach some of their parent's bushes, and bush claims are world-wide. `game_v25` = `game_v24` + `SocietyRules::relations` (`RelationsRule`, `WorldRules::V25_RELATIONS`). Every earlier scene has `relations: None` and is bit-identical (identity I1–I8, all pins). The live game starts in `game_v25`.

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
- Two villages whose foragers keep meeting at the same bushes quarrel; when it gets bad they stop sending anything and keep to their own side of the land between them, until time heals it.
- The chronicle (C key) gets `PorterDeparted`, `PorterDelivered`, `Quarrel`, `VillagesEstranged` and `VillagesReconciled` entries ([CHRONICLE.md](CHRONICLE.md), kinds 17–21). Their wording and the porter's look are the next art pass's.
- The bridge: `get_settlement_relations()` → `{a, b, value, estranged, base}` in key order; `get_porter_snapshot()` → `{ids (entity bits), from, to, kind (0 food / 1 wood), amount, stage (0 out / 1 back)}` by AgentId. Until the art line reads it, a porter shows in the existing snapshots as `Seeking { Agent(itself) }`.

## Measured (Pass 20)

`game_v25` vs `game_v24`, seeds 19400–19431, 120 days (19 of the 32 worlds had ≥ 2 villages at once):

- **Porters:** 38 trips, 71 porters, all delivered (no turn-back, no lost load): 634 food and 34 wood moved; 67 porters on kin food aid, 4 on wood, exchange never (see above). 9 of the 19 multi-village worlds saw a delivery (T1's bar is 0.6: **missed**). Two things keep aid rare. The store target keeps every village between about 1.5 and 6.5 food days, so a rich giver and a short receiver seldom coincide (the tuned food-aid condition held on 214 of 2,642 in-reach pair-days); and **estrangement blocked 176 of those 214** — only 38 fell between villages still on speaking terms, and 36 of them became food trips (the other two had nothing above the keep line or nobody free); the remaining 2 of the 38 trips were wood aid.
- **Does aid save lives?** In the villages that received aid, non-old-age deaths were 0.44 fewer per village than in the same region's village under `game_v24` (9 of 9 matched; interval [−1.33, 0.00]). But daughter villages as a whole lost **more** people (26 vs 6 non-old-age deaths over the 19 worlds, T2a **missed**), almost all in one world (19414: a daughter estranged from the camp on day 24 lost 19 to exposure and 5 to starvation against 5 in `game_v24`). Daughters alive at day 118: 24 in both.
- **Conflict:** disputes and quarrels in 13 of 19 multi-village worlds (1,834 disputes, 511 quarrels), 14 estrangements, **no reconciliation**; at day 120, 14 of the 32 living pairs were estranged (0.438 ≤ 0.5, T3 passes). 426 of the 511 quarrels happened after the pair was already estranged: the opportunistic Gather arm (pick up any food within 8 tiles) does not consult the avoid list, so foragers near the border keep meeting. Applying it there (a temporary build, four worlds) cut those quarrels from 52 / 73 / 94 / 74 to 1 / 21 / 7 / 31 — the first fix for the next pass.
- **Health:** alive, non-old-age deaths, shelters, food and wood stores, social rate, F1 and F5 all within Pass 17's bars (T4 passes). **Cost:** +2.7 % (stress), ×1.045 (natural multi-village worlds) (T6 passes).
- **Accounting:** every departure, delivery, quarrel and estrangement has exactly one chronicle entry; every trip balances (T7 passes).


See METRICS.md `## Pass 20` (rules), [diag](probe/p20-2_diag/diag.md) (7.1), [tuning](probe/p20-2_diag/tuning.md) (7.3), [eval](probe/p20-2_eval/eval.md) (T1–T4, T7) and [t6](perf/p20-2/t6.md) (performance).
