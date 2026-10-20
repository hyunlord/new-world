# REPORT WM-1.3 (한국어)

## 구현 완료 보고

### 구현 의도
WM-1.2의 동점(16개 시험 세계, 80개 학습 세계, 3,000스텝 상한)이 데이터 부족인지 학습 부족인지 진짜 동점인지 판정할 수 없었다 → 학습 세계를 5배(400)로, 시험 세계를 80개로 늘리고, 9,000스텝까지 학습하고(WM-1.2는 3,000스텝 상한), 세계 단위 부트스트랩 신뢰구간으로 판정했으며, 분열하는 세계(`game_v17`)를 처음 함께 보았다.

### 구현 내용
- **Part 0**: `GOAL_1_3.md` 원문 커밋, Pass 11 최종 헤드 `31559232` 병합(`8f6b10ed`, 충돌 없음), DGX 빌드 클론 `_build/src_wm13`(93초), 게이트 117/0 스킵.
- **Part 1**: `wm1/recorder_equiv.py`(열 이름 매칭, 정수 정확·실수 비트 일치), `wm1/w13_manifest.py`, `wm1/w13_describe.py`; `game_v16` 30000–30559, `game_v17` 32000–32559 기록(1,120개, 892 MB), `w13_manifest.json`, 레지스트리 `w13_v16_n80/n200/n400`, `w13_v17_n400`.
- **Part 2**: `data.py` 데이터셋별 추가 상태 열(`extra_state`), `common.py` 중첩 학습 집합(`train_seeds`), `m1_gru.py`/`m1_tf.py` `ramp_steps`와 WM-1.2 정지 규칙 온라인 재현(`best_wm12rule.pt`), `final --ckpt/--eval-dataset/--out`, `evaluate.py` 세계별 점수(`per_world`), `wm1/bootstrap.py`, `climatology.py final --eval-dataset`, `probe_world.py` Y6 라벨, `wm1/judge_1_3.py`, DGX 실행 스크립트 `wm1/w13_launch/`, 테스트 `test_w13.py`·`test_judge_1_3.py`·`test_recorder_equiv.py`.
- **Part 3**: `METRICS_1_3.md` 사전 등록(`166eb737`), 리뷰 1차 후 개정(`60347cc3`).
- **Part 4**: M1b 8개 실행(전부 완료, 드롭 없음), 각 실행의 시험 평가·WM-1.2 규칙 체크포인트·옛 시험 평가·탐침.
- **Part 5**: 달력 계열·persistence·ridge를 네 데이터셋과 옛 시험에서 평가, 모든 Y 비교의 신뢰구간, 변수별 skill.
- **Part 6**: `WM1_3.md`(영문 분석), `WM1_2.md` 첫머리에 후속 문서 안내.
- **Part 7**: 독립 리뷰 2회, 맥·DGX 게이트, 녹색 브랜치 fast-forward, PR #512 코멘트, 이 보고서.

### 구현 방법
- **병합**: `git merge --no-ff 31559232` (`8f6b10ed`). `git merge-tree` 종료 코드 0, 253개 파일 모두 `wm1/` 밖. 병합 후 `wm1/` 밖 파일은 건드리지 않았다(`git diff --name-only 8f6b10ed..HEAD` = `wm1/` 경로뿐).
- **기록기 동등성**: 새 바이너리로 `game_v16` 20000–20003을 기록해 `p10/pilot/game_v16`(스키마 4, `58119582`)과 비교. 에피소드당 공유 열 62개(이름 매칭)가 모두 비트 단위 동일. 새 기록에만 있는 열: `settlements_alive`, `fissions_total`, `parent_slot`, `founded_tick`.
- **1,120개 세계 기록·검증·매니페스트**: `game_v17` 32000–32559(1,298초), `game_v16` 30000–30559(811초), 8 워커, 892 MB. 전부 `verify_episode` 통과, `meta.git_sha` = `8f6b10ed…`. 역할은 GOAL 4.2 그대로(학습 400 / 검증 80 / 시험 80). 기록기의 `--seeds a..b`는 반열린 구간이다(G1.1).
- **스키마 5 입력**: 데이터셋별 `extra_state`. `game_v16` 갈래는 WM-1.2의 65개 입력을 순서까지 그대로, `game_v17` 갈래는 상태 35개 + `g.settlements_alive` + `g.fissions_total`(원 누적값) + 달력 30개 = 67개(정확히 단언). 스키마 5 `game_v16` 에피소드의 특성표는 p10 짝과 동일하다(DGX 게이트 테스트).
- **커리큘럼과 스텝 상한 분리(`ramp_steps`)**: 기본값 0 = `max_steps // 2`(기존 동작, 테스트로 확인). 긴 실행은 `--ramp-steps 1500 --max-steps 9000 --patience 12 --eval-every 150`. WM-1.2의 정지 규칙(3,000 상한, 1,500부터 인내 5)을 같은 실행 안에서 온라인으로 재현해 `best_wm12rule.pt`로 저장하고, 검증으로 반올림을 따로 골랐다.
- **세계 단위 부트스트랩**: `t2`가 세계(시드)별 변수별 nMAE를 원점수와 편향 제거 점수로 저장한다. 세계별 평균의 평균이 합산 평균과 1e-12 안에서 같다(테스트, 실제 파일에서는 7e-16). `bootstrap.py`: 시험 세계 80개를 복원 추출, B = 2,000, `default_rng(0)`, 95 % 백분위 구간. 모델 점수는 시드 평균, 비교는 짝지음.
- **교차 데이터셋 평가**: `final --eval-dataset`은 입력 목록이 같을 때만 허용한다. 실행 자신의 정규화기와 반올림을 쓰고, 표준편차는 대상 시험 분할에서 잡는다. WM-1.2의 세 실행을 새 시험에서, N = 400 모델을 옛 시험에서 평가했다. 달력 계열도 같은 방식(`climatology.py final --eval-dataset`)으로 옛 시험에서 평가했다.
- **사전 등록**: `METRICS_1_3.md`를 `166eb737`(커밋 12:23:37, 푸시 12:25:13)으로 올렸고, 모든 시험 숫자는 그 뒤에 나왔다. 리뷰 1차 후 개정 `60347cc3`(분열 시점 서술 정정, 기술 통계 공개, 수렴 판정 기준, 실행 스크립트 감시 장치)은 `game_v17`·N = 80·N = 200의 시험 숫자보다 먼저 들어갔다. 이후 시험 작업은 `60347cc3`에서 돌렸고, 채점 코드는 `166eb737`과 바이트 단위로 같다.
- **학습 대기열과 처리량**: k = 168에서 혼자 1.50 s/step, 둘이면 각각 3.27 s/step(2.17배 ≥ 1.8배)이라 한 번에 하나씩 돌렸다. 순서는 9.1의 쌍 순서를 펼친 것이다. `--max-seconds` = min(30000, 마감 − 지금 − 3600). 실행이 끝나면 시험 평가를 자동으로 띄웠고, 30분 동안 로그가 없으면 죽이고 재시도하는 감시 장치를 붙였다(G4.3 이후).
- **탐침**: WM-1.2 탐침(`M1TF.hidden`, 166행, 검증으로 λ 선택, 시험 R²)에 지역 매개변수 3개를 더했다. 학습이 끝난 뒤 관측 열에서 읽은 라벨 3개를 추가했다: 120일까지 분열 여부, 2,832행의 정착지 수, 첫 분열일. 첫 분열일은 학습 범위 안의 세계만으로도 따로 계산했다.

