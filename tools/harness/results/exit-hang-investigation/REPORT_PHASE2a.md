# Generator exit-hang PHASE 2a 적용 보고

> Repo: hyunlord/new-world · Branch: lead/main · base HEAD: 256322dd
> 변경 파일: `tools/harness/harness_pipeline.sh` 1개 (+ exit-hang-investigation/ 산출물). rust/·scripts/ 0줄.

## 적용 의도
PHASE 1에서 확정된 **증폭기**(`| tee` held-fd 블록 + 단일-pid SIGKILL이 손자 잔존)를 제거하고
(F1, F2), 다음 실제 Generator 실행(2-2 Gather 재개)에서 hang을 포착할 **watchdog**(S0)를 부착.
F3(완료-신호 조기 종료)는 트리거(H1-고아 vs H2) 미확정 상태라 PHASE 2b로 보류 — watchdog 캡처가
F3의 완료-판정 기준을 정의함.

## 변경 내용

**F1 — Generator 호출 `| tee` → `> file 2>&1`** (`run_generator`, ~L1075).
- `2>&1 | tee LOG` (+ `gen_rc=${PIPESTATUS[0]}`) → `> LOG 2>&1` (+ `local gen_rc=0; … || gen_rc=$?`).
- **중요 발견:** 스크립트는 `set -euo pipefail`이고 `run_generator`는 plain 호출(L2290/2447)이라
  set -e가 활성. 원본 `| tee` 형태는 타임아웃(142)에서 **파이프라인에서 set -e로 즉시 abort** →
  L1084의 `GENERATOR_TIMEOUT` 마커 기록과 `die`에 **도달조차 못 함**. 이것이 PHASE 1에서 관측된
  "지목 4사례 모두 142-마커 부재"의 원인. `|| gen_rc=$?` 형태만이 타임아웃 분기를 도달가능하게
  만들어 graceful die + 마커 기록을 비로소 작동시킴 (프롬프트 문자 그대로의 `local gen_rc=$?`는
  set -e로 abort되어 불가 — 측정으로 확인 후 안전형으로 교정).

**F2 — `run_with_timeout` perl 경로 프로세스-그룹 kill** (~L274).
- child: `exec @ARGV` → `setpgrp(0,0); exec @ARGV` (child를 그룹 리더로).
- ALRM 핸들러: `kill "TERM",$pid` / `kill "KILL",$pid` → `kill "TERM",-$pid` / `kill "KILL",-$pid`
  (음수 pid = 프로세스 그룹 전체).
- **데드라인 경로만 변경.** 정상완료 경로(`waitpid` 반환 → `exit ($?>>8)`)는 그대로 → 11개 콜러
  모두 타임아웃 안 나면 동일 동작 (exit-code 전파 검증: stub `exit 7` → rc 7).
- GNU `timeout` 경로는 호스트에 부재(perl fallback 사용). 설치 시 그룹킬 패리티는 flagged 후속.

**S0 — step-0 live-capture watchdog** (`_exithang_watchdog` + `_exithang_find_claude`, run_generator 감쌈).
- Generator 호출 직전 백그라운드 기동, 호출 반환 직후 `kill`+`wait`로 teardown (성공·die 양 경로 커버,
  `|| true`로 set -e 회피). 캡처는 `tools/harness/results/exit-hang-investigation/live_capture_<ts>_attempt<N>/`에만.
- hang 의심(프로세스 생존 + rust/ 90s 무변동 + 120s 경과) 시 ps/lsof/sample **1회** 스냅샷 →
  다음 실제 hang이 H1-고아(main 종료, 손자가 파이프 보유) vs H2(main이 network read/API 소켓에서
  블록) 판별 실험이 됨. `_exithang_find_claude`는 perl 래퍼를 제외하고 node-claude pid를 타겟
  (PHASE 1 deep_probe 교훈). watchdog 내부 `set +e`로 pgrep/lsof/find의 비정상 종료가 watchdog를
  죽이지 않게 함. 빠른 종료 시 fallback `outcome.txt` 기록.

## 검증 결과
- **F2 그룹킬 (mech_f2_test.sh, M4 재실험):** SIGTERM 무시 손자(동일 그룹) 대상 —
  **pre-F2 survivors=1**(손자 누수, pid 49243), **post-F2 survivors=0**(그룹 회수). **F2 PASS.**
  (setsid로 자기 세션 만드는 손자는 그룹 이탈 → 잔존; 실제 claude 자손은 setsid 안 함 → 커버. 알려진 edge.)
- **F1 redirect (verify_f1_s0.sh):**
  - A1 stub>3s 데드라인 → gen_rc=142, **타임아웃 분기 도달 ✓** (마커+die 작동).
  - A2 stub exit0 → gen_rc=0, 로그 기록("HELLO_FROM_STUB", 16B).
  - B 실제 light `claude --agent harness-generator -p 'reply DONE'` → gen_rc=0, 로그 non-empty("DONE", 5B).
- **watchdog teardown (B):** 잔존 `_exithang_watchdog` 프로세스 **NONE ✓**, `outcome.txt` 기록됨
  ("completed gen_rc=0" — 5초 종료라 첫 10s 폴 전에 끝나 fallback 경로; 분 단위 실 Generator는 정상 폴).
- **공유 인프라 비회귀:** `test_pipeline_infra.sh` **12 passed / 0 failed (exit 0)**.
- **exit-code 전파:** post-F2 정상완료 경로 stub `exit 7` → captured rc=7 (byte-equiv 확인).
- **git:** `harness_pipeline.sh` 1개만 수정 + investigation 산출물. **rust/·scripts/ 0줄.** 누수 프로세스 없음.

## 변경 파일
- `tools/harness/harness_pipeline.sh` (F1 L~1075 / F2 L~274 / S0 watchdog fns + run_generator wiring)
- `tools/harness/results/exit-hang-investigation/`: `mech_f2_test.sh`+`mech_f2_result.txt`,
  `verify_f1_s0.sh`+`verify_f1_s0_result.txt`, `REPORT_PHASE2a.md`

## 미해결 / 다음
- **트리거(H1-고아 vs H2) 여전히 미확정** — 다음 실제 Generator 실행(2-2 Gather 재개)에서 watchdog가
  `live_capture_*/capture.txt` + `sample_*.txt`로 포착. lsof에 network/API 소켓 + sample이 network
  read면 H2(F3 필요), main 종료 후 손자가 fd 보유면 H1-고아(F1이 이미 해소), wait4면 child-reaping.
- **F3는 그 캡처 후 PHASE 2b로 지시** (측정 없이 조기종료 적용 금지 — 출력 잘림 위험).
- GNU `timeout` 그룹킬 패리티: flagged 후속 (현재 호스트 무관).

## Governance chain
256322dd → (PHASE 2a, harness infra: F1+F2+S0)
