# V7 Hook Governance v3.3.1 — Score Scale Amendment

> **Type**: Architecture / Policy Amendment (claude.ai 영역)
> **Trigger**: v3.3 implementation 진행 중 발견된 score scale 갭 (Claude Code N5 commit bac9ba53 보고)
> **Parent**: v3.3 통합 명령 (.harness/prompts/governance_v3_3.md)
> **Status**: Active amendment, supersedes v3.3 §5.4 + §5.5
> **Implementation impact**: N5 재수정 + N7 spec 수정

---

## 📋 메타 정보

| 항목 | 내용 |
|------|------|
| 작성일 | 2026-05-05 |
| 작성자 | claude.ai (워크플로우 boundary 회복) |
| 버전 | v3.3.1 amendment v1.0 |
| Trigger | Claude Code N5 단위 보고 — transient state over-permissive |
| 이전 ticket | v3.3 통합 명령 §5.4 + §5.5 |

### 워크플로우 boundary 회복

이전 message에서 저(claude.ai)가 Alternative A/B/C 비교 + 권장을 사용자 답변에 직접 작성했습니다. 이는 **architecture/policy 결정의 책임 영역 모호화**. Claude Code가 정확히 catch.

올바른 절차:
1. v3.3 통합 명령 자체 갭 발견 → 사용자에게 보고
2. 사용자가 claude.ai에 v3.3.1 amendment 요청
3. claude.ai가 deep analysis로 amendment 작성
4. 사용자 명시 approve 후 Claude Code dispatch

이 amendment는 그 절차의 결과물입니다.

---

## v3.3.2 Correction Reflected (2026-05-05)

이 amendment는 v3.3.2 정정 반영:
- §3.1 line 193 (정책 텍스트): "Threshold 90/100" → "Threshold 72/100"
- §3.1 line 210 (판정 logic): "threshold = 90" → "threshold = 72"
- §4.1 line 316 (N5.1 spec): SCORE_THRESHOLD=90 → 72
- §4.3 line 363 (N7 spec): threshold = 90 → 72
- 종합 line 465 (요약): "threshold 90/100" → "threshold 72/100"
- §2.3 Alt C section: historical proposal 보존 + supersede 메모 추가 (D2 보강)
- §3.2 Z 보강 model이 ground truth (변경 없음)
- v3.3.2 amendment file: .harness/prompts/governance_v3_3_2_amendment.md (commit f078c155)
- Audit chain: v3.3 ticket → v3.3.1 amendment → v3.3.2 amendment

---

## v3.3.3 Reflected (2026-05-05)

이 amendment는 v3.3.3 정정 반영 (V.4.2 commit per dispatch instruction):
- §3.1 dimension list: Tests 20 dimension 추가 (max raw 80 → 100)
- §3.1 Hot threshold: 72 → 90 (raw 100 × 90% v3.2.1 자연 호환)
- §3.2 Z 보강 model: max raw 80 → 100, Hot 72 → 90
- §3.3 표시와 산출 분리: scale 변환 불필요 (raw 100 그대로)
- §3.4 T2 retroactive: 75/100 boundary → 95/100 안전 PASS (Tests 20 인지)
- §4.1 N5.1 spec D4α 단순화: visual_auto_credit hook 제거, generate_report.sh 책임
- §4.3 N7 spec: threshold 72 → 90
- §종합: dimension list + Hot threshold 갱신
- §2.3 supersede memo: Alt C 원안 90 회복 정합성 명시
- v3.3.3 amendment file: .harness/prompts/governance_v3_3_3_amendment.md (commit 871f131b)
- Audit chain: v3.3 ticket → v3.3.1 → v3.3.2 → v3.3.3 → 이 self-correction

---

## 📑 Section 1: 갭의 본질

### 1.1 발견된 갭 (Claude Code 보고)

```
v3.2.1 hook 실측 (line 35):
  SCORE_THRESHOLD=90
  hook 출력 표기: "${score}/100" (예: "Score 92")
  → score scale: 0-100

v3.3 통합 명령 §5.4 (claude.ai 작성):
  Hot tier max: 80 (Mech 10 + Plan 5 + CQ 15 + Visual 20 + Reg 15 + Eval 15)
  Cold tier max: 60 (Visual 20 차원 자체 제외)
  → score scale: 0-80 (hot) / 0-60 (cold)

Mismatch:
  Hook은 0-100 scale 처리
  v3.3 §5.4 정의는 0-80 / 0-60 scale
  Verdict 산출이 0-100인지 0-80인지 명확하지 않음
```

