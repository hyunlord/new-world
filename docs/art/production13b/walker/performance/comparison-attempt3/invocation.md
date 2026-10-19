# Corrected pair-consistency gate

Review round1 identified that the supplemental W500 gate omitted the inherited >=2/3 faster-pairs condition. The validator now requires that condition for both worlds while retaining W200<=0.97 and W500<=1.00 pooled ceilings. The inherited validator is unchanged.

A regression probe against the pre-fix acceptance expression failed as expected for W500 ratio0.99 with only1 faster pair. After the fix six checks passed: W500(.99,1)->FAIL; W500(1.00,2)->PASS; W500(1.001,3)->FAIL; W200(.97,2)->PASS; W200(.97,1)->FAIL; W200(.971,3)->FAIL. These lightweight checks evaluate the actual parsed acceptance expression.

Reran only the comparator against the same completed measurements with P13_REFERENCE=/Users/rexxa/work/new-world-art-prod/docs/art/production13b/perf/calibration.json. Exit0. Both worlds remain3/3 faster; ratios unchanged. Attempts1 and2 preserved. No recapture, product change, build or HEAD change. Candidate acceptance still requires identity.
