# Generator exit-hang 진단 보고 (PHASE 1 — 측정 전용, 수정 없음)

> Repo: hyunlord/new-world · Branch: lead/main · HEAD: 256322dd
> 작성: 2026-06-14 · 대상: `tools/harness/harness_pipeline.sh` `run_generator()` / `run_with_timeout()`
> sim 코드(`rust/`, `scripts/`) 변경 0줄. `harness_pipeline.sh` 변경 0줄 (계측은 독립 스크립트로 수행, 원본 미수정).

---

## TL;DR (먼저 결론)

- **증폭 메커니즘은 확정(결정론적 재현):** `… 2>&1 | tee LOG`의 `| tee`가 stdout/stderr 파이프
  write-end을 **누군가 한 명이라도** 쥐고 있으면 EOF를 못 받아 무한 대기한다. `> file 2>&1`
  (리다이렉트)로 바꾸면 같은 상황에서도 즉시 종료한다 → **`| tee`가 정확히 증폭기**다.
- **`run_with_timeout` 단일-pid kill은 손자 프로세스를 살려둔다(확정):** perl 핸들러가
  `$pid`(직속 claude)만 죽이고 프로세스 그룹을 안 죽여서, 데드라인에 SIGKILL이 떨어져도
  detached 손자는 생존한다 → H4 사실로 확인.
- **H3(stdin) 기각:** `< /dev/null`을 붙여도 합성 부하는 그대로 5~10초에 깨끗이 종료.
- **H1-via-Bash-tool-background 기각:** 에이전트가 Bash 툴로 백그라운드 자식(`sleep 45 &`)을
  남겨도 파이프라인은 14초에 정상 반환했고 `sleep 45`(PID 96960)는 그 뒤에도 생존 →
  **Claude Code Bash 툴이 백그라운드 자식의 fd를 격리**하므로 "에이전트가 빌드를 백그라운드로
  돌려서" 가설은 성립하지 않음.
- **현재(2026-06 시점) 잔존 hang의 트리거는 PHASE 1에서 미확정.** 합성 부하는 모든 조합에서
  5~31초에 정상 종료(=flush+exit)하여 다분(multi-minute) hang을 재현하지 못했고, 실제 Generator는
  `rust/`를 편집하므로(금지) 돌릴 수 없으며, 4개 지목 사례의 아티팩트는 모두 **복구 후 잔여물**
  (0-byte 로그)이라 H1(고아 무한 대기) vs H2(claude 자체 미종료)를 가르지 못함.
- **이전 두 수정이 놓친 "잔존 hang"의 정체와 일치:** count-guard(`414d585c`/`c65be4eb`)는 빌드/테스트
  비용을, I-Phase-A(`6b47926d`, Stop-hook)는 force-continue 루프를 각각 제거한 *기여자*였고,
  본 잔존 hang은 그 둘과 별개다. **여기서 H2를 단정하면 세 번째 "기여자만 잡고 잔존 놓침"을 반복**하게
  되므로, 트리거는 라이브 hang을 잡아야만 확정 가능하다(PHASE 2 step 0).

---

## 조사 의도

4연속 sim-core/sim-systems 피처(death-visual, slice 2-1, slice 2-2, membership-model-reform)에서
Generator 단계가 동일 양상으로 멈췄다: 작업(소스 편집 + gen_result)은 디스크에 끝까지 쓰였는데
프로세스가 종료되지 않고, `2>&1 | tee`가 `GENERATOR_TIMEOUT_SECONDS`(기본 1800s)까지 막혔다가
perl SIGKILL → `die "Generator hung"`. 피처당 ~30분 손실 + 2~3시간 수동 복구. 매 sim 피처마다
재발하므로 측정으로 **원인의 위치**를 먼저 특정하는 것이 목적(가설 점프 금지).

