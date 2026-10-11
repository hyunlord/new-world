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
| M2 | agent model (architecture chosen on val: set transformer `m2_set_s0`, val score 1.109, over the MLP `m2_mlp_s0`, 1.222; only the chosen one is test-evaluated) | — | 0 | `m2_set_s0` |
| M3 | text world model (LoRA) | — | 0 | `m3_s0` (optional, first to drop) |

A recipe change after a failed target (GOAL.md 0.3.7) gets a new run id suffix (`_r2`, `_r3`) and is added to
this table before its test evaluation.

### Recipe r2 (added 18:47, before any r2 test number; DECISIONS D2.2)

| Config | Model | Variant | Seeds | Run ids |
|---|---|---|---|---|
| M1-main r2 | M1 GRU, recipe r2 | sev-blind, action-aware | 0, 1, 2 | `m1gru_main_r2_s0`, `m1gru_main_r2_s1`, `m1gru_main_r2_s2` |
| M1-sevon r2 | M1 GRU, recipe r2 | sev-on, action-aware | 0 | `m1gru_sevon_aa_r2_s0` |
| M1-blind-noact r2 | M1 GRU, recipe r2 | sev-blind, action-blind | 0 | `m1gru_blind_ab_r2_s0` |
| M1-sevon-noact r2 | M1 GRU, recipe r2 | sev-on, action-blind | 0 | `m1gru_sevon_ab_r2_s0` |
| M1-main r2 rerun (W7) | M1 GRU, recipe r2 | sev-blind, action-aware | 0 | `m1gru_main_r2_s0_rerun` |
| M1b r2 | transformer, recipe r2 | sev-blind, action-aware | 0 | `m1tf_main_r2_s0` (optional) |

### Recipe r3 (added 20:10, before any r3 number; DECISIONS D2.5)

| Config | Model | Variant | Seeds | Run ids |
|---|---|---|---|---|
| M1-main r3 | M1 GRU, recipe r3 | sev-blind, action-aware | 0, 1, 2 | `m1gru_main_r3_s0`, `m1gru_main_r3_s1`, `m1gru_main_r3_s2` |
| M1-main r3 rerun (W7) | M1 GRU, recipe r3 | sev-blind, action-aware | 0 | `m1gru_main_r3_s0_rerun` |
| M1-sevon r3 | M1 GRU, recipe r3 | sev-on, action-aware | 0 | `m1gru_sevon_aa_r3_s0` |
| M1-blind-noact r3 | M1 GRU, recipe r3 | sev-blind, action-blind | 0 | `m1gru_blind_ab_r3_s0` |
| M1-sevon-noact r3 | M1 GRU, recipe r3 | sev-on, action-blind | 0 | `m1gru_sevon_ab_r3_s0` |

The main recipe of the W table is chosen on validation (T2 168 h of the main seed-0 run, then T1), never on test.

