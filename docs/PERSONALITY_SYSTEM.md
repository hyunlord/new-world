# HEXACO 성격 시스템 — 설계 문서

## 1. 모델 선택 근거

### 왜 HEXACO인가 (Big Five 대비)
- HEXACO는 Big Five에 Honesty-Humility(H) 축을 추가
- Big Five의 Agreeableness가 HEXACO에서 H(정직/공정/탐욕회피)와 A(용서/온화/인내)로 분리
- H축은 협력/배신/부패 등 사회적 딜레마와 가장 강한 상관 (ρ̂ ≈ -0.48, Zettler et al.)
- Emotionality는 Big Five의 Neuroticism과 달리 애착/감상성 성분 포함
- 출처: Ashton & Lee (2007), Lee & Ashton (2004)

### 왜 24 facet까지 구현하는가
- E(감정성)은 facet 분해 없이는 "겁 많은데 공감 낮은" 같은 조합 불가
- H, E, C는 facet 분해의 효용이 특히 큼
- 본 프로젝트는 "변태적 디테일"을 지향하므로 24 facet 전부 구현

## 2. 데이터 출처 및 수치

### 2-1. 인구 분포 (HEXACO-60 논문)
커뮤니티 성인 표본 평균/SD (1~5 Likert → 0~1 변환: trait01 = (raw-1)/4):

| 축 | 여성 M(SD) | 남성 M(SD) | Cohen's d |
|----|-----------|-----------|-----------|
| H  | 3.98(0.50) | 3.76(0.55) | 0.41 |
| E  | 3.37(0.54) | 2.87(0.49) | 0.96 |
| X  | 3.32(0.65) | 3.26(0.59) | 0.10 |
| A  | 3.38(0.54) | 3.23(0.56) | 0.28 |
| C  | 3.73(0.51) | 3.73(0.52) | 0.00 |
| O  | 3.59(0.65) | 3.62(0.64) | -0.04 |

출처: Ashton & Lee (2009), HEXACO-60 논문 Table 3

### 2-2. 축 간 상관 행렬 (대학생 표본)

|   | H    | E    | X    | A    | C    | O    |
|---|------|------|------|------|------|------|
| H | 1.00 | 0.12 |-0.11 | 0.26 | 0.18 | 0.21 |
| E | 0.12 | 1.00 |-0.13 |-0.08 | 0.15 |-0.10 |
| X |-0.11 |-0.13 | 1.00 | 0.05 | 0.10 | 0.08 |
| A | 0.26 |-0.08 | 0.05 | 1.00 | 0.01 | 0.03 |
| C | 0.18 | 0.15 | 0.10 | 0.01 | 1.00 | 0.03 |
| O | 0.21 |-0.10 | 0.08 | 0.03 | 0.03 | 1.00 |

출처: HEXACO-60 논문 Table 3
→ Cholesky decomposition으로 상관 반영 샘플링에 사용

### 2-3. 유전율 (축별 heritability)

| 축 | h² |
|----|----|
| H  | 0.45 |
| E  | 0.58 |
| X  | 0.57 |
| A  | 0.47 |
| C  | 0.52 |
| O  | 0.63 |

출처: 확장 쌍둥이-가족(extended twin-family) 모델, Vernon et al. (2008) 등
→ mid-parent 회귀 모델에 사용: z_child = h² × z_mid + sqrt(1 - 0.5×h²²) × z_random

### 2-4. 나이에 따른 변화 (성숙 원리)

| 축 | 18→60세 변화 | 연간 drift |
|----|-------------|-----------|
| H  | +1.0 SD     | +0.024 SD/yr |
| E  | +0.3 SD     | +0.007 SD/yr |
| X  | +0.3 SD     | +0.007 SD/yr |
| A  | ~0           | ~0 |
| C  | ~0           | ~0 |
| O  | ~0           | ~0 |

출처: Ashton & Lee (2016) 요약
→ OU(Ornstein-Uhlenbeck) 프로세스로 구현 (θ=0.03, σ=0.03)

### 2-5. 성격 → 행동 도메인 매핑 (메타분석 상관)

| 축 | 행동 도메인 | ρ̂ |
|----|-----------|-----|
| H  | Exploitation(착취/부정) | -0.48 |
| E  | Insecurity(불안/회피) | +0.27 |
| X  | Sociality(사회성/리더십) | +0.53 |
| A  | Obstruction(갈등/보복 반응) | +0.33 |
| C  | Duty(의무/자기통제) | +0.41 |
| O  | Exploration(탐구/창의) | +0.38 |

