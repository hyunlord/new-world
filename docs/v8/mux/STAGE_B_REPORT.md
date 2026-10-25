# Stage B 구현 보고 — 구조 검토와 확장성

**진행 중 / 전체 gate·게시 대기.** 아래 완료는 개별 검증 단위이며 Stage B 전체 완료를 뜻하지 않는다. 최종 기준 HEAD·gate·remote 결과는 실행 후 기록한다. 현재 오너에게 게시된 버전은 Stage A `ca9f531ca`다.

## 검토 결과와 처리

전체 검토는 `docs/v8/review/CODE_REVIEW_2026-10.md`에 아키텍처·시스템/data flow·콘텐츠/기계 enum 판정·hard-coded ID·baseline 의존·dead code·중복·결정성·시험 건강도·문서 차이를 보존했다. 역사적 조사와 변경 후 현재 상태를 구분하며 최종 B 소스 갱신은 대기다.

| 영역 | 처리 | 근거와 남는 경계 |
|---|---|---|
| 아이템·설계도/건축 스타일·레시피/작업장 | 안정 숫자 ID와 버전 데이터 registry/resolver/category fallback으로 이행 | `B_CORE_VERIFICATION.md`; 실제 과거 저장 이행, 폭 넓은 ID와 데이터만 변경하는 fixture |
| 동물·압력·질병 효과 | 실제 identity를 생성·행동·전리품·history·presentation으로 전달 | `B_ECOLOGY_VERIFICATION.md`; 고정 과거6/7/8 reader와 format9, 별도 sparse illness provenance |
| Godot UI/표현 | catalog bootstrap·full-width snapshot·표시 이름/아이콘 lookup | API103, snapshot57getters/837keys; bridge가 authoritative source |
| 확장 팩 | item/recipe/mud-brick style/pressure/animal/knowledge/alert/lesson을 데이터로 추가 | gate data_only_ext; 시뮬레이션·headless39·window40, UI284/scale220 등은 각 실행 범위로 기록 |
| 큰 모듈 | 자원 snapshot과 Food/Water/Sleep 실행 본문을 응집된 모듈로 분리 | `B6_VERIFICATION.md`; 기존 실행 순서·공개 경로 유지, 성능 향상 주장 없음 |
| 남은 enum/debt | Top·Designation·고정32지식 영역·기계 enum 등 판정별 유지 | 무조건 모든 enum을 콘텐츠로 간주하지 않음. 세부 사용처·판정·후속 우선순위는 검토 문서 |
| dead code | 사용하지 않는 helper/library 삭제 근거 수집 | 실제 삭제는 Stage C에서 현재 참조 재확인 후 수행 |

## 커밋별 안전망

| 단위 | 커밋 | 검증 |
|---|---|---|
| B p1 core | `e90b7bb86339d1c83b9fb061f6ae36ba2b448f1e` | 원본228748값/0차이, Chronicle64/19683 기존git_sha 예외·정규화 raw,9literal OFF, frozen Stage A ON 요약/명령8개 동일 |
| B p2 ecology | `76f572fffeb2a2312eb823c9e04a49b9bfc45927` | 같은 원본/9OFF/ON 안전망, 실제 과거 저장·확장 팩·renderer/bridge/문서 검증 |
| B p3 B6 | `c9ed292d77902f30f5c8975f01d770bd43e52927` | 원본/9OFF/ON 동일; bridge51·decision22·integration128, renderer10agents/3tiles/10appearances |
| B p4 observer | `f0d5140a257628a6aeb470deb12ea8c6787fa286` | 원본/9OFF/정확한 ON, 관측 OFF/ON 요약·명령·save4000 동일. 첫 ON 입력 오류 보존 |
| B p5 population | `bbae211b3ee8cf8651cbe43d1014831b8a606001` | 경계10, 원본/9OFF 동일, 보정8×48. 자연 보정에서 cap 사건0이므로 자연 개선 주장은 없음 |
| B p6 trader | `54a55a421361d9c24268e1b3a6a5d2c2d1c6c211` | 경계15·observer8·기존 이웃16·저장7·과거 이행4, 원본 동일; OFF3동일/6교역·경제 차이 보존 |
| B p7 meat haul | `bdbd4fb2ea9c5b446648778ea7b0b2da14635049` | 정상 대조4/예상 실패3→최종23통과, 기존 회귀33통과; 보정8×48 VALID, 원본 동일·OFF2동일/7차이 보존 |

