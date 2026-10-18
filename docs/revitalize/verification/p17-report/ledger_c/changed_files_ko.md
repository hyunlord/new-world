## 변경된 파일 목록

커밋 원장은 C `21d2b478a81d55ac138f0a60b830e06719ec0af3`까지의 **Pass17 25개 커밋·순변경 605개 경로**를 담는다. 아래 경로 수는 각 커밋의 부모 대비 개수이므로 중복을 포함한다. 생성된 수백 개 요약 파일은 반복 나열하지 않고 전체 경로 원장으로 연결한다.

[전체 경로 JSON](commit_files.json) · [커밋별 상세 원장](commit_files.md). JSON SHA256 `87f14b5483bb8030c919759c73c0cbff0b54389329e73bebded9e06428471c08`; 상세 Markdown SHA256 `13491db6c46fd76bfce52823a3af65e968ec72bb6394dd682c281128e4e306b6`.

P16 부모 `2c079df265ef7c16baf8ee5bb119591091b5c3eb`(19개 경로)는 기준 이력으로만 보존하며 Pass17 작업 수에서 제외한다. 최초 검증 기준은 `adfdecfece79ab019964729319389fd602685fab`다.

| 전체 SHA | Part | 경로 수 | 의도·주요 경로 | 전체 목록 |
|---|---:|---:|---|---|
| [e6bde09c5859bf8ba715d240f59092bd7d45eadf](https://github.com/hyunlord/new-world/commit/e6bde09c5859bf8ba715d240f59092bd7d45eadf) | 0 | 1 | 승인된 실행 계약 보존<br>`docs/revitalize/PASS17_GOAL.md` | [원장](commit_files.md#commit-e6bde09c5859bf8ba715d240f59092bd7d45eadf) |
| [4f10de7534a213607e9bcd15e1af6c4f3cc24d68](https://github.com/hyunlord/new-world/commit/4f10de7534a213607e9bcd15e1af6c4f3cc24d68) | 0 | 1 | 기존 v20 동작 핀 고정<br>`rust/crates/sim-test/tests/harness_revitalize_pass17_frozen.rs` | [원장](commit_files.md#commit-4f10de7534a213607e9bcd15e1af6c4f3cc24d68) |
| [bc43c8ddf9a536afd8fb2ae38acec9c6d1beebf2](https://github.com/hyunlord/new-world/commit/bc43c8ddf9a536afd8fb2ae38acec9c6d1beebf2) | 0 | 11 | 기존 규칙의 선택 기능 비활성 유지<br>`rust/crates/sim-core/src/rules.rs`<br>`rust/crates/sim-core/src/rules/routine.rs` | [원장](commit_files.md#commit-bc43c8ddf9a536afd8fb2ae38acec9c6d1beebf2) |
| [c47ad7ec688c19995f4856b6c0bfd34aed58591b](https://github.com/hyunlord/new-world/commit/c47ad7ec688c19995f4856b6c0bfd34aed58591b) | 0 | 19 | 엄격 비교·잠금·게이트 도구<br>`tools/revitalize/identity_p17.sh`<br>`tools/revitalize/quiet_lock_p17.sh`<br>`docs/revitalize/verification/p17-0/` | [원장](commit_files.md#commit-c47ad7ec688c19995f4856b6c0bfd34aed58591b) |
| [65ec44c90d49dffa0908c0458f3c8ecec371bd96](https://github.com/hyunlord/new-world/commit/65ec44c90d49dffa0908c0458f3c8ecec371bd96) | 0 | 14 | 변경 전 I1–I4 기준 증거<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/PLAN_PASS17.md`<br>`docs/revitalize/probe/p17-0_base/` | [원장](commit_files.md#commit-65ec44c90d49dffa0908c0458f3c8ecec371bd96) |
| [8de4073adbd152ecdf91ab28a1a348085e87a20f](https://github.com/hyunlord/new-world/commit/8de4073adbd152ecdf91ab28a1a348085e87a20f) | 1 | 69 | 일상 행동·관측·브리지·평가 구현<br>`rust/crates/sim-core/src/components/activity.rs`<br>`rust/crates/sim-systems/src/runtime/decision/routine/mod.rs`<br>`docs/revitalize/verification/p17-1/` | [원장](commit_files.md#commit-8de4073adbd152ecdf91ab28a1a348085e87a20f) |
| [753b51942c20489538faac59c15c5ac45ab48075](https://github.com/hyunlord/new-world/commit/753b51942c20489538faac59c15c5ac45ab48075) | 1 | 19 | 튜닝 전 실제 관측 QA<br>`tools/revitalize/evaluate_pass17_read.py`<br>`tools/revitalize/test_evaluate_pass17.py`<br>`docs/revitalize/verification/p17-1/` | [원장](commit_files.md#commit-753b51942c20489538faac59c15c5ac45ab48075) |
| [9a0bd85470f4ddbe26d51b8c5062107bf5c1f371](https://github.com/hyunlord/new-world/commit/9a0bd85470f4ddbe26d51b8c5062107bf5c1f371) | 2 | 10 | Stage A 최초 튜닝·기본값 고정<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/probe/p17-a_tuning/r1/` | [원장](commit_files.md#commit-9a0bd85470f4ddbe26d51b8c5062107bf5c1f371) |
| [6ad99666deab7acde6cf34d66de20867df639eaa](https://github.com/hyunlord/new-world/commit/6ad99666deab7acde6cf34d66de20867df639eaa) | 2 | 25 | 관측기 수정 전 동일성 증거 보존<br>`docs/revitalize/ROUTINE.md`<br>`docs/revitalize/probe/p17-2_a_identity/` | [원장](commit_files.md#commit-6ad99666deab7acde6cf34d66de20867df639eaa) |
| [50a73474e06d3a9abbb535729e9c7e1e134954b7](https://github.com/hyunlord/new-world/commit/50a73474e06d3a9abbb535729e9c7e1e134954b7) | 2 | 16 | 사망과 같은 틱의 일상 이벤트 관측 보정<br>`rust/crates/sim-test/src/probe/routine_observer.rs`<br>`rust/crates/sim-test/src/probe/pass17_activity.rs`<br>`docs/revitalize/verification/p17-2_observer_fix/` | [원장](commit_files.md#commit-50a73474e06d3a9abbb535729e9c7e1e134954b7) |
| [7a6f74b4b0913ff27597de38678bf6f89f48e1a0](https://github.com/hyunlord/new-world/commit/7a6f74b4b0913ff27597de38678bf6f89f48e1a0) | 2 | 53 | 보정 후 스모크·동일 매개변수 튜닝 증거<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/ROUTINE.md`<br>`docs/revitalize/probe/p17-a_tuning/r1-observerfix/` | [원장](commit_files.md#commit-7a6f74b4b0913ff27597de38678bf6f89f48e1a0) |
| [09eb5c2671d86334683b2273115635c34580dcea](https://github.com/hyunlord/new-world/commit/09eb5c2671d86334683b2273115635c34580dcea) | 2 | 75 | 최종 A 동일성·성능 FAIL 보존<br>`docs/revitalize/PERF_BUDGET.md`<br>`docs/revitalize/PLAN_PASS17.md`<br>`docs/revitalize/perf/p17-a/` | [원장](commit_files.md#commit-09eb5c2671d86334683b2273115635c34580dcea) |
| [000d3706c6463dd19cd6b1efce20ae60a7acaa9f](https://github.com/hyunlord/new-world/commit/000d3706c6463dd19cd6b1efce20ae60a7acaa9f) | 3 | 2 | Stage A 평가 규칙 사전 등록<br>`docs/revitalize/METRICS.md`<br>`docs/revitalize/DECISIONS.md` | [원장](commit_files.md#commit-000d3706c6463dd19cd6b1efce20ae60a7acaa9f) |
| [44aebafa5acf5668d21418f611d8f0ce3a5d13d3](https://github.com/hyunlord/new-world/commit/44aebafa5acf5668d21418f611d8f0ce3a5d13d3) | 3 | 29 | Stage A 평가 판정·I5 기준 보존<br>`docs/revitalize/METRICS.md`<br>`docs/revitalize/ROUTINE.md`<br>`docs/revitalize/probe/p17-a_eval/` | [원장](commit_files.md#commit-44aebafa5acf5668d21418f611d8f0ce3a5d13d3) |
| [47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21](https://github.com/hyunlord/new-world/commit/47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21) | 4 | 69 | 가족·친족·가구·브리지·평가 구현<br>`rust/crates/sim-core/src/components/family.rs`<br>`rust/crates/sim-systems/src/runtime/agent/pairing.rs`<br>`docs/revitalize/verification/p17-4/` | [원장](commit_files.md#commit-47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21) |
| [7082258c3c6e5bcfe07f69558bd3ba82451c13e7](https://github.com/hyunlord/new-world/commit/7082258c3c6e5bcfe07f69558bd3ba82451c13e7) | 4 | 46 | 실제 가족 런타임·추적 QA<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/PLAN_PASS17.md`<br>`docs/revitalize/probe/p17-4_smoke/` | [원장](commit_files.md#commit-7082258c3c6e5bcfe07f69558bd3ba82451c13e7) |
| [c7ec0b5a53d46958ecb9562a9e2b38efe9a687ee](https://github.com/hyunlord/new-world/commit/c7ec0b5a53d46958ecb9562a9e2b38efe9a687ee) | 5 | 28 | Stage B 1차 튜닝·300 고정·F5 FAIL 보존<br>`docs/revitalize/FAMILY.md`<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/probe/p17-b_tuning/r1/` | [원장](commit_files.md#commit-c7ec0b5a53d46958ecb9562a9e2b38efe9a687ee) |
| [f0410d398d72045b69e621d0d244395d85286a21](https://github.com/hyunlord/new-world/commit/f0410d398d72045b69e621d0d244395d85286a21) | 5 | 10 | Stage B 가족·분리 수동 검증과 F5 실패 원인 기록<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/verification/p17-b_tuning_r1/README.md` | [원장](commit_files.md#commit-f0410d398d72045b69e621d0d244395d85286a21) |
| [d2cde5e9d50d2650ec5f9b2745b636116295a754](https://github.com/hyunlord/new-world/commit/d2cde5e9d50d2650ec5f9b2745b636116295a754) | 5 | 85 | Stage B 최종 F7 동일성·F8 성능 증거 보존 및 Part 5 종료<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/PLAN_PASS17.md`<br>`docs/revitalize/perf/p17-b/` | [원장](commit_files.md#commit-d2cde5e9d50d2650ec5f9b2745b636116295a754) |
| [43f22e81f120a755de33712b02a62469c67ab840](https://github.com/hyunlord/new-world/commit/43f22e81f120a755de33712b02a62469c67ab840) | 6 | 3 | Stage B 평가 규칙·분모·부트스트랩 및 평가기 해시 사전 등록<br>`docs/revitalize/METRICS.md`<br>`docs/revitalize/DECISIONS.md`<br>`docs/revitalize/verification/p17-b_prereg/` | [원장](commit_files.md#commit-43f22e81f120a755de33712b02a62469c67ab840) |
| [32e2a009966d6d429a62282897241510400dbc45](https://github.com/hyunlord/new-world/commit/32e2a009966d6d429a62282897241510400dbc45) | 6 | 7 | 평가 실행 중·분석 결과 전 F3 추정량 불일치 고지와 독립 F7/F8 감사 보존<br>`docs/revitalize/verification/p17-b_prereg/f3_review_notice.md`<br>`docs/revitalize/verification/p17-5_b_identity/final_audit.md`<br>`docs/revitalize/perf/p17-b/` | [원장](commit_files.md#commit-32e2a009966d6d429a62282897241510400dbc45) |
| [9a06b11167984116071c444e3540479f74059539](https://github.com/hyunlord/new-world/commit/9a06b11167984116071c444e3540479f74059539) | 6 | 2 | 분석 결과 전 전체 형제 모집단 누락까지 F3 수정 범위 고지<br>`docs/revitalize/verification/p17-b_prereg/f3_review_notice.md`<br>`docs/revitalize/DECISIONS.md` | [원장](commit_files.md#commit-9a06b11167984116071c444e3540479f74059539) |
| [e353f18f5b523b4198fd785b44c5ad962b15576d](https://github.com/hyunlord/new-world/commit/e353f18f5b523b4198fd785b44c5ad962b15576d) | 6 | 35 | 수정 전 등록 분석·F1 FAIL·원본 평가 보존<br>`docs/revitalize/probe/p17-b_eval/original_registered/eval.md`<br>`docs/revitalize/METRICS.md`<br>`docs/revitalize/verification/p17-b_eval/` | [원장](commit_files.md#commit-e353f18f5b523b4198fd785b44c5ad962b15576d) |
| [51e1629a059ada0aa6ea9c8434b348d7587cc9aa](https://github.com/hyunlord/new-world/commit/51e1629a059ada0aa6ea9c8434b348d7587cc9aa) | 6 | 16 | F3 두 모집단을 원문대로 수정하고 회귀 검증<br>`tools/revitalize/evaluate_pass17_family_pairs.py`<br>`tools/revitalize/test_evaluate_pass17_stage_b.py`<br>`docs/revitalize/verification/p17-f3_correction/` | [원장](commit_files.md#commit-51e1629a059ada0aa6ea9c8434b348d7587cc9aa) |
| [21d2b478a81d55ac138f0a60b830e06719ec0af3](https://github.com/hyunlord/new-world/commit/21d2b478a81d55ac138f0a60b830e06719ec0af3) | 6 | 41 | 동일 데이터 F3 재분석·독립 감사·F1 FAIL·Part 6 종료 보존<br>`docs/revitalize/probe/p17-b_eval/eval.md`<br>`docs/revitalize/probe/p17-b_tuning/r1-f3-corrected/eval.md`<br>`docs/revitalize/verification/p17-f3_correction/` | [원장](commit_files.md#commit-21d2b478a81d55ac138f0a60b830e06719ec0af3) |

`7082258c3c6e5bcfe07f69558bd3ba82451c13e7`은 Part4 QA다. 필수 제목 접두사와 푸시 전 원격/브랜치 표시 누락은 행정적 일탈로 보존했으며 이력을 수정하지 않았다.

F7/F8 최종 증거는 `d2cde5e9`, 독립 감사는 `32e2a009`에 보존했다. F3의 두 모집단 문제는 **평가 시작 후·분석 결과 전**에 고지했다(`32e2a009`, `9a06b111`). 원래 등록 분석을 `e353f18f`에 보존한 뒤 `51e1629a`로 평가기만 수정하고 같은 원시 데이터의 재분석·독립 검증을 C에 기록했다. 수정 분석은 변경 없는 사전 등록 분석이 아니다. Stage A R5 FAIL, Stage B F1 FAIL, 튜닝 F5 FAIL은 유지한다.

### 실행 기록

실제 집계 RUNLOG: `docs/revitalize/DECISIONS.md`의 Pass17 `### RUNLOG` 절(872행부터 이어지는 Part 표). 이 원장의 출처는 `21d2b478a81d55ac138f0a60b830e06719ec0af3:docs/revitalize/DECISIONS.md`, SHA256 `70979ea662da7b95df1a9462213b66069ffb7148ae7f5bc22247904467ed5016`다. [구조화 시간 원장](part_times.json).

| 구간 | 시작 UTC | 종료 UTC | 소요 |
|---|---|---|---|
| Part0 | 2026-10-15 06:01:55 | 2026-10-15 06:36:24 | 00:34:29 |
| Part0→1 작업 전환 | 2026-10-15 06:36:24 | 2026-10-15 06:37:47 | 00:01:23 |
| Part1 | 2026-10-15 06:37:47 | 2026-10-15 07:15:30 | 00:37:43 |
| Part2 | 2026-10-15 07:15:30 | 2026-10-15 08:31:35 | 01:16:05 |
| Part3 | 2026-10-15 08:31:35 | 2026-10-15 09:20:29 | 00:48:54 |
| Part4 | 2026-10-15 09:20:29 | 2026-10-15 09:59:08 | 00:38:39 |
| Part5 | 2026-10-15 09:59:08 | 2026-10-15 10:45:31 | 00:46:23 |
| Part6 | 2026-10-15 10:45:31 | 2026-10-15 11:32:37 | 00:47:06 |
| Part7 | N/A | N/A | 원문에 정의되지 않음 |
| Part8 | 2026-10-15 11:32:37 | 진행 중 | 미확정 |

**83초 전환은 작업 시간이며 quiet-lock 대기가 아니다.** 완료된 Part0–6 자체 시간은 **5:29:19**, 전환을 포함한 06:01:55→11:32:37은 **5:30:42**다. Part7은 빠진 기록이 아니라 원문에 없는 구간이다. Part8의 종료·최종 게이트·게시 완료 시각은 아직 기록하지 않는다. 측정 잠금 보유15.0초/16.8초는 실제 측정 시간으로 대기와 구분한다.

작업 트리 스냅샷 245개 경로는 커밋 원장에서 제외하고 보존했다. `.gitignore`, `.ignore`, 기존 P6–P16 로그와 현재 병행 작성 중인 문서/게이트 산출물의 전체 경로는 JSON `working_tree_snapshot`에 있다. skip-worktree `.claude/settings.json`도 수정하지 않았다.

### 보고서 자체 SHA·종료 시각의 외부 기록

이 원장은 보고서 커밋 전의 확정된 C까지 포함한다. 보고서 커밋 R의 자체 SHA, R의 마지막 게이트와 원격 게시 완료 시각은 보고서 안에 미래 사실로 넣지 않는다. 외부 `/Users/rexxa/work/p17-data/final_publication_pass17.json`과 최종 PR/메시지의 게시 부록에 기록한다. R을 포함한 최종 커밋 원장도 외부 새 스냅샷으로 보존한다. [게시 순서와 영수증 계획](publication_commands.md).