후보(가정하지 말고 측정으로 in/out):
- **H1** — claude의 자손이 살아남아 stdout/stderr 파이프 fd를 쥠 → tee가 EOF 못 받음.
- **H2** — claude 프로세스 자체가 종료 시점에 막힘(백그라운드 flush/telemetry/MCP·API 소켓 미정리/node 이벤트루프 미배수).
- **H3** — stdin 열림 → `-p`에도 불구하고 입력 대기.
- **H4** — `run_with_timeout` 단일-pid kill이 자손을 살려둠(데드라인에 H1을 가중).

---

## 재현

PHASE 1 제약상 두 갈래로 재현:

1. **메커니즘 결정론적 재현 (`mech_test.sh`, claude API 미사용)** — `run_with_timeout`의 perl
   분기(이 호스트엔 GNU `timeout` 부재 — 확인됨, 항상 perl 경로)를 그대로 복사하고, "작업을 끝내고
   즉시 종료하지만 fd 1/2를 물려받은 백그라운드 손자를 남기는" worker를 `| tee` / `> file` /
   짧은 데드라인으로 돌림. **반복 가능, 100% 결정론적.**

2. **라이브 재현 (`live_probe.sh`, `deep_probe.sh`)** — 파이프라인과 **동일한** 호출
   (`CLAUDECODE="" ; unset CLAUDE_CODE_ENTRYPOINT ; run_with_timeout … claude --agent
   harness-generator -p … --dangerously-skip-permissions --output-format text [2>&1 | tee | > file | < /dev/null]`)
   을 백그라운드로 띄우고, 로그 증가 멈춤 시점(작업완료) vs 종료 시점, 프로세스 트리, lsof, sample을 폴링.
   합성 프롬프트(파일 쓰기 + `cargo --version` 등)는 **모든 조합에서 정상 종료** → 실제 다분 hang은
   합성으로 재현 안 됨(아래 미해결 참조).

> 실제 Generator를 그대로 돌리는 것이 가장 충실한 재현이나, 그것은 `rust/`/`scripts/`를 편집하므로
> PHASE 1 금지조항(“No file under rust/ or scripts/ modified”)에 걸려 **수행 불가**. 이 한계가 트리거
> 미확정의 근본 이유다.

---

## 측정 결과 (M1~M7)

| 측정 | 케이스 | 관측 | 아티팩트 |
|---|---|---|---|
| **M3a** (det.) | `run_with_timeout worker \| tee` , worker가 fd 물린 손자 남기고 즉시 exit | worker <1s exit, **파이프라인 30s 대기**(손자 죽을 때까지 tee 블록) | `mech_result.txt`, `mech_tee.log` |
| **M3b** (det.) | 동일 worker, `> file 2>&1` | **1s** 종료(리다이렉트는 물린 fd를 기다리지 않음) | `mech_result.txt`, `mech_redir.log` |
| **M4** (det.) | 3s 데드라인, worker가 `setsid sleep 40` detached 손자 생성 | perl rc=142(SIGKILL 발동), **`sleep 40` 손자 생존**(단일-pid kill) | `mech_result.txt`, `mech_kill.log` |
| **M7-light** | `claude -p "DONE"` (agent/tool 없음) | ~5-6s 종료, 완료→종료 gap **1s** | `live_light.probe.txt` |
| **M7-heavy** | `claude --agent harness-generator` + echo/ls/cargo | ~11s(moderate)~31s(cold cargo) 종료, gap 2-3s | `live_tee.probe.txt`, `deep_tee.probe.txt` |
| **M1/M2** | deep_probe, 작업완료 순간 node-claude 스냅샷 | **자손 0개**, fd **1·2·4·10이 동일 tee 파이프**(`0xfe…e83c`)를 가리킴, 소켓 grep 0 | `deep_tee.probe.txt` |
| **M2-bg** | 에이전트가 `nohup sleep 45 &` 남김, `\| tee` | 파이프라인 14s 정상 반환, **`sleep 45`(PID 96960) 이후에도 생존** → Bash 툴이 자식 fd 격리 | `live_tee.probe.txt` |
| **M5** (H3) | `< /dev/null` | ~10s 정상 종료(stdin 무관) | `live_devnull.probe.txt` |
| **M6** | hung 프로세스 syscall | 합성 hang 미발생; perl wrapper는 `Perl_pp_waitpid → __wait4`(정상 대기); node-claude는 hang 안 해 sample 무의미 | `live_tee.sample.txt` |
| **M7-volume** | 30개 순차 Bash 툴콜(파일은 /tmp만), 99초 지속 세션 | logsize=0 유지(버퍼링) 후 **정상 flush+종료**, gap **1s** → **세션 길이/툴콜 볼륨 단독으로는 hang 재현 안 됨** | `vol.probe.txt`, `vol.log` |