### 기능 설명
- **`game_v16`, N = 400**: 원점수에서 모델이 달력 계열(`an5`)을 이긴다. 신뢰구간이 0 아래다(168 h −0.013, 24 h −0.025). 다만 20 % 차이(KEEP)에는 못 미친다. N = 400에서 세계별 평균 오차를 빼면 둘이 같다(Y2). 그 크기에서 모델의 우위는 "그 세계의 수준"을 더 잘 읽는 데서 오고, 일주일 동안 세계가 어떻게 움직이는지를 더 잘 맞히는 데서 오지 않는다. (N = 200 단일 시드는 편향 제거에서도 −0.006 [−0.012, −0.001]로 앞서고, N = 80은 −0.004 [−0.009, 0.002]다. 둘 다 사전 등록 규칙이 아니다.)
- **주로 학습 부족이었다**: 같은 N = 80 실행을 WM-1.2 규칙 체크포인트(≤ 3,000스텝)에서 보면 0.214, 9,000스텝까지 학습하면(대부분 미수렴) 0.197이다. 세계를 5배로 늘려도 모델은 거의 그대로이고(−0.002, 구간에 0 포함), 달력 계열은 −0.020 좋아진다. WM-1.2 규칙 체크포인트는 N = 80에서는 `an5`(N = 80)를 이기고(−0.014 [−0.020, −0.007]) N = 400에서는 진다(+0.008 [0.003, 0.013]). 둘 다 사전 등록되지 않은 짝 비교다.
- **분열하는 세계(`game_v17`)**: 모델의 우위가 사라진다(원점수 TIE). 편향 제거 점수에서는 모델이 진다(NONE). 탐침에서 모델은 세계의 먹이 쪽 매개변수를 원시 행보다 훨씬 잘 알아내지만, 앞으로의 분열은 원시 행 이상으로 알지 못한다.
- **데이터셋 v2에 정해진 것**: 장면당 학습 400 이상, 검증 80, 시험 80 이상(구간 폭은 시험 세계 수가 정한다). 실행당 9,000스텝 이상(GB10에서 약 4시간, 한 번에 하나), 인내 창 1,800스텝 이상, 구성당 시드 3개. 기록할 판은 `game_v18`인데, UBS(이력에서 생기는 세계 간 차이)가 `game_v17`보다 구간 밖으로 높아야 기록할 가치가 있다. Pass 12의 J7b 결과로는 아직 아니다.

