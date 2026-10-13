# Pass 12 diversity pilot (Part 5)

Seeds 24000-24111 (train 24000-24095, test 24096-24111), 120 days, `--every 60`, frozen recorder. eta2, R2_abs and dyn-eta2 on the test split (`diversity_p11.py`); R2_static and UBS on all 112 episodes (`diversity_p12.py`, mean over days 60/90/118).

| Variable | game_v16 | game_v17 | game_v18_nofission | game_v18 |
|---|---|---|---|---|
| food_stock | η² 0.332, R²abs 0.651, R²st 0.596, UBS 0.295, dyn 0.195 | η² 0.472, R²abs 0.475, R²st 0.305, UBS 0.502, dyn 0.502 | η² 0.311, R²abs 0.672, R²st 0.633, UBS 0.271, dyn 0.185 | η² 0.532, R²abs 0.457, R²st 0.388, UBS 0.536, dyn 0.534 |
| wood_stock | η² 0.069, R²abs 0.929, R²st 0.049, UBS 0.084, dyn 0.069 | η² 0.190, R²abs 0.786, R²st 0.031, UBS 0.210, dyn 0.198 | η² 0.068, R²abs 0.930, R²st 0.044, UBS 0.084, dyn 0.069 | η² 0.236, R²abs 0.749, R²st 0.130, UBS 0.249, dyn 0.240 |
| alive | η² 0.900, R²abs -0.102, R²st 0.899, UBS 0.320, dyn 0.998 | η² 0.735, R²abs 0.184, R²st 0.442, UBS 0.801, dyn 0.785 | η² 0.899, R²abs -0.106, R²st 0.894, UBS 0.329, dyn 0.998 | η² 0.741, R²abs 0.230, R²st 0.445, UBS 0.837, dyn 0.756 |
| deadwood_total | η² 0.207, R²abs 0.781, R²st 0.959, UBS 0.107, dyn 0.107 | η² 0.330, R²abs 0.669, R²st 0.978, UBS 0.089, dyn 0.147 | η² 0.358, R²abs 0.638, R²st 0.995, UBS 0.043, dyn 0.132 | η² 0.351, R²abs 0.644, R²st 0.988, UBS 0.069, dyn 0.150 |
| food_on_bushes_total | η² 0.342, R²abs 0.631, R²st 0.488, UBS 0.236, dyn 0.409 | η² 0.640, R²abs 0.337, R²st 0.626, UBS 0.446, dyn 0.424 | η² 0.835, R²abs 0.155, R²st 0.979, UBS 0.130, dyn 0.410 | η² 0.751, R²abs 0.232, R²st 0.684, UBS 0.472, dyn 0.508 |
| adults_working | η² 0.474, R²abs 0.491, R²st 0.324, UBS 0.202, dyn 0.474 | η² 0.438, R²abs 0.528, R²st 0.272, UBS 0.244, dyn 0.439 | η² 0.473, R²abs 0.492, R²st 0.331, UBS 0.204, dyn 0.476 | η² 0.480, R²abs 0.488, R²st 0.373, UBS 0.240, dyn 0.479 |

## Settlements alive at days 30/60/90/118 (test seeds): mean / distribution

- game_v16: mean [1.0, 1.0, 1.0, 1.0], distribution [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}]
- game_v17: mean [1.5, 1.75, 2.25, 2.25], distribution [{'1.0': 8, '2.0': 8}, {'1.0': 4, '2.0': 12}, {'1.0': 4, '2.0': 4, '3.0': 8}, {'1.0': 4, '2.0': 4, '3.0': 8}]
- game_v18_nofission: mean [1.0, 1.0, 1.0, 1.0], distribution [{'1.0': 16}, {'1.0': 16}, {'1.0': 16}, {'1.0': 16}]
- game_v18: mean [1.06, 1.25, 1.56, 2.06], distribution [{'1.0': 15, '2.0': 1}, {'1.0': 12, '2.0': 4}, {'1.0': 8, '2.0': 7, '3.0': 1}, {'1.0': 3, '2.0': 10, '3.0': 2, '4.0': 1}]

