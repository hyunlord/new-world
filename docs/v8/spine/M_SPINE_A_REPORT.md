## M-SPINE-a 게시 보고

이 문서는 a 게시 당시의 10개 항목 보고를 보존한다. 이후 b 구현은 제품에 통합 중(IN_PROGRESS)이며, b 검증은 NOT_RUN·미게시다. 아래의 “외부 초안”은 a 게시 시점의 상태다. 원본 근거는 외부 `a-publication-receipt.json`, `fullgate-a-attempt5/postrun-binding.json`, `M-SPINE-a-report.md`이며, 현재 a 화면 시트는 최종 b 통합 큐레이션 전까지 유지한다.

### 구현 의도
두 시대에 고정된 발전 표시를 데이터 기반 네 시대·열 레인과 원인이 있는 지식 발견·회색 콘텐츠로 확장한다.

### 구현 내용
- eras.ron: 네 시대·열 레인·별칭·임계값. 기존 643‰ 상한 제거.
- knowledge: 기존31개 재분류+신규12개, tier/status/debt와 Practice, 64비트 지식 마스크.
- items: 기존38개+5개, 끈 제작 연결·기존 창 제작 게이트 이동·형태/색조 메타데이터.
- get_tech와 tech_panel: 40셀·43카드·시대 게이지·회색 빗금/준비 중/debt·선행 사슬 강조.
- save format18: 실제 format17 저장의 지식·숙련·시대 상태 마이그레이션.
- CONTENT_DEBT 생성/일치 시험·선택 팩 확장 검증·ADR0060·ko/en.

### 구현 방법
시대별 양의 marker_weight 합을 분모로 기존 성인 실천자 coverage를 가중 평균한다. 최고 도달 시대와 바로 다음 시대의 전환을 판정한다. Inert 항목은 표시하되 발견·게이트·초기 지식·이웃 유입에서 제외한다. 기존 ID 순서와 카탈로그의 로컬 부트스트랩을 보존한다.

### 기능 설명
기술 지도에서 네 시대의 기술과 준비 중인 항목을 함께 보고, 선택한 기술의 선행 조건과 미연결 이유를 확인한다. 실제 작업 완료가 새 지식 발견의 원인이 된다.

### 게시 SHA와 게이트 결과
2026-11-08 01:01:16 KST, 시작 후10시간48분에 c467c0a36466ec86419ae66bfb702e37c2566b30을 v8/wip→v8/main 순서로 fast-forward 게시했다. 두 원격과 review 모두 정확히 이 SHA로 확인했다. a-publication-receipt.json에 argv·시간·실제원격SHA를 보존했다.

전체5차 게이트49/49 PASS·exit0(22:10:29–01:00:11KST). DEBUG3119PASS/0FAIL/IGNORE14, RELEASE 원본카운터3118PASS/0FAIL/IGNORE14, M1 297PASS/0FAIL, play6/6. RELEASE 한 건 차이는 동시 stdout으로 child summary가 분리된 것으로 증명했으며 원본/카운터를 수정하지 않았다(release-count-interleaving-receipt.json). 기존step41면제만 사용: 확대율1.0 [57.528,51.19]ms,0.5 [58.899]ms,60ms초과0. 추가면제없음. baseline identity228748공유키/차이0/단측키0,64연대기19683항목동일. 결정론·저장·clippy·docs·화면·legacy6–10·보존 검증 모두PASS. 전체gate.txt·exitreceipt·config/source/binary해시는 fullgate-a-attempt5/postrun-binding.json에 결합했다.

게시후 canonical외부성능원장 M-SPINE-a-PERF_LEDGER.md에 실제green gate를 인수로 기존bench_ledger.py로6행추가했다. 소스를 바꿔 게이트조상에PASS를 붙이지 않았으며, 이 원장은 다음b문서커밋에 반영한다.