### 변경된 파일 목록
- `60083d4d` Part 0 — GOAL_1_3.md (unchanged); WM-1.3 headings in DECISIONS.md and RUNLOG.md — DECISIONS.md GOAL_1_3.md RUNLOG.md 
- `8f6b10ed` Part 0 — merge codex/revitalize-2026-10 at 31559232 (Pass 11 final: recorder schema 5, game_v17 scenes) — (병합: Pass 11의 253개 파일, 모두 `wm1/` 밖)
- `d68114c3` Part 0 — build clone _build/src_wm13 at 8f6b10ed; DGX gate 117 OK / 0 skipped; run log — RUNLOG.md 
- `a804cf82` Part 1 — recorder_equiv.py (columns matched by name, exact / bit-identical) and its tests — recorder_equiv.py test_recorder_equiv.py 
- `c51914c6` Part 2 — harness for scale: schema-5 inputs (extra_state per dataset), nested train_seeds, ramp_steps, WM-1.2 rule online (best_wm12rule.pt), per-world T2 scores, bootstrap.py, final --ckpt/--eval-dataset/--out, probe Y6 labels; tests — bootstrap.py climatology.py common.py data.py evaluate.py m1_gru.py m1_tf.py probe_world.py test_pilot_harness.py test_w13.py 
- `5c7299dc` Part 2 — judge_1_3.py (Y rules, intervals, tables) and test; climatology final --eval-dataset (legacy test); w13 manifest and describe tools; DGX launch scripts — climatology.py judge_1_3.py test_judge_1_3.py w13_describe.py w13_launch/finals.sh w13_launch/gate.sh w13_launch/README.md w13_launch/record.sh w13_launch/slot.sh w13_manifest.py 
- `b7a48fed` Part 1 — w13_manifest.json (1,120 episodes of game_v16 / game_v17, schema 5, verified on the DGX); Part 1-2 decisions; WM1_2.md points to WM1_3.md — DECISIONS.md w13_manifest.json WM1_2.md 
- `166eb737` Part 3 — METRICS_1_3.md pre-registration (before any test-role number); calendar validation runs, data description and recorder equivalence outputs; run log, Part 4 decisions — DECISIONS.md METRICS_1_3.md RUNLOG.md runs/calendar_val_w13_v16_n200_b7a48fed/ runs/calendar_val_w13_v16_n400_b7a48fed/ runs/calendar_val_w13_v16_n80_b7a48fed/ runs/calendar_val_w13_v17_n400_b7a48fed/ runs/recorder_equiv_a804cf82/ runs/w13_describe_b7a48fed/ 
- `a257073a` Part 5 — calendar family, persistence and ridge on test for the four w13 datasets and the legacy test (from 166eb737, after the pre-registration) — runs/calendar_w13_v16_n200_166eb737/ runs/calendar_w13_v16_n400_166eb737/ runs/calendar_w13_v16_n400_on_p10_v16_166eb737/ runs/calendar_w13_v16_n80_166eb737/ runs/calendar_w13_v17_n400_166eb737/ 
- `902ed7c5` Part 4 — run log (pre-registration, baselines, DGX gate 142 OK / 0 skipped on 166eb737); push decision G0.2 — DECISIONS.md RUNLOG.md 
- `8dd46359` Part 5 — WM-1.2's three runs on the new game_v16 test (Y3 delta_steps_xworld inputs) — RUNLOG.md runs/m1tf_p10v16_r3_s0_on_w13v16/ runs/m1tf_p10v16_r3_s1_on_w13v16/ runs/m1tf_p10v16_r3_s2_on_w13v16/ 
- `608b3b92` Part 4 — m1tf_w13v16_n400_s0 run files and its three test evaluations; run log (pause, queue) — RUNLOG.md runs/m1tf_w13v16_n400_s0_legacy/ runs/m1tf_w13v16_n400_s0_short/ runs/m1tf_w13v16_n400_s0/ 
- `2c9a368f` Part 4 — slot.sh watchdog (a silent run is killed, moved aside and retried once) after a hung CUDA context; decision G4.3, run log — DECISIONS.md RUNLOG.md w13_launch/slot.sh 
- `406a917c` Part 4 — m1tf_w13v16_n400_s1 run files and its three test evaluations; run log — RUNLOG.md runs/m1tf_w13v16_n400_s1_legacy/ runs/m1tf_w13v16_n400_s1_short/ runs/m1tf_w13v16_n400_s1/ 
- `60347cc3` Part 7 — review round 1 (no BLOCKING; 1 MAJOR, 3 MINOR): METRICS amendment (fission timing, describe disclosure, convergence flag, launcher), probe first-fission R2 inside the train range, judge flags and per-seed CIs; REVIEW.md — DECISIONS.md judge_1_3.py METRICS_1_3.md probe_world.py REVIEW.md test_w13.py 
- `9e5bd790` Part 7 — run log (review round 1, DGX gate 143 OK / 0 skipped on 60347cc3); times corrected — METRICS_1_3.md REVIEW.md RUNLOG.md 
- `627f04cd` Part 4 — m1tf_w13v16_n400_s2 run files and its three test evaluations; run log — RUNLOG.md runs/m1tf_w13v16_n400_s2_legacy/ runs/m1tf_w13v16_n400_s2_short/ runs/m1tf_w13v16_n400_s2/ 
- `17be0429` Part 4 — m1tf_w13v17_n400_s0 run files, its test evaluation and the Y6 probe; run log — RUNLOG.md runs/m1tf_w13v17_n400_s0/ runs/probe_m1tf_w13v17_n400_s0/ 
- `024e08a6` Part 4 — m1tf_w13v16_n80_s0 run files and its two test evaluations; run log — RUNLOG.md runs/m1tf_w13v16_n80_s0_short/ runs/m1tf_w13v16_n80_s0/ 
- `ab43c3da` Part 4 — run log (pause, connection loss over the Tailscale relay) — RUNLOG.md 
- `4473d81c` Part 4 — m1tf_w13v16_n80_s1 run files and its two test evaluations; run log — RUNLOG.md runs/m1tf_w13v16_n80_s1_short/ runs/m1tf_w13v16_n80_s1/ 
- `9dbe06d2` Part 4 — m1tf_w13v17_n400_s1 run files and its test evaluation; run log — RUNLOG.md runs/m1tf_w13v17_n400_s1/ 
- `9b7d9281` Part 6 — m1tf_w13v16_n200_s0 and its evaluations; final judge_1_3 (Y1 WEAK, Y1b WEAK, Y2 TIE, Y4 TIE, Y4br NONE); WM1_3.md; run log — RUNLOG.md runs/judge_1_3/ runs/m1tf_w13v16_n200_s0_short/ runs/m1tf_w13v16_n200_s0/ WM1_3.md 
- `03580fef` Part 6 — Part 4-6 decisions (nothing dropped, baselines once, skill intervals, assembly of WM1_3.md) — DECISIONS.md 
- `ad24cd47` Part 7 — review round 2 (no BLOCKING; 2 MAJOR, 9 MINOR wording / records): WM1_3.md prose corrected, launcher deviation recorded, REVIEW.md, run log — DECISIONS.md METRICS_1_3.md REVIEW.md RUNLOG.md WM1_3.md
- (이 보고서 커밋과 그 뒤 게이트 기록 커밋은 최종 메시지에 적는다.)

