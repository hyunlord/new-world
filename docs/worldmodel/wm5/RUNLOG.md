# WM-5 — run log

Times are KST (Mac clock). The 46-hour training deadline (GOAL.md 0.3.5) is 2026-10-20 19:13:47 KST plus recorded
pauses.

| Part | Start | End | Notes |
|---|---|---|---|
| start | 2026-10-18 21:13:47 | | Mac: Darwin 27.0.0 arm64 (T6041). DGX (21:15:40): Linux 6.14.0-1015-nvidia aarch64, GB10, driver 580.126.09, CUDA 13.0, `nproc` 20; `nvidia-smi`: Xorg (126 MiB), gnome-shell (18 MiB) and another user's ComfyUI python (29,209 MiB), 0 % util; RAM 119 GiB, 62 GiB available; swap 0 B; disk 3.7 T, 1.3 T free; load average 1.60 / 3.74 / 2.85; venv torch 2.14.1+cu130, numpy 2.5.2. No git hooks in the Mac clone; `.claude/settings.json` = `{}` with skip-worktree `S`. Base `64d633ff` = both WM branches; DGX code clone at `64d633ff`, clean |
| 0 | 21:15 | | sha256 on the DGX equal to their recorded values; WM-4 checkpoints recorded (DECISIONS L0.3); seeds 47000–48639 unused (L0.4) |
| 1 (6.2.1) | 21:17:47 | | round 0 (`record_obs2.sh r0`, code `a15e1549`): `--seeds 47000..48640 --days 120 --every 60`, 8 workers (no training yet), nice 10 → `obs2/rounds/r0/` |
| 1 (6.2.2) | 21:24:24 | 21:24:24 | METRICS.md part a committed and pushed (`2a97b4cd`) before any round-1 episode |
| 4 (cost) | 21:29:44 | 21:35:44 | `_cost_crn_v2` (code `38bea0c5`, 300 s on `v2_v19`, beside round 0, load average 22–27): encoder 0.097 s/step, decoder 0.040 s/step, `val_roll` 0.14 s (80 val episodes, 800 jobs), peak GPU 973 MiB → predicted 3,694 s for the authors' 15,700 + 53,800 steps < 5,400 s: plan not reduced |
| 4.1 | 21:35:44 | | `crn_v2_s0` started (queue, `--budget 5400`, safety cap 6,750 s) |
| 3 | 21:39:47 | 21:39:47 | METRICS.md part b committed and pushed (`6e9e88ed`); `find` under `wm1-runs/wm5/` (outside `_code/`): no `effects*.json`, `obs2_base.json`, `ridge_*.json`, `floor*.json`, `rescored*.json`; `_setup/prereg_sha` = `6e9e88ed` |
| 4.5 (D2) | 21:40:41 | 21:42:15 | `finals_rescore` (`finals5.sh rescore`, code `6e9e88ed`): WM-4's eight CRN / CT runs re-scored with rounding on their WM-4 test splits (3–35 s each); unrounded readouts equal WM-4's committed predictions (≤ 9e-16) except `ct_obs_s1` (GPU float nondeterminism, max 0.30; DECISIONS L4.2) |
| 6 | 21:47 | | references: a `general-purpose` subagent verified 28 entries on the web (22:05); `references.bib`, `REFERENCES_CHECK.md` |
| 1 (6.2.1) | | 22:08:39 | round 0: 1,640 episodes, all verified, 2,881 rows (≈ 51 min at 8 workers beside `crn_v2_s0`, load average 10–27) |
| 1 (6.2.3) | 22:08:43 | | `obs2_pipeline.sh` (code `e875e413`, 4 workers): round 1 schedules (1,640 worlds read, 1,587 with a first act: 552 `gf240`, 702 `sp240`, 333 `rw120`), recording |
| 4.1 | 22:19:44 | | `crn_v2_s0` ended: planned 15,700 + 53,800 steps both reached, α 0.9999 α_max, 2,618 s (load average 10–36); test effects (last, best) 22:20:47 / 22:20:54 (`finals_watch_v2`) |
| 4 (cost) | 22:19:44 | 22:26:44 | `_cost_ct_v2` (300 s, beside round 1 at 4 workers, load average 33–36): 0.998 s/step, `val_roll` 11.7 s, peak GPU 13,013 MiB → predicted 11,998 s for 11,850 steps < 14,400 s: plan not reduced |
| 4.1 | 22:26:44 | | `ct_v2_s0` started (≈ 0.95 s/step) |
| 1 (6.2.3) | | 23:51:29 | round 1: 1,587 worlds recorded (≈ 13–17 episodes / min at 4 workers), `round1_check` all OK (prefix, casts, sha256; 1,580 differ after the act) |
| 1 (6.2.3) | 23:51:33 | 01:08:53 | round 2: 1,318 worlds with a second act (550 `gf240`, 494 `sp240`, 274 `rw120`), `round2_check` all OK (1,305 differ after the act) |
| 1 (6.2.3) | 01:08:55 | | round 3: 757 worlds with a third act (319 / 299 / 139) |
| 4.1 | 00:50 | | `ct_v2_s0` slowed from ≈ 1.0 to 1.65–2.1 s/step: another user's ComfyUI process computing on the shared GPU (22 GiB, 95 % utilization; not ours, untouched); projected end ≈ 14,500 s, inside the 18,000-s safety cap |
| 4.1 | | 02:05 | `ct_v2_s0` ended: 11,850 / 11,850 steps, α 0.99991 α_max, 13,114 s (GPU shared with another user's job from ≈ 00:50); lowest `val_roll` at step 790; test effects (last, best) by `finals_watch_v2`; `crn_v2_s1` started |
| 1 (6.2.3) | 01:53:00 | | round 3 verified (757); round 4: 265 worlds with a fourth act (134 / 84 / 47) |
| 1 (6.2.3–6.2.4) | 02:08:57 | 02:11:30 | round 4 verified (265; recorded in 954 s); round 5: 39 worlds with a fifth act (144 s), verified; round 6: no new act (every world capped, past day 60 or undrawn) → rounds end. Round recording times 6,140 / 4,619 / 2,637 / 954 / 144 s. `finalize`: `observed/` symlinks, the policy replayed on every observed episode reproduces every round's acts and rows (1,640 / 1,640); 3,966 acts (2.42 per world), never 3.2 %, 60.0 % of acts with an earlier act in the history |
| 1 (6.2.5) | 02:11:30 | | family schedules; families recording (val then test, one call per (world, ref), 4 workers) |
| 1 (6.2.6) | 02:14 | 02:15 | observed-only manifest (1,640 episodes verified) committed `835a8670` (DECISIONS L1.4); `obs2_v19` loads from it (val 200 episodes); obs2 queue lines appended after `ct_v2_s1` (cost checks, `m1b_obs2_s0`, `crn_obs2_s0`, `ct_obs2_s0`, `crn_obs2_s1`, `ct_obs2_s1`, `m1b_obs2_s1`) |
| 4.1 | 02:05:46 | 02:54:51 | `crn_v2_s1`: planned 15,700 + 53,800 reached, α 0.9999 α_max, 2,905 s; test effects 02:54:56 / 02:54:59; `ct_v2_s1` started |
| 1 (6.2.5) | 02:11:30 | 04:01:45 | val families: 400 recorder calls (c1, c2; replicate 0), linked; test families recording |
| 1 (6.2.6) | 04:02:12 | 04:02:31 | `describe` (`finals5.sh describe`, train worlds + val families only): IPW ESS 396–445 (7 d) and 127–141 (30 d) per act; DATASET_OBS2.md (train / val parts) |
| 4.1 | 02:54:47 | 06:15:49 | `ct_v2_s1`: 11,850 / 11,850 steps, α 0.99991 α_max, 12,051 s (GPU shared at times); lowest `val_roll` at step 1,580; test effects by the watcher. The four v2 runs are done; `_cost_m1b_obs2` started |
| 1 (6.2.5) | 04:01:45 | 08:27:22 | test families: 480 recorder calls (c1, c2; replicates 0, 1), linked; unforked recordings of 3 sampled test worlds |
| pause | 08:18 | 10:27 | **SSH stall** (Mac ↔ DGX): the connection multiplexer broke ("mux_client_request_session: read from master failed"), every DGX call hung; DGX jobs continued (`m1b_obs2_s0` training). ≈ 2.1 h of wall time without control; recorded as a pause (GOAL.md 0.3.5) |
| 1 (6.2.5) | 08:28:02 | 10:27:52 | first `fam-check` failed 417 / 880 (a checker bug: the recorder's extra `none` member compared with the observed episode, DECISIONS L1.5); corrected `da300d9c`; re-run: 880 / 880 OK (none_r0 = factual in all 880, pre_r0 = unforked in 3) |
| 1 (6.2.6) | 10:27:52 | 10:28:40 | full manifest (7,080 episodes, every one sha256-verified); its 1,640 observed entries are identical to the observed-only manifest of `835a8670` |
| 4.1 | 06:40 | 10:29 | `m1b_obs2_s0`: stop `max_steps` (8,000), kept step 7,950 (val T2 24 h 0.0622; not converged by WM-3's definition), 12,992 s; `_cost_crn_obs2` started |
| 4.2 | 10:29:21 | 10:30:16 | `finals_watch_obs2` (`FINALS_SHA` `f7cf620a`, DECISIONS L4.3): `obs2_base` (val selection `zero` at both lags), `ridge_obs`, `ridge_priv` (DGX nearly idle, load ≈ 2); then the M1 / CRN / CT references on the obs2 test |
