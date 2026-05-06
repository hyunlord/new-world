# V7 Hook Governance v3.3.4 — Direct-Implementation Lane Formalization

> **Type**: Architecture / Policy Implementation (claude.ai 영역)
> **Trigger**: T6.6.4 BLOCK 발견 — STRUCTURAL-COMMIT procedure만 존재, mechanism 부재
> **Parent**: v3.3.3 amendment (871f131b) + V.4 cascade + N6 cascade
> **Status**: Active implementation, formalizes direct-implementation lane
> **Implementation impact**: Hook patch (STRUCTURAL marker 인식) + authorize script 신설 + T6.6 first user

---

## 📋 메타 정보

| 항목 | 내용 |
|------|------|
| 작성일 | 2026-05-06 |
| 작성자 | claude.ai (워크플로우 boundary 준수) |
| 버전 | v3.3.4 amendment v1.0 |
| Trigger | T6.6.4 BLOCK 발견 (post-V.4 cascade direct-implementation lane 부재) |
| 이전 amendment | v3.3.3 (871f131b) + V.4 cascade (39e68a86) + N6 cascade (957a9503) |

### claude.ai 자가 학습 (4번째)

이번이 v3.3 작성 후 4번째 amendment. 누적 패턴:
- v3.3.1: Score scale 갭 (claude.ai hook scale 가정 부정확)
- v3.3.2: Hot threshold stale (§3.1 vs §3.2 모순 미식별)
- v3.3.3: Architecture mismatch (Hook = consumer, generate_report = producer)
- **v3.3.4 (이번)**: Direct-implementation lane 부재 (post-V.4 환경 미인지)

근본 원인: 사용자 axiom #2 (수학적 정밀, 학술/게임사례, 실측) 위반 누적.

근본 차이점 (v3.3.3 → v3.3.4):
- v3.3.3까지: amendment 정정 (text + spec 변경)
- v3.3.4: implementation 신설 (코드 신설 + Hook lane 추가)
- 즉 v3.3.4는 governance 진화 + 코드 진화 둘 다

---

## 📑 Section 1: 진짜 Architecture 발견 (T6.6.4 BLOCK)

### 1.1 발견 사실 (실측, T6.6.4 BLOCK 시점)

```
Claude Code grep 검증 결과:
- pre-commit-check.sh bypass paths 2개:
  a. APPROVED verdict file in .harness/reviews/ (set by harness_pipeline.sh)
  b. .harness/audit/env_bypass_active marker (set by authorize_env_bypass.sh)
- STRUCTURAL-COMMIT marker 인식 logic 부재
- Hook은 .harness/audit/structural_commits.log 미참조

T2-T5 (91d4e7c0 2026-05-05) + T1 (77764531 2026-05-04) 성공 이유:
- V.4 cascade pre-date (post-V.4 hook 부재)
- 당시 hook은 less strict (단일 SCORE_THRESHOLD=90)
- 직접 implementation 통과 가능했음

Post-V.4 환경 (2026-05-06+):
- Hook V.4.4 (0c5d88a0) tier branching strict
- Hook V.4.5 (62031050) cold tier auto credit
- Hook N6.x (4680378d/2f90ffed/13c5814c) FFI vacuous + RE-CODE + penalty
- 모두 "harness pipeline verdict" base
- 직접 implementation lane 부재 (T6.6.4에서 발견)
```

### 1.2 v3.3.4-C 갭 (정확한 표현)

```
정확한 갭 (T6.6.4 발견):
- STRUCTURAL-COMMIT은 procedure만 존재 (.harness/audit/structural_commits.log 형식 정의)
- 실행 mechanism 부재 (Hook 인식 logic, authorize script, marker file 모두 없음)
- T6.6처럼 직접 implementation 작업 (Codex CLI Generator/Evaluator 통하지 않음)이 정합 commit 불가

→ v3.3.4-C 정식화: lane 정식화 + Hook patch + authorize script
```

### 1.3 다른 deferred items (T6.6 진행 중 발견, post-T6 audit)

```
v3.3.4 amendment scope 외 (deferred to v3.3.5 or v3.4):
- v3.3.4-A: properties.rs distribution ↔ definition.rs natural_in semantic duplication
            (T6.6.2 verify 발견)
- v3.3.4-B: "5 카테고리" v0.1 design wording vs 6-variant reality (5 base + Mod(u8))
            (T6.6.2 verify 발견)
- v3.3.4-D: per-attempt cap 의미 명확화 (각 attempt 단독 -5 누적, 글로벌 cap 아님)
            (W3.4 발견)
- v3.3.4-E: attempts/ subdirectory 구조 명세 (W3.1 directory mismatch 해결 패턴)
            (W3.1 발견)
- v3.3.4-F: SCORE_CODE attempt-aware base 공식 명세 (15/11/8)
            (W3.1.6 발견)

이번 v3.3.4는 -C만 정식화. A, B, D, E, F는 post-T6 audit (또는 별도 amendment).
```

