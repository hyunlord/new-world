# V7 Hook Governance v3.3.2 — Hot Threshold Stale Legacy Correction

> **Type**: Architecture / Policy Correction (claude.ai 영역)
> **Trigger**: v3.3.1 amendment §3.1 vs §3.2 자체 모순 발견 (Step V N5.1 재수정 직전)
> **Parent**: v3.3.1 amendment (a5f90f4d)
> **Status**: Active correction, supersedes v3.3.1 §3.1 stale legacy text
> **Implementation impact**: N5 hook Cold threshold 54→75 정정 + v3.3 ticket inheritance 6 line 정정

---

## 📋 메타 정보

| 항목 | 내용 |
|------|------|
| 작성일 | 2026-05-05 |
| 작성자 | claude.ai (워크플로우 boundary 준수) |
| 버전 | v3.3.2 amendment v1.0 |
| Trigger | Claude Code W3 보고 — v3.3.1 §3.1 vs §3.2 모순 + 3-layer cascade |
| 이전 amendment | v3.3.1 (.harness/prompts/governance_v3_3_1_amendment.md, a5f90f4d) |
| 이전 ticket | v3.3 통합 명령 inline patched (.harness/prompts/governance_v3_3.md, 2f34dbcd) |

### claude.ai 오류 인정

v3.3.1 amendment 작성 시 §3.1 정책 텍스트에서 "Hot threshold 90 (v3.2.1 호환)"이라 단순화 표기 → §3.2 Z 보강 결론 ("Hot threshold 72")과 모순. 이건 **amendment 자체 갭**이며 사용자 axiom #1 (정교/정합성) 위반.

좋은 점: 
- Claude Code의 W3 보고로 발견 (Step V 진행 전)
- N5 hook bac9ba53가 §3.2 Z 보강 model 따라 Hot threshold 72 정확
- 단 N5 Cold threshold 54는 v3.3 v1.0 60-scale 잔존 (3중 모순)

---

## 📑 Section 1: 모순 구조 분석

### 1.1 4-Layer Cascade

Claude Code W3 보고에 기반한 정확한 cascade:

```
Layer 1 (claude.ai 오류):
  v3.3.1 amendment §3.1 line 193:
    "- Threshold: 90/100 (기존 v3.2.1 동일)"
  → stale legacy text. v3.2.1 hook의 SCORE_THRESHOLD=90 hardcoded와 호환을 
    너무 단순화하여 표현. Hot raw max 80 < threshold 90 → 영원히 BLOCK 
    (수학적 불가능).

Layer 2 (정합 model):
  v3.3.1 amendment §3.2 lines 240-243 (Z 보강 채택):
    "Hot: 72 (raw 80 × 90%), Cold: 75 (raw 100 × 75%)"
  → 일관된 결론. Z 보강 model이 ground truth.

Layer 3 (Inheritance):
  v3.3 ticket §5.4 (Step U inline patch 시 §3.1 verbatim copy → 2f34dbcd):
    L770-783 judgment block: threshold = 90 for hot
    L700, L706, L744, L862, L907: "Threshold 90/100"
  → 모순 전파. v3.3 ticket이 자기 모순 (§5.4 amendment §3.1 verbatim, 
    §5.2 L711 "Alternative D ... Threshold 72 for cold tier" doubly-stale).

Layer 4 (Hook 부분 정합):
  N5 hook bac9ba53 (tools/harness/hooks/pre-commit-check.sh:103-107):
    Hot SCORE_THRESHOLD=72 ✓ §3.2 일치
    Cold SCORE_THRESHOLD=54 ✗ neither §3.1 (90) nor §3.2 (75)
  → Cold 54는 v3.3 v1.0 Alt D 60-scale (60×0.9=54). 100-scale migration 
    (§3.1+§3.2의 Cold 75) 미반영. 3중 모순.
```

### 1.2 정확한 정정 범위

