# TRIAL2b readability round 1 — FAIL

Both independent reviewers missed activity/carrying thresholds on both grounds at
both scales. Original thresholds and 24-attribute denominator are unchanged.
`READABILITY_V2_ROUND1.json` preserves every response verbatim, per-answer grading,
expected attributes, source review SHA256, capture SHA256 and answer-key SHA256.
Original captures/keys/runtime are frozen externally under `trial2b/round1/`.

| Reviewer | Ground | 1× action/carry | 1× attributes | 2× action/carry | 2× attributes |
|---|---|---:|---:|---:|---:|
| blind_a | meadow | 7/12 | 19/24 | 7/12 | 20/24 |
| blind_a | forest | 7/12 | 19/24 | 7/12 | 20/24 |
| blind_b | meadow | 7/12 | 17/24 | 7/12 | 20/24 |
| blind_b | forest | 7/12 | 17/24 | 7/12 | 20/24 |

Action failures are figures 2, 3, 6, 7, 11: walking looked standing, chopping looked
like merely holding an axe, and knapping looked like spreading empty hands.
Sex/presentation errors recur for figures 1, 2, 7, 8; ages were correctly recognized.
Holding the recognizable log bundle counts as carrying a load; an axe-like item
can earn item identity while earning no chopping point. Lying maps to sleep even
when the inferred sleep wording is tentative; “standing or walking” earns no
walking point. No activity alternatives are resolved in favor of the key.

G1 remains FAIL: every 1× score is 7, only +1 versus historical 6 and +3 versus
historical 4, below the conservative requirement of +3 versus both.

The single allowed targeted revision addresses gait silhouette, overhead axe
movement, actual knapping tools/hand pose, and generated facial presentation.
Round 1 evidence remains frozen. Round 2 must use two fresh reviewers and the same
rubric; no further revision round is authorized by this protocol.
