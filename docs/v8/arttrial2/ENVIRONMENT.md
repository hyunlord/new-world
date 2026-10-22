# Environment production ledger

The first pass produced 38 generated candidates plus two derived ghosts. No first candidate satisfies the full production gate. The first numeric style pass was hill rock (1/38); it failed seam checks. Preserve that denominator even if later candidates improve.

| Family | Requested | Generation calls through bounded retry 2 | Numeric eligible after retries | Accepted |
|---|---:|---:|---:|---:|
| Ground | 6 | 12 | 2/6 with mirror-blend and seam PASS | 0; model/calibration pending |
| Plants | 9 | 18 | 0/9 | 0 |
| Buildings | 6 | 12 | 0/6 | 0 |
| Items | 10 | 10 | 0/10 | 0 |
| Misc | 7 | 7 | 0/7 | 0 |
| Derived ghosts | 2 | 0 | Standalone alpha80 is outside numeric foreground alpha≥128 mask | 0; composite review required |

The 59 environment generation calls expose 38 unique generated subjects. Family mean calls per requested subject is2.0 for ground/plants/buildings and1.0 for items/misc; these are not calls per accepted subject. No asset exceeds two attempts, no family has been pushed beyond the3.0 stop threshold, and no limit was raised. Trials need not consume every permitted retry: the≤4 ceiling is not a requirement to spend four calls on known-failing candidates.

All six ground textures failed seam checks twice. Authorized 32px cosine-reflection mirror-blend leaves the448×448 interior unchanged and gives identical opposite borders. All six seam tests then pass; only packed earth and hill rock pass the frozen numeric style test. Their repeated motifs and softened seam strips remain visually visible and are not hidden by the metric.

Plants: second attempts pass line density9/9 and saturation8/9, improving saturation from1/9, but colour-histogram distance fails9/9. Buildings: second attempts fail histogram6/6; hut, lean-to, rack and bench pass the other two metrics; knapping spot and storage pit fail all three. No colour quantization or recolouring was used to force acceptance.

External records: `ground/ledger.json`, `ground/retry_summary.json`, `plants/generation_ledger.json`, `plants/generation_ledger_try2.json`, `buildings/LEDGER.json`, `items/LEDGER.json`, `misc/LEDGER.json` under `/Users/rexxa/work/arttrial2-data/`. Every retry retains its prompt, anchors, raw/working hashes, processing and numeric verdict. Model scorecards are separate. COST.md distinguishes summed overlapping call brackets from elapsed wall time.

`art/style05/boards/environment_first_attempt.png` is a review-only40-up, not an accepted asset pack. It retains the rejected first candidates so the report cannot present only the best later samples. Accepted count and model/calibration outcome will be finalized without changing first-attempt results.