```
v3.3.2 정정 의무:

A. v3.3.1 amendment 자체 정정 (.harness/prompts/governance_v3_3_1_amendment.md):
   §3.1 line 193: "Threshold: 90/100" → "Threshold: 72/100 (raw 80 × 90%)"
   메타에 v3.3.2 reflected 명시

B. v3.3 ticket inline patch 재정정 (.harness/prompts/governance_v3_3.md):
   §5.4 judgment block (~L770-783): hot threshold 90 → 72
   L700, L706, L744, L862, L907: "Threshold 90/100" → "Threshold 72/100"
   §5.2 L711 doubly-stale: "Alternative D ... 60 + Threshold 72 for cold" 
                           → "Alternative D ... raw 100 + Threshold 75 for cold"
                           (cold tier는 100-scale, threshold 75)
   메타에 v3.3.2 reflected 추가

C. N5 hook 정정 (tools/harness/hooks/pre-commit-check.sh):
   line 103-107 영역:
   Cold SCORE_THRESHOLD=54 → 75 (100-scale, §3.2 Z 보강 model)
   Hot SCORE_THRESHOLD=72 (이미 정확, 변경 X)
   Cold visual_auto_credit=20 (이미 amendment §4.1 명세, 추가 의무)
   단 N5 hook이 amendment §4.1 정확히 따랐는지 grep 검증 의무

이 3 정정이 v3.3.2 implementation 범위.
```

---

## 📑 Section 2: 채택 정책 — Z 보강 Model Confirmation

### 2.1 Hot tier (확정)

```
Hot tier (cold-tier 4 Signal 미충족):
- Max raw: 80 (Mech 10 + Plan 5 + CQ 15 + Visual 20 + Reg 15 + Eval 15)
- Threshold: 72/100 (raw 80 × 90%)  ← v3.3.2 정정 (90 → 72)
- VLM SKIP +8 보정 유지 (environmental failure)
- Score 표시: raw / 80 × 100 = X/100 (UI 표시), 또는 raw 그대로 X/80 표기

근거:
- §3.2 Z 보강 model이 정합 결론
- raw 80 < threshold 90 = 영원히 BLOCK (수학적 불가능)
- raw 80 × 90% = 72 = 합리적 threshold
- v3.2.1 hook의 SCORE_THRESHOLD=90 hardcoded는 v3.3 implementation 시 
  tier별 변수 분기로 대체 (이미 N5에서 처리됨)
```

### 2.2 Cold tier (확정)

```
Cold tier (4 Signal 충족):
- Max raw: 100 (Mech 10 + Plan 5 + CQ 15 + Visual 20 (auto credit) + 
                Reg 15 + Eval 15 + Plan_Quality_Bonus 0)
  → 80 + 20 (Visual auto credit) = 100
- Threshold: 75/100 (raw 100 × 75%)  ← v3.3.2 확인 (54 → 75)
- VLM SKIP +0 (cold tier는 environmental 부재 자체)
- Score 표시: raw 그대로 X/100

근거:
- §3.2 Z 보강 model 결론
- Cold tier는 raw 100 (Visual auto credit 포함)
- Threshold 75는 cold tier 검증 부족 인정 (Hot 90% 대비 -15 보수적)
- 100-scale로 통일 (60-scale 잔존 X)
```

### 2.3 v3.3.1 §3.1 정확한 정정 (paste용)

v3.3.1 amendment §3.1의 다음 line 정정 (verbatim 교체):

**Before** (stale):
```
- Threshold: 90/100 (기존 v3.2.1 동일)
```

**After** (v3.3.2):
```
- Threshold: 72/100 (raw 80 × 90%, v3.2.1 SCORE_THRESHOLD=90 hardcoded는 
  v3.3 implementation에서 tier별 변수로 대체. v3.3.2 §2.1 정정.)
```

이 정정이 §3.1과 §3.2 사이 모순 해소.

### 2.4 v3.3 ticket §5.4 정확한 정정 (paste용)

`.harness/prompts/governance_v3_3.md` §5.4 judgment block (~L770-783):

**Before** (stale):
```
  if cold_tier_classifier exit 0:
    visual_score = 20 (auto credit)
    threshold = 75
    vlm_env_cost = 0
  else:  # hot tier
    visual_score = (산출된 score, VLM SKIP 시 0)
    threshold = 90  ← stale (§3.1 verbatim copy)
    vlm_env_cost = 8 if VLM SKIP else 0
```

**After** (v3.3.2):
```
  if cold_tier_classifier exit 0:
    visual_score = 20 (auto credit)
    threshold = 75
    vlm_env_cost = 0
  else:  # hot tier
    visual_score = (산출된 score, VLM SKIP 시 0)
    threshold = 72  ← v3.3.2 정정 (90 → 72, raw 80 × 90%)
    vlm_env_cost = 8 if VLM SKIP else 0
```

