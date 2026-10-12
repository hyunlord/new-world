# WM-1.2 구현 완료 보고

## 구현 완료 보고

### 구현 의도
WM-1.1에서는 v0 시드들이 거의 같은 궤적을 그려(η² 0.02–0.09) 학습 모델이 달력(유사 사례·이상치 지속)을 이길 여지가 없었다. Pass 10의 `game_v16`은 세계마다 숨은 파라미터(창립 인원, 덤불·고사목 수와 용량, 숲 비율, 기후, 식량 재생)를 뽑는다 → WM-2가 쓸 하네스(스키마 4, 편향 제거 점수, 가족 평가기)를 만들고, "세계가 숨은 파라미터로 다를 때 학습 모델이 달력 기준선을 이기는가, 그 차이는 세계 식별인가 역학 예측인가"를 파일럿에서 사전 등록 규칙(Z1~Z5)으로 처음 판정한다.

### 구현 내용
- **Part 0** `GOAL_1_2.md` 커밋, `## WM-1.2` 헤딩, `origin/codex/revitalize-2026-10` = `14372ea2`를 `--no-ff` 병합(`b20a991f`, 충돌 없음: `wm_data.py` 스키마 1–4·`load_family`, `bench_v1.py`, Rust 레코더·프로브). DGX 코드 클론 detach, 빌드 클론 `_build/src_wm12`에서 `cargo build --release -p sim-test`(74초).
- **Part 1** 파일럿 336개 에피소드와 `p9/unforked/game_v15_s12000_none`을 DGX로 복사·검증; `docs/worldmodel/wm1/p10_pilot_manifest.json`(+`wm1/pilot_manifest.py`); `wm1/common.py`에 데이터셋 레지스트리 `Dataset`/`DATASETS`(`v0`, `p10_v16`, `p10_v16n`, `p10_v15`: 매니페스트·sha, 학습/검증/시험 그룹, 반사실 그룹, 과제 목록) + `test_registry.py`.
- **Part 2** `wm1/golden_v0.py`·`golden_v0.json`·`test_golden_v0.py`(병합 시점 v0 골든 값과 비트 동일성); `wm1/data.py` 스키마 ≥ 2 관측 테이블(`episode_table_observable`, `bench_of`, `ALL_STATE_OBS`, `Variant.obs_state`, `variant_for`), `wm1/evaluate.py`의 `evaluate(tasks=, bias_removed=)`, `demeaned_errors`/`bias_removed_score`, 벡터화 `Persistence.rollout`, 스키마 ≥ 2에서 severity 입력 거부; `wm1/climatology.py --dataset`(`tasks_for`, `cal_best_anom`, persistence·관측 ridge 참조); `m1_gru.py`/`m1_tf.py --dataset`, `final`은 `config.json`에서 데이터셋을 읽음(`test_plan`); `test_pilot_harness.py`.
- **Part 3** `docs/worldmodel/wm1/METRICS_1_2.md`(사전 등록, `1a7dfdb6`), `wm1/judge_1_2.py` + `test_judge_1_2.py`.
- **Part 4** M1b(r3 레시피) `game_v16` 세 시드와 `game_v15` 대조 한 시드의 학습·시험 평가; `M1TF.hidden`과 `wm1/probe_world.py`(세계 식별 탐침) + `test_probe.py`.
- **Part 5** `wm1/families.py`(가족 평가기) + `test_family_eval.py`; 실데이터 스모크 `runs/families_smoke_096d92d2`.
- **Part 6** `docs/worldmodel/wm1/WM1_2.md`(영문 분석), `WM1_1.md` 첫머리 한 줄.
- **Part 7** 독립 리뷰 2라운드(`REVIEW.md` `## WM-1.2`), 게이트, 이 보고서.

