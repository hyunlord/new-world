# Character template v3 — whole figure candidate

TRIAL5 route I, owner selected I-1. Candidate contract, not adopted standard.
Historical pawn contract remains in TEMPLATE.md and TEMPLATE_V2_HISTORICAL.md
until the overall trial verdict authorizes adoption.

Frozen pixel rules: `docs/v8/arttrial5/WHOLE_FIGURE_CONTRACT.md`.
Look space: sex2 x age2 x top3 x hair3; S and E, W mirrors entire E figure;
N excluded. One sheet contains idle, walk_a, work_a, carry, sleep left to right.
All items are baked in. No layer or material-mask obligations apply to v3.

Standing idle structural height56 adult /40 child; same idle-derived factor for
all five poses. Canvas64x96 baseline y88; sleep96x64 baseline y56, head left.
The baseline names the lowest alpha>=128 pixel coordinate, not a half-open extent.

Animation: walk alternates walk_a and idle with a 1px whole-figure bob; work
alternates work_a and idle; carry retains carry plus a 1px whole-figure bob;
sleep remains sleep. No leg-region flip, pixel surgery or independent limb motion.
Bobs happen only at display time; checked source baselines remain unchanged.
Structural pass cannot certify tool identity, head-left sleeping or limb readability.

Output route_i/<look>/<S|E>/{idle,walk_a,work_a,carry,sleep}.png plus JSON sidecars.
Attempt metadata preserves raw hash, prompt hash, try, judge numbers, crop boxes,
scale, split columns, all checker findings and diagnostics path. Rejected sheets
never populate delivered route_i outputs. Retry budget is two generations/sheet.
