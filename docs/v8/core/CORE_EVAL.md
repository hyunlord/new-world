# M-CORE 전환 평가

상태: **완료된 outcome·why 범위 PASS, 전체 캡처/검사 chain 완료**. BASE/new 각32세계×48일(총64세계), seed1232–1263을 평가했다. `session-terminal.json`은 `CAPTURE_AND_CHECKS_PASS`, 일곱 exit 모두0이며 UTC `2026-10-26T18:18:37.050714`에 종료했다. Authoritative outcomes/why는 precheck와 byte-identical임을 확인했다. 이는 전체 M-CORE gate·성능·native 시각·게시 수용과 구분한다.

## 고정 입력

ON 조건은 steward/defence/story/neighbours, `--mux2-observe --check-determinism`이며 두 명령·시간·exit는 `heldout-final-1/{base,mind}-probe.exit.json`에 있다. 새 두뇌는 p7 `be12050c951dfb025f3440d34b657eec0663b817`, 정책은 `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`, probe는 `772a193fd2ebcfff7823c41068b75ea4a296fb2fb8d6d4fedd100f62490d7d41`이다. 네 보정 회차 소진 뒤 정책을 바꾸지 않았다. 사전 동결은 영수증의 진술로 입증 범위를 제한하며 해시 일치만으로 시점을 증명한다고 하지 않는다.

## 결과

| 기준 | BASE | 새 두뇌 | 요구 BASE / 새 두뇌 | 판정 |
|---|---:|---:|---|---|
| 1일 비축 | 32/32 | 32/32 | 31 / 31 | PASS |
| 10일 벽 있는 침상 | 32/32 | 32/32 | 31 / 31 | PASS |
| 추위 사망0 | 32/32 | 32/32 | 31 / 31 | PASS |
| 48일 생존 | 32/32 | 32/32 | 31 / 31 | PASS |
| 늑대 사망≤1 | 32/32 | 31/32 | 29 / 29 | PASS |
| 정확한 원장 | 32/32 | 32/32 | 32 / 32 | PASS |
| 한파 노출 중 장작 | 17/17 | 19/19 | 16 / 18 | PASS |

한파 노출은 BASE17세계/비노출15, 새 두뇌19세계/비노출13으로 서로 다르다. 비노출을 성공이나 실패로 세지 않고 각 실제 분모를 유지한다. **seed1238의 늑대 사망은 BASE0→새 두뇌3**이며 해당 seed의 ≤1 기준은 실패다. 집계31/32가 요구29를 통과해도 이 악화는 삭제하지 않는다. 원인별 인과 입증이나 정책 튜닝은 없다.

## 이유 기록

2,211,840 관측 mind-tick에서 예상805,730건 전부 기록했다. 첫 선택+행동 변경469,721/469,721, 노드만 변경/restart336,009/336,009, 엄격 critical 선택12,159/12,159, 실패0이다. 단순 critical 행동 변경12,020과 엄격 선택 분모를 구분한다. 행동≥99%와 모든 critical 선택 기록 기준 PASS이며 critical 노출은 실제 관측됐다. 이는 native 사람 정보창의 시각 수용을 대신하지 않는다.

## 증거와 남은 경계

근거 루트 `/Users/rexxa/work/core-data/`:
- `heldout-outcomes-precheck.json` SHA-256 `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed`.
- `heldout-why-precheck.json` SHA-256 `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8`.
- `heldout-final-1/mind-probe/summary.json` SHA-256 `b205767e9b5ee6415dce285e15d3deb76611b486fadcc31f4bbe75a022064cec`; `mind-receipt.json` SHA-256 `cf189e5d58de415ec8656bb0f6af7083c4625eab861f9146b157c3738e04544e`.
- `heldout-final-1/base-probe/summary.json` SHA-256 `ffbdb3ef71c83d5e35ecac4faa4fefd96a1e8ef039674a6198468b1d77754c6e`; `base-receipt.json` SHA-256 `251d959d65a1eec263120b6b655a3ba7c659643e5d2dfc7a80d5e7cb98c9ac58`.

Outcome artifact의 acceptance scope는 outcome_rows_only_not_full_CORE_gate이고 acceptance_claim=false다. 전체 runner 종료와 authoritative 결과 대조는 완료했다. 성능·native UI·story·전체 gate·게시 수용은 별도다. OFF 전환 경고와 ISS336 이월은 그대로 보존한다.

Authoritative 파일 SHA-256:
- `heldout-final-1/outcomes.json`: `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed`.
- `heldout-final-1/why.json`: `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8`.

이야기 측정은 [STORY_YIELD](STORY_YIELD.md)에 별도 기록하며 수치 합격 막대가 없는 참고 결과다.
