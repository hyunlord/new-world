# T-2014-10: HEXACO Personality System Documentation

## Objective
Create `docs/PERSONALITY_SYSTEM.md` documenting the academic basis, data sources, design decisions,
and implementation details of the HEXACO personality system.

## Non-goals
- Do NOT modify any code files
- Do NOT create or modify any GDScript files
- Only create the documentation file

## Scope
Files to CREATE:
- `docs/PERSONALITY_SYSTEM.md` — Complete design document

## Document Content

Create the following document:

```markdown
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
- CK3의 "성격과 반대 행동 시 스트레스" 메커니즘 향후 적용 가능

## 4. 게임 구현 사례 참고

| 게임 | 방식 | 참고점 |
|------|------|--------|
| Dwarf Fortress | beliefs/values/facets | 성격 = 기질, values = 규범으로 분리 |
| RimWorld | 이산 Trait + Mood/Thought | 가시적 라벨/버프/디버프 |
| Crusader Kings 3 | Trait + Stress 시스템 | 성격 반대 행동 → 스트레스 누적 → Mental Break |

## 5. 참고 문헌

- Ashton, M. C., & Lee, K. (2007). Empirical, theoretical, and practical advantages of the HEXACO model of personality structure. *Personality and Social Psychology Review*, 11(2), 150-166.
- Lee, K., & Ashton, M. C. (2004). Psychometric properties of the HEXACO personality inventory. *Multivariate Behavioral Research*, 39(2), 329-358.
- Ashton, M. C., & Lee, K. (2009). The HEXACO-60: A short measure of the major dimensions of personality. *Journal of Personality Assessment*, 91(4), 340-345.
- Ashton, M. C., & Lee, K. (2016). Age trends in HEXACO-PI-R self-reports. *Journal of Research in Personality*, 64, 102-111.
- Vernon, P. A., Villani, V. C., Vickers, L. C., & Harris, J. A. (2008). A behavioral genetic investigation of the Dark Triad and the Big 5. *Personality and Individual Differences*, 44(2), 445-452.
- Zettler, I., et al. Meta-analysis of HEXACO personality and behavioral outcomes.
- Gurven, M., & Kaplan, H. (2007). Longevity among hunter-gatherers. *Population and Development Review*, 33(2), 321-365.
```

## Acceptance Criteria
- [ ] `docs/PERSONALITY_SYSTEM.md` exists
- [ ] Contains all 5 sections (model rationale, data sources, hybrid design, game references, bibliography)
- [ ] All numerical values match the spec (correlation matrix, heritability, Cohen's d, etc.)
- [ ] Korean and English text present where appropriate
- [ ] Bibliography entries are complete
