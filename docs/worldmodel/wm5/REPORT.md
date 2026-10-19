## 구현 완료 보고

### 구현 의도
WM-2 … WM-4는 자기회귀 세계 모형이 신의 개입을 무시하고, 짝 갈래 학습은 첫 주 효과만 가르치며, 교란된 관측 분할에서는 어떤 학습 모델도 "효과 없음"(`zero`)을 이기지 못하고, 이 분야의 방법(CRN, CT)은 두 분할 모두에서 최하위라는 것을 보였다. 논문 심사자가 제기할 세 가지 반론에 WM-5가 답한다. (1) "CRN·CT를 제대로 학습시키지 않았다" — CT의 유지 체크포인트는 원저자 옵티마이저 스텝의 4 %, α_max의 20 %였고, 개수 반올림이 없었으며, 마지막이 아닌 최적 검증 체크포인트였다. (2) "관측 분할이 너무 작아 균형화가 의미 없었다" — 세계당 1회 투입, 학습 세계 400개, IPW 유효 표본 44–55. (3) "아무것도 zero를 못 이긴다면 벤치마크가 판별력이 있는가?" WM-5는 CRN·CT를 원저자 방식(계획 스텝, 실제 진행률에 따른 α, 마지막 체크포인트, 판독 시 반올림)으로 다시 학습하고, 세 배 크기의 순차 다중 투입 관측 분할 obs2를 녹화하며, 바닥 ← 특권 회귀 ← 관측 회귀 ← `zero`의 여유 사다리를 재고, 벤치마크 논문 초안을 쓴다.

### 구현 내용
(Part별 파일과 핵심 구조)
- **Part 0** `docs/worldmodel/wm5/GOAL.md`(원문 그대로), `DECISIONS.md`, `RUNLOG.md`: 시작 시각, 기계, 녹화기·WM-2/3 체크포인트 sha256(기록값과 일치), WM-4 체크포인트 sha256 최초 기록, 시드 미사용 확인, 46시간 마감.
- **Part 1** `tools/worldmodel/wm5/policy2.py`(순차 다중 투입 정책: ×2 확률, 불응기, 최대 5회, 별도 난수열, 라운드 재생, 기준 행·가족 일정), `obs2.py`(라운드 일정·접두 검사·finalize 재생·가족 일정·가족 검사·관측 전용/전체 매니페스트), `dataset_doc.py`, `launch/record_obs2.sh`, `launch/obs2_pipeline.sh`; `docs/worldmodel/wm5/obs2_manifest.json`(7,080 에피소드), `DATASET_OBS2.md`, `runs/record/`.
- **Part 2** `effects5.py`(obs2 사례: 두 기준 행 × 복제의 rep 부호, 효과 바닥, 다중 투입 naive·ipw·match5, `ridge_obs`·`ridge_priv`, obs2의 M1 예측), `causal5.py`(CRN·CT: 계획 스텝, 진행률 α, 마지막+최적 체크포인트, 비용 점검, 반올림 판독), `m1_obs2.py`(`obs2_v19` 등록, M1b-obs2), `test_wm5.py`(29개 테스트), `METHODS.md`(D16–D24).
- **Part 3** `METRICS.md` part a(정책·역할·시드·기준 행, 라운드 1 전), part b(실행·예산·스텝 계획·체크포인트·사례·척도·바닥·기준선·회귀·규칙, 테스트 수치 전).
- **Part 4** `launch/queue.sh`(단일 학습 큐), `finals5.sh`·`finals_watch5.sh`(사전 등록 이후 1회 평가, CUDA 재시도), `fetch5.sh`, `gate.sh`; `docs/worldmodel/wm5/runs/<run>/`(작은 파일만), `runs/rescored_*`(D2), `runs/_aborted/`(오염된 비용 점검, 공개).
- **Part 5** `judge5.py` → `runs/judge5/judge.md`, `judge.json`(규칙, 사다리, 교란, 주 결과표, 비용, 계산, 사후 블록, 셀별 표).
- **Part 6** `WM5.md`; 논문 초안 `docs/worldmodel/paper/`(`paper_src.md` → `paper.md`, `paper_numbers.json` 463개 숫자, 부록 A–F, `figures/` 5개 그림 PNG·PDF, `references.bib` 28개, `REFERENCES_CHECK.md`), `figures.py`; `docs/worldmodel/wm4/WM4.md` 맨 위 한 줄.
- **Part 7** `REVIEW.md`(독립 리뷰 2회와 응답), 게이트, 이 보고서.

### 구현 방법
- **순차 다중 투입 정책과 녹화 라운드.** `policy2.py`: WM-4의 고정 삼분위(METRICS에서 읽음, 재계산 안 함), 일일 확률 ×2(선물 0.060/0.016/0.004, 부패 0.004/0.016/0.060, 썩힘 0.024/0.008), 별도 난수열 `default_rng([seed, day, 2])`, 투입 뒤 7일 불응기(다음 결정은 d + 8일 이후), 최대 5회. 정책은 이전 투입으로 이미 바뀐 궤적을 읽는다: 라운드 0(사실 1,640) → 라운드 k는 라운드 k − 1에 투입한 세계를 그때까지의 모든 투입 일정으로 다시 녹화하고, 마지막 투입 다음 행부터 정책을 계속 읽는다(라운드 1–5: 1,587 / 1,318 / 757 / 265 / 39, 라운드 6에서 새 투입 없음). 라운드마다 새 에피소드가 새 투입 행까지 이전 라운드와 같은지(접두 검사)와 모든 이전 투입을 담는지 확인. 최종 관측 에피소드에 정책을 재생하면 모든 라운드의 투입과 행별 확률이 재현됨(1,640 / 1,640) → 모든 결정 행의 성향이 기록됨(`policy.json`).
- **두 기준 행과 `pre` 일정.** val·test 세계마다 c1(첫 투입 행, 없으면 30일 06:00)과 c2(두 번째 투입 행, 없으면 c1 + 14일). 기준 행 c마다 `pre` = c 이전 투입들, 갈래 = `pre` + c에서의 투입(c 이후 없음), c에서 분기(녹화기 호출 1회/(세계, c)). `pre`가 비면 녹화기가 늘 함께 만드는 `none` 구성원이 곧 `pre`.
- **노이즈 복제와 효과 바닥.** val 가족은 replicate 1개, test 가족은 2개(r0는 무작용, r1은 분기 노이즈). 바닥 = 각 사례를 다른 복제의 참 효과로 예측(obs2: r0 사례를 r1 쌍둥이로; v2: 가족의 다른 복제). 같은 셀·척도로 채점; 연속 대칭 노이즈에서는 노이즈 한계 오차의 ≈ √2배, 희소 개수 셀에서는 `zero`보다 나쁠 수 있어 "천장"이 아니라 기준점.
- **다중 투입 IPW.** 위험 집합 = p_tot(c) > 0인 결정 행. 처치: c에서 a를 투입하고 (c, c + h)에 다른 투입 없음, 가중치 1 / (p_a(c) · ∏_{(c, c+h)}(1 − p_tot)); 대조: [c, c + h)에 투입 없음, 가중치 1 / ∏_{[c, c+h)}(1 − p_tot); 불응기 행은 인자 1. Hájek 정규화. 합성 다중 투입 교란 정책 단위 테스트: ipw 불편, naive 편향, 투입 뒤 생존 인자를 빼면 편향(참 +3.0에 ipw 3.07, 생존 인자 없으면 1.54).
- **직접 효과 회귀(관측·특권).** 셀(갈래 × 결과 × 지평)마다 릿지, 목표 = 참 효과 / 셀 척도, 특징 = 37개 상태(c 시점, 24시간·168시간 평균) + 달력 30 + (obs2) 이전 투입 수·마지막 투입 후 일수, v2 학습 표준편차로 정규화, λ ∈ {0.01 … 1,000}은 v2 val 가족에서 셀마다 선택, 닫힌 해(numpy). 학습 데이터 = v2 학습 가족(양 복제, 모든 사례). `ridge_priv`는 숨은 필드(전역 3, latent 7, 정착지 3×합·최대, world_variety 11)를 더한 진단용(순위에 안 들어감).
- **CRN·CT의 계획 스텝·α 일정·마지막 체크포인트·반올림.** `causal5.py`가 `wm4.causal`을 감싸 학습 루프만 교체: 단계마다 원저자 옵티마이저 스텝(CRN 15,700 + 53,800, CT 11,850)을 계획으로 학습하고, 실제 데이터 300초 비용 점검이 예산 초과를 예측할 때만 줄임(한 번도 줄지 않음); 안전망 1.25 × 예산. α = `alpha_at(step / planned, α_max)`를 매 스텝 설정 → 완주하면 α = 0.99991 α_max. 주 체크포인트 = 마지막 스텝(CT는 EMA 가중치, CRN은 마지막 인코더+디코더), 보조 = 최저 `val_roll`. 판독 시 M1 규칙대로 식량·생존자·누적 사망을 반올림(되먹임 안에서는 안 함), 비반올림은 보조로 기록. WM-4의 선택 구성(CRN 3, CT 2) 재사용.
- **사전 등록.** METRICS part a(`2a97b4cd`, 21:24:24) → 라운드 1 녹화(22:08:43); part b(`6e9e88ed`, 21:39:47) → 모든 테스트 산출물(첫 21:40:45). 모든 평가는 1회(덮어쓰기 거부); obs2 평가는 매니페스트를 담은 이후 커밋 `f7cf620a`에서(규칙 변경 없음, DECISIONS L4.3).
- **논문 숫자 추적과 인용 검증.** `paper_src.md`의 모든 결과 숫자는 `{{j5:경로#형식}}` 자리표시자로 쓰고, `figures.py render`가 커밋된 judge 파일에서 값을 읽어 `paper.md`를 만들며 `paper_numbers.json`(숫자별 원본 파일·키·형식)을 기록; 단위 테스트가 모든 숫자를 원본과 대조. 참고문헌 28개는 하위 에이전트가 권위 있는 페이지(학회 논문집, OpenReview, DOI의 Crossref 기록)에서 확인(`REFERENCES_CHECK.md`); 확인 못 한 Hájek(1971)은 인용하지 않음.