### 측정 결과

**Y 표**(80개 시험 세계, 세계 단위 짝 부트스트랩 95 % 구간; WEAK는 구간이 0 아래여야 하므로 WM-1.2의 점추정 WEAK보다 엄격하다)

| # | 비교 | M1b 시드 평균(시드별) | 기준 | 모델 − 기준 [95 % CI] | 판정 |
|---|---|---|---|---|---|
| Y1 | v16 N = 400 원점수 T2 168 h(매시 시작) | 0.194 (0.193, 0.194, 0.195) | `an5` 0.207 | −0.013 [−0.019, −0.008] | **WEAK** |
| Y1b | 같은 조건 24 h | 0.133 (0.134, 0.133, 0.132) | `an5_off` 0.158 | −0.025 [−0.027, −0.023] | **WEAK** |
| Y2 | v16 N = 400 편향 제거 168 h | 0.201 (0.200, 0.203, 0.200) | `an5`(cal_best_anom) 0.199 | +0.002 [−0.003, 0.008] | **TIE** |
| Y3 | 데이터 대 스텝(판정 없음) | Δ_steps −0.0170 [−0.0196, −0.0143] · Δ_data −0.0023 [−0.0051, 0.0003] · Δ_cal −0.0196 [−0.0237, −0.0158] · Δ_data − Δ_cal +0.0173 [0.0138, 0.0208] → "규모가 모델을 더 돕는다" 아님 · Δ_steps_xworld(교란됨) −0.0291 [−0.0329, −0.0254] | | | 보고 |
| Y4 | v17 N = 400 원점수 168 h | 0.187 (0.189, 0.184) | `an5` 0.189 | −0.002 [−0.008, 0.003] | **TIE** |
| Y4br | v17 N = 400 편향 제거 168 h | 0.191 (0.197, 0.186) | `an5` 0.179 | +0.013 [0.007, 0.018] | **NONE** |
| Y5 | 학습 곡선 168 h(N = 80 / 200 / 400) | M1b 0.197 [0.188, 0.205] / 0.187 [0.179, 0.195] / 0.194 [0.186, 0.202] | `an5` 0.227 [0.218, 0.237] / 0.215 [0.207, 0.223] / 0.207 [0.200, 0.215] | 24 h: M1b 0.134 / 0.131 / 0.133, 달력 최선 0.167 / 0.163 / 0.158 | 보고 |
| Y6 | v17 탐침(`n400_s0`, R²; 분류는 정확도) | starting_agents 0.999(원시 행 0.903) · berry_bushes 0.752(0.568) · berry_capacity 0.753(0.386) · food_regen 0.571(0.117) · wild_regions 0.470(0.456) · deadwood_capacity 0.266(0.224) · region_deadwood 0.214(0.211) · deadwood_spots 0.175(0.117) · region_bushes 0.012(0.026) · forest −0.040(−0.028) · climate 0.388(0.388, 다수 클래스 비율과 같아 식별 없음) · 120일까지 분열 0.912(0.925, 다수 클래스 0.662) · 2,832행 정착지 수 0.437(0.437) · 첫 분열일 −0.019(−0.022), 학습 범위(29–36일) 안 52개 세계 −0.006(−0.011) | 원시 37열 | | 보고 |

시드별 짝 CI — Y1: −0.014 [−0.020, −0.009], −0.013 [−0.018, −0.008], −0.012 [−0.018, −0.006]. Y2: 0.001 [−0.005, 0.007], 0.005 [−0.001, 0.010], 0.001 [−0.005, 0.007]. Y4: 0.000 [−0.006, 0.006], −0.005 [−0.010, 0.001]. Y4br: 0.018 [0.012, 0.024], 0.007 [0.002, 0.013]. 같은 규칙으로 보고만 한 것: N = 80 −0.030 [−0.037, −0.024] WEAK, N = 200 −0.028 [−0.033, −0.022] WEAK.

**데이터셋·크기·지평별 점수**(시험, 매시 시작; 원점수 / 편향 제거)

