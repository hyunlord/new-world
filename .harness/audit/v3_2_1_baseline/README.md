# V3.2.1 Baseline Backup

V7 governance v3.2.1 정확한 baseline 보존 (audit 가치).
v3.3 implementation 시작 시점: 2026-05-05 (post-commit `3fad6134`).

## 내용

- `pre-commit-check.sh.bak`: Layer 2 hook v3.2.1 (Claude Code PreToolUse)
- `harness_pipeline.sh.bak`: pipeline orchestration v3.2.1

## Rollback 절차 (필요 시)

```bash
cp .harness/audit/v3_2_1_baseline/pre-commit-check.sh.bak \
   tools/harness/hooks/pre-commit-check.sh

cp .harness/audit/v3_2_1_baseline/harness_pipeline.sh.bak \
   tools/harness/harness_pipeline.sh
```

V3.3 land 후에도 영구 보존 (audit chain).

## 관련 문서

- `.harness/prompts/governance_v3_3.md` — v3.3 official ticket (claude.ai)
- `.harness/audit/policy_gap.log` — T2-T5 POLICY-GAP-V3.3 authorization log
- `.harness/audit/v7_progress.md` — Phase 1 W1.2~W1.6 deferred until v3.3 lands