### 기능 설명
연구 관점에서 확인된 것:
- **원저자 방식으로 학습해도 CRN·CT는 "효과 없음"을 이기지 못한다(Q1).** 원저자 방식 = CT 저자의 공개 구현(CRN 포함)의 학습 일정. 여덟 개 주 실행 모두 그 구현의 옵티마이저 스텝 100 %, α = 0.9999 α_max, 마지막 체크포인트, 반올림 판독(R4 충족). obs2·v2, 7일·30일의 R1–R3 비교 12개가 모두 NONE. v2에서는 WM-4의 시간 제한 실행보다 오히려 더 나쁘다(사후). 실패 양상은 그대로: obs2에서 교란 방향으로 과반응(CRN 선물 7일 식량 +78.8 vs 참 +21.7), v2에서 두 롤아웃이 사망 수에서 정수 단위로 벌어짐(30일 사망 셀, 시드 0 / 1: CRN 14.5 / 10.2, CT 5.0 / 5.5 vs zero 0.58).
- **반올림만으로는 거의 아무것도 바뀌지 않는다(Q4, D2).** WM-4 CRN·CT 재채점의 차이는 −0.006 … +0.018, 모든 구간이 0을 포함.
- **관측 데이터를 늘려도 거의 도움이 안 된다(Q3, D1).** M1b-obs2는 WM-4의 M1b-obs보다 아주 조금 낫고(두 지평 모두 WEAK, 점수의 ≈ 1.5–2 %; 다만 M1b-obs2는 8,000스텝 상한에서 수렴하지 못함 — 두 시드 모두 유지 스텝 7,950, 검증 손실 하락 중), CT-obs2는 7일 TIE, 30일 NONE(더 나쁨; 데이터 크기와 학습 일정·체크포인트가 섞인 비교). 균형화에 쓸 데이터가 부족했던 것이 원인이 아니다(유효 표본 396–445 vs WM-4 44–55).
- **벤치마크는 판별력이 있다(Q2, F1–F2).** 노이즈 바닥이 노이즈 복제가 있는 두 분할(v2, obs2)·두 지평 모두에서 `zero`보다 훨씬 낮다(v2 7일 0.187 vs 0.612). v2 7일에서 관측 상태의 선형 회귀는 `zero`를 이기지만(E1 WEAK) 갈래 평균 효과 수준에 그치고(사후: `arm_mean`·`an5_arm`과 TIE), 숨은 상태가 세계별 정보를 조금 더한다(사후). obs2에서는 v2로 학습한 회귀가 전이되지 않는다(투입 행 지지 영역 밖).
- **교란은 실재하고 측정된다.** obs2 7일에서 다중 투입 IPW가 naive의 초과 오차를 대부분 걷어내고(−0.220) 세 행위 중 둘의 평균 효과를 naive보다 훨씬 잘 복원하지만(선물 +45.6 vs 참 +21.7, naive +128.2; 부패 −76.7 vs −78.0; 썩힘은 +4.1 vs −4.8로 부호가 틀림 — 추정은 학습 세계 합동, 참값은 테스트 사례 평균이라 모집단이 다름), 세계별 오차에서는 `zero`를 못 이긴다.

논문이 주장할 수 있는 것: (i) 하나의 고정 시뮬레이터에서 세 분할(개입·단일 투입 관측·순차 다중 투입 관측, 성향 기록)과 노이즈 바닥을 갖춘, 판별력 있는 개입 효과 벤치마크; (ii) 두 시간 가변 처치 효과 방법(CRN, CT)이 원저자 학습 일정·마지막 체크포인트·반올림으로도 두 분할에서 "효과 없음"을 이기지 못한다; (iii) 반올림은 그 원인이 아니다; (iv) 관측 상태의 선형 회귀는 v2에서 갈래 평균까지만 도달(사후). 30일 부호 반전은 사후 기술로만: 부패에서는 어떤 학습 동역학 모델도 재현하지 못하고(CT만 부호가 맞고 크기가 작음), 선물에서는 시드마다 갈림(CRN-obs2 s1 −15.9 vs 참 −17.3, s0 +12.2) — 평균이 맞아도 세계별 예측은 `zero`보다 나쁨(0.822 vs 0.279); 반전의 원인은 분석하지 않음.
주장할 수 없는 것: 다른 시뮬레이터·장면·개입으로의 일반화; 원저자의 다른 하이퍼파라미터(합성 벤치마크 기본값이 아닌, obs에서 고른 구성 하나만 사용)나 시간 해상도에서의 결과; 바닥이 정확한 천장이라는 것; obs2에서 관측 vs 숨은 상태의 몫(회귀가 전이되지 않음); 30일 반전의 메커니즘; "더 오래 학습해서 나빠졌다"는 인과적 주장(사후 관찰).

