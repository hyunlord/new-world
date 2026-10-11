# WM-1 notes: fields, hidden state, tasks

One page of working notes written in Part 0 from `docs/worldmodel/README.md`, `DATASET_v0.md`, `BENCH_v0.md` and `tools/worldmodel/bench_v0.py`.

## The data as WM-1 reads it

- One row per simulated hour (`--every 60`); row t is the state after t ticks, **before** tick t's interventions. An intervention logged at tick t is applied after row t and first shows in row t+1. Tick 0 is 08:00 of display day 0; the tick of one-based day D at minute m is (D − 1)·1440 + m − 480, its row is that tick / 60.
- Every v0 episode has exactly one settlement (`settlements.npy` is [T, 1, 9]); it forms at row 1 (row 0 is all NaN). Seasons are 7 days: spring 1–7, summer 8–14, autumn 15–21, winter 22–28 (one-based days, repeating every 28). OOD_v12 has seasons but `winter_severity` is −1 throughout.
- `global.npy` (33 fields): calendar (`tick`, `day_index`, `minute_of_day`, `season`, `day_of_season`, `year`), `winter_severity`, population (`alive`, `children`, `elders`, `adults`), cumulative tallies (`births`, four `deaths_*`, `meals_from_store_total`, `ground_meals_total`, `forage_trips_total`, `wood_trips_total`, `wood_gathered_total`, `urgent_wood_trips_total`, `wood_retargets_total`, `cold_member_hours_total`, `interventions_applied_total`), instantaneous counts (`cold_members`, `deadwood_total`, `food_on_bushes_total`, `adults_working`, `idle_adults`, `settlements_under_rain`, `cold_snap_nights_left`, `cold_snap_tonight`).
- `settlements.npy` (9 fields): `members`, `food_stock`, `wood_stock`, `wood_target`, `fire_lit`, `wood_in_hand`, `formation_x/y`, `rain_ticks_left`.

## Fields WM-1 uses

| Group | Fields | Sev-blind | Sev-on |
|---|---|---|---|
| global state | `alive children elders adults cold_members deadwood_total food_on_bushes_total adults_working idle_adults settlements_under_rain cold_snap_nights_left cold_snap_tonight` | yes | yes |
| global tallies (as hourly differences) | `births deaths_* meals_from_store_total ground_meals_total forage_trips_total wood_trips_total wood_gathered_total urgent_wood_trips_total wood_retargets_total cold_member_hours_total interventions_applied_total` | yes | yes |
| settlement (summed; fire = max) | `members food_stock wood_stock fire_lit wood_in_hand rain_ticks_left` + presence flag | yes | yes |
| `wood_target` | settlement | **no** | yes |
| `winter_severity` | one-hot (mild/normal/harsh/none) | **no** | yes |
| calendar | hour one-hot, season one-hot incl. −1, day of season | yes | yes |
| actions | requested amount per kind at the row of its tick (as `bench_v0.episode_arrays`) | action-aware only | action-aware only |

Never inputs: `latent.npy` (its `wood_winter_acc` change reveals the day's fall, hence the severity), `lifespan_days` (the only field marked `hidden` in schema 1), `formation_x/y` (omitted as in the bench), `tick`, `day_index`, `year` (calendar enters as hour, season, day of season only).

Severity leaks besides the label: `wood_target` in winter is 4 × burn × 13 = 104 / 156 / 208, so it is severity-on only.

## The tasks in my own words

- **T1** — from rows ≤ t predict V at t+1, for every t from 23 to T−2 of every test episode. Score: per-variable MAE divided by the std of that variable over the split's scored rows (`split_std` = std of `v[24:]` pooled over the split's episodes), averaged over the eight V.
- **T2** — from 06:00 of one-based days 3, 10, 17, … roll the model forward on its own predictions, 24 h and 168 h; score V at the horizon with the same denominator. T2-any (new) adds starts at all 24 hours of the same days.
- **T3** — at 18:00 of every winter evening of A test, the probability that the fire is out at 02:00 (8 rows later). Brier, AUC, accuracy. Nearly arithmetic in v0.
- **T3b** (new) — at 06:00 of each winter's third day (days 24, 52, 80, 108 …) predict how many of the five remaining winter nights will be dark (fire out at 02:00). MAE and exact accuracy vs the train mean, a ridge on observables and the severity oracle.
- **T4** — C_cf test: from the factual row at the intervention tick roll out with and without the intervention to day 29 07:00 and score the predicted effect on dark nights, cold member-hours, food store and deaths against the true effect (variant − factual). Per variant and target: MAE, sign accuracy, and MAE / mean |true effect|.
- **T5** (new) — agent level, next hour: state tag and target kind accuracy, Chebyshev position error, hunger/thirst/fatigue nMAE, against persistence and the same agent 24 h earlier.

## Dataset verification (Part 0, on the DGX)

`sha256sum -c` over every manifest file of one episode per split (60 files: `A_obs/game_v13_s0_none`, `B_act/game_v13_s1000_b`, `C_cf/game_v13_s2000_none`, `OOD_v12/game_v12_s3000_none`, `OOD_long/game_v13_s4000_none`): all OK. Episode files on disk: 4,660 (plus 192 schedule files under `v0/schedules/`, not in the manifest); their size 1,434,758,588 bytes = 1,368.3 MiB, equal to the manifest's `total_size`.