---

## 📑 Section 2: 채택 정책 — STRUCTURAL-COMMIT Lane 정식화

### 2.1 STRUCTURAL-COMMIT lane 정의

```
v3.3.4 §2.1 STRUCTURAL-COMMIT Lane

목적: 직접 implementation 작업 (Generator/Evaluator pipeline 외)이 정합 commit하는 lane.

조건 (5개 모두 만족 의무):
1. Cold tier only (sim-core, sim-data 등 — Hot tier 거부)
2. cargo test --workspace PASS 의무
3. cargo clippy --workspace --all-targets -- -D warnings: 0 warnings 의무
4. cold_tier_classifier 4 Signals 모두 통과 의무
5. 사용자 명시 authorization (authorize_structural_commit.sh --authorized-by)

Mechanism:
- Marker file: .harness/audit/structural_commit_active
- Marker metadata: authorized_by, reason, cargo_test_result, clippy_result, cold_tier, timestamp, staged_files
- Authorize script: tools/harness/authorize_structural_commit.sh
- Hook 인식: pre-commit-check.sh (3번째 bypass lane)
- Audit log: .harness/audit/structural_commits.log (이미 존재)
- Manual verification log: .harness/audit/manual_verification.log (이미 존재)
- One-time use (marker consumed after commit)

Commit subject 강제:
- [STRUCTURAL] tag 의무 (audit 가시성)
- Hook 미포함 시 ERROR + commit 거부

Hot tier 거부:
- authorize script가 cold_tier_classifier 검사 후 Hot 시 즉시 deny
- "STRUCTURAL-COMMIT denied: Hot tier requires harness pipeline" 메시지
```

### 2.2 Hot tier 거부 이유 (명시)

```
v3.3.4 §2.2 Hot tier 거부 근거

Hot tier (sim-systems, sim-engine):
- runtime simulation logic
- ECS systems registration
- agent behavior
- 직접 implementation 위험 ↑

위험 사항:
- 사이드 이펙트 (다른 system 영향)
- runtime 회귀 (visual artifact, perf 저하)
- visual verification 의무 (VLM 검증 필요)
- multi-agent integration test 의무

→ Hot tier는 정식 harness pipeline (Generator/Evaluator + Visual Verify) 의무
→ STRUCTURAL-COMMIT 우회 거부
```

### 2.3 Cold tier 적용 근거

```
v3.3.4 §2.3 Cold tier 적용 근거

Cold tier (sim-core, sim-data 등):
- Pure logic (no runtime, no visual)
- ECS components, data definitions
- Static computation (deterministic)
- cargo test 충분 (visual 의존 X)

직접 implementation safe 사유:
- Visual artifact 0 (VLM 검증 불필요)
- Runtime side effects 0 (system 등록 X)
- Determinism 보장 (f64 + serde)
- cargo test + clippy 충분 검증

T6.6 사례:
- by_category 인덱스 (registry.rs 확장)
- granite.ron sample (data file)
- 3 integration tests
- 71 tests + clippy clean = governance 검증 동등 수준
```

---

## 📑 Section 3: Implementation 명세

### 3.1 Hook Patch (pre-commit-check.sh)

```bash
# v3.3.4 §3.1: STRUCTURAL-COMMIT lane (direct implementation)
STRUCTURAL_MARKER="$HARNESS_DIR/audit/structural_commit_active"
if [[ -f "$STRUCTURAL_MARKER" ]]; then
    # Required metadata 검증
    if grep -q "^cargo_test_result=PASS" "$STRUCTURAL_MARKER" && \
       grep -q "^clippy_result=CLEAN" "$STRUCTURAL_MARKER" && \
       grep -q "^cold_tier=CONFIRMED" "$STRUCTURAL_MARKER" && \
       grep -q "^authorized_by=" "$STRUCTURAL_MARKER"; then
        report_step "STRUCTURAL-COMMIT" "AUTHORIZED" \
            "Direct-implementation lane (v3.3.4 §3.1)"
        # [STRUCTURAL] tag 강제
        if [[ -n "$COMMIT_MSG_FILE" ]] && [[ -f "$COMMIT_MSG_FILE" ]]; then
            FIRST_LINE=$(head -1 "$COMMIT_MSG_FILE")
            if [[ ! "$FIRST_LINE" =~ \[STRUCTURAL\] ]]; then
                echo "ERROR: STRUCTURAL-COMMIT requires [STRUCTURAL] tag in commit subject."
                exit 1
            fi
        fi
        # One-time use (marker consumed)
        rm -f "$STRUCTURAL_MARKER"
        exit 0
    else
        echo "ERROR: STRUCTURAL-COMMIT marker missing required metadata."
        exit 1
    fi
fi
```