### 구현 방법
- **병합**: `git merge --no-ff 14372ea2`. 병합 후 변경은 `tools/worldmodel/wm1/**`, `docs/worldmodel/wm1/**`만(확인: `git diff --name-only b20a991f..HEAD` 밖 0개).
- **파일럿 데이터**: `rsync -a --checksum`(13초, 228,815,078 B), DGX에서 `wm_data.verify_episode` 337/337 OK, 336개 모두 스키마 4·2,881행·`git_sha` `58119582…`. 매니페스트는 각 `meta.json`의 배열 sha256으로 작성; 역할 train 20000–20079(80), val 20080–20095(16), test 20096–20111(16), 항목마다 `split` = 장면, `schedule_id` = `none`.
- **스키마 4 하네스**: v0의 열 순서·특성은 그대로, 스키마 ≥ 2는 `bench_v1.observable_column`을 거친 관측 열만(숨은 열 요청 시 `HiddenInput`), `s.wood_target` 제외, `d.deaths_exposure`(시간별 차분)·`g.injured_members` 추가, 정착지 합/최대 축약 유지, `Layout.deaths`·`bench.deaths`에 노출 사망 포함, `EpisodeData.bench`는 `bench_v0.Ep`(severity = 숨은 severity, 라벨 전용). 입력 65개를 리터럴로 단언하고 숨은 필드 부재를 테스트. **v0 동일성**: `data.py` 수정 전 병합 커밋에서 샘플의 특성 테이블·타깃·하네스 수치 골든을 커밋, 이후 테스트가 비트 동일성을 단언(리뷰 후 ridge·달력 수치 추가; ridge 두 값만 BLAS 차이로 상대 1e-12).
- **편향 제거 점수**: 에피소드·변수별로 채점 시작점들의 오차 e = 예측 − 실제에서 그 에피소드의 평균 오차를 뺀 뒤 같은 split std로 nMAE. 세계별 정적 수준 오차를 지우고 움직임 오차만 남긴다(평균을 빼므로 치우친 오차에서는 원점수보다 클 수 있음).
- **달력 기준선 선택**: WM-1.1의 9개 멤버를 train 80개로 적합, val 16개에서 T1, T2 24/168 h(06:00·매시)와 각 편향 제거 버전으로 과제별 `cal_best`/`cal_best_fair`/`cal_best_anom` 선택, 시험에서 재선택·재적합 없음. persistence와 `bench_v1` 관측 ridge(λ = 100, val T1)를 같은 하네스로 평가.
- **사전 등록**: 검증 실행(`calendar_val_*_da379c17`)과 탐침·판정 코드(`ef880949`) 뒤 `METRICS_1_2.md`를 `1a7dfdb6`(21:59:00 커밋, 21:59:02 푸시)로 커밋; 모든 시험 수치는 그 뒤 `1a7dfdb6`에서 계산(달력 21:59:29, 모델 23:19–01:34).
- **M1b 학습**: WM-1 r3 레시피 그대로(`--batch 32 --clamp-nonneg 1 --eval-every 150 --k-max 168 --max-steps 3000 --roll-min-context 167 --roll-starts 1 --tf-l1 1.0 --threads 8 --v-weight 5.0 --window 336`), 차이는 데이터와 `sev-blind_action-blind`(파일럿에 개입 없음). DGX 작업은 동시에 최대 2개.
- **세계 식별 탐침**: 06:00 8일차(행 166, 단언) 창 −1..166(행 −1 마스크)에서 `M1TF.hidden`(마지막 위치의 `self.ln(z)`), train으로 ridge 적합·val MSE로 λ 선택, test R²; climate는 one-vs-rest 정확도. 참조는 같은 행의 관측 원시 행(35열). 라벨은 학습 후 탐침에서만 `meta.json`으로 읽음.
- **가족 평가기**: `load_family`로 읽은 가족에서 포크 행(공유 접두부, 검사)부터 사실·변형을 각자 기록된 행동으로 굴려 예측 효과를 만들고, 복제별로 짝지은 실제 효과의 분포(평균, 표본 표준편차)로 평균 효과 MAE·부호 정확도·포함률(실제 평균이 예측 ± spread 안)을 계산. 어두운 밤은 홈 정착지 슬롯(`settlement_ids.npy` 최소 id), 사망에 노출 사망 포함, 목표 창은 v1의 C_cf 첫 겨울(행 동일성 테스트).