| 데이터셋 | 지평 | M1b 시드들 → 평균 | cal_best | cal_best_fair | cal_best_anom | apfit_abs | persistence | ridge |
|---|---|---|---|---|---|---|---|---|
| v16 N=80 | 168 h 원 | 0.190, 0.203 → 0.197 | 0.227 (an5) | 0.248 | 0.227 | 0.232 | 0.590 | 0.318 |
| v16 N=80 | 168 h 편향 제거 | 0.198, 0.211 → 0.205 | 0.208 | 0.241 | 0.208 (an5) | 0.226 | 0.597 | 0.276 |
| v16 N=80 | 24 h 원 | 0.132, 0.136 → 0.134 | 0.167 (apfit_abs) | 0.168 | 0.167 | 0.167 | 0.216 | 0.231 |
| v16 N=80 | 24 h 편향 제거 | 0.139, 0.141 → 0.140 | 0.167 | 0.168 | 0.167 (apfit_abs) | 0.167 | 0.227 | 0.215 |
| v16 N=200 | 168 h 원 | 0.187 | 0.215 (an5) | 0.245 | 0.215 | 0.229 | 0.590 | 0.315 |
| v16 N=200 | 168 h 편향 제거 | 0.196 | 0.202 | 0.238 | 0.202 (an5) | 0.224 | 0.597 | 0.275 |
| v16 N=200 | 24 h 원 | 0.131 | 0.163 (an5_off) | 0.168 | 0.163 | 0.166 | 0.216 | 0.229 |
| v16 N=200 | 24 h 편향 제거 | 0.138 | 0.165 | 0.168 | 0.165 (an5_off) | 0.167 | 0.227 | 0.215 |
| v16 N=400 | 168 h 원 | 0.193, 0.194, 0.195 → 0.194 | 0.207 (an5) | 0.246 | 0.207 | 0.230 | 0.590 | 0.314 |
| v16 N=400 | 168 h 편향 제거 | 0.200, 0.203, 0.200 → 0.201 | 0.199 | 0.239 | 0.199 (an5) | 0.225 | 0.597 | 0.276 |
| v16 N=400 | 24 h 원 | 0.134, 0.133, 0.132 → 0.133 | 0.158 (an5_off) | 0.169 | 0.158 | 0.166 | 0.216 | 0.228 |
| v16 N=400 | 24 h 편향 제거 | 0.140, 0.141, 0.138 → 0.140 | 0.161 | 0.169 | 0.161 (an5_off) | 0.167 | 0.227 | 0.214 |
| v17 N=400 | 168 h 원 | 0.189, 0.184 → 0.187 | 0.189 (an5) | 0.246 | 0.189 | 0.229 | 0.478 | 0.307 |
| v17 N=400 | 168 h 편향 제거 | 0.197, 0.186 → 0.191 | 0.179 | 0.241 | 0.179 (an5) | 0.227 | 0.483 | 0.273 |
| v17 N=400 | 24 h 원 | 0.120, 0.116 → 0.118 | 0.126 (an5_off) | 0.132 | 0.126 | 0.129 | 0.158 | 0.191 |
| v17 N=400 | 24 h 편향 제거 | 0.125, 0.121 → 0.123 | 0.129 | 0.134 | 0.129 (an5_off) | 0.131 | 0.164 | 0.180 |

(06:00 시작 표와 각 값의 신뢰구간은 `WM1_3.md` 부록 A와 `runs/judge_1_3/judge.md`에 있다. v17 24 h, 보고만: 모델 − `an5_off` −0.009 [−0.010, −0.007], 편향 제거 −0.006 [−0.008, −0.004].)

**WM-1.2 규칙 체크포인트(같은 실행, ≤ 3,000스텝) 대 9,000스텝 학습**(v16, 168 h 원점수): N = 400 s0/s1/s2 0.208 / 0.213 / 0.225(평균 0.216) 대 0.193 / 0.194 / 0.195. N = 80 s0/s1 0.209 / 0.218(0.214) 대 0.190 / 0.203. WM-1.2의 세 실행을 새 시험에 올리면 0.221 / 0.227 / 0.228(평균 0.226).

**실행별 종료 이유·최적 스텝·시간**

| 실행 | 종료 | 최적 스텝(검증 T2 24 h) | 마지막 스텝 | 초 | 수렴 | WM-1.2 규칙 최적 스텝 |
|---|---|---:|---:|---:|---|---:|
| v16 n400_s0 | max_steps | 8,100 (0.0772) | 9,000 | 14,217 | 미수렴 | 2,400 (0.0967) |
| v16 n400_s1 | max_steps | 7,500 (0.0778) | 9,000 | 12,909 | 수렴 | 2,850 (0.0951) |
| v16 n400_s2 | max_steps | 8,700 (0.0772) | 9,000 | 14,253 | 미수렴 | 3,000 (0.0958) |
| v16 n80_s0 | max_steps | 9,000 (0.0770) | 9,000 | 13,764 | 미수렴 | 3,000 (0.0927) |
| v16 n80_s1 | max_steps | 8,400 (0.0794) | 9,000 | 14,182 | 미수렴 | 2,850 (0.0933) |
| v16 n200_s0 | max_steps | 8,700 (0.0784) | 9,000 | 13,612 | 미수렴 | 2,550 (0.0918) |
| v17 n400_s0 | patience | 5,400 (0.0616) | 7,200 | 11,153 | 수렴 | 2,550 |
| v17 n400_s1 | max_steps | 7,350 (0.0604) | 9,000 | 13,953 | 수렴 | 2,100 |

