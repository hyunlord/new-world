# V7 Hook Governance v3.3.3 — Architecture Mismatch Correction

> **Type**: Architecture / Policy Correction (claude.ai 영역)
> **Trigger**: Claude Code W2 보고 — amendment §4.1 hook spec mismatch + dimension list Tests 누락 발견
> **Parent**: v3.3.1 amendment (a5f90f4d) + v3.3.2 amendment (f078c155)
> **Status**: Active correction, supersedes v3.3.1 §3.1 dimension list + §4.1 N5.1 spec
> **Implementation impact**: V.4 hook 정정 단순화 + generate_report.sh 정정 추가 의무 + amendment §3.1/§4.1 cascade fix

---

## 📋 메타 정보

| 항목 | 내용 |
|------|------|
| 작성일 | 2026-05-05 |
| 작성자 | claude.ai (워크플로우 boundary 준수) |
| 버전 | v3.3.3 amendment v1.0 |
| Trigger | Claude Code W2.1/W2.2 보고 — generate_report.sh 존재 + Tests 20 dimension 발견 |
| 이전 amendment | v3.3.1 (a5f90f4d) + v3.3.2 (f078c155) |

### claude.ai 자가 학습 (3번째)

이번이 v3.3 작성 후 3번째 amendment. 누적 패턴:
- v3.3.1: Score scale 갭 (0-80 vs 0-100)
- v3.3.2: §3.1 stale "Threshold 90/100" → 72/100 정정
- **v3.3.3 (이번)**: Hook architecture mismatch + dimension list Tests 누락

근본 원인 분석:
1. **v3.3 통합 명령 작성 시 실제 hook + generate_report.sh 미검증**
2. **Claude Code 1166줄 draft의 SCORE_TESTS=18을 "임의 수치"로 무효화** — 실제로는 정확한 reference (Tests 20 dimension 실재)
3. 사용자 axiom #2 (수학적 정밀, 학술/게임사례/실측) 위반 — 실측 검증 부족

**좋은 점**: Claude Code의 W2 grep 검증으로 V.4 진행 전 발견. 사용자 axiom #1 사전 보호.

---

## 📑 Section 1: 진짜 Architecture 식별 (실측)

### 1.1 Score 산출 책임 분담 (실측)

Claude Code W2.1/W2.2 grep 검증 결과:

```
Score 산출: tools/harness/generate_report.sh L260-362 (단독 책임)
  - Mechanical Gate (max 10)
  - Plan Quality (max 5)
  - Code Quality (max 15)
  - Test Coverage (max 20)  ← v3.3.1 §3.1 dimension list 누락!
  - Visual Verify (max 20)
  - Regression (max 15)
  - Evaluator (max 15)
  Total max: 100

Verdict 산출: harness_pipeline.sh + Evaluator
  verdict file (.harness/reviews/<feature>/verdict, 5-line format):
    Line 1: APPROVED/REJECTED/RE-CODE
    Line 2: <feature_name>
    Line 3: <epoch>
    Line 4: <total score> ← hook이 consume
    Line 5: PIPELINE_ACCEPTABLE (optional grade)

Hook (pre-commit-check.sh): Pure consumer
  - verdict line 4 추출 또는 pipeline_report.md에서 score 추출
  - dimension별 raw 산출 X (재계산 X)
  - SCORE_THRESHOLD와 비교 (cold tier 75, hot tier 72)
  - vlm_env_cost 보정 (hot tier 시 +8, cold tier 시 +0)
```

### 1.2 v3.3.1 §3.1 dimension list 오류

v3.3.1 §3.1:
```
Hot tier max: 80 (Mech 10 + Plan 5 + CQ 15 + Visual 20 + Reg 15 + Eval 15)
```

**실측 (generate_report.sh L262-362)**:
```
Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15 = 100
```

→ v3.3.1 §3.1은 **Tests 20 dimension 누락**. Hot tier max는 **80이 아니라 100**.

### 1.3 v3.3.1 §4.1 N5.1 spec 오류

v3.3.1 §4.1:
```
- raw 산출 시 visual_auto_credit 더해서 합산
- adjusted = raw + vlm_env_cost
- compare adjusted >= SCORE_THRESHOLD
```

**실제 hook 동작**:
```
score = sed -n '4p' verdict_file (verdict consumer)
adjusted = score + vlm_env_cost
PASS if adjusted >= SCORE_THRESHOLD
```

→ Hook은 **raw 산출 X**. amendment §4.1의 "raw 산출 시 visual_auto_credit" 적용은 **이중 가산 위험** 실재.

### 1.4 Claude Code draft "SCORE_TESTS=18"의 진짜 의미