### 사슬 시험·데이터 전용 시험 결과, 캡처 경로
- 기존 scoped 시험: core514PASS, spine_eras_from_data/Practice2PASS, debt1PASS, data-only1PASS.
- 저장70PASS/기존IGNORE2, 신규7개 포함. 실제 BASE17→18 byte roundtrip·1440tick 쌍 비교 PASS.
- c467 실제 창1920×1080/1440×900, ko/en: 각각56검사 PASS. 선택 팩55검사 PASS. 콘솔 오류0·독립 원본PNG10장 시각 검토 PASS.
- 열린/닫힌 기술 지도 각120프레임, 총8맥락:119.04–120.08 FPS, 모두 시뮬레이션 tick 진행,60FPS 기준 충족. 단기 관측이며 전체 게임 성능 주장은 아니다.
- 원본: /Users/rexxa/work/spine-data/a-c467-captures/{tech-1920,tech-1440,extension}; source-receipt.json·visual-review.md.
- 건물 사슬 spine_chain_log_house와00..04 캡처는 두 번째 게시 범위이며 아직 실행하지 않았다.

### 크래시 감지 결과
seeds0–7×48일, 집사ON·명령없음. BASE 생존 [10,12,11,13,10,13,11,14], 후보 [10,13,11,13,10,13,13,12]. 양쪽 KnowledgeLost [0,0,0,0,0,0,0,0]. 새 전멸0·KnowledgeLost 상한 충족 PASS. Inert 전환0회. 원본 crash-base/summary.json·crash-a/summary.json·crash-a-comparison.log/json. 별도 크래시 검증 결과이며 전체 게이트를 대신하지 않는다.

### CONTENT_DEBT 항목 수
노드10 / 템플릿0 / 아이템4. 기존 M6b 수동 debt를 보존했다.

### 새 도구 줄 수와 그 이유
- debt 공유 렌더러63+실행기24=87줄: 데이터와 문서 불일치를 검출·재생성할 경로가 없었음.
- spine_save_check77줄: 기존 probe에 load 인수가 없어 실제 이전 저장의 마이그레이션·이어하기를 검증하지 못함.
- spine_tech_test224줄: 기존 화면 시험의24카드·643상한 가정이 신규43카드·40셀·시대 게이지·debt를 다루지 못함.
- 기존 m4 시험 추가25줄·선택 팩 화면 시험 추가42줄까지 포함한 보수적 추가량455줄. 상한3000줄 이내.

### 확인된 제한사항·후속
- 기존 flax_stalks2개→cord1개를 사용한다. 존재하지 않는 bark/flax straw 원료는 추가하지 않았다. 바구니 운반+5와 나머지 미연결 항목은 C1 debt이다.
- 건물 템플릿·명령·건물 패널·자리표시 렌더링은 b 외부 초안으로만 준비돼 있으며, a 게시 후 통합·실행한다.
- 5차 실제짝성능3라운드×장면별1회 중앙값: N1000 BASE1683.2→후보1741.4µs(+3.5%), feature_first REPORT_ONLY. 여섯진입부하 [4.60,4.34,3.43,4.04,3.38,3.06] 모두8미만. C3시간/메모리상한PASS. 원본6개bench.json은변경하지않고 파생중앙값projection에원본해시를기록했다. 실패한4차N1000+4.3%는보존하지만정식게시근거에서제외했다.
- 시도1 locale 중첩 오류 중단, 시도2 레시피26번 예상 행 누락 실패·임신시험 미완료 중단, 시도3 보호 경로 기본값 접근 중단을 모두 보존했다. 어느 시도도 전체 통과로 처리하지 않았다.
- 시도3 단계21에서 PLAY_ROOT 기본값을 사전에 덮어쓰지 못해 금지된 new-world-play/main·m1의 빌드/import/실행이 발생했다. main HEAD가 fbc03bddb8f5b68f43fba1b2dad283982e5cf436→68beae3afb5d8e83ebc171746e5cf30a20531e20으로 변경됐다. 즉시 소유 프로세스만 종료·생존0 확인했다. 해당 경로를 복원·정리·삭제하지 않았고, 원본 로그·HEAD reflog·상태를 읽기 전용 감사해 fullgate-a-attempt3/protected-path-side-effects.json·ABORTED_PROTECTED_PLAY_ROOT.json에 보존했다. v7 진입은 해당 시도에서 관측되지 않았다.
- 시도4부터 명시적 외부 PLAY_ROOT를 사용했다. 시도5는 같은 외부 root와 새 fullgate-a-attempt5-play-data, 검증된4개legacyfixture환경변수로49단계전체를처음부터실행했다. 4차47PASS/2FAIL(v7최초import signal11/폰트시작오류,legacyenv누락), 집중preflight정상실행후5차전체PASS; 어느실패도성공으로대체하지않았다.
