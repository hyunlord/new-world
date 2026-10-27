# M-CORE story diagnostics — P5

**관찰 일치 검증 PASS, 이야기 품질 향상·전체 목표 수용 판정 아님.** 원래 held-out 결과를 바꾸지 않은 추가 진단에서 BASE7,676→mind19,722항목의 증가12,046을 종류별로 확인했다. 새 `mind_choice`12,142항목이 증가분 전부 이상을 설명하며 나머지 종류의 순변화는−96이다. 각32세계×2년=64 world-years이므로119.9375→308.15625항목/년, `mind_choice`189.71875/년, 기타 합계119.9375→118.4375/년이다. 사건 수 증가를 더 깊은 이야기 증가로 해석하지 않는다.

## 비교와 관찰 범위

시드1232–1263,48일, stewardship/defence/story/neighbours ON, player orders 없음. 추가 양쪽 캡처 exit0, 분석 exit0. 분석기는32개 고유 시드·각2개 완전 연도·전역 entry index·참조 sample coverage를 검증하고 원래 `story_yield.json`의 inputs/definition/fingerprints/연간 모든 지표를 비교했다. source_sha만 별도로 보존하며 동일성 비교에서 제외한다. 각 arm32세계의 최종 fingerprint와64연도 지표 모두 원본과 같다. 분석은 raw typed cause를 따라 깊이·missing·cycle·종류 경로 중복제거·인물 수를 독립 재계산했다. 원래 heldout-final-1 파일은 덮어쓰지 않았다.

이 도구는 **Chronicle 그래프**만 읽는다. 전체 Mind EventId parent/fact 그래프는 내보내거나 검사하지 않았다. `cause=None`은 이 그래프에 기록된 부모가 없다는 뜻이며, 의도한 근본 원인인지 다른 namespace의 투영 누락인지는 emitter별 검토가 필요하다.

## 전체 종류별 결과 (0행 포함)

BASE 레지스트리에 없는 `mind_choice`도 두 arm 합집합에서0으로 표시한다. 합계7,676/19,722, 종류48행이다.

| Kind | BASE count | Mind count | Delta | BASE/year | Mind/year |
|---|---:|---:|---:|---:|---:|
| became_elder | 0 | 0 | 0 | 0.000000 | 0.000000 |
| born | 53 | 51 | -2 | 0.828125 | 0.796875 |
| breakdown | 1 | 1 | 0 | 0.015625 | 0.015625 |
| came_of_age | 0 | 0 | 0 | 0.000000 | 0.000000 |
| daughter_founded | 0 | 0 | 0 | 0.000000 | 0.000000 |
| departed | 0 | 0 | 0 | 0.000000 | 0.000000 |
| departure_cancelled | 0 | 0 | 0 | 0.000000 | 0.000000 |
| died | 2 | 5 | 3 | 0.031250 | 0.078125 |
| drafted | 1072 | 1050 | -22 | 16.750000 | 16.406250 |
| eureka | 0 | 0 | 0 | 0.000000 | 0.000000 |
| fight | 0 | 0 | 0 | 0.000000 | 0.000000 |
| fission_cancelled | 0 | 0 | 0 | 0.000000 | 0.000000 |
| fission_started | 0 | 0 | 0 | 0.000000 | 0.000000 |
| founding_member | 320 | 320 | 0 | 5.000000 | 5.000000 |
| hunted | 291 | 285 | -6 | 4.546875 | 4.453125 |
| hut_built | 0 | 0 | 0 | 0.000000 | 0.000000 |
| joined | 0 | 0 | 0 | 0.000000 | 0.000000 |
| knowledge_at_risk | 1 | 0 | -1 | 0.015625 | 0.000000 |
| knowledge_discovered | 80 | 80 | 0 | 1.250000 | 1.250000 |
| knowledge_learned | 62 | 61 | -1 | 0.968750 | 0.953125 |
| knowledge_lost | 0 | 0 | 0 | 0.000000 | 0.000000 |
| left_settlement | 0 | 0 | 0 | 0.000000 | 0.000000 |
| migrants_arrived | 91 | 91 | 0 | 1.421875 | 1.421875 |
| migrants_declined | 91 | 91 | 0 | 1.421875 | 1.421875 |
| migrants_joined | 0 | 0 | 0 | 0.000000 | 0.000000 |
| mind_choice | 0 | 12142 | 12142 | 0.000000 | 189.718750 |
| paired | 120 | 120 | 0 | 1.875000 | 1.875000 |
| porter_delivered | 0 | 0 | 0 | 0.000000 | 0.000000 |
| porter_departed | 0 | 0 | 0 | 0.000000 | 0.000000 |
| pressure_began | 178 | 180 | 2 | 2.781250 | 2.812500 |
| pressure_ended | 153 | 155 | 2 | 2.390625 | 2.421875 |
| quarrel | 279 | 276 | -3 | 4.359375 | 4.312500 |
| recovered | 388 | 376 | -12 | 6.062500 | 5.875000 |
| room_formed | 82 | 80 | -2 | 1.281250 | 1.250000 |
| settled | 0 | 0 | 0 | 0.000000 | 0.000000 |
| settlement_dissolved | 0 | 0 | 0 | 0.000000 | 0.000000 |
| settlement_formed | 32 | 32 | 0 | 0.500000 | 0.500000 |
| sick | 427 | 413 | -14 | 6.671875 | 6.453125 |
| steward_action | 3189 | 3150 | -39 | 49.828125 | 49.218750 |
| steward_defence | 158 | 155 | -3 | 2.468750 | 2.421875 |
| tool_broke | 10 | 8 | -2 | 0.156250 | 0.125000 |
| traded | 0 | 0 | 0 | 0.000000 | 0.000000 |
| trader_arrived | 218 | 218 | 0 | 3.406250 | 3.406250 |
| trader_left | 218 | 218 | 0 | 3.406250 | 3.406250 |
| villages_estranged | 0 | 0 | 0 | 0.000000 | 0.000000 |
| villages_reconciled | 0 | 0 | 0 | 0.000000 | 0.000000 |
| widowed | 2 | 5 | 3 | 0.031250 | 0.078125 |
| wolf_killed | 158 | 159 | 1 | 2.468750 | 2.484375 |