추가 아티팩트 증거(과거 실제 run):
- **4개 지목 사례 모두 `generator_log_attempt1.txt` = 0 byte**, gen_result에 `GENERATOR_TIMEOUT`
  마커 **없음**. 전체 results 트리에서 142-마커는 `harness-count-guard-speedup` 1건뿐.
- `--output-format text`는 **stdout을 종료 직전 한 번에 flush** (합성 run에서 작업완료까지 logsize=0
  유지 확인). ∴ **0-byte 로그 = claude가 정상 종료/flush에 도달 못함** (H1·H2 공통, 구분 불가).
- 과거 *완료된* 로그들(material_schema, food-economy, p12-beta2 등)은 마지막 줄이 전부 Stop-hook
  서사("The stop hook completed cleanly…")로 끝남 — 그러나 이들은 **I-Phase-A 이전(pre-2026-05-30)
  잔여물**이고, `HARNESS_SUBAGENT=1`(L13, stop-check.sh L15-16 exit 0)로 이미 무력화된 force-continue
  경로의 흔적. **4개 지목 사례(post-fix)는 0-byte라는 다른 시그니처** → Stop-hook은 *이전 기여자*이지
  현재 트리거 아님.
- `.claude/settings.json` 훅은 PreToolUse(pre-commit-check)·Stop(stop-check) 2개뿐이고 **둘 다
  백그라운드 프로세스를 안 띄움** → 훅발 H1-고아 기각.

---

## 확정된 근본 원인

**완전 확정(결정론적·아티팩트 근거):**
1. **증폭기 = `| tee`** (M3a vs M3b). 파이프 write-end을 한 fd라도 쥐고 있으면 tee 무한 대기,
   리다이렉트는 무관. `| tee`가 1800s 대기를 *만든다*.
2. **`run_with_timeout` 단일-pid kill = H4 사실** (M4). 데드라인 SIGKILL이 직속 claude만 죽이고
   프로세스 그룹을 안 죽여 자손 생존 → 자손이 파이프를 계속 쥐면 SIGKILL 후에도 tee가 안 풀림.

**기각:**
- **H3(stdin)** — `< /dev/null` 무영향.
- **H1-via-Bash-tool-background** — Bash 툴이 백그라운드 자식 fd를 격리(M2-bg).
- **Stop-hook force-continue** — `HARNESS_SUBAGENT=1`로 이미 수정됨; 현재 0-byte hang의 원인 아님.

