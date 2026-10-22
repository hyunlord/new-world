# TRIAL2b blind readability protocol (registered before review)

Two new image-only reviewers see the four actual Godot captures: meadow and forest floor at 1× and 2×. They receive no answer keys, implementation details, expected scores, previous reviews or pass bars. Read twelve figures in row-major order. For every figure report activity and carrying yes/no/unclear; for the first eight also report perceived sex/presentation, age group and held item. Unknowns remain unknown. Preserve their responses verbatim and bind reviews to image SHA256.

Use the historical TRIAL2 grading policy in `READABILITY.json` unchanged: activity AND carrying earns one point; standing=idle, walking=walk, lying= sleep, carrying a load/basket=carry. Holding a tool alone is not chop; sitting alone is not knap. Ambiguous alternatives are wrong. Actually held axes, logs, baskets and elder cane count as carrying. The 24 attribute answers remain eight sex, eight age and eight item answers; no waived fields or smaller denominator.

Each review of each image must achieve ≥8/12 at 1× and ≥10/12 at 2×; attribute scores must reach ≥20/24. Different presentation is authored through generated parts; reviewer perception is tested, never inferred from the key.

If any bar is missed, preserve this first round, revise only specific implicated parts/poses within the existing try ceilings, and run exactly one second round with two fresh reviewers. No threshold changes or reviewer answer coaching.

## G1 comparison

TRIAL2's two historical 1× scores were 6/12 and 4/12 on each ground (`READABILITY.md`). Report both differences for each new reviewer and ground. Conservative G1 requires every new 1× score to be ≥9/12, therefore at least three above both historical scores. This is an end-to-end TRIAL2 versus TRIAL2b comparison (camera, parts and ground changed), not a causal ground-only claim.

Also capture the exact new twelve-person composition on old and new ground at the same scale for `ground_before_after.png`. Freeze animation positions before paired screenshots so timing does not masquerade as a ground effect. A visual before/after board alone earns no numeric G1 score.

## Crowd

Use the 200-person 1× run's 640×360 crop and exact metadata-derived intersecting-person count. A separate fresh image-only reviewer counts visible people without seeing the truth. Partial figures count when their visible body intersects the crop. Report the absolute error divided by truth; require ≤10%. Keep 60-second actual post-draw FPS separate from process callbacks.
