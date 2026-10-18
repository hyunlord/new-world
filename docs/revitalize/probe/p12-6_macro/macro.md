# Pass 12 macro sensitivity (12.2)

Scene `game_v18`, seeds 12000..12032, R = 2, 120 days, fork at 28800. Shape = settlements alive at days 60 / 90 / 118 and the display day of the first fission founded after the fork. Factual r0: {'seeds_with_a_fission': 18, 'seeds_with_a_post_fork_fission': 16, 'seeds_split_only_before_the_fork': 2}.

| Comparison | Share differing (counts or post-fork day) | Share with post-fork day differing | Mean abs delta post-fork day (n both split) | Mean abs delta alive day 118 |
|---|---:|---:|---:|---:|
| (a) factual r0 vs r1 (noise only) | 0.688 | 0.688 | 34.500 (14) | 18.406 |
| (b) rot_wood vs factual, same replicate | 0.016 | 0.016 | 0.000 (35) | 0.703 |
| (b) gift_wood vs factual, same replicate | 0.016 | 0.016 | 2.444 (36) | 0.688 |
| (b) spoil_food vs factual, same replicate | 0.000 | 0.000 | 0.000 (36) | 0.031 |
