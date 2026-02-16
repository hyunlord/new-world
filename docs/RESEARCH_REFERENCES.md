# Research References

게임 내 수치와 모델의 학술적 근거를 정리한 문서.

---

## Siler 사망률 모델

### 왜 Siler 모델인가?

Gompertz-Makeham 법칙 (μ(x) = αe^{βx} + λ)은 성인(~30~90세) 사망률을 잘 모델링하지만,
영아기의 높은 사망률과 급격한 감소(bathtub curve의 왼쪽 벽)를 잡지 못한다.

Siler(1979)는 3개 항의 합으로 전 생애 "욕조 곡선"을 한 번에 표현:

```
μ(x) = a₁·e^{-b₁·x} + a₂ + a₃·e^{b₃·x}

항 1: a₁·e^{-b₁·x} = 영아/소아 위험 (출생 직후 높고 빠르게 감소)
항 2: a₂             = 배경 위험 (사고, 감염, 폭력 — 나이와 무관)
항 3: a₃·e^{b₃·x}   = 노쇠 위험 (성인 이후 지수적 증가, Gompertz 역할)
```

### 파라미터 선택 근거

- Gurven & Kaplan의 수렵채집 사회 생존곡선 크로스컬처 비교 연구:
  성인 이후 Gompertz 증가의 doubling time이 6~9년 (β ≈ 0.08~0.11).
  이 값은 현대 선진국과 크게 다르지 않음 → "노화 속도 자체는 보편적"

- Siler 파라미터는 아래 목표를 만족하도록 튜닝:
  - tech=0: 영아사망률(q0) ≈ 0.40, 출생기대수명(e0) ≈ 33년
  - 이는 !Kung San, Hadza, Ache 등 현대 수렵채집민 데이터와 정합

- Tsimane 자료: 영아사망률 ~13%(감염 55% 주도) — 우리 게임은 "가혹한 원시 베이스라인"이므로 Tsimane보다 높은 40%를 채택

- 시대 차이는 β(노화 기울기)가 아닌 λ(=a₂, 배경위험)와 a₁(영아항)에서 주로 발생한다는 것이 20세기 사망률 감소 연구의 핵심 발견

### 게임 내 파라미터 (tech=0)

| 파라미터 | 값 | 의미 | 목표 |
|---------|---|------|------|
| a1 | 0.60 | 영아 위험 수준 | 0세에서 μ_infant ≈ 0.60/yr → q0 ≈ 0.40 |
| b1 | 1.30 | 영아 위험 감소 속도 | 1세: 0.60×e^{-1.30} ≈ 0.16, 5세: 거의 0 |
| a2 | 0.010 | 배경 위험 (연 1%) | 모든 나이에서 연 1% 배경 위험 (감염/사고/폭력) |
| a3 | 0.00006 | 노쇠 스케일 | 40세에서 ≈ 0.002, 70세에서 ≈ 0.033 |
| b3 | 0.090 | 노화 기울기 | doubling time ≈ ln2/0.09 ≈ 7.7년 |

### 기대 연간 사망확률 (tech=0)

| 나이 | 사망확률/년 | 지배 항 |
|------|-----------|--------|
| 0세 | ~40% | 영아(a₁) |
| 1~4세 | 5~10% | 영아(감소중) |
| 5~14세 | 1~2% | 배경(a₂) |
| 15~39세 | 1~2% | 배경(a₂) |
| 40~59세 | 1%→5% | 노쇠(a₃) 상승 |
| 60~79세 | 5%→20%+ | 노쇠(a₃) 지배 |
| 80+ | 20%→40%+ | 노쇠(a₃) |

---

## 기술 수정자

### 왜 항별 독립 수정자인가?

역사적 사실: 기대수명 증가의 대부분은 영아사망률 감소에서 왔다.
- 백신, 항생제, 산과의학 → 영아/감염 사망 극적 감소 (m₁ 크게 감소)
- 공중보건, 위생, 수도 → 배경 감염 사망 감소 (m₂ 크게 감소)
- 만성질환 치료 → 노년부 개선은 후행하며 효과 작음 (m₃ 작게 감소)

이 패턴은 "인구 전환 이론(Demographic transition)"과 정합.

### 감쇠율

