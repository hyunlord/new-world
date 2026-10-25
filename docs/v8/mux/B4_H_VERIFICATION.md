# B4 H — 사냥 고기의 화덕 운반 검증

기준 T는 `54a55a421361d9c24268e1b3a6a5d2c2d1c6c211`이다. D42/ADR0043에 사전 등록한 의도적 동작 변경이며 BPT 보정·archive32파일 고정 후 시작했다. 원시 자료는 `/Users/rexxa/work/mux-data/stage-b/b4/`에 보존한다. 전체 B gate·게시·홀드아웃 판정은 대기다.

## 변경 경계

수용 가능한 비축 구역을 찾는 기존 순서를 그대로 유지한다. 없을 때만 사냥꾼이 grazer_meat 역할의 고기를 실제 화덕 건물이 존재하고 현재 hearth anchor 반경2 안인 안전한 지면으로 운반한다. 현재 source 거리·y·x 순서로 선택하며 source 자체와 다른 작업자의 예약을 제외한다. 기존 ItemSpec 적재 공간·이동 가능성·운반량 제한을 재사용한다.

줍기 전에는 실제 source spec, 운반 중에는 실제 Carry spec으로 목적지를 검증한다. 마지막 화덕·anchor coverage·안전한 공간 등이 사라지면 기존 취소 경로가 고기를 내려놓고 예약을 해제한다. 양의 남은 공간이면 기존 merge/spill 규칙을 따르며 전체 짐이 한 칸에 모두 들어간다고 가정하지 않는다. 비축 구역 경로에는 새 화덕 조건을 적용하지 않는다. 순간 식량 지급·새 job/save 필드·일반 운반 지정·loot·feed 변경은 없다.

## 현재 시험 근거

변경 전 phase1은 정상 대조4개 통과, 실제 kill→Haul이 없어서3개 예상 실패다(`h-before-first.log`, 실행0.02초). 변경 후 같은7개가 모두 통과했다(`h-phase1-after-first.log`, 컴파일17.92초·실행0.04초). 사냥 명령·실제 kill·source 줍기·Carry 이동·지면 배달·이후 시간별 AnimalSystem feed를 각각 관측했다. 화덕 제거 전후에도 고기·가죽·ledger·예약 보존을 검사한다.

시험 세계는 명시적 보장 명중/짧은 사냥 및 제한된 시스템 스케줄을 쓴다. 실제 운반 메커니즘의 증거이며 자연48일 생존 성능은 아니다. 독립 생산 소스 검토(`h-independent-review.md`)는 두 파일의 최종 hash를 확인하고 구체적 차단 결함을 찾지 못했다. 추가 경계·저장 재개·명령 일정 replay를 포함한22개가 첫 실행에서 통과했다. 독립 검토가 사전 계획의 anchor-only 상실 조건 누락을 발견해, 실제 화덕 건물이 남은 상태에서 anchor만 사라지는 ToSource/Carry 두 경우를 추가했다. 최종23개 통과(0.25초), 전체 strict clippy18.71초·변경 target 재검사10.22초·fmt 통과다. 기존 가죽5·작업9·야생동물12·저장7도 통과했다(저장7개 직렬351.77초). 독립 검토는 최종5파일 hash를 다시 확인하고 해당 필수 경계 공백을 닫았다. 제품2114경로와 probe `9e38284555a1b8325bd19654fd74f274c212530ffd76f39d0c71f4cbfb047967`를 고정했다. 새 원본 identity228748값 차이0·Chronicle64파일/19683행 동일(기존 git_sha 예외), 9 OFF 정확한 시드 집합과 불변식 위반0을 확인했다.

## 남는 한계

저장된 Haul에는 사냥 기원 표지가 없다. 실행 검증은 고기와 목적지의 현재 적법성을 확인하며 사냥 기원을 증명하지 않는다. 실제 화덕 건물 하나가 남으면 여러 formation anchor에 대한 전역 존재 조건이 성립하고 feed는 기존 첫 정렬 settlement를 쓴다. 정착지별 소유 운반을 새로 보장하지 않는다. in-test 명령 일정 replay는 CLI 명령 파일 parser/replay 증거와 구분한다. 관측기의 UNRESOLVED 종료는 성공 배달로 바꾸지 않는다.

## 보정

고정된 H source의8세계×48일 보정은 VALID/NOT_CLAIMED이며 자연 제작1665건을 검증했다. 통제 Cold2304행·제작32행은 BPT와 전체 행이 같다(같은 산술 manifest, arm/envelope는 구분). 보정 F4a/F9a는 각각8/8이나 홀드아웃 수용 판정이 아니다. 자연 Cold 의복0/9067·무의복0/6665로 자연 비율은 NOT_MEASURED다. 현재 provenance는 H 커밋 전 source-freeze와 부모T를 명시한다. H 자체 커밋 뒤 최종 archive manifest를 고정한다. 원본 identity·9OFF 재캡처와 독립 차이 검토를 완료했다.

독립 보정 검토(`h-calibration-independent-review.md`)는 참조10개·제품2114경로·산술23원문과 scorer binding을 확인하고 통제 Cold2304·제작32 및 H 자연 제작1665건을 재계산해 불일치0을 확인했다. T의 stockpile 운반 시작80개에서 H는 stockpile79·hearth28개로 바뀌었다. H 종료107개 모두 UNRESOLVED이므로 자연 성공 배달107개로 부르지 않는다. ledger/reservation 위반0·시작 누락0·끝에서 진행 중0이다. 고기 eaten325→318, spoiled16→14, dropped225→228로 일률적 식량 개선은 관측되지 않았다. 메커니즘의 완결 배달 증거는 별도의 통제23시험이다.

## OFF 재캡처

`h-safety1-receipt.json`은 제품2114경로 불변과 원본 기준선 동일을 확인한다. T 대비 soak·knowledge_f10은 literal 동일하다. story1434·neighbours871·m55_pressure7172·m55_nodraft4164·m55_storyoff5924·m55_noneighbours2094·m55_quiet2905개 경로가 달라졌다. H는 이웃 기능과 독립적인 사냥 운반 변경이므로 neighbours OFF도 동일성 대상이라고 가정하지 않는다. 변경 전후 요약과 모든 차이 파일을 보존하며 golden을 수정하지 않는다. 정확한9개 fixture 시드 집합과 ledger/reservation 불변식 위반0을 확인했다. 독립 차이 검토 결과는 아래와 같다.

독립 `h-fixture-diff-review.md`는18개 요약 hash·9개 전체 차이 목록과128개 정규화 raw digest·128개 Chronicle hash를 재계산했다. 첫 fingerprint 차이는 시드별22–43일이며 모두 첫 사냥 관측 뒤다. 사냥 없는 soak·knowledge_f10은 동일하다. 변경 fixture의 고기 운반 집계는 story22→29, neighbours24→29, pressure80→107, nodraft84→98, storyoff79→102, noneighbours90→105, quiet89→103이다. 고기 eaten은 각각61→65,75→82,325→318,294→330,296→288,185→189,193→191로 모두 좋아지지는 않는다.

사냥 뒤 추가 운반·위치·소비·작업 시간이 바뀌어 지식·기분·제작·교역·인구 차이로 이어질 수 있다는 설명은 소스·일별 시점과 일치하는 추론이다. 일반 요약은 정확한 최초 fallback tick·당시 비축 거절·각 운반 종료를 기록하지 않아 경로별 인과나 성공 배달을 증명하지 않는다. 독립 검토에서 차단할 artifact 불일치나 구체적 범위 밖 결함은 찾지 못했다. 새로운 시드 평가 전 고정하며 수치·golden 조정은 없다.
