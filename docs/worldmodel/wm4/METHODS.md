# WM-4 — CRN and the Causal Transformer on WorldSim (GOAL.md 7.4)

What was implemented, from what source, and **every deviation** from the papers — Counterfactual Recurrent Network
(CRN; Bica, Alaa, Jordon and van der Schaar, ICLR 2020) and Causal Transformer (CT; Melnychuk, Frauen and Feuerriegel,
ICML 2022) — and from the authors' public implementation (`github.com/Valentyn1997/CausalTransformer` at
`c49a24fa`, MIT; it contains both methods). Code: `tools/worldmodel/wm4/causal.py`, `tokens.py`; vendored modules:
`tools/worldmodel/wm4/third_party/causal_transformer/` (`SOURCE.md`).

## Source

Vendored unchanged (plain PyTorch only): `VariationalLSTM` (`utils_lstm.py`), `grad_reverse`, `bce`,
`BRTreatmentOutcomeHead` (`utils.py`, the file's Lightning / MLflow parts removed), `TransformerMultiInputBlock`,
`MultiHeadedAttention`, `RelativePositionalEncoding`, `LayerNorm`, `PositionwiseFeedForward` (`utils_transformer.py`).
Written for WM-4 (no Lightning, Hydra, MLflow or Ray): the data pipeline, both training loops, the α schedule (the
authors' `AlphaRise` "exp" rule), the EMA helper (`torch_ema`'s update rule), checkpoint selection, rollouts and the
prediction contract. No extra package was installed (the WM-2/3 venv is untouched).

## The setting (both methods)

| item | papers (tumour-growth simulator) | WM-4 |
|---|---|---|
| time step | one day | one day: a token every 24 hourly rows at a clock phase (D1) |
| outcome | tumour volume (1-d), predicted as a level | the 37 state features of WM-2's action-aware input list, predicted as normalized daily changes and fed back (D2, D3) |
| time-varying covariates ("vitals") | none (cancer) / vitals (MIMIC) | the 30 calendar features at the token, history only (D4) |
| static features | patient type | none (D4) |
| treatments | 4 classes (none, chemo, radio, both) | 4 classes (none, gift_food, spoil_food, rot_wood) + the amount / `ACTION_SCALE` (D5) |
| horizon | τ ≤ 5 (CT), ≤ 10 (CRN) | 7 and 30 tokens (D6) |
| assignment | confounded by the tumour diameter, strength γ | obs split: the WM-4 policy (METRICS.md part a); v2: a fixed rule (unconfounded) |

## Deviations

- **D1 Daily tokens.** An hourly episode is sampled at rows r0 + 24 k (k = 0 … 119). On the obs split every cast is at
  06:00, so r0 = 22 (06:00) for training and evaluation and every decision row is a token. On v2 casts are at 06:00
  (`L`) or 18:00 (`E`), so training draws r0 uniformly in 0 … 23 per sequence and epoch, and evaluation uses the cast
  row's phase (validation: both 06:00 and 18:00). The token's treatment is the sum of the actions requested over its 24
  rows r … r + 23, so the cast falls in exactly one token, the token whose state is the pre-cast state.
- **D2 Tallies as daily sums.** The hourly table holds the tallies (`d.*`, e.g. deaths by cause, births) as hourly
  differences; a token carries their sum over the 24 rows ending at it (the day before the token). Levels are read at
  the token row. Cumulative deaths at c + 7 d / c + 30 d = the true deaths at c + the summed predicted daily deaths of
  the five causes.
- **D3 Outcome vector.** All 37 state features are outcomes (the papers have one): the target of token t is
  (state_{t+1} − state_t) / the train std of the daily change; the next state is fed back, clamped ≥ 0 (every feature
  is a count, a stock or a flag) and ≤ 1 for `s.fire_lit` and `s.present` (binary in the hourly data, treated as
  continuous at daily resolution). Loss: MSE averaged over the 37 normalized targets (no feature weights; M1's
  `v_weight` is not used). Normalizer: mean / std of the daily state and std of the daily change over every train row
  r ≥ 23 of every phase.
- **D4 Covariates, no static features.** The calendar (hour one-hot, season one-hot, day of season / 7) is the only
  covariate. CT treats it as the authors treat vitals: masked (zeroed, excluded from attention) after the last observed
  token at prediction, and training batches are augmented with a copy masked after a random split (`augment_with_masked
  _vitals`). CRN's encoder reads it, its decoder does not (as the authors' decoder: previous treatments and outcomes
  only). There are no static features; CT's static term x_s is zero.