H 세부:
- 공격성: -0.40, 반사회성: -0.45, cheating: -0.25
- counterproductive behavior: -0.41, low integrity: -0.55
- unethical decision making: -0.51
- 친사회적 행동(경제게임): +0.20

X 세부:
- social network: +0.35, positivity: +0.56, leadership: +0.26

C 세부:
- self-control vs impulsivity: +0.75, perfectionism: +0.46

출처: Zettler et al. 메타분석
→ Utility AI 가중치 함수의 계수로 사용 (Phase C1에서 연동 예정)

### 2-6. 성격 호환성

가중 유사성 기반, 가중치:
- H: 3.0, A: 2.0, C: 1.5, E: 1.0, X: 1.0, O: 0.8
- 신뢰/보복/신뢰성 차원이 관계 안정에 가장 큰 영향
- 상보적 매칭은 실증 혼재 → 약한 보너스로만 처리

## 3. 하이브리드 설계 (연속값 + 이산 Trait)

- 연속값(0.0~1.0): 매 순간 행동 확률을 미세하게 비트는 편향
- 극단(상하 10%): 명명 가능한 Trait 발현 → 서사적 사건 생성
- RimWorld의 "읽히는 캐릭터성" + DF의 "미세한 차이"를 동시에 달성
- CK3의 "성격과 반대 행동 시 스트레스" 메커니즘: TraitViolationSystem으로 구현 완료

### 3-1. Facet Threshold 차별화 설계

**문제**: 모든 facet에 균일 threshold(t_on=0.90) 적용 시,
- 행동 민감도가 높은 facet(X_sociability 등) → 발현 과소, 캐릭터 평탄화
- 극단에서만 서사적 의미가 있는 facet(H_fairness 등) → 발현 과다, 인구 다수가 "부패"

**설계 원칙**: 각 facet의 행동 민감도 · 사회적 비용 · 서사 중요도에 따라 per-facet threshold 차등화.
목표: 에이전트당 facet trait 평균 2~4개, Dark 인구 2~8% 유지.

| Axis | Facet | High_thr | Low_thr | 설계 의도 |
|------|-------|----------|---------|----------|
| H | H_sincerity | 0.92 | 0.08 | 기만은 극단에서만 서사적 |
| H | H_fairness | 0.93 | 0.07 | 부패/착취는 정말 극단에서만 |
| H | H_greed_avoidance | 0.88 | 0.12 | 탐욕/절제는 비교적 자주 체감 |
| H | H_modesty | 0.90 | 0.10 | 오만/겸손은 중간~극단 구간 차등 |
| E | E_fearfulness | 0.82 | 0.18 | 겁/대담은 조금만 치우쳐도 체감 |
| E | E_anxiety | 0.82 | 0.18 | 불안/침착도 민감하게 |
| E | E_dependence | 0.84 | 0.16 | 의존/자립은 중간 민감도 |
| E | E_sentimentality | 0.86 | 0.14 | 공감/냉담은 중간~강 |
| X | X_social_self_esteem | 0.82 | 0.18 | 자신감/열등감은 민감 |
| X | X_social_boldness | 0.82 | 0.18 | 대담/소심은 민감 |
| X | X_sociability | 0.80 | 0.20 | 사교/고독은 체감이 커서 넓게 |
| X | X_liveliness | 0.80 | 0.20 | 활기/무기력도 넓게 |
| A | A_forgiveness | 0.84 | 0.16 | 복수/용서는 중간 |
| A | A_gentleness | 0.84 | 0.16 | 잔혹/온화도 중간 |
| A | A_flexibility | 0.84 | 0.16 | 완고/유연도 중간 |
| A | A_patience | 0.80 | 0.20 | 다혈질/인내는 조금만 낮아도 티남 |
| C | C_organization | 0.86 | 0.14 | 질서/혼돈은 중간~강 |
| C | C_diligence | 0.84 | 0.16 | 근면/게으름은 중간 |
| C | C_perfectionism | 0.86 | 0.14 | 완벽/대충은 중간~강 |
| C | C_prudence | 0.84 | 0.16 | 신중/충동은 중간 |
| O | O_aesthetic_appreciation | 0.86 | 0.14 | 심미/무감각은 중간~강 |
| O | O_inquisitiveness | 0.84 | 0.16 | 호기심/무관심은 중간 |
| O | O_creativity | 0.84 | 0.16 | 창의/고정은 중간 |
| O | O_unconventionality | 0.84 | 0.16 | 비순응/순응은 중간 |

threshold별 대략적 발현률 (SD=0.25, 평균 0.5 기준):
- thr=0.93: 극히 드문 극단 (~4~5%)
- thr=0.84~0.86: 중간 극단 (~6~7%)
- thr=0.80~0.82: 민감 극단 (~9~11%)

