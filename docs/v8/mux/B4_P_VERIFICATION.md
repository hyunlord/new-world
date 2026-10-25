# B4 P — 인구 상한 입장 검사 검증

기준은 O `f0d5140a257628a6aeb470deb12ea8c6787fa286`이다. D40/ADR0041에 먼저 등록한 **의도적 동작 변경**이며, 기존 golden을 덮어쓰지 않는다. 원시 증거는 `/Users/rexxa/work/mux-data/stage-b/b4/`에 있다. 전체 Stage B gate와 게시, 홀드아웃 판정은 아직 대기다.

## 변경

모든 살아 있는 Agent(아이 포함)와 전체 이주단 규모의 합을 현재 상한과 비교한다. 정확히 맞으면 전원 입장하고, 상한0·초과는 거절한다. 이동 중 수락은 자리를 예약하지 않는다. 도착 직전 같은 경로에서 다시 확인하고, 수락한 이주단이 더는 들어갈 수 없으면 답을 지워 정상 Waiting 상태로 둔다. 정상 도착 기록·대기 기한은 유지하며 즉시 태도 벌점이나 부분 입장은 없다. 출생 자체는 이 제한의 대상이 아니다.

임시 AcceptError를 기존 공개 Blocked/NoVisitor로 변환하며 저장 필드·enum은 추가하지 않는다. 실패한 명령은 기존 명령 기록과 commands_rejected 외 도메인 상태를 변경하지 않는다. generation-only capped 집계는 그대로이며, 평가기는 AcceptMigrants+Blocked 명령과 도착 시 답 제거를 별도 관측한다. 명령 중복이나 매 Waiting tick을 입장 성공으로 세지 않는다.

## 경계·회귀 증거

- 최초 B0 red 실행은10실패였다. 신생 tick0 세계에 Settlement가 없는데 소속을 확인한 fixture 전제 오류를 실제 public Settlement 설정으로 고쳤다. 첫 로그를 보존하고 ISS290에 기록했다. 소속·지식·의복·도메인 원자성 assertion은 제거하지 않았다.
- 수정된 fixture를 생산 코드 변경 전 B0에서 다시 실행하면3양성 통과·7예상 실패다. 구현 후10/10통과(3.72초). 정확한 정원, 한 자리/두 사람, full/zero/over-cap, 아이 포함, 경쟁하는 두 이주단의 순서, 중복 수락, 도착 전 인구 증가/감소, 저장복원, 도메인 원자성을 확인한다.
- 기존 colony_neighbours16·engine_neighbours_ffi2·save_load7·systems neighbours5 통과. 독립 scorer36·fmt·strict clippy 통과(16.30초). `p-focused-first-receipt.json`과 각 로그가 실행 결과를 보존한다.
- 독립 소스 검토는 입장 원자성·현재 정원·도착 기한·태도·버전·저장 경로에 blocking 결함을 찾지 못했다(`p-independent-review.md`). 검토자는 실행을 반복하지 않았으며 위 수치는 root 실행 증거다.

출생·사망 경계 시험은 살아 있는 Agent 추가와 실제 despawn helper를 사용한 통제된 인구 변동이다. 자연 출산/수명 스케줄 전체를 시험했다고 주장하지 않는다. 도착 비교는 같은 저장 세계의 정상 무응답 도착을 대조군으로 두어 일반 도착 기록과 추가 입장 부작용을 구분한다. 명령 실패 저장 비교에서 제외한 것은 command_log와 commands_rejected 두 항목뿐이다.

## 고정 실행 재측정

고정 BP release probe `702bc651ab671acfa4c33df7eb1960fa40a375338b70e079f6cece7f6aeafd32`, 제어 바이너리 `272b14e34dfebda7d10d34b31a6f5c2c85f58eda3ec8365a5ed03e36ff05fea0`로 8시드×48일 보정을 완료했다. scorer는 VALID/NOT_CLAIMED다. 제작1773건·제어 cold2304행/제작32건 산술 일치, F4a/F9a는 각각8/8이다. 일반 summary 차이0·명령8개 literal 동일이며 정원 거절0·도착 답 제거0, 일별 인구 최대30으로 이 보정에서는 정원 경계가 발생하지 않았다. 따라서 자연 세계의 개선을 주장하지 않으며 경계10시험과 구분한다. 자연 Cold는 clothed0/8460, unclothed0/7332라 보호 비율 NOT_MEASURED다. 원본 identity 공유228748값 차이0·한쪽 경로0, Chronicle64파일/19683항목 동일(기존 git_sha 예외), raw digest·결정성 및 OFF9개 literal 요약 동일을 확인했다. 정확한 seed 집합과 모든 fixture 불변식 위반0도 통과했다. 재캡처794.60초·비교6.12초 종료 후 제품2105경로 hash 불변을 확인했다(`p-pipeline-receipt.json`, `p-safety1-compare.log`). 정원 경계의 의도적 동작 변경이지만 이 고정 fixture들에서는 차이가 관측되지 않았다. B0/BP/BPT/BPTH가 모두 고정되기 전1100–1131은 열지 않는다.

## 제한

인구 제한 거절도 기존 UI에서는 “Something is in the way”/“무언가 막고 있음”으로 보인다. 거절 결과와 열린 대화상자는 정확하지만 이유 설명이 부족하다. 별도 UX 이슈로 보존하며 일반 Blocked 문구를 전역 변경하지 않는다. 자연 세계 효과가 없더라도 경계 시험 결과와 구분해 보고한다. LSP는 ISS284로 불가이며 compiler/clippy를 LSP clean으로 주장하지 않는다.

문서 검사와 schema57getter/837key, staged inventory2338경로/1944행/미포함0 및 diff check가 통과했다. 잘못된 inventory_closure.py 경로를 조회한 첫 명령은 파일 없음으로 끝났고 실제 gate의 inventory.py check 명령으로 확인했다. 제품이나 검사 기준 변경은 없었다.