**미확정(정직하게):**
- **현재 잔존 hang의 트리거** — claude-node가 작업완료 후 *왜* 종료/flush에 도달 못하는가(H2),
  혹은 깨끗이 exit한 뒤 *어떤 자손*이 파이프를 쥐어 tee가 무한대기 하는가(H1-고아). 둘을 가르는
  결정적 증거(=라이브 hang 순간의 claude-node 생존 여부 + 파이프 보유자 lsof + sample)를
  PHASE 1에서 확보 못함. 합성 부하는 **모든 조합(light/moderate/cargo-cold/bg-child/devnull/redir/
  30툴콜-99초 볼륨)에서 ≤101초에 정상 종료** → 트리거는 합성으로 재현 불가.
  - 아티팩트 단서는 오히려 **엇갈림**: 사용자 서술("1800s에 perl SIGKILL → die")은 H2(데드라인에
    claude 생존 → perl ALRM → rc142 → L995 마커)를 가리키나, *실제 아티팩트*는 142-마커 부재 +
    0-byte = "perl이 rc0로 이미 빠지고 tee가 무한대기 → 운영자가 수동 kill"(H1-고아)에 더 부합.
    이 모순은 복구 과정에서 원본 상태가 덮인 탓일 수 있어, 기억/관찰 기록과 대조 필요.
  - **볼륨/지속시간 단독 기각(M7-volume):** 30 툴콜·99초 세션도 정상 종료 → "세션이 길어서/툴콜이
    많아서" 가설 기각. 트리거는 **특정 연산**(실제 Generator가 도는 `cargo test --workspace`,
    Codex MCP dispatch 등 합성이 안 부르는 것) **또는 환경적(API rate-limit 중 네트워크 read 블록)**
    쪽에 가깝다. 후자는 메모리의 반복 기록("Generator stalled (0-byte, env)" + 해당 세션대 잦은
    rate-limit)과 정합적이고, `--output-format text`의 종료-직전-flush 특성과 결합하면
    **세션 중간 API 블록 = 0-byte 로그 + 미종료**라는 관측 시그니처를 정확히 만든다. 단 이 또한
    라이브 hang 포착 전엔 미확정.

---

## 후보 수정 방향 (적용 X, 제안만 — PHASE 2)

설계 원칙: **트리거가 H1이든 H2든 모두 막도록** 한다(미확정 상태에서도 안전하게 진행 가능).

### F1. `| tee` 제거 → `> file 2>&1` + 사후 표시 (증폭기 직접 제거)
- 근거: M3b에서 리다이렉트는 fd를 쥔 손자가 있어도 즉시 종료. tee가 만드는 1800s 대기를 원천 제거.
- 장점: 최소 변경, 결정론적 효과 확인됨. `gen_rc`도 `| tee`의 PIPESTATUS 의존에서 벗어나 직접 rc 취득.
- 단점/주의: 진행상황 실시간 콘솔 스트리밍이 사라짐(사후 `cat`/`tail`로 대체). **H2(claude 자체 미종료)
  는 여전히 1800s까지 perl이 기다림** → F1 단독으로는 H2 hang의 *대기 시간*을 못 줄임(데드라인엔
  걸리지만 30분 손실은 남음). 따라서 F3와 병행 필요.

### F2. 프로세스 그룹 실행 + 그룹 kill (`setsid` + `kill -- -PGID`) — H4 해소
- 근거: M4에서 단일-pid kill이 손자를 살림. perl 핸들러(또는 래퍼)를 `setsid`로 새 세션/그룹에서
  실행하고 데드라인에 `kill -TERM -- -$pgid` → grace → `kill -KILL -- -$pgid`.
- 장점: 데드라인에 자손까지 확실히 회수 → SIGKILL 후 tee가 풀림(H1-고아 데드라인 회수).
- 단점/주의: perl `fork`+`exec`를 `setsid` 기반으로 재작성하거나 `exec setsid …`로 감싸야 함.
  bash 4 `coproc`/`set -m`가 macOS 기본 bash 3.2엔 제한적. 잘못 짜면 래퍼 자신이 그룹에 포함돼
  자기를 kill할 수 있음 → 그룹 분리 주의.

### F3. 완료-신호 조기 종료 (가장 큰 시간 절감) — H1·H2 공통
- 동작: gen_result/소스가 쓰였고 **로그 크기·프로세스 트리가 N초간 안정**이면 작업 완료로 판정,
  1800s를 기다리지 않고 프로세스 **그룹**에 SIGTERM→SIGKILL. 대기를 1800s→N초로 단축.
