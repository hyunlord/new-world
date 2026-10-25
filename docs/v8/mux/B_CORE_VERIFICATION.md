# Stage B 첫 핵심 리팩터링 — 검증 범위

2026-10-25. items / blueprints·construction styles / recipes, 단일 카탈로그 부트스트랩, format8 저장·과거6/7 이행, 최소 bridge/UI 소비자 변경을 하나의 리팩터링 묶음으로 검증했다. **Stage B 전체 완료나 게시 판정이 아니다.** 압력·동물, 실제 일곱 계열 확장 팩, 최종 전체 gate와 후속 작업이 남아 있다.

## 동작 안전망

원본 기준선은 `67a6232ab41a4bf5051b11303bcac97543b111f0`, 비교 대상 집사 ON 기준은 게시된 Stage A `ca9f531ca6ef700ca223d99dba14d834b94b94b45da`다. 원시 증거 루트는 `/Users/rexxa/work/mux-data/stage-b/`다.

| 검사 | 최종 결과 | 원문 |
|---|---|---|
| 기준선 identity | 공유 값228,748개, 차이0; Chronicle64파일19,683항목 동일(기존 git_sha 제외 규칙); raw sample digest 동일 | `core-safety2-compare.log` |
| colony OFF | soak, story, neighbours, knowledge_f10, m55_pressure, m55_nodraft, m55_storyoff, m55_noneighbours, m55_quiet의 요약9개 literal-byte 동일 | `core-safety2-compare.log`, `core-safety2-receipt.json` |
| 결정성·불변식 | baseline 및 soak 결정성 PASS;9fixture 정확한 seed집합·불변식 위반0 | `core-safety2/`, `core-safety2-capture.log` |
| 집사 ON | seeds0–7 ×48일, story/neighbours ON; 요약과 명령 기록8개가 Stage A와 literal-byte 동일 | `core-on2-receipt.json` |

최종 릴리스 probe SHA256: `bb9a10a82970bafc0f0d564ae8c88e98c7f4d0f95ac3ad08403ef54da1238845`. 두 번째 캡처는 최종 Rust 경고 수정 후 다시 빌드한 바이너리를 사용한다. 이전 core-safety1/ON1 성공을 새 바이너리 결과로 재사용하지 않았다. 정규화·평가 기준 변경·held-out 튜닝은 하지 않았다.

## 별도 검증

- `cargo clippy --workspace --all-targets -- -D warnings`: exit0,22.65초 (`core-clippy-second.log`). 첫3경고 실패와 최소 표현 수정 기록 유지.
- sim-core372통과/0실패 (`core-all-second.log`), save14통과/0실패 (`save-tests-third.log`), fmt PASS (`fmt-fourth.log`). 앞선 save15에서 중복된 numeric codec 시험1개를 제거하고 더 강한 기존 golden을 유지한 뒤14개를 재실행했다.
- 실제 M5.5·M-ART2 format6 및 Stage A format7 저장 계속 실행·재저장 비교3/3통과 (`historical-migration-first.log`). 이 검사는 앞선 같은 저장 구현에서 실행했고 이후 Rust 변경은 두 검증 조건의 논리적 표현과 타입 별칭뿐이다. 최종 전체 gate 및 후속 format 변경에서 다시 실행한다.
- 확장 blueprint layering/dig/save와 재사용 cutter 경로 통과 (`extension-runtime-first.log`, `cutter-positive.log`). 내구도 설정 키 오류는 일반 제작의 부정대조 실패 뒤 수정했다. 이 시험들은 일곱 계열 B3 전체를 대신하지 않는다.
- 실제 창 UI-fifth284검사/0실패, SCRIPT ERROR·Parse Error0. 독립8장 회귀 검토 PASS. 문과 실제 경고4개 복원 확인. 기존 한국어 줄바꿈·잘림 ISS274/275는 절대 품질 REVISE, 오너 승인 NOT_CLAIMED (`ui-fifth-receipt.json`, `ui-fifth-visual.json`).
- bridge 재빌드, 변경된 harness2개 엄격 검사, schema57getter/820key,103export API 검사 및 시스템 표를 포함한 문서 검사 PASS (`docs-core-third.log`). 인벤토리2274경로/1944행/미포함0, 스테이징 공백 검사 PASS. 전체 gate는 아직 실행하지 않았다.

## 저장 형식과 실패 이력

새 format8은 카탈로그 revision을 typed payload보다 먼저 확인한다. 과거6/7 reader는 필드 순서·one-byte 원인·저장된 인덱스를 보존한다. core8의 seed5 ON tick5000/6000 저장도 별도 획득해 이후 호환 검증용으로 보존했다(`core8/acquisition-receipt.json`).

RON 배열 표현 오류, UI-first exit0이지만 SCRIPT ERROR2개였던 실패, UI-second/third 시각 REVISE, cutter 음성 대조 실패, 첫 clippy/format/엄격 harness 실패는 외부 원문과 D36·ISS269–273에 남아 있다. 부분 시험 성공을 전체 gate 성공으로 바꾸지 않는다. A8 원래 실패와 ISS030 지연도 그대로 유지한다.
