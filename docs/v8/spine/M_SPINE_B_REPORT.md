## M-SPINE-b 게시 보고

**현재 WIP — 구현 통합 IN_PROGRESS / 최종 검증·게시 미완료.** 아래 PASS는
각 개발 스냅샷의 제한된 증거이며 최종 b SHA의 전체 게이트가 아니다.
원본 근거는 `/Users/rexxa/work/spine-data/`에 보존한다. 실행 중 결과는 최종
영수증을 받은 뒤 채운다. 마일스톤 #13 전체 완료를 선언하지 않는다.

### 구현 의도

기존 조각 건설을 데이터 기반 건물 계획으로 묶어, 실제 재료·작업·완료와
누가 왜 지었는지를 저장·화면·연대기에서 이어서 확인하게 한다.

### 구현 내용

- `rust/crates/sim-core/data/colony/buildings.ron`: Connected6개(hut,
  log_house, stone_house, workshop_shed, storehouse, drying_shed), Inert16개.
  같은 카탈로그의 `zone_kinds`는 housing Connected, storage/workshop Inert다.
- `colony/building_template{.rs,/}`: 선택 카탈로그 안에서 계획·회전·물리
  발자국·작업 타일·실제 blueprint 비용과 구역 참조를 검증한다.
- `colony/building_site.rs`, `runtime/building_templates/`: BuildTemplate,
  UpgradeBuilding, AddHousingZone과 영속 aggregate/piece 상태·원인,
  기존 조각 활성화·물리 완료 callback·보관 기능·업그레이드 연결.
- format19 sidecar, 이전 저장 변환과 의미 검증. 주거 구역은 stockpile과
  별도 색인이고, 집사는 지식·실제 미약정 재료가 있는 최고 tier를 고른다.
- bridge의 카탈로그·계획 미리보기·site/options·과거 원인·배정 침상 기반
  집 읽기. 기술 지도는 a 결과를 유지하고 건물 패널에 지은 것/지을 수 있는
  것/다음에 열릴 것을 추가했다. 원인은 명령/주거 구역/집사 요청으로 읽는다.
- 자리표시 아트: Round/Angular 지붕×기존 재료 tint×glyph10종, 아이템
  shape8종×tint. Inert는 미리보기에서만50% alpha와 빗금으로 표시한다.
  실제 세계의 완성 지붕은 저장된 site와 물리 intact 증거를 요구한다.

### 구현 방법

기존 blueprint와 재료 비용을 그대로 전개하며, site-id 순서와 authored
piece 순서로 기존 작업을 활성화한다. 비바닥 조각·작업 타일은 겹치지 않고,
바닥과 가구의 허용된 겹침은 바닥을 먼저 완성한다. 계획 생성·시간 경과·
미리보기는 물리 완료를 부여하지 않는다. storehouse는 기존 stockpile 경로를 쓴다.

기존 물리 ColonySite에는 영속 SiteId 할당기가 없고 타일/ECS를 사용하며
완료 시 entity가 사라진다. 따라서 aggregate/piece provenance에 checked
단조 카운터를 추가했다. 주거 구역은 기존 next_zone_id를 재사용하고
workshop/bill/room 할당기를 빌리지 않는다.

완료 이벤트60은 완료 당시 template ordinal과 site/accepted command record/
origin을 고정한다. 업그레이드 뒤에도 이전 집의 이름·원인을 현재 site로
재해석하지 않는다. save19는 기존 중첩 positional wire를 바꾸지 않는
sidecar이며, 복원 후 물리·명령·연대기 의미 검증과 방 연관 재구성을 거친다.
카탈로그 revision은 ordinal 의미를 고정한다.

주거 후보는 Connected·지식 충족·유효 전개·실제 재료 가용성으로 거른 뒤
높은 tier, 동일 tier에서는 id 순으로 고른다. 활성 필요량·운송 약속·대기
건설 비용을 반영하고 미래 생산량을 가용 재료로 세지 않는다. 구역이 있으면
전체 발자국이 한 주거 구역에 들어가야 한다. a의 시대 진척은 시대별 authored
양의 가중치 합에 대한 성인 실천자 coverage이며 이 b에서 수치 조정하지 않는다.

### 기능 설명

플레이어는 건물 종류와 작업장을 골라 회전·배치하고, 실제 주민의 운반과
건설을 기다린다. 주거 구역은 집사의 집 배치 범위를 정한다. 주민의 집은
현재 위치가 아니라 실제 배정 침상에서 확인한다. 지은 건물과 연대기에서
원인·단계를 보고, 다음 건물의 부족 지식/재료 또는 준비 중인 이유를 읽는다.