### 3-2. Hysteresis (이력현상) 메커니즘

Oscillation 방지를 위한 이중 threshold (`trait_defs_v2.json`):
- `t_on`: 발현 임계 (높음) — facet ≥ t_on → trait 발현
- `t_off`: 소멸 임계 (낮음) — facet ≤ t_off → trait 소멸 (t_off ≈ t_on - 0.06)
- 구현: `trait_system.gd:update_trait_strengths()` 의 hysteresis 로직

### 3-3. 모순 처리 (Contradiction Handling)

같은 facet에서 High/Low trait 동시 발현 가능 (personality_generator.gd: facet_spread=0.75):
- 허용은 하되, 축당 극단 facet trait 최대 2개 유지
- axis ≥ 0.65이면 해당 축의 low facet trait 효과 0.5배 감쇠
- 모순 조합 → "이중성", "위선", "분열" 이벤트 훅 트리거 소재로 활용

### 3-4. Trait 계층 구조 (187개 — trait_defs_v2.json)

| 계층 | 접두어 | 개수 | 발현 기준 | 설계 의도 |
|------|--------|------|----------|----------|
| Facet Trait | — | 48 | facet 1개 극단 (per-facet thr) | 미세 기질 차이 |
| Composite 2축 | c_ | 64 | axis 2개 ≥ 0.75/0.25 | 기질 조합 특성 |
| Composite 3축/역할 | c_ | 60 (3축 59 + 4축 1) | axis 3개 ≥ 0.70/0.30 | 서사적 원형·사회 역할 |
| Dark Trait | d_ | 15 | axis ≤ 0.20~0.30 + facet 극단 | Dark Triad/Tetrad 변형 |

Dark trait 15종: d_psychopath_primary, d_psychopath_secondary, d_machiavellian,
d_narcissist_grandiose, d_narcissist_vulnerable, d_sadist, d_con_artist, d_cult_leader,
d_opportunist, d_bully, d_callous, d_backstabber, d_corrupt_official, d_predatory_raider, d_histrionic

Valence 분포 (trait_defs_v2.json 분석): 긍정 80 / 부정 61 / 중립 46

### 3-5. 발현 확률 목표

| 유형 | 에이전트당 평균 | 전체 인구 |
|------|--------------|---------|
| Facet trait | 2~4개 | facet별 한쪽 4~11% |
| Composite | 1~3개 | 조합 의존 |
| Dark | 0~0.5개 | 전체 2~8% |

### 3-6. Effects 스키마 (6범주)

각 trait의 `effects` 딕셔너리 구조:

| 범주 | 기능 | 수치 범위 |
|------|------|----------|
| `behavior_weights` | Utility AI 행동 가중치 multiplier | Facet: 0.70~1.30 / Composite: 0.60~1.60 / Dark: max 1.80 (실제 JSON: 0.42~2.34, d_corrupt_official 초과 존재) |
| `emotion_modifiers` | Plutchik 감정 민감도·기본선 조정 | 0.75~1.30 |
| `relationship_modifiers` | 신뢰/친밀/갈등 변화율 | 0.75~1.30 |
| `work_modifiers` | 작업속도/품질/실수율/학습속도 | 0.70~1.30 |
| `combat_modifiers` | 공격성/도주임계/위험감수/전술 | 0.70~1.40 |
| `stress_modifiers` | stress_gain/recovery/violation_stress (tier: 6/12/18/24) | — |

효과 적용 방식: 로그 합산 (가산식은 발산 위험):
```
w_final[action] = exp(Σ log(modifier_i))
```
구현: `trait_system.gd:get_effect_value()`, 결과 clamped [0.1, 3.0]

### 3-7. 표시 우선순위 알고리즘

구현: `trait_system.gd:get_display_traits()`

| 상수 | 값 | 의미 |
|------|-----|------|
| TOP_K | 5 | 최대 표시 trait 수 |
| MAX_PER_AXIS | 2 | 축당 최대 표시 수 |
| MAX_DARK_DISPLAY | 2 | Dark 최대 표시 수 |
| MIN_DISPLAY_SALIENCE | 0.10 | 최소 표시 강도 |

우선순위: **Dark/Legendary Composite > Social Role Composite > Extreme Facet > Normal Facet**
Composite 발현 시 해당 구성 facet trait는 UI에서 숨김 (중복 제거).
색상: Positive=초록, Negative=빨강, Neutral=회색/파랑.

## 4. 타 시스템 연동

### 4-1. Utility AI 연동 (BehaviorSystem)

