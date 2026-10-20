# WM-5 — CRN and the Causal Transformer as their authors train them (GOAL.md 7.4)

WM-4's `METHODS.md` (D1–D15) lists every deviation of WM-4's CRN (Bica et al., ICLR 2020) and Causal Transformer (CT;
Melnychuk et al., ICML 2022) from the papers and from the authors' implementation (`github.com/Valentyn1997/
CausalTransformer` at `c49a24fa`, MIT). WM-5 keeps all of them except where a line below changes one, and keeps the
vendored modules (`tools/worldmodel/wm4/third_party/causal_transformer/`) exactly as WM-4 vendored them. Every change
lives in `tools/worldmodel/wm5/causal5.py`, which imports `wm4.causal` (models, data pipeline, rollouts, `val_roll`)
and replaces only the two training loops, the checkpoint bookkeeping and the readout.

## Changes against WM-4

- **D16 Step plan instead of a time cap (replaces D7's cap).** Each stage trains for a planned number of optimizer
  steps equal to the authors' (CRN encoder 15,700, CRN decoder 53,800, CT 11,850; the same numbers WM-4 derived in D7
  from the authors' epochs × batches on 10,000 sequences). The plan is reduced only if the 300-s cost check on the
  actual dataset predicts that the stage(s) exceed the pre-registered budget (CRN 5,400 s for both stages, CT
  14,400 s): planned = floor((budget − 15 × the measured evaluation time) / measured s per step), both CRN stages scaled
  by one factor. A safety cap of 1.25 × the budget stops a run that is slower than predicted; such a run reports
  `stop` = "time" and its reached steps. *Reason:* WM-4's time caps stopped CT at 4 % of the authors' steps (D14); a
  reviewer would rightly call that not training CT.
- **D17 α on real progress (replaces D7's nominal epochs).** α = `alpha_at(step / planned_steps, α_max)` (the authors'
  `AlphaRise` "exp" rule, 2 / (1 + e^(−10 p)) − 1), set before every optimizer step (the authors update it at each epoch
  end; with hundreds of epochs the two schedules coincide to < 1 % of α_max), so a run that completes its plan ends at
  α = 0.99991 α_max. *Reason:* WM-4 tied α to nominal epochs the budget never reached, so CT's kept checkpoints had
  α ≈ 0.2 α_max.
- **D18 Last checkpoint as primary (replaces D8 as the primary rule).** The authors keep the last epoch. WM-5's primary
  checkpoint is the last optimizer step: CT's EMA weights (β 0.99) at the last step; CRN's last encoder with the last
  decoder (the decoder is trained on the last encoder). The secondary checkpoint is WM-4's rule, the lowest `val_roll`
  evaluation: CT's EMA weights at that evaluation; for CRN, the last encoder with its lowest-`val_roll` decoder (a
  different encoder would need its own decoder training; the encoder's lowest one-step loss is logged only). Both are
  evaluated on test.
- **D19 Evaluations per stage.** At most 15 `val_roll` evaluations per stage (14 evenly spaced over the plan and the
  stage's end), on the whole val set unless one evaluation takes longer than 60 s, then on a fixed sample of the first
  120 val episodes (recorded in `val_metrics.json`). WM-4 evaluated every budget / 15 seconds.
- **D20 Rounding at readout (replaces D13's "no rounding").** M1's readout rounds the V features, cumulative cold hours
  and cumulative deaths when `round_counts` (`wm3/effects3.py` `_chunk_h` L149–150). WM-5 applies the same rule to
  CRN / CT at the readout tokens: the food store and alive (V features) are rounded at the 7- and 30-token marks before
  the arm − none difference, and cumulative deaths (the true deaths at c plus the summed predicted daily deaths of the
  five causes) are rounded; settlements alive is not rounded (M1 reads it from an extra channel it does not round).
  Rounding never enters the autoregressive feedback. Unrounded predictions are written beside the rounded ones and
  reported as secondary. *Reason:* WM-4's fractional death effects (deaths cells 1.4–4.2 vs 0.3–0.6 for the other
  methods) were partly an artifact of not rounding.
- **D21 obs2 data.** `obs2_v19`: the 1,200 observed train and 200 observed val episodes (one per world), daily tokens
  at φ = 06:00 as `obs_v19` (every obs2 cast is at 06:00). A token's treatment carries the cast of its 24 rows, so every
  earlier cast of a world appears in the history tokens (`wm4.tokens.encode`; the refractory period of the policy keeps
  casts ≥ 8 days apart, so a token never holds two). Test evaluation on obs2 rolls out from the case's `pre` member (the
  world's casts before c, nothing at or after c), at the cast row's phase, as D13.
- **D22 Configurations.** WM-4's grid choices are reused without a new grid: CRN configuration 3 (encoder BR 64,
  FC 128, dropout 0.1; decoder BR 64, FC 32), CT configuration 2 (lr 1e-3, BR 64, FC 128); every other setting as D10.
- **D23 What did not change.** Daily tokens (D1), tallies as daily sums (D2), the 37-dimensional outcome vector and
  normalizer (D3), covariates and no static features (D4), the 4-class treatment plus amount (D5), horizons and the CRN
  decoder windows τ = 29 (D6), CT's EMA used for evaluation only (D9), sizes and optimizers (D10), training data (D12).
- **D24 M1 action history on obs2.** `wm3.effects3.predictions` replaces the whole action array of the history by
  zeros plus the arm at c (as in WM-2 … WM-4, where no history held a cast). On obs2 an earlier cast lies at least
  8 days (192 rows) before c, outside M1's 168-row warm-up window, so the zeroed history never removes a cast that the
  model would read. M1 rollouts on obs2 are 720 rows (both lags; the split shift is not predicted there).

## Cost checks

Filled in from `runs/_cost_*/cost.json` (GOAL.md 9.1): see the table in WM5.md and the judge's runs table.
