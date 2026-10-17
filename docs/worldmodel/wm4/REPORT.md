## 구현 완료 보고

### 구현 의도
WM-2/WM-3는 데이터셋 v2(모든 갈래를 모든 세계에 고정 규칙으로 투입, 짝 갈래, 노이즈 바닥)에서 자기회귀 세계 모형이 신의 개입을 무시하고, 짝 갈래 학습이 첫 주 효과만 가르친다는 것을 보였다. 벤치마크 논문으로 가기 전에 남은 두 구멍은 (1) 이 분야의 방법 — 시간 가변 처치 효과 추정기 CRN(Bica 외, ICLR 2020)과 Causal Transformer(CT, Melnychuk 외, ICML 2022) — 이 없다는 것, (2) 그 방법들이 존재하는 이유인 **교란된 처치 배정**이 v2에 없다는 것이었다. WM-4는 같은 고정 세계에서 교란된 관측 분할을 녹화하고, CRN·CT를 두 분할 모두에서 M1 모델·관측 기준선과 나란히 돌리며, WM-3의 두 미결 항목(절제 시드, 위험도 head 보정)을 닫는다.

### 구현 내용
(Part별 파일과 핵심 구조)
- **Part 0** `docs/worldmodel/wm4/GOAL.md`(원문 그대로), `DECISIONS.md`, `RUNLOG.md`: 시작 시각, 기계, 녹화기·WM-2/WM-3 체크포인트 sha256, 시드 미사용 확인, 40시간 마감.
- **Part 1** `tools/worldmodel/wm4/policy.py`(결정 행, 상태 f·w, 삼분위, 일일 확률, 추첨), `obs_split.py`(검증, 삼분위, 정책, 관측·가족 점검, 매니페스트), `launch/record_obs.sh`, `launch/post_families.sh`; `docs/worldmodel/wm4/obs_manifest.json`(1,200 에피소드), `DATASET_OBS.md`, `runs/record/`.
- **Part 2** `effects4.py`(관측 사례, 하한 척도, naive·ipw·match5 기준선, val 선택, M1의 관측 테스트 예측), `m1_obs.py`(`obs_v19` 등록, M1b-act 레시피, 정지 감시), `tokens.py`(일 단위 토큰), `causal.py`(CRN·CT 학습·그리드·롤아웃·예측 계약), `third_party/causal_transformer/`(원저자 순수 PyTorch 모듈, MIT, `SOURCE.md`), `recal_hazard.py`(Platt 재보정), `test_wm4.py`(22개 테스트), `METHODS.md`(논문 대비 편차 D1–D15).
- **Part 3** `METRICS.md` part a(정책·삼분위, 처치 녹화 전), part b(실행·예산·그리드·선택·규칙, 테스트 수치 전).
- **Part 4** `launch/queue.sh`·`queue_run.txt`(단일 학습 큐), `finals4.sh`·`finals_watch.sh`(사전 등록 커밋에서 1회 평가), `cost4.sh`, `recal.sh`, `dryrun.sh`, `fetch4.sh`, `gate.sh`; `docs/worldmodel/wm4/runs/<run>/`(작은 파일만).
- **Part 5** `judge4.py` → `runs/judge4/judge.md`, `judge.json`.
- **Part 6** `WM4.md`(분석, 논문 개요), `docs/worldmodel/wm3/WM3.md` 맨 위 한 줄.
- **Part 7** `REVIEW.md`(독립 리뷰 2회), 게이트, 이 보고서.

### 구현 방법
- **정책과 교란.** 표시일 8–60일 06:00(53개 결정 행)에 사실 궤적만 읽고 1인당 식량 f, 1인당 나무 w를 계산. f의 33·67 백분위(8.0792, 12.1)와 w의 67 백분위(3.1119)는 학습 세계 400개의 사실 에피소드에서 정해 METRICS part a에 고정(`bfa36208`, 처치 녹화 전). 굶는 세계는 선물(0.030/일), 배부른 세계는 부패(0.030/일), 나무 많은 세계는 썩힘(0.012/일); 모든 상태에서 모든 갈래 확률 > 0(겹침). 추첨 u = `default_rng([seed, day]).random()`, 세계당 최대 1회.
- **관측 분할 녹화와 가족.** WM-2의 고정 녹화기로 사실 560 → 정책 → 관측 1,019(`_none` 560 + `_obs` 459) → val·test 세계마다 기준 행 c_w(투입 행, 없으면 30일)에서 갈래를 친 가족 160개(세계별 녹화기 호출, replicate 0). 관측 에피소드는 투입 행까지 사실과 배열 단위로 동일, `none_r0`=사실, 투입 세계의 `_obs`=같은 갈래 가족 구성원(배열별 sha256).
- **관측 기준선.** 학습 관측 에피소드와 기록된 성향만 사용. `naive`(처치 평균 − 대조 평균), `ipw`(처치 1/p_a(c), 대조 1/∏(1−p_tot), Hájek), `match5`(an5 거리 최근접 처치 5·동일일 대조 5). 지평별로 val 가족 12셀 평균으로 선택(→ 두 지평 모두 `zero`). 합성 교란 정책 단위 테스트에서 naive는 60 이상 편향, ipw는 참값 ±15.
- **CRN·CT 이식과 편차.** 원저자 저장소(`Valentyn1997/CausalTransformer` `c49a24fa`, MIT)의 순수 PyTorch 모듈(`VariationalLSTM`, `grad_reverse`, `BRTreatmentOutcomeHead`, `TransformerMultiInputBlock` 등)을 그대로 가져오고 학습 루프·데이터·롤아웃은 새로 작성(Lightning·Hydra·MLflow 없음, venv 변경 없음). 편차 D1–D15: 일 단위 토큰, 집계값 일합, 37차원 결과(정규화 일 변화, 되먹임), 달력=이력 전용 공변량, 4클래스+양 처치, 7·30 토큰 지평, 논문 스텝 수 맞춤 epoch, val 사실 손실로 체크포인트 선택, CT EMA는 평가 전용, 크기(hidden/d_model 128 등), 그리드, 학습 데이터, 예측 계약, 유지 체크포인트의 스텝·α(D14), 인코더 예산 절반(D15).
- **일 단위 토큰.** 24행마다 한 토큰(관측: 06:00 고정; v2: 학습 시 위상 무작위, 평가 시 투입 행 위상). 투입은 정확히 한 토큰에 들어가며 그 토큰의 상태는 투입 전 상태.
- **사전 등록.** part a(정책·삼분위) → 처치 녹화, part b(실행·예산·그리드·선택·규칙) `956c8a05`(02:16:31) → 모든 테스트 평가는 그 커밋에서 1회(덮어쓰기 거부). part b 시점에 테스트 산출물 0개(코드 작업트리 제외).
- **절제 시드.** `wm3_ctrl_s1/s2`, `wm3_os_s1/s2`를 WM-3의 정확한 레시피(`wm3/finetune.py`, init `wm2_m1bact_v19_s1/s2`)로 학습·선택·평가.
- **위험도 재보정.** `wm3_full_s{0,1,2}`의 몸통을 고정하고 학습 가족 745개 투입 행에서 720스텝 무행동 롤아웃의 위험도로 Platt(a, b)를 최우도 적합("30일 내 분열"), head 가중치에 써 넣음(`wm4_haz_s*`). val Brier가 원래보다 낮을 때만 채택(세 시드 모두 채택).