### 기능 설명
- **확인된 것**: 숨은 파라미터가 다른 `game_v16`에서도 168시간 M1b는 검증에서 고른 유사 사례 `an5`를 넘지 못한다(Z1 NONE, 0.221 vs 0.206). 다만 세계가 달라지면 달력 계열의 오차는 크게 늘고(`an5` +50 %, `apfit_abs` +59 %, 달력 평균 +80–130 %) 모델의 오차는 거의 그대로(+5 %)여서, `an5`의 우위가 v15의 0.071에서 0.014로 줄었다. 모델이 얻는 것은 **세계 식별**이다: 은닉 상태가 창립 인원(R² 0.998)과 덤불 수(0.89)·용량(0.82)을 거의 복원하고, 168시간 `alive`를 마지막 값으로 유지해 persistence와 같은 0.080을 낸다. 세계별 수준을 지우면(편향 제거) 움직임 예측은 달력 평균 `cal_abs`보다 못하다(Z2 NONE). 24시간 매시 시작에서는 원점수(Z1b WEAK)와 편향 제거 모두 앞서지만 06:00 시작에서는 뒤진다.
- **WM-2에 준비된 것**: 스키마 4(다정착지까지 합/최대 축약) 하네스와 데이터셋 레지스트리, 원점수·편향 제거 점수의 기본 보고, 검증 선택 달력 기준선(`cal_best`, `cal_best_fair`, `cal_best_anom`)·`apfit_abs`·persistence·관측 ridge, 세계 식별 탐침, 데이터셋 v2용 가족 평가기(복제 분포 채점, 홈 슬롯, 노출 사망).

### 변경된 파일 목록
(커밋별; 모두 `codex/worldmodel-2026-10-wip`)
- `7a14a408` (Part 0) `docs/worldmodel/wm1/GOAL_1_2.md`, `DECISIONS.md`, `RUNLOG.md`
- `b20a991f` (Part 0) `14372ea2` 병합(revitalize Pass 8–10 파일들; `tools/worldmodel/wm_data.py`, `bench_v1.py`, `make_dataset.py`, 테스트, Rust 레코더·프로브 등)
- `b0a12b93` (Part 0) `RUNLOG.md`
- `d1b15ecf` (Part 2) `tools/worldmodel/wm1/golden_v0.py`, `golden_v0.json`, `test_golden_v0.py`
- `45556317` (Part 1) `docs/worldmodel/wm1/p10_pilot_manifest.json`, `tools/worldmodel/wm1/pilot_manifest.py`, `common.py`, `test_registry.py`
- `da379c17` (Part 2) `wm1/data.py`, `evaluate.py`, `climatology.py`, `m1_gru.py`, `m1_tf.py`, `test_pilot_harness.py`, `DECISIONS.md`
- `ef880949` (Part 3) `wm1/m1_tf.py`(`hidden`), `probe_world.py`, `judge_1_2.py`, `test_judge_1_2.py`, `test_probe.py`, `runs/calendar_val_p10_{v16,v15,v16n}_da379c17/`
- `1a7dfdb6` (Part 3) `METRICS_1_2.md`, `RUNLOG.md`
- `096d92d2` (Part 5) `wm1/families.py`, `test_family_eval.py`, `DECISIONS.md`
- `a7d816f8` (Part 5) `runs/calendar_p10_{v16,v15,v16n}_1a7dfdb6/`, `runs/families_smoke_096d92d2/`, `RUNLOG.md`
- `c2465716`, `10624bf9` (Part 7, 리뷰 1) `judge_1_2.py`, `test_judge_1_2.py`, `families.py`, `test_pilot_harness.py`, `golden_v0.*`, `test_golden_v0.py`, `REVIEW.md`, `METRICS_1_2.md`(수정 기록), `WM1_1.md`
- `7cec01fa` (Part 4) `runs/m1tf_p10v16_r3_s0/`, `runs/m1tf_p10v16_r3_s1/`, `RUNLOG.md`
- `57316f63` (Part 4) `runs/m1tf_p10v16_r3_s2/`, `runs/probe_m1tf_p10v16_r3_s{0,1,2}/`, `RUNLOG.md`
- `ad25396d` (Part 6) `runs/m1tf_p10v15_r3_s0/`, `runs/judge_1_2/`, `WM1_2.md`
- `1fabb472` (Part 6) `RUNLOG.md`, `DECISIONS.md`
- `cc913c43` (Part 7) `test_golden_v0.py`(ridge 두 값 상대 1e-12), `DECISIONS.md`
- `643230cf` (Part 7, 리뷰 2) `WM1_2.md`, `REVIEW.md`, `RUNLOG.md`
- 이 보고서와 게이트 기록 커밋(최종 메시지에 해시)