## 깊이와 누락 경로

| 항목 | BASE | Mind |
|---|---:|---:|
| 깊이0 항목 |6766|18303|
| 깊이1 항목 |898|1405|
| 깊이2 항목 |12|14|
| 깊이≥3 항목 / 연도별 distinct deep chains 합 |0 / 0|0 / 0|
| explicit unknown 직접 종결 |108|107|
| explicit unknown에 도달한 후손 시작항목 |13|18|
| missing 전체 |121|125|
| 존재하지 않는 entry / sample |0 / 0|0 / 0|
| cycle |0|0|

108/107개의 직접 unknown은 전부 `quarrel`, `CauseRef::Modifier { code:100, entry:None }`이다. 실제 대상으로 지정한 entry/sample가 사라진 dangling link는0이다. missing은 시작항목마다 세므로 같은 unknown 근본을 참조한 후손도 다시 센다. 따라서121/125개의 서로 다른 손상이라고 할 수 없다.

실제 경로 예시 (index는 세계 전체에 걸쳐 유지):

- BASE seed1232 index77, tick12273, 사람0/1: quarrel의 Modifier100 부모가None. 직접 unknown, 깊이0.
- BASE seed1238 index86(tick16209, 사람6/8) → index80(tick14559, 같은 사람쌍) → Modifier100/None. 부모 entry는 실제 존재하며 깊이1 후 unknown으로 끝난다.
- Mind seed1232 index183, tick13641, 사람7/3: Modifier100/None, 깊이0.
- 같은 세계 index492, tick53511, 사람7/3 → index183, tick13641 → None. 연도1에서 연도0으로 정상 연결된 깊이1 경로이며 연도 경계에서 잘린 결과가 아니다.

`runtime/mood/quarrel.rs:277–299`는 부정적 modifier가 없을 때 관계를 원인으로 선택하고 bond의 `last_quarrel`이 있으면 해당 index를 참조한다. 이전 quarrel이 없으면None을 명시한다. 연대기 cause 숫자6은 emitter의 `1+code`이고 typed Modifier code100과 다른 namespace이다(:326–332). 이 경로와 직접 unknown 실제 레코드가 일치한다. 정상적인 첫 관계 다툼의 미기록 기원을 실제 샘플/사건으로 바꾸는 제품 변경은 이번 진단에 포함하지 않는다. 없는 이전 다툼을 만들어 연결하거나 unknown을 숨겨 missing을 낮추지 않는다.

## 깊이0의 해석과 계측 한계

