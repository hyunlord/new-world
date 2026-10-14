# Part 3 presentation verification

- `test_restless_meter.gd`: failures0. Exact rational thirds (theta1000 boundaries333/334/666/667, theta3 equality1/2), raw pressure, overflow stage, zero/missing theta or pressure, reset, actual meter colour/native width, zoom1hidden and2/4/8visible, selected inspector title/state in en/ko and no-selection hidden.
- `test_settlement_labels.gd`, `test_offscreen_marker.gd`: failures0 after meter changes.
- `gdscript_strict_check.sh`:41files clean,16FFI exports matched.
- Localization compile strictduplicates0; four keys additive.
- This evidence is headless behavioral verification. Real-scene pressure/capture acceptance belongs to the parent capture run and is not claimed by these tests. Rust getter tests belong to the bridge lane.

Actual windowed v19 capture: 3 frames succeeded, no SCRIPT ERROR. Parent inspected1920px originals. d32 20:00 original village P323/theta332 (97.29%, ready), d33 06:30 after departure P0/theta2792, d33 08:59 daughter P0/theta198 (calm). Selected inspector title/state fits its panel; nameplate meter26x6px, stable colours. Manifest and compressed raw sidecars in verification/p3-meter; extracted exactvalues verification/p3-values.json. Capture source cad63594 plus Part3workingtree; postcapture INT64 overflow guard is equivalent for all capturedvalues, separately unit-tested. FinalPart5 captures rebind finalsource.
