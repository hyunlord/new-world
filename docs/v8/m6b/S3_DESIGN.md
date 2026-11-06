# Stage3 — physical temperature and dwelling progression

Implementation starts from published Stage2 `90544f1f9`, at2026-11-07 04:02:56KST. The24-hour deadline is2026-11-08 04:02:56KST. [ADR0060](../../decisions/0060-physical-room-temperature-and-dwellings.md) fixes the mechanical boundaries. This is starting arithmetic, not a calibration or outcome target.

## Hourly integer model

For an authoritative roofed room:

`next = floor((capacity * previous + loss * outdoors + paid_heat + body_heat) / (capacity + loss))`.

Capacity is750 per interior tile. Roof/floor loss is100 per tile; a material edge loses `1000 - insulation_permille`, with oak900 giving100. A closed door contributes250; a fully open door9000. Actual passage opens it for2ticks, and overlapping intervals count once. Partial-hour extra door loss rounds upward. All boundaries initially lose to outdoors, including a shared wall; there is no neighboring-room energy exchange.

Season bases are `[120,220,100,-30]` deci-degrees. The authored24-hour offsets range−20..40; winter night is therefore−50, and a snap adds−50 to give−100. Rated primary-hearth heat450000 requires the existing positive, fully paid fuel withdrawal. Each actual adult adds4500, child2500. Worn insulation adds20 for hide wrap,50 for fur cloak,30 for linen tunic and20 for linen wrap. Felt below50 derives Cold; equality is warm.

## Reference arithmetic

A3×3interior has capacity6750. Eleven oak-facing edges, one closed door and nine floor/roof cells give loss2250. Two adults and a paid hearth give459000 heat units. Linen tunic adds30 to place temperature.

| Case | Initial place | Next eight hourly place temperatures | Scope |
|---|---:|---|---|
| Closed, paid winter night | 30 | 61,84,101,114,124,131,136,140 | Warm-start illustration |
| Closed, paid snap night | −100 | −49,−11,17,38,54,66,75,82 | New rooms start outdoors; startup is real |
| Closed, unlit winter night | 30 | 11,−4,−15,−23,−29,−34,−37,−40 | Bodies do not replace paid fuel |

The closed winter equilibrium before rounding is154; snap equilibrium104. A held-open door changes loss to11000 and winter equilibrium to−8.27. These numbers do not assert natural population comfort or a success percentage. Larger rooms, sparse occupation, material choice and fuel interruptions change the result. No parameter is adjusted to fit smoke outcomes.

Current construction starting costs for the reference5×5outer shell are15log walls plus one door:31logs/630base workticks. Two bedrolls add2logs/20ticks. Nine packed floors nominally add135workticks and no items. Actual command eligibility and occupied fixture footprints determine which floor cells can be laid; the scripted scene must use legal construction and the native floor-coverage denominator, not inject finished rooms.

## Dwelling and visibility

Authored stages require a roof and bed, then complete eligible floor coverage, then paid current warmth, then one adult-day of reachable stored food within8navigation tiles. Upgrades are immediate. Continuous deficiency waits2880ticks before downgrade; recovery clears that timer. Physical roof loss removes shelter immediately. Mood values10/20/30/40permille are initial stage effects; one resident receives one effect from their actual dwelling.

Use actual successful movement for door state. The existing fuel transaction supplies a receipt, not a second fuel debit. Reuse RoomSystem152 for integration and current felt refresh; hunger130 sees the previous committed Cold on the hourly boundary. Saved partial door intervals, pending downgrade time and last hourly stamps must resume identically. Baseline keeps its original simulation and format17 bytes; colony format18 adds only the thermal sidecar with an exact17 migration.

Existing room/person inspectors, room overlay, dwelling marker, diagnosis, chronicle and technology gauges show native values. There is no GDScript heat or stage solver, no automatic UI command, no extra heater and no new autonomous housing objective in this stage.

## Proof required before publication

The real scripted scene constructs around the physical primary hearth, pays actual fuel, observes warm/Cold and dwelling transitions, opens a door by movement, exhausts fuel, cools, recovers and continues identically through save/load. Capture the visible steps, causal diagnosis, chronicle and era gauges in the actual game. Explicit starting raw supplies and knowledge are disclosed; heat, payments and stages are never injected.

Retain determinism, baseline identity, replay, ledger, no panics, tests/Clippy, data-only content proofs and absolute C3 ceilings. Paired seeds0–7×48days use published Stage2 as BASE: no new extinction where BASE survives, KnowledgeLost total≤2×BASE+2. Full gate and the existing step41 waiver apply. No held-out1584–1615,95%comfort target, improvement bar or balancing loop is run. Current implementation and all Stage3 acceptance results remain pending.