**변수별 skill**(1 − nMAE / 기준, T2 168 h 매시, 시드 평균, [95 % CI])
- v16 N=400 원점수, `an5` 대비: 식량 +0.093 [0.025, 0.157], 생존자 +0.302 [0.207, 0.395], 덤불 +0.062 [0.037, 0.088], 일하는 성인 +0.065 [0.050, 0.080]. `apfit_abs` 대비: 식량 +0.319, 생존자 −0.131 [−0.261, 0.003], 덤불 +0.204, 일하는 성인 +0.087.
- v16 N=400 편향 제거, `an5` 대비: 식량 +0.139 [0.079, 0.200], 생존자 −0.186 [−0.339, −0.037], 덤불 +0.057, 일하는 성인 +0.001 [−0.016, 0.019].
- v17 N=400 원점수, `an5` 대비: 식량 +0.164 [0.109, 0.213], 생존자 +0.273 [0.182, 0.365], 덤불 −0.017 [−0.055, 0.022], 일하는 성인 +0.014 [−0.003, 0.032], 땔감 −0.030 [−0.078, 0.017].
- v17 N=400 편향 제거, `an5` 대비: 식량 +0.090, 생존자 −0.022, 덤불 −0.020, 일하는 성인 −0.050 [−0.069, −0.030], 땔감 +0.018.
- 모델의 생존자 오차(0.110)는 persistence와 정확히 같다. 인구를 그대로 들고 간다는 뜻이다.

**v17 데이터 설명**(시험 80개 세계): 1회 이상 분열한 세계는 학습 243/400(61 %), 검증 56/80(70 %), 시험 53/80(66 %). 첫 분열은 29일에 몰려 있다(학습 155/243, 시험 34/53). 시험의 나머지는 30–36일 18개, 85일 1개다. 마지막 행의 정착지 수는 시험에서 1개 30, 2개 20, 3개 30이다. η²(v16 → v17): 식량 0.35 → 0.60, 덤불 0.35 → 0.75, 땔감 0.08 → 0.24. R²_abs: 식량 0.65 → 0.38, 덤불 0.64 → 0.24. dyn-η²: 식량 0.23 → 0.59. 노출 사망(120일까지 평균)은 2.8 → 8.5.

**옛 시험 비교**(WM-1.2의 `p10_v16` 시험 세계 16개, 168 h 원점수): N = 400 모델 0.198 [0.172, 0.228], `an5`(N = 400) 0.198, 차 0.000 [−0.009, 0.011]. 24 h는 0.131 대 `an5_off` 0.155, −0.024 [−0.029, −0.019]. 편향 제거 168 h는 0.201 대 0.190, +0.011 [−0.000, 0.023]. 같은 16개 세계에서 WM-1.2는 모델 0.221, `an5` 0.206이었다. 모델의 0.221 → 0.198 변화는 학습 세계, 스텝, 검증 세계가 모두 달라 교란된 비교다.

### 환경
- **DGX**: GB10, 드라이버 580.126.09, CUDA 13.0, `nproc` 20. RAM 119 GiB(가용 53–102 GiB). 스왑 15.5/16 GiB는 시작부터 끝까지 다른 서비스가 쓰고 있었다. 디스크 1.6 T 여유. 다른 사용자 부하 평균은 1–50.
- **처리량**: 혼자 1.50 s/step, 동시 2개 3.27 s/step(각), 작업당 GPU 메모리 21,085 MiB. 실제 실행은 k = 168에서 1.5–1.6 s/step, 9,000스텝 실행 하나가 3.6–4.0 h(11,153–14,253 s). 시험 평가 1회(80개 세계)는 5–11분, 옛 시험(16개 세계)은 약 2분, 학습 끝의 검증 작업은 6–7분, 탐침은 2분, 달력 계열 평가는 51–188초.
- **연결과 장애**: 2026-10-13 16:47 DGX 커널이 NVRM 메모리 부족을 기록했고, 이어 n400_s1 첫 시도가 CUDA 호출에서 멈췄다. 죽이고 재시작해 약 47분을 잃었다(G4.3). 10-14 08:54–09:59에는 맥과 DGX 사이 경로가 끊겼다(Tailscale 직결 실패, 도쿄 DERP 중계로만 연결). DGX 작업은 영향이 없었다. 세션 정지는 10-13 14:55–15:11, 18:44–19:54, 10-14 08:12–08:54, 18:38–20:29(맥 무응답)였다.

### 독립 리뷰
- **1차**(새 `general-purpose` 서브에이전트, 10-13 22:13–22:25, 헤드 `2c9a368f`): BLOCKING 없음, MAJOR 1, MINOR 3, NOTE 4.
  - 재현한 것: 병합(253개 파일, `wm1/` 밖은 `31559232`와 동일), 입력 목록 65/67, 사전 등록 순서(시험 산출물 전부 12:26:20 이후, `166eb737`, `forced` false), 부트스트랩 손계산(`n400_s0` − `an5`: −0.0144294 [−0.0204632, −0.0087725], `bs.paired`와 일치), `Wm12Rule` 재현.
  - R1-1(MAJOR): METRICS의 "29일 고정" 서술이 데이터와 다르다(첫 분열 29–36일, 시험 한 세계 85일) → 날짜를 박은 개정, 학습 범위 안 R²를 추가 보고(탐침 실행 전).
  - R1-2: 사전 등록 전에 기술 통계가 시험 분할을 읽음 → 공개.
  - R1-3: 수렴 판정의 경계 → 경계 포함으로 결정(G4.4), 표에 표시.
  - R1-4: 감시 장치 개정 → 개정에 기록.
  - NOTE 4건 처리.
