# Art Pass 6 보고서 사실 대조

판정: **현재 보고서 본문에서 수정이 필요한 사실 모순이나 §14/§15 항목 누락을 발견하지 못했다.** 최종 게이트·브랜치 게시·PR 상태는 이 판정에 포함하지 않는다.

- 감사 대상: `REPORT.md`, SHA256 `001af2232187fae4f1bff50eee13d7fa5dfd3a54cab06a95b2ef9b2f86fc01d1`.
- 소스 커밋 범위: `ca47e703..7afa014dbca504f965c63d5581cd6220200615c6`, first-parent 17개. 본문에 17개 전체 해시가 존재한다.
- 허용된 쓰기: 이 `review/report_audit.md`만. 보고서, 측정값, 관찰 원문, 소스 및 기존 판정은 변경하지 않았다.

## 확인한 근거

| 항목 | 대조 결과 |
|---|---|
| §15 형식 | `구현 의도`부터 `정직한 판단`까지 요구된 소제목 10개 모두 존재하며 순서 일치 |
| §14 인게임 8항목 | 보고서의 명시적인 8항목 대응 문단 존재. 미수락 입김·한기·바람·군중을 완료로 바꾸지 않음 |
| A1–A25 | `measurements.json`의 25개 상태와 보고서 표를 프로그램으로 비교: 불일치 0. 21 PASS / 4 FAIL(A18/A21/A22/A24) 일치 |
| 캡처 수·SHA | `gates/p7/capture_audit.json`: ART94 + LONG21 + GOD8 + calibration4 = 127, 모두 `44011f92347dc4176cc115195b1cc55a16ff93ae`. ART/LONG115 중 lineup/calibration4 제외 자연111 일치. 네 로그 SCRIPT ERROR0 |
| A10 | 원장 n300, live simulation true, 중앙8.212ms/p95 14.918ms 일치. 성능을 장기 안정성으로 확대하지 않음 |
| A11 | `blind/result.json`의 두 관찰자 4×/2× 혼동 행렬 4개를 본문 표와 프로그램으로 대조: 모두 동일. 87.5/87.5%, 87.5/91.67%, carry3/3 일치 |
| A17 | 실제 운반14관측, 고유 ID10, adult_a/adult_b 2체형, fallback0과 일치 |
| A18 | 111장, 초과12장, 최대34명, d24 F2 6명 재계산 일치 |
| A19 | 실제 visible cold0, body presentation N/A, target labels108. 보고서가 PASS의 제한된 범위를 명시 |
| A20 | installations2, final_bound_frames74, changed_villager_frames61, animation_groups20 일치 |
| A21–A25 | 밤8장 중 입김 반복 실패3장, 아침3장 실패, HUD111장, GOD8장 중 바람2장 실패, 자연 추운3장 성공 일치 |
| 아침 한기 | `review/morning_trace_summary.json`의 d29 rendered42/31명과 독립 식별0명 일치. d27 warm-control 전제 불성립을 숨기지 않음 |
| 운반 제한 | source/rendered46개, certified0; 승인 food6그룹20프레임 proxy만 통과; wood2그룹 거절 유지. `carry/IMPLEMENTATION.md`, round2 원문과 일치 |
| 운반 비교판 | `carry_2x.json`은 staged-native 10그룹36프레임. 보고서가 staged 비교판과 runtime framebuffer 비교판을 구분 |
| 독립 화면 판정 | 자연 추운 밤 NEEDS WORK / 아침 ACCEPT / 운반2× ACCEPT / 신의 손 NEEDS WORK 일치. 아침 ACCEPT를 한기 승인으로, 운반 staged ACCEPT를 runtime 전면 승인으로 확대하지 않음 |
| 관찰 원문 보존 | `scene_effect_observations.json.original_report_sha256`와 현재 `scene_observations.json` SHA256 일치: `1e2f3ad72304d67ac16993b86f7ee0d92f690885a0651cdbe3970c7c973f9520` |
| 코드 리뷰 제한 | 32개 동일 기록 이후 transient 재시작 누락 및 wood 명령도 food label에 flash하는 제한을 `code_audit.md`와 일치하게 기재 |
| 캡처 이후 렌더 변경 | `git diff --name-only 44011f92..7afa014d -- scripts scenes assets localization rust/crates/sim-bridge/src` 결과 비어 있음. 후속 정적 테스트/측정 변경과 캡처 source 구분 타당 |
| 링크 | 보고서의 모든 상대 링크 대상이 현재 파일시스템에 존재 |

## 의도된 보류와 최종 응답 인계

1. `FINAL_GATE_PLACEHOLDER`는 요청에 따라 의도된 보류다. 이 감사에서 전체 게이트를 실행하거나 PASS/FAIL을 추정하지 않았다. 실제 완료 로그로 채우는 일은 리더 소유다.
2. GOAL 마지막 문장이 요구하는 양쪽 브랜치 HEAD와 `git ls-remote` 일치 확인, PR #511 설명 갱신 여부 및 draft 유지 상태는 최종 게시 후 최종 응답에 넣어야 한다. 현재 본문을 근거로 게시 성공을 주장하면 안 된다.
3. 자기 자신의 최종 보고 커밋 해시를 본문에서 요구하지 않았다. 지시대로 최종 응답에서 해당 해시와 Part를 추가하면 된다.
4. 이 감사는 기존 수치와 판정의 보고 정확성을 검사했다. 새 픽셀 리뷰, 게이트 인증, 미해결 시각 목표의 수락은 아니다. 원본 NEEDS WORK 및 FAIL을 그대로 유지해야 한다.