### 변경된 파일 목록
(커밋별)
- **Part 0**: `30d8205b` GOAL.md (unchanged); `b7f22da6` DECISIONS.md, RUNLOG.md: start time, machines, input sha256 (recorder, WM-2/3 checkpoints; WM-4 checkpoints recorded), seeds unused, 46-hour deadline
- **Part 1**: `a15e1549` wm5 package, launch scripts (code.sh, launch.sh, record_obs2.sh round 0), obs2.py verify; `2a97b4cd` METRICS.md part a (obs2 policy, roles, seeds, reference rows; before round 1); policy2.py (sequential multi-cast policy), obs2.py (rounds, prefix checks, replay, families, manifest), record_obs2.sh stages, policy tests; DECISIONS L1.1-L1.3; `e875e413` obs2_pipeline.sh (rounds until none adds a cast, finalize, families, unforked check, manifest), obs2.py unforked-seeds; run log; `227f6557` run log; `e4d9bdff` run log (round 1 verified); `2051a1e6` obs2 rounds 0-6 recorded and verified (prefix checks, replay = rounds); record checks; run log; `b25cf578` dataset_doc.py (DATASET_OBS2.md from the record files and describe.json); `c3fe12ee` obs2.py manifest --observed-only (the training part of the manifest before the families finish); `835a8670` obs2_manifest.json, observed episodes (1,640; the families are appended later); DECISIONS L1.4; `043ae6ed` DATASET_OBS2.md (rounds, acts per world, confounding, ESS, val effect distribution); describe; run log; `da300d9c` fam-check compares the recorder's extra none member with the round-0 factual (not the observed episode); DECISIONS L1.5; `f7cf620a` full obs2_manifest.json (7,080 episodes: observed + families), fam-check 880/880; DATASET_OBS2; run log (SSH stall 08:18-10:27)
- **Part 2**: `38bea0c5` causal5.py (CRN / CT on the authors' step plan: alpha on real progress, last + best checkpoints, cost check, rounded readout), m1_obs2.py (obs2_v19 registration, M1b-obs2), training queue; tests; `18871a7c` effects5.py (obs2 cases with two references and replicates, effect floors, multi-cast IPW / naive / match5, direct effect regressions ridge_obs / ridge_priv, M1 on obs2); tests (IPW unbiased vs naive and vs no survival factor, weights by hand, floor, ridge recovery and leakage)
- **Part 3**: `6e9e88ed` METRICS.md part b (runs, budgets, step plan, primary / secondary checkpoints, cases, scales, floors, baselines, regressions, rules R1-R4, F1-F2, E1-E2, D1-D2, S) before any test-role number
- **Part 4**: `679d4036` finals5.sh (test evaluations once from the pre-registration commit; CUDA-context retries); `d5117f73` D2 re-scoring of WM-4's CRN / CT runs with rounding (from the pre-registration commit); DECISIONS L4.1-L4.2; `847804aa` finals_watch5.sh (test evaluations as trainings end); `c34e617c` crn_v2_s0 (authors' full plan, 2,618 s) and its v2 test effects; cost checks crn_v2 / ct_v2; fetch5.sh; `76f6af3a` ct_v2_s0 (authors' full plan, 13,114 s) and its v2 test effects; run log; `b8269ac6` run log (obs2 queue); `e40368ec` crn_v2_s1 and its v2 test effects; paper datasheet appendix; run log; `cc148a06` ct_v2_s1 and its v2 test effects; run log; `fb239081` DECISIONS L0.6 (pause, deadline), L4.3 (obs2 evaluations from the manifest commit); `65cb6aca` obs2_base (cases, baselines, val selection, floor), ridge_obs, ridge_priv, m1b_obs2_s0; DATASET_OBS2 test distribution; run log; `57bfdadf` DECISIONS L4.4 (CT-obs2 cost check repeated without our own concurrent evaluations); aborted run files; run log; `f45e06df` DECISIONS L4.4 second attempt; `5e17666d` obs2 references, m1b_obs2_s0 effects and factual T2, crn_obs2_s0 and effects, clean CT-obs2 cost check (full plan); DECISIONS L4.4; run log; `6dfc605c` ct_obs2_s0 (authors' full plan) and its obs2 test effects; WM5.md Q2 and caveats; run log; `b9a21412` crn_obs2_s1 and its obs2 test effects; run log; `73f93a42` ct_obs2_s1 and its obs2 test effects; run log (Mac sleep 17:58-19:40); DECISIONS L0.7
- **Part 5**: `c755b65e` judge5.py (R1-R4, E1-E2, D1-D2, headroom ladder F1-F2, confounding, per-cell tables); METHODS.md D16-D24; DECISIONS L3.1; `4d051e14` judge5 main_table (seed-averaged scores per method group on v2, obs, obs2)
- **Part 5/6**: `3d27cc07` final judge5 (judge.md / judge.json), every run's small files, WM5.md, DATASET_OBS2, paper.md rendered (432 traced numbers) with figures, WM4.md pointer line; DECISIONS L6.1-L6.2
- **Part 6**: `fa6b1878` figures.py (paper rendering with traced numbers, figures), paper source skeleton (benchmark, methods, policies, limitations); tests; `118fcc91` references.bib (28 entries verified on the web), REFERENCES_CHECK.md; paper appendix A; `7a3db601` paper results tables and appendix B as traced placeholders; figures.py str format; `bcdc3c59` judge5 cost_checks block; paper obs2 counts and appendix B cost numbers as traced placeholders; WM5.md draft; `73e8f67f` paper analysis section (headroom, confounding, failure modes, 30-day reversal) as traced placeholders; `cc3f744c` judge5 compute block; paper appendix E; `cdf79f6b` training-curve figure with the obs2 runs (review N5); `8ead8ddf` paper abstract, introduction findings, results text (traced placeholders); reversal claim corrected for CT-obs2; `8cb92b35` paper reversal sentence with both CT seeds; `cb038a16` paper wording (headroom reference, scope of NONE, post hoc labels, table notes); WM5.md Q1, Q3, what changed; `c669e8c6` paper note on the W1 single-run cap
- **Part 7**: `c5ecf7b7` review round 1 (no BLOCKING; 2 MAJOR, 5 MINOR, NOTEs) and responses: judge post hoc block (ridge vs arm-level baselines, v2 vs obs2 support), paper wording (observable rung = arm mean; obs2 ridge = transfer; floor not a ceiling), DECISIONS L2.1-L2.5 and L7.1, missing effects file, ESS 126; `add525cc` gate.sh (DGX half of the gate, wm5 paths); `09b63eff` run log (last training, final evaluations, judge, paper render); `7405a470` review round 2 and responses: reversal claim post hoc and out of the abstract, mechanism hedged, scope and seed labels, M1b-obs2 non-convergence disclosed, Q3 "barely", CRN attribution; verdict table, R4 and D2 to paper appendix F; judge.md renders the post hoc block and labels obs2 transfer rows; DECISIONS L7.2
- **Part 7 (최종)**: 이 보고서(REPORT.md)와 게이트 결과·PR 댓글 URL을 담은 마지막 커밋 — 해시는 최종 메시지에 기록(자기 해시를 자기 안에 쓸 수 없음).

### 측정 결과
**사전 등록 규칙 표 (R1–R4, F1–F2, E1–E2, D1–D2)** — 12셀 척도 효과 MAE(낮을수록 좋음), Δ = 모델 − 기준 [세계 단위 짝 부트스트랩 95 % CI, B = 2,000]:

| 규칙 | 모델 (시드별) | 기준 | 비율 | Δ [95 % CI] | 판정 |
|---|---|---|---|---|---|
| R1 obs2 7일: `crn_obs2` | 0.402 (0.403, 0.401) | `zero` 0.217 | 1.851 | +0.185 [+0.170, +0.200] | **NONE** |
| R1 obs2 7일: `ct_obs2` | 0.311 (0.304, 0.317) | `zero` 0.217 | 1.430 | +0.093 [+0.077, +0.110] | **NONE** |
| R1 obs2 7일: `m1b_obs2` | 0.216 (0.215, 0.218) | `zero` 0.217 | 0.996 | −0.001 [−0.003, +0.001] | **TIE** |
| R2 obs2 30일: `crn_obs2` | 0.780 (0.770, 0.790) | `zero` 0.188 | 4.138 | +0.591 [+0.548, +0.639] | **NONE** |
| R2 obs2 30일: `ct_obs2` | 0.483 (0.502, 0.465) | `zero` 0.188 | 2.565 | +0.295 [+0.264, +0.328] | **NONE** |
| R2 obs2 30일: `m1b_obs2` | 0.194 (0.194, 0.194) | `zero` 0.188 | 1.027 | +0.005 [+0.004, +0.007] | **NONE** |
| R3 v2 30일: `crn_v2` vs `an5_arm` | 3.645 (4.187, 3.104) | `an5_arm` 0.594 | 6.133 | +3.051 [+2.865, +3.253] | **NONE** |
| R3 v2 7일: `crn_v2` vs `an5_arm` | 1.649 (1.795, 1.504) | `an5_arm` 0.485 | 3.400 | +1.164 [+1.063, +1.266] | **NONE** |
| R3 v2 30일: `crn_v2` vs `zero` | 3.645 (4.187, 3.104) | `zero` 0.487 | 7.485 | +3.158 [+2.967, +3.364] | **NONE** |
| R3 v2 7일: `crn_v2` vs `zero` | 1.649 (1.795, 1.504) | `zero` 0.612 | 2.694 | +1.037 [+0.932, +1.142] | **NONE** |
| R3 v2 30일: `ct_v2` vs `an5_arm` | 1.920 (1.950, 1.889) | `an5_arm` 0.594 | 3.229 | +1.325 [+1.194, +1.457] | **NONE** |
| R3 v2 7일: `ct_v2` vs `an5_arm` | 1.243 (1.118, 1.368) | `an5_arm` 0.485 | 2.562 | +0.758 [+0.662, +0.854] | **NONE** |
| R3 v2 30일: `ct_v2` vs `zero` | 1.920 (1.950, 1.889) | `zero` 0.487 | 3.942 | +1.433 [+1.299, +1.565] | **NONE** |
| R3 v2 7일: `ct_v2` vs `zero` | 1.243 (1.118, 1.368) | `zero` 0.612 | 2.031 | +0.631 [+0.530, +0.733] | **NONE** |
| E1 v2 30일: `ridge_obs` vs `zero` | 0.602 (0.602) | `zero` 0.487 | 1.235 | +0.115 [+0.073, +0.155] | **NONE** |
| E1 v2 7일: `ridge_obs` vs `zero` | 0.480 (0.480) | `zero` 0.612 | 0.783 | −0.133 [−0.178, −0.088] | **WEAK** |
| E1 obs2 30일: `ridge_obs`(v2→obs2 전이) vs `zero` | 0.514 (0.514) | `zero` 0.188 | 2.730 | +0.326 [+0.309, +0.342] | **NONE** |
| E1 obs2 7일: `ridge_obs`(v2→obs2 전이) vs `zero` | 0.537 (0.537) | `zero` 0.217 | 2.470 | +0.319 [+0.302, +0.335] | **NONE** |
| E2 v2 30일: `crn_v2` vs `ridge_obs` | 3.645 (4.187, 3.104) | `ridge_obs` 0.602 | 6.059 | +3.044 [+2.859, +3.244] | **NONE** |
| E2 v2 7일: `crn_v2` vs `ridge_obs` | 1.649 (1.795, 1.504) | `ridge_obs` 0.480 | 3.439 | +1.170 [+1.068, +1.273] | **NONE** |
| E2 v2 30일: `ct_v2` vs `ridge_obs` | 1.920 (1.950, 1.889) | `ridge_obs` 0.602 | 3.190 | +1.318 [+1.191, +1.449] | **NONE** |
| E2 v2 7일: `ct_v2` vs `ridge_obs` | 1.243 (1.118, 1.368) | `ridge_obs` 0.480 | 2.592 | +0.763 [+0.666, +0.863] | **NONE** |
| E2 v2 30일: `wm3_full` vs `ridge_obs` | 0.511 (0.517, 0.504, 0.512) | `ridge_obs` 0.602 | 0.849 | −0.091 [−0.133, −0.046] | **WEAK** |
| E2 v2 7일: `wm3_full` vs `ridge_obs` | 0.496 (0.532, 0.472, 0.484) | `ridge_obs` 0.480 | 1.034 | +0.016 [−0.035, +0.081] | **TIE** |
| E2 obs2 30일: `crn_obs2` vs `ridge_obs`(v2→obs2 전이) | 0.780 (0.770, 0.790) | `ridge_obs` 0.514 | 1.516 | +0.265 [+0.222, +0.313] | **NONE** |
| E2 obs2 7일: `crn_obs2` vs `ridge_obs`(v2→obs2 전이) | 0.402 (0.403, 0.401) | `ridge_obs` 0.537 | 0.749 | −0.135 [−0.154, −0.117] | **KEEP** |
| E2 obs2 30일: `ct_obs2` vs `ridge_obs`(v2→obs2 전이) | 0.483 (0.502, 0.465) | `ridge_obs` 0.514 | 0.940 | −0.031 [−0.062, +0.001] | **TIE** |
| E2 obs2 7일: `ct_obs2` vs `ridge_obs`(v2→obs2 전이) | 0.311 (0.304, 0.317) | `ridge_obs` 0.537 | 0.579 | −0.226 [−0.244, −0.208] | **KEEP** |
| E2 obs2 30일: `m1b_obs2` vs `ridge_obs`(v2→obs2 전이) | 0.194 (0.194, 0.194) | `ridge_obs` 0.514 | 0.376 | −0.321 [−0.337, −0.304] | **KEEP** |
| E2 obs2 7일: `m1b_obs2` vs `ridge_obs`(v2→obs2 전이) | 0.216 (0.215, 0.218) | `ridge_obs` 0.537 | 0.403 | −0.320 [−0.336, −0.303] | **KEEP** |
| E2 obs2 30일: `wm3_full` vs `ridge_obs`(v2→obs2 전이) | 0.200 (0.204, 0.196, 0.199) | `ridge_obs` 0.514 | 0.388 | −0.315 [−0.332, −0.298] | **KEEP** |
| E2 obs2 7일: `wm3_full` vs `ridge_obs`(v2→obs2 전이) | 0.239 (0.218, 0.257, 0.242) | `ridge_obs` 0.537 | 0.445 | −0.298 [−0.314, −0.282] | **KEEP** |
| D1 obs2 30일: ct_obs2_vs_ct_obs_as_kept | 0.483 (0.502, 0.465) | `ct_obs (WM-4, as_kept)` 0.378 | 1.278 | +0.105 [+0.076, +0.135] | **NONE** |
| D1 obs2 7일: ct_obs2_vs_ct_obs_as_kept | 0.311 (0.304, 0.317) | `ct_obs (WM-4, as_kept)` 0.317 | 0.980 | −0.006 [−0.021, +0.009] | **TIE** |
| D1 obs2 30일: ct_obs2_vs_ct_obs_rounded | 0.483 (0.502, 0.465) | `ct_obs (WM-4, rounded)` 0.378 | 1.280 | +0.106 [+0.076, +0.137] | **NONE** |
| D1 obs2 7일: ct_obs2_vs_ct_obs_rounded | 0.311 (0.304, 0.317) | `ct_obs (WM-4, rounded)` 0.323 | 0.963 | −0.012 [−0.026, +0.004] | **TIE** |
| D1 obs2 30일: m1b_obs2_vs_m1b_obs | 0.194 (0.194, 0.194) | `m1b_obs (WM-4)` 0.197 | 0.981 | −0.004 [−0.006, −0.001] | **WEAK** |
| D1 obs2 7일: m1b_obs2_vs_m1b_obs | 0.216 (0.215, 0.218) | `m1b_obs (WM-4)` 0.219 | 0.986 | −0.003 [−0.005, −0.001] | **WEAK** |

**R4 (원저자 방식 학습 도달)** — 모든 주 실행이 계획 스텝 100 %, 마지막 스텝 α = 0.9999 α_max, 계획 = 원저자 스텝, 정지 이유 planned, 안전망 미도달:

| 실행 | 계획 스텝 | 도달 | α/α_max | 판정 |
|---|---|---|---|---|
| `crn_obs2_s0` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | 0.99991 | trained as the authors |
| `crn_obs2_s1` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | 0.99991 | trained as the authors |
| `crn_v2_s0` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | 0.99991 | trained as the authors |
| `crn_v2_s1` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | 0.99991 | trained as the authors |
| `ct_obs2_s0` | {'ct': 11850} | {'ct': 11850} | 0.99991 | trained as the authors |
| `ct_obs2_s1` | {'ct': 11850} | {'ct': 11850} | 0.99991 | trained as the authors |
| `ct_v2_s0` | {'ct': 11850} | {'ct': 11850} | 0.99991 | trained as the authors |
| `ct_v2_s1` | {'ct': 11850} | {'ct': 11850} | 0.99991 | trained as the authors |

**D2 (반올림만의 효과)** — WM-4 CRN/CT를 자기 테스트 분할에서 반올림으로 재채점(시드 평균, 반올림 − WM-4 커밋값):

| 실행 | 지평 | 반올림 | WM-4 | Δ [95 % CI] |
|---|---|---|---|---|
| `crn_obs` | 30d | 0.860 | 0.863 | −0.003 [−0.008, +0.002] |
| `crn_obs` | 7d | 0.498 | 0.497 | +0.001 [−0.007, +0.010] |
| `crn_v2` | 30d | 1.543 | 1.547 | −0.004 [−0.029, +0.021] |
| `crn_v2` | 7d | 1.476 | 1.474 | +0.003 [−0.085, +0.099] |
| `ct_obs` | 30d | 0.369 | 0.373 | −0.004 [−0.010, +0.003] |
| `ct_obs` | 7d | 0.315 | 0.321 | −0.006 [−0.014, +0.003] |
| `ct_v2` | 30d | 1.390 | 1.374 | +0.016 [−0.013, +0.043] |
| `ct_v2` | 7d | 1.025 | 1.007 | +0.018 [−0.060, +0.098] |

**여유 사다리 (F1 v2, F2 obs2)** — 바닥 ← ridge_priv ← ridge_obs ← zero:

| 분할 | 지평 | 바닥 | ridge_priv(진단) | ridge_obs | zero | zero − 바닥 | zero − ridge_obs | ridge_obs − ridge_priv | ridge_priv − 바닥 |
|---|---|---|---|---|---|---|---|---|---|
| v2 | 30d | 0.267 | 0.590 | 0.602 | 0.487 | +0.220 [+0.114, +0.327] | −0.115 [−0.155, −0.073] | +0.012 [−0.005, +0.031] | +0.323 [+0.227, +0.411] |
| v2 | 7d | 0.187 | 0.434 | 0.480 | 0.612 | +0.425 [+0.374, +0.483] | +0.133 [+0.088, +0.178] | +0.045 [+0.028, +0.063] | +0.247 [+0.213, +0.282] |
| obs2 | 30d | 0.095 | 0.497 | 0.514 | 0.188 | +0.094 [+0.070, +0.116] | −0.326 [−0.342, −0.309] | +0.017 [−0.011, +0.043] | +0.402 [+0.370, +0.436] |
| obs2 | 7d | 0.058 | 0.391 | 0.537 | 0.217 | +0.159 [+0.144, +0.175] | −0.319 [−0.335, −0.302] | +0.146 [+0.131, +0.159] | +0.333 [+0.318, +0.349] |

사후(review M1): v2에서 ridge vs 갈래 수준 기준선 — ridge_obs vs an5_arm 30d +0.007 [−0.021, +0.038] (TIE); ridge_obs vs an5_arm 7d −0.006 [−0.037, +0.025] (TIE); ridge_obs vs arm_mean 30d −0.017 [−0.035, +0.001] (TIE); ridge_obs vs arm_mean 7d −0.017 [−0.037, +0.002] (TIE); ridge_priv vs an5_arm 30d −0.004 [−0.041, +0.030] (TIE); ridge_priv vs an5_arm 7d −0.051 [−0.080, −0.022] (WEAK); ridge_priv vs arm_mean 30d −0.029 [−0.053, −0.006] (WEAK); ridge_priv vs arm_mean 7d −0.063 [−0.082, −0.043] (WEAK).
사후(review M2): v2 학습 사례의 투입 행 — 선물·썩힘 [490], 부패 11개 행; obs2 테스트는 61개 행; v2 학습 행에 놓인 obs2 사례 비율 선물 0.0%, 썩힘 0.0%, 부패 20.4%.

**obs2의 교란·투입 통계**:

- 세계 1,640, 투입 3,966 (세계당 2.42), 무투입 세계 3.2%, 이전 투입이 이력에 있는 투입 60.0%; 세계당 0/1/2/3/4/5회 = 53 / 269 / 561 / 492 / 226 / 39.
- f 삼분위별 갈래 비율(선물 / 부패 / 썩힘): 낮음 80.9% / 5.3% / 13.8% (투입 1444, 일일 투입률 0.0693); 중간 40.0% / 34.2% / 25.8% (투입 860, 일일 투입률 0.0423); 높음 4.3% / 72.9% / 22.7% (투입 1662, 일일 투입률 0.0830).
- 다중 투입 IPW 유효 표본(처치): gf240 7d 429/1163; gf240 30d 130/540; sp240 7d 396/1174; sp240 30d 126/542; rw120 7d 445/572; rw120 30d 141/269 (WM-4: 44–55).
- obs2 테스트 교란(12셀 평균): 30d: zero 0.188, naive 0.333, ipw 0.405, match5 1.132; ipw − naive +0.072 [+0.068, +0.076]; 7d: zero 0.217, naive 0.517, ipw 0.297, match5 0.797; ipw − naive −0.220 [−0.230, −0.209].

**주 결과표 (12셀 MAE, 시드 평균)**:

| 방법 | v2 7일 | v2 30일 | obs 7일 | obs 30일 | obs2 7일 | obs2 30일 |
|---|---|---|---|---|---|---|
| zero | 0.612 | 0.487 | 0.218 | 0.168 | 0.217 | 0.188 |
| naive | — | — | 0.599 | 0.323 | 0.517 | 0.333 |
| ipw | — | — | 0.391 | 0.382 | 0.297 | 0.405 |
| match5 | — | — | 0.945 | 0.968 | 0.797 | 1.132 |
| an5_arm | 0.485 | 0.594 | — | — | — | — |
| ridge_obs | 0.480 | 0.602 | — | — | 0.537 | 0.514 |
| ridge_priv(진단) | 0.434 | 0.590 | — | — | 0.391 | 0.497 |
| M1b(v2, 3시드) | 0.592 | 0.501 | 0.216 | 0.174 | 0.221 | 0.199 |
| wm3_full(3시드) | 0.496 | 0.511 | 0.229 | 0.175 | 0.239 | 0.200 |
| M1b-obs(WM-4) | — | — | 0.214 | 0.175 | 0.219 | 0.197 |
| M1b-obs2 | — | — | — | — | 0.216 | 0.194 |
| CRN WM-4(시간 제한) | — | — | 0.497 | 0.863 | 0.519 | 0.925 |
| CT WM-4(시간 제한) | — | — | 0.321 | 0.373 | 0.317 | 0.378 |
| CRN-v2 WM-4 | 1.474 | 1.547 | — | — | — | — |
| CT-v2 WM-4 | 1.007 | 1.374 | — | — | — | — |
| CRN-obs2(원저자 방식) | — | — | — | — | 0.402 | 0.780 |
| CT-obs2(원저자 방식) | — | — | — | — | 0.311 | 0.483 |
| CRN-v2(원저자 방식) | 1.649 | 3.645 | — | — | — | — |
| CT-v2(원저자 방식) | 1.243 | 1.920 | — | — | — | — |
| 바닥 | 0.187 | 0.267 | — | — | 0.058 | 0.095 |

