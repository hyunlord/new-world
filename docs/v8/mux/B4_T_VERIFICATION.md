# B4 T — 거래 품목 희소성·희귀성 검증

기준 P 커밋은 `bbae211b3ee8cf8651cbe43d1014831b8a606001`이다. D41/ADR0042에 먼저 등록한 의도적 동작 변경이며, 원시 자료는 `/Users/rexxa/work/mux-data/stage-b/b4/`에 보존한다. 전체 B gate·게시·홀드아웃 판정은 대기다.

## 정책과 저장 경계

실제 살아 있는 Agent+Knowledge 성인 중 기준 숙련도와 최근 수행 기간을 충족하는 수를 거래 생성 때 계산한다. 캐시된 수행자 수는 읽지 않는다. 이웃이 생산 기술을 아는 품목만 대상으로, 등록된 희귀 시대·수행자0·진행도750미만이면 가중치1을 먼저 적용하고, 나머지 수행자2이하면30, 생필품은10이다. 품목 순서·중복 행·keyed draw 슬롯·수량·품질·물물교환과 기존 unknown_bought 의미는 유지한다. 동일 후보에서 3/1을30/10으로 함께 늘리는 것 자체는 선택 구간을 바꾸지 않는다.

정책은 저장되는 NeighbourRule 밖의 optional 데이터이며 모든 새 가중치와 희귀 시대를 한곳에 둔다. 생산 코드가 Neolithic 이름으로 분기하지 않는다. 기존 저장의 규칙3/1과 이미 생성된 물품은 유지하지만, 로드 후 새 거래는 설치된 정책을 따른다. 이전 빌드와의 연속 실행 동일성을 주장하지 않는다. 데이터 검증은 전달된 Catalog만 사용하고 catalog 초기화가 끝난 뒤 neighbours 캐시를 만든다. neighbours.ron은 catalog_revision에 포함되지 않으므로 별도 원문 hash로 고정한다.

## 시험과 검토

기존 P에서 새 시험7개를 실행하면 기존3명 이상 제외의 양성 대조1개 통과·새 정책6개 예상 실패다(`t-tests-before-first.log`). 구현 뒤 전체15개가 통과했다(10.77초). 최초 설명의16개는 집계 오류였으며 실제15개(기존7+추가8)로 정정한다. 숙련도·최근성·아이/비Agent·미래 수행일·날짜 포화·진행도249/250/749/750·실제 caravan 생성·strict parser·u32 최대 등록 item·데이터만 바꾼 희귀 시대·옛 draw 벡터·저장된 물품을 확인한다.

독립 검토는12개 파일의 최종 hash를 다시 확인하고 차단 결함을 찾지 못했다(`t-independent-review.md`). 관측기는 실제 정책 tuple과 era를 독립 검증하며 생산 분류 helper를 호출하지 않는다. 별도의 Python 산술 재계산이 RNG 공식 증거를 맡는다. 옛 벡터 단위시험은 기존 keyed RNG를 재사용하므로 독립 RNG 구현 검증이라고 주장하지 않는다.

기존 이웃16·저장7·실제 과거 저장 이행4(M5.5/M-ART2/StageA/core8)·core 규칙2·runtime 거래2·observer8이 통과했다. 최초 clippy는 새 저장 시험의 Copy 타입 clone 한 곳으로 실패했고, 동일한 직접 복사로 바꾼 뒤 경계15·fmt·strict clippy가 통과했다(ISS292, 첫 실패 보존). 제품2111경로를 고정했다. 선택 팩 시험도 부모2개와 별도 프로세스9개가 통과했다. 원본 identity228748개 값/0차이, Chronicle64파일/19683항목의 기존 git_sha 예외 및 정규화 raw digest, 재현성 검사가 통과했다. 9개 OFF 모두 정확한 시드 범위와 불변식 위반0을 확인했고2111개 제품 경로 hash가 유지됐다. 변경 원문을 부모 커밋에 잘못 연결하지 않기 위해 T 커밋 후 정확한 commit-linked arithmetic 입력으로 보정을 수행한다. BPT 보정과 네 빌드 고정 전에는 홀드아웃을 열지 않는다.

## 의도적 동작 차이