원본 identity의 Chronicle/raw 동일성은 기존 git_sha 제외·정규화 계약을 따른다. byte-identical로 명시한 colony summary/command/save 비교와 구분한다. 모든 상세 실패/수정/실행 조건은 각 검증 문서와 `MUX_DECISIONS.md`에 남긴다.

## 동작 변경과 평가

P는 명령 수락과 실제 도착 모두 현재 살아 있는 전체 Agent 인구에 파동 전체가 들어갈 때만 수용한다. 조기 수락은 자리를 예약하지 않으며 도착 시 초과하면 정상 Waiting으로 돌아간다. 일반 blocked 표시 문구는 ISS291로 남는다.

T는 신선한 성인·최근 수행자 수로 희소/희귀 후보를 분류하고 데이터 정책30/10/1을 사용한다. 저장된 규칙 layout과 이미 생성된 물품을 보존하며 로드 후 미래 거래는 설치된 정책을 따른다. 자연 보정 제작1773→1644, scarce 제안73스택, rare 후보14caravan/15행·선택0이다. 이 값은 보정이며 개선/홀드아웃 수용 판정이 아니다. 독립 재계산은 두 arm caravan108개·자연 제작3417건·통제 제작32건·Cold2304건에서 불일치0이다.

H는 비축 우선 뒤에 실제 화덕 주변 안전 지면 운반을 추가하며 일반 pickup/walk/drop/feed와 취소 보존을 사용한다. 새 job/save 필드나 순간 식량 지급은 없다. 보정 제작1665건, stockpile79·hearth28 운반 시작이나 종료107개 모두 UNRESOLVED다. 자연 성공 배달·일률적 식량 개선으로 해석하지 않는다. 통제23시험이 완결 배달·취소 보존을 별도 증명한다.

F4/F9는 `B4_PREREGISTRATION.md`의 고정 산술·표본·판정 기준을 적용한다. 네 빌드 B0/BP/BPT/BPTH를 모두 고정한 후1100–1131×48일×4를 실행한다. 현재 홀드아웃 NOT_RUN; 선택적 full-M55 controls NOT_RUN. 순차 조건부 차이이며 독립 factorial 효과가 아니다.

나무 종 B5는 준비 계획만 있으며 구현·검증 대기다. 구현되지 않은 것을 완료로 표시하지 않는다.

## 전후 구조와 성능

B6 snapshot facade1481→1238행과 새 resources256행, decision3494→3487행(새 회귀46행 포함)과 consume114행이다. facade 감소를 총 코드 감소로 제시하지 않는다. 최종 전체 LOC/enum 수는 남은 B 단위 후 갱신한다.

B1의 clean Stage A N1000 fed baseline 하루 표본은 tick mean0.88171087ms, decision steady71.007724%다. 역사 M2의80%를 현재 측정으로 재사용하지 않는다. 이것은 최종 B 성능·활성 게임 FPS 증거가 아니다. 최종 gate/perf 결과는 대기다.

## 제한사항과 게시

LSP Transport closed(ISS284), 공유 settings 격리 누락(ISS283, 이전 상태 미확보), 기존 UI PRE01/02(ISS274/275), Stage A A8 실패 기준 및 실제 물리 입력 NOT_VERIFIED는 해결로 표시하지 않는다. O의 운반 종료 UNRESOLVED는 성공 배달 증거가 아니다. baseline/V7 코드와 research 경로는 보존하며 분리는 오너 결정 사항이다.

전체 gate가 status0이고 정확한 HEAD가 깨끗할 때만 `v8/mux→v8/wip→v8/main` fast-forward 게시한다. **현재 B gate/remote 확인 대기.**