**갈래·결과·지평별 효과 (obs2 테스트, 셀 척도 MAE / 부호 정확도; 참값 평균)** — 전체 표는 `runs/judge5/judge.md`. 대표 방법:

| 셀 | 참값 평균 | zero | ipw | ridge_obs | M1b-obs2 s0 | CRN-obs2 s0 | CT-obs2 s0 | wm3_full s0 | 바닥 |
|---|---|---|---|---|---|---|---|---|---|
| gf240|food_stock|7d | +21.72 | 0.436 | 0.558 | 1.024 | 0.431 | 0.881 | 0.452 | 0.401 | 0.172 |
| gf240|food_stock|30d | -17.31 | 0.279 | 0.306 | 0.884 | 0.285 | 0.734 | 0.480 | 0.289 | 0.191 |
| gf240|alive|7d | -0.56 | 0.140 | 0.210 | 0.162 | 0.141 | 0.192 | 0.225 | 0.140 | 0.016 |
| gf240|alive|30d | -2.38 | 0.238 | 0.409 | 0.520 | 0.239 | 0.371 | 0.450 | 0.239 | 0.066 |
| gf240|deaths|7d | +0.00 | 0.002 | 0.202 | 0.008 | 0.024 | 0.335 | 0.296 | 0.047 | 0.004 |
| gf240|deaths|30d | +0.00 | 0.052 | 0.938 | 0.059 | 0.066 | 1.402 | 0.575 | 0.082 | 0.094 |
| gf240|settlements_alive|7d | -0.05 | 0.148 | 0.280 | 0.148 | 0.151 | 0.197 | 0.163 | 0.150 | 0.020 |
| gf240|settlements_alive|30d | -0.06 | 0.207 | 0.403 | 0.766 | 0.208 | 0.388 | 0.432 | 0.208 | 0.077 |
| sp240|food_stock|7d | -77.99 | 1.045 | 0.709 | 0.860 | 0.950 | 0.804 | 0.643 | 0.769 | 0.183 |
| sp240|food_stock|30d | +32.79 | 0.344 | 0.568 | 0.762 | 0.345 | 0.881 | 0.595 | 0.403 | 0.161 |
| sp240|alive|7d | +0.82 | 0.204 | 0.358 | 0.679 | 0.205 | 0.250 | 0.332 | 0.205 | 0.028 |
| sp240|alive|30d | +4.10 | 0.339 | 0.613 | 0.681 | 0.339 | 0.463 | 0.633 | 0.339 | 0.054 |
| sp240|deaths|7d | +0.10 | 0.098 | 0.236 | 0.098 | 0.120 | 0.457 | 0.504 | 0.126 | 0.053 |
| sp240|deaths|30d | +0.12 | 0.150 | 0.229 | 0.239 | 0.165 | 1.504 | 0.750 | 0.169 | 0.127 |
| sp240|settlements_alive|7d | +0.09 | 0.184 | 0.393 | 0.676 | 0.187 | 0.286 | 0.273 | 0.187 | 0.013 |
| sp240|settlements_alive|30d | +0.12 | 0.298 | 0.534 | 0.699 | 0.302 | 0.467 | 0.579 | 0.303 | 0.056 |
| rw120|food_stock|7d | -4.84 | 0.171 | 0.192 | 0.434 | 0.176 | 0.535 | 0.243 | 0.375 | 0.164 |
| rw120|food_stock|30d | +2.16 | 0.134 | 0.157 | 0.242 | 0.136 | 0.916 | 0.384 | 0.162 | 0.158 |
| rw120|alive|7d | -0.09 | 0.036 | 0.099 | 1.535 | 0.037 | 0.271 | 0.134 | 0.038 | 0.017 |
| rw120|alive|30d | +0.16 | 0.047 | 0.195 | 0.186 | 0.048 | 0.374 | 0.316 | 0.051 | 0.030 |
| rw120|deaths|7d | +0.23 | 0.129 | 0.220 | 0.802 | 0.144 | 0.379 | 0.300 | 0.157 | 0.019 |
| rw120|deaths|30d | +0.17 | 0.109 | 0.278 | 0.936 | 0.120 | 1.336 | 0.502 | 0.131 | 0.062 |
| rw120|settlements_alive|7d | +0.00 | 0.013 | 0.113 | 0.013 | 0.017 | 0.242 | 0.089 | 0.018 | 0.007 |
| rw120|settlements_alive|30d | +0.01 | 0.065 | 0.235 | 0.199 | 0.069 | 0.404 | 0.331 | 0.071 | 0.059 |

**실행별 계획/도달 스텝·α·종료 이유·시간; 주(마지막)/보조(최저 val_roll) 체크포인트 비교 (12셀 MAE, 7일 / 30일)**:

| 실행 | 계획 | 도달 | 정지 | α/α_max | 초 | 주(마지막, 반올림) | 보조(최적, 반올림) | 마지막 비반올림 |
|---|---|---|---|---|---|---|---|---|
| `crn_v2_s0` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | ['planned', 'planned'] | 0.99991 | 2,618 | 1.795 / 4.187 | 1.780 / 3.249 | 1.816 / 4.165 |
| `crn_v2_s1` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | ['planned', 'planned'] | 0.99991 | 2,905 | 1.504 / 3.104 | 1.633 / 1.531 | 1.775 / 3.125 |
| `ct_v2_s0` | {'ct': 11850} | {'ct': 11850} | ['planned'] | 0.99991 | 13,114 | 1.118 / 1.950 | 1.203 / 1.500 | 1.234 / 1.952 |
| `ct_v2_s1` | {'ct': 11850} | {'ct': 11850} | ['planned'] | 0.99991 | 12,051 | 1.368 / 1.889 | 1.203 / 0.969 | 1.390 / 1.884 |
| `crn_obs2_s0` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | ['planned', 'planned'] | 0.99991 | 2,249 | 0.403 / 0.770 | 0.401 / 0.709 | 0.400 / 0.770 |
| `crn_obs2_s1` | {'crn_dec': 53800, 'crn_enc': 15700} | {'decoder': 53800, 'encoder': 15700} | ['planned', 'planned'] | 0.99991 | 1,848 | 0.401 / 0.790 | 0.456 / 1.125 | 0.399 / 0.791 |
| `ct_obs2_s0` | {'ct': 11850} | {'ct': 11850} | ['planned'] | 0.99991 | 10,665 | 0.304 / 0.502 | 0.294 / 0.346 | 0.301 / 0.500 |
| `ct_obs2_s1` | {'ct': 11850} | {'ct': 11850} | ['planned'] | 0.99991 | 10,047 | 0.317 / 0.465 | 0.303 / 0.370 | 0.316 / 0.465 |
| `m1b_obs2_s0` | 8,000 | 8000 | max_steps | — | 12,992 | 유지 스텝 7950, val T2 24 h 0.0622, 반올림 True | — | — |
| `m1b_obs2_s1` | 8,000 | 8000 | max_steps | — | 11,964 | 유지 스텝 7950, val T2 24 h 0.0620, 반올림 True | — | — |

M1b-obs2 사실 T2 (obs2 테스트, 원 단위, 매시간): `m1b_obs2_s0` 24 h 0.1158, 168 h 0.1785; `m1b_obs2_s1` 24 h 0.1157, 168 h 0.1782.

비용 점검: _cost_crn_obs2 0.044 s/step → 계획 {'crn_dec': 53800, 'crn_enc': 15700}; _cost_crn_v2 0.097 s/step → 계획 {'crn_dec': 53800, 'crn_enc': 15700}; _cost_ct_obs2 0.910 s/step → 계획 {'ct': 11850}; _cost_ct_v2 0.998 s/step → 계획 {'ct': 11850}; aborted:_cost_ct_obs2_contaminated 1.894 s/step → 계획 {'ct': 7349}; aborted:_cost_ct_obs2_contaminated_2 1.833 s/step → 계획 {'ct': 7735}.
계산: 녹화 40,085 s, 학습 22.3 GPU 시간(동시 1개).

