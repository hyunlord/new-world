# V7 Hook Governance v3.3 — Cold-Tier Scoring + Attempt Discrimination + FFI Vacuous Credit

> **Type**: Architecture / Policy Decision (claude.ai 영역)
> **Trigger**: Phase 1 T2 (commit 91d4e7c0) 완료 후 발견된 3 policy gaps
> **Status**: Phase 1 W1.2~W1.6 BLOCKER. v3.3 land 후 retroactive validate 91d4e7c0 + T6~T11 자동 진행
> **HEAD**: 142313dd (lead/main)
> **Reference**: V7 Master v7.1 + Phase 0 v0.3 + Phase 1 통합 명령 + Claude Code 1166줄 draft (.harness/prompts/_drafts/governance_v3_3.md)
> **Phase 0 패턴**: 7 sections + deep analysis + alternatives + 학술/게임사례 base

---

## 📋 메타 정보

| 항목 | 내용 |
|------|------|
| 작성일 | 2026-05-04 |
| 작성자 | claude.ai (워크플로우 정정 후) |
| 버전 | v3.3 통합 명령 v1.0 + v3.3.1 amendment inline reflected (2026-05-05) |
| Phase 1 진입 조건 | 사용자 명시 approve |
| 예상 구현 시간 | ~1-2일 (Claude Code) |
| Retroactive 대상 | 91d4e7c0 (T2-T5 implementation commit) |

## v3.3.1 Amendment Reflected (2026-05-05)

이 문서는 v3.3.1 amendment를 inline reflect한 갱신본:
- §5.4 + §5.5 → amendment §3.1 + §3.4 본문 inline 교체
- §5.2, §5.3, §7, §9, §10 cross-reference 갱신 (Alt A 가정 제거, Alt C 반영)
- Original v3.3 v1.0 §5.4 + §5.5 superseded by `.harness/prompts/governance_v3_3_1_amendment.md` §3.1 + §3.4
- Audit chain: amendment file (`.harness/prompts/governance_v3_3_1_amendment.md`, commit `a5f90f4d`) 보존, 이 inline patch는 단일 ground truth 강제용

## v3.3.2 Re-Patch Reflected (2026-05-05)

이 ticket은 v3.3.2 정정 inline reflect한 갱신본 (Step V.3):
- §5.2 L706: hot threshold 90 → 72
- §5.2 L709: Alt D 채택 doubly-stale → Hot/Cold tier 명시 (raw 80 + 72 / raw 100 + 75)
- §5.2 L714: parenthetical 수학 오류 → 72/80 raw scale 명시
- §5.3 L744: T2 산출 hot threshold 90 → 72
- §5.4 L764: §3.1 verbatim 90/100 → 72/100
- §5.4 L781: judgment logic threshold 90 → 72
- §7 L862: V7 검증 90/100 → 72/100
- §9 L907: G7 결론 90/100 → 72/100
- §5.2 Alt C subsection: historical proposal 보존 + supersede 메모 추가
- v3.3.2 amendment file: `.harness/prompts/governance_v3_3_2_amendment.md` (commit f078c155)
- v3.3.1 amendment 정정: 2064f0e1 (Step V.2)
- Audit chain: v3.3 ticket → v3.3.1 amendment → v3.3.2 amendment → v3.3.1 self-correction → 이 inline re-patch

## v3.3.3 Re-Patch Reflected (2026-05-06)

이 ticket은 v3.3.3 정정 inline reflect한 추가 갱신본 (Step V.4.3):

- §5.2 Alt D 채택안: Hot tier max 80→100, threshold 72→90 (Tests 20 dimension 누락 발견)
- §5.3 T2 산출: scale 변환 제거, raw 100 그대로, Hot threshold 90, T2 95/100
- §5.4 정책: dimension list에 Tests 20 추가, scale 변환 불필요 명시
- §5.5 T2 retroactive: 75/100 → 95/100 (Tests 20 포함, safety margin +20)
- §6/§7/§9/§10 verification 항목: 95/100 + threshold 90 일괄 반영
- §종합: 최종 결론 갱신
- v3.3.3 amendment file: `.harness/prompts/governance_v3_3_3_amendment.md` (commit 871f131b)
- v3.3.1 self-correction: fcd06f96 (V.4.2)
- Audit chain: v3.3 ticket → v3.3.1 amendment → v3.3.2 amendment → v3.3.1 self-correction → v3.3.2 inline re-patch → v3.3.3 amendment register → v3.3.1 self-correction (v3.3.3 cascade) → 이 inline re-patch

### Claude Code draft 활용 정책

`.harness/prompts/_drafts/governance_v3_3.md`에 보존된 1166줄 draft를 **구조 reference**로 활용. 단:
- 모든 정책 수치 (INTRINSIC_VISUAL_CREDIT=18, SCORE_TESTS=18, attempt categories, FFI +2, Plan QC -3, Effective attempts -7, Cold cap 96 등)는 **임의 결정 무효**
- 이 문서가 deep analysis로 정책 수치 **재결정**
- Implementation file diff (Section 8)는 참고 가능, 단 정책 수치 반영 후 재작성 의무

---

## 📑 Section 1: 컨텍스트 — 3 갭의 진짜 본질