### 1.2 v3.2.1 verdict 산출 검증 (실측)

기존 verdict 파일 분석 (`.harness/reviews/*/verdict`):
- `Score: 92/100` 형태 — 100 scale
- 단, raw 산출 dimension은 v3.3 §5.4와 동일 (Mech 10 + Plan 5 + CQ 15 + Visual 20 + Reg 15 + Eval 15 = 80)
- 80 → 100 변환은 어떻게? **명시 정책 부재**

추정 (v3.2.1 verdict 분석):
- 일부 verdict는 raw 80 scale
- 일부 verdict는 ad-hoc 100 scale 변환 (×1.25 또는 80→90 매핑)
- → v3.2.1 자체에 scale 모호성 존재

### 1.3 갭의 진짜 정체

이건 v3.3 통합 명령 작성 시 발견된 **v3.2.1 자체의 잠재 갭** — claude.ai가 v3.3 통합 명령 §5.4에서 명시화하려 했으나 두 가지 모순:

1. **v3.2.1 hook이 100 scale 가정**: SCORE_THRESHOLD=90 hardcoded
2. **v3.3 §5.4가 80 scale 정의**: dimension 합계 80

→ N5 implementation 시 Claude Code가 두 정의 중 하나를 선택해야 했음 → bac9ba53에서 Alternative A 가정 (max 80, threshold 72)

### 1.4 사용자 axiom #1 적용

"매 시스템마다 아주 정교하고 자세하게" → score scale 모호성은 axiom #1 위반. v3.3.1 amendment가 명시적으로 해결.

---

## 📑 Section 2: 3 Alternatives Deep Analysis

### 2.1 Alternative A — Verdict scale 강제 변경 (0-80 / 0-60)

**제안**: 모든 verdict raw 산출을 0-80 (hot) / 0-60 (cold) scale로 강제. v3.3 §5.4 그대로 유지.

**구현 영향**:
- N5 hook: SCORE_THRESHOLD 80 scale로 변경 (hot 72, cold 54)
- N7 score_model.sh: Verdict raw를 80/60 scale로 출력
- 기존 v3.2.1 verdict (100 scale)는 retroactive 변환 또는 무효 처리
- harness_pipeline.sh: Visual 차원 자체 제외 mechanism (cold tier 시)

**장점**:
- v3.3 §5.4 정의 그대로 (claude.ai 본 의도)
- "Visual 차원 자체 제외" 정신 명확

**단점**:
- 기존 verdict 호환성 깨짐
- v3.2.1 → v3.3 transition 시 모든 verdict 재산출 필요
- "왜 max 80?"의 직관적 이해 어려움 (100이 더 자연스러움)
- Claude Code N5 bac9ba53 commit이 Alternative A 가정 → 수정 불필요한 면 있음

**T2 retroactive (Alt A)**:
```
Cold tier max 60:
  Mech 10 + Plan 5 + CQ 10 (LOCK -5) + Reg 15 + Eval 15 = 55
  Visual 차원 자체 제외 (산출 X)
  Threshold 54 (60×90%)
  Result: 55 ≥ 54 PASS ✓
```

### 2.2 Alternative B — Max scale 100 재정의 (차원별 가중치 재조정)

**제안**: Hot tier max를 100으로 재조정. Visual 25, Mech 12 등 차원별 가중치 변경.

**구현 영향**:
- 모든 차원 산출 공식 변경 (10→12, 20→25 등)
- v3.2.1 verdict 호환성 깨짐 (가중치 다름)
- Cold tier max도 80 또는 75 (Visual 25 제외)

**장점**:
- Score scale 100 자연스러움
- threshold 90/100 = 90% 직관적

**단점**:
- 모든 차원 가중치 재조정 = v3.3 통합 명령 §5 전반 재작성
- v3.3 §5.5 T2 retroactive 산출 (55/60) 모두 재계산
- "왜 Visual 25?"의 정당화 추가 필요
- 가장 파괴적 변경

**거부**.

### 2.3 Alternative C — Threshold만 조정 (max 100 scale 유지, Visual auto credit)

**제안**: 
- Hot tier max scale 0-100 유지 (v3.2.1 호환)
- Cold tier 4 Signal 충족 시 Visual 차원 0/20 → +20 auto credit
- Threshold: hot 90 (기존), cold 75 (보수적 차감)

