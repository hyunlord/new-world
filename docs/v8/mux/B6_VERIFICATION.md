# Stage B6 두 도메인 추출 — 검증 기록

기준은 ecology 커밋 `76f572fffeb2a2312eb823c9e04a49b9bfc45927`이다. focused 검증과 최종 identity/OFF/ON 비교가 모두 통과했다. **Stage B 전체 gate·게시 판정이 아니다.** 원문은 `/Users/rexxa/work/mux-data/stage-b/`에 있다.

## 변경과 범위

| 파일 | 이전 줄 수 | 이후 줄 수 | 의미 |
|---|---:|---:|---|
| `sim-bridge/src/ffi/snapshot.rs` |1481|1238| 자원 수집·marshal·시험을 child로 이동, 기존 공개 경로 재수출 |
| `sim-bridge/src/ffi/snapshot/resources.rs` |0|256| 기존250줄과 header/import, 시험은 파일 끝 |
| `sim-systems/src/runtime/decision/system.rs` |3494|3487| Food/Water/Sleep만 추출; 새 public-tick 시험46줄 포함 |
| `sim-systems/src/runtime/decision/mod.rs` |37|38| private child 등록 |
| `sim-systems/src/runtime/decision/consume.rs` |0|114| 기존 세 소비 동작, 구체적 borrowed 인자 |

전체 줄 수는 늘었으며 큰 decision cascade 전체를 분해하거나 성능을 개선했다고 주장하지 않는다. 스냅숏 함수 본문과 기존 두 시험은 그대로다. 소비 동작은 optional reference 이름, food_version의 참조 역참조, helper return과 caller continue 대응만 바꿨다. 본체의 per-agent match 뒤에 실행할 문장이 없음을 확인했다. 자원 정렬·무한/빈 자원·strict/legacy 처리·계산식·tally·night wake·routine/구조 변경 순서와 API를 유지한다. ADR0039/D38에 경계를 기록했다.

## 실행 증거

- 수정 전 bridge51, decision21, 통합128(7 targets) 모두 통과: `b6-before-*.log`.
- 야간 급한 hunger/thirst에 대응 자원이 없을 때 계속 자고 회복하며 finite sleep tile을 소모하지 않는 세 경우를 public tick으로 추가했다. 추출 전1시험 통과: `b6-night-before.log`.
- 추출 후 decision22·통합128·bridge51 통과: `b6-after-decision-first.log`, `b6-after-integration-first.log`, `b6-after-bridge-second.log`.
- 첫 clippy는 child의 시험 모듈 뒤 production 항목 경고로 실패했다. 시험을 본문 변경 없이 끝으로 이동한 뒤 strict clippy와 fmt PASS: `b6-clippy-second.log`, `b6-fmt-second.log`. 실패 원문과 ISS285 보존.
- 새 debug bridge/release probe를 빌드한 실제 Godot 장면과 text renderer가 tick900에서 사람10명·3개 위치·외형10개 일치: `b6-renderer-agreement.log`. 외부 설정 경로를 명시했다.
- API103, schema57getter/837key 및 문서 검사 PASS. LSP status/diagnostics는 Transport closed로 불가이며 compiler/clippy 결과를 LSP clean으로 바꾸지 않는다(ISS284).

독립 소스 검토도 회귀 결함을 찾지 못했다(`b6-independent-review.md`). 이동한 일곱 struct/함수/시험 본문은 기준 HEAD와 일치하며, helper return과 caller continue가 기존 per-agent 흐름을 유지함을 확인했다. 이는 실행 안전망을 대체하지 않는다.

## 최종 동작 안전망

고정 probe SHA256 `633d197cd1e552f09b3732e93ac0db0ae736749df7a4d25d898d72cdd8700427`, 제품2087파일 SHA는 `b6-final-source.json`에 보존한다. 원본 baseline 공유228748값 차이0, Chronicle64파일/19683항목 동일(기존 git_sha 제외), raw digest 동일과 baseline/soak 결정성을 확인했다. OFF9개 요약은 literal-byte 동일이며 정확한 seed집합·불변식 위반0도 통과했다(`b6-safety1-receipt.json`). Stage A ON8시드×48일 요약과8개 명령 기록도 literal-byte 동일하다(`b6-on1-receipt.json`). 전체 파이프라인 exit0 뒤 제품2087경로 해시 불변을 재확인했다. 2026-10-25 13:00 KST 기록이며 전체 B gate와는 별개다.

## 제한

전체 B gate와 게시 전이다. 물리 입력·오너 시각 승인·A8 실패·기존 UI ISS274/275·설정 누락 ISS283은 이 구조 정리로 해결되지 않는다. LSP 서버 복구는 제품 패스에서 수행하지 않았다. 공개 경로와 저장 형식은 바꾸지 않았다.