위치: pre-commit-check.sh의 기존 ENV-BYPASS 검사 다음 (3번째 bypass lane).

### 3.2 Authorize Script (authorize_structural_commit.sh)

```bash
#!/usr/bin/env bash
# Cold tier only, 4 prerequisite check, dual log write, marker file 작성
# Usage: bash authorize_structural_commit.sh --reason "..." --authorized-by "..."

set -euo pipefail

# 1. Cargo test prerequisite
# 2. Clippy prerequisite
# 3. cold_tier_classifier prerequisite
# 4. Marker file + audit logs 작성

# (전체 코드는 dispatch Section A.3 W2.3 참조)
```

위치: tools/harness/authorize_structural_commit.sh (chmod +x)

### 3.3 Audit Log Format

```
.harness/audit/structural_commits.log entry 형식:

<ISO 8601 timestamp> | STRUCTURAL-COMMIT authorized | <reason> | <authorized_by> | 
cargo_test=PASS, clippy=CLEAN, cold_tier=CONFIRMED | <commit hash if known>

예시:
2026-05-06T12:34:56Z | STRUCTURAL-COMMIT authorized | T6.6 Material loader | kwan hyeon | 
cargo_test=PASS, clippy=CLEAN, cold_tier=CONFIRMED | abc1234
```

```
.harness/audit/manual_verification.log entry 형식:

<ISO 8601 timestamp> | STRUCTURAL-COMMIT | <authorized_by> | <reason> | 
cargo_test=PASS, clippy=CLEAN, cold_tier=CONFIRMED | staged: <count> files
```

---

## 📑 Section 4: 사용자 결정 사항 (E1~E5)

```
□ E1: "v3.3.4 amendment design은 정교해, lane 정식화 OK" 사용자 명시 approve
□ E2: D-β3 채택 (Hook patch first + T6.6 commit)
□ E3: STRUCTURAL-COMMIT marker mechanism 정식화
□ E4: 사용 conditions (5개 모두 만족 의무)
□ E5: Cold tier only (Hot tier 명시 거부)
```

---

## 🎯 v3.3.4 종합

### 정정 범위 요약

| 발견 | v3.3.4 정정 |
|-----|-------------|
| STRUCTURAL-COMMIT mechanism 부재 | Hook patch + authorize script 신설 |
| Hot tier 직접 implementation 위험 | 거부 + harness pipeline 의무 |
| audit trail mechanism 부재 | dual log (manual_verification + structural_commits) |
| Commit subject 가시성 | [STRUCTURAL] tag 강제 |

### Implementation Impact

```
v3.3.4 cascade 4 commits:
- W1: amendment register
- W2: Hook patch + authorize script
- W3: T6.6 first user (post-V.4)
- W4: audit log + progress

T6.1~T6.5 (100 RON files) 정합 처리 가능 (각 sub-step STRUCTURAL-COMMIT lane).
```

### claude.ai 자가 학습 명시

이번 v3.3.4는 4번째 amendment. 누적 패턴 인지:
- v3.3.1: Score scale (가정 부정확)
- v3.3.2: Hot threshold stale (모순 미식별)
- v3.3.3: Architecture mismatch (실측 부족)
- v3.3.4: Direct-implementation lane 부재 (post-V.4 환경 미인지)

**근본 원인 일관**: 사용자 axiom #2 (실측, 학술/게임사례) 위반 누적.

**자가 개선 의무**: 향후 governance 문서 작성 시 grep + 실제 코드 실측 의무. 추정/가정 최소화.

---

**문서 버전**: v3.3.4 amendment v1.0
**다음 갱신**: v3.3.5 (deferred A/B/D/E/F 처리) 또는 v3.4 (Phase 1 land 후)
**갱신 책임자**: claude.ai (architecture/policy 영역)

*Direct-implementation lane 정식화 (T6.6.4 BLOCK 해결).*
*Hook patch + authorize script + dual log mechanism.*
*Cold tier only, prerequisite 4 check, [STRUCTURAL] tag 강제.*
*v3.3.4-A/B/D/E/F deferral (post-T6 audit).*