**구현 영향**:
- N5 hook: SCORE_THRESHOLD 변수 분기 (hot 90, cold 75)
- N7 score_model.sh: Cold tier 시 Visual auto credit +20 적용
- 기존 v3.2.1 verdict 호환 (100 scale 유지)
- harness_pipeline.sh: 변경 최소

**장점**:
- v3.2.1 호환성 100% (기존 verdict 그대로)
- Cold tier 정신 유지 (Visual 차원 자동 부여 = 차원 자체 제외와 등가)
- threshold 90 → 75는 cold tier 검증 부족 인정 (보수적)
- Implementation 영향 최소

**T2 retroactive (Alt C)**:
```
Cold tier max 100:
  Mech 10 + Plan 5 + CQ 10 (LOCK -5) + Visual 20 (auto credit) + Reg 15 + Eval 15 = 75
  Threshold 75 (cold)
  Result: 75 ≥ 75 PASS ✓ (boundary case)
```

**Boundary case 분석**:
- T2 score 정확히 75 = threshold 75
- 한 점만 떨어지면 BLOCK
- 이는 "v3.3 정책이 T2 정확히 boundary에서 PASS"라는 정확성 — 사용자 axiom #1 충족
- 단, 마진 부족 → 다른 cold tier work이 75 미만 가능성

**Threshold 70 vs 75 sub-decision**:
- 70: T2 75 ≥ 70 안전 마진 +5, 다른 work도 통과 여유
- 75: T2 정확히 boundary, 정책 정밀도 ↑