- **D5 Treatment encoding.** `BRTreatmentOutcomeHead.linear3` (the treatment classifier) outputs 4 classes; the outcome
  head reads [BR, 5-value treatment] (the amount distinguishes `sp120L` from `sp240L` on v2). Cross entropy is
  `F.cross_entropy(..., reduction="none")` (the authors' `bce(..., "multiclass")` with the deprecated `reduce=False`).
- **D6 Horizons and the CRN decoder.** Effects are read 7 and 30 tokens after the cast token. CRN: the encoder's head
  predicts the first step (with the planned treatment), the decoder (initialized with h = c = the encoder's BR at the
  last observed token, as the authors' `init_state`) the next 29; the decoder is trained with teacher forcing on every
  window of τ = 29 steps (k = 0 … 89 per sequence, the authors' rolling-origin explosion). CT: autoregressive over the
  whole sequence for 30 steps (the authors' `get_autoregressive_predictions`).
- **D7 Number of optimizer steps.** The authors train on 10,000 sequences (100 epochs for CRN at batch 64, CRN decoder
  ≈ 55 windows per sequence at batch 1,024; 150 epochs for CT at batch 128). WM-4 has 400 (obs) or ≈ 3,800 (v2) train
  sequences, so the number of epochs is scaled to give the authors' number of optimizer steps (CRN encoder 15,700, CRN
  decoder 53,800, CT 11,850), each run capped by its time budget (METRICS.md part b); α's schedule runs over those
  nominal epochs, so a run stopped by its budget never reaches α_max (D14).
- **D8 Checkpoint selection.** The authors keep the last epoch. WM-4 keeps the epoch with the lowest validation
  factual loss (GOAL.md 7.4): CRN encoder — one-step teacher-forced MSE; CRN decoder and CT — `val_roll`, the MSE (train
  std units) of 30-token factual rollouts with the observed treatments from tokens 6, 18, 30, 42, 54 (days 8, 20, 32,
  44, 56) of every validation episode. Evaluations at most every budget / 15 seconds (≤ 16 per stage), because a CT
  validation rollout re-encodes the whole sequence 30 times (≈ 40 s for 14 starts per episode in the dry run, beside a
  training).
- **D9 CT's EMA.** The authors keep an EMA (β 0.99) of both parameter groups and use the other group's EMA weights inside
  each update's forward pass (`torch_ema.average_parameters`); WM-4 updates the EMAs with the same rule but uses them
  only for evaluation and the kept checkpoint (swapping inside a forward pass that is then back-propagated would modify
  saved tensors in place).
- **D10 Sizes (GOAL.md).** CRN encoder hidden size 128 (the authors' cancer γ = 4 configuration has 24), BR 24, FC 96,
  dropout 0.2, lr 1e-3, batch 64; decoder hidden = the encoder's BR (24), BR 24, FC 12, dropout 0.1, lr 1e-3, batch
  1,024; α_max 1 (gradient reversal). CT d_model 128, 2 layers, 4 heads (the authors' γ = 4: 12 / 16, 1 layer, 2 heads),
  BR 16, FC 48, dropout 0.1 (attention dropout too), lr 0.01, batch 128, α 0.01 (domain confusion), β 0.99, relative
  positional encodings for keys and values (trainable, max distance 15, shared by the blocks), no absolute encoding.
  Adam without weight decay, no gradient clipping, no learning-rate schedule (as the authors).
- **D11 Tuning grid** (GOAL.md 7.4; METRICS.md part b): at most four configurations per method on `obs_v19`, validation
  factual loss only, seed 0, each capped at 20 minutes; the chosen one is `<method>_obs_s0` and is reused for every other
  seed and for v2.
- **D12 Training data.** obs: the 400 observed train episodes (one per world, 06:00 tokens). v2: every member of the
  400 train families (factual and arms, both replicates), as WM-2's M1b-act. Validation: the 80 observed val episodes
  (obs); the 80 `v19_fact` val episodes (v2).
- **D14 Training actually reached (review round 1, M1).** Because the budgets stop the runs long before the nominal
  epochs, the kept checkpoints are early in α's ramp. CT-obs (config 2): kept epoch 158 of 3,950 nominal = **477
  optimizer steps, 4 % of the authors' 11,850**, α = 0.0020 (20 % of α_max 0.01; 1,437 steps and α 0.0054 at the time
  stop). CT-obs s1: kept at 477 steps (α 0.0020). CT-v2 s0 / s1: kept at epoch 16 = 493 steps (α 0.0019) of 2,668 /
  2,755 run. CT's validation factual loss rose after its kept checkpoints (CT-obs s0 0.291, **0.273**, 0.310, 0.337,
  0.400, 0.343, 0.341 at 240 … 1,437 steps; CT-v2 s0 1.489, **1.484**, 1.541 … 2.004 at the stop), so these are early
  checkpoints chosen on purpose by the selection rule, with α's ramp tied to the nominal epochs. CRN-obs s0 (config 3):
  kept encoder 1,878 steps (α 0.53), kept decoder 8,435 steps (α 0.65); CRN-obs s1: encoder 1,566 (α 0.46), decoder
  41,195 (α 1.0). CRN-v2 s0 / s1 ran **every** nominal epoch (encoder 15,753, decoder 54,116 steps): kept encoder 7,257 /
  15,753 steps (α 0.98 / 1.0), decoder 33,200 / 49,136 (α 1.0). So CRN-v2 is trained as the authors train it; CT is not a
  test of domain confusion as such.
- **D15 CRN's two stages share the budget.** The encoder gets the first half of a run's budget (`enc_time_share` 0.5),
  the decoder the rest; every grid encoder stopped on time. `CRNEncoder.out_dropout` is assigned and never used (no
  effect: the authors' output dropout is inside `VariationalLSTM` / the head).
- **D13 Prediction contract** (shared with the baselines, GOAL.md 7.4): for a case (world, arm, c) the predicted effect
  = the rollout with the arm at the cast token (none after) − the rollout with none, both from the same observed history
  (`none_r` of the family) at the cast row's phase. No rounding of counts. The restricted split-day shift is not
  predicted (30-token rollouts end before most splits): it is absent from CRN's and CT's tables.

## Cost checks and the grid

Cost checks (`runs/_cost_crn`, `runs/_cost_ct`; 300-s trainings on `obs_v19`, train and val data only):

| method | s / optimizer step | validation (`val_roll`) | peak GPU | note |
|---|---|---|---|---|
| CRN | encoder ≈ 0.069, decoder ≈ 0.029 | ≈ 0.05 s | 477 MiB | beside the `m1b_obs_s0` evaluations |
| CT | ≈ 0.83 (batch 128 doubled by the masked-covariate copy) | 5.3 s | 14,343 MiB | GPU otherwise idle |

Within the pre-registered 1,200 s a CRN grid member reached 9,366–14,148 encoder and 32,340–53,830 decoder steps of
the authors' 15,700 / 53,800; a CT member ran ≈ 1,440 of 11,850 steps (12 %; DECISIONS L4.4) and the kept CT-obs checkpoint has 477 (4 %, D14). The grid (seed 0, `obs_v19`, val
factual loss `val_roll` only; the chosen member is the `_obs_s0` run):

| method | config | overrides of the defaults | `val_roll` | seconds | training reached | chosen |
|---|---|---|---|---|---|---|
| CRN | 0 | `{}` | 0.5919 | 1200 | encoder 9366 steps (best epoch 520 of 1561), decoder 32340 steps (best epoch 120 of 924, time) | |
| CRN | 1 | `{"dec_br": 64, "dec_fc": 32, "enc_br": 64, "enc_fc": 128}` | 0.4249 | 1200 | encoder 11046 steps (best epoch 520 of 1841), decoder 45605 steps (best epoch 1302 of 1303, time) | |
| CRN | 2 | `{"dec_lr": 0.0001, "enc_lr": 0.0001}` | 0.5232 | 1200 | encoder 13398 steps (best epoch 2232 of 2233), decoder 53095 steps (best epoch 420 of 1517, time) | |
| CRN | 3 | `{"dec_br": 64, "dec_fc": 32, "enc_br": 64, "enc_dropout": 0.1, "enc_fc": 128}` | 0.4132 | 1180 | encoder 14148 steps (best epoch 312 of 2358), decoder 53830 steps (best epoch 240 of 1538, epochs) | **yes** |
| CT | 0 | `{}` | 1.1813 | 1207 | 1440 steps (best epoch 237 of 480, time) | |
| CT | 1 | `{"lr": 0.001}` | 0.3098 | 1207 | 1443 steps (best epoch 316 of 481, time) | |
| CT | 2 | `{"br": 64, "fc": 128, "lr": 0.001}` | 0.2727 | 1206 | 1437 steps (best epoch 158 of 479, time) | **yes** |
| CT | 3 | `{"br": 64, "fc": 128, "lr": 0.0001}` | 0.2855 | 1207 | 1440 steps (best epoch 479 of 480, time) | |

Both methods prefer a wider balanced representation (BR 64) than the authors' tumour defaults (BR 24 / 16): the outcome here is 37-dimensional, not 1.