추가 정정 line:
- L700, L706, L744: "Threshold 90/100" → "Threshold 72/100"
- L862 (§7 V7 Hot tier 정상 동작): "max 80, threshold 90" → "max 80 raw, threshold 72/100"
- L907 (§9 G7): 동일

§5.2 L711 doubly-stale:
**Before**:
```
**채택**: **Alternative D (Max score 80 + Threshold 72 for cold tier)**.
```
→ 이건 v3.3 v1.0 60-scale 잔존. v3.3.1 amendment에서 100-scale로 migration했어야 했는데 누락.

**After** (v3.3.2):
```
**채택 (v3.3.1 update, v3.3.2 confirmation)**: 
  **Cold tier: Max raw 100 (Visual auto credit +20) + Threshold 75 (100×75%)**
  **Hot tier: Max raw 80 + Threshold 72 (80×90%)**
  
  V3.3.1 amendment에서 100-scale migration 채택, v3.3.2에서 Hot threshold 
  72 confirmation (§3.1 stale "90" 정정).
```

### 2.5 N5 Hook 정확한 정정 (paste용)

`tools/harness/hooks/pre-commit-check.sh` line 103-107 영역:

**Before** (N5 bac9ba53 — Cold 54 잔존):
```bash
if cold_tier_classifier exit 0:
    SCORE_THRESHOLD=54  # ← stale, v3.3 v1.0 60-scale 잔존
    visual_auto_credit=0  # ← stale, amendment §4.1 따라 20이어야
    vlm_env_cost=0
else:
    SCORE_THRESHOLD=72  # ✓ §3.2 Z 보강 정확
    visual_auto_credit=0
    vlm_env_cost=$((VLM SKIP ? 8 : 0))
```

(실제 N5 bac9ba53 line은 Claude Code가 grep으로 정확한 형식 확인 의무)

**After** (v3.3.2 N5.1):
```bash
if cold_tier_classifier exit 0:
    SCORE_THRESHOLD=75  # v3.3.2 §2.2 (raw 100 × 75%)
    visual_auto_credit=20  # v3.3.1 §4.1 cold tier auto credit
    vlm_env_cost=0
else:
    SCORE_THRESHOLD=72  # v3.3.2 §2.1 (raw 80 × 90%, 변경 없음)
    visual_auto_credit=0
    vlm_env_cost=$((VLM SKIP ? 8 : 0))
```

raw 산출 logic도 visual_auto_credit 포함 의무:
```bash
raw=$((mech + plan + cq + visual_score + visual_auto_credit + reg + eval))
adjusted=$((raw + vlm_env_cost))
```

PASS 조건:
```bash
if [[ $adjusted -ge $SCORE_THRESHOLD ]]; then PASS; else BLOCK; fi
```

---

## 📑 Section 3: Implementation 명세 (Step V REVISED)

### 3.1 Step V 절차 (v3.3.2 적용)

