# Pass 11 macro sensitivity (12.2)

Scene `game_v17`, seeds 12000..12032, R = 2, 120 days, fork at 28,800. Shape = settlements alive at days 60 / 90 / 118 and the first-fission day.

| Comparison | Share differing | Mean abs delta alive day 118 |
|---|---:|---:|
| (a) factual r0 vs r1 (noise only) | 0.09375 | 1.5625 |
| (b) rot_wood vs factual, same replicate (seed-replicate pairs) | 0.266 | 1.953 |
| (b) gift_wood vs factual, same replicate (seed-replicate pairs) | 0.172 | 0.500 |

Per variant: share of seeds differing in at least one / in every replicate: rot_wood 0.312 / 0.219, gift_wood 0.219 / 0.125
