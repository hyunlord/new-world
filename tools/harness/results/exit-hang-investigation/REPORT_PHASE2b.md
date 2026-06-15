# Generator exit-hang PHASE 2b — capture analysis + MCP-disable fix

> Repo: hyunlord/new-world · Branch: lead/main · base HEAD: 0612b872
> 변경 파일: `tools/harness/harness_pipeline.sh` (8개 subagent 호출) + `tools/harness/empty-mcp.json` (신규).
> rust/·scripts/ 0줄. harness-infra → pipeline-exempt.

## 캡처 분석 (live_capture_20260615_100249_attempt1)

PHASE 2a watchdog가 실제 hang을 포착 (run: `inventory-2-2-pickup --full`, Generator code-attempt-1).
- `outcome.txt`: `gen_rc=142 log_bytes=0` — 30분 내내 **0바이트** 출력, SIGALRM(142)으로 강제 종료.
- main `claude` (pid 42587) **끝까지 생존**, 부모 = perl 래퍼(42586) → **H1-고아 배제**.
- main 스택: `kevent64` / `mach_msg2_trap` = libuv 이벤트 루프 **idle** (CPU 스핀·소켓 read 아님).
- main lsof: Anthropic API(160.79.104.10:443) ESTABLISHED 11개 — 연결은 열려 있으나 read 중 아님.
- 자식 6개 = **전부 stdio MCP 서버** (omc-bridge, team-mcp, server-github, agentmemory, firecrawl,
  codex-mcp), 전원 `start (in dyld) + 6992`에 정지 = **dyld 부트스트랩에서 wedge** (이번 실행에서
  기계적 게이트가 ~2.8h 걸린 것과 동일한 dyld/ReportCrash 병목).

## 판정 — H3 "완료했으나 MCP 자식이 dyld에 끼어 정상 종료 불가"

Generator는 첫 ~31초에 Gather 코드를 디스크에 다 작성 (watchdog의 "rust/ 90s 무변동" 게이트가
작성→침묵을 확인). 그 뒤 Node가 종료하지 못함 — wedge된 MCP 자식 핸들이 이벤트 루프를 살려둠.
`--output-format text`는 **정상 종료 시점에만** 출력을 flush하므로 → `log_bytes=0` → 1800s 타임아웃.

이것은 H1(고아)도 H2(API read stall)도 아니다. F3(완료-신호 조기 종료) 단독으로는 불충분 —
종료를 막는 것은 출력 버퍼가 아니라 **wedge된 자식 핸들**이기 때문.

## Fix — harness subagent에서 MCP 서버 비활성화

8개 `claude --agent` 호출 전부에 `--strict-mcp-config --mcp-config tools/harness/empty-mcp.json`
추가 (drafter ×2, challenger, quality-checker, generator, vlm-analyzer ×2, evaluator).
`empty-mcp.json` = `{"mcpServers":{}}`. `--strict-mcp-config`는 다른 모든 MCP 설정(.mcp.json,
~/.claude.json, 플러그인)을 무시 → **MCP 자식 0개 기동** → wedge될 자식 없음 → 정상 종료 가능.

harness subagent는 MCP가 전혀 불필요 (built-in Bash/Edit/Write/Read로 Rust·GDScript 작성·테스트).
Codex evaluator는 별도 경로(메인 세션의 `mcp__codex__codex`, `claude --agent` 아님)라 영향 없음.

## 검증 (실측)

`claude --strict-mcp-config --mcp-config tools/harness/empty-mcp.json --output-format text
-p 'reply DONE'` → MCP 자식 **0개** (pgrep -P 빈 결과), 출력 `DONE`, `EXIT=0`. `bash -n` 문법 OK.

## 미해결 / 다음

- **환경 잔류:** dyld/ReportCrash wedge + 누적된 leftover MCP 프로세스(~39개 system-wide). 재실행 전
  잔류 MCP 정리 + 리부트로 wedge 리셋 권장.
- **재실행:** 환경 리셋 후 `harness_pipeline.sh inventory-2-2-pickup --full` 재실행 → 이제 Generator는
  MCP 자식 없이 깨끗이 종료 → 정상적으로 Visual/Evaluator까지 진행할 것.
- **F3 후순위:** MCP-disable로 H3가 제거되면 F3(완료-신호 조기 종료)의 필요성은 크게 낮아짐. 잔존
  "work-done-no-exit" 시그니처가 재발할 때만 재검토 (deferred).
- **디스크의 Gather 코드:** Generator가 남긴 ~223 LOC는 UNVERIFIED(Evaluator 미통과) → 커밋 안 함.
  재실행이 새로 생성·검증·커밋함.

## Governance chain
0612b872 → (PHASE 2b, harness infra: strict-mcp-config on 8 subagent calls)
