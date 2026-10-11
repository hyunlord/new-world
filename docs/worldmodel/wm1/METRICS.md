# WM-1 metrics

Every run: id, configuration, validation and test numbers, DGX run directory, retrain command. Runs live under
`/home/hyunlord/work/wm1-runs/<run_id>/` on the DGX (`config.json`, `train_log.jsonl`, `val_metrics.json`,
`metrics.json` = test, `best.pt`). Checkpoints are not in git.

## Configurations evaluated on test (listed before any test number exists — GOAL.md 4.2)

Test roles, OOD_v12, OOD_long and C_cf test are evaluated once per configuration below, by
`m1_gru.py final` (or the model's own `final`), after training and validation are finished.

| Config | Model | Variant | Seeds | Run ids |
|---|---|---|---|---|
| M1-main | M1 GRU | sev-blind, action-aware | 0, 1, 2 | `m1gru_main_s0`, `m1gru_main_s1`, `m1gru_main_s2` |
| M1-sevon | M1 GRU | sev-on, action-aware | 0 | `m1gru_sevon_aa_s0` |
| M1-blind-noact | M1 GRU | sev-blind, action-blind | 0 | `m1gru_blind_ab_s0` |
| M1-sevon-noact | M1 GRU | sev-on, action-blind | 0 | `m1gru_sevon_ab_s0` |
| M1-main rerun (W7) | M1 GRU | sev-blind, action-aware | 0 (same seed) | `m1gru_main_s0_rerun` |
| M1b | causal transformer | sev-blind, action-aware | 0 | `m1tf_main_s0` (optional, drop order) |
| M2 | agent model | — | 0 | `m2_s0` |
| M3 | text world model (LoRA) | — | 0 | `m3_s0` (optional, first to drop) |

A recipe change after a failed target (GOAL.md 0.3.7) gets a new run id suffix (`_r2`, `_r3`) and is added to
this table before its test evaluation.