v3.3 통합 명령 작성 시 (저) Claude Code 1166줄 draft의 SCORE_TESTS=18을 "임의 수치 무효"로 분류. 그러나:
- 실제 generate_report.sh L342에 SCORE_TESTS dimension 존재 (max 20)
- draft의 18은 "Cold tier에서 일부 차감" 의도일 가능성
- 즉 **draft가 실측 더 정확**, 제 v3.3 통합 명령이 부정확

이건 사용자 axiom #2 위반: "Claude Code 자체 결정 = 임의"라는 추정이 너무 강했음. Draft도 일부 실측 base 가능성.

---

## 📑 Section 2: 채택 정책 — D4α (Hook 단순화 + 산출 책임 분리)

### 2.1 Architecture 결정 (D4α 채택)

| 책임 | 컴포넌트 | 행위 |
|-----|---------|------|
| Dimension verdict 산출 | Generator + Evaluator | 각 dimension verdict (VISUAL_OK 등) |
| Raw score 산출 | `generate_report.sh` (L260-362) | dimension verdict → 점수 변환 + total 계산 |
| Cold tier auto credit | `generate_report.sh` (cold tier 분기 추가) | cold tier 시 SCORE_VISUAL=20 강제 |
| Threshold 비교 | Hook (`pre-commit-check.sh`) | score consumer + threshold 비교 |
| Visual auto credit 변수 | `generate_report.sh` 내부 | hook에는 부재 (이중 가산 회피) |

### 2.2 Score Model (REVISED — 실측 dimension 반영)

```
정책 v3.3.3 §2.2: Score Model Tier-Aware (REVISED, dimension 정확)

Score Scale: 0-100 (실측 generate_report.sh dimension 합계)

Hot tier (cold-tier 4 Signal 미충족):
- Max raw: 100 (Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15)
- Threshold: 90/100 (raw 100 × 90% = 90, v3.2.1 SCORE_THRESHOLD=90 호환)
  ★ v3.3.1 §3.1 stale "72" 정정 — 실제 max 100이므로 90이 정합
- VLM SKIP +8 보정 유지 (environmental failure)

Cold tier (4 Signal 충족):
- Max raw: 100 (동일, Visual 20 dimension은 cold tier 시 자동 부여)
- Cold tier auto credit: SCORE_VISUAL = 20 강제 (generate_report.sh L322 분기)
- Threshold: 75/100 (raw 100 × 75% = 75)
- VLM SKIP +0 (cold tier auto credit이 보정 대체)

판정 logic (Hook):
  if [[ "$TIER" == "cold" ]]; then
    SCORE_THRESHOLD=75
    APPLY_VLM_COST=0
  else  # hot tier
    SCORE_THRESHOLD=90  ★ v3.3.3 정정 (72 → 90)
    APPLY_VLM_COST=1
  fi
  
  # Hook은 score 산출 X. verdict line 4에서 score 추출.
  score = $(sed -n '4p' "$verdict_file")
  
  if [[ "$APPLY_VLM_COST" == "1" ]]; then
    adjusted = score + vlm_env_cost  # vlm_env_cost = 8 if VLM SKIP else 0
  else
    adjusted = score
  fi
  
  PASS if adjusted >= SCORE_THRESHOLD
```

### 2.3 v3.3.1 §3.1 정정 (paste용)

**Before** (v3.3.1 §3.1, 실측 누락):
```
Hot tier (cold-tier 4 Signal 미충족):
- Max raw: 80 (Mech 10 + Plan 5 + CQ 15 + Visual 20 + Reg 15 + Eval 15)
- Threshold: 72/100 (raw 80 × 90%, v3.2.1 SCORE_THRESHOLD=90 hardcoded는 
  v3.3 implementation에서 tier별 변수로 대체. v3.3.2 §2.1 정정.)
- VLM SKIP +8 보정 유지 (environmental failure)
```

**After** (v3.3.3, dimension Tests 20 추가):
```
Hot tier (cold-tier 4 Signal 미충족):
- Max raw: 100 (Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15)
  ★ v3.3.3 §1.2 정정: Tests 20 dimension 누락 → 추가 (실측 generate_report.sh 일치)
- Threshold: 90/100 (raw 100 × 90%, v3.2.1 SCORE_THRESHOLD=90 hardcoded와 정합)
  ★ v3.3.3 §1.2 정정: max 80 → 100으로 인해 threshold 72 → 90 (90% 일관)
- VLM SKIP +8 보정 유지 (environmental failure)
```

Cold tier도 동일:

**Before** (v3.3.1 §3.1):
```
Cold tier (4 Signal 충족):
- Max raw: 100 (Visual 20 자동 부여 = +20 auto credit)
- Visual Verify: 0/20 → +20 (cold tier auto credit)
- Threshold: 75/100 (cold 보수적 -15)
- VLM SKIP +0 (cold tier auto credit이 보정 대체)
```

**After** (v3.3.3, dimension list 명시):
```
Cold tier (4 Signal 충족):
- Max raw: 100 (Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 (auto credit) + Reg 15 + Eval 15)
  ★ v3.3.3 §1.2: Tests 20 dimension 명시 (실측 generate_report.sh)
- Visual Verify auto credit: cold tier 시 SCORE_VISUAL=20 강제 (generate_report.sh L322 분기 추가)
- Threshold: 75/100 (cold 보수적 -15 from hot 90)
- VLM SKIP +0 (cold tier auto credit이 보정 대체)
```

### 2.4 v3.3.1 §4.1 N5.1 spec 정정 (paste용)

**Before** (v3.3.1 §4.1):
```
N5.1 (재수정):
- pre-commit-check.sh tier branching logic 유지
- SCORE_THRESHOLD 변수 분기:
  if cold_tier:
    SCORE_THRESHOLD=75
    visual_auto_credit=20  # cold tier에서 Visual 차원 자동 부여
    vlm_env_cost=0
  else:
    SCORE_THRESHOLD=90  # 기존 v3.2.1 동일
    visual_auto_credit=0
    vlm_env_cost=(8 if VLM SKIP else 0)

- raw 산출 시 visual_auto_credit 더해서 합산
- adjusted = raw + vlm_env_cost
- compare adjusted >= SCORE_THRESHOLD
```

**After** (v3.3.3, hook 단순화 + generate_report.sh 책임 명시):
```
N5.1 (재수정 — D4α 채택, Hook은 verdict consumer):

A. Hook (pre-commit-check.sh) 정정:
  - Tier branching logic 유지 (cold_tier_classifier 호출)
  - SCORE_THRESHOLD 변수 분기:
    if [[ "$TIER" == "cold" ]]; then
      SCORE_THRESHOLD=75
      APPLY_VLM_COST=0
    else  # hot tier
      SCORE_THRESHOLD=90  ★ v3.3.3 §2.2 정정 (72 → 90, dimension 100 일관)
      APPLY_VLM_COST=1
    fi
  - Hook 내부 raw 산출 X (이미 verdict line 4에서 추출)
  - visual_auto_credit 변수 hook에는 부재 (이중 가산 회피)
  - adjusted = score + (vlm_env_cost if APPLY_VLM_COST=1 else 0)
  - PASS if adjusted >= SCORE_THRESHOLD

B. generate_report.sh 정정 (별도 작업):
  - L314-339 SCORE_VISUAL 산출 시 cold tier 분기 추가:
    if cold_tier_classifier exit 0 (cold tier confirmed):
      SCORE_VISUAL=20  # cold tier auto credit (VLM verdict 무시)
    else:
      # 기존 logic 유지
      case "$VISUAL_VERDICT" in
          VISUAL_OK)      SCORE_VISUAL=+7  ;;
          VISUAL_WARNING) SCORE_VISUAL=+4  ;;
          VISUAL_FAIL)    SCORE_VISUAL=+0  ;;
          SKIPPED)        SCORE_VISUAL=+0  ;;
      esac
      [[ $SCORE_VISUAL -gt 20 ]] && SCORE_VISUAL=20

  → cold tier 시 generate_report.sh가 SCORE_VISUAL=20 자동 부여
  → hook은 verdict line 4 (이미 cold tier 보정 적용된 score)를 받아 threshold만 비교

bash -n PASS 의무 (둘 다)
시뮬레이션 4 cases (Hot/Cold/Mixed/GDScript) 모두 expected 결과
```

### 2.5 Step V.4 절차 (REVISED)