- **2차**(새 서브에이전트, 10-14 20:30–20:45, 헤드 `9b7d9281`/`03580fef`): BLOCKING 없음, MAJOR 2, MINOR 9, NOTE 4.
  - 재현한 것: judge 바이트 단위 재현, numpy 손계산으로 Y1/Y1b/Y2/Y4/Y4br/Y3 일치, WM1_3 표 330/359행 원문 일치와 나머지 29행·산문 숫자 약 45개 재계산, 개정 후 산출물 전부 `60347cc3` 이후·`forced` false, `166eb737..60347cc3`의 `tools/` 변경은 4개 파일뿐.
  - R2-1(MAJOR): §1에서 "WM-1.2 규칙에서 N = 80 실행이 진다"고 했으나 실제로는 N = 80에서 이기고 N = 400에서 진다 → 수정, 짝 구간 병기.
  - R2-2(MAJOR): §8(a)에서 "모든 N에서 편향 제거 동점"이라 했으나 N = 200(단일 시드)은 −0.006 [−0.012, −0.001] → N = 400(Y2)로 한정.
  - MINOR 9건: "수렴까지 학습" 표현, 추측성 문장, 옛 시험 교란 표시, 장면 간 nMAE 비교, 코드 커밋 표기, 검증 곡선 서술과 0.092–0.097, `n200`의 `--max-seconds` 26246 실행기 편차(마감 파일을 슬롯 시작 때 한 번만 읽음; 영향 없음, G4.6), G6.1 문구, 미수렴 표시 범위 → 전부 수정.
  - NOTE 4건: 개정 시각 22:25, 시간 수치, climate가 다수 클래스와 같음, Pass 13의 PS → 반영.
- 기록: `REVIEW.md` `## WM-1.3`.

### 자율 결정 목록
- **G0.1–G0.2**: graft 관련 파일은 스테이징하지 않는다. 마감 = 시작 + 36 h + 정지 시간. 큰 JSON을 푸시할 때는 HTTP/1.1을 쓴다.
- **G1.1–G1.4**: 시드 범위는 반열린 구간이다. 기록기 동등성은 동일로 확인했다. 30000–30111 재사용은 유출이 아니다(Pass 10의 임시 시드, 어떤 데이터셋에도 없음). `game_v17`을 먼저 기록했다.
- **G2.1–G2.10**: 데이터셋별 추가 상태 열, 중첩 학습 집합, `ramp_steps`, WM-1.2 규칙 온라인 재현(시간 상한은 재현하지 않음), 세계별 점수, 부트스트랩 정의, 교차 데이터셋 평가, 옛 시험용 달력 평가, 탐침 확장, 작업별 git worktree.
- **G4.1–G4.6**: 한 번에 하나씩 학습(처리량 2.17배). 멈춘 실행은 재시작하고 감시 장치를 붙였다. 수렴 판정은 경계 포함(best ≥ 0.9 × last)이다. 개정 후 시험 작업은 `60347cc3`에서 돌렸다. 아무 실행도 드롭하지 않았다.
- **G5.1–G5.3**: 기준선 시험 평가는 사전 등록 직후 1회만 했다. skill 구간은 평균의 비로 계산했다. v17 24 h 비교는 보고만 한다.
- **G6.1–G6.2**: WM1_3.md는 judge 산출물에서 조립했다. `game_v18` 서술은 Pass 12 문서를 인용한 것이다.
- **G7.x**: 리뷰 1·2차에 따른 개정(`60347cc3`), 문구 수정(`ad24cd47`). 2차 이후 숫자나 규칙은 바꾸지 않았다.

### 실행 기록
| Part | 시작 | 종료 | 소요 |
|---|---|---|---|
| 0 병합·빌드·게이트 | 10-13 11:27 | 11:33 | 6분 |
| 1 동등성·기록·매니페스트 | 11:32 | 12:11 | 39분(기록 35분) |
| 2 하네스 코드·검증 달력 | 11:35 | 12:12 | 37분 |
| 3 사전 등록 | 12:22 | 12:25 | 3분 |
| 4 처리량 확인 | 12:12 | 12:20 | 8분 |
| 4 학습 대기열(8개 실행, 직렬) | 12:21:56 | 10-14 19:12:50 | 30.8 h(멈춘 실행 재시작 47분 포함) |
| 4 시험 평가·탐침 | 10-13 16:18 | 10-14 19:22 | 각 실행이 끝날 때마다 |
| 5 기준선·교차 평가 | 10-13 12:25 | 12:57 | 32분 |
| 6 분석 | 10-14 20:29 | 20:30 | 1분(조립; 초안은 학습 대기 중 작성) |
| 7 리뷰·게이트·보고 | 리뷰 1차 10-13 22:13–22:25, 리뷰 2차 10-14 20:30–20:52, 게이트 20:53–20:55, 녹색 fast-forward·PR 코멘트 20:55–20:56, 보고서 20:56–21:05 | | |

**학습 대기열 타임라인**: n400_s0 12:21:56–16:18:56 → n400_s1(첫 시도 16:18:56–17:04 멈춤, 재시작 17:05:36)–20:41:37 → n400_s2–00:39:40 → v17_s0–03:45:41 → n80_s0–07:35:44 → n80_s1–11:32:46 → v17_s1–15:25:48 → n200_s0–19:12:50. 멈춘 구간: 16:50–17:04(n400_s1 멈춤). 정지 구간은 위 "환경"에 적었다.

**DGX 실행 디렉터리**(`/home/hyunlord/work/wm1-runs/`): `m1tf_w13v16_n400_s0`, `_s1`, `_s2`, `m1tf_w13v16_n80_s0`, `_s1`, `m1tf_w13v16_n200_s0`, `m1tf_w13v17_n400_s0`, `_s1`(각각 `_short` / `_legacy` 평가 디렉터리 포함), `m1tf_p10v16_r3_s{0,1,2}_on_w13v16`, `probe_m1tf_w13v17_n400_s0`, `calendar_val_*_b7a48fed`, `calendar_*_166eb737`, `w13_describe_b7a48fed`, `recorder_equiv_a804cf82`, 멈춘 시도 `_aborted/m1tf_w13v16_n400_s1_hang_1650`. 데이터는 `/home/hyunlord/work/worldsim-wm-data/w13/`.