### 기능 설명
연구 관점에서 확인된 것:
- 교란된 관측 분할이 실제로 교란되어 있다: 굶는 삼분위 투입의 86 %가 선물, 배부른 삼분위의 73 %가 부패; 테스트에서 naive가 zero보다 7일 +0.381 나쁘고, ipw가 그 초과 오차의 절반 이상을 걷어낸다(ipw − naive −0.209).
- 이 분야의 두 방법(CRN, CT)은 사전 등록된 예산에서 어느 분할에서도 "효과 없음"을 이기지 못한다. 관측 분할에서 M1b-obs는 행동을 무시하고(≈ zero), CRN·CT는 행동에 반응하되 교란 방향으로 과하게 반응한다. CRN-v2는 논문의 스텝 수를 모두 채웠는데도 그렇다.
- 짝 갈래 학습의 7일 개선은 세 시드에서도 짝 구성(짝 손실·짝 창·위험도 손실)의 몫이다(ctrl 대비 −0.075, os 대비 −0.068, WEAK). 30일에는 도움이 없다. 관측 테스트에서는 어떤 학습 모델도 "효과 없음"을 이기지 못한다.
- 위험도 head는 보정하면 기저율 수준까지 맞지만(Brier ≈ 기후값) 분열 시점·이동에 대한 정보는 거의 없다.

논문이 주장할 수 있는 것: (i) 사실 흐름으로 학습한 세계 모형은 개입을 무시하고, 개입 분할(v2)에서 짝 갈래는 첫 주 효과만 개선한다(관측 분할에서는 효과 없음을 못 이김); (ii) 관측 분할의 교란은 실재하고 측정 가능하다(7일에 ipw가 naive 초과 오차의 절반 이상 제거, 30일에는 아님); (iii) CRN·CT는 문서화된 편차와 고정 예산 아래에서 두 분할 모두 "효과 없음"을 이기지 못하고 교란 방향으로 과반응한다; (iv) 30일 부호 반전과 분열 시점은 아무 방법도 배우지 못한다; (v) 장기 개입 효과 벤치마크는 "효과 없음"을 반드시 보고해야 한다.
주장할 수 없는 것: CT의 domain confusion이 돕는지 여부(유지 체크포인트가 이른 시점이라 가중치가 최대치의 20 %, 논문 스텝의 4 %; CRN-v2는 전체 스텝·전체 역전 가중치로 학습됨); 반사실 데이터가 일반적으로 관측 데이터보다 낫다는 것(O3는 데이터 양·세계·초기화·손실이 함께 다름); 한 시뮬레이터·한 장면·단일 개입 밖으로의 일반화; 더 큰 예산에서의 결과.