```
Step V.1: amendment v3.3.2 등록
  V1.1 .harness/prompts/governance_v3_3_2_amendment.md 작성 (이 문서 verbatim)
  V1.2 단독 commit:
       chore(harness)[V7][V3.3-AMEND-V2]: v3.3.2 Hot threshold stale legacy correction
       
       v3.3.1 amendment §3.1 stale legacy text "Threshold 90/100" 정정:
       - §3.1 line 193: 90/100 → 72/100 (raw 80 × 90%)
       - §3.2 Z 보강 model 정합 결론 confirmation
       - 4-layer cascade fix: amendment + v3.3 ticket + N5 hook
       
       Implementation impact:
       - amendment §3.1 정정 (Step V.2)
       - v3.3 ticket §5.4/§5.2/§7/§9 cross-reference 정정 (Step V.3)
       - N5 hook Cold threshold 54→75, visual_auto_credit 20 추가 (Step V.4)
       
       Authored by claude.ai, approved by user (kwan hyeon) 2026-05-05.
       
       Co-Authored-By: Claude <noreply@anthropic.com>
  V1.3 push + 검증

Step V.2: v3.3.1 amendment 자체 정정
  V2.1 .harness/prompts/governance_v3_3_1_amendment.md §3.1 line 193 정정:
       "Threshold: 90/100 (기존 v3.2.1 동일)" 
       → "Threshold: 72/100 (raw 80 × 90%, v3.2.1 SCORE_THRESHOLD=90 hardcoded는 
          v3.3 implementation에서 tier별 변수로 대체. v3.3.2 §2.1 정정.)"
  V2.2 메타 영역에 v3.3.2 reflected 명시 추가
  V2.3 단독 commit:
       docs(harness)[V7][V3.3-AMEND-FIX]: correct v3.3.1 §3.1 stale legacy text
       
       v3.3.1 §3.1 line 193 "Threshold 90/100" → "Threshold 72/100" per v3.3.2 §2.1.
       §3.2 Z 보강 model이 정합 결론 (raw 80 < threshold 90 = 수학적 불가능).
       
       v3.3.2 amendment file: .harness/prompts/governance_v3_3_2_amendment.md
       
       Co-Authored-By: Claude <noreply@anthropic.com>
  V2.4 push + 검증

Step V.3: v3.3 ticket inline patch 재정정
  V3.1 .harness/prompts/governance_v3_3.md 정정:
       - §5.4 judgment block (~L770-783): threshold 90 → 72
       - L700, L706, L744, L862, L907: "Threshold 90/100" → "Threshold 72/100"
       - §5.2 L711 doubly-stale: 100-scale로 update (v3.3.2 §2.4)
       - 메타 영역에 "v3.3.2 reflected" 추가
  V3.2 단독 commit:
       docs(harness)[V7][V3.3-INLINE-FIX]: re-patch v3.3 ticket per v3.3.2
       
       v3.3 ticket inherits stale "Threshold 90/100" from v3.3.1 §3.1 verbatim
       copy at Step U (2f34dbcd). v3.3.2 §2.4 정정.
       
       Cross-references:
       - §5.4 judgment block: Hot threshold 90 → 72
       - L700, L706, L744, L862, L907: cardinality update
       - §5.2 L711: doubly-stale Alt D 60-scale → 100-scale
       
       Co-Authored-By: Claude <noreply@anthropic.com>
  V3.3 push + 검증

Step V.4: N5 hook 정정 (N5.1 commit)
  V4.1 baseline diff 0 검증 (vs .harness/audit/v3_2_1_baseline/)
  V4.2 pre-commit-check.sh line 103-107 정정:
       cold tier:
         SCORE_THRESHOLD=75 (v3.3.2 §2.2)
         visual_auto_credit=20 (v3.3.1 §4.1)
         vlm_env_cost=0
       hot tier:
         SCORE_THRESHOLD=72 (변경 없음, v3.3.2 §2.1 confirmation)
         visual_auto_credit=0
         vlm_env_cost=8 if VLM SKIP else 0
  V4.3 raw 산출 logic 정정 (visual_auto_credit 포함):
       raw = mech + plan + cq + visual_score + visual_auto_credit + reg + eval
       adjusted = raw + vlm_env_cost
       PASS if adjusted >= SCORE_THRESHOLD
  V4.4 bash -n PASS
  V4.5 시뮬레이션 4 cases:
       Hot tier (sim-systems): max raw 80, threshold 72, VLM 보정 가능
       Cold tier (sim-core): max raw 100 (Visual +20), threshold 75
       Mixed: hot fallback (Signal A 위반)
       GDScript: hot fallback (Signal C 위반)
  V4.6 T2 retroactive simulation:
       Files: rust/crates/sim-core/src/material/*.rs
       Expected: cold tier 확정, raw 75/100, threshold 75, PASS (boundary)
  V4.7 N5.1 단위 commit:
       chore(harness)[V7][V3.3-N5.1]: tier branching REVISED per v3.3.1+v3.3.2
       
       bac9ba53 (N5) had Cold SCORE_THRESHOLD=54 (v3.3 v1.0 60-scale 잔존, 
       3중 모순 — neither v3.3.1 §3.1 "90" nor §3.2 "75").
       
       v3.3.1 §4.1 + v3.3.2 §2.5 적용:
       - Cold SCORE_THRESHOLD: 54 → 75 (100-scale)
       - Cold visual_auto_credit: 0 → 20 (auto credit)
       - Hot SCORE_THRESHOLD: 72 (변경 없음)
       - raw 산출에 visual_auto_credit 포함
       
       Verification 4 cases all expected:
       - Hot tier: max 80 raw, threshold 72 ✓
       - Cold tier: max 100 raw (Visual +20), threshold 75 ✓
       - Mixed: hot fallback ✓
       - GDScript: hot fallback ✓
       
       T2 retroactive simulation: cold tier confirmed, raw 75/100 = threshold ✓
       
       Co-Authored-By: Claude <noreply@anthropic.com>
  V4.8 push + 검증

Step V 후 N6 (Step W) 진행 GO 보고.
```