### 측정 결과
**Z 표** (시험, `judge_1_2.py`; 리뷰 2라운드에서 손으로 재계산해 일치)

| # | 비교 | M1b (시드) | 기준 | 판정 |
|---|---|---|---|---|
| Z1 | v16 원점수 T2 168 h 매시 | 0.221 (0.215, 0.223, 0.223) | `an5` 0.206 | **NONE** |
| Z1b | v16 원점수 T2 24 h 매시 | 0.160 (0.159, 0.164, 0.159) | `an5_off` 0.167 (KEEP은 ≤ 0.134) | **WEAK** |
| Z2 | v16 편향 제거 T2 168 h 매시 | 0.223 (0.215, 0.227, 0.228) | `cal_abs` 0.211 | **NONE** |
| Z3 | v15 대조 원점수 T2 168 h 매시 | 0.209 (시드 0) | `an5` 0.138 | **NONE** (예상대로) |
| Z4 | 변수별 skill (아래) | | `cal_best`/`cal_best_anom`, `apfit_abs` | 보고 |
| Z5 | 세계 식별 탐침 (아래) | | 관측 원시 행 ridge | 보고 |

**분할·지평별 원점수와 편향 제거 점수** (시험 16개 세계, 평균 nMAE; `cal_best` 등은 해당 과제의 검증 선택)