```
V.4 절차 (v3.3.3 §2.4 spec 따라):

V4.1 — Hook 정정 (단순화):
  pre-commit-check.sh L103:
  Before: SCORE_THRESHOLD=54
  After:  SCORE_THRESHOLD=75  # v3.3.3 §2.2
  
  pre-commit-check.sh L107:
  Before: SCORE_THRESHOLD=72
  After:  SCORE_THRESHOLD=90  # v3.3.3 §2.2 정정 (72 → 90, dimension 100)
  
  visual_auto_credit 변수 신설 X (이중 가산 회피)
  raw 산출 logic 변경 X (hook은 consumer)
  
  bash -n PASS 의무

V4.2 — generate_report.sh 정정 (별도):
  L314-339 SCORE_VISUAL 산출 영역 cold tier 분기 추가:
  
  # 기존 logic 전:
  if bash tools/harness/cold_tier_classifier.sh "$STAGED" >/dev/null 2>&1; then
      SCORE_VISUAL=20  # v3.3.3 §2.4 cold tier auto credit
  else
      # 기존 case "$VISUAL_VERDICT" 그대로
      case "$VISUAL_VERDICT" in
          VISUAL_OK)      SCORE_VISUAL=+7  ;;
          VISUAL_WARNING) SCORE_VISUAL=+4  ;;
          VISUAL_FAIL)    SCORE_VISUAL=+0  ;;
          SKIPPED)        SCORE_VISUAL=+0  ;;
      esac
      [[ $SCORE_VISUAL -gt 20 ]] && SCORE_VISUAL=20
  fi
  
  bash -n PASS 의무

V4.3 — 4 case 시뮬 (V.4 기존 명세 그대로):
  Hot tier (sim-systems): max 100, threshold 90
  Cold tier (sim-core): max 100 (Visual auto credit), threshold 75
  Mixed: hot fallback
  GDScript: hot fallback

V4.4 — T2 retroactive simulation:
  T2 변경 set으로 cold_tier_classifier exit 0 확인
  generate_report.sh 시뮬 — SCORE_VISUAL=20 자동 부여
  
  T2 score 산출:
    Mech 10 + Plan 5 + CQ 10 (LOCK -5) + Tests 20 + Visual 20 (auto credit) + Reg 15 + Eval 15 = 95
    Threshold 75
    PASS (95 >= 75) ✓ — boundary 아님, 안전 마진 +20
    
    ★ v3.3.3 §1.2 정정으로 T2 retroactive: 75/100 boundary → 95/100 안전 PASS
    ★ Tests 20 dimension 추가로 score 자연스럽게 상승

V4.5 — cargo test --workspace 회귀 0건:
  Hook + generate_report.sh 변경이 cargo test와 무관 의무

V4.6 — 단독 commit (또는 두 commit 분리):
  Option 1: 단일 commit (hook + generate_report.sh 함께)
    chore(harness)[V7][V3.3-N5.1]: tier branching v3.3.3 + cold visual auto credit
  
  Option 2: 두 commit (hook + generate_report.sh 분리)
    chore(harness)[V7][V3.3-N5.1a]: hook tier branching v3.3.3
    chore(harness)[V7][V3.3-N5.1b]: generate_report.sh cold tier auto credit
  
  권장: Option 1 (logical unit, audit chain 단순)
```

---

## 📑 Section 3: Implementation Impact

### 3.1 정정 cascade (3 layer)

| Layer | 정정 |
|-------|------|
| Layer A: v3.3.1 amendment (a5f90f4d → 후속 정정 commit) | §3.1 dimension list Tests 20 추가 + Hot threshold 72→90 |
| Layer B: v3.3 ticket inline (13d08c4a → 후속 정정 commit) | §5.4 + cross-ref Hot threshold 72→90 (다시) |
| Layer C: V.4 implementation | Hook (Cold 54→75, Hot 72→90) + generate_report.sh (cold tier auto credit) |

### 3.2 v3.3.2 amendment 처리

v3.3.2 amendment (f078c155)는 "Hot threshold 90 → 72" 정정이었지만 **v3.3.1 dimension list 자체가 잘못됨** (Tests 20 누락) 때문에 v3.3.2 정정 자체가 잘못된 base에서 정정한 것.

**v3.3.2 amendment 처리**:
- 그대로 보존 (audit chain 가치)
- v3.3.3 amendment가 supersede 명시 (메타 영역에 superseded 추가)
- v3.3.2 amendment의 Hot 72 정정은 dimension list 80 가정에서는 정합, 100 가정에서는 90 정합
- v3.3.3 §2.2가 ground truth: Hot 90 (dimension 100 일관)

### 3.3 V.4 commit 구조 (v3.3.3 적용)

```
V.4.1: amendment v3.3.3 등록
  .harness/prompts/governance_v3_3_3_amendment.md (이 문서 verbatim)
  단독 commit

V.4.2: v3.3.1 amendment 자체 정정
  §3.1 dimension list Tests 20 추가 + Hot threshold 72→90
  §4.1 N5.1 spec hook 단순화 (이중 가산 회피)
  메타 v3.3.3 reflected
  단독 commit

V.4.3: v3.3 ticket inline 재정정
  §5.4 dimension list 갱신 + Hot threshold 72→90
  Cross-reference 모두 90으로 정정
  메타 v3.3.3 reflected
  단독 commit

V.4.4: Hook 정정
  pre-commit-check.sh:
    cold SCORE_THRESHOLD: 54 → 75
    hot SCORE_THRESHOLD: 72 → 90
  단독 commit (hook 자체)

V.4.5: generate_report.sh 정정
  L314-339 cold tier 분기 추가
  단독 commit (또는 V.4.4와 묶기)

V.4.6: T2 retroactive verify
  T2 새 score 95/100 확인
  push + audit chain 갱신
```

