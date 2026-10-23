# Round 1 frozen scoring notes

**Raw every-cell noninferiority FAIL.** All 36 task cells and 18 separate activity cells remain in `comparison.md`; 144 task observations and 72 activity subpoints remain in `scores.json`. The key and six response files were byte-hashed before mapping, then checked unchanged after scoring. `pre_scoring_freeze.json` records those hashes. Fourteen copied raw files also match their recorded SHA-256 values.

| Regressing cell | Before → after | Evidence-backed reason |
|---|---:|---|
| Day 1× activities | 6→4 /6 | Both reviewers read C as walking rather than awake idle. |
| Winter 1× activities | 6→4 /6 | Both reviewers read C as walking rather than awake idle. |
| Night 1× people | 2→1 /2 | Reviewer b counted 7 against the frozen visible key of 10. Reviewer a counted 9, within ten percent. |
| Day 1× C | 2→0 /2 | The primary answers explicitly select walking. |
| Winter 1× C | 2→0 /2 | The primary answers explicitly select walking. |

There are 3/36 task-cell regressions and 2/18 activity-cell regressions. Night 1× C also receives 0/2 after, but its baseline is 0/2, so it is not a regressing cell. This distinction does not erase the wrong answers.

The after half-zoom count gate passes all 6/6 answers: day 8/8, night 9/10, winter 8/8 for each reviewer. Every image timer is observed and within the registered limit: 24/24, 12–41 seconds, zero late answers. These are tool/model elapsed times, including recording and the reviewers' disclosed write-error corrections, not human response times.

## Activity mapping consistency

Exactly 72 mappings preserve each raw answer, expected key and rationale. Of these, 58 receive one point. All 12 after-image A answers now identify chopping or using an axe. `Using an axe` is scored 1 under the same ordinary-meaning rule used for the Round 0 answer `Using an axe.`. A note that motion cannot be confirmed in a still image does not replace the unambiguous selected primary answer.

All six after 1× C answers explicitly select walking. Each receives 0 against awake idle, just as walking was scored in Round 0. The absence of a tool or load is not evidence that the reviewer selected standing. No generic hand-tool answer or static `holding an axe` answer occurs in Round 1; the conservative Round 0 treatment of those cases has not been changed.

Native night B and C remain keyed to their displayed sleep frames, not hidden routine labels. Four carrying readings and four standing-at-bench readings receive 0 against sleep. Nearby context is not treated as proof of a hidden interaction.

No raw response, key, image, threshold, scorer or production code was edited during scoring. The one permitted compositor repair is exhausted; these results do not authorize another repair or retry. Different fresh reviewers across rounds mean score differences do not isolate the repair's causal effect.

## Inference limits

Controlled noninferiority remains **NOT_VERIFIED**. The declared after scene changes object content, actor actions and some positions. Storage, carrier, wolf and activity comparisons remain `NOT_COMPARABLE` as a controlled style test; people retain their position mismatch. No cell was removed. Owner approval remains **PENDING**.

- Activity mappings SHA-256: `4eacd3f2fd9769190670b5eb2354228c07f979cd6d6c123e52c9b224872db152`.
- Scores SHA-256: `3faf3babffafa3059939e4fbd7e10e642894130d083707ab15dca50dc1f1d808`.
- Scoring command: `uv run --with pydantic python -m art.pipeline_env1.readability_score --round 1` (exit 0).