| 데이터·과제 | 점수 | M1b s0 | s1 | s2 | M1b 평균 | `cal_best` | `cal_best_fair` | `apfit_abs` | persistence | ridge |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| v16 168 h 매시 | 원점수 | 0.215 | 0.223 | 0.223 | **0.221** | 0.206 (`an5`) | 0.251 (`apfit_year`) | 0.238 | 0.551 | 0.319 |
| v16 168 h 매시 | 편향 제거 | 0.215 | 0.227 | 0.228 | **0.223** | 0.211 (`cal_abs`) | 0.221 (`cal_year`) | 0.232 | 0.557 | 0.288 |
| v16 24 h 매시 | 원점수 | 0.159 | 0.164 | 0.159 | **0.160** | 0.167 (`an5_off`) | 0.169 (`apfit_year`) | 0.167 | 0.202 | 0.220 |
| v16 24 h 매시 | 편향 제거 | 0.162 | 0.167 | 0.161 | **0.163** | 0.169 (`an5_off`) | 0.166 (`apfit_year`) | 0.165 | 0.210 | 0.209 |
| v16 168 h 06:00 | 원점수 | 0.150 | 0.150 | 0.159 | **0.153** | 0.130 (`an5_off`) | 0.190 (`apfit_year`) | 0.176 | 0.434 | 0.327 |
| v16 168 h 06:00 | 편향 제거 | 0.137 | 0.140 | 0.148 | **0.142** | 0.121 (`cal_abs`) | 0.132 (`cal_year`) | 0.145 | 0.440 | 0.227 |
| v16 24 h 06:00 | 원점수 | 0.085 | 0.086 | 0.088 | **0.086** | 0.076 (`an5_off`) | 0.090 (`ap1_year`) | 0.094 | 0.114 | 0.166 |
| v16 24 h 06:00 | 편향 제거 | 0.077 | 0.075 | 0.077 | **0.076** | 0.075 (`an5_off`) | 0.079 (`apfit_year`) | 0.079 | 0.119 | 0.133 |
| v15 대조 168 h 매시 | 원점수 | 0.209 | — | — | **0.209** | 0.138 (`an5`) | 0.193 (`apfit_year`) | 0.150 | 0.648 | 0.256 |
| v15 대조 168 h 매시 | 편향 제거 | 0.224 | — | — | **0.224** | 0.141 (`an5`) | 0.202 (`cal_year`) | 0.152 | 0.679 | 0.239 |
| v15 대조 24 h 매시 | 원점수 | 0.143 | — | — | **0.143** | 0.114 (`an5`) | 0.129 (`apfit_year`) | 0.123 | 0.191 | 0.189 |
| v15 대조 24 h 매시 | 편향 제거 | 0.151 | — | — | **0.151** | 0.117 (`an5`) | 0.131 (`apfit_year`) | 0.126 | 0.209 | 0.180 |

변수별 (v16, 168 h 매시, 시험; M1b는 세 시드 평균):

| 원점수 | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| M1b | 0.253 | 0.207 | 0.000 | 0.000 | 0.080 | 0.334 | 0.471 | 0.421 |
| `cal_best` | 0.220 | 0.181 | 0.000 | 0.000 | 0.156 | 0.262 | 0.416 | 0.417 |
| `apfit_abs` | 0.325 | 0.200 | 0.001 | 0.000 | 0.091 | 0.334 | 0.509 | 0.446 |
| persistence | 0.823 | 1.174 | 0.000 | 0.000 | 0.080 | 1.094 | 0.665 | 0.570 |
| ridge | 0.353 | 0.278 | 0.061 | 0.229 | 0.132 | 0.387 | 0.554 | 0.557 |

| 편향 제거 | food_stock | wood_stock | fire_lit | cold_members | alive | deadwood_total | food_on_bushes_total | adults_working |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| M1b | 0.234 | 0.204 | 0.000 | 0.000 | 0.123 | 0.307 | 0.462 | 0.457 |
| `cal_best` | 0.261 | 0.190 | 0.002 | 0.000 | 0.064 | 0.221 | 0.474 | 0.477 |
| `apfit_abs` | 0.320 | 0.191 | 0.002 | 0.000 | 0.114 | 0.290 | 0.474 | 0.463 |
| persistence | 0.823 | 1.174 | 0.000 | 0.000 | 0.123 | 1.095 | 0.668 | 0.574 |
| ridge | 0.325 | 0.231 | 0.049 | 0.209 | 0.128 | 0.284 | 0.538 | 0.540 |

| 변수 | 원점수 skill vs `cal_best`(an5) | 원점수 skill vs `apfit_abs` | 편향 제거 skill vs `cal_best_anom`(cal_abs) | 편향 제거 skill vs `apfit_abs` |
|---|---:|---:|---:|---:|
| food | -0.151 | 0.221 | 0.104 | 0.270 |
| alive | 0.488 | 0.122 | -0.937 | -0.086 |
| bushes | -0.132 | 0.076 | 0.026 | 0.025 |
| working adults | -0.009 | 0.057 | 0.040 | 0.013 |

