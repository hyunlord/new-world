# Pass 13 replicate pilot — first fission and J7

| Scene | worlds × R | censored runs (no split by day 120) | worlds split in every replicate | W (within) | B (between) | PS first-fission day | within-world SD (d) | between-world SD (d) | PS (worlds split in every replicate) | r(day, θ) | r(day, early s) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|
| game_v16 | 64 × 4 | 256 | 0 | 0.000 | 0.000 | — | 0.000 | 0.000 | — | — (n —) | — (n —) |
| game_v17 | 64 × 4 | 96 | 40 | 0.128 | 1983.283 | 0.000 | 0.357 | 44.534 | 0.088 | — (n —) | — (n —) |
| game_v18 | 64 × 4 | 118 | 22 | 716.276 | 972.484 | 0.424 | 26.763 | 33.935 | 0.839 | — (n —) | — (n —) |
| game_v19 | 64 × 4 | 87 | 42 | 28.910 | 1368.250 | 0.021 | 5.377 | 37.087 | 0.089 | 0.472 (n 43) | -0.271 (n 43) |

J7 (a) replicate 0, test split: food_stock η² 0.624 / R²_abs 0.373 PASS; wood_stock η² 0.317 / R²_abs 0.681 PASS; alive η² 0.821 / R²_abs 0.175 PASS
J7 (b): {"alive_in_0.10_0.50": {"PS": 0.018570313594127957, "passed": false}, "food_in_0.10_0.60": {"PS": 0.02378874190839123, "passed": false}, "alive_below_v18": {"diff": -0.2570096441812432, "ci90": [-0.33131740290629685, -0.19271003399274295], "passed": true}}
J7: FAIL