### 게시 SHA와 게이트 결과

a는 `c467c0a36466ec86419ae66bfb702e37c2566b30`, 2026-11-08
01:01:16KST에 v8/wip→v8/main fast-forward 게시했다. 5차 전체49/49 PASS,
기존 step41 면제만 사용했다. DEBUG3119/RELEASE 원본3118 PASS는 a 결과이며
RELEASE interleave 증거와 원본을 유지한다. [a 보고서](M_SPINE_A_REPORT.md) 참조.

**b 게시 SHA 미정·미게시. 전체 게이트 NOT_RUN, b 면제 적용 결과 없음.**
초기 개발 스냅샷 core529 PASS, 집중 runtime14 PASS, Rust 데이터 전용 확장
PASS는 최종 SHA 전체 통과를 뜻하지 않는다. 실제 import에서 발견한 rotation
이름 충돌을 수정한 후 기존 strict 검사89파일·79FFI 호출 PASS를 확인했다.
네이티브 UI 시나리오·시각 수용은 아직 NOT_RUN이다.

근거: `b-core-initial/{receipt.json,test.log}`, `b-focused-clockfix1/`,
`b-extension-rust-first.log`, `b-native-preparation/{import.log,strict-first.log,
strict-first-receipt.json}`. API/schema 정적 문서는 별도 검사이며 실행 증거를
대체하지 않는다. 최종 SHA·단계 수·exit·면제·source/binary binding은 후속 기입한다.

### 사슬 시험·데이터 전용 시험 결과, 캡처 경로

- 실제 BASE format17 및 게시 a에서 만든 format18→19 roundtrip·1440tick
  쌍 이어하기 PASS. `b-preliminary-predecessor-save-receipt.json`은 clock
  fix 전 개발 스냅샷이라고 명시하며 최종 head 재검증이 필요하다.
  원본: `b-save17-first.log`, `b-save18-first.log`,
  `base-v17-save.bin`, `a-published-c467-format18-save.bin`.
- 선택 팩의 실제 일반 BuildTemplate·Inert 거절·debt·저장 읽기는
  `b-extension-rust-first.log`의 Rust 시험 PASS. 네이티브 선택 팩 카드/
  미리보기/아이콘 검증은 NOT_RUN이다.
- `scene/chain-attempt1` 진단 중단, attempt2 경계 fixture 실패,
  attempt3 계절상 flax 부재, attempt4 실제48일 수확 FAIL을 보존한다.
  attempt5는 현재 실행 중이며 사슬 PASS를 주장하지 않는다. 완료/업그레이드와
  침상·바닥 등 실제 단계00..04의 JSON/wsave는 해당 attempt 원본에 남겼다.
  이 저장 단계 파일은 아직 네이티브 PNG 캡처나 최종 사슬 완료 증거가 아니다.
- 일시정지 명령 경계의 원인 조회 결함 수정 후 회귀2개 PASS:
  `scene/chain-attempt5/{bridge-tests.log,bridge-tests-receipt.json}`.
  과거 완료는 실제 실행 완료 증거를 요구하고, 현재 일시정지 명령은 실제
  accepted boundary까지만 허용한다. 이 수정은 나머지 사슬 성공을 뜻하지 않는다.
- 최종 `spine_chain_log_house`,00..04 실제 화면, crafted cord 실제 세계,
  8shape×tint/10glyph/Inert gallery, ko/en·크기별 UI 영수증은 후속 기입한다.

### 크래시 감지 결과

**b NOT_RUN.** BASE/후보 생존 배열, KnowledgeLost 배열, 새 전멸 여부 및
Inert로 되돌린 항목은 아직 없다. a의 seeds0–7×48일 PASS를 b 결과로 옮기지
않는다. b 실제 검증 후 원본 경로와 비교를 채우며, red이면 문제 항목을
한 번 Inert 처리하는 brief 규칙을 따른다. 수리 반복·held-out·보정·수치 튜닝은 없다.

### CONTENT_DEBT 항목 수

현재 카탈로그: **노드10 / 템플릿16 / 아이템4 / 예약 구역2**.
예약 storage/workshop은 자동 성장 planner 미연결이며 수동 Connected
storehouse/workshop 건설과 별개다. 기존 M6b 수동 debt는 유지한다.
생성 원천은 기존 `content_debt` 공유 renderer/실행기이며 최종 gate에서
실제 문서 일치를 다시 확인한다.

### 새 도구 줄 수와 그 이유