### 환경
- **Mac**(`/Users/rexxa/work/new-world-wm1`, Darwin 27.0.0 arm64 T6041): 편집, 커밋, 푸시, 작은 단위 테스트, 웹 인용 확인, 그림 생성(matplotlib 3.10.8). 녹화·학습·평가 없음, `.mac_quiet.lock` 미사용.
- **DGX Spark**(Linux 6.14.0-1015-nvidia aarch64, NVIDIA GB10, 드라이버 580.126.09, CUDA 13.0, 20코어, 메모리 119 GiB, venv torch 2.14.1+cu130 / numpy 2.5.2, 설치 변경 없음): 녹화기 `_build/src_wm2`(재빌드 안 함, sha256 `52b36541…`, `WORLDSIM_GIT_SHA=f5373ab2…`), 데이터 `/home/hyunlord/work/worldsim-wm-data/obs2/`, 실행 `/home/hyunlord/work/wm1-runs/wm5/<run>/`, 코드 작업트리 `wm5/_code/<commit>`. 다른 사용자 작업이 계속 있었음(부하 2–36, 다른 사용자의 ComfyUI가 GPU 최대 29 GiB 점유·간헐적 계산).
- 브랜치 `codex/worldmodel-2026-10-wip`, 기반 `64d633ff`. 훅 없음, `.claude/settings.json` = `{}`(skip-worktree, 손대지 않음).

### 독립 리뷰
독립 리뷰 2회(각각 새 `general-purpose` 서브에이전트, 읽기 전용, `REVIEW.md`에 원문 + 응답).
- **1회차**(10-19 11:55–12:25, HEAD `cc3f744c`): BLOCKING 0, MAJOR 2, MINOR 5, NOTE 12.
  - M1: v2의 관측 회귀가 갈래 평균 이상을 하는지 → judge 사후 블록(`ridge_obs`는 `arm_mean`·`an5_arm`과 TIE, `ridge_priv`는 7일 WEAK).
  - M2: obs2 회귀 단계는 전이 시험 → 투입 행 지지 영역 블록(선물·썩힘 0 %, 부패 20.4 %), 전이로 표기.
  - m1–m5: 누락 파일 가져오기, DECISIONS L2.1–L2.5, ESS 126–141, 바닥을 천장으로 부르지 않기, DATASET_OBS2 인용.
  - 대응 커밋 `c5ecf7b7`.
- **2회차**(10-19 21:51–22:09, HEAD `3d27cc07`): BLOCKING 0, MAJOR 2, MINOR 9, NOTE 8.
  - 기계적 재현: judge·`paper.md`·`paper_numbers.json`·그림 PNG 모두 바이트 단위 재현, 판정 5개 독립 재계산 일치.
  - MJ1: 30일 반전 주장이 사후·선택적(선물에서는 CRN s1이 평균을 거의 맞힘) → 초록에서 삭제, 본문은 사후·두 행위·두 시드로.
  - MJ2: 반전 메커니즘을 사실로 서술 → 미검증 가설로 축소.
  - m1–m9: 범위 표기, Q3 "barely", M1b-obs2 비수렴 공개, 검증 곡선 일반화 수정, IPW 문구, `judge.md` 사후 블록 렌더링·전이 표기, 분량(판정 표를 부록 F로), CRN 일정의 출처(CT 저자 구현), 시드 표기.
  - 대응은 이 보고의 마지막 커밋 전 WIP 커밋에 포함(DECISIONS L7.2). 규칙·판정·수치 변경 없음(`judge.json` 바이트 동일).
- 리뷰 루프는 GOAL 상한대로 2회로 끝냄.

### 자율 결정 목록
`DECISIONS.md` L0.1–L7.2. 주요 항목:
- **L0.2 / L0.6 / L0.7** 46시간 마감 2026-10-20 19:13:47 + 일시정지(SSH 정지 2.15시간 → 21:22:47; Mac 덮개 수면 1.71시간 — 모든 학습이 이미 시작됨).
- **L0.3** WM-4 체크포인트 sha256을 이 실행에서 처음 기록. **L0.4** 시드 47000–48639 미사용(이동 없음).
- **L1.1** 라운드·가족은 세계별 녹화기 호출(W개 병렬). **L1.2** 빈 `pre`는 녹화기의 `none` 구성원. **L1.3** 추가 검사(sha256, 재생, `none_r0` = 사실). **L1.4** 관측 전용 매니페스트를 먼저 커밋해 GPU 공백 없이 obs2 학습 시작(최종 매니페스트의 관측 항목과 동일). **L1.5** 첫 `fam-check`의 검사기 버그(녹화기의 여분 `none`을 관측 에피소드와 비교) 수정, 880/880 통과.
- **L2.1–L2.5** IPW 위험 집합 = p_tot > 0 행; c2가 60일 이후일 때 `match5` 대조 행; obs2 사례 rep 부호; obs2의 M1은 720행 롤아웃; `ridge_priv`의 숨은 필드 범위(`world_variety.regions` 제외).
- **L3.1** GOAL의 "v2 7일 0.487/0.189"는 실제로 30일 셀 값 — 두 지평 모두 커밋된 파일에서 보고.
- **L4.1** 단일 큐. **L4.2** WM-4 재채점이 커밋값을 재현(`ct_obs_s1`만 1e-4 상대 수준 GPU 비결정성). **L4.3** obs2 평가는 매니페스트를 담은 `f7cf620a`에서(규칙 변경 없음). **L4.4** CT-obs2 비용 점검을 우리 평가와 겹쳐 두 번 측정(1.89·1.83 s/step → 계획 축소) → 오염으로 판단해 세 번째 깨끗한 측정(0.91 s/step, 계획 유지)을 사용, 세 측정 모두 공개.
- **L6.1** WM4.md 안내 줄. **L6.2** 논문 숫자는 judge 파일에서 자리표시자로 해석. **L7.1** 리뷰 1회차 응답. **L7.2** 리뷰 2회차 응답(문구, 부록 F, `judge.md` 사후 블록 렌더링, 그림 시드 표기; 규칙·판정·수치 변경 없음).

### 실행 기록
`RUNLOG.md` 전체. Part별 시간(KST, 일시정지 포함 벽시계):
- **Part 0** 10-18 21:13:47 – 21:16 (설정, sha256, 시드 검사).
- **Part 1** 21:17:47 – 10-19 10:28:40: 라운드 0 51분(8 workers), 라운드 1–5 6,140 / 4,619 / 2,637 / 954 / 144초(4 workers), finalize 02:11, val 가족 02:11–04:01, test 가족 04:01–08:27, 가족 검사·매니페스트 10:27–10:28 (SSH 정지 08:18–10:27 포함).
- **Part 2** 21:20 – 21:50 (코드·테스트), 이후 리뷰 대응까지 수정.
- **Part 3** part a 21:24:24, part b 21:39:47.
- **Part 4** 학습 10-18 21:29:44 – 10-19 21:28:57 (CRN-v2 2,618·2,905초, CT-v2 13,114·12,051초, M1b-obs2 12,992·11,964초, CRN-obs2 2,249·1,848초, CT-obs2 10,665·10,047초, 비용 점검 7회(M1b-obs2 포함), 학습 합계 22.3 GPU 시간); 테스트 평가 21:40 – 21:49:53.
- **Part 5** judge 10-18 21:46 첫 커밋 ~ 최종 10-19 21:52.
- **Part 6** 논문 10-18 21:47 ~ 10-19 21:52 (학습과 병행), 리뷰 2회 대응으로 22:02–22:10 수정·재렌더.
- **Part 7** 리뷰 1회 11:55–12:25, 리뷰 2회 21:51:10–22:08:46(대응 22:02–22:10, 커밋 `7405a470` 22:10:14), 게이트 Mac 22:11 / DGX 22:10:37–22:14:14, fast-forward·PR 댓글 22:14, 보고 커밋 그 직후.
- 일시정지: SSH 정지 08:18–10:27(7,740초), Mac 덮개 수면 17:58:33–19:40:59(6,146초); DGX 작업은 계속.

