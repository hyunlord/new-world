# Frozen image-model comparison protocol V2

One full round, same9 candidates, each separate tool invocation; max3 concurrent. No retries except tool error. V1 remains unchanged. Numeric thresholds unchanged. This revision disambiguates candidate identity and relative reference matching; no expected labels or prior outcomes are included in prompts. Frozen before any V2 calls.

## Exact prompt
Image3 alone is candidate; images1and2 are references ONLY. Compare candidate's actual rendering style RELATIVELY with the supplied references, not with an idealized art brief. Score1unrelated,2substantially different,3mixed/noticeable drift,4close match,5indistinguishable language in MEDIUM,LINE,LIGHT,PALETTE. Fine ink/tone is the reference name; inspect the actual marks and tonal transitions present in the references. Judge content/camera separately, do not penalize different depicted subject. Outputplain scorecard four scores, all>=4PASS, one reason describing candidate content plus actual style difference. Do not evaluate/repaint reference instead of candidate; don't assume expectedoutcome.

## Inputs
Reference1: locked person.png. Reference2: locked scene.png. Candidate3: each of six anchors plus same three earlier comparison images. Actual classifications are excluded from tool prompts.
