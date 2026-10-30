# M6a R1 원인 진단 — 제품 수정 전 확정 기록

근거: planner/owner decision **2026-10-30 11:20 KST**, [R1 계약](M6A_DECISIONS.md#repair-round-r1--owner-decision-2026-10-30-1120-kst). 제품 소스는 P9와 동일한 P13 `e8d5c1668` 상태다. P14 `c2d8193bf`는 프로토콜 문서만 추가했다. 원본1296–1327은 진단용으로 은퇴했으며1328–1359는 아직 실행하지 않았다.

## 1. 사망별 인과 사슬

기아사 **66건 전수**를 확보했다. 외부 read-only 관측기는 생산 엔진의 starvation priority139 직전138에서 사망 사슬을 기록했다. soak4개는20일 일별 fingerprint와 사망/KnowledgeLost가 모두 일치했고, 은퇴 held-out32개는 최종 fingerprint·식량 tally가 모두 일치했다. held-out29명의 agent/tick도 원본 사건과 일치한다. **36/36 재현 동일성 PASS**, 제품 Rust/data 변경0. 이는 진단 재현이며 평가·성능 결과가 아니다.

|분류|건수|확인된 직접 원인|상위 원인과 한계|
|---|---:|---|---|
|A|58 (soak37, held-out21)|사망 당시 물리 음식0·익은 식물0|soak는 기존 자율 비축 정책의 M6a 이식 누락을 확인. 수리 전에는 모든 겨울 고갈을 이것 하나로 설명하지 않음|
|B|7 held-out|식량은 선반에 있고 실제 먹을 대상은 없음; meal 노드가 수확보다 높은 점수로 반복 선택|계획용 reserve와 섭취 가능한 owner의 admission 불일치|
|C|1 held-out|수확 중단 뒤 먼 음식 추적, 도착 전 고갈; 물/수면 전환 후 사망|마지막16개 선택에는 B의 반복 증거가 없어 별도 보존|

B의 `camp_stocked`는 Shelf를 포함하지만 실제 eater는 Shelf를 제외한다. `meal_idle_water_not_critical`은 `food_available` 검사 없이 score1,000,000으로 유효한 수확보다 먼저 선택되고 대상 없는 lifecycle이 즉시 종료된다. 계획용 reserve에서 선반을 지우는 대신 실제 섭취 admission을 일치시키는 수리가 필요하다.

A의 soak는 steward OFF, 음식 bill 없음, 생식 수확량=섭취량428/425/428/427, 부패0·잔량0이다. BASE5b0c8881은 steward와 무관한 forage/gather 작업6개가 기존 `stock_shortfall`/`stock_hoard_order`를 사용했다. M6a는 물리 stock target/cap 사실을 유지했으나 배고픔+음식 없음의 수확2개만 남겼다. 기존 겨울 대비 target/cap 아래 자율 수확을 물리 Job으로 복원하는 것은 이식 결함 수리다. steward 활성화·새 자동 조리·target 수치 변경은 하지 않는다. lifecycle의 routine 상태 정리, 취소·예약·player veto를 회귀 검증해야 한다.

C는1307/person6, 마지막 식사80899, 수확81461→81462 Needs 취소,82167→82187 취소,82607→82622 취소다. 세 번째만 실제 `critical_hunger_job_hunger` 선택 기록이 남는다. 이후(97,142)에서(125,87)의 음식으로 향했으나 도착 전에 없어졌고83285에 사망했다. 첫 두 번의 정확한 취소 노드는 추정하지 않는다.

각 행의 마지막 식사, 당시 품목별 모든 owner/저장량, 도달 가능한 식물, 예약, 열린/소유 harvest/cook/haul 작업과 사람, 마지막 MindChoice·blocked 사실·top3, 집사 목표는 [66건 전체 상세 JSON](</Users/rexxa/work/m6a-data/20261030-r1/death-summaries.json>)에 raw trace 경로/행과 함께 있다. 마지막 식사 시점의 저장/작업 상태는 [식사 시점62개 관측](</Users/rexxa/work/m6a-data/20261030-r1/last-meal-contexts.json>)으로66명에 연결했다. 이는 식사 전 순간이 아니라 같은 engine tick 종료 상태다. 아래는 사망 색인이다. 현재 provider 관측과 당시 실제 MindChoice를 구분하며 MissingDerived는 오류로 남긴다.

[재현 동일성](</Users/rexxa/work/m6a-data/20261030-r1/replay-identity.json>), [관측 범위·한계](</Users/rexxa/work/m6a-data/20261030-r1/food-diagnosis.md>), [C의 전체 사슬](</Users/rexxa/work/m6a-data/20261030-r1/heldout-1307-agent6-chain.json>), [기존 비축 정책 대조](</Users/rexxa/work/m6a-data/20261030-r1/gate-protocol-plan.md>).

|mode|seed/person|death tick/day|lastmeal tick:item|stored kinds|reachable ripe units|class|
|---|---|---:|---|---|---:|---|
|heldout|1300/9|85150/59|82723:Roots|{'Berries': 1}|10018|B|
|heldout|1300/5|85969/60|85207:Roots|{}|14682|B|
|heldout|1300/3|96639/67|94171:Hazelnuts|{}|0|A|
|heldout|1300/4|96644/67|94174:Hazelnuts|{}|0|A|
|heldout|1300/2|97228/67|96010:DriedBerries|{}|0|A|
|heldout|1300/0|97240/67|94028:Meat|{}|0|A|
|heldout|1300/6|97278/67|94261:Meat|{}|0|A|
|heldout|1300/29|98066/68|94402:Hazelnuts|{}|0|A|
|heldout|1300/7|98490/68|95656:Meat|{}|0|A|
|heldout|1300/24|98546/68|96007:DriedBerries|{}|0|A|
|heldout|1300/1|98674/68|95608:Meat|{}|0|A|
|heldout|1305/1|11555/8|9208:Roots|{'Roots': 2}|9638|B|
|heldout|1305/3|12149/8|9520:Berries|{'Roots': 3}|9625|B|
|heldout|1305/0|13485/9|12232:Roots|{'Roots': 2}|9629|B|
|heldout|1305/4|14425/10|13810:Roots|{'Roots': 2}|9637|B|
|heldout|1305/6|27539/19|24718:Berries|{}|0|A|
|heldout|1305/9|28292/19|26104:Roots|{}|0|A|
|heldout|1305/2|28819/20|26408:RoastedRoots|{}|0|A|
|heldout|1305/8|29045/20|26411:RoastedRoots|{}|0|A|
|heldout|1305/5|29423/20|26890:DriedBerries|{}|0|A|
|heldout|1307/6|83285/58|80899:Roots|{'Roots': 3}|9656|C|
|heldout|1307/4|83421/58|80641:RoastedRoots|{'Roots': 3}|9656|B|
|heldout|1307/0|96851/67|94082:Hazelnuts|{}|0|A|
|heldout|1307/9|97347/67|94936:Hazelnuts|{}|0|A|
|heldout|1307/2|97561/68|96040:DriedBerries|{}|0|A|
|heldout|1307/3|98057/68|95323:Meat|{}|0|A|
|heldout|1307/20|98058/68|94282:Hazelnuts|{}|0|A|
|heldout|1307/7|98478/68|95371:Meat|{}|0|A|
|heldout|1307/19|98826/68|94933:Hazelnuts|{}|0|A|
|soak|0/6|28290/19|25897:Roots|{}|0|A|
|soak|0/1|28350/20|25894:Roots|{}|0|A|
|soak|0/9|28390/20|25813:Roots|{}|0|A|
|soak|0/3|28450/20|25723:Berries|{}|0|A|
|soak|0/0|28630/20|25894:Roots|{}|0|A|
|soak|0/7|28690/20|25921:Roots|{}|0|A|
|soak|0/8|28750/20|25936:Roots|{}|0|A|
|soak|0/4|28750/20|25963:Roots|{}|0|A|
|soak|0/5|28790/20|26044:Roots|{}|0|A|
|soak|1/6|28090/19|25291:Roots|{}|0|A|
|soak|1/9|28150/19|25825:Roots|{}|0|A|
|soak|1/4|28190/19|25825:Roots|{}|0|A|
|soak|1/3|28210/19|25636:Berries|{}|0|A|
|soak|1/7|28290/19|25777:Roots|{}|0|A|
|soak|1/8|28390/20|25768:Roots|{}|0|A|
|soak|1/2|28450/20|25690:Berries|{}|0|A|
|soak|1/1|28630/20|25837:Berries|{}|0|A|
|soak|1/5|28750/20|26119:Roots|{}|0|A|
|soak|2/4|28290/19|25780:Hazelnuts|{}|0|A|
|soak|2/3|28350/20|25816:Hazelnuts|{}|0|A|
|soak|2/1|28450/20|25693:Berries|{}|0|A|
|soak|2/9|28610/20|25816:Hazelnuts|{}|0|A|
|soak|2/2|28630/20|25855:Berries|{}|0|A|
|soak|2/5|28690/20|25870:Hazelnuts|{}|0|A|
|soak|2/6|28750/20|25909:Berries|{}|0|A|
|soak|2/8|28790/20|25960:Hazelnuts|{}|0|A|
|soak|2/7|28790/20|25954:Hazelnuts|{}|0|A|
|soak|3/1|28210/19|25852:Hazelnuts|{}|0|A|
|soak|3/5|28290/19|25777:Roots|{}|0|A|
|soak|3/2|28350/20|25852:Hazelnuts|{}|0|A|
|soak|3/0|28450/20|25693:Roots|{}|0|A|
|soak|3/8|28610/20|25855:Hazelnuts|{}|0|A|
|soak|3/3|28630/20|25813:Hazelnuts|{}|0|A|
|soak|3/4|28690/20|25867:Hazelnuts|{}|0|A|
|soak|3/7|28750/20|25924:Hazelnuts|{}|0|A|
|soak|3/6|28790/20|25975:Hazelnuts|{}|0|A|
|soak|3/9|28790/20|25960:Hazelnuts|{}|0|A|

## 2. 조리 비율 미달

[32세계·96연도 품목별 생산](</Users/rexxa/work/m6a-data/20261030-r1/cooked-diagnosis.json>), [bill·작업자·토기·곡물·열 관측](</Users/rexxa/work/m6a-data/20261030-r1/cooked-runtime-diagnosis.json>), [해석·인과 한계](</Users/rexxa/work/m6a-data/20261030-r1/cooked-runtime-diagnosis.md>).

|원인|실행 증거|수리 경계|
|---|---|---|
|1305 열·연료 없음|tick1440/2880/4320/7200/10080 불 꺼짐·연료0, Roots2 선반, RoastedRoots Maintain24 runnable·비중단, 조리 지식 보유자 생존|즉접 조리 차단 확인. 음식이 있으나 선반에서 먹지도 조리하지도 못함|
|연료 생산 지연|위 시점 Logs106/254/207/268/305, SplitFirewood runnable이나 완료0. 최초 공급12194·제작12269. 분할 자체는 열을 요구하지 않음|기존 우선순위 순서와 식사 빈 반복이 함께 존재. 독립적 우선순위 결함으로 단정하거나 수치를 조정하지 않고 식사 수리 후 재검증|
|겨울 조리 재료 고갈|비어 있지 않은 미달7세계 겨울 표본은 주로 견과·말린 열매, 곡물 없음.1312 day44 견과159+말린열매19·토기2, hearth empty·6 bills suspended|조리 선호 실패가 아니라 입력 없음. 모든 해당 생식 선택의 cooked_available 사실0|
|조리 목표와 재료 요청 불일치|cooking은 기존 daily_milli×cooked_days_milli 목표를 recipe별 배분하나 dependencies는 recipe별 한 batch 입력만 요청|기존 목표를 실제 recipe input/output 보존식으로 전파하는 수리 후보. 새 겨울 조리 비율·목표 수치를 발명하지 않음|
|겨울 비축의 메뉴 한계|durable reserve는 raw_edible만 포함해 보존 가능한 WildGrain을 제외. 견과만으로 칼로리 목표를 채울 수 있음|전체 겨울 조리 배분은 기존 정책에서 유일하게 결정되지 않음.50% bar를 거꾸로 목표로 쓰지 않음. 기계적 수리 뒤 충분성은 보정에서 검증|

all32_v1에는32세계의 실제 craft 할당자·tick, bill 활성/중단 표본, 토기·식량·작업장 경계 상태가 있다. 품목 ID는 WildGrain이다. 비겨울 곡물은31/32세계에서 관측됐고1305는 없었다. 겨울216표본 중1318만7개에서 곡물이 있었으며 나머지31세계는0개였다. 따라서 겨울 전체에 단 한 번도 곡물이 없었다는 주장은 하지 않는다.

조리 음식이 존재하는데 생식을 골랐다는 증거는 없다. 품목 보존식으로 추론한96연도 완료 생산량은 craft_output_units와 일치하지만 작업 할당은 완료와 다르다. runnable=true도 실제 열을 사용한 실행 허가와 다르다. 표본 사이 연속 상태는 추정하지 않는다. 전역 입력의 접근성 및 첫 작업장 선택 문제는 미확정 가설이며 수리 근거로 쓰지 않는다.

원본 평가 구간 days[24,72)의 전체32세계는 다음과 같다. raw=all−cooked이며0/0은 실패다.

|Seed|Cooked/all|Winter cooked/all|Nonwinter cooked/all|
|---|---:|---:|---:|
|1296|686/1173|16/293|670/880|
|1297|663/1165|12/298|651/867|
|1298|689/1193|18/303|671/890|
|1299|722/1272|17/323|705/949|
|1300|407/994|18/160|389/834|
|1301|754/1186|86/300|668/886|
|1302|862/1274|69/327|793/947|
|1303|645/1207|12/306|633/901|
|1304|746/1262|40/328|706/934|
|1305|0/0|0/0|0/0|
|1306|651/1304|15/335|636/969|
|1307|366/1032|17/158|349/874|
|1308|718/1286|21/333|697/953|
|1309|720/1352|18/358|702/994|
|1310|866/1195|94/299|772/896|
|1311|724/1166|17/290|707/876|
|1312|568/1219|20/319|548/900|
|1313|696/1314|30/339|666/975|
|1314|740/1276|104/330|636/946|
|1315|591/1198|22/309|569/889|
|1316|873/1266|71/331|802/935|
|1317|588/1086|29/250|559/836|
|1318|590/1222|33/318|557/904|
|1319|491/1222|26/317|465/905|
|1320|700/1227|38/322|662/905|
|1321|670/1222|47/317|623/905|
|1322|713/1209|23/303|690/906|
|1323|732/1273|21/317|711/956|
|1324|892/1243|43/317|849/926|
|1325|746/1199|145/308|601/891|
|1326|603/1175|18/291|585/884|
|1327|734/1251|79/331|655/920|

## 3. 반배율 가시성 원인

**동일 상태 반사실 촬영으로 확인**: 상단 Actions 버튼 배경/글자가 주민을 가린다. 식물은 원인이 아니며, 그림 순서/카메라/판정값 변경은 필요하지 않다.

|동일tick240·카메라·포즈의 조건|idle 유효픽셀|work 유효픽셀|
|---|---:|---:|
|원래 UI·식물|52|45|
|UI 숨김|193|213|
|식물만 숨김|52|45|
|둘 다 숨김|193|213|
|원복|52|45|

BASE 제품과 같은 카메라에서 이 주민은 원래 툴바 밖에 있었고, M6a 궤적에서는 툴바와 겹친다. 이전45→52는 포즈 차이로 재현되어 개선으로 세지 않는다. Actions의 엔진 기본 버튼 배경alpha0.60만 외부에서0.45로 낮추면89/98픽셀, 원복하면52/45였다. 최소 수리 후보는 해당 Actions의 Button/OptionButton 전용 스타일이며 문자·입력·배치·전역 테마는 보존한다. 이 실험은 제품 수정이 아니며 부하가 높은 두 번째 진단을 정식 수용 결과로 세지 않는다.

[전체 진단/수치/출처](</Users/rexxa/work/m6a-data/20261030-r1/visibility-diagnosis.md>), [수정 전 주민](</Users/rexxa/work/m6a-data/20261030-r1/visibility-crops/counterfactual-full.png>), [UI 없는 반사실](</Users/rexxa/work/m6a-data/20261030-r1/visibility-crops/counterfactual-ui-hidden.png>), [alpha0.45 진단](</Users/rexxa/work/m6a-data/20261030-r1/visibility-crops/person-alpha045.png>), [툴바 원본](</Users/rexxa/work/m6a-data/20261030-r1/visibility-crops/toolbar-before.png>), [툴바 진단](</Users/rexxa/work/m6a-data/20261030-r1/visibility-crops/toolbar-alpha045.png>). 정식 수리 후 낮은 부하에서 원래8장면 전체를 재검증해야 한다.

## 4. 산수와 실제 값

[24개 seed/year 수확 행·24개 겨울 행·보정1–4 원본 해시](</Users/rexxa/work/m6a-data/20261030-r1/arithmetic-diagnosis.md>).

설계는 반경16의 식물을 즉시 전량 수확하고 노동·이동·부패·날씨·수요 제한이 없는 이상적 공급 상한이다. 실제 보정4의 세계 전체 식물 수확 환산량은 그 지역 상한의16.62–72.41%로 모든 연간 행에서20% 이상 낮다. 비교의 범위와 가정을 구분해야 하며, 실제 노동/예약/운반 병목을 식물 밀도 증가로 덮을 근거가 아니다. 현재 증거는 지도 공급량의 불가능성을 증명하지 않는다. yields/densities 변경을 정당화하지 않는다.

성인10명 수요는 연576단위·겨울144단위다. seed5 첫 겨울 저장83/144는−42.36%, 세 번째 겨울104/130은정확히−20%다. 후자는 물리적 음식160 중56이 저장 밖에 있었고 경계에서뿌리50개가 썩었다. 실제 겨울 섭취 완화량이 성인 전용 기준을20% 이상 넘는4행은 어린이 수요를 포함하면 차이가약2.1% 이내로 줄어든다. 실제 전량 수요·수확 예측에는 인구·노동·이동·저장 경계를 포함해야 하며, 상한을 실현 가능한 수확 예측으로 취급한 모델의 간극을 보존한다. 수용 기준은 바꾸지 않는다.

## 5. 진단 범위 확정과 수리 경계

진단 문서 커밋 후 확정된 식사 admission·자율 비축 이식·조리 목표 입력 전파·가시성 원인만 수리한다. 남은 조리/일정 인과 가설은 수리 후 진단으로 재검증하며 별도 근거 없이 정책 수치를 바꾸지 않는다. 확정 원인마다 별도 실패 회귀→수리→통과와 리뷰 브랜치 커밋을 남긴다. R1 보정은 아직0/2, 새 held-out은 NOT_RUN이다. 기존44/48 실패는 그대로 보존한다.

## R1 calibration round1 follow-up (P21 source; before further repairs)

Round1 uses `b6c491ac68c1b2adb7bbe878011e122bd3c9fd95`, frozen probe SHA256 `5f015316b794660ffdbfb7578c93663d46a663f2b37a5f5b5d40484ea2479311`. All8×72day processes complete with unchanged source. Calibration counts: alive8, winter-hunger-free8, cooked8, diversity8, ledger8, stockpile8, beds8, wolf8; winter stores7; spoilage0. These are calibration observations, not held-out acceptance. Unchanged soak determinism passes, but starvation0/10/0/0 and KnowledgeLost0/0/0/0 fail the required survival bar. One of the two R1 rounds is consumed; fresh1328–1359 remains unopened.

The new v3 external observer preserves all20 daily fingerprints for seed1 and unchanged production hashes. All10 deaths occur with zero physical food and zero ripe food plants; exact victim/tick/last meal/job/choice/facts/steward chains are in `cal1-diagnostic-soak-1/death-chains.json`. The original soak summary has no independent victim/tick list, so those details are from the fingerprint-matched replay. Harvest471 = intake427 + spoil44, cooked0/lost0/remaining0. Full report: `/Users/rexxa/work/m6a-data/20261030-r1/cal1-seed1-death-diagnosis.md`.

A confirmed acquisition/transport defect precedes depletion: settlement exists at tick0, scripted storage appears360. Stock work harvests60 by120,135 by240,205 by360,250 by480 while stored pieces are0/0/10/54 respectively. Stored+carried stock accounting deliberately excludes loose ground; a failed harvest-to-store handoff thus permits another stock harvest without any deliverable storage destination. At360,194 of204 physical pieces are loose. Of44 spoiled pieces,22roots born0 expire loose onday6,15roots born0 expire stored onday10,7berries born9 expire roofed onday14. This proves the missing acquisition boundary, not that saving those44 pieces alone guarantees survival. No new target or yield is derived from this failure.

Separately, all8 calibration worlds overharvest and spoil while intake stays approximately flat. Old→new seed0 harvest2105→2943, spoil448→1372, intake1799→1816; seed1 harvest1763→3085, spoil308→1625, intake1787→1798. Most loss is stored raw berries/roots, not raw grain or cooked output. Owner expiry sums and recipe production balances reconcile16/16. Conservative raw-only lower bounds on younger food losing lifetime through oldest-date mixing are274/150/273/290/172/70/224/319 pieces. Example seed7:160berries beforeday31,40new harvest byday32,196expireday33 at3day lifetime; at least36 younger units inherit the old date. `M6A_DESIGN` explicitly requires oldest-date merging: preserve that contract and authored lifetimes. The bounded repair surface is choosing available empty/same-age storage before an avoidable mixed-age merge, including pot/ground allocation at arrival. Details: `cal1-spoilage-diagnosis.md/json` and `age-allocation-repair-design.md` in the R1 evidence directory.

The inherited204unit summer/autumn target and306unit cap for10 ordinary adults are unchanged. Three-day berries cannot alone represent an8.5day target; this mismatch is noted without inventing a replacement cap or assuming a particular harvesting choice caused every loss. Yield/density changes remain unjustified. A corrected acquisition/transport boundary and age-preserving destination choice need failing-first tests and real soak verification before the second and final calibration.