### 확인된 제한사항
- 시뮬레이터 하나, 장면 하나, 개입 3종(고정 양). obs2 정책은 합성(알려진 성향, 불응기·상한), 사례는 정책의 투입 시점을 물려받음.
- M1b-obs2는 8,000스텝 상한에서 수렴하지 못함(유지 스텝 7,950, 검증 손실 하락 중; WM-4의 `m1b_obs_s1`과 같은 상태) — Q3의 M1b 비교가 데이터 증가의 효과를 과소평가할 수 있음.
- CRN·CT는 일 단위 해상도, WM-4가 obs에서 고른 구성 하나, 시드 2개. 주 체크포인트(마지막)가 이 데이터에서는 최적 사실 체크포인트가 아님(보조로 함께 보고).
- 직접 회귀는 선형이고 v2의 반사실 라벨로 학습: obs2에서는 지지 영역 밖 외삽이라 관측/숨은 상태의 몫을 말할 수 없음(사후 확인). `ridge_priv`는 `world_variety.regions` 미포함.
- 바닥은 다른 복제의 효과: 연속 대칭 노이즈에서 노이즈 한계 오차의 ≈ √2배, 희소 개수 셀에서는 `zero`보다 나쁠 수 있음 — 천장이 아님.
- 단일 실행(ridge)은 W1 규칙상 WEAK가 상한; E2 obs2의 KEEP는 ridge의 전이 실패를 반영.
- CT-obs2 비용 점검을 반복(L4.4, 공개); obs2 학습은 관측 전용 매니페스트 사용(L1.4).
- GPU 수치 재현성 미측정(WM-4 CT 재채점 1건 1e-4 상대 차이). DGX는 다른 사용자와 공유(부하 2–36, GPU 공유 구간 있음).
- WM-4와 WM-5의 실행 이름 충돌(`crn_v2_s0` 등) — 문서에서 `wm4:` 접두로 구분.
- 논문: 저자·학회·공개 계획은 자리표시자([AUTHORS], [VENUE], [OWNER: release plan]); Hájek(1971)은 검증 불가로 인용 안 함.

### 게이트 결과
게이트 대상 커밋 `7405a470f3244f4598b5b6cd3f319e3aee1615bd`(리뷰 2회 대응 포함):
- **Mac** `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'`: 233개 실행, **OK (skipped=27)**, 16.3초. 건너뛴 테스트는 Mac에 없는 녹화기·데이터·torch를 쓰는 것으로, 허용 대상.
- **DGX** `tools/worldmodel/wm5/launch/gate.sh 7405a470…`:
  - 작업트리 `/home/hyunlord/work/wm1-runs/wm5/_code/7405a470`, venv `wm1-venv`;
  - 결과: 233개 실행, **OK, 건너뜀 0**, 214.9초(22:10:37–22:14:14), exit 0;
  - 출력: `/home/hyunlord/work/wm1-runs/wm5/gate_7405a470/gate.{txt,exit}`.
- **경로 검사** `git diff --name-only 64d633ff..7405a470`: `tools/worldmodel/wm5/**`, `docs/worldmodel/wm5/**`, `docs/worldmodel/paper/**`, `docs/worldmodel/wm4/WM4.md`(제목 아래 한 줄)만 포함. 그 밖의 경로는 0개.
- **`figures.py check`**: 463개 숫자 모두 원본과 일치. `judge5.py` 재실행 결과 `judge.json`은 바이트 동일.
- **브랜치 이동:**
  - 녹색 확인 뒤 `codex/worldmodel-2026-10`을 `64d633ff..7405a470`로 fast-forward(강제 푸시 없음).
  - PR #512는 draft 상태 유지. 규칙 표와 논문 링크를 담은 댓글을 남김: https://github.com/hyunlord/new-world/pull/512#issuecomment-6268093038
  - 그 뒤의 마지막 커밋(이 보고서와 실행 기록)은 문서만 바꾸며 `-wip` 브랜치에만 올라감.

### 정직한 판단
- **원저자 방식의 CRN·CT는 효과 없음과 관측 기준선을 이기는가?** 아니다. 계획 스텝을 100 % 채우고 α를 최대로, 마지막 체크포인트·반올림까지 원저자 방식으로 했는데도 obs2·v2, 7일·30일 모두 NONE이다. obs2에서 `zero`(val로 고른 관측 기준선) 대비 CRN +0.185·+0.591, CT +0.093·+0.295. v2에서는 WM-4의 시간 제한 실행보다 더 나쁘다(사후). "제대로 학습시키지 않았다"는 반론은 이 데이터에서 성립하지 않는다.
- **벤치마크는 판별력이 있는가?** 있다. 바닥과 `zero` 사이 여유가 노이즈 복제가 있는 두 분할(v2, obs2)의 두 지평에서 크고 구간이 0에서 멀다(v2 +0.425 / +0.220, obs2 +0.159 / +0.094). 관측 상태의 몫: v2 7일에서 선형 회귀가 `zero`보다 낫지만 갈래 평균 수준(사후 TIE)이다. 숨은 상태의 몫: 세계별 정보를 조금 더하지만(사후 WEAK) 바닥까지는 거리가 멀다. 즉 남은 여유의 대부분은 시작 상태의 선형 함수가 아니다. obs2에서는 v2 회귀가 전이되지 않아 이 분해를 할 수 없다.
- **관측 데이터를 늘리면 나아지는가?** 거의 아니다. M1b는 아주 조금(1.5–2 %, WEAK), CT는 7일 동률·30일 악화. M1b-obs2는 수렴 전에 멈췄으므로 과소평가일 수 있다. 유효 표본이 7–10배인데도 CRN·CT의 교란 방향 과반응은 그대로다.
- **WM-4의 결론 중 바뀐 것은?** "CT의 결과는 domain confusion에 대해 말해 주는 게 적다"는 단서가 사라졌다 — 완전히 학습해도 같다. "반올림이 사망 셀 오차의 원인일 수 있다"는 의심은 기각(D2). "아무것도 `zero`를 못 이긴다"는 v2 7일에서 선형 회귀가 갈래 평균 수준으로 이긴다는 점에서 수정되지만, 관측 분할에서 학습 모델에 대한 결론은 그대로다. 바뀌지 않은 것: 세계 모형은 행동에 거의 반응하지 않고, 짝 갈래 학습(`wm3_full`)이 v2 7일 최선의 학습 모델이며, 30일 반전은 부패에서 어떤 학습 동역학 모델도 재현하지 못한다(사후; 선물은 시드마다 갈림).
- **논문 초안은 제출 가능한 수준인가, 무엇이 남았는가?** 구조·숫자 추적·참고문헌 검증·사전 등록 판정이 갖춰진 초안(본문 ≈ 9–10쪽: 산문 약 5,200단어, 표 2개, 그림 3개 + 부록 A–F)이다. 제출 전 남은 것: 저자·학회·데이터 공개 계획(자리표시자), 두 번째 시뮬레이터 장면 또는 다른 개입 양(일반화), CRN·CT의 하이퍼파라미터 탐색이 v2/obs2에서 없었다는 점의 보강(원저자 기본값 대비), 비선형 직접 회귀(관측 상태의 몫을 선형 이상으로), obs2에 맞춘 회귀 학습(전이가 아닌 분해), 그림 다듬기와 분량 조정.