---

## 📑 Section 4: 사용자 결정 사항 (E1~E5)

```
□ E1: "v3.3.3 amendment design은 정교해, implementation OK" 사용자 명시 approve
□ E2: D4α 채택 OK (Hook 단순화 + generate_report.sh 책임)
□ E3: Hot tier threshold 90 (max 100 × 90%) 인정 — v3.3.2 §2.1 supersede
□ E4: Tests 20 dimension 추가 인정 — Claude Code draft가 실측 정확
□ E5: V.4 commit 구조 (5 commits, V.4.1~V.4.6) OK 또는 다른 분할
      옵션 a: 5 commits 순차 (audit 명확)
      옵션 b: amendment 정정 묶음 (V.4.1+V.4.2+V.4.3 1 commit) + Hook+generate 1 commit = 2 commits
      권장: 옵션 a (V.2/V.3 패턴 일관성)
```

---

## 🎯 v3.3.3 종합

### 정정 범위 요약

| 발견 | v3.3.3 정정 |
|-----|-------------|
| Hook = pure consumer (raw 산출 X) | §4.1 N5.1 spec 단순화 |
| generate_report.sh가 score 단독 산출 | §4.1 별도 작업으로 명시 |
| Tests 20 dimension 누락 | §3.1 dimension list 추가 |
| Hot max 80 → 100 (Tests 추가) | Hot threshold 72 → 90 (90% 일관) |
| Claude Code draft SCORE_TESTS=18 | "임의 무효" 분류 정정 (실측 base) |

### Cascade Fix

| Layer | 영향 | 정정 |
|-------|-----|------|
| v3.3.1 amendment | §3.1 + §4.1 | Tests 추가 + threshold 90 + spec 단순화 |
| v3.3.2 amendment | (보존) | superseded 메타 추가 |
| v3.3 ticket inline | §5.4 + cross-ref | dimension list + threshold 90 |
| Hook | bac9ba53 | Cold 54→75, Hot 72→90 |
| generate_report.sh | L314-339 | Cold tier 분기 추가 |

### claude.ai 자가 학습 명시

이번 v3.3.3는 **3번째 amendment**. 누적 패턴 분석:

| amendment | 발견 | 본질 |
|-----------|------|------|
| v3.3.1 | Score scale 갭 (0-80 vs 0-100) | claude.ai가 hook scale 가정 부정확 |
| v3.3.2 | §3.1 stale "Threshold 90" → 72 | claude.ai가 §3.1 vs §3.2 모순 미식별 |
| v3.3.3 | Hook architecture mismatch + Tests 누락 | claude.ai가 generate_report.sh 미검증 + draft 무효화 과도 |

**근본 원인**: 사용자 axiom #2 (수학적 정밀, 학술/게임사례, 실측) — 실측 검증 부족.

**claude.ai 자가 개선**: 향후 governance 문서 작성 시 grep + 실제 코드 실측 의무. 추정/가정 최소화.

### V.4 진행 시간 추정 (REVISED)

V.4.1 (amendment v3.3.3 등록): ~10분
V.4.2 (v3.3.1 amendment 정정): ~20분
V.4.3 (v3.3 ticket inline 재정정): ~20-30분
V.4.4 (Hook 정정): ~15분
V.4.5 (generate_report.sh 정정): ~30분 (cold tier 분기 추가 + 시뮬)
V.4.6 (T2 retroactive verify): ~20분
**총 V.4: ~2시간** (이전 추정 30분-1시간보다 길어짐, 정확성 우선)

---

**문서 버전**: v3.3.3 amendment v1.0
**다음 갱신**: v3.3.4 (필요 시) 또는 v3.4 (Phase 1 land 후)
**갱신 책임자**: claude.ai (architecture/policy 영역)

*v3.3.1 §3.1 dimension list (Tests 20 누락) 정정.*
*v3.3.1 §4.1 N5.1 spec (이중 가산 위험) 정정 — D4α 채택.*
*v3.3.2 amendment supersede (Hot threshold 90, dimension 100 일관).*
*Claude Code draft SCORE_TESTS=18 "임의 무효" 분류 정정 (실측 base 인정).*
*사용자 axiom #1 + #2 회복 (정합성 + 실측).*
