# WM-3 post-hoc comparisons (not pre-registered; `tools/worldmodel/wm3/posthoc.py`)

| comparison | model (seeds) | baseline | Δ [95 % CI] | rule label |
|---|---|---|---|---|
| `wm3_full` vs WM-2 M1b-act, 30d cells | 0.511 (0.517, 0.504, 0.512) | `wm2_m1bact_v19 (seed mean)` 0.501 | +0.010 [+0.006, +0.014] | NONE |
| `wm3_full` vs WM-2 M1b-act, 7d cells | 0.496 (0.532, 0.472, 0.484) | `wm2_m1bact_v19 (seed mean)` 0.592 | -0.096 [-0.122, -0.067] | WEAK |
| `wm3_full_s0` vs `wm3_ctrl_s0`, 7 d cells | 0.532 (0.532) | `wm3_ctrl_s0` 0.622 | -0.091 [-0.108, -0.073] | WEAK |
| `wm3_os_s0` vs `wm3_ctrl_s0`, 7 d cells | 0.617 (0.617) | `wm3_ctrl_s0` 0.622 | -0.005 [-0.043, +0.043] | TIE |
| `wm3_full_s0` vs `wm3_os_s0`, 7 d cells | 0.532 (0.532) | `wm3_os_s0` 0.617 | -0.085 [-0.136, -0.049] | WEAK |
| `wm3_ctrl_s0` vs `wm2_m1bact_v19_s0`, 7 d cells | 0.622 (0.622) | `wm2_m1bact_v19_s0` 0.650 | -0.027 [-0.033, -0.021] | WEAK |
| `wm3_full_s0` vs `wm2_m1bact_v19_s0`, 7 d cells | 0.532 (0.532) | `wm2_m1bact_v19_s0` 0.650 | -0.118 [-0.136, -0.098] | WEAK |
| `wm3_full` vs `an5_arm`, 7 d, food_stock only | 0.822 (0.963, 0.741, 0.763) | `an5_arm` 0.709 | +0.113 [+0.060, +0.165] | NONE |
| `wm3_full` vs `an5_arm`, 7 d, alive only | 0.482 (0.482, 0.482, 0.482) | `an5_arm` 0.595 | -0.113 [-0.175, -0.051] | WEAK |
| `wm3_full` vs `an5_arm`, 7 d, deaths only | 0.437 (0.441, 0.421, 0.451) | `an5_arm` 0.360 | +0.078 [-0.082, +0.293] | TIE |
| `wm3_full` vs `an5_arm`, 7 d, settlements_alive only | 0.241 (0.241, 0.243, 0.240) | `an5_arm` 0.277 | -0.036 [-0.082, +0.017] | TIE |