**채택**: Alt C + threshold 75
**근거**:
- T2가 정확히 boundary = 정책 정밀도 (사용자 axiom #1)
- 보수적 threshold = cold tier 검증 부족 인정
- 만약 cold tier work이 75 미만 PASS 하려면 추가 dimension 향상 (CQ 15/15 등) — 자연스러운 압력

**Note (v3.3.2 → v3.3.3)**: Alt C 원안의 hot threshold 90은 v3.3.2 §3.2 Z 보강에서 72로 supersede (raw 80 가정 base). v3.3.3 §1.2에서 Tests 20 dimension 인지 → max raw 100 정합 → Hot threshold 90 회복 (Alt C 원안 정합성 복원). 이 section은 historical proposal 보존. 정합 결론은 §3.1 + §3.2 (v3.3.3 reflected).

---

## 📑 Section 3: 채택 정책 — Alternative C

### 3.1 v3.3 §5.4 REVISED

```
정책 v3.3.1 §5.4: Score Model Tier-Aware (REVISED)

Score Scale: 0-100 (v3.2.1 호환 유지)

Hot tier (cold-tier 4 Signal 미충족):
- Max raw: 100 (Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15)
- Hook은 pure consumer (raw 산출 X — verdict line 4 / pipeline_report.md 추출)
- Threshold: 90/100 (raw 100 × 90%, v3.2.1 SCORE_THRESHOLD=90 자연 호환. v3.3.3 §2.3 정정.)
- VLM SKIP +8 보정 유지 (environmental failure)

Cold tier (4 Signal 충족):
- Max raw: 100 (Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 auto credit + Reg 15 + Eval 15)
- Visual Verify: 0/20 → +20 (cold tier auto credit, generate_report.sh 책임)
  → 즉 Visual 차원 자체 점수 부여, 차감 없음
- Threshold: 75/100 (cold 보수적 -15)
- VLM SKIP +0 (cold tier auto credit이 보정 대체)

판정 logic:
  if cold_tier_classifier exit 0:
    visual_score = 20 (auto credit)
    threshold = 75
    vlm_env_cost = 0
  else:  # hot tier
    visual_score = (산출된 score, VLM SKIP 시 0)
    threshold = 90  # v3.3.3 §2.3 정정 (raw 100 × 90%, Tests 20 dimension 추가)
    vlm_env_cost = 8 if VLM SKIP else 0
  
  raw = mech + plan + cq + visual_score + reg + eval
  adjusted = raw + vlm_env_cost
  PASS if adjusted >= threshold
```

### 3.2 Hot tier scale 명확화

v3.2.1 hook은 100 scale 가정. v3.3 §5.4 정의는 raw 80. 변환:

**Option X**: Raw 80을 100 scale로 ×1.25 변환
- 80×1.25 = 100, 60×1.25 = 75
- 단순하지만 직관적이지 않음 ("raw 64는 score 80?")

**Option Y**: Raw 그대로 사용, threshold도 raw 기준
- Hot threshold: 72 (90% of 80)
- Cold threshold: 54 (90% of 60)
- 직관적 raw 매칭, 단 v3.2.1 hook 호환성 깨짐

**Option Z (채택)**: Raw 100 scale로 직접 산출 (각 차원 가중치 그대로 합산하면 80, 단 hook은 100 scale 처리)
- Hot tier max raw: 80 (실제 산출)
- Hook은 raw를 그대로 100 scale로 처리 (max 100 가정)
- Threshold 90 → raw 90 (실제 dimension 합계 80을 초과하므로 raw 80도 BLOCK)
- **이건 명백히 잘못됨** → v3.2.1 자체 모순

**최종 채택 (Z 보강 + v3.3.3 §2.3 정정)**: 
- Hot tier max는 raw 합계 100 (Tests 20 dimension 추가, v3.3.3 §1.2 인지)
- Cold tier max는 raw 합계 100 (Tests 20 + Visual 20 auto credit)
- Hook의 SCORE_THRESHOLD는 tier별 변수 분기:
  - Hot: 90 (raw 100 × 90%, v3.2.1 자연 호환)
  - Cold: 75 (raw 100 × 75%)
- `score/100` 표기는 그대로 유지 (UI 표시), 실제 비교는 tier별 threshold

### 3.3 표시와 산출 분리 (v3.3.3 §2.3 정정 — scale 변환 불필요)

```
Hook 표시 (UI):
  "Score 90/100 (hot tier)" 또는 "Score 75/100 (cold tier)"
  raw 100 scale로 통합 (Tests 20 dimension 추가, 변환 불필요)

내부 산출 (generate_report.sh 책임):
  raw = sum(dimensions) — Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15
  Hot tier raw max: 100
  Cold tier raw max: 100 (Visual 20 auto credit, Tests 20 동일)
  
  Threshold 비교:
  Hot: raw >= 90 (90% of 100)
  Cold: raw >= 75 (75% of 100)

scale 변환 불필요:
  v3.3.3 §1.2에서 Tests 20 dimension 인지 → max raw 100으로 정합
  Hot 표시 score = raw (변환 X, 100 scale 그대로)
  Cold 표시 score = raw (변환 X, 100 scale 그대로)

→ N5 hook은 verdict consumer (line 4 / pipeline_report.md regex). raw 산출 + 표시는 generate_report.sh 단독 책임 (D4α).
```

### 3.4 v3.3 §5.5 T2 Retroactive REVISED

```
T2 retroactive validate (91d4e7c0) — Alt C (v3.3.3 §1.2 Tests 20 dimension 인지 정정):

Cold tier 4 Signal 검증:
✓ A: 모든 변경 sim-core/material/ + lib.rs (sim-core crate)
✓ B: *.rs, *.md 파일만
✓ C: GDScript/Godot 변경 0
✓ D: register_runtime_system 호출 0, RuntimeSystem impl 0
→ Cold tier 확정

Score 산출 (raw, v3.3.3 §1.2 Tests 20 dimension 추가):
Mechanical:  10/10  (FFI vacuous +2 회복)
Plan:         5/5
Code Quality: 10/15 (LOCK_VIOLATION -5 from A2, A1 TEST_RIGOR -0)
Tests:       20/20  (v3.3.3 §1.2 dimension 추가, cold tier 정상 산출)
Visual:      20/20  (cold tier auto credit, generate_report.sh 책임)
Regression:  15/15
Evaluator:   15/15
─────────────────────
Raw:         95/100
Threshold:   75/100
Result:      PASS ✓ (95 ≥ 75, 안전 마진 +20)

VLM cost: 0 (cold tier 자동 부여, 보정 불필요)

★ v3.3.3 §1.2 Tests dimension 추가로 boundary case (75) → 안전 PASS (95) 회복.
v3.3.1 작성 시점에는 Tests 20 dimension 미인지 상태에서 산출 (75/100),
v3.3.3에서 정정 (D5a 채택, V.2 D1b 패턴 일관성).
```

---

## 📑 Section 4: Implementation Impact

### 4.1 N5 재수정 의무 (bac9ba53 → 새 commit)

Claude Code N5 commit bac9ba53는 **Alternative A 가정** (threshold 72/54). v3.3.1 채택 후 재수정 의무:

```
N5.1 (재수정 — v3.3.3 §2.4 D4α 단순화):
- pre-commit-check.sh tier branching logic 유지
- SCORE_THRESHOLD 변수 분기 (D4α — visual_auto_credit hook 제거):
  if cold_tier:
    SCORE_THRESHOLD=75
    vlm_env_cost=0
  else:
    SCORE_THRESHOLD=90  # v3.3.3 §2.4 정정 (raw 100 × 90%, Tests 20 dimension 추가)
    vlm_env_cost=(8 if VLM SKIP else 0)

- Hook은 verdict consumer (raw 산출 X — verdict line 4 / pipeline_report.md 추출)
- Cold tier auto credit은 generate_report.sh 책임 (이중 가산 위험 회피, v3.3.3 §1.3)
- adjusted = score + vlm_env_cost  # score는 verdict에서 추출
- compare adjusted >= SCORE_THRESHOLD

bash -n PASS 의무
시뮬레이션 4 cases (Hot/Cold/Mixed/GDScript) 모두 expected 결과
```

### 4.2 N6 영향 (현재 진행 중)

N6 진행에는 영향 없음. N6는 FFI vacuous integration + RE-CODE 분류 + penalty 산출. Score scale은 N7 담당.

단 N6 시뮬레이션 시 expected 결과 갱신:
```
N6 Step 6 (T2-style cold tier dummy run):
- FFI vacuous CONFIRMED → Mech 10/10
- 1 LOCK_VIOLATION attempt → CQ 10/15 (penalty -5)
- Plan 5/5, Reg 15/15, Eval 15/15
- Visual 0/20 (raw 산출, auto credit은 N7에서 적용)
- Raw subtotal: 10+5+10+0+15+15 = 55/100  ← 변경 없음
- N7에서 cold tier auto credit +20 적용 → 75/100
- Threshold 75 → PASS (boundary)

기존 N6 step 6 (v3.3 원본 v0): "CQ 10/15" 정확. v3.3.1에서도 동일.
```

### 4.3 N7 score_model.sh spec REVISED

```
N7 score_model.sh:
- Input: tier (cold/hot), raw dimensions
- Output: 
  - raw total
  - adjusted score (after VLM env cost or cold tier auto credit)
  - threshold (tier별)
  - PASS/FAIL verdict
- Logic (v3.3.3 §2.3 정정 — Tests 20 dimension 추가):
  if tier == "cold":
    visual_score = 20  # auto credit (generate_report.sh L314-339 분기 책임)
    threshold = 75
    vlm_cost = 0
  else:
    visual_score = input_visual  # 산출된 값
    threshold = 90  # v3.3.3 §2.4 정정 (raw 100 × 90%, Tests 20 dimension 추가)
    vlm_cost = 8 if vlm_skip else 0
  
  raw = mech + plan + cq + tests + visual_score + reg + eval  # Tests 20 추가
  adjusted = raw + vlm_cost
  
  output:
    raw_total: $raw/100  # Hot/Cold 동일 (max raw 100)
    adjusted_total: $adjusted/100
    threshold: $threshold/100
    verdict: PASS if adjusted >= threshold else FAIL
```

### 4.4 구현 순서 갱신

기존 v3.3 구현 순서:
```
N5 → N6 → N7 → N8 self-test → N9 retroactive
```

v3.3.1 적용 순서:
```
N5.1 (재수정) → N6 (진행 중) → N7 (REVISED spec) → N8 → N9
```

N5 재수정은 N6와 병행 가능 (별개 logic 영역).

---

## 📑 Section 5: 사용자 결정 사항

### 5.1 Q6 + Q7 REVISED 답변 의무

| Q# | 원래 (v3.3 v0) | REVISED (v3.3.1) | 근거 |
|---|--------------|------------------|------|
| Q6 | Cold-tier max score = 60 | **Cold-tier max score = 100, Visual auto credit +20** | scale 호환, 100 자연스러움 |
| Q7 | Cold-tier threshold = 54 (60×90%) | **Cold-tier threshold = 75 (100×75%)** | 보수적 차감, T2 boundary 정확성 |

**제 권장**: Q6=A (REVISED), Q7=A (REVISED, threshold 75)

다만 사용자 명시 결정 의무. 만약 다른 결정 원하시면:
- Q7 threshold 70 (안전 마진): T2 75 ≥ 70 마진 +5
- Q7 threshold 80 (엄격): T2 75 < 80 → FAIL → 추가 정책 갭 노출

### 5.2 N5 처리 방법 결정

옵션 P1: bac9ba53 그대로 두고 N5.1 재수정 commit 추가
- 장점: 작업 history 명확 (Alt A → Alt C 진화 audit)
- 단점: Hot tier threshold가 일시적으로 72 (잘못됨)

옵션 P2: bac9ba53 revert + N5 새 commit (Alt C 직접)
- 장점: 깨끗한 history
- 단점: revert overhead

**제 권장**: P1 (재수정 commit 추가)
**근거**:
- v3.3.1 amendment 자체가 progress 표현 → audit chain
- Revert 비용 회피
- N5 + N5.1 두 commit이 "implementation iteration" 자연스러움

### 5.3 v3.3 통합 명령 자체 갱신

`.harness/prompts/governance_v3_3.md`를 v3.3.1 반영해서 갱신할지 결정:
- 옵션 G1: 새 amendment 파일 별도 (`governance_v3_3_1_amendment.md`)
  - 장점: v3.3 원본 보존 (audit chain)
  - 단점: 두 파일 참조 부담

- 옵션 G2: v3.3 원본 inline patch (§5.4 + §5.5 수정)
  - 장점: 단일 ground truth
  - 단점: v3.3 v0 history 손실 (단 git history는 보존)

**제 권장**: G2 (inline patch) + 별도 amendment 보존 (이 문서 자체)
**근거**:
- 단일 ground truth가 implementation 시 혼란 회피
- 이 amendment 문서 (.harness/prompts/governance_v3_3_1_amendment.md)는 audit 가치 보존
- v3.3 통합 명령 §5.4 + §5.5 inline patch + 메타 정보에 "v3.3.1 reflected" 명시

---

## 📑 Section 6: Acceptance Gate (5 추가 items)

v3.3.1 amendment land 전 충족 의무:

```
□ A1: 사용자 명시 approve "v3.3.1 amendment design은 정교해, implementation OK"
□ A2: Q6 + Q7 REVISED 결정 명시 (권장값 또는 다른 결정)
□ A3: N5 처리 방법 결정 (P1 재수정 commit 추가 또는 P2 revert)
□ A4: v3.3 통합 명령 갱신 방법 결정 (G1 별도 또는 G2 inline patch)
□ A5: T2 retroactive 산출 검증 (75/100 ≥ 75 PASS boundary 인정)

A1~A5 모두 충족 후:
- Claude Code dispatch (N5.1 재수정 + N6 계속 + N7 REVISED spec)
- 또는 N7만 REVISED spec 적용 (N6 영향 없음)
```

---

## 🎯 v3.3.1 종합

### 주요 결정

1. **Score scale**: 0-100 유지 (v3.2.1 호환)
2. **Hot tier**: max 100 raw (Tests 20 dimension 추가), threshold 90/100 (v3.3.3 §2.3), VLM SKIP +8
3. **Cold tier**: max 100 raw (Tests 20 + Visual 20 auto credit), threshold 75/100, VLM +0
4. **T2 retroactive**: 95/100 ≥ 75 PASS (안전 마진 +20, v3.3.3 §1.2 Tests 20 인지)
5. **N5 처리**: bac9ba53 + N5.1 재수정 commit 추가

### v3.3 통합 명령 변경 사항

§5.4: 채택 정책 REVISED (위 §3.1)
§5.5: T2 retroactive REVISED (위 §3.4)
§10.1: Q6 + Q7 답변 REVISED (위 §5.1)

### Implementation Impact

| 단계 | 영향 | 대응 |
|-----|-----|------|
| N1~N4 | 영향 없음 | 그대로 |
| N5 | bac9ba53 재수정 의무 | N5.1 commit |
| N6 | 영향 없음 (진행 중) | 그대로 |
| N7 | spec REVISED | §3.3 spec 따라 작성 |
| N8 | self-test 갱신 | T2 75/100 PASS 검증 |
| N9 | retroactive 갱신 | T2 75/100 ≥ 75 PASS |
| N10~N12 | 영향 없음 | 그대로 |

---

**문서 버전**: v3.3.1 amendment v1.0
**다음 갱신**: v3.3.2 (필요 시) 또는 v3.4 (Phase 1 land 후 정식 governance update)
**갱신 책임자**: claude.ai (architecture/policy 영역)

*v3.3 통합 명령 §5.4 + §5.5 supersede.*
*사용자 axiom #1 (정교/명세) 충족 — 갭 발견 → 명시 amendment.*
*Workflow boundary 회복 (Claude Code의 정확한 catch).*