| 탐침 (시험, R²; climate는 정확도) | berry_bushes | berry_capacity | climate | deadwood_capacity | deadwood_spots | food_regen_permille | forest_permille | starting_agents |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| m1tf_p10v16_r3_s0 | 0.907 | 0.844 | 0.500 | -0.054 | 0.538 | 0.342 | -0.087 | 0.998 |
| m1tf_p10v16_r3_s1 | 0.881 | 0.799 | 0.562 | -0.029 | 0.415 | 0.343 | -0.069 | 0.998 |
| m1tf_p10v16_r3_s2 | 0.879 | 0.817 | 0.500 | 0.019 | 0.588 | 0.245 | -0.099 | 0.998 |
| 세 시드 평균 | 0.889 | 0.820 | 0.521 | -0.021 | 0.514 | 0.310 | -0.085 | 0.998 |
| 참조: 관측 원시 행 ridge | 0.755 | 0.808 | 0.375 | 0.066 | 0.518 | 0.253 | -0.020 | 0.828 |

- `game_v16_neutral`(기준선만): 168 h 매시 `an5` 0.149(편향 제거 0.153), `apfit_abs` 0.159, `cal_abs` 0.170(0.158), persistence 0.666, ridge 0.264 — v15와 같은 수준.
- T1(다음 시간, 시험): v16 M1b 0.080 / 0.084 / 0.078 vs `an5_off` 0.088, persistence 0.139, ridge 0.110; v15 M1b 0.075 vs `an5_off` 0.070.
- **v15 대조**: Z3 NONE(0.209 vs `an5` 0.138), WM-1.1과 같은 결론.

### 환경
- DGX(GB10, 드라이버 580.126.09, CUDA 13.0): 시작 시 GPU에는 Xorg·gnome-shell뿐, RAM 119 GiB 중 110 GiB 가용, 스왑 16 GiB 중 15.5 GiB 사용(다른 사용자 서비스, 실행 내내 동일). M1b 학습 하나당 GPU 메모리 3.4 GB(시작)→20.9 GB(k = 168), 동시 작업 최대 2개, 다른 사용자의 GPU 점유 없음, RAM 가용 ≥ 77 GiB.
- 작업 시간: 레코더 빌드 74초; 달력 검증 10–11초, 달력 시험 15–16초(데이터셋당); M1b v16 s0 7,256초(max_steps), s1 4,949초(patience), s2 7,190초(max_steps), v15 5,644초(patience); final 각 125–129초(v15 60초); 탐침 각 약 51–60초; 가족 스모크 < 1초; DGX 게이트 약 65초.
- 연결 끊김: 없음.

### 독립 리뷰
- **1라운드**(새 `general-purpose` 서브에이전트, 22:05–22:18): BLOCKING 0. MAJOR 1 — `judge_1_2.py`가 검증/시험 달력 디렉터리에 한 sha만 받아 실제 디렉터리에서 실패 → `--val-sha`/`--test-sha`로 고치고 METRICS에 날짜 있는 수정 기록(모델 시험 수치 이전). MINOR 2 — 스크립트 `sys.path` 누락(수정), 골든이 persistence만 덮음(병합 커밋 코드로 ridge·달력 수치 추가). NOTE 6 — 가족 접두부 검사에 `settlement_ids` 추가, 시작점 테스트를 2,881행으로 등.
- **2라운드**(새 서브에이전트, 01:37–01:49): BLOCKING·MAJOR 0. 사전 등록 순서 확인(21:59:00 커밋 → 21:59:02 푸시 → 21:59:13 DGX fetch → 모든 시험 파일), Z1–Z3·Z4·Z5 독립 재계산·판정 스크립트 재실행 바이트 동일, 입력 65개 숨은 필드 없음, 80/16/16. MINOR 8 — 모두 WM1_2.md 문장(persistence가 `alive` 최선이라는 주장: 실제로는 `an5_off` 0.058이 더 좋음, 식량 수준 우위 과장, "+50–60 %" 일반화, 24 h 우위가 매시 시작에서만 성립, 반올림, (c) 표현, 탐침 서술)과 RUNLOG 시각 1건 → 모두 수정. NOTE 5 — 기록.