현재 집계 whole-pass **2964/3000줄**, 더 엄격한 시간순 positive-additions
**2975/3000줄·여유25줄**. `b-tooling-post-fixture-delta.json`이 앞선
`b-tooling-budget-integrated-audit.{json,md}`에 fixture delta65줄을 더한다.
이후 변경 시 재집계해야 한다. 외부 준비 코드도 면제 없이 포함했다.

- 공유 debt renderer87+실행기24: 기존 authored debt의 문서 불일치를 검출/
  재생성하는 경로. a의 실제 불일치 검증과 같은 기존 도구를 확장했다.
- spine_save_check90: 기존 probe에 load 인수가 없어 실제 이전 저장의
  변환·이어하기를 검증할 수 없었다. 신규 범용 runner 대신 작은 검사다.
- spine_tech_test224, 기존 m4 추가25: 기존24카드/643상한 화면 시험의
  실제 스키마 불일치를 다룬다. 기존 data_only_ext 추가149와 mart2 gallery
  추가138, spine_build_test500은 확장 팩/새 건물 조작·도형 검증을 위해
  작성했으나 해당 네이티브 실행은 대기다. 실행 전 성과를 주장하지 않는다.
- 사슬/단계/fixture·동작 helper 전체와 probe colony 추가96줄까지 모두
  도구로 청구한다. attempt2 경계, attempt3 계절 원료, attempt4 실제 수확
  실패와 원본을 보존하며 fixture 수정이 물리 동작을 대신하지 않게 한다.
- 외부 preparation4개446줄(manifest115, root seams124, planning101,
  save106), 별도 미사용 capture85줄, 기존 CLI/probe 호환 추가38줄도 포함한다.
  provenance manifest와 planning/save 생성기는 통합 실수 방지 목적이지 각각
  관측된 실행 실패로 필요성이 증명된 도구가 아니다. 따라서 **수치 상한
  충족이 brief의 “새 도구마다 실제 실패 동기” 요건까지 충족했다는 뜻은 아니다.**
  이 동기 gap은 남아 있으며 최종 보고에서 숨기거나 가상 실패로 메우지 않는다.

전체 파일별 길이·해시는 위 원장과 `b-tooling-external-preparation-audit.md`에
남긴다. 줄 수 PASS는 runtime·시각·최종 도구 동기 수용·출판 PASS가 아니다.

### 확인된 제한사항·후속

- 현재 사슬 attempt5 실행 중. b 최종 동일성·결정론·전체 저장·clippy/tests·
  크래시·paired 성능·전체 게이트·게시 **NOT_RUN/미완료**다. 초기 focused/
  core/이전 저장 성공은 최종 통합 head 검증을 대신하지 않는다.
- connected 업그레이드는 log4×4→stone4×4만이다. hut3×3을4×4로 확장하지
  않으며 해당 upgrade edge는 없다. workshop/drying4×4는 기존2×1 station과
  남쪽 work tile을 수용하는 최소 실행안이다. 기본 재료 합계 hut12log,
  log25log, stone33stone+3log, workshop29log+2stone, storehouse23log,
  drying27log는 실제 blueprint 합이며 brief 예시와 차이가 있다. 숫자 튜닝이 아니다.
- 바구니 운반 등 미연결 기능과 후대 건물16개, 예약 자동 성장 구역2개는
  authored debt에 남긴다. 새 온도 모델이나 storage/workshop 자동 성장 job을
  연결한 것으로 표시하지 않는다. C1/C2 후속 범위는 각 debt의 실제 표기를 따른다.
- **보호 경로 사고 공개:** a attempt3 단계21에서 상속된 PLAY_ROOT 기본값을
  사전에 덮어쓰지 못해 금지된 `new-world-play/main`·`m1`에서 빌드/import/
  실행이 발생했고, main HEAD가
  `fbc03bddb8f5b68f43fba1b2dad283982e5cf436`→
  `68beae3afb5d8e83ebc171746e5cf30a20531e20`으로 바뀌었다. 소유 프로세스만
  종료해 생존0을 확인했으며 reset/복원/정리/삭제로 덮지 않았다. 원본 상태·
  reflog·감사는 `fullgate-a-attempt3/{protected-path-side-effects.json,
  ABORTED_PROTECTED_PLAY_ROOT.json}`에 보존한다. 현재 PLAY_ROOT/PLAY_DATA는
  모두 명시적 외부 경로다. 이후 통과로 사고나 앞선 실패를 취소하지 않는다.
- 최종 b SHA/49단계 결과·크래시 배열·성능·실제 PNG/시각 검토·게시 영수증은
  검증 뒤 채운다. 현 문서는 WIP이며 게시 완료 보고로 사용하지 않는다.