behavior_weights를 행동 스코어에 곱셈 적용:
```
score[action] *= exp(Σ log(trait.effects.behavior_weights[action]))
```

Zettler 메타분석 ρ̂을 가중치로 변환 예시:
- H↑: steal/lie/betray ↓ (weight~0.70), trust_gain ↑ (weight~1.25), guilt ↑
- H↓: exploit/cheat ↑ (weight~1.40~1.80), guilt ↓
- E↑: flee ↑, stress_gain ↑, risk_taking ↓
- X↑: social/leadership ↑, morale ↑; X↓: solitary_work ↑
- A↑: revenge ↓, patience ↑; A↓: aggression ↑
- C↑: plan/build/research 품질 ↑, error ↓; C↓: impulsive ↑
- O↑: explore/research/innovate ↑; O↓: change_stress ↑

### 4-2. 감정 시스템 (Plutchik 기반) 연동

3단 계층 구조:
1. **axis**: 기저 감정 민감도 설정 (연속값 편향)
2. **facet trait** (`emotion_modifiers`): 상황별 반응 편향
3. **composite trait**: 강하고 지속적인 편향 (높은 multiplier)

### 4-3. CK3식 반대 행동 스트레스 (TraitViolationSystem)

구현: `scripts/systems/trait_violation_system.gd` (prio=37, tick_interval=1)

```
violation_stress = base_tier × VIOLATION_ALPHA^frequency × context_modifier × witness_modifier
```

- `violation_stress` tier: 6 (경미) / 12 (보통) / 18 (심각) / 24 (극단)
- `VIOLATION_ALPHA = 1.2` (반복 위반 시 증폭)
- `allostatic_load` 임계 초과 시:
  - 적응 → Desensitization (Bandura 1999: 감각 둔화)
  - 과부하 → PTSD 분기 (Kindling Theory: 역치 하강)
- DSM-5 기반 intrusive thoughts (재경험), Post-Traumatic Growth 포함
- 이론 기반: `violation_mappings.json`에 opposite_actions 목록 정의

## 5. 제약 및 향후 계획

### 현재 알려진 문제 (trait_defs_v2.json — 2026-02)

| 문제 | 상세 | 권장 조치 |
|------|------|----------|
| synergies 누락 | 187개 중 60개가 `synergies: []` 빈 배열 | 향후 synergy 관계 정의 필요 |
| Low threshold 과극단 | 일부 facet low thr=0.06~0.08 → SD=0.25 분포에서 거의 발현 안 됨 | low thr 최솟값 0.12 이상으로 상향 권장 |
| behavior_weight cap 초과 | `d_corrupt_official.take_bribe=2.34` — 의도 한도(1.80) 초과 | 1.80으로 캡 적용 필요 |

### 다음 단계

- Low threshold 재조정 (0.06~0.08 → ≥0.12)
- d_corrupt_official 및 cap 초과 trait behavior_weight 클램핑
- synergies 관계 정의 (어떤 trait 조합이 시너지를 가지는가)

## 6. 게임 구현 사례 참고

| 게임 | 방식 | 참고점 |
|------|------|--------|
| Dwarf Fortress | beliefs/values/facets | 성격 = 기질, values = 규범으로 분리 |
| RimWorld | 이산 Trait + Mood/Thought | 가시적 라벨/버프/디버프 |
| Crusader Kings 3 | Trait + Stress 시스템 | 성격 반대 행동 → 스트레스 누적 → Mental Break (→ TraitViolationSystem) |

## 6. 참고 문헌

- Ashton, M. C., & Lee, K. (2007). Empirical, theoretical, and practical advantages of the HEXACO model of personality structure. *Personality and Social Psychology Review*, 11(2), 150-166.
- Lee, K., & Ashton, M. C. (2004). Psychometric properties of the HEXACO personality inventory. *Multivariate Behavioral Research*, 39(2), 329-358.
- Ashton, M. C., & Lee, K. (2009). The HEXACO-60: A short measure of the major dimensions of personality. *Journal of Personality Assessment*, 91(4), 340-345.
- Ashton, M. C., & Lee, K. (2016). Age trends in HEXACO-PI-R self-reports. *Journal of Research in Personality*, 64, 102-111.
- Vernon, P. A., Villani, V. C., Vickers, L. C., & Harris, J. A. (2008). A behavioral genetic investigation of the Dark Triad and the Big 5. *Personality and Individual Differences*, 44(2), 445-452.
- Zettler, I., et al. Meta-analysis of HEXACO personality and behavioral outcomes.
- Gurven, M., & Kaplan, H. (2007). Longevity among hunter-gatherers. *Population and Development Review*, 33(2), 321-365.