### 자율 결정 목록
- F0.1 graft 도구의 `.gitignore`/`.ignore` 변경은 커밋하지 않음(명시 경로만 스테이징).
- F1.1 매니페스트는 `meta.json`에서만 작성, 파일럿 그룹 이름 `train`/`val`/`test`. F1.2 `--dataset`은 이번 실행에 쓰는 스크립트(m1_gru, m1_tf, climatology)에만; 나머지 v0 전용 스크립트는 그대로.
- F2.1 스키마 4의 `bench.deaths`에 노출 사망 포함. F2.2 `EpisodeData.names/schema`, `Variant.obs_state`(v0 기본값 유지). F2.3 `Persistence.rollout` 벡터화(값 동일). F2.4 시작점·split std가 `bench_v1`과 동일함을 확인. F2.5 편향 제거 선택 키 `T2br_*`/`T2anybr_*`. F2.6 관측 ridge = `fit_ridge_without_severity`.
- F4.1 DGX 동시 작업 2개 이하로 순서 배치. F4.2 s0·s1은 `da379c17`, s2·v15는 `1a7dfdb6`에서 학습(학습 코드 차이 없음), 모든 시험은 `1a7dfdb6`. F4.3 persistence보다 나쁜 런 없음; 두 시드는 max_steps에서 종료. F4.4 drop order 적용 불필요(전 항목 수행).
- F5.1 포함률의 spread = 표본 표준편차(ddof 1). F5.2 시드·변형당 예측 효과 하나, 예측 쪽 불은 모델의 V `fire_lit`(다정착지 한계로 기록). F5.3 목표 창 = v1 C_cf 첫 겨울.
- F7.1 골든 ridge 두 값은 맥/DGX BLAS 차이로 상대 1e-12 비교(그 외 비트 동일).

### 실행 기록
(KST, 2026-10-12 → 13)
| Part | 시작 | 종료 | 소요 | 비고 |
|---|---|---|---|---|
| 0 | 21:40:06 | 21:44 | 4분 | 병합, DGX 설정, 빌드 74초 |
| 1 | 21:42 | 21:47:39 | 6분 | 복사 13초, 검증, 매니페스트·레지스트리 |
| 2 | 21:46 | 21:54 | 8분 | 골든, 하네스, 달력 검증, 스모크 |
| 3 | 21:55 | 21:59:00 | 4분 | 탐침·판정 코드, 사전 등록 |
| 3(기준선 시험) | 21:59:13 | 21:59:30 | 17초 | 달력·참조 시험 |
| 4 | 21:54:26 | 01:34:01 | 3시간 40분 | 학습 4개(최대 2개 동시), final 4개, 탐침 3개 |
| 5 | 22:00 | 22:04 | 4분 | 가족 평가기, 스모크 |
| 6 | 01:34 | 01:36 | 2분(+ 리뷰 반영 01:49까지) | 판정, WM1_2.md |
| 7 | 22:05 | 01:5x | 리뷰 1(13분), 리뷰 2(12분), 게이트·보고 | |
- 멈춘 구간: 없음(학습 대기 중에도 세션 유지). 작업 시간 약 4시간 15분(9시간 한도 내).

