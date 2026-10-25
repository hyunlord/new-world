# Stage B 압력·동물과 일곱 계열 확장 — 검증 기록

2026-10-25. 기준 core 커밋은 `e90b7bb86339d1c83b9fb061f6ae36ba2b448f1e`다. **이 단위의 동작 안전망은 통과했으며 Stage B 전체 gate·게시 판정은 아니다.** 원시 증거는 `/Users/rexxa/work/mux-data/stage-b/`에 보존한다.

## 구현 범위

PressureKind·AnimalKind를 안정된 u32 식별자와 versioned RON으로 옮겼다. 기존 스케줄·keyed draw·규칙 override를 유지하며 실제 효과·사냥·전리품·인과·Chronicle·bridge·UI에 선택된 콘텐츠 정체성을 전달한다. 새 질병의 provenance는 Sick의 기존 until 형식을 바꾸지 않고 sparse map으로 저장한다. 전파·압력 종료 후 효과·회복·사망·저장 복원 경로를 검증한다. 같은 mechanic의 서로 다른 활성 압력은 일반 진입 경로에서 거부한다.

format9는 실제 과거6/7과 획득된 bundled core8 revision만 명시적 reader로 이행한다. 기존 인과 링의 순서·인덱스·history를 직접 변환한다. 새 construction style은 검증된 기존 art 스타일 별칭이며 바이너리 아트는 추가하지 않았다. grazer 총수 상한과 pack bookkeeping은 공유되며 독립 종별 개체수 정책을 구현했다고 주장하지 않는다.

## 실행 증거

| 범위 | 결과 | 원문 |
|---|---|---|
| core | unit378 통과, 별도 integration2+8+22+6 통과 | `ecology-core-final.log` |
| save |19 통과; 첫 wrong-decoder 음성 assertion 실패 보존 | `ecology-save-second.log` |
| 기존 story/wildlife/FFI |10+12+6 통과 | `ecology-runtime-third.log` |
| 실제 과거 저장 계속 실행 | M5.5/M-ART2 v6, Stage A v7, core v8의4사례 통과 | `ecology-historical-first.log` |
| 일곱 계열 Rust | 실제 제작·건설·학습·압력·사냥·loot·save와 public collector assertion, bundled-before/after 및 잘못된 팩 거부 통과 | `ecology-pack-final.log` |
| 새 질병 | full-width ID,4321tick 전파·효과·종료·저장·회복·사망 통과 | `illness-runtime-first.log`, `ecology-pack-final.log` |
| 새 포식자 | 실제 생성·authored HP/이동·공격·처치·선언 loot·인과·pack 종료·저장 계속 실행 통과 | `predator-runtime-first.log` |
| 확장 팩 UI | headless39/0, window40/0, SCRIPT ERROR0; 실제 경고 클릭·학습 Next·EN/KO 적합·미니맵 | `data-only-gate-third/`, `data-only-window-second/` |
| 기존 UI |284/0, 오류0, 독립 scoped시각 PASS; 기존 ISS274/275 절대 품질 REVISE | `ecology-ui-first.log`, `ecology-ui-first-visual.json` |
| M5 실제 창 | 격리 설정으로43/0, 오류0; 늑대·한파·질병·why·연말 요약 관찰 | `ecology-m5-ui-third.log` |
| 재질·렌더러·배율 | material39/flora352 일치, 문 스타일 의미 보존, 배율220/0,9장 독립 scoped시각 PASS(기존 ISS274/275 유지) | `ecology-resolver.log`, `ecology-renderer.log`, `ecology-scale.log` |
| 정적 검증 | fmt·strict workspace clippy·변경 driver strict PASS, 확장 grep10 PASS | `ecology-fmt-final.log`, `ecology-clippy-final.log`, `ecology-driver-strict-final.log`, `ecology-grep-first.log` |

## 안전망

최종 probe SHA256은 `afacc5a209e33b408ef6ee4e16bc455714f6a5e895ab2a2e885e4f9bbb0fdbaa`다. product source2085경로의 SHA를 `ecology-final-source.json`에 고정했다. 원본 baseline identity 공유228,748값 차이0, Chronicle64파일19,683항목 동일(기존 git_sha 제외 규칙), raw digest 동일이다. OFF9개 요약은 literal-byte 동일하고 정확한 seed집합·불변식 위반0·baseline/soak 결정성을 확인했다(`ecology-safety1-compare.log`, `ecology-safety1-receipt.json`). Stage A ON8개 시드×48일 요약과8개 명령 기록도 literal-byte 동일하다(`ecology-on1-receipt.json`). 완료 뒤2085개 제품 소스 해시 불변을 다시 확인했다.

문서·UID·schema57getter/837key 검사는 PASS, 인벤토리는2310추적경로/1944행/미포함0이다. 최종 전체 Stage B gate는 아직 실행하지 않았다.

## 실패 이력과 제한

첫 B3 실행은 harness 실패0이어도 런타임 Array 형식 예외가 있어 FAIL이었다. bootstrap은 명시적 성공 receipt만 받도록 수정했고 locale/category 음성 대조도 통과했다. 첫 확장 팩 화면은 제목 잘림·빈 미니맵으로 REVISE였으며 둘째 실제 화면에서 수정 확인했다. 첫 M5 headless는 오래된 열 폭과64px viewport 때문에 실패했다. 각 실패 원문을 지우지 않았다.

보조 M5 두 실행에서 설정 격리 환경 변수를 누락했다(ISS283 OPEN). 공유 settings.cfg의 수정시각이 겹치며 실행 전 사본이 없어 변경 내용과 복원은 알 수 없다. 사후 사본을 보존하고 임의 복원하지 않았다. 외부 설정 경로 없는 실행을 시작 전에 거부하는 guard를 추가했다. 격리한 재실행43/0 동안 공유 파일 SHA·mtime_ns가 불변임을 확인했지만 앞선 사건이 해결됐다는 뜻은 아니다.

authored drought 배율1001은 거부하고0/1000은 허용한다. 기존 직접 StoryRule의 과도한 배율은 ISS277 OPEN으로 분리했으며 이 리팩터링에서 기본 동작을 바꾸지 않았다. 조작된 save의 동시 mechanic 충돌 상태를 별도 거부하는 hardening은 미구현이다. 물리 트랙패드·오너 시각 수용은 NOT_CLAIMED이며 A8 실패와 ISS030 지연은 유지한다.
