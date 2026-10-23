# M5 evaluation (Part 6, goal section 11)

Held-out seeds **800–831** (`--seeds 800..832`; before the evaluation `git grep` found them only in the goal,
DECISIONS D7 and `colony_story.rs`'s schedule test, which checks F1's mercy and gap rules on the storyteller's plan —
registered in Part 1, not a tuning input). Calibration used seeds 0–7 in eight rounds r0–r7 (the goal's limit;
DECISIONS D14), then the simulation was frozen. The bars and their reading were registered in the scorer's docstring
(`docs/v8/m5/m5_score.py`, DECISIONS D13) before the first held-out run; F4's "hides ≥ 6" was added to the scorer after
calibration and before that run (D13). Raw outputs (outside the repository): `$DATA` = `/Users/rexxa/work/m5-data/`.

Final: `$DATA/eval_final/` at `3a8d0759` (after the reviews and D18; see Bars). **Result of record: `$DATA/eval/`, the probe built
from `6f1713b4`** (the Part 4 head; the working tree's other
changes were the bridge, documents and gate scripts — `$DATA/eval/dirty.txt` — none in the simulation crates).
Runs took 546 s (load 2.7 at the start; the UI agent and these runs loaded the host to 29 by the end; timings are not
used).

| run | command | used for |
|---|---|---|
| `pressure` | `worldsim_probe --scene colony --seeds 800..832 --days 48 --workers 6 --commands tools/v8/m5_u_script.json --check-determinism --commands-out $DATA/eval/log` | F1–F8, F10; determinism (**DETERMINISM PASS**) |
| `control` | the same with `--story off` (no storyteller; wildlife, hunting, mood on) | F6a, F9a, F10 |
| `nodraft` | `--commands tools/v8/m5_u_nodraft_script.json` (storyteller on, nobody drafted) | F3b, F10 |
| `long` | no script, `--story off`, `--days 72` (three years) | F9b, F10 |
| `replay_*` | seed 800 with the script vs. seed 800 from its recorded command log, 48 days, `cmp` of `summary.json` | replay (**REPLAY PASS**) |
| `p4_soak` | `tools/v8/soak.sh` (seeds 0–3 × 20 days, `--story no-beasts`) | F10 (**SOAK PASS**) |
| bench | `$DATA/bench5/session.sh` | F11 |

## Bars