## Baseline nMAE at 168 h (test split): persistence / cal_abs / apfit_abs

- game_v16: food_stock 0.954 / 0.458 / 0.281, wood_stock 1.186 / 0.187 / 0.186, alive 0.154 / 0.829 / 0.118
- game_v17: food_stock 0.779 / 0.592 / 0.298, wood_stock 0.949 / 0.311 / 0.258, alive 0.149 / 0.758 / 0.137
- game_v18_nofission: food_stock 0.968 / 0.453 / 0.283, wood_stock 1.193 / 0.184 / 0.183, alive 0.148 / 0.831 / 0.111
- game_v18: food_stock 0.733 / 0.563 / 0.307, wood_stock 0.977 / 0.366 / 0.293, alive 0.139 / 0.709 / 0.160

## Attribution (delta eta2 / delta UBS)

- rain wet burn, region fill, daughter fixes, J6 fix: food_stock -0.160 / -0.231, wood_stock -0.123 / -0.126, alive 0.164 / -0.472, deadwood_total 0.029 / -0.046, food_on_bushes_total 0.195 / -0.316, adults_working 0.036 / -0.040
- hazard fission (with the region fill): food_stock 0.221 / 0.265, wood_stock 0.168 / 0.165, alive -0.158 / 0.509, deadwood_total -0.007 / 0.026, food_on_bushes_total -0.085 / 0.342, adults_working 0.006 / 0.036
- Pass 12 as a whole: food_stock 0.061 / 0.034, wood_stock 0.046 / 0.038, alive 0.006 / 0.036, deadwood_total 0.021 / -0.020, food_on_bushes_total 0.111 / 0.026, adults_working 0.042 / -0.004

## Paired bootstraps of delta UBS (90 % intervals)

- game_v18 - game_v17: food_stock 0.034 [-0.063, 0.140], wood_stock 0.038 [0.010, 0.076], alive 0.036 [-0.102, 0.163], deadwood_total -0.020 [-0.033, -0.009], food_on_bushes_total 0.026 [-0.046, 0.097], adults_working -0.004 [-0.036, 0.029]
- game_v18 - game_v18_nofission: food_stock 0.265 [0.187, 0.375], wood_stock 0.165 [0.141, 0.203], alive 0.509 [0.432, 0.630], deadwood_total 0.026 [0.019, 0.036], food_on_bushes_total 0.342 [0.295, 0.430], adults_working 0.036 [0.010, 0.065]
- game_v18_nofission - game_v17: food_stock -0.231 [-0.307, -0.182], wood_stock -0.126 [-0.144, -0.112], alive -0.472 [-0.616, -0.385], deadwood_total -0.046 [-0.057, -0.039], food_on_bushes_total -0.316 [-0.395, -0.264], adults_working -0.040 [-0.069, -0.013]

**J7 (game_v18): FAIL** — food_stock: PASS, wood_stock: FAIL, alive: PASS, ubs_alive: FAIL, ubs_food_stock: FAIL, r2_static_alive: FAIL

```json
{
 "alive": {
  "eta2": 0.7412896512030827,
  "passed": true,
  "r2_abs": 0.22985937590877936
 },
 "food_stock": {
  "eta2": 0.5321314789787777,
  "passed": true,
  "r2_abs": 0.4574327522863624
 },
 "r2_static_alive": {
  "passed": false,
  "v17": 0.4421577758201634,
  "v18": 0.4453603772554442
 },
 "ubs_alive": {
  "ci90": [
   -0.10154868054178447,
   0.16305784097303996
  ],
  "delta": 0.03616411539598685,
  "passed": false
 },
 "ubs_food_stock": {
  "ci90": [
   -0.06299095193302576,
   0.14015736866275885
  ],
  "delta": 0.033607299168411875,
  "passed": false
 },
 "wood_stock": {
  "eta2": 0.23617032150879463,
  "passed": false,
  "r2_abs": 0.7493956307751086
 }
}
```