### 3.2 검증 (G6 보강)

v3.3.2 정정 후 검증 의무:

```
□ V_check_1: amendment §3.1 line 193 "Threshold 72/100" 확인 (grep)
□ V_check_2: v3.3 ticket §5.4 judgment block "threshold = 72" 확인
□ V_check_3: v3.3 ticket L700-L907 cross-reference 모두 "72/100" 또는 "75/100" 
            (90/100 활성 잔존 0)
□ V_check_4: N5 hook line 103-107 cold SCORE_THRESHOLD=75, visual_auto_credit=20 확인
□ V_check_5: 4 case 시뮬레이션 모두 expected
□ V_check_6: T2 retroactive simulation 75/100 PASS 재확인
□ V_check_7: cargo test --workspace 회귀 0건 (hook 변경이 cargo 영향 X 의무)
```

---

## 📑 Section 4: 사용자 결정 사항 (B1~B3)

```
□ B1: "v3.3.2 amendment design은 정교해, 정정 implementation OK" 사용자 명시 approve
□ B2: 4-layer cascade fix 진행 방식:
      옵션 a: V.1 → V.2 → V.3 → V.4 순차 4 commit
      옵션 b: V.1+V.2 묶음 + V.3 + V.4 = 3 commit (amendment 정정 묶기)
      옵션 c: V.1+V.2+V.3 묶음 + V.4 = 2 commit (문서 정정 모두 묶기)
      
      권장: 옵션 a (4 commit 순차)
      근거: 각 layer 단위 audit, rollback 가능, history 명확
□ B3: 만약 N5 hook 정정 시 cargo test --workspace 회귀 발견:
      - hook 자체는 cargo와 무관 → 회귀 0건 의무
      - 만약 회귀 발견 → 즉시 rollback + 사유 분석 (다른 변경 누적?)
```

---

## 🎯 v3.3.2 종합

### 정정 범위 요약

| Layer | 위치 | 정정 |
|-------|-----|------|
| Layer 1 | v3.3.1 amendment §3.1 line 193 | "90/100" → "72/100 (raw 80 × 90%)" |
| Layer 2 | v3.3.1 §3.2 Z 보강 | 변경 없음 (정합 model, ground truth) |
| Layer 3 | v3.3 ticket inline | §5.4 + 5 line cross-reference + §5.2 L711 정정 |
| Layer 4 | N5 hook bac9ba53 | Cold 54→75, visual_auto_credit 0→20 |

### Implementation Impact

```
N5.1 (V.4 단계):
  Cold SCORE_THRESHOLD: 54 → 75
  Cold visual_auto_credit: 0 → 20
  Hot SCORE_THRESHOLD: 72 (변경 없음)
  
N6 (Step W): 영향 없음
N7 (Step X): spec REVISED 적용 (visual_auto_credit logic 포함)
N8~N12: 영향 없음
```

### claude.ai 자가 학습

이번 amendment 작성 시 §3.1과 §3.2 사이 모순을 만든 것은 **사용자 axiom #1 위반의 작은 사례**. v3.3 작성 시 발견된 score scale 갭과 동일 패턴 (claude.ai가 명세 정합성 사전 검증 부족).

좋은 점:
- Claude Code의 W3 보고로 사전 발견 (Step V 진행 전)
- 4-layer cascade 정밀 분석 → 정확한 정정 가능
- v3.3 governance 자가 진화의 정상 작동 (v3.3 → v3.3.1 → v3.3.2)

워크플로우 boundary 준수:
- claude.ai = amendment 작성 (정책 결정)
- Claude Code = 모순 발견 + implementation orchestration
- 사용자 = 명시 approve + 결정

---

**문서 버전**: v3.3.2 amendment v1.0
**다음 갱신**: v3.3.3 (필요 시) 또는 v3.4 (Phase 1 land 후)
**갱신 책임자**: claude.ai (architecture/policy 영역)

*v3.3.1 §3.1 stale legacy text "Threshold 90/100" 정정.*
*§3.2 Z 보강 model이 ground truth (Hot 72, Cold 75).*
*4-layer cascade 정밀 정정.*
*사용자 axiom #1 (정교/정합성) 회복.*