Two measurements on the same held-out seeds: **of record** (`$DATA/eval/`, `6f1713b4`) and **after the reviews**
(`$DATA/eval_review/`, the code review's fixes D17, and `$DATA/eval_final/` at `3a8d0759`, the final simulation with
the quarrel cap D18 and the probe's corrected pack window). `eval_final`'s summaries equal `eval_review`'s on every
seed of all four runs outside the pack rows (the cap never binds in a 10–13-person colony), and its scores are the
column below. Neither was tuned on these seeds.

| # | bar | pass | of record (`6f1713b4`) | after the reviews (final, `3a8d0759`) |
|---|---|---|---|---|
| F1 | ≥ 1 ColdSnap and ≥ 1 Beasts in two years; mercy and gaps hold | 32/32 | **32/32 PASS** (snaps 1–2 a world, packs 2–4, droughts on 18, illness on 32) | **32/32 PASS** (same counts) |
| F2 | firewood below one day's need during a snap in ≥ 24/32 **and** cold deaths ≤ 1 in ≥ 28/32 | both | **PASS**: F2a 26/32 (803, 805, 810, 822, 829, 830 kept enough wood; at the same hour 25/32 — 815 flips, numbers review); F2b 30/32 (804: 9 exposure deaths, 814: 8 — 8 and 7 in the first snap, days 21–24, with an illness at the same time and usable wood at 0; one more each on day 48, in the second snap) | **PASS**: F2a 26/32 (803, 805, 818, 820, 822, 830); F2b 30/32 (804: 9, 814: 6; 806: 1) |
| F3 | the scripted draft kills the pack with 0 deaths in ≥ 24/32; without the draft ≥ 1 injury in ≥ 24/32 | both | **PASS**: F3a 30/32 (100 packs, all beaten; 804 and 814 fail on deaths booked to a gone pack — see the note); F3b 32/32 (6–20 person-packs bitten a world) | **PASS**: F3a 32/32 (corrected window: all 100 packs beaten — none left on the map, half or more killed — with 0 deaths while they were there); F3b 32/32 (6–20) |
| F4 | ≥ 6 deer killed by day 48, meat eaten ≥ 12, hides ≥ 6 | ≥ 28/32 | **25/32 FAIL**: deer 6–18 (median 13.5), hides = deer; meat eaten < 12 on 804, 805, 806, 809, 814, 823, 829 | **26/32 FAIL**: deer 4–21 (median 14); meat eaten < 12 on 800, 804, 805, 814, 823 (eaten 0–10, spoiled 22–31); 829 killed 4 deer |
| F5 | ≥ 4 wear a hide wrap by day 30; their winter-night cold rate ≤ half the unclothed | ≥ 24/32 | **20/32 FAIL**: wraps at day 30 0–6 (median 4), fewer than 4 on 12 worlds; the rate half trivially met on the 28 worlds without a winter-night cold hour, failed on 804 (0.202 vs 0.184) and 812 (0.0169 vs 0.0331) | **23/32 FAIL**: fewer than 4 wraps on 9 worlds (800, 804, 806, 814, 819, 823, 827, 828, 829); the rate half holds where cold hours exist (806: 0.029 vs 0.242) |
| F6 | control: breakdowns ≤ 1 in ≥ 28/32; pressure: ≥ 1 world in 4 has a breakdown | both | **FAIL**: F6a 32/32; F6b 2/32 | **FAIL**: F6a 32/32 (no breakdown in the control); F6b 7/32 (804, 806, 811, 814, 816, 818, 819), one short of 8 |
| F7 | ≥ 2 quarrels a world-year (≥ 4) with a resolvable chain; fights ≤ quarrels/4; 0 quarrel deaths | ≥ 28/32 | **25/32 FAIL**: quarrels 0–14 (median 6.5), below 4 on 803, 804, 807, 808, 812, 814, 820; every chain ≥ 2 links (by construction; 35 of 210 reach a chronicle entry beyond the quarrel); 0 fights; 0 combat deaths | **27/32 FAIL**: quarrels 1–17 (median 7), below 4 on 804, 807, 814, 818, 820; 31 of 218 chains reach a further entry; 0 fights; 0 combat deaths |
| F8 | ≥ 10 blades used up by knives by day 48; top knapper ≥ 6; quality days 37–48 − days 1–12 ≥ 8 | ≥ 24/32 | **0/32 FAIL**: 7–22 knife uses, 0–1 knives used up; top knapper 6–12; no blade knapped after day 36 on 28 worlds | **0/32 FAIL**: 5–22 uses, 0–1 used up; top knapper 6–9; a late blade on 2 worlds |
| F9 | control births / woman-year 0.20–0.30; the 3-year no-orders run ≥ 30/32 without a cold death | all | **PASS**: F9a pooled 0.277 (38 / 137.3; M4 measured the same way on seeds 0–7: 0.336); F9b 32/32 (0 deaths; 99 births / 337.5 woman-years = 0.293) | **PASS**: F9a 0.284 (39 / 137.3); F9b 32/32 (0 deaths; 99 / 337.5) |
| F10 | ledger per kind exact; reservations exact; soak green (knowledge, wolves off) | all | **PASS**: 0 / 0 violation ticks in 128 runs; SOAK PASS | **PASS**: 0 / 0 in 128 runs; soak in the gate |
| F11 | M5 systems ≤ 120 µs/tick at N = 1,000; alloc/agent ≤ M4 + 40 B; C3; ≤ +10 % vs M4's quiet rows; identity; replay | all | identity PASS, replay PASS; bench PENDING | identity, replay PASS (DETERMINISM PASS on the pressure run); bench PENDING (`$DATA/bench5`) |

Summary (final): F1, F2, F3, F9, F10 pass; F4, F5, F6 (F6b), F7, F8 fail; F11 pending the bench. Against the record
the fixes moved F3a 30 → 32 (the probe's window), F4 25 → 26, F5 20 → 23, F6b 2 → 7, F7 25 → 27; per-seed values
changed on most worlds (the hunt and fight fixes change who is where), so the failing seed lists differ.

## Control against pressure (same seeds, same script; of record)

| measure (32 worlds) | pressure | control (`--story off`) | no-draft |
|---|---|---|---|
| deaths | 18 (exposure 17, dehydration 1) | 1 (dehydration) | 11 (804: 1 combat + 2 exposure; 814: 7 exposure + 1 dehydration) |
| people at day 48 | 1–13 (two worlds collapsed to 1) | — | — |
| breakdowns | 2 | 2 | — |
| quarrels (median a world) | 6.5 | 8 | — |
| births / woman-year (pooled) | — | 0.277 | — |
| wolves: packs / beaten / killed / left | 100 / 100 / 212 / 171 | — | 100 / 100 / 199 / 191 |
| bites / person-packs bitten | 1,224 / 374 | — | 978 / 396 |

Pressure is felt in the body (17 exposure deaths on two worlds, 15 of them in their first cold snap; the firewood line
fell below a day's need on 26 worlds) but hardly in the mood: breakdowns are as rare under pressure as without it, and people
quarrel a little *less* under pressure (hypothesis, not measured: illness keeps people in bed and drafted people
cannot quarrel, and a quarrel needs two idle people within 2 tiles). Drafting changes the packs' outcome little: undrafted colonies also beat every pack (people strike back with what
they hold) and take fewer bites, because the scripted draft sends four people at the wolves. The only wolf-caused
death in all 64 storyteller runs is in the no-draft run (804, during its day-40 pack); the drafted runs had none. One
death does not show that drafting matters, and F3b passes as registered (≥ 1 injury) without showing it either. Not
explained: no-draft 804 lost 2 people in its first snap against 8 with the draft (its state going into the snap
differed; not measured). The probe does not record whether the drafted people held spears (the resolver counts an axe
as armed), and 814 never made a spear.

## Why the failing bars fail (diagnosis on the held-out data; nothing was tuned on it)

- **F4 (meat eaten)**: a kill drops its meat at the deer's tile, 20–48 tiles from the hearth; the meat counts as eaten
  only when it reaches a stockpile cell or the hearth, and Haul is the last work type. On the seven failing worlds the
  meat spoils on the ground (6 days) before haulers bring it (spoiled 4–32 a world). Two of them (804, 814) had
  collapsed to one person.
- **F5 (wraps by day 30)**: a wrap needs two cured hides, a cured hide one scraping with the rack's knife, a hide one
  kill; with the first kills around day 8–12, worlds that kill fewer than about eight deer by day 26 cannot dress four
  people by day 30. The cold-rate half is met on 28 worlds trivially (no cold hour at all); where cold hours exist a
  wrap's goal factor (700 ‰) gives an expected ratio of 0.7, not ≤ 0.5 (D15).
- **F6b (breakdowns under pressure)**: a breakdown needs mood < 100; the worst common day (hungry −150, cold −120,
  slept outside −120, pressure −40) sums to −430 before temperament (about −390 at average harm avoidance), from 500 a
  target near 110, so it takes a kin's death (−250) or a quarrel (−100) on top; deaths are rare except in the two
  collapsed worlds.
- **F7 (quarrels)**: quarrels need two awake people within 2 tiles who are not working; 7 worlds fall below four in
  two years. Hypothesis (not measured): worlds with more work and fewer idle hours have fewer such meetings. The chain
  half holds for every quarrel.
- **F8 (knives)**: the goal's knife sink is scraping (one use a hide) and butchering (one use a kill with a held
  blade): 7–22 uses a world in 48 days, at 8 uses a knife one or two knives — ten would need about 80 uses. With
  `Maintain 6 FlintBlade` met early, no blade is knapped after day 36 on 28 worlds, so the late quality mean does not
  exist (the same structural failure as M4's F8).

## Notes

- F3a's pack deaths count deaths while the probe's `wildlife.pack_day` is set, which is never cleared: a pack's
  window runs until the next pack comes or the run ends. On 804 and 814 the first snap's 8 and 7 exposure deaths were
  booked to the day-16 / day-14 packs (gone on the day they came) and the day-48 death to the day-40 pack. Every one
  of the 100 Beasts pressures has `cost.deaths` 0 (its cost runs until no wolf is left) and combat deaths are 0 on all
  32 pressure worlds, so with the window limited to wolves on the map F3a would be 32/32; the registered 30/32 is the
  result of record. The probe's window is fixed (Part 7; measurement only: a pack's row is updated while its Beasts
  pressure is active or its wolves are on the map); re-run on 804 and 814 with the fixed probe, the deaths are identical
  (9 and 8 exposure) and every pack's `deaths` is 0, so both pass F3a (`$DATA/review/packfix_804`, `packfix_814`).
- F7's "resolvable" (≥ 2 links) cannot fail: every mood quarrel is chronicled with a modifier cause (or the relation),
  so its chain has the quarrel and that cause. 175 of 210 chains stop at the modifier (slept outside, hunger …, which
  have no chronicle entry); 35 reach another entry (a pressure's beginning, a previous quarrel). Read as "the chain
  reaches a chronicle entry beyond the quarrel", no world passes F7. F7 fails either way.
- F7's "0 quarrel deaths" is checked by the scorer as combat deaths ≤ the pack-window deaths, which the long windows
  make weak; the strict check (combat deaths 0) gives the same 25/32 here (0 fights, 0 combat deaths).
- Replay: seed 800's recorded command log (`$DATA/eval/log/colony_s800_commands.json`) reproduces the scripted run's
  `summary.json` byte for byte over 48 days.
- Per-seed values: `$DATA/eval/appendix.json` (the scorer's `--out`). The fresh numbers review recomputed F2, F3, F7 and
  F8 from the raw summaries with its own code and matched every count and failing seed
  (`$DATA/review/numbers_review.md`); the corrections above are its findings.
