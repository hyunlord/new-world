# Pass 14 Part 4.2 — evaluation amendments (PASS14_GOAL.md 4.3), exact command (Mac, no rerun of any simulation)

```bash
python3 tools/revitalize/evaluate_pass14.py \
  --j6 "held-out (p13-4, seeds 64–127 / 32–95)=docs/revitalize/probe/p13-4_heldout_v19n_targets/j_targets.json" \
  --j6 "tuning round 3 (p13-3, seeds 0–31)=docs/revitalize/probe/p13-3_round3_v19n_targets/j_targets.json" \
  --paths docs/revitalize/probe/p13-5_pilot/paths.json --diversity docs/revitalize/probe/p13-5_pilot/diversity.json \
  --out docs/revitalize/probe/p14-4_amendments
```

Inputs are Pass 13's committed outputs (the held-out run compares with Pass 11's final held-out `game_v16` on the same seeds, the tuning round 3 with `p13-0_v16`; both as Pass 13's evaluator recorded in each `j_targets.json` `inputs`).