기존 `story_yield/metric.rs`는 Entry/Pressure/Modifier의 실제 entry 연결을1edge, 실제 sample까지1edge와 그 sample의 entry까지 추가1edge로 센다. None인 부모가 있는 sample은 유효한 sample root이며, unknown suffix 앞에서 확인한 깊이는 보존한다. 이번 원시 그래프에서 깊이3 이상은 실제로0이다. 수치가 모든 mind 설명의 깊이까지 측정한 것은 아니다.

`runtime/mind/record.rs`는 band0에 새로 들어갈 때만 ChronicleKind::MindChoice를 남긴다. 그 외 선택은 별도 mind history/causal log에 남고 이 표의 항목이 아니다. 첫 우선순위 부모만 `source_cause`로 Chronicle에 투영한다. `record/provenance.rs::source_cause`는 Chronicle 원천에 대응하는 fact-source Event만 Chronicle cause로 돌려주며 command-only 원천이나 이전 choice Event의 전체 parent 경로를 자동으로 재구성하지 않는다. 따라서 `mind_choice` 수와 strict critical coverage 분모12,159 또는 행동 critical 분모12,020을 같다고 가정하면 안 된다. 이번에 직접 센 값은12,142다.

정리하면 **종류 증가의 정확한 원인은 추가 mind_choice 기록**, **missing은 모두 명시적 unknown origin 및 그 후손**, **dangling entry/sample 손상은 이 corpus에서0**이다. 전체 Mind EventId 그래프의 완전성, causeNone의 모든 의도, 깊은 이야기의 체감 품질은 이 결과로 판정하지 않는다. [ISS341/#647](https://github.com/hyunlord/new-world/issues/647)의 증거 보강이며 원격 폐쇄나 제품 수정 완료를 주장하지 않는다.

## 출처와 실행 provenance

실제 후보는 **candidate1 uncommitted source**이다. `source_sha=48f4b2fa2c0ffa076af87a2c166d97c1b988213b`는 부모 커밋 라벨이지 그 커밋만 빌드했다는 뜻이 아니다. `repair-candidate-bin-1/exploratory-build-receipt.json`은 kind=exploratory_uncommitted_source, committed_source_claim=false, features[]를 명시한다.

| 근거 | SHA-256 / 값 |
|---|---|
| 후보 Rust source |`f114f7815af48baa3ca8cf6edaddd904e6c6c4f65b26b2f71df251110c834df5`|
| 후보 source archive |`ba89444f3915b3b26656619a86fe094b7695952bc99d765e127978fd2b28bb65`|
| 후보 probe binary |`fa623d9174d8fd4bdf81ae4dc1f16352ea888752824f62fc9ea77f65d7e9af51`|
| BASE 진단 probe binary |`225a2c0942750afcfa5991230cfc92b86233b8a14e0cab03b2115320240ac2e6`|
| BASE 추가 관찰 source manifest |`2bfb10f36a59b59bf6f6539b6fcadfb407813db79b6da922189cc34d7b6da85d`|
| 분석 analysis.json |`2b9ae9aac3ee18ead728cb17ec3c1c2fb493e11ef5339b32cee51daa4e67d7a8`|
| 분석 report.md |`d6b9f0e3c95b163780e4cfa421ad32b51ecb29df6d39de8f264a1a3a4fca88fe`|
| 분석 script |`05f6354aaaf97df71e837ffe7bedd8d888f5f32dee2ec49f66943662aa629320`|

모든 원시 경로는 `/Users/rexxa/work/core-data/` 아래다: `p5-story-diagnostics-base-1/`, `p5-story-diagnostics-candidate-1/`의32개 sidecar 및 story_yield.json, `p5-story-analysis-result-1/{analysis.json,report.md}`, `p5-story-analysis-1.exit`(0), `p5-story-capture-repair-1/{binaries.json,base-exit.json,candidate-exit.json,terminal.json}`. terminal의 ANALYSIS_PENDING은 캡처 종료 당시 값이고 후속 분석exit0이 완료 근거다. analysis.json은 모든 입력 파일 해시와64연도별 표, 범주별 최대4개 실제 경로를 보존한다.

BASE는 runtime `9855bd082ad88a4c49296410f206f148d98c4df9`에 원래 관찰 source `e5835e51760fc08808b324e7cb62c718ee79fa22` 및 별도 additive diagnostics를 더한 빌드이다. exact BASE source/binary 빌드라고 하지 않는다. 원래4개 story_yield instrument 파일은 frozen provenance와 byte-identical이며 기존 heldout의 story 형식/정의는 변경하지 않았다. 이 추가 계측은 성능 측정 근거가 아니다.