### 변경된 파일 목록
(커밋별)
- **Part 0**: `7408fd8e` GOAL.md (unchanged); `9f1f1b90` DECISIONS.md, RUNLOG.md: start time, machines, input sha256 (recorder, WM-2/WM-3 checkpoints), seeds unused, d
- **Part 1**: `c4cef524` policy.py (decision rows, state, terciles, daily probabilities, draw), obs_split.py (verify, terciles, policy,; `7c58415d` run log (factual recording, terciles, policy); `dd7f51e8` fam-schedules writes its summary to <dir>/summary.json (it tried to write over the schedule directory); `2f2240c8` run log (observed episodes, family restart), L4.3; `446d11e6` post_families.sh (manifest, describe on train + val); `7b5a380c` the observational split: obs_manifest.json (1,200 episodes verified), DATASET_OBS.md (policy and confounding b
- **Part 2**: `6149d1ed` effects4.py (obs cases, floored scales, naive / ipw / match5 baselines, val selection, M1 on obs test), m1_obs; `19608337` CRN and the Causal Transformer at daily resolution (causal.py: train, grid, rollouts, prediction contract), da; `5cc0515d` decisions L1–L4; step counters in CRN / CT logs; cost4.sh; `9e287282` --role val dry runs of the obs / causal evaluation paths (val families only); `16d46953` the grid writes every member's records to OUT/train_log.jsonl (the queue's watchdog), grid_log.jsonl at the en; `ffe03bba` dryrun.sh (train / val only code-path checks before part b); `039e480c` CRN / CT evaluations time-gated (≤ every budget / 15 s), val_roll from 5 start tokens per episode; METHODS.md 
- **Part 3**: `bfa36208` METRICS.md part a: the observational policy, its frozen terciles (q_f1 8.0792, q_f2 12.1, q_w2 3.1119 from the; `956c8a05` METRICS.md part b: runs, budgets and grids, selection (models on val factual loss, baselines on val families),
- **Part 4**: `97c6391f` launch scripts: code.sh (wm4 worktrees under a lock), launch.sh, queue.sh (one training at a time, deadline-ca; `5542c5cf` run log (families, manifest, dry runs, part b, finals and queue launched); `1fd221d9` obs_base (baselines' val selection: zero at 7 d and 30 d; test cases and predictions); DATASET_OBS.md test eff; `9555ab71` runs: the six M1 references on the obs test, wm4_haz_s{0,1,2} (recalibration, test effects, split timing, test; `93c43e22` runs: wm3_ctrl_s1, wm3_ctrl_s2 (training records, test effects, split, factual), wm3_os_s1 training record; `e83c0c5d` run log (finals, ablation seeds trained, M1b-obs cost check, m1b_obs_s0 started); `540abb41` runs: wm3_os_s1, wm3_os_s2 (training records and finals), M1b-obs cost check; `6bdaa09b` finals_watch.sh (each run's finals after its training, in queue order) and the queue file as run (queue_run.tx; `6023a7f5` runs: m1b_obs_s0 (training record, obs test effects), CRN cost check; `f570f337` runs: crn_obs_s0 (grid of 4, chosen config 3, obs test effects), m1b_obs_s0 factual, CT cost check; `5e4bd64d` run log (m1b_obs_s0, CRN grid, CT cost), L4.4; `645b5cb6` runs: ct_obs_s0 (grid of 4, chosen config 2, obs test effects); `4a2e939d` run log (CT grid, review 1, m1b_obs_s1); `297e09ca` runs: m1b_obs_s1, crn_obs_s1, ct_obs_s1 (training records, obs test effects, factual)
- **Part 5**: `fa693771` judge4.py (O1–O4, B1–B2, A1, H1 from the committed evaluation files; tables); `a3896ade` runs: crn_v2_s{0,1}, ct_v2_s{0,1}; judge4 (judge.md, judge.json) on every committed evaluation
- **Part 6**: `e59227e4` METHODS.md cost checks and the grids; `33d97e1a` WM4.md (obs split, verdicts, Q1–Q4, v2 standing with post-hoc deaths breakdown, ablation, hazard head, caveats; `f2dc2005` run log times
- **Part 7**: `135f20a0` review round 1 (no BLOCKING; 2 MAJOR, 4 MINOR, 7 NOTE): METHODS D14 (steps and alpha at the kept checkpoints),; `c569ba32` steps run vs kept (review round 1, M1); `2539e744` review round 1 time; `96efb337` gate.sh (DGX half of the gate, wm4 paths); `c3b6025e` review round 2 (no BLOCKING; 2 MAJOR, 5 MINOR, NOTEs) fixed: Q2 per-cell claim removed (post hoc reading), per
- **Part 7(이 보고)**: `REPORT.md`와 RUNLOG 마지막 행을 담은 문서 전용 커밋(해시는 최종 메시지에 기재).

### 측정 결과
**판정표 (사전 등록 규칙; CI = 세계 단위 짝 bootstrap 95 %, B = 2,000)**

| rule | model (seeds) | baseline | ratio | Δ = model − baseline [95 % CI] | verdict |
|---|---|---|---|---|---|
| O1 obs 7 d: `crn_obs` | 0.497 (0.464, 0.530) | `zero` 0.218 | 2.282 | +0.279 [+0.245, +0.316] | **NONE** |
| O1 obs 7 d: `ct_obs` | 0.321 (0.296, 0.346) | `zero` 0.218 | 1.474 | +0.103 [+0.079, +0.128] | **NONE** |
| O1 obs 7 d: `m1b_obs` | 0.214 (0.213, 0.215) | `zero` 0.218 | 0.983 | -0.004 [-0.008, +0.001] | **TIE** |
| O2 obs 30 d: `crn_obs` | 0.863 (0.686, 1.041) | `zero` 0.168 | 5.131 | +0.695 [+0.617, +0.789] | **NONE** |
| O2 obs 30 d: `ct_obs` | 0.373 (0.368, 0.378) | `zero` 0.168 | 2.217 | +0.205 [+0.186, +0.225] | **NONE** |
| O2 obs 30 d: `m1b_obs` | 0.175 (0.175, 0.175) | `zero` 0.168 | 1.039 | +0.007 [+0.003, +0.011] | **NONE** |
| O3 obs 7d: `wm3_full` vs `crn_obs` | 0.229 (0.210, 0.244, 0.232) | `crn_obs (seeds 0, 1)` 0.497 | 0.460 | -0.268 [-0.304, -0.236] | **KEEP** |
| O3 obs 7d: `wm3_full` vs `ct_obs` | 0.229 (0.210, 0.244, 0.232) | `ct_obs (seeds 0, 1)` 0.321 | 0.712 | -0.092 [-0.111, -0.074] | **KEEP** |
| O3 obs 7d: `wm3_full` vs `m1b_obs` | 0.229 (0.210, 0.244, 0.232) | `m1b_obs (seeds 0, 1)` 0.214 | 1.069 | +0.015 [+0.001, +0.029] | **NONE** |
| O3 obs 30d: `wm3_full` vs `crn_obs` | 0.175 (0.179, 0.171, 0.175) | `crn_obs (seeds 0, 1)` 0.863 | 0.203 | -0.688 [-0.781, -0.610] | **KEEP** |
| O3 obs 30d: `wm3_full` vs `ct_obs` | 0.175 (0.179, 0.171, 0.175) | `ct_obs (seeds 0, 1)` 0.373 | 0.469 | -0.198 [-0.218, -0.179] | **KEEP** |
| O3 obs 30d: `wm3_full` vs `m1b_obs` | 0.175 (0.179, 0.171, 0.175) | `m1b_obs (seeds 0, 1)` 0.175 | 1.001 | +0.000 [-0.004, +0.004] | **TIE** |
| B1 v2 7 d: `crn_v2` | 1.474 (1.357, 1.590) | `an5_arm` 0.485 | 3.038 | +0.989 [+0.917, +1.062] | **NONE** |
| B1 v2 7 d: `crn_v2` | 1.474 (1.357, 1.590) | `wm3_full (seeds 0, 1, 2)` 0.496 | 2.973 | +0.978 [+0.901, +1.061] | **NONE** |
| B1 v2 7 d: `ct_v2` | 1.007 (0.860, 1.154) | `an5_arm` 0.485 | 2.077 | +0.522 [+0.486, +0.560] | **NONE** |
| B1 v2 7 d: `ct_v2` | 1.007 (0.860, 1.154) | `wm3_full (seeds 0, 1, 2)` 0.496 | 2.032 | +0.512 [+0.457, +0.556] | **NONE** |
| B2 v2 30 d: `crn_v2` | 1.547 (1.530, 1.564) | `zero` 0.487 | 3.177 | +1.060 [+0.943, +1.200] | **NONE** |
| B2 v2 30 d: `ct_v2` | 1.374 (1.446, 1.303) | `zero` 0.487 | 2.822 | +0.887 [+0.840, +0.934] | **NONE** |
| A1 v2 7d: `wm3_full` vs `ctrl` | 0.496 (0.532, 0.472, 0.484) | `wm3_ctrl (seeds 0, 1, 2)` 0.571 | 0.868 | -0.075 [-0.100, -0.047] | **WEAK** |
| A1 v2 30d: `wm3_full` vs `ctrl` | 0.511 (0.517, 0.504, 0.512) | `wm3_ctrl (seeds 0, 1, 2)` 0.501 | 1.019 | +0.010 [+0.006, +0.013] | **NONE** |
| A1 v2 7d: `wm3_full` vs `os` | 0.496 (0.532, 0.472, 0.484) | `wm3_os (seeds 0, 1, 2)` 0.564 | 0.879 | -0.068 [-0.091, -0.046] | **WEAK** |
| A1 v2 30d: `wm3_full` vs `os` | 0.511 (0.517, 0.504, 0.512) | `wm3_os (seeds 0, 1, 2)` 0.509 | 1.004 | +0.002 [-0.005, +0.008] | **TIE** |
| H1 timing (days) | 35.99 (36.58, 36.10, 35.27) | `ridge_day` 29.21 | 1.232 | +6.77 [+2.52, +10.92] | **NONE** |
| H1 shift (days) | 10.81 (11.14, 10.85, 10.43) | `an5_arm` 11.41 | 0.947 | -0.60 [-2.41, +1.34] | **TIE** |

O = 관측 테스트(80 세계 × 3 갈래 = 240 사례, 12셀, 하한 척도, 기준선 = val 가족에서 고른 `zero`); B·A = v2 테스트(WM-2의 604 사례·척도); H1 = WM-3의 V3·V4 과제, 재보정 head. 요약: O1 NONE·NONE·TIE, O2 NONE × 3, O3 KEEP × 4 (CRN·CT 대비)·NONE/TIE (M1b-obs 대비), O4 보고, B1 NONE × 4, B2 NONE × 2, A1 WEAK·WEAK (7일)·NONE·TIE (30일), H1 시점 NONE·이동 TIE.

**관측 분할의 교란 (삼분위별 갈래 비율, 투입 행의 f 기준)**

| f 삼분위 | 투입 수 | `gf240` | `sp240` | `rw120` |
|---|---|---|---|---|
| f < 8.08 (굶음) | 148 | 85.8 % | 3.4 % | 10.8 % |
| 8.08 ≤ f < 12.1 | 109 | 38.5 % | 30.3 % | 31.2 % |
| f ≥ 12.1 (배부름) | 202 | 6.9 % | 73.3 % | 19.8 % |

| 역할 | 세계 | 무투입 | 투입 gift / spoil / rot |
|---|---|---|---|
| all | 560 | 101 (18.0 %) | 183 / 186 / 90 |
| train | 400 | 80 (20.0 %) | 129 / 132 / 59 |
| val | 80 | 4 (5.0 %) | 30 / 26 / 20 |
| test | 80 | 17 (21.2 %) | 24 / 28 / 11 |

IPW 유효 표본(학습): 처치 129 / 132 / 59 → ESS 44.3 / 54.9 / 46.7 (최대 가중치 500); 대조 8,441(7일) / 5,018(30일).

## O4 — confounding visible (obs test, 12-cell averages; Δ with world-bootstrap 95 % CI)

| lag | zero | naive | ipw | match5 | naive − zero | ipw − zero | ipw − naive |
|---|---|---|---|---|---|---|---|
| 7d | 0.218 | 0.599 | 0.391 | 0.945 | +0.381 [+0.336, +0.423] | +0.173 [+0.146, +0.200] | -0.209 [-0.231, -0.182] |
| 30d | 0.168 | 0.323 | 0.382 | 0.968 | +0.155 [+0.148, +0.162] | +0.214 [+0.202, +0.225] | +0.059 [+0.052, +0.066] |

Per arm (4-cell averages):

| lag | arm | zero | naive | ipw | match5 |
|---|---|---|---|---|---|
| 7d | gf240 | 0.198 | 0.545 | 0.446 | 0.993 |
| 7d | sp240 | 0.376 | 0.759 | 0.458 | 0.978 |
| 7d | rw120 | 0.080 | 0.493 | 0.268 | 0.864 |
| 30d | gf240 | 0.158 | 0.265 | 0.395 | 0.877 |
| 30d | sp240 | 0.259 | 0.539 | 0.485 | 1.073 |
| 30d | rw120 | 0.088 | 0.167 | 0.267 | 0.956 |

Pooled estimates on the observed train worlds (food store; `naive` and `ipw` are constants per arm and lag):

| arm | lag | naive | ipw | treated | ESS treated | controls | ESS controls | max weight treated / controls |
|---|---|---|---|---|---|---|---|---|
| gf240 | 7d | 139.2 | 10.5 | 129 | 44.3 | 8441 | 8419.3 | 500.0 / 1.37 |
| gf240 | 30d | 8.5 | -33.7 | 129 | 44.3 | 5018 | 4673.1 | 500.0 / 3.21 |
| sp240 | 7d | -232.2 | -151.8 | 132 | 54.9 | 8441 | 8419.3 | 500.0 / 1.37 |
| sp240 | 30d | -39.2 | -8.6 | 132 | 54.9 | 5018 | 4673.1 | 500.0 / 3.21 |
| rw120 | 7d | -71.3 | 21.7 | 59 | 46.7 | 8441 | 8419.3 | 250.0 / 1.37 |
| rw120 | 30d | -16.8 | -16.9 | 59 | 46.7 | 5018 | 4673.1 | 250.0 / 3.21 |

**관측 테스트, 갈래·결과·지평별 척도화 효과 MAE (시드 평균; 괄호 = 평균 예측, 참값 평균은 둘째 열)**

| 갈래 | 결과 | 지평 | 참값 평균 | zero | naive | ipw | match5 | M1b-obs | CRN-obs | CT-obs | WM-2 M1b-act | wm3_full |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| gf240 | food_stock | 7d | 18.64 | 0.463 (0.0) | 1.664 (139.2) | 0.451 (10.5) | 1.630 (111.9) | 0.490 (-6.2) | 1.037 (86.9) | 0.615 (44.1) | 0.456 (2.6) | 0.580 (25.3) |
| gf240 | food_stock | 30d | -16.93 | 0.245 (0.0) | 0.277 (8.5) | 0.379 (-33.7) | 0.744 (7.4) | 0.246 (-0.6) | 0.665 (2.2) | 0.311 (0.9) | 0.247 (0.9) | 0.254 (3.6) |
| gf240 | alive | 7d | -0.84 | 0.207 | 0.227 | 0.333 | 0.779 | 0.207 | 0.299 | 0.278 | 0.208 | 0.207 |
| gf240 | alive | 30d | -2.19 | 0.195 | 0.310 | 0.324 | 0.599 | 0.195 | 0.321 | 0.246 | 0.197 | 0.196 |
| gf240 | deaths | 7d | 0.00 | 0.000 | 0.120 | 0.754 | 1.175 | 0.012 | 0.423 | 0.372 | 0.000 | 0.000 |
| gf240 | deaths | 30d | 0.03 | 0.016 | 0.208 | 0.565 | 1.546 | 0.040 | 1.828 | 0.520 | 0.035 | 0.016 |
| gf240 | settlements_alive | 7d | -0.04 | 0.121 | 0.169 | 0.245 | 0.389 | 0.124 | 0.275 | 0.266 | 0.124 | 0.123 |
| gf240 | settlements_alive | 30d | -0.06 | 0.177 | 0.263 | 0.310 | 0.618 | 0.179 | 0.432 | 0.348 | 0.180 | 0.180 |
| sp240 | food_stock | 7d | -81.42 | 1.086 (0.0) | 1.950 (-232.2) | 1.020 (-151.8) | 0.950 (-124.1) | 0.957 (-12.3) | 0.897 (-132.0) | 0.673 (-102.6) | 0.993 (-8.7) | 0.754 (-65.8) |
| sp240 | food_stock | 30d | 32.39 | 0.341 (0.0) | 0.549 (-39.2) | 0.360 (-8.6) | 0.782 (-1.5) | 0.340 (-1.6) | 0.878 (-27.2) | 0.468 (-22.5) | 0.341 (-0.0) | 0.377 (-9.5) |
| sp240 | alive | 7d | 1.31 | 0.256 | 0.453 | 0.258 | 0.645 | 0.256 | 0.362 | 0.281 | 0.257 | 0.256 |
| sp240 | alive | 30d | 4.36 | 0.339 | 0.343 | 0.342 | 0.829 | 0.339 | 0.567 | 0.411 | 0.340 | 0.342 |
| sp240 | deaths | 7d | 0.00 | 0.000 | 0.419 | 0.301 | 1.914 | 0.012 | 0.633 | 0.296 | 0.000 | 0.004 |
| sp240 | deaths | 30d | 0.07 | 0.048 | 0.936 | 0.915 | 1.857 | 0.072 | 1.928 | 0.688 | 0.066 | 0.050 |
| sp240 | settlements_alive | 7d | 0.07 | 0.162 | 0.215 | 0.252 | 0.405 | 0.163 | 0.428 | 0.265 | 0.165 | 0.164 |
| sp240 | settlements_alive | 30d | 0.14 | 0.308 | 0.327 | 0.323 | 0.823 | 0.309 | 0.678 | 0.449 | 0.312 | 0.311 |
| rw120 | food_stock | 7d | -4.09 | 0.170 (0.0) | 0.923 (-71.3) | 0.370 (21.7) | 1.134 (18.4) | 0.185 (-8.2) | 0.616 (1.8) | 0.339 (-19.2) | 0.238 (-11.1) | 0.504 (-35.1) |
| rw120 | food_stock | 30d | 2.52 | 0.144 (0.0) | 0.219 (-16.8) | 0.219 (-16.9) | 0.792 (14.4) | 0.146 (-1.1) | 0.895 (-20.1) | 0.216 (-7.5) | 0.148 (-1.2) | 0.159 (-4.9) |
| rw120 | alive | 7d | -0.06 | 0.034 | 0.284 | 0.174 | 0.674 | 0.034 | 0.265 | 0.101 | 0.035 | 0.035 |
| rw120 | alive | 30d | 0.80 | 0.071 | 0.215 | 0.351 | 0.672 | 0.071 | 0.363 | 0.126 | 0.073 | 0.073 |
| rw120 | deaths | 7d | 0.20 | 0.115 | 0.538 | 0.304 | 1.260 | 0.126 | 0.487 | 0.293 | 0.115 | 0.115 |
| rw120 | deaths | 30d | 0.23 | 0.100 | 0.131 | 0.274 | 1.698 | 0.122 | 1.328 | 0.547 | 0.113 | 0.100 |
| rw120 | settlements_alive | 7d | 0.00 | 0.000 | 0.227 | 0.225 | 0.389 | 0.004 | 0.243 | 0.074 | 0.004 | 0.004 |
| rw120 | settlements_alive | 30d | 0.01 | 0.035 | 0.103 | 0.226 | 0.660 | 0.038 | 0.478 | 0.146 | 0.041 | 0.041 |

**v2 테스트, 갈래·결과·지평별 척도화 효과 MAE (시드 평균)**

| 갈래 | 결과 | 지평 | zero | an5_arm | arm_mean | WM-2 M1b-act | wm3_full | wm3_ctrl | wm3_os | CRN-v2 | CT-v2 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| gf240E | food_stock | 7d | 1.560 | 0.636 | 0.746 | 1.428 | 0.956 | 1.498 | 1.311 | 1.172 | 1.072 |
| gf240E | food_stock | 30d | 0.424 | 0.542 | 0.588 | 0.430 | 0.464 | 0.429 | 0.435 | 0.603 | 0.436 |
| gf240E | alive | 7d | 0.072 | 0.101 | 0.129 | 0.072 | 0.072 | 0.072 | 0.072 | 3.220 | 1.189 |
| gf240E | alive | 30d | 0.322 | 0.465 | 0.566 | 0.324 | 0.328 | 0.322 | 0.327 | 0.369 | 0.343 |
| gf240E | deaths | 7d | 0.089 | 0.124 | 0.159 | 0.384 | 0.473 | 0.325 | 0.561 | 4.912 | 4.688 |
| gf240E | deaths | 30d | 0.424 | 0.517 | 0.431 | 0.489 | 0.514 | 0.489 | 0.553 | 8.791 | 9.195 |
| gf240E | settlements_alive | 7d | 0.000 | 0.000 | 0.000 | 0.005 | 0.003 | 0.002 | 0.003 | 0.127 | 0.094 |
| gf240E | settlements_alive | 30d | 0.343 | 0.490 | 0.585 | 0.352 | 0.350 | 0.345 | 0.348 | 0.626 | 0.583 |
| sp240L | food_stock | 7d | 1.265 | 0.780 | 0.750 | 1.172 | 0.792 | 1.123 | 1.022 | 1.292 | 0.799 |
| sp240L | food_stock | 30d | 0.857 | 0.833 | 0.916 | 0.858 | 0.925 | 0.858 | 0.875 | 1.205 | 0.872 |
| sp240L | alive | 7d | 0.843 | 0.884 | 0.936 | 0.843 | 0.843 | 0.843 | 0.843 | 0.866 | 0.867 |
| sp240L | alive | 30d | 0.695 | 0.780 | 0.892 | 0.697 | 0.697 | 0.696 | 0.697 | 0.733 | 0.704 |
| sp240L | deaths | 7d | 0.000 | 0.000 | 0.000 | 0.020 | 0.040 | 0.027 | 0.066 | 0.402 | 0.285 |
| sp240L | deaths | 30d | 0.441 | 0.539 | 0.556 | 0.472 | 0.457 | 0.478 | 0.466 | 1.901 | 1.473 |
| sp240L | settlements_alive | 7d | 0.715 | 0.830 | 0.942 | 0.717 | 0.716 | 0.716 | 0.716 | 0.882 | 0.823 |
| sp240L | settlements_alive | 30d | 0.568 | 0.791 | 0.871 | 0.571 | 0.570 | 0.570 | 0.570 | 0.914 | 0.660 |
| rw120E | food_stock | 7d | 1.478 | 0.710 | 0.716 | 1.118 | 0.719 | 0.903 | 0.818 | 2.902 | 0.829 |
| rw120E | food_stock | 30d | 0.410 | 0.515 | 0.477 | 0.427 | 0.435 | 0.427 | 0.424 | 0.599 | 0.423 |
| rw120E | alive | 7d | 0.530 | 0.799 | 0.759 | 0.530 | 0.530 | 0.530 | 0.530 | 0.803 | 0.587 |
| rw120E | alive | 30d | 0.243 | 0.422 | 0.415 | 0.249 | 0.255 | 0.257 | 0.252 | 0.348 | 0.320 |
| rw120E | deaths | 7d | 0.793 | 0.955 | 0.824 | 0.800 | 0.800 | 0.804 | 0.805 | 1.005 | 0.809 |
| rw120E | deaths | 30d | 0.870 | 0.801 | 0.708 | 0.877 | 0.876 | 0.878 | 0.879 | 1.727 | 1.057 |
| rw120E | settlements_alive | 7d | 0.000 | 0.000 | 0.000 | 0.012 | 0.005 | 0.011 | 0.015 | 0.100 | 0.048 |
| rw120E | settlements_alive | 30d | 0.247 | 0.439 | 0.420 | 0.267 | 0.260 | 0.265 | 0.277 | 0.749 | 0.425 |

**반사실 학습 데이터의 가치 (관측 테스트 12셀 평균)**: `wm3_full`(v2 짝 갈래) 7일 0.229 (0.210 / 0.244 / 0.232), 30일 0.175; WM-2 M1b-act(v2, 짝 없음) 0.216 / 0.219 / 0.213, 0.183 / 0.169 / 0.172; M1b-obs(관측) 0.214, 0.175; `zero` 0.218, 0.168. 사후: WM-2 참조 vs `zero` 7일 TIE·30일 NONE, `wm3_full` vs `zero` 7일 TIE [−0.004, +0.026]·30일 NONE.

**v2에서 CRN·CT의 오차 분해 (사후, DECISIONS L5.1)**: 사망 3셀 평균 7일 CRN 2.75 / 1.46, CT 1.41 / 2.45 (다른 방법 0.29–0.45), 30일 ≈ 3.6–4.2 (0.58–0.62); 나머지 9셀 7일 CRN 0.892 / 1.633, CT 0.678 / 0.724 (`an5_arm` 0.527, `wm3_full` 0.488–0.562), 30일 0.644 / 0.721, 0.530 / 0.529 (`zero` 0.457). 사실 30토큰 롤아웃 손실(테스트): CT-v2 0.233 / 0.240, CRN-v2 0.298 / 0.278, CT-obs 0.260 / 0.268, CRN-obs 0.368 / 0.368.

**위험도 Brier·보정 전후**
| seed | a | b | kept | train Brier before → after (clim.) | val Brier orig → recal (clim.) | val mean pred orig → recal (obs) | test Brier orig → recal (clim.) | test mean pred orig → recal (obs) |
|---|---|---|---|---|---|---|---|---|
| 0 | 0.927 | -2.892 | True | 0.670 → 0.215 (0.216) | 0.576 → 0.247 (0.242) | 0.990 → 0.317 (0.412) | 0.647 → 0.221 (0.224) | 0.989 → 0.306 (0.338) |
| 1 | 1.504 | 1.243 | True | 0.510 → 0.212 (0.216) | 0.444 → 0.242 (0.242) | 0.866 → 0.323 (0.412) | 0.487 → 0.218 (0.224) | 0.856 → 0.305 (0.338) |
| 2 | 1.505 | 1.636 | True | 0.408 → 0.212 (0.216) | 0.357 → 0.237 (0.242) | 0.763 → 0.319 (0.412) | 0.389 → 0.220 (0.224) | 0.750 → 0.295 (0.338) |
- s0 test_original: bin 0 pred 0.982 obs 0.200 (n 30); bin 1 pred 0.987 obs 0.267 (n 30); bin 2 pred 0.989 obs 0.400 (n 30); bin 3 pred 0.991 obs 0.300 (n 30); bin 4 pred 0.994 obs 0.516 (n 31)
- s0 test_recal: bin 0 pred 0.280 obs 0.200 (n 30); bin 1 pred 0.295 obs 0.267 (n 30); bin 2 pred 0.304 obs 0.400 (n 30); bin 3 pred 0.315 obs 0.300 (n 30); bin 4 pred 0.337 obs 0.516 (n 31)
- s1 test_original: bin 0 pred 0.832 obs 0.067 (n 30); bin 1 pred 0.846 obs 0.267 (n 30); bin 2 pred 0.855 obs 0.367 (n 30); bin 3 pred 0.865 obs 0.500 (n 30); bin 4 pred 0.881 obs 0.484 (n 31)
- s1 test_recal: bin 0 pred 0.274 obs 0.067 (n 30); bin 1 pred 0.290 obs 0.267 (n 30); bin 2 pred 0.302 obs 0.400 (n 30); bin 3 pred 0.315 obs 0.433 (n 30); bin 4 pred 0.342 obs 0.516 (n 31)
- s2 test_original: bin 0 pred 0.715 obs 0.133 (n 30); bin 1 pred 0.739 obs 0.267 (n 30); bin 2 pred 0.750 obs 0.267 (n 30); bin 3 pred 0.761 obs 0.500 (n 30); bin 4 pred 0.784 obs 0.516 (n 31)
- s2 test_recal: bin 0 pred 0.247 obs 0.133 (n 30); bin 1 pred 0.275 obs 0.233 (n 30); bin 2 pred 0.290 obs 0.333 (n 30); bin 3 pred 0.306 obs 0.433 (n 30); bin 4 pred 0.354 obs 0.548 (n 31)
- H1 split-shift right-direction share: `wm4_haz_s0` sp240L 0.99, gf240E 0.04; `wm4_haz_s1` sp240L 1.00, gf240E 0.04; `wm4_haz_s2` sp240L 1.00, gf240E 0.67

**실행별 종료 이유·최적 스텝·시간**
| run | stop | best | seconds | val | detail |
|---|---|---|---|---|---|
| `_cost_crn` | time | epoch 30 (decoder) | 301 | 0.5413 | {"decoder": {"best_epoch": 30, "best_val_roll": 0.5412658584643786, "epochs": 1538, "epochs_run": 127, "stop": "time"}, "encoder": {"best_epoch": 361, "best_val_one_step": 0.23668532073497772, "epochs": 2617, "epochs_run": 362, "stop": "time"}, "val_roll": 0.5412658584643786} |
| `_cost_ct` | time | epoch 118 | 308 | 1.3731 | {"best_epoch": 118, "epochs": 3950, "epochs_run": 119, "stop": "time", "val_roll": 1.3731159830098172} |
| `_cost_m1b` | max_steps | 100 | 669 | 0.1143 | selected None, not converged None |
| `crn_obs_s0` | epochs | epoch 240 (decoder) | 1180 | 0.4132 | {"decoder": {"best_epoch": 240, "best_val_roll": 0.413165744378812, "epochs": 1538, "epochs_run": 1538, "stop": "epochs"}, "encoder": {"best_epoch": 312, "best_val_one_step": 0.23566985130310059, "epochs": 2617, "epochs_run": 2358, "stop": "time"}, "val_roll": 0.413165744378812} |
| `crn_obs_s1` | time | epoch 1176 (decoder) | 1200 | 0.3890 | {"decoder": {"best_epoch": 1176, "best_val_roll": 0.38904215594761465, "epochs": 1538, "epochs_run": 1177, "stop": "time"}, "encoder": {"best_epoch": 260, "best_val_one_step": 0.24065594375133514, "epochs": 2617, "epochs_run": 1455, "stop": "time"}, "val_roll": 0.38904215594761465} |
| `crn_v2_s0` | epochs | epoch 99 (decoder) | 2350 | 1.5609 | {"decoder": {"best_epoch": 99, "best_val_roll": 1.560896605085102, "epochs": 163, "epochs_run": 163, "stop": "epochs"}, "encoder": {"best_epoch": 122, "best_val_one_step": 0.5569235682487488, "epochs": 267, "epochs_run": 267, "stop": "epochs"}, "val_roll": 1.560896605085102} |
| `crn_v2_s1` | epochs | epoch 147 (decoder) | 1517 | 1.5416 | {"decoder": {"best_epoch": 147, "best_val_roll": 1.5416371238962518, "epochs": 163, "epochs_run": 163, "stop": "epochs"}, "encoder": {"best_epoch": 266, "best_val_one_step": 0.5617067217826843, "epochs": 267, "epochs_run": 267, "stop": "epochs"}, "val_roll": 1.5416371238962518} |
| `ct_obs_s0` | time | epoch 158 | 1206 | 0.2727 | {"best_epoch": 158, "epochs": 3950, "epochs_run": 479, "stop": "time", "val_roll": 0.2727081026964337} |
| `ct_obs_s1` | time | epoch 158 | 1206 | 0.2772 | {"best_epoch": 158, "epochs": 3950, "epochs_run": 426, "stop": "time", "val_roll": 0.2771939637151982} |
| `ct_v2_s0` | time | epoch 16 | 2419 | 1.4841 | {"best_epoch": 16, "epochs": 409, "epochs_run": 92, "stop": "time", "val_roll": 1.4841158158720795} |
| `ct_v2_s1` | time | epoch 16 | 2416 | 1.4905 | {"best_epoch": 16, "epochs": 409, "epochs_run": 95, "stop": "time", "val_roll": 1.490460387844446} |
| `m1b_obs_s0` | patience | 4650 | 9449 | 0.0571 | selected None, not converged None |
| `m1b_obs_s1` | max_steps | 7800 | 11261 | 0.0573 | selected None, not converged None |
| `wm3_ctrl_s1` | patience | 1000 | 4339 | 0.5456 | selected step 1000, not converged False |
| `wm3_ctrl_s2` | patience | 1500 | 4930 | 0.5337 | selected step 1500, not converged False |
| `wm3_os_s1` | patience | 0 | 5058 | 0.5489 | selected init, not converged False |
| `wm3_os_s2` | patience | 2000 | 7759 | 0.5146 | selected step 2000, not converged False |

CRN·CT 그리드(`METHODS.md`): CRN val_roll 0.592 / 0.425 / 0.523 / **0.413**(구성 3), CT 1.181 / 0.310 / **0.273**(구성 2) / 0.285. 비용 점검: M1b-obs 100스텝 14 s(k ≤ 12), 4,775 MiB; CRN 인코더 ≈ 0.069 s/스텝, 디코더 ≈ 0.029, 477 MiB; CT ≈ 0.83 s/스텝, 14,343 MiB. M1b-obs 사실 T2(관측 테스트, 원값, 매시): 24 h 0.113 / 0.112, 168 h 0.173 / 0.169.

### 환경
- Mac: Darwin 27.0.0 arm64(T6041), 편집·커밋·푸시·작은 단위 테스트만(녹화·학습·평가·cargo 없음). Python 3.14, torch 2.11.
- DGX Spark: Linux 6.14.0-1015-nvidia aarch64, NVIDIA GB10(통합 메모리 119 GiB), 드라이버 580.126.09, CUDA 13.0, `nproc` 20; venv `/home/hyunlord/work/wm1-venv`(Python 3.12.3, torch 2.14.1+cu130, numpy 2.5.2; 변경 없음). 다른 사용자 부하 평균 1–56.
- 녹화기 `_build/src_wm2` `worldsim_wm` `52b36541…`(WM-2, 재빌드 없음), `WORLDSIM_GIT_SHA=f5373ab2…`. 데이터 `/home/hyunlord/work/worldsim-wm-data/obs/`, 실행 `/home/hyunlord/work/wm1-runs/wm4/<run>/`, 코드 작업트리 `wm4/_code/<sha8>`.
- GPU 재현성은 측정하지 않음.

### 독립 리뷰
- **1차** (2026-10-17 11:28–11:44, HEAD `e59227e4`, 판정은 잠정): BLOCKING 없음, MAJOR 2(CT 유지 체크포인트의 α·스텝이 문서와 다름; O3가 반사실 데이터의 가치를 분리하지 못함), MINOR 4(드라이런의 val 가족 읽기 공개, 기준선 분열 이동의 대조 오염 주의, 척도 문구, CRN 인코더 예산 절반), NOTE 7. 손 계산: 정책 재생 560 세계·결정 행 14,698개 불일치 0, 삼분위 재유도 일치, 사전 등록 순서, 접두 일치(sha256), 누설 없음, naive·ipw 손 계산 = `obs_base.json`, vendored 파일 = 원저자 `c49a24fa`(MIT). 응답 끝부분은 API 오류로 잘림(받은 항목 기록). 수정 `135f20a0`, `c569ba32`, `2539e744`(문서만).
- **2차** (17:38–17:46, HEAD `96efb337`, 모든 실행 완료): BLOCKING 없음; `judge4.py` 재실행이 judge.md·judge.json을 정확히 재현; 사전 등록 순서(part b 02:16:31, 첫 테스트 산출물 02:17:22, 모든 평가 파일의 `code_commit` = `956c8a05`, 평가 43개 각 1회·모두 exit 0); 허용 경로 OK. MAJOR 2(Q2의 '짝 학습이 부패 손실 크기를 가르친다' 주장이 셀 오차와 모순 — 삭제·사후 표시로 대체; CRN '덜 학습' 단서가 CRN에는 틀리고 CT에는 과장 — 방법별로 고침, D14 보완), MINOR 5(v2 격차 중 사망 셀 비중 문구, 논문 주장의 지평·분할 한정, A1 문구(짝+위험도 손실), judge 실행표의 None, 마지막 드라이런이 part b보다 20초 늦게 끝남), NOTE 5. 손 계산: M1b-obs O1 0.2140 vs 0.2178 [−0.0078, +0.0010] TIE, A1 7일 0.4957 vs 0.5712·0.5637. 수정 `c3b6025e`. 3차 없음(GOAL 0.3.3).

### 자율 결정 목록
`DECISIONS.md` L0.1–L6.1. 주요 항목: L0.2 40시간 마감 2026-10-18 15:13:31; L0.3 WM-3 체크포인트 sha256 최초 기록; L1.1 가족은 세계별 녹화기 호출; L1.2 매니페스트 경로·분할 이름; L1.3 추가 점검(가족 `none_r0`=사실, 관측=가족 구성원); L2.1 대조 단위는 결정 행만(양성 조건; 이 데이터에선 영향 없음); L2.2 기준선의 분열 이동(보고만, 주의점); L2.3 논문 스텝 수 맞춤 epoch; L2.4 그리드 선택 구성이 곧 `_obs_s0`; L2.5 드라이런이 val 가족을 읽음(선택에는 미사용, 공개); L4.1 단일 큐; L4.2 재보정 적합을 첫 학습 옆에서 실행; L4.3 GB10 메모리 압박에서 CUDA 컨텍스트 실패(환경); L4.4 CT의 예산은 사전 등록대로 유지(비용 점검 후 변경 안 함); L5.1 사후 분석 표시; L6.1 WM3.md 안내 줄.

### 실행 기록
(KST, 맥 시계; 일시정지 없음 — `pmset` 로그에 실행 중 수면 기록 없음; 전체 `RUNLOG.md`)
- 시작 2026-10-16 23:13:31. Part 0 23:13–23:15 (`7408fd8e`, `9f1f1b90`).
- Part 4 (9.3·9.4) 23:16:15 절제 큐 시작; 23:28–23:32 재보정 적합(세 시드 채택).
- Part 1 23:20:02–01:56: 사실 560 (23:20–23:57), part a 커밋 23:57:56, 정책 23:58:20, 관측 1,019 (23:58–01:06), 가족 160 (01:09–01:50; 첫 시도 요약 쓰기 버그 수정 `dd7f51e8`), 매니페스트·DATASET_OBS `7b5a380c` 01:52.
- Part 2 23:19–02:16: 코드·테스트(23:19–00:15), 드라이런(01:56–02:17), 평가 시간 제한(`039e480c`).
- Part 3 part a 23:57:56 (`bfa36208`), part b 02:16:31 (`956c8a05`).
- Part 4 학습: 절제 4개 23:16–05:27 (ctrl_s1 4,339 s, ctrl_s2 4,930 s, os_s1 5,058 s, os_s2 7,759 s), M1b-obs s0 05:39–08:17, CRN 그리드 08:23–09:43, CT 그리드 09:49–11:10, M1b-obs s1 11:10–14:18, CRN·CT obs s1 14:18–15:00, v2 4개 15:00–17:28; 큐 종료 17:30:25 (≈ 18 h GPU, 탈락 실행 없음). 평가는 02:16:50부터 각 학습 직후 사전 등록 커밋에서(마지막 17:29:16).
- Part 5 17:30–17:31 (judge4), Part 6 08:30–17:32 (WM4.md, `33d97e1a`).
- Part 7 리뷰 1차 11:28–11:44, 리뷰 2차 17:38–17:46, 수정 `c3b6025e` 17:50, 게이트 17:51:50–17:54:15, fast-forward 17:55, PR 댓글 17:56, 보고서 17:58.
- 환경 사건: 00:36 메모리 압박으로 CUDA 컨텍스트 생성 실패(속도 시험 1회 생략, L4.3); 다른 사용자 부하 평균 1–56.

### 확인된 제한사항
- CT의 유지 체크포인트는 이른 시점: 477–493스텝(논문 11,850의 4 %), domain-confusion 가중치는 최대치의 20 % — 그 뒤 val 사실 손실이 올라갔기 때문(CT-obs 0.273 → 0.341, CT-v2 1.484 → 2.00)이며, α 증가 일정이 명목 epoch에 묶여 있음. CRN-v2는 논문 스텝 수 전부(인코더 15,753·디코더 54,116)를 역전 가중치 0.98–1.0으로 학습; CRN-obs의 유지 인코더는 이른 시점(가중치 0.46–0.53). 원저자 어텐션 구현의 느림(스텝당 ≈ 0.83초)이 CT의 실행 스텝을 제한.
- CRN·CT는 개수 반올림이 없어(D13) 두 롤아웃의 사망 일일값 차이가 v2의 아주 작은 사망 척도에서 크게 벌점을 받음(사후: 사망 3셀 평균 1.4–4.2 vs 다른 방법 0.3–0.6). 사망 셀을 빼도 순위는 같음.
- 일 단위(CRN·CT) 대 시간 단위(M1) 입력: 사실 손실을 직접 비교할 수 없어 O3에서 obs 모델 간 선택은 하지 않음.
- 관측 분할이 작음(학습 세계 400, 투입 320, 처치 IPW 유효 표본 44–55).
- O3는 반사실 데이터의 가치를 분리하지 못함(데이터 양·세계·초기화·짝 손실이 함께 다름).
- 리뷰 1차 응답은 API 오류로 끝부분이 잘려 받은 항목만 기록.
- `wm3_os_s1`은 init(=WM-2 s1) 그대로 선택됨; A1의 `os` 평균에 포함.
- val 세계의 무투입이 기대보다 적음(4 vs 13.4 ± 3.3; 추첨 재현 확인, 우연).
- GPU 재현성 미측정.

### 게이트 결과
- 커밋 `c3b6025e`: 맥 `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` → **204 OK, 27 skipped**(녹화기 필요 테스트); DGX(venv, 녹화기 경로 설정) → **204 OK, 0 skipped**, 145 s (`wm4/gate_c3b6025e`, 17:51:50–17:54:15). WM-4 테스트 22개 포함(기존 182개 그대로 녹색).
- 허용 경로: `git diff --name-only 1bfb6ae0..HEAD`는 `tools/worldmodel/wm4/**`, `docs/worldmodel/wm4/**`, `docs/worldmodel/wm3/WM3.md`(맨 위 한 줄)만.
- 녹색 → `codex/worldmodel-2026-10`을 `1bfb6ae0..c3b6025e`로 fast-forward(강제 푸시 없음). 이 보고서 커밋(문서 전용) 뒤 게이트를 다시 돌려 녹색이면 두 브랜치를 같은 커밋으로 맞춤(최종 메시지에 `git ls-remote` 기재).
- PR #512(draft 유지) 댓글: https://github.com/hyunlord/new-world/pull/512#issuecomment-6231993032

### 정직한 판단
- **교란이 있는 관측 데이터에서 CRN·CT는 단순 기준선과 M1보다 나은가?** 아니다. 관측 테스트에서 CRN-obs(7일 0.497, 30일 0.863)와 CT-obs(0.321, 0.373)는 "효과 없음"(0.218, 0.168)보다 분명히 나쁘고(O1·O2 NONE), M1b-obs(0.214, 0.175)는 효과 없음과 비긴다. 다만 CT는 논문 스텝의 4 %, 약한 균형 가중치로 학습된 상태라, "이 방법들이 이 예산에서 실패한다"까지만 말할 수 있다.
- **반사실 데이터(짝 갈래)는 얼마나 가치가 있는가?** 이 분할에서는 거의 없다. `wm3_full`은 CRN·CT-obs보다는 훨씬 낫지만(KEEP — 그 두 방법의 실패 때문) M1b-obs와는 같고(NONE/TIE), 짝 없는 WM-2 모델도 관측 테스트에서 같은 수준이며, 어떤 모델도 "효과 없음"을 이기지 못한다(사후). 사후 셀 분석에서 `wm3_full` 두 시드는 부패의 7일 식량 손실 평균(−76/−83 vs 참값 −81)을 맞히지만, 그 셀의 세계별 오차는 CT-obs가 더 낮다. O3는 데이터 양·세계·초기화·짝 손실이 함께 달라 반사실 데이터 자체의 가치를 분리하지 못한다.
- **데이터셋 v2에서 이 분야 방법들은 어디에 서는가?** 맨 아래다. 7일 CRN-v2 1.474, CT-v2 1.007 vs `an5_arm` 0.485·`wm3_full` 0.496; 30일 1.547, 1.374 vs `zero` 0.487(B1·B2 모두 NONE). 사실 30토큰 롤아웃은 나쁘지 않으므로 실패는 반사실 대비에서 온다.
- **WM-3의 7일 개선은 짝 구성 덕인가(세 시드)?** 그렇다(짝 손실·짝 창·위험도 손실을 묶은 구성). ctrl(3시드 평균) 대비 −0.075 [−0.100, −0.047], os 대비 −0.068 [−0.091, −0.046](둘 다 WEAK: 실재하나 20 % 미만). 30일에는 아니다(ctrl 대비 NONE).
- **논문은 지금 낼 수 있는가, 무엇이 남았는가?** 데이터셋·벤치마크 논문의 뼈대(두 분할, 과제·지표, 모든 방법의 결과표, 사전 등록)는 갖췄다. 내기 전에 남은 것: 예산에 맞춘 α 일정과 빠른 어텐션으로 CRN·CT 재학습(지금은 사후), 롤아웃의 개수 반올림, 균형화가 일할 수 있을 만큼 큰 관측 분할(세계당 여러 투입 또는 더 많은 세계), 최소 한 개 더의 장면. 지금 그대로라면 "기존 방법은 이 벤치마크에서 효과 없음을 이기지 못한다"는 결과 논문은 가능하지만, CT의 이른 체크포인트(α 일정)와 CRN·CT의 개수 비반올림이 심사에서 가장 먼저 지적될 것이다.