### 확인된 제한사항
- 80/16/16 세계, 파일럿 한 번의 기록, 개입 없음(T3·T3b·T4 미실행, 모델은 행동 비인식).
- M1b의 GPU 재현성 미측정; 두 v16 시드는 3,000스텝 상한에서 아직 개선 중(수렴 미확인).
- 편향 제거 점수는 평균을 빼므로 치우친 오차에서는 원점수보다 클 수 있음.
- 탐침은 선형이고 시험 16개 세계의 R²에 구간 없음; climate(와 숲 두 시드)는 λ 격자 최댓값을 선택.
- v15 대조는 한 시드라 v16/v15 비율은 3시드 평균 대 1시드 비교.
- 가족 평가기의 실데이터 확인은 단일 정착지 두 가족의 스모크뿐(모델 주장 없음); 예측 쪽 불은 정착지 축약값.
- 출처(커밋) 기록은 `WM1_CODE_COMMIT` 자기 보고와 DGX `stat` 시각·reflog로 확인.

### 게이트 결과
- 맥: `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'` → **114 tests OK, 17 skipped**(전부 레코더 바이너리 미빌드; torch 테스트 스킵 없음).
- DGX(venv, `WM_BIN`/`PROBE_BIN` = `_build/src_wm12` 빌드, `WM_DATA_ROOT`): **114 tests OK, 0 skipped**(`gate_cc913c43`, 약 65초). 이전 `1fabb472`의 사전 점검은 골든 ridge 마지막 자리 차이로 1건 실패 → F7.1로 처리 후 통과. 이후 커밋은 문서만 변경. 보고서 커밋 `0dc77783`에서 다시 실행: 맥 114 OK / 17 skipped, DGX 114 OK / 0 skipped(`gate_0dc77783`) → `codex/worldmodel-2026-10`을 `d5380301..0dc77783`로 fast-forward. 이 기록을 담은 문서 전용 커밋에서도 게이트를 다시 돌리고 녹색 브랜치를 그 커밋으로 옮김(최종 메시지에 해시와 `git ls-remote`).
- PR #512 코멘트: https://github.com/hyunlord/new-world/pull/512#issuecomment-6128723524
- 허용 경로: `git diff --name-only b20a991f..HEAD` 중 `tools/worldmodel/wm1/`, `docs/worldmodel/wm1/` 밖 **0개**.

### 정직한 판단
- **숨은 파라미터가 다른 세계에서 학습 모델은 달력 기준선을 이기는가?** 168시간에서는 아니다(Z1 NONE; 세 시드 모두 `an5` 뒤). 24시간 매시 시작에서는 근소하게 앞선다(Z1b WEAK, KEEP 기준에는 한참 못 미침). 다만 세계 다양성이 달력 계열을 크게 약화시키고 모델은 거의 영향받지 않아, 격차는 v15의 0.071에서 0.014로 줄었다. 검증에서는 모델이 `an5`보다 앞섰고 시험에서는 뒤졌으므로, 16개 세계로는 0.01–0.02 차이의 부호가 안정적이지 않다.
- **그 차이는 세계를 알아보는 것인가, 앞으로를 예측하는 것인가?** 주로 알아보는 것이다. 은닉 상태가 인구·덤불 파라미터를 거의 복원하고(Z5), 168시간 `alive`는 persistence와 똑같다. 세계별 수준을 지우면 움직임 예측은 달력 평균보다 못하다(Z2 NONE). 짧은 지평(24시간, 바쁜 시간대)에만 약한 역학 이득이 보인다.
- **데이터셋 v2와 WM-2는 무엇을 기본으로 보고해야 하는가?** 원점수와 편향 제거 점수를 나란히(매시 시작 기본, 24·168시간, 변수별), 검증으로 고른 `cal_best`·`cal_best_anom`·`cal_best_fair`·`apfit_abs`·persistence·관측 ridge를 같은 하네스·같은 split std로; 모델 주장에는 세계 식별 탐침(원시 행 참조와 함께)과 세계 단위 변동(부트스트랩 등)을 붙인다. 80개 세계로 충분하다는 증거는 없다(두 시드가 스텝 상한에서 아직 개선 중) — v2는 더 많은 세계와 묶이지 않는 스텝 상한이 필요하다. 개입 효과는 `families.py`로 복제 분포에 대해 채점한다.