```
m_i(tech) = exp(-k_i × tech)

k₁ = 0.30 → tech=10이면 m₁ ≈ 0.05 (영아 위험 95% 감소)
k₂ = 0.20 → tech=10이면 m₂ ≈ 0.14 (배경 위험 86% 감소)
k₃ = 0.05 → tech=10이면 m₃ ≈ 0.61 (노쇠 위험 39% 감소)
```

### 검증: tech=10 기대 결과

- q0 ≈ 0.005 (영아사망률 0.5%)
- e0 ≈ 78~82년
- 15세 조건부 기대수명 ≈ 80~85년

---

## 개인별 Frailty 모델

Vaupel et al.(1979)의 frailty 모델: 개인별 체질 차이가 고령 사망률 분산을 설명.

```
μ_final(x) = z × μ(x)
z ~ N(1.0, 0.15), clamp [0.5, 2.0]
```

z는 출생 시 결정, 생애 불변. 높은 z = 더 허약 = 사망률 곡선 전체 상향.

---

## 임신/출산

### 재태기간 분포

- 평균 재태기간: 280일 (40주)
- 실제 분포: 정규분포에 가까움, σ ≈ 10~12일
- 95%가 260~300일 (37~43주) 범위
- 조산 정의: < 259일 (< 37주)
- 과숙 정의: > 294일 (> 42주)

게임 구현: `randfn(280.0, 10.0)`, clamp [154, 308]

### 조산 생존률

| 주수 | 현대 NICU (tech=10) | 원시시대 (tech=0) | 출처 |
|------|-------------------|-----------------|------|
| 22주 | 생존 2~3% | 거의 0% | ACOG periviability consensus |
| 24~27주 | 1년 내 사망 26.2% | 거의 0% | WHO "Born Too Soon" |
| 28~31주 | 1년 내 사망 6.0% | 생존 가능하나 허약 | WHO |
| 32~33주 | 1년 내 사망 2.4% | 50% 미만 | WHO |
| 34~36주 | 만삭 대비 유의하게 높음 | 생존하나 리스크 있음 | WHO |
| 37~42주 | 정상 만삭 | 정상 만삭 | - |

게임 w50 (50% 생존 기준 주수): tech=0 → 35주, tech=10 → 24주

### 모성사망

- Pre-industrial 모성사망: 출산 1회당 ~1~1.5% (18~19세기 영국 추정)
- 현대 글로벌 MMR: 197/100,000 (2023 WHO 추정)
- 난산(obstructed labor): 약 5% 추정

게임: tech=0 기본 1.5%, tech=10 기본 0.02%

### 쌍둥이

자연 상태 쌍둥이 확률: ~9.1/1000 출산 (약 0.9%). 일란성은 ~3.5/1000.

---

## 출처 목록

1. **Siler, W. (1979)**. "A competing-risk model for animal mortality."
   *Ecology*, 60(4), 750-757.
   → 3항 위험도 합 모델 원저

2. **Gompertz, B. (1825)**. "On the nature of the function expressive of the law of human mortality."
   *Phil. Trans. Royal Society*.
   → 성인 사망률 지수 증가 법칙 원저

3. **Makeham, W.M. (1860)**. "On the Law of Mortality and Construction of Annuity Tables."
   *J. Inst. Actuaries*.
   → Gompertz에 상수항(배경위험) 추가

4. **Gurven, M. & Kaplan, H. (2007)**. "Longevity Among Hunter-Gatherers: A Cross-Cultural Examination."
   *Population and Development Review*.
   → 수렵채집 사회 생존곡선, 조건부 기대수명, β 범위

5. **Vaupel, J.W. et al. (1979)**. "The impact of heterogeneity in individual frailty on the dynamics of mortality."
   *Demography*.
   → 개인별 frailty(z) 곱 모델

6. **WHO (2012)**. "Born Too Soon: The Global Action Report on Preterm Birth."
   → 조산아 생존률 시대/의료 격차

7. **WHO (2014)**. "Every Newborn: an action plan to end preventable deaths."
   → 신생아 사망 감소 전략 및 데이터

8. **ACOG (American College of Obstetricians and Gynecologists)**.
   Periviability consensus (22-25 weeks).
   → 극조산 생존/후유증 앵커

9. **Jukka Corander et al.** Tsimane 영아사망 연구.
   → 영아사망의 55%가 감염성 원인