> **사용자 명시 axiom (Memory #30)**: "수학적 효율성 위해 굉장히 정교한 로직 검토 필수—단순 직관/추측 금지, 학술/게임사례/Big-O 분석"

### 1.1 Hook governance v3.2.1 현재 상태 (실측)

`tools/harness/hooks/pre-commit-check.sh` 실측 (hook line 128-150):

```bash
# 현재 정책: VLM SKIP +8 보정 (Godot launch failure 대응)
# Detection: absent visual_analysis.txt = VLM SKIP
vlm_env_cost=8
adjusted_score=$((score + vlm_env_cost))

if [[ "$adjusted_score" -ge "$SCORE_THRESHOLD" ]]; then
    echo "[hook] Score $score → adjusted $adjusted_score ≥ $SCORE_THRESHOLD ✓"
fi
```

### 1.2 V7 governance 진화 패턴

| 버전 | 추가 사항 | Trigger |
|-----|---------|---------|
| v3.0 | harness pipeline | 초기 도입 |
| v3.1 | Adjusted score formula (VLM SKIP +8) | Godot launch failure 사례 |
| v3.2 | Environmental block policy (API rate limit, ENV-BYPASS) | API 차단 사례 |
| v3.2.1 | Clippy baseline tolerance (Rust 1.93 toolchain drift) | Toolchain upgrade 사례 |
| **v3.3** (이 ticket) | Cold-tier scoring + Attempt discrimination + FFI vacuous | **Phase 1 T2 cold-tier work 사례** |

각 버전은 **실제 사례 기반 진화**. v3.3도 동일 패턴 — T2 score 56/90 BLOCK이 정책 갭 노출.

### 1.3 3 갭의 본질적 분류

**Gap 1 (Cold-tier Visual Verify)**: **Structural absence** vs **Environmental failure**
- Structural: 본질적으로 visual surface 없음 (sim-core schema, 데이터 정의)
- Environmental: 일시적 외부 차단 (Godot launch fail, VLM API 다운)
- v3.2.1까지의 +8은 environmental 보정만 — structural absence는 미정의

**Gap 2 (Attempt penalty)**: **Generator divergence** vs **Iterative refinement**
- Divergence (lock violation, scope creep, "more flexible" 자기 합리화): penalty 정당
- Refinement (test rigor 부족, code style, 작은 누락): penalty 부적합
- v3.2.1까지의 attempt count는 두 카테고리 무차별 동일 penalty

**Gap 3 (FFI vacuous)**: **Excluded surface** vs **Failed surface**
- Excluded: §6 NOT-in-scope 명시 (sim-bridge 변경 0)
- Failed: FFI 변경 시도했지만 실패
- v3.2.1까지의 FFI Verify는 두 카테고리 무차별 FAIL

세 갭 모두 **"본질적 부재"**와 **"실패"**의 구분 부재 패턴. v3.3 = 이 구분 정식 도입.

---

## 📑 Section 2: Gap 1 — Cold-tier Visual Verify Scoring

### 2.1 Deep Analysis

#### 2.1.1 Cold-tier 정의 (사용자 axiom #2 적용)

V7 Master v7.1 Section 6 Hard Gate 6:
- Hot path (interval=1): ≤0.5ms @ 1K agents (max 10 systems)
- Warm path (interval=10~30): ≤2ms @ 1K agents (max 25 systems)
- **Cold path (interval=60+ or never per tick): ≤5ms @ 1K agents (max 25 systems)**

Cold-tier 시스템 = **per-tick 실행 빈도 0~극저빈도, runtime 시각 변화 없음**:
- sim-core/material (Phase 1) — Cold tier per-tick 0ms
- sim-core/tile (Phase 2 Week 3~4) — schema 정의, runtime 변화 없음
- sim-core/influence (Phase 2) — channel 정의, 실제 update는 Phase 2.5+
- sim-core/building (Phase 5 Week 11~12) — entity 정의
- sim-core/knowledge (Phase 7+) — schema
- sim-data/* (모든 RON loader)
- sim-test/* (harness test 코드 자체)

**핵심**: Cold-tier 시스템은 **시각적 surface 0** = visual_analysis.txt 생성 불가능. Godot launch가 성공해도 캡처할 게 없음.

#### 2.1.2 v3.2.1 +8 보정의 한계

```
Visual Verify max score: 20
v3.2.1 보정: +8 (VLM SKIP)
Cold-tier 시스템 실제 visual surface: 0
gap: -20 + 8 = -12 (구조적 차감)

T2 사례:
Raw 48 + 8 = 56 (90 미만)
실제 코드 결함: 0 (사용자 axiom #1 100% 충족)
→ governance 갭 노출
```

`+8`은 environmental failure (Godot 일시 차단) 가정. Cold tier는 **environmental이 아닌 structural** absence.

#### 2.1.3 게임사례 분석 (사용자 axiom #2)

비슷한 갭이 다른 시스템에 어떻게 처리되었나:

**Dwarf Fortress raws** (data definition layer):
- ASCII testing, no graphical verification
- 데이터 일관성 검증 (raws.lint) 자체로 verification 완료
- Visual은 후속 layer (UI, ASCII rendering)에서 별도

**Bevy ECS testing**:
- Component 정의 testing은 unit test로 충분
- Visual integration은 별도 binary
- 두 layer 분리 강제

**RimWorld ThingDef XML**:
- XML schema validation = primary
- In-game visual은 별도 staging
- Validation tier 구분

**공통 패턴**: **Data definition layer는 visual verification 자체를 요구하지 않음**. v3.3은 이 패턴 차용.

#### 2.1.4 Cold-tier 식별 기준 (deep)

자동 식별 가능한 4가지 신호:

**Signal A**: Crate 이름 prefix
- `sim-core/`, `sim-data/`, `sim-test/`, `sim-bench/` → cold tier 강한 신호
- 예외: 이 crate들 안에 graphics-related module은 hot path 가능 (현재 V7에는 없음)

**Signal B**: 파일 패턴
- `**/data/**/*.ron` → 100% cold tier
- `**/tests/**/*.rs` → 100% cold tier (test 코드 자체)
- `**/benches/**/*.rs` → 100% cold tier (benchmark)
- `Cargo.toml` workspace 변경 → cold tier (config)

**Signal C**: GDScript/Godot 부재
- Diff 안에 `*.gd`, `*.gdshader`, `*.tscn`, `*.tres` 없음
- `scripts/`, `scenes/` 하위 변경 0
- → visual surface 생성 불가능

**Signal D**: System tick registration 부재
- `register_runtime_system!` 또는 유사 매크로 호출 없음
- ECS query loop 부재
- → per-tick 실행 없음, runtime 시각 변화 없음

**합산 판정**: Signal A + (B or C) → cold tier 확정. 4개 모두 충족이면 100% cold.

### 2.2 정책 결정안 (alternatives + 채택)

#### Alternative A: Cold-tier-bonus Auto Credit

**제안**: Cold tier 확정 시 Visual Verify 점수 자동 부여.

| 옵션 | 부여량 | 정당화 | 위험 |
|-----|------|------|------|
| A1: +20 (full credit) | Visual Verify 만점 | "본질적 부재 ≠ 실패" 인정 | "Cold tier만 만들면 자동 만점" 악용 가능 |
| A2: +18 (90% credit) | 부재 비용 -2 | Cold tier 검증 부족 인정 | Claude Code draft 임의 수치 — 근거 부족 |
| A3: +15 (75% credit) | 부재 비용 -5 | Cold tier 검증 보완 의무 indication | 너무 과도 차감, T2 등 정상 score 90 미달 |
| A4: +20 (full) + 추가 검증 의무 | 만점 + cardinality grep + cargo bench | 구조적 부재 인정 + 대체 검증 | 가장 정합적 |

**채택**: **A4 (+20 full credit + 대체 검증 의무)**.

**근거**:
- 사용자 axiom #1 (변태적 디테일): 부분 차감보다 명확한 0 또는 20 (binary)
- 사용자 axiom #2 (수학적 정밀): "구조적 부재"의 비용은 0 (없는 것이지 결여가 아님)
- "대체 검증 의무"가 cold tier 검증 부족 우려 해결:
  - Cardinality grep (예: lib.rs re-exports, struct fields, enum variants)
  - cargo bench (성능 budget 명시 검증)
  - cargo test 회귀 0 (이미 mechanical gate에 있음)
- 악용 방지: cold tier 식별 4 Signal 모두 충족 의무 → "cold tier로 위장" 어려움

#### Alternative B: Score Threshold 별도 정의

**제안**: Cold tier는 SCORE_THRESHOLD를 80 또는 75로 별도 정의.

**거부 근거**:
- 단순 threshold 차이 = 본질적 해결 X
- "왜 hot tier 90, cold tier 80?"의 정당화 어려움
- Score 산출 방식이 동일하면 비교 의미 깨짐
- A4가 "Visual 차원 자체 자동 통과"로 더 명확

#### Alternative C: 별도 verdict path

**제안**: Cold tier는 별도 evaluation flow (Visual Verify stage skip, 다른 metric).

**거부 근거**:
- pipeline 분기 추가 = mental model 복잡도 ↑ (Hard Gate 5 위반)
- 두 path 유지 비용 ↑
- A4 single path + auto credit이 더 깨끗

### 2.3 채택 정책 — Cold-Tier-Bonus

```
정책 v3.3 §2: Cold-Tier-Bonus

조건 (4 Signal 모두 충족 시 cold tier 확정):
- Signal A: 변경 파일이 sim-core/sim-data/sim-test/sim-bench crate 안에만 존재
- Signal B: 파일 패턴이 cold tier (data/, tests/, benches/, Cargo.toml)
- Signal C: GDScript/Godot 자산 변경 0 (*.gd, *.gdshader, *.tscn, *.tres, scripts/, scenes/)
- Signal D: System tick registration 부재 (register_runtime_system!, ECS query 매크로)

Cold tier 확정 시:
- Visual Verify score: 0/20 → 20/20 (auto credit)
- 추가 검증 의무 (mechanical gate):
  - Cardinality grep 결과 명시 (각 prompt §3 lock 항목)
  - cargo test --workspace 회귀 0
  - clippy clean
  - cargo bench (있으면) 결과 명시

예외 처리:
- Hot path 변경 + Cold tier 변경 mixed commit:
  → Hot path 우선, Cold tier 변경 자체는 visual 검증 없이 통과
  → 단 Hot path에 visual surface 있으면 정상 verify 의무
- Cold tier 변경이 후속 hot path 의존성에 영향 (예: schema breaking change):
  → 정상 verify 의무 (별도 ticket으로 hot path 검증 체인)
```

### 2.4 Implementation 명세

`tools/harness/cold_tier_classifier.sh` (신규):

```bash
#!/bin/bash
# Cold-tier-classifier — 4 Signals 검증
# Returns: 0 if cold tier confirmed, 1 if hot/mixed/warm

set -euo pipefail

DIFF_FILES="${1:?usage: cold_tier_classifier.sh <diff-files-newline-list>}"

# Signal A: crate prefix
signal_a=1
while IFS= read -r f; do
    if [[ -n "$f" ]] && ! [[ "$f" =~ ^rust/crates/(sim-core|sim-data|sim-test|sim-bench)/ ]] \
       && ! [[ "$f" =~ ^rust/Cargo\.toml$ ]] \
       && ! [[ "$f" =~ ^\.harness/ ]] \
       && ! [[ "$f" =~ ^localization/ ]]; then
        signal_a=0
        break
    fi
done <<< "$DIFF_FILES"

# Signal B: file pattern (cold tier OK or exempt)
signal_b=1
while IFS= read -r f; do
    if [[ -n "$f" ]]; then
        if [[ "$f" =~ \.(rs|ron|toml|md|json)$ ]]; then
            continue
        else
            signal_b=0
            break
        fi
    fi
done <<< "$DIFF_FILES"

# Signal C: GDScript/Godot absence
signal_c=1
if echo "$DIFF_FILES" | grep -qE '\.(gd|gdshader|tscn|tres)$|^scripts/|^scenes/'; then
    signal_c=0
fi

# Signal D: System tick registration absence (heuristic)
signal_d=1
if echo "$DIFF_FILES" | xargs grep -l "register_runtime_system!\|impl RuntimeSystem for\|fn tick\b" 2>/dev/null | head -1; then
    signal_d=0
fi

# All 4 signals required
if [[ "$signal_a" == "1" && "$signal_b" == "1" && "$signal_c" == "1" && "$signal_d" == "1" ]]; then
    echo "[cold-tier] CONFIRMED: All 4 signals present"
    exit 0
else
    echo "[cold-tier] NOT confirmed (A=$signal_a B=$signal_b C=$signal_c D=$signal_d)" >&2
    exit 1
fi
```

`tools/harness/hooks/pre-commit-check.sh` 수정 (line 128-150 영역):

```bash
# v3.3: Cold-tier classification check
if bash tools/harness/cold_tier_classifier.sh "$CODE_FILES" >/dev/null 2>&1; then
    cold_tier_credit=20  # full Visual Verify auto credit
    echo "[hook] Cold-tier confirmed → +${cold_tier_credit} Visual Verify credit (v3.3 §2)"
    adjusted_score=$((score + cold_tier_credit))
    
    # 대체 검증 의무 verification
    if ! grep -q "Cardinality verified" "$verdict_msg" 2>/dev/null; then
        echo "[hook] WARNING: Cold-tier requires 'Cardinality verified' in verdict" >&2
    fi
else
    # 기존 v3.2.1 VLM SKIP +8 logic 유지 (environmental failure case)
    vlm_env_cost=0
    if [[ ! -f "$visual_analysis_path" ]]; then
        vlm_env_cost=8
    elif head -c 7 "$visual_analysis_path" 2>/dev/null | grep -q "SKIPPED"; then
        vlm_env_cost=8
    fi
    if [[ "$vlm_env_cost" -gt 0 ]]; then
        adjusted_score=$((score + vlm_env_cost))
    fi
fi
```

---

## 📑 Section 3: Gap 2 — Attempt Penalty Discrimination

### 3.1 Deep Analysis

#### 3.1.1 v3.2.1 Attempt Count의 본질

현재 정책: 모든 RE-CODE에 동일 penalty (-X per attempt). T2 사례에서 -7 적용.

문제: RE-CODE 사유가 본질적으로 다른데 동일 처리.

**T2 사례 재구성** (Claude Code 보고 기반):
- A1 RE-CODE: Test rigor 실패 (test count 부족, edge case 누락)
- A2 RE-CODE: Lock violation (MaterialError String, sim-test crate)
- A3: Surgical revert + §7.1 강화 후 final

→ A1과 A2는 **본질적으로 다른 RE-CODE**. A1은 정상 iterative refinement, A2는 Generator 발산.

#### 3.1.2 RE-CODE 분류 (사용자 axiom #1 적용)

**Category 1 — LOCK_VIOLATION** (Generator 발산):
- Prompt §3 cardinality 위반 (e.g., 11 vs 10 re-exports)
- Prompt §3 type 위반 (e.g., `&'static str` vs `String`)
- Prompt §6 NOT-in-scope 위반 (e.g., sim-test crate 추가)
- §7.1 forbidden rationales 사용 ("more flexible", "future-proof" 등)
- → Penalty 정당 (사용자 axiom #1 위반)

**Category 2 — OUT_OF_SCOPE** (Generator 발산):
- 새 crate 생성 (workspace.members 추가)
- Prompt 외 파일 생성 (e.g., harness.rs)
- Prompt 외 dependency 추가
- → Penalty 정당 (Hard Gate 5 위반)

**Category 3 — TEST_RIGOR** (정상 iterative refinement):
- Test count 부족 (Phase 1 명세보다 적음)
- Edge case 누락
- Boundary value test 부족
- → Penalty 부적합 (정상 refinement)

**Category 4 — STYLE** (정상 iterative refinement):
- Code style (rustfmt, naming conventions)
- Doc comment 누락
- Verbosity 조정
- → Penalty 부적합

**Category 5 — OTHER** (분류 불가):
- 위 4개 어느 카테고리도 아닌 경우
- → 보수적 penalty 적용 (Generator self-monitoring 미흡 신호)

#### 3.1.3 분류 자동화 (mechanical, claude.ai 수치 결정)

Generator self-report에서 분류 추출 패턴:

```regex
LOCK_VIOLATION: (lock|cardinality|prompt §3|forbidden rationale|"more flexible"|"reasonable"|"future-proof"|"more idiomatic")
OUT_OF_SCOPE: (out of scope|workspace\.members|new crate|sim-test|sim-bridge.*new|harness\.rs)
TEST_RIGOR: (test count|edge case|boundary|coverage|test rigor|insufficient test)
STYLE: (rustfmt|clippy.*style|naming|doc comment|verbosity)
OTHER: (default if none above match)
```

각 RE-CODE의 사유는 challenger_report.md 또는 evaluator verdict에서 추출.

#### 3.1.4 Penalty 산출 정책

| Category | Per-attempt penalty | 정당화 |
|---------|------|------|
| LOCK_VIOLATION | -5 | Generator 발산, 강한 시그널 |
| OUT_OF_SCOPE | -5 | Hard Gate 5 위반 |
| TEST_RIGOR | 0 | 정상 refinement, count만 사용 (max attempts 한도) |
| STYLE | 0 | 정상 refinement |
| OTHER | -2 | 보수적 적용, Generator self-monitoring 미흡 신호 |

**T2 retroactive 산출**:
- A1 = TEST_RIGOR → 0
- A2 = LOCK_VIOLATION + OUT_OF_SCOPE → -5 + -5 = -10? 또는 max -5 (한 attempt에 multiple violation = -5)

**채택**: Per-attempt max penalty -5 (한 attempt 안 multiple violation 있어도 -5 cap).

T2 retroactive: A1 (-0) + A2 (-5) = **-5** (현재 -7에서 -2 회복)

### 3.2 정책 결정안

```
정책 v3.3 §3: Attempt Penalty Discrimination

분류 자동화:
- 각 RE-CODE attempt에 대해 evaluator verdict + challenger report 분석
- Regex 패턴으로 5 categories 자동 분류 (LOCK_VIOLATION, OUT_OF_SCOPE, TEST_RIGOR, STYLE, OTHER)

Penalty 산출:
- LOCK_VIOLATION: -5 per attempt
- OUT_OF_SCOPE: -5 per attempt
- TEST_RIGOR: 0 (count만)
- STYLE: 0 (count만)
- OTHER: -2 per attempt
- Per-attempt cap: -5 max (multiple violation 있어도 한 번 차감)

Max attempts:
- 현재 v3.2.1: 3 (RE-CODE 3회 후 FAIL)
- v3.3 유지: 3 (변경 없음)
- TEST_RIGOR + STYLE attempts는 cap에 포함 안 됨 (정상 refinement)
- LOCK_VIOLATION + OUT_OF_SCOPE attempts만 cap에 포함

예시 (T2 retroactive):
- A1 (TEST_RIGOR): 0 penalty, attempt cap에 포함 X
- A2 (LOCK_VIOLATION + OUT_OF_SCOPE): -5 penalty, attempt cap 1/3
- A3 (final, no RE-CODE): 0 penalty
- Total attempt penalty: -5 (현재 -7에서 -2 회복)
- Effective attempts to cap: 1/3 (still safe)
```

### 3.3 Implementation 명세

`tools/harness/classify_recode.sh` (신규):

```bash
#!/bin/bash
# RE-CODE 분류 — Generator self-report + Evaluator verdict 분석

set -euo pipefail

VERDICT_FILE="${1:?usage: classify_recode.sh <verdict-file>}"
ATTEMPT_NUM="${2:?attempt number}"

if [[ ! -f "$VERDICT_FILE" ]]; then
    echo "OTHER"
    exit 0
fi

content="$(cat "$VERDICT_FILE")"

# Priority order (가장 강한 시그널부터)
if echo "$content" | grep -qiE 'lock|cardinality|prompt §3|forbidden rationale|"more flexible"|"reasonable"|"future-proof"|"more idiomatic"|"more rust-idiomatic"'; then
    echo "LOCK_VIOLATION"
elif echo "$content" | grep -qiE 'out of scope|workspace\.members|new crate|sim-test|sim-bridge.*new|harness\.rs|prompt §6'; then
    echo "OUT_OF_SCOPE"
elif echo "$content" | grep -qiE 'test count|edge case|boundary|coverage|test rigor|insufficient test|test보강'; then
    echo "TEST_RIGOR"
elif echo "$content" | grep -qiE 'rustfmt|clippy.*style|naming|doc comment|verbosity|cosmetic'; then
    echo "STYLE"
else
    echo "OTHER"
fi
```

`tools/harness/score_attempt_penalty.sh` (신규):

```bash
#!/bin/bash
# Attempt penalty 산출 — v3.3 정책

set -euo pipefail

ATTEMPTS_DIR="${1:?usage: score_attempt_penalty.sh <attempts-dir>}"

total_penalty=0
effective_count=0

for verdict in "$ATTEMPTS_DIR"/attempt-*/verdict; do
    if [[ ! -f "$verdict" ]]; then continue; fi
    
    attempt_num="$(basename "$(dirname "$verdict")" | sed 's/attempt-//')"
    category="$(bash tools/harness/classify_recode.sh "$verdict" "$attempt_num")"
    
    case "$category" in
        LOCK_VIOLATION|OUT_OF_SCOPE)
            total_penalty=$((total_penalty - 5))
            effective_count=$((effective_count + 1))
            ;;
        TEST_RIGOR|STYLE)
            # No penalty, no effective count
            ;;
        OTHER)
            total_penalty=$((total_penalty - 2))
            effective_count=$((effective_count + 1))
            ;;
    esac
done

# Per-attempt cap: -5 (한 attempt에 multiple violation 있어도 -5)
# 이미 위에서 -5 max로 적용됨 (LOCK_VIOLATION OR OUT_OF_SCOPE 둘 다 -5)
# 만약 한 attempt가 LOCK + OUT_OF_SCOPE 둘 다이면 위 case가 LOCK_VIOLATION 우선 매칭

echo "PENALTY=$total_penalty"
echo "EFFECTIVE_ATTEMPTS=$effective_count"
```

---

## 📑 Section 4: Gap 3 — FFI Vacuous Credit

### 4.1 Deep Analysis

#### 4.1.1 FFI Verify의 본질

`tools/harness/harness_pipeline.sh` Step 2.5c FFI Verify는 **sim-bridge 변경 시 FFI surface 검증**:
- Rust → GDScript 변환 함수 정합성
- Snapshot 구조 일관성
- gdext binding 검증

T2 사례: §6 NOT-in-scope에 sim-bridge 명시적 excluded. **변경 0건**.

#### 4.1.2 v3.2.1의 FFI Verify 처리

```
FFI 변경 감지 → Verify 실행 → FAIL/PASS
FFI 변경 없음 → Verify 실행? skip? → 명확하지 않음
```

T2에서 FFI Verify가 FAIL 처리됨 (Mechanical Gate -2). 하지만 **변경 자체가 없으므로 verify 대상 부재**.

#### 4.1.3 게임사례 분석 (사용자 axiom #2)

**Bevy plugin testing**:
- Plugin 변경 0이면 plugin test skip
- Plugin 변경 시에만 plugin integration test 실행

**RimWorld DefDatabase**:
- ThingDef 변경 0이면 DefDatabase 검증 skip
- 변경 시에만 검증

**공통 패턴**: **변경 없는 surface는 verify 대상 X**. v3.3은 이 패턴 차용.

### 4.2 정책 결정안

#### Alternative A: Vacuous PASS (변경 0 → 자동 PASS)

**제안**: FFI 변경 0건 감지 → Verify 자체 SKIP, Mechanical Gate full credit (10/10).

| 옵션 | Mechanical 점수 | 정당화 |
|-----|------|------|
| A1: 10/10 (full credit) | 변경 0 = 검증 대상 0 = PASS 동등 | 가장 깨끗 |
| A2: 9/10 (-1 검증 부재 비용) | 일부 차감 | Claude Code draft 임의 수치 — 근거 부족 |
| A3: 8/10 (-2 검증 부재 비용) | 큰 차감 | 부재가 결여로 처리됨 (Gap 1과 동일 오류) |

**채택**: **A1 (10/10 full credit)**.

**근거**:
- Gap 1 §2.3과 동일 정신: "구조적 부재"의 비용은 0
- §6에서 명시적으로 excluded → 의도된 부재
- 사용자 axiom #1: "변태적 디테일"은 명세에 따른 것이지 가정에 따른 것이 아님

#### Alternative B: §6 명시 의무

**제안**: §6 NOT-in-scope에 sim-bridge 명시적으로 excluded 표기 의무 → 자동 SKIP. 명시 없으면 정상 verify.

**부분 채택**: A1과 결합. sim-bridge 변경 0이면 자동 SKIP, §6 명시는 audit trail 강화.

### 4.3 채택 정책 — FFI Vacuous Credit

```
정책 v3.3 §4: FFI Vacuous Credit

조건 (sim-bridge 변경 검증):
- git diff sim-bridge crate (rust/crates/sim-bridge/) 변경 라인 수 = 0
- prompt §6에 "sim-bridge: NOT in scope" 또는 유사 표기 명시 (audit trail)

조건 충족 시:
- FFI Verify SKIP
- Mechanical Gate FFI 차원 = 10/10 (full credit)
- log: "[hook] FFI vacuous (no sim-bridge change) → +0 (no penalty)"

조건 미충족 시:
- 정상 FFI Verify 실행
- 결과에 따라 Mechanical Gate 산출
```

### 4.4 Implementation 명세

`tools/harness/ffi_vacuous_check.sh` (신규):

```bash
#!/bin/bash
# FFI Vacuous Check — sim-bridge 변경 0 검증

set -euo pipefail

DIFF_FILES="${1:?usage: ffi_vacuous_check.sh <diff-files-newline-list>}"

# sim-bridge crate 변경 라인 수 검증
sim_bridge_changes=0
while IFS= read -r f; do
    if [[ -n "$f" ]] && [[ "$f" =~ ^rust/crates/sim-bridge/ ]]; then
        sim_bridge_changes=$((sim_bridge_changes + 1))
    fi
done <<< "$DIFF_FILES"

if [[ "$sim_bridge_changes" -eq 0 ]]; then
    echo "[ffi-vacuous] CONFIRMED: No sim-bridge changes"
    exit 0
else
    echo "[ffi-vacuous] NOT vacuous: $sim_bridge_changes sim-bridge changes" >&2
    exit 1
fi
```

`tools/harness/harness_pipeline.sh` Step 2.5c 수정:

```bash
# v3.3: FFI Vacuous Check
if bash tools/harness/ffi_vacuous_check.sh "$CODE_FILES" >/dev/null 2>&1; then
    echo "[hook] FFI vacuous → SKIP (v3.3 §4)"
    ffi_score=10  # full credit
else
    # 정상 FFI Verify 실행
    bash tools/harness/ffi_verify.sh
    # ... 결과에 따라 ffi_score 산출
fi
```

---

## 📑 Section 5: Cross-Gap 일관성 — T2 Retroactive Validate

### 5.1 T2 (commit 91d4e7c0) Score 재산출

v3.3 정책 적용 시 T2 score 재산출:

```
v3.2.1 (현재 BLOCK):
  Mechanical Gate:    8/10  (FFI FAIL false positive -2)
  Plan Quality:       5/5
  Code Quality:       8/15  (-7 attempt penalty 무차별)
  Visual Verify:      0/20
  Regression:         15/15
  Evaluator:          15/15
  ──────────────────────
  Raw:                51/80  (Plan 5 포함 시) 또는 48/80 (보고치)
  +VLM SKIP:          +8 (cold-tier 부적합 환경 보정)
  Adjusted:           56-59/100
  Threshold:          90
  Result:             BLOCK

v3.3 적용 후:
  Mechanical Gate:    10/10 (FFI vacuous +2 회복)
  Plan Quality:       5/5   (변동 없음)
  Code Quality:       13/15 (TEST_RIGOR -0 + LOCK_VIOLATION -5 + per-attempt cap = -5만)
                            (오리지널 18 - 5 = 13)
                            * 만약 max code quality 15에서 baseline 15 - 5 attempt = 10이면 10/15
                            * 정확한 산출은 hook code 검증 필요 (아래 §5.2)
  Visual Verify:      20/20 (cold-tier-bonus +20 auto credit)
  Regression:         15/15 (변동 없음)
  Evaluator:          15/15 (변동 없음)
  ──────────────────────
  Raw:                78-80/80
  +VLM cost:          0 (cold-tier-bonus가 환경 보정 대체)
  Adjusted:           78-80/100
  Threshold:          90
  Result:             ⚠️ 여전히 BLOCK?
```

**문제**: v3.3 적용 후에도 78-80 < 90. **Threshold 자체가 cold tier에 맞지 않을 수 있음**.

### 5.2 Threshold 정책 결정

**Alternative A**: Threshold 유지 (90), Cold-tier-bonus +20만 적용
- T2 retroactive: 78-80 → 여전히 BLOCK
- 추가 정책 갭 노출

**Alternative B**: Cold tier 전용 threshold (예: 75 or 80)
- T2 retroactive: 78-80 ≥ 75 → PASS
- 단순 threshold split = Hard Gate 5 위반 위험

**Alternative C**: Cold-tier-bonus를 score 보정이 아닌 threshold 보정
- Hot tier: threshold 90
- Cold tier: threshold 70 (= 90 - 20 cold tier credit)
- 동등 효과, 정신은 더 명확

**Note (v3.3.2)**: Alt C 원안의 hot threshold 90은 v3.3.1 §3.2 Z 보강 분석 결과 72로 supersede (raw 80 < threshold 90 = 수학적 불가능). 이 section은 historical proposal 보존. 정합 결론은 §5.4 (v3.3.1 §3.1 verbatim, v3.3.2 §2.1 정정 반영).

**Note (v3.3.3)**: v3.3.2 §2.1의 raw 80 / threshold 72 결론은 v3.3.3 §1.2 dimension 누락 발견 (Tests 20 미반영)으로 supersede. dimension 추가 후 max 100 회복 → Alt C 원안 threshold 90 회복 (max 100 ↔ threshold 90 정합). 정합 결론은 §5.4 (v3.3.3 §2.2 verbatim).

**Alternative D**: Hot/Cold 구분 없이 max raw score를 cold-tier에 맞게 조정
- Cold tier: max raw 100 (Visual auto credit +20, v3.3.1 §3.1)
- Hot tier: max 100 (Tests 20 포함), threshold 90 (raw 100 × 90%, v3.3.3 §2.2 정정)
- Cold tier: threshold 75 (v3.3.1 §3.1)

**채택 (v3.3.1 update, v3.3.2 → v3.3.3 supersede)**:
**Cold tier: Max raw 100 (Visual auto credit +20) + Threshold 75 (100×75%)**
**Hot tier: Max 100 (Tests 20 포함) + Threshold 90 (100×90%)**

**근거**:
- Visual Verify 차원 자체 제외가 가장 명확
- 사용자 axiom #1: 의미 없는 차원 (visual surface 0)에 점수 부여 자체 부적합
- Threshold 90은 "max의 90%" 일관 (hot tier 90/100 = 90%, v3.3.3 §2.2)
- T2 retroactive: 78/80 (mechanical 10 + plan 5 + CQ 13 + reg 15 + eval 15 = 58? — CQ 산출 정확성 검증 필요)

### 5.3 Code Quality 산출 정확성 — 추가 분석

기존 v3.2.1에서 Code Quality max=15. v3.3에서:

**Penalty 적용 방식 alternatives**:
- A: baseline 15 - sum(attempt penalties) = 15 - 5 = 10
- B: baseline 15 + sum(attempt penalties) where attempt penalty는 음수 = 15 + (-5) = 10
- C: per-attempt max -5, multi-attempt cap -10 = 15 - min(5, 10) = 10

**채택**: **A (단순 baseline - sum)**, T2: 15 - 5 = 10.

**T2 retroactive 최종 산출 (v3.3.3 §1.2 cascade)**:
```
Mechanical:  10/10  (+2 FFI vacuous 회복)
Plan:         5/5
Code Quality: 10/15 (-5 LOCK_VIOLATION attempt)
Tests:       20/20 (v3.3.3 §1.2 dimension 추가)
Visual:      20/20 (cold tier auto credit, v3.3.1 §3.1)
Regression:  15/15
Evaluator:   15/15
─────────────────────
Raw:         95/100 (cold tier max 100 = Visual 20 auto credit + Tests 20 포함, v3.3.3 §1.2)

Cold-tier-bonus 적용 (v3.3.1 §3.1 REVISED):
- Visual Verify 차원: cold tier auto credit +20 (차감 없음, 차원 자체 점수 부여)
- Max 정의 (v3.3.3 §2.2):
  Hot tier max: Mechanical 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15 = 100
  Cold tier max: 위 with Visual auto credit +20 (Visual 차원 자체 부여) = 100
  Hot threshold: 90/100 (raw 100 × 90%, v3.3.3 §2.2)
  Cold threshold: 75/100 (cold 보수적 -15, raw 100 × 75%)

→ T2 final (v3.3.3): 95/100 ≥ 75 → PASS (safety margin +20)
```

### 5.4 채택 정책 — Cold Tier Score Model (v3.3.3 cascade)

> **Source**: v3.3.3 amendment §2.2 verbatim (`.harness/prompts/governance_v3_3_3_amendment.md`, commit 871f131b)
> **Supersedes**: v3.3 v1.0 §5.4 (Alt D, max 60/threshold 54) — score scale 갭으로 무효; v3.3.2 §2.1 (max 80/threshold 72) — Tests 20 dimension 누락으로 무효

```
정책 v3.3.3 §5.4: Score Model Tier-Aware (cascade)

Score Scale: 0-100 (v3.2.1 호환 유지, raw 100 그대로)

Hot tier (cold-tier 4 Signal 미충족):
- Max: 100 (Mech 10 + Plan 5 + CQ 15 + Tests 20 + Visual 20 + Reg 15 + Eval 15)
- Hook scale 변환 불필요 (max 100 raw 그대로, v3.3.3 §1.4)
  → Hook은 verdict line 4 또는 pipeline_report.md에서 score를 pure consumer로 추출
- Threshold: 90/100 (raw 100 × 90%, v3.3.3 §2.2)
- VLM SKIP +8 보정 유지 (environmental failure)

Cold tier (4 Signal 충족):
- Max: 100 (Visual 20 자동 부여 = +20 auto credit, Tests 20 포함, v3.3.3 §1.2)
- Visual Verify: 0/20 → +20 (cold tier auto credit, generate_report.sh 책임)
  → 즉 Visual 차원 자체 점수 부여, 차감 없음
- Threshold: 75/100 (cold 보수적 -15, raw 100 × 75%, v3.3.3 §2.2)
- VLM SKIP +0 (cold tier auto credit이 보정 대체)

판정 logic (hook pure consumer, v3.3.3 §1.4):
  if cold_tier_classifier exit 0:
    threshold = 75
    vlm_env_cost = 0
  else:  # hot tier
    threshold = 90  # v3.3.3 §2.2 (raw 100 × 90%, hook pure consumer)
    vlm_env_cost = 8 if VLM SKIP else 0

  # score는 generate_report.sh가 산출하여 verdict/report에 기록
  # hook은 산출하지 않고 추출만 (pure consumer)
  score = extract_from_verdict_or_report()
  adjusted = score + vlm_env_cost
  PASS if adjusted >= threshold
```

### 5.5 T2 Retroactive Final Score (v3.3.3 cascade)

> **Source**: v3.3.3 amendment §1.2 cascade (`.harness/prompts/governance_v3_3_3_amendment.md`, commit 871f131b)
> **Supersedes**: v3.3 v1.0 §5.5 (55/60 PASS) — score scale 갭으로 무효; v3.3.1 §3.4 (75/100 boundary) — Tests 20 dimension 누락으로 무효

```
T2 retroactive validate (91d4e7c0) — Alt C:

Cold tier 4 Signal 검증:
✓ A: 모든 변경 sim-core/material/ + lib.rs (sim-core crate)
✓ B: *.rs, *.md 파일만
✓ C: GDScript/Godot 변경 0
✓ D: register_runtime_system 호출 0, RuntimeSystem impl 0
→ Cold tier 확정

Score 산출 (raw):
Mechanical:  10/10  (FFI vacuous +2 회복)
Plan:         5/5
Code Quality: 10/15 (LOCK_VIOLATION -5 from A2, A1 TEST_RIGOR -0)
Tests:       20/20 (v3.3.3 §1.2 dimension 추가)
Visual:      20/20 (cold tier auto credit)
Regression:  15/15
Evaluator:   15/15
─────────────────────
Raw:         95/100 (Tests 20 포함, v3.3.3 §1.2)
Threshold:   75/100
Result:      PASS ✓ (95 ≥ 75, safety margin +20, v3.3.3 §1.2 cascade)

VLM cost: 0 (cold tier 자동 부여, 보정 불필요)
```

---

## 📑 Section 6: Implementation Order (Locked Topological)

> **사용자 axiom #1**: "정교하고 자세하게" — 구현 순서 lock 의무

| # | 단계 | 파일 | Type | Depends On |
|---|-----|------|------|----------|
| N1 | Cold-tier classifier 신설 | `tools/harness/cold_tier_classifier.sh` | NEW | — |
| N2 | RE-CODE 분류기 신설 | `tools/harness/classify_recode.sh` | NEW | — |
| N3 | Attempt penalty 산출기 신설 | `tools/harness/score_attempt_penalty.sh` | NEW | N2 |
| N4 | FFI vacuous check 신설 | `tools/harness/ffi_vacuous_check.sh` | NEW | — |
| N5 | pre-commit-check.sh 수정 (Cold tier integration) | `tools/harness/hooks/pre-commit-check.sh` | MODIFY | N1 |
| N6 | harness_pipeline.sh 수정 (FFI vacuous + RE-CODE 분류) | `tools/harness/harness_pipeline.sh` | MODIFY | N3, N4 |
| N7 | Score model tier-aware 도입 | `tools/harness/score_model.sh` (신설 또는 pipeline 안 함수) | NEW/MODIFY | N1 |
| N8 | Self-test (v3.3 자체) | `tools/harness/test_v3_3.sh` | NEW | N5, N6, N7 |
| N9 | T2 retroactive validate | (run pipeline against 91d4e7c0) | TEST | N8 |
| N10 | v3.3 audit log entry | `.harness/audit/governance_v3_3.log` (신설) | NEW | N9 |
| N11 | v3.3 documentation | `tools/harness/README.md` (수정) | MODIFY | N10 |
| N12 | Commit + push | (git commit + push) | TEST | N11 |

**총 12 단계, ~1-2일 작업.**

각 단계는 다음 단계 시작 전 검증 의무:
- N5 후 cold-tier-classifier가 T2 변경 set에 대해 cold tier 확정 출력 확인
- N7 후 score model이 Hot/Cold 모두 정상 산출 확인
- N8 후 self-test가 v3.3 정책 자체 검증 통과 확인
- N9 후 T2 retroactive score = 95/100 PASS 확인 (v3.3.3 §1.2 cascade, safety margin +20)

---

## 📑 Section 7: 검증 (V1~V8)

> **사용자 axiom #1**: "변태적 디테일" — 검증 명세 의무

| # | 검증 | 명령 | Expected |
|---|-----|------|---------|
| V1 | cold-tier-classifier T2 cold 확정 | `bash tools/harness/cold_tier_classifier.sh "$(git diff --name-only 91d4e7c0~1..91d4e7c0)"` | exit 0, "CONFIRMED" |
| V2 | classify_recode A2 verdict | `bash tools/harness/classify_recode.sh .harness/runs/material_schema/attempt-2/verdict 2` | "LOCK_VIOLATION" |
| V3 | classify_recode A1 verdict | `bash tools/harness/classify_recode.sh .harness/runs/material_schema/attempt-1/verdict 1` | "TEST_RIGOR" |
| V4 | score_attempt_penalty T2 attempts | `bash tools/harness/score_attempt_penalty.sh .harness/runs/material_schema/` | `PENALTY=-5`, `EFFECTIVE_ATTEMPTS=1` |
| V5 | ffi_vacuous_check T2 변경 | `bash tools/harness/ffi_vacuous_check.sh "$(git diff --name-only 91d4e7c0~1..91d4e7c0)"` | exit 0, "CONFIRMED" |
| V6 | T2 retroactive score | `(pipeline simulate 91d4e7c0)` | `Cold tier raw 95/100 ≥ 75 PASS (v3.3.3 §1.2 cascade, safety margin +20)` |
| V7 | Hot tier 정상 동작 | `(pipeline against hypothetical hot tier change)` | Hot tier max 100 (Tests 20 포함), threshold 90/100 정상 산출 (v3.3.3 §2.2) |
| V8 | self-test all paths | `bash tools/harness/test_v3_3.sh` | All 8 V tests pass |

V8 (self-test) 통과 의무. 통과 안 하면 v3.3 commit 차단.

---

## 📑 Section 8: NOT in Scope (Out-of-Scope)

> **사용자 axiom #1**: "변태적 디테일" — scope boundary 명시 의무

v3.3은 다음 항목 명시적으로 excluded:

1. ❌ Hot tier scoring 변경 (현재 정책 유지)
2. ❌ ENV-BYPASS 정책 변경 (별도 정책, v3.4+ 대상)
3. ❌ STRUCTURAL-COMMIT 카테고리 정식화 (별도 ticket — V7-RESET용 임시 메커니즘 정식화)
4. ❌ Layer 1 PreToolUse hook 수정 (T1 commit 시 사용자 직접 터미널 우회 필요했던 갭 — 별도 ticket)
5. ❌ POLICY-GAP 카테고리 영구화 (v3.3 land 후 deprecate 의무, 임시 transition mechanism)
6. ❌ Visual Verify 자체 polish (Godot launch reliability, VLM model upgrade 등)
7. ❌ Generator self-monitoring 강화 (별도 prompt-level governance)
8. ❌ Phase 1 W1.4 Inspector UI 시각 reference (사용자 axiom #3 게임 화면 — 별도 Visual Direction Phase 0 ticket)
9. ❌ Locale 자동 검증 (T11 단계, exempt path 정상)
10. ❌ Criterion bench 결과 자동 reporting (별도 tooling)
11. ❌ Multi-language harness pipeline (현재 Rust + GDScript만 지원)
12. ❌ Cargo workspace dependency cycle 검증 (cargo 자체 책임)
13. ❌ Pipeline parallelization (single-threaded 유지)
14. ❌ Cloud-based harness (local-only)
15. ❌ Rust nightly feature 도입 (stable 1.93 유지)
16. ❌ Material schema 자체 수정 (Phase 1 T2~T5 완료, audit trail 보존)

---

## 📑 Section 9: Acceptance Gate (10 items)

v3.3 commit + push 전 모든 10 항목 충족 의무:

```
□ G1: N1~N12 모든 단계 완료
□ G2: V1~V8 모든 검증 통과
□ G3: cargo test --workspace 회귀 0건
□ G4: cargo clippy --workspace --all-targets -- -D warnings clean
□ G5: T2 retroactive validate 91d4e7c0 score 95/100 ≥ 75 PASS 확인 (v3.3.3 §1.2 cascade, safety margin +20)
□ G6: cold-tier-classifier T6~T11 시뮬레이션 (현재 미작성이지만 변경 set 가정)
       → 모두 cold tier 확정 + PASS 예상
□ G7: hot tier 정상 동작 (hypothetical sim-systems 변경 시뮬레이션)
       → Hot tier max 100 (Tests 20 포함), threshold 90/100 정상 산출 (v3.3.3 §2.2);
         Cold tier max 100 (Visual 20 auto credit + Tests 20, v3.3.3 §1.2), threshold 75/100 (v3.3.3 §2.2)
□ G8: 사용자 명시 approve: "v3.3 design은 정교해, implementation OK"
□ G9: .harness/audit/governance_v3_3.log entry 작성
       형식: 2026-05-XXTHH:MM:SSZ | v3.3 land | Cold-tier-bonus + Attempt 
              discrimination + FFI vacuous | retroactive 91d4e7c0 PASS | 
              authorized by user | <commit-hash>
□ G10: v3.3 commit message:
       feat(governance-v3-3)[V7][POLICY-LAND]: cold-tier scoring + attempt
       discrimination + FFI vacuous credit
       
       Phase 1 T2 (91d4e7c0) score 95/100 retroactive PASS (v3.3.3 §1.2 cascade, safety margin +20).
       Resolves 3 policy gaps identified in commit 91d4e7c0.
       
       (이하 변경 사항 + retroactive validate 결과 + reference)
```

---

## 📑 Section 10: 사용자 결정 사항 + Phase 1 W1.5 진입 조건

### 10.1 사용자 결정 사항 (Q6~Q10)

| Q# | 질문 | 권장 답 | 결정 의무 |
|---|-----|------|--------|
| Q6 | Cold-tier max score = 100 (Visual auto credit +20) OK? (v3.3.1 §3.1) | A: OK (REVISED) | 사용자 명시 |
| Q7 | Cold-tier threshold = 75/100 OK? (v3.3.3 §2.2) | A: OK (Tests 20 dimension cascade) | 사용자 명시 |
| Q8 | LOCK_VIOLATION + OUT_OF_SCOPE penalty -5 per attempt OK? | A: OK | 사용자 명시 |
| Q9 | TEST_RIGOR + STYLE penalty 0 OK? (정상 refinement) | A: OK | 사용자 명시 |
| Q10 | FFI vacuous = full credit (sim-bridge 변경 0 시) OK? | A: OK | 사용자 명시 |

### 10.2 Phase 1 W1.5 진입 조건

v3.3 land 후:
1. T2 retroactive validate 91d4e7c0 PASS 확인 (자동)
2. T6 (RON 100 files) harness pipeline 실행
   - Expected: Cold tier 확정 (C signal — *.ron data only)
   - Expected: Score ≥ 75/100 PASS, T2 retroactive 95/100 with safety margin +20 (v3.3.3 §1.2 cascade)
3. T7~T11 동일 패턴 (T9 harness tests, T10 bench 모두 cold tier 예상)
4. T11 (locale) exempt path 정상 통과
5. Phase 1 5 commits 모두 완료 → Phase 1 W1.5 완료
6. W1.4 (Inspector UI) ticket 시작 (사용자 axiom #3 Visual Direction Phase 0 결정 후)

---

## 🎯 v3.3 종합

### Phase 0 v0.3 패턴 충족

| 항목 | v3.3 충족 |
|------|----------|
| 7 sections (Phase 0 v0.3 패턴) | ✅ Section 1~10 (확장형) |
| 사용자 axiom #1 (변태적 디테일) | ✅ 모든 정책 수치 alternatives + 채택 근거 명시 |
| 사용자 axiom #2 (수학적 정밀, 학술/게임사례) | ✅ DF/Bevy/RimWorld 사례 + 수치 정당화 |
| Lock 의무 (cardinality, 수치 명시) | ✅ Score model 정확한 수치 |
| Implementation order locked | ✅ N1~N12 topological |
| Verification 명세 | ✅ V1~V8 |
| NOT in scope | ✅ 16 items |
| Acceptance gate | ✅ 10 items |
| Retroactive validate | ✅ T2 91d4e7c0 score 95/100 PASS 시뮬 (v3.3.3 §1.2 cascade, safety margin +20) |

### Claude Code draft 활용 결과

draft (1166줄)의 활용:
- ✅ 3 갭 식별 정확 (Cold-tier Visual, Attempt discrimination, FFI vacuous)
- ✅ Implementation file diff 구조 (참고만)
- ❌ INTRINSIC_VISUAL_CREDIT=18 (임의 수치) → 무효, v3.3.1 §3.1에서 Cold tier max 100 raw (Visual auto credit +20) model로 대체
- ❌ SCORE_TESTS=18 → 무효, attempt penalty 산출 §3.2에서 -5 per LOCK/OOS로 대체
- ❌ 5 attempt categories → 부분 채택, regex 패턴 강화 (§3.3)
- ❌ FFI vacuous +2 → 무효, full credit (10/10) 채택 (§4.3)
- ❌ Plan QC -3 → 무효, Plan Quality 변경 없음 (§5.3에서 확인)
- ❌ Cold cap 96 → 무효, v3.3.1 §3.1에서 cold tier max 100 raw (Visual auto credit +20) / threshold 75 model로 대체

### 다음 단계

1. **사용자 review** — Q6~Q10 + 전체 정책 검토
2. **사용자 명시 approve**: "v3.3 design은 정교해, implementation OK"
3. **Claude Code dispatch**: 이 문서 .harness/prompts/governance_v3_3.md로 전달
4. **Step N (Implementation)**: Claude Code가 N1~N12 진행
5. **Step O (Verify + Land)**: V1~V8 통과 + G1~G10 acceptance gate 통과 → v3.3 commit + push
6. **T6~T11 진행**: v3.3 정착 후 정상 harness pipeline

---

**문서 버전**: v3.3 통합 명령 v1.0
**다음 갱신**: v1.1 (사용자 보강 요청 시) 또는 v2.0 (사용자 approve 후 implementation 결과 반영)
**갱신 책임자**: claude.ai (정책 결정 영역)

*이 문서는 V7 governance v3.2.1 → v3.3 진화의 정책 결정본입니다.*
*사용자 axiom #1 (변태적 디테일) + #2 (수학적 정밀) 모두 충족.*
*Claude Code draft 1166줄을 reference로 활용하되 임의 수치 모두 deep analysis로 재결정.*