### 확인된 제한사항
- `game_v16` 실행 6개 중 5개가 9,000스텝 상한에서 "미수렴"이다. Δ_data는 스텝 상한이 있는 학습으로 잰 값이다. 더 학습했을 때 N = 400이 N = 80보다 더 좋아지는지는 재지 않았다. 늦은 검증 곡선은 잡음이 크다(n400_s0: 7,500스텝 0.1148, 8,100 0.0772, 이후 0.0780–0.0866).
- N = 200은 시드 1개, N = 80과 v17은 2개다. 부트스트랩은 세계만 재표집하고 시드는 재표집하지 않는다. N = 80 두 시드의 차이(0.012)가 N = 400 모델의 우위와 비슷한 크기다.
- `game_v17`의 첫 분열일은 29–36일에 몰려 있다(시험의 한 세계만 85일). 그래서 탐침 R²는 거의 정보가 없다. 레시피도 v17용으로 조정하지 않았다.
- 데이터 기술 통계(`w13_describe`)는 사전 등록 전에 시험 분할을 읽었다. 예측기 점수도 선택도 아니며, 개정에서 공개했다.
- 장면마다 자기 분할 표준편차로 나누므로, v16과 v17은 모델 − 기준의 차이로만 비교한다.
- GPU 재현성은 측정하지 않았다.

### 게이트 결과
- **`ad24cd47`**(리뷰 2차 수정 후 헤드):
  - 맥 `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'`: **143 OK, 22 skipped**. 스킵은 기록기와 데이터 의존 테스트로, 맥에는 빌드와 데이터가 없다.
  - DGX(벤브, `WM_BIN`/`PROBE_BIN` = `_build/src_wm13`, `WM_DATA_ROOT`): **143 OK, 0 skipped**(72초).
  - 병합 직후 기대치 117(WM-1.2의 114 + Pass 11의 3)은 병합 커밋에서 맥 117 OK/20 skip, DGX 117 OK/0 skip으로 확인했다. 이번 실행에서 26개 테스트를 더했다.
- 앞선 DGX 게이트: `8f6b10ed` 117/0, `c51914c6` 141/1(매니페스트 이전), `166eb737` 142/0, `60347cc3` 143/0, `03580fef` 143/0.
- **허용 경로**: `git diff --name-only 8f6b10ed..ad24cd47`에서 `tools/worldmodel/wm1/`·`docs/worldmodel/wm1/` 밖 파일은 0개.
- **녹색 브랜치**: `codex/worldmodel-2026-10`을 `a43bdab2` → `ad24cd47`로 fast-forward(`git ls-remote` 확인).
- **보고서 커밋**(문서 전용)에서도 게이트를 다시 돌려 녹색 브랜치를 옮겼다(최종 메시지 참조).
- PR #512 코멘트: https://github.com/hyunlord/new-world/pull/512#issuecomment-6167749573

### 정직한 판단
- **WM-1.2의 동점은 주로 학습 부족이었다.** 같은 N = 80 실행이 WM-1.2 규칙에서는 0.214, 9,000스텝까지 학습하면 0.197이다. 세계를 5배로 늘려도 모델은 사실상 그대로이고(−0.002, 구간에 0 포함), 좋아진 쪽은 달력 계열이다(−0.020). 같은 16개 옛 시험 세계에서는 이번 N = 400 모델도 `an5`와 0.198 대 0.198로 같다. 80개 새 시험 세계와는 세계 자체가 달라서, 구간 폭만의 문제가 아니다.
- **5배의 세계와 9,000스텝 학습이면, 모델은 원점수에서 신뢰구간 밖으로 달력 계열을 이긴다.** 168 h −0.013 [−0.019, −0.008], 24 h −0.025 [−0.027, −0.023]. 그러나 KEEP(20 %)에는 한참 못 미친다(6 %, 16 %). N = 400에서 세계별 수준을 빼면 진짜 동점이다(+0.002 [−0.003, 0.008]). 모델이 가진 것은 "이 세계가 어떤 세계인가"를 읽는 능력이고, "이 세계가 일주일 동안 어떻게 움직이는가"를 예측하는 능력은 analog 다섯 개 평균 이상이 아니다.
- **분열하는 세계(v17)에서는 우위가 사라진다.** 원점수는 TIE(−0.002 [−0.008, 0.003]), 편향 제거는 모델이 진다(+0.013 [0.007, 0.018]). analog 라이브러리는 분열한 세계로 분열하는 세계를 예측하지만, 모델은 분열을 미리 알지 못한다(탐침에서 원시 행과 같은 수준).
- **데이터셋 v2**: 장면당 학습 400 이상, 검증 80, 시험 80 이상. 학습 세계 수보다 시험 세계 수가 결론을 정한다. 실행당 9,000스텝 이상, 구성당 시드 3개. 기록할 판은 `game_v18`이되, 이력에서 생기는 세계 간 차이(Pass 12의 UBS)가 `game_v17`보다 구간 밖으로 높아야 한다. Pass 12의 J7b(식량·생존자 ΔUBS 구간에 0 포함)와 J7 땔감 실패로 보면 v18은 아직 그 조건을 충족하지 못했다. `game_v16`은 분열 없는 대조군으로 함께 기록해야 한다.