`t-safety1-receipt.json`과 각 `t-*-differences.json`에 원문과 경로별 차이를 보존한다. knowledge_f10, m55_noneighbours, m55_quiet는 문자 그대로 동일하다. 변경 경로 수는 soak124, story283, neighbours4463, m55_pressure17283, m55_nodraft17797, m55_storyoff16527이다. soak/story도 이웃이 켜진 입력이며, 차이는 교역 관측과 colony fingerprint에 한정된다. 전체 결과를 동작 보존으로 부르지 않는다.

m55_pressure 시드0의 첫 caravan은 양쪽 모두 id1·이웃0·day9·도착13237이며 품질 draw 여섯 값55/30/56/49/35/40을 유지한다. P의 hide/meat/flint 후보가 T에서는 clay/fur_cloak 후보로 바뀐다. 최종 caravan7·거래7·이주 수락2·합류8·eureka0·unknown_bought0은 같지만 지불 가치456→616, 받은 가치285→287, 내보낸 blade125→166으로 달라진다. 이후 M4/M5·경제 차이는 이 단일 정책 변경 후 나타난 결과이며 모든 중간 원인을 각각 증명한 것으로 주장하지 않는다. 원래 golden은 수정하지 않았다.

독립 원시 자료 검토(`t-fixture-diff-review.md`, `t-fixture-review-data.json`)는 요약18개 hash·기록된 모든 diff 값·정규화 raw digest128회 재계산·Chronicle64개 git_sha 이외 동일성을 확인했다. soak/story는 구매0이며 교역 goods/offered/goods_left와 fingerprint suffix 외 차이가 전혀 없다. 구매 시나리오의 첫 M4/M5 일별 차이는 첫 달라진 제안 도착보다 앞서지 않는다. 차단할 구체적 범위 밖 결함은 발견되지 않았으나 일별 요약으로 모든 후속 인과를 입증하지는 못한다.

## 한계

raw ron deserialization과 의미 검증 parser는 다르며 runtime은 후자를 사용한다. 이번 검증은 상품 관련 범위·참조를 엄격히 확인하며 모든 과거 규칙 필드를 일반화하지 않는다. 이주 의복의 기존 저장 Top 두 슬롯과32노드 지식 영역은 그대로다. 기존 저장을 읽은 뒤 미래 거래가 바뀌는 것은 명시적 정책 효과다. LSP 불가 ISS284와 기존 UI·A8 제한은 해결된 것으로 표시하지 않는다.

## 커밋 후 보정

T `54a55a421361d9c24268e1b3a6a5d2c2d1c6c211`의 실제 원문23개를 연결한 BPT 보정8×48일은 VALID/NOT_CLAIMED다. 제작1644건 산술과 통제 Cold2304행·제작32건이 검증됐다. 자연 Cold는 의복0/9487·무의복0/6329로 비율 NOT_MEASURED이며 인과 보호 효과를 주장하지 않는다. 첫 비교기는 각 제작 행의 출처 manifest hash까지 빌드 간 같아야 한다고 잘못 검사했다(ISS293). 각 hash를 자체 명세와 대조하고 나머지 모든 값이 같음을 확인했으며 원시 결과·정책·scorer는 수정하지 않았다. BPT32파일 hash와2111제품 원문 hash가 검증됐고 빌드가 고정됐다. 홀드아웃은 아직 실행하지 않았다.

독립 보정 감사(`t-calibration-independent-review.md`)는 archive32파일·참조10개·T commit의2111 blob·산술23원문을 재검증했다. BP/BPT caravan108개와 자연 제작3417건 및 통제 제작32건·Cold2304건을 독립 재계산해 불일치0이다. BPT scarce 제안73스택, rare 후보는14caravan/15행이었으나 선택0으로, 희귀품 불가능이나 개선 실패로 해석하지 않는다. 자연 제작은1773→1644이며 일률적 생산 개선도 주장하지 않는다. 양쪽 운반 종료80개는 명시적 UNRESOLVED로 남아 성공 배달로 간주하지 않는다. 별도 artifact의 비교이며 단일 BPT scorer index의 formal paired NOT_RUN 상태를 바꾸지 않는다.