- 장점: 트리거가 H1이든 H2든 30분 손실 제거. 본 hang의 운영비용을 직접 없앰.
- 단점/주의(중요): **조기 종료 시 출력 잘림 위험**. `--output-format text`는 종료 직전 일괄 flush
  →killing 전에 (a) gen_result.md가 완결됐는지(말미 마커/EOF), (b) 마지막 assistant 텍스트가
  로그에 들어왔는지 확인 후에만 kill. 안정 판정 윈도(N)와 "완결" 판정 기준을 보수적으로. 너무
  공격적이면 정상 cold-build(최대 ~31s 관측, 실제 대형 피처는 더 길 수 있음)를 hang으로 오인.

### 권장 조합
**F1 + F2 + F3** 동시 적용이 트리거 미확정 상태에서 가장 견고: F1이 증폭기 제거, F2가 데드라인 자손
회수, F3가 정상완료 후 대기 단축. 단 **F3의 "완료 판정"은 PHASE 2 step 0(아래)에서 라이브 hang을
한 번 잡아 H1/H2를 가른 뒤** 그 시그니처(예: claude-node 생존 여부)에 맞춰 확정해야 오탐이 없다.

---

## 미해결 / PHASE 2에서 볼 것

1. **PHASE 2 step 0 (결정적 1회 측정):** `run_generator`에 일회성 라이브 계측을 붙여 **다음 실제
   hang**을 포착 — 데드라인 도달 시 (a) perl rc, (b) node-claude 생존 여부, (c) 파이프(`0x…`)를 쥔
   프로세스의 lsof, (d) `sample <claude-pid>`의 블로킹 syscall(pipe read / wait4 / kqueue / network).
   이 한 번이 H1-고아 vs H2를 확정한다. 합성으로는 재현 불가(모두 정상 종료), 실제 Generator는
   `rust/` 편집이라 PHASE 1에서 못 돌림 → **실측이 곧 수정과 함께 진행되어야 함**.
2. 사용자 관찰 대조: 과거 hang들이 실제로 "1800s에 perl SIGKILL(rc142)"였는지, 아니면 "무한 대기
   중 수동 kill"이었는지. 142-마커 부재 + 0-byte는 후자(H1-고아)에 더 부합 — 기억과 대조해 모순 해소.
3. F3 "완료 판정" 기준 확정(대형 cold-build 상한 측정). 합성 최대 관측 31s는 하한일 뿐.

---

## 변경 / 산출물

- **sim 코드 변경: 0줄.** `rust/`, `scripts/` 무수정. `tools/harness/harness_pipeline.sh` 무수정
  (계측은 아래 독립 스크립트로만 수행, 원본 미터치 — `git status`는 investigation 디렉터리만 표시).
- `tools/harness/results/exit-hang-investigation/` raw 아티팩트:
  - `REPORT.md` (본 문서)
  - `mech_test.sh` + `mech_result.txt` / `mech_tee.log` / `mech_redir.log` / `mech_kill.log` / `worker.sh` / `worker_slow.sh` — 결정론적 메커니즘(M3a/M3b/M4)
  - `live_probe.sh` — 라이브 폴링 하니스
  - `live_light.probe.txt` (M7-light), `live_tee.probe.txt`(M7-heavy + M2-bg + sample 포인터),
    `live_tee.sample.txt`(perl `__wait4` 스택), `live_devnull.probe.txt`(M5/H3)
  - `deep_probe.sh` — node-claude 정밀 타겟팅(perl 제외) + 트리/lsof/sample 덤프
  - `deep_tee.probe.txt`(M1/M2: 자손0, fd 1/2/4/10 동일 파이프), `deep_redir.probe.txt`(M3-live)
  - `vol_probe.sh` + `vol.probe.txt` / `vol.log` — M7-volume(30 툴콜·99초, 정상 종료 = 볼륨 단독 기각)
  - `*.log` / `*.exit_epoch` — 각 run의 원시 로그/종료 epoch
