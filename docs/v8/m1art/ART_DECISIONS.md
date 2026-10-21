# M1-ART decisions

Decisions taken without asking the owner (goal §0). One line of context, the decision, the reason.

- **D1 (Part 0) — "asleep" and "facing" need no Rust change.** The agent snapshot already carries `states`
  (state tag: 3 = `Consuming` a non-agent target) and `target_kinds` (3 = `Sleep`), so "asleep" is
  `states == 3 and target_kinds == 3`; "facing west" is the sign of the last drawn step, kept by the agent layer.
  The `sleeps` column is the sleep *need* (fatigue), not a state, so it is not used for the pose. Goal 3.3's
  optional read columns are therefore not added; `rust/` stays untouched.
- **D2 (Part 0) — before shots.** `tools/v8/capture.gd --m1-set` at `a4a7eaf4` (code identical to `bba3110e`),
  seed 20261010, saved to `/Users/rexxa/work/m1art-data/before/` (not committed): work tick 481, hut finished at
  766, P4 at 826, P5 at 900.
