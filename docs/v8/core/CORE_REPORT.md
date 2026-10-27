## 구현 완료 보고 — M-CORE

**2026-10-27 후속 작업 진행 중:** planner P-1~P-6/D25에 따라 성능 실패를 진단·수정하고 전체 게이트를 다시 실행한다. 아래 p8 실패 기록은 당시 결과로 보존하며, 중단 문구는 P-1로 정정됐다. 현재 게시·최종 수용은 아직 하지 않았다. D26의 provenance 규칙과 추가 성능 기록은 CORE_DECISIONS/CORE_PERF를 따른다.


**최종 판정: FAIL — 미게시, 전체 목표 미수용.** p8 `44e0a0266c085d409c16c77f4a2a0181b8a2a750`에서 최종 게이트42/45·exit1이다. 실패는25 bench,41 mux2_stall_receipt,45 core_evaluation이며 목표의 중단·보고 조건에 따라 튜닝·면제·재시도를 하지 않는다. §9의15개 항목으로 결과와 한계를 보존한다. p5 구현은 로컬 커밋 [0ef68a1d2b6eb946e150963071b2e623fa58d4b0](https://github.com/hyunlord/new-world/commit/0ef68a1d2b6eb946e150963071b2e623fa58d4b0)에 있으며 아직 게시하지 않았다. p6 생산 등록 전환은 로컬 커밋 [fbc9a9babc9bb5df1d06ca1e655901e480521f66](https://github.com/hyunlord/new-world/commit/fbc9a9babc9bb5df1d06ca1e655901e480521f66)에 있고 전환 캡처가 완료됐다. p7 기준선 분리는 로컬 `be12050c951dfb025f3440d34b657eec0663b817`로 커밋했고 해당 최종 빌드가 통과했다. `heldout-final-1`의 outcome·why는 PASS이며 story를 포함한 전체 캡처/검사 chain이 일곱 exit0으로 완료됐다. 전체 목표 수용은 아직 주장하지 않는다. 최종 성능은 유효 FAIL이며 필수 C3 위반으로 게시 BLOCKED다. 실패한 게이트와 완료된 한정 검증을 구분해 기록한다.

### P-5 추가 진단 (2026-10-27, report-only)

추가 관측은 원래 held-out의 각 arm32개 최종 fingerprint와64개 연도 지표를 그대로 재현했다. 연대기7,676→19,722항목(119.9375→308.15625/년)의 증가12,046은 `mind_choice`12,142개와 기타 종류 순변화−96개로 설명된다. [전체48종 표와 원인 경로](CORE_STORY_DIAGNOSTICS.md)를 별도로 보존했다.

missing121/125는 직접 explicit-unknown108/107개와 그 후손13/18개다. 실제 지정된 entry/sample 대상의 누락과 cycle은 모두0이며, 직접 unknown은 전부 이전 quarrel을 기록하지 못한 Modifier100/None이다. 깊이≥3은 관측한 Chronicle 그래프에서 실제0이나, 전체 Mind EventId 인과 그래프는 이 계기가 내보내지 않아 전체 이야기 깊이를 판정할 수 없다. [ISS341/#647](https://github.com/hyunlord/new-world/issues/647#issuecomment-6431336839)에 원인과 범위를 공개했고 미해결로 유지한다. 이번 추가 진단은 런타임 성능 합격이나 전체 인과 완전성 주장이 아니다.

### 측정 (결정 시스템 점유율 s, 갈래별 비용)

BASE는 `9855bd082ad88a4c49296410f206f148d98c4df9`다. D10의 시작 부하 조건(1분 load <8)을 만족한 세 인구 × 세 반복에서 얻은 평균이다.

| 인구 | 전체 tick 평균 µs | 기존 결정 평균 µs | 결정 점유율 |
|---|---:|---:|---:|
| 200 | 424.806019 | 280.263230 | 0.659744020 |
| 1,000 | 2092.741204 | 1462.832342 | 0.699002982 |
| 2,000 | 4807.145139 | 3576.836370 | 0.744066648 |

N1000의 정확한 `s = 0.6990029820337981`. [CORE_PERF](CORE_PERF.md)의 BASE 원시 자료와 audit 해시를 따른다. 갈래 계측은 91개 의미 행을 247개 등록 키에 연결했고, 11개 콜로니 그룹·66세계·3,824,640 world-tick을 집계했다. 계수는 고유 인물-tick이 아니라 해당 scope 진입 수다. 유효 재계측의 비용은 중첩 포함 비용이므로 합산하지 않는다. 예를 들어 `phase.tick`은 3,824,640회/393,802,472,135ns, `phase.routine_setup`은 같은 횟수/25,015,511,117ns다.

유효 집계 `core-arms-ledger.json` SHA-256은 `7eaf23ff385a0c5bb84d3b089282850a391547165f39618409d46876dce7d82e`. 최초 경합 캡처의 elapsed 값은 INVALID로 보존하며 최종 속도 근거로 쓰지 않는다. 전체 행·0회 행·fixture별 수치와 비용 해석은 [CORE_INVENTORY](CORE_INVENTORY.md)에 있다.

### 콜로니 갈래 목록 (옮김 / 바꿈 / 버림, 각각 이유)

91개 행의 개별 판정·읽기/쓰기·gate·관측 키는 [전체 목록](CORE_INVENTORY.md)을 기준으로 한다. 다음은 요약이며 목록을 대체하지 않는다.

| 판정 | 범위와 이유 |
|---|---|
| 옮김 | 욕구 지속/중단·도착/소비·식사·사회적 상호작용·채집·일과 수명주기·기존 작업/도주/징집 소유와 변경 순서를 보존한다. 관측된 갈래를 편의상 광범위 replacement로 재분류하지 않는다. |
| 바꿈 | 우선순위 선택을 데이터 띠/노드로 표현한다. 선언한 일과 선택 및 필수 `(agent,tick,node)` 난수 키 변경만 좁게 구분한다. D18의 정착지 간 채집 다툼은 난수 키만 변경하고 효과는 옮긴다. |
| 버림 | 실제 0회 증거와 생성 규칙을 함께 확인한 이동/분열·자연 V7 건설·운반 등 목록의 명시 행만 해당한다. 단순히 0회라는 이유로 다른 희귀 갈래를 삭제하지 않는다. |

채집 rival/dispute/quarrel 관측은 0회다. D18은 실제 두 정착지 채집자 회귀 RED→GREEN으로 검증했으며 단일 정착지 플레이에서 발생했다는 주장은 없다. 식사 critical-water 예외 횟수는 `selected.meal_wait_check - selected.meal_wait`로 도출한다. 후보·최종 선택·조건 평가 계수를 혼동하지 않으며 난수 변경 때문에 그림자 포트 분모를 제외하지 않는다.

### 새 두뇌 구조 (띠·노드·사실·행동, 데이터만으로 노드 추가 시험)

작성된 정책은 6개 띠(critical/duty/need/work/leisure/idle), 157개 노드, 참조 사실 80개, 참조 행동 19개다. 등록 행동에는 데이터 전용 시험용 targeted rest도 있다. 컴파일러/선택기는 sim-core, 읽기 전용 사실과 대상 해석·효과·기록은 sim-systems의 mind 모듈에 둔다. 사실 공급자는 정책을 고르거나 자원을 예약하지 않고, 효과는 변경된 공유 도메인을 무효화한다. 선택기는 정수 점수·작성 순서 동점·히스테리시스·노드 주기를 처리한다.

[작성 안내](MIND_AUTHORING.md)의 따뜻한 화덕 rest 예시는 실제 대상/도착/선택 기록을 요구한다. 최종 gate43에서 `data_only_warm_at_hearth_reaches_real_target_and_retains_deciding_record` 실제 시험1개가 PASS다. 문서 예시만으로 통과를 주장하지 않는다. 생산 등록은 p6에서 콜로니 mind/기준선 legacy로 전환했고, 저장 로드는 해독한 규칙을 사용한다. 기존 콜로니 저장의 빈 legacy 결정 슬롯 하나만 같은 위치의 빈 mind 슬롯으로 바꾸는 엄격한 이행을 둔다(D19).

### 그림자 비교 (일치율, 불일치 범주와 판정)

| 회차 | OFF 행동 일치 | ON 행동 일치 | 판정 |
|---|---:|---:|---|
| 1 | 91.333235% | 94.367286% | FAIL 보존 |
| 2 | 99.913210% | 99.804743% | 수치 기준 통과, Idle+Job 포트 결함 12건으로 수용 거절 |
| 3 | 99.913210% | 99.805452% | 그림자 단계 수용, 미설명/잔여 포트 결함 0 |
| 4 | 2,036,477/2,038,246 = 99.913210% | 1,687,813/1,691,103 = 99.805452% | 최종 그림자 단계 PASS, 미설명/잔여 포트 결함 0 |

4회차는 seed0–7 ×20일 OFF/ON이며 띠 일치는 OFF99.866601%, ON99.726451%다. 기존 fingerprint 계약과 관측 회계는 통과했지만 전체 직렬화 상태의 동등성을 뜻하지 않는다. 모든 불일치와 no-place 포트 분모를 유지했다. 범주/사례/판정은 [CORE_SHADOW](CORE_SHADOW.md) 및 외부 `shadow-round-4/round4-independent-audit.{json,md}`에 보존한다.

별도 48일 훈련 후보는 6개 결과 기준 모두8/8, 추위 노출5/5(요구4, 비노출3), 결정성 PASS다. 3·4회차 gameplay summary는 동일하며 SHA-256 `70cccdc7e6a00703cba8ea04ef9210e6d13de6e1d94ec5c95c10bea21dc7e68b`. 이것은 held-out 평가가 아니다.

### 전환 평가 (held-out 1232–1263: 새 두뇌 / 옛 두뇌 표)

| 평가 | 옛 두뇌 | 새 두뇌 |
|---|---|---|
| held-out 1232–1263 결과/노출 분모 | 완료, 아래 실제 분모 유지 | outcome PASS, 전체 캡처/검사 chain 완료 |
| 전환 전후 콜로니 digest | 아래 10개 원본 해시 보존 | 10개 새 해시와 실제 gameplay 차이 감사 완료 |

| 기준 | BASE | 새 두뇌 | 요구 BASE / 새 두뇌 | 판정 |
|---|---:|---:|---|---|
| 1일 비축 | 32/32 | 32/32 | 31 / 31 | PASS |
| 10일 벽 있는 침상 | 32/32 | 32/32 | 31 / 31 | PASS |
| 추위 사망0 | 32/32 | 32/32 | 31 / 31 | PASS |
| 48일 생존 | 32/32 | 32/32 | 31 / 31 | PASS |
| 늑대 사망≤1 | 32/32 | 31/32 | 29 / 29 | PASS |
| 정확한 원장 | 32/32 | 32/32 | 32 / 32 | PASS |
| 한파 노출 중 장작 | 17/17 | 19/19 | 16 / 18 | PASS |

완료된 held-out outcome·why precheck 범위는 PASS다([CORE_EVAL](CORE_EVAL.md)). 한파 비노출은 BASE15/새 두뇌13으로 분모를 통일하지 않는다. **seed1238 늑대 사망 BASE0→새 두뇌3**은 실제 불리한 결과이며 해당 seed 기준 FAIL을 보존한다. 집계 기준 통과가 개별 악화를 없애지 않는다. 정책 튜닝 없음. 최종 authoritative outcomes/why와 precheck의 byte equality를 확인했고 전체 story chain도 완료됐다.

전환 캡처 체인은 exit0이다. 독립 감사는 OFF9fixture(56세계)+ON2세계×24일의 총 **58세계/3,271,680tick**을 확인했다. seed/day coverage가 정확하고 ledger/reservation 위반tick0·최초위반null이다. 행동 기록468,387/468,387·엄격 critical 기록22,080/22,080이며 soak/ON 결정성은 PASS다. 이는 held-out 결과나 OFF 성능 통과가 아니다.

감사 원본은 `/Users/rexxa/work/core-data/switch-dev/fixture-audit.json`(SHA-256 `b6bcc6231c7640c6548eb03eefa950188da1023d1dba6d4eee44dbb8d65e77e6`)이다. 실제 probe SHA-256은 `629ea00c191e8df2e5f2a826fcf3e1a30615071be245311566316629e4f442dc`: p5와 소스 patch `9411d508336f0fd6c125463d0431ffe5734f23de1780b0b06259cdfbd956dfc6`에서 빌드한 것이며 나중의 p6 커밋에서 빌드했다고 소급 표시하지 않는다.

| Fixture | BASE summary SHA-256 | 전환 summary SHA-256 |
|---|---|---|
| soak | `c07dab3200fc3be1cfa87dd5abee629964ba8c17ac62469ae86c29bd73d78b69` | `203d098630d061261c76e83f6b1d3951d662884846758004ccd0c52bc4ca2b5f` |
| story | `d6952313fe88f1caab37b18c85d42a352689a56467d5af8109927c21dc227feb` | `94f177b3ef924a1ab59357ded06a5e0a7fed22ac02b5646eaabed6040e149d28` |
| neighbours | `eab12f6964891d9b2675b538ec1e3d6141ad4d3637eb4a9198e33d773e6f677c` | `0281170659f77c905ff7b9a10b63550587c9881e688521d70e45020f66493267` |
| knowledge_f10 | `f39c21ae1d9bf6a819714be84510ad1f6d9d12664236f37d0f1942efec4b6d46` | `fc48d580d159a03d83d00125d2f2d2d9e41bbbb606930c071a185c9f280ea985` |
| m55_pressure | `74ec901346b50a7ba6dad2a34993101c16b57fbaeaf3153deb321044a7d2cd96` | `4a0cda9ea85041210834e8124e04005b2cad3b5c1cd5e636ae897b638f674a36` |
| m55_nodraft | `32ab17b017a6311fea0556071c01d8d0ed594ae2002aaf0dcf70895e9051d5ee` | `1f319da7c4b76ad59861c810862667e02e0afa0fbdee6d87345f917b83b9adb2` |
| m55_storyoff | `9065d5679657056c0f407564a381843fa5a6cdc4204473c10fbd9b81d33bb97c` | `139682522580c142f1d5f715efdf33b5a9ed724ee4d099c1642ab257c7cb4f93` |
| m55_noneighbours | `fef5b45dd8eae35533c5d6738f1c430ef525fecc13b41816ead26449823deecb` | `bc651e11dda612aa54f3e349ccc56c26e35980ba36adbedbdd183f28710b3f41` |
| m55_quiet | `b62668f754ed0b1e1d251b9b81a63ecbfde64e8e3040bc03587de8e4814978fa` | `a08bbb39f93a889c4a1cbece1879220cddddf2809b056eaa44910e8fdffacb33` |
| steward_on | `7de55ce7ff34aebc978769de8fc63037ecda67c0929b1b783ea6939799087196` | `cad9811728445bba5d8aa9027d6450ceb9aec3b40635258572577b711c387958` |

**불리한 OFF 결과도 보존한다.** `m55_nodraft` seed1/4/6의 combat 사망은 각각0→2, seed7은1→2다. `m55_noneighbours` seed6의 exposure 사망은0→2다. seed6의 U3는 pressure/nodraft/storyoff/noneighbours/quiet 다섯 fixture에서 true→false다. craft-loss 증감까지 포함한 경고21행을 감사 JSON과 D19에 모두 보존한다. 예를 들어 pressure seed5 craft-loss2→5, noneighbours seed3은0→2다. 전체 jobs/crafts·재고·지식·사회/인구 궤적도 달라지며 이를 단순 mind metadata 변화로 숨기지 않는다.

승인된 전환은 legacy cascade 대신 데이터의 띠/노드 순서와 node-keyed 추첨을 사용하므로 후속 궤적이 달라질 수 있다. 이것은 가능한 기제의 설명이며 각 changed leaf의 원인이 입증됐다는 뜻이 아니다. 같은 command script도 실제 수락 명령 수까지 같음을 보장하지 않는다. 정책을 다시 튜닝하지 않으며, 정식 ON held-out outcome precheck는 아래 고정 입력으로 PASS이며, 캡처/검사 chain은 완료했으나 전체 M-CORE 수용은 최종 gate 실패로 **FAIL**이다.

정책은 `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`로 동결했다. `policy-freeze-round-4.json` SHA-256 `8306d91c24ee200f2fb308c332ecbbb93f6a90c14c18b5ca2a9e89b22a97675d`, 동결 시각 `2026-10-26T16:35:29Z`. 동결 후 held-out을 실행했으며 위의 완료된 precheck 결과만 주장한다. 원래 동결 시점과 네 보정 회차 한도는 유지한다.

### "왜?" 기록 범위와 사람 정보창

훈련 4회차의 첫 선택+행동 변경 기록은135,368/135,368, 엄격한 critical 선택 전이는2,984/2,984, 실패0이다. 후자는 첫 선택/노드 변경/행동 변경/restart 중 critical에 도달한 경우이며 단순 행동 변경 수가 아니다. held-out 기록 precheck는 **PASS**: 예상805,730/805,730, 행동469,721/469,721, 엄격 critical12,159/12,159, 실패0이다. 최종 authoritative why 대조와 전체 캡처/검사 chain도 완료됐다.

현재 행동·실제 선택 근거1–2개와 Why의 상위3후보를 EN/KO로 표시하는 소스와 실제 창 캡처 하네스를 마련했다. 근거는 bridge의 선택 노드 조건/점수 기여 순서를 사용하며 UI에서 숨은 정책을 고르지 않는다. Native03 integrity와 독립 pixel review는 실제14PNG 범위 **PASS**다. Godot/wrapper/provenance exit0, 입력해시 불변1이다. Agent8 do_job(event82/tick1)→eat(event90/tick154), Agent4 do_job(event4129/tick18787)→flee(event4187/tick18906), danger cause4186을 실제 기록과 대조했다. 각 popup은 실제 후보1개이므로 3행 배치/순서는 입증하지 않았고, 요약은 단일 필드지만2–3줄로 줄바꿈되어 물리적 한 줄을 보장하지 않는다. p7+UI patch 캡처이며 p8 빌드로 소급하지 않는다.

실제 캡처 원본: [배고픔 EN](../../images/core_mind_hunger_en.png), [위협 KO](../../images/core_mind_threat_ko.png). chronicle 회귀 RED10실패와 중간 joiner fixture1실패를 보존한 뒤 최종12검사/0실패, strict5파일·FFI60검사가 통과했다. 별도 chronicle Why 링크 클릭은 이번 증거 범위 밖이다.

### 성능 (전후 표, 막대 산수)

최종 교차 측정은 **유효 FAIL**, C3 시간 상한 위반으로 **게시 BLOCKED**다([D23](CORE_DECISIONS.md), [전체 표](CORE_PERF.md), [ISS339/#645 OPEN](https://github.com/hyunlord/new-world/issues/645)). 33쌍/66회 모두 유효, 무효0이며 입장 1분 부하는 4.1201171875–7.06201171875로 모두8 미만이다. p8 전체45단계 gate는42/45·exit1로 완료됐다(D24).

| N1000 비교 | 동일 세션 BASE | M-CORE |
|---|---:|---:|
| 전체 tick 평균 µs | 2082.978819 | 19927.196181 |
| 결정/mind 평균 µs | 1455.127483 | 19136.358476 |
| 전체 tick new/old | 1.0 | 9.566682097070183 |
| mind/old decision | 1.0 | 13.150984160978739 |

앞선 설계 측정의 `s=0.6990029820337981`과 산수 `1-s/2+0.05=0.7004985089831011`은 보존한다. 최종 비교는 과거 장부 대신 동일 세션 BASE를 사용한다. CORE 참고용 두 막대(전체≤0.700498508983101, mind≤0.5)는 모두 미달이다. 별도로 필수 standing6장면 평균/p99의1.1배 상한12건 및 C3 N1000≤5ms·N2000≤10ms 두 건도 실패했다. N2000 세 반복 평균은38.421873ms다. 영수증의16개 실패문자열을 보존하며 참고용 예외를 C3 면제로 확대하지 않는다.

After-build allocator/RSS의 네 C3 메모리 상한은 PASS, 할당/인물 new/BASE 비는1.0이다. 별도 합성 full-ring companion은9060.066666666668B/인물로4096 상한을 초과하며,80.076416015625B/tile은96 상한을 통과한다. 실제 자연 점유율이 아닌16개 기록 포화 용량 참고 측정이며 standing after-build 결과를 대체하지 않는다. 소스 리뷰의 eager fact/routine·캐시 무효화·기록 검증 비용은 가설로만 남긴다. 내부 프로파일로 개별 비용을 입증하지 않았다.

근거 `/Users/rexxa/work/core-data/final-perf-session-1/receipt.json` SHA-256 `966c6cd06e7fd7c78da4d9bdb6090fb5113e69b80b084163ef3c3452a4e81cf3`, 측정 커밋p7 `be12050c951dfb025f3440d34b657eec0663b817`. 독립 검사 exit1은 동일 FAIL을 재현했고 영수증 byte 비교는 exit0이다. 후속 문서 HEAD의 새 측정이나 전체 gate 실패를 주장하지 않는다.

### 기준선 분리 (등록표, 동일성)

전환 전 identity4는 baseline 차이0, chronicle64파일/19,683항목 동일, raw sample digest 동일, colony OFF9개 summary와 ON summary의 BASE byte identity를 확인했다(D18). 이것은 전환 후 증거가 아니다.

현재 등록표는 baseline `AgentDecisionSystem`, colony `ColonyMindSystem`이고 다른 시스템 순서는 유지한다. 실제 생성자/저장 경로와 표의 집중 검사는 통과했다. p6 전환 후 baseline은228,748값 차이0·chronicle/relations64파일/19,683항목 동일·raw digest 동일이다. 비교 로그 `/Users/rexxa/work/core-data/switch-dev/baseline-compare.log` SHA-256은 `8f5601d2424a458af813d5c42c1ac819b295ec1d7c1bff64b23b7772b7c2beae`다. p7의 `runtime/baseline/decision` 이동·죽은 gate/seam 삭제 후 동일성 캡처는 **PASS**다. baseline228,748값 차이0, git_sha만 제외한 chronicle64파일/19,683항목 동일, OFF9개와 ON summary 모두 전환 캡처와 같고 ON 결정성 PASS다. `isolation-safety-1-compare.log` SHA-256은 `39b246855ca52c88359390b8b1b561aae4447e8867c72f18e6981834e7e92155`, ON summary는 `cad9811728445bba5d8aa9027d6450ceb9aec3b40635258572577b711c387958`, 캡처 Rust tree는 `c3a0e98c30e1758bf785f9a6936056d87b58b370`이다. 완료 로그는 `/Users/rexxa/work/core-data/isolation-capture-1.log`이며 p7 커밋은 `be12050c951dfb025f3440d34b657eec0663b817`이며 p8 전체 gate는42/45·exit1이다. 별도 baseline crate는 D20에 따라 이번 범위에서 제외했다. 공유 crate-private mechanics와 등록자의 추가 분리를 피하는 선택이며 module/import 경계 검증 의무는 남는다.

### 이야기 수확량 (BASE / M-CORE)

| 지표 | BASE 합계 | M-CORE 합계 | BASE / world-year | M-CORE / world-year |
|---|---:|---:|---:|---:|
| 연대기 항목 | 7676 | 19722 | 119.9375 | 308.15625 |
| 검증 깊이≥3 항목 | 0 | 0 | 0.0 | 0.0 |
| 서로 다른 깊은 kind 경로 | 0 | 0 | 0.0 | 0.0 |
| 연도별 ≥3항목 등장 인물 | 556 | 651 | 8.6875 | 10.171875 |
| 누락 ancestry 사슬 | 121 | 125 | 1.890625 | 1.953125 |
| 순환 사슬 | 0 | 0 | 0.0 | 0.0 |

양쪽32세계/64개 완전 world-year를 관측했다. BASE는 p2 `e5835e51760fc08808b324e7cb62c718ee79fa22`의 동결 관측 계기를 더한 BASE runtime으로 정확한 BASE 소스 빌드와 구분한다. 깊은 항목/사슬0은 깊은 이야기 성공을 입증하지 않고 누락 사슬121/125도 보존한다. 수치 합격 막대 없이 참고용 MEASURED이며 [STORY_YIELD](STORY_YIELD.md)에 정의·provenance를 기록한다. `story.json` SHA-256 `7fb7a6ceb55ac1d08cd5667e679e7e3064e19c8aa21761e8c8bede594970d9ce`.

### 닫은 GitHub 이슈 / 새로 연 이슈

이 보고서에서 새로 닫았다고 주장하는 GitHub 이슈는 없다. ISS333–335의 로컬 수정은 p5에 있고 [#639](https://github.com/hyunlord/new-world/issues/639), [#640](https://github.com/hyunlord/new-world/issues/640), [#641](https://github.com/hyunlord/new-world/issues/641)은 등록 완료·**OPEN**이다. 게시 전 폐쇄하지 않는다. 원장 fixed 상태는 로컬 수정 뜻이며 원격 폐쇄와 다르다. ISS336/[#642](https://github.com/hyunlord/new-world/issues/642)는 로컬 deferred·원격 OPEN이다. ISS337 Why locale/[#643](https://github.com/hyunlord/new-world/issues/643), ISS338 MindChoice47 누락/[#644](https://github.com/hyunlord/new-world/issues/644)은 로컬 fixed·원격 OPEN이다. Native03 pixel/integrity PASS로 수정 확인했지만 게시 전 원격 폐쇄를 주장하지 않는다. 소스 수정은 검증·폐쇄 완료를 뜻하지 않는다. 광범위 ISS048/#524와 ISS132 역사 기록은 그대로 둔다.

### 변경된 파일 목록 (커밋별)

[p0–p7 전체 변경 경로와 상태](CORE_CHANGED_FILES.md)는 실제 커밋에서 추출한 목록이다. 아래는 의도와 범위 요약이며, 이후 전달 커밋은 존재한 뒤 추가한다.

| 커밋/범위 | 상태와 주요 파일 |
|---|---|
| p0 `001f236c308a88615a650fca9af83d80041aa7a8` | 측정 전에 오너 계약을 보존: CORE_GOAL 원문 추가. |
| p1 `06f431b33d32962169eaa7abf8e4be14ce7978c3` | 판단 구현 전에 승인된 깊이 계약을 보존: 로드맵·정의·기록 문서 정리. |
| p2 `e5835e51760fc08808b324e7cb62c718ee79fa22` | 교체 전에 기존 두뇌를 관측 가능하게 함: 갈래 계측·집계와 평가/이야기 도구, 동일성 근거. |
| p3 `23378db34f4a55b698ccbf5177d097658d454e3e` | 계획자 승인 timing 입장 조건을 복구: D10의 ledger 규칙 문서화, 기존 실패 보존. |
| p4 `891c6cae7a2427b35b8f4524788c9861660cac48` | 허용된 측정으로 교체 예산을 확정: BASE 점유율·유효 갈래 비용·성능 막대 근거. |
| p5 `0ef68a1d2b6eb946e150963071b2e623fa58d4b0` | 로컬 미게시, 159파일. `sim-core/src/colony/mind`, `data/colony/mind.ron`, `sim-systems/src/runtime/{mind,agent_actions}`, 계측/그림자 도구, UI/번역, CORE 문서. 정확한 전체 경로는 해당 커밋의 `git show --name-status`를 기준으로 한다. |
| p6 `fbc9a9babc9bb5df1d06ca1e655901e480521f66` | 로컬 미게시. `sim-engine/src/registration.rs`, `sim-systems/src/{lib.rs,save,worldgen}`, 그림자 fixture와 관련 문서. 실제 캡처는 앞의 p5+patch 빌드 영수증에 귀속한다. |
| p7 `be12050c951dfb025f3440d34b657eec0663b817` | 로컬 미게시, 57파일. 기준선 모듈 이동·죽은 colony gate/seam 제거·live shadow 폐기와 fixture/검증 문서. 정확한 전체 경로는 이 커밋의 `git show --name-status`를 기준으로 한다. |

최종 빌드 영수증은 `/Users/rexxa/work/core-data/final-bin-1/build-receipt.json`이다. p7에서 `cargo build --locked --release --manifest-path rust/Cargo.toml -p sim-test --bin worldsim_probe --bin worldsim_perf`를 features[]로 실행해 exit0, UTC `2026-10-26T17:58:41.971467`–`18:00:47.316163`에 완료했다. rustc는 `1.93.1 (01f6ddf75 2026-02-11)`이다. 아래 빌드 근거는 held-out·성능 결과가 아니다.

| 빌드 바인딩 | 값 |
|---|---|
| Rust git tree | `c3a0e98c30e1758bf785f9a6936056d87b58b370` |
| 빌드 전후 Rust SHA-256 (동일) | `9a6cdde2052d909b816dcd0e5a0a5674dfdce75eed873b5ab6412b6bfe55fb21` |
| policy SHA-256 | `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8` |
| Cargo.lock SHA-256 | `ddc99ac9fc161d9b610c0f5d8dbf5e279c8908c10717647ba50ab2fef70c36d1` |
| worldsim_probe SHA-256 | `772a193fd2ebcfff7823c41068b75ea4a296fb2fb8d6d4fedd100f62490d7d41` |
| worldsim_perf SHA-256 | `91a75c3397eba02b2d19632bde3fc6bc965cd762cc67304e7c4a6a4566b9a56e` |
| build log SHA-256 | `52b74409bcdb3e00601f672fb224f07f23acdef1df6039016030279a74257cb3` |

### 확인된 제한사항

네 번의 허용 보정 회차를 모두 사용했다. 동결 후 정책 재조정이나 held-out 정보를 이용한 튜닝을 허용하지 않는다. 최초 무효 elapsed 측정, 그림자1 FAIL/2 수용 거절, 잘못된 정책을 읽은 초기 no-place RED, 전환 fixture 실패 systems5건/integration2건을 삭제하거나 소급 통과로 바꾸지 않는다.

최신 p7 동결 amendment는 `/Users/rexxa/work/core-data/baseline-dev/shadow-test-migration.json`(SHA-256 `47899e4aacd73008f885a2e0d4fb62f630021d64c30e42c7cf471e3d9e2891a3`)이다. 동결 검사 대상419파일은 정확히 두 test-only import 변경을 제외하고 일치한다. 이전 switch amendment `b15b4b7b1b0b2bbee788a604131b728fbe6f3c7e924c871d110e4ea2745f3e50`는 역사 근거로 보존한다. 최신 amendment는 old-shadow factory 폐기 후 worldgen helper 전체 파일 바인딩을 갱신했으며 helper 본문과 두 import 치환은 그대로다. 원본 manifest/checker/binary는 보존하고 wrapper는 임시 디렉터리에 원래 시험 바이트를 투영해 검사한다. 현재 전체 closure가 동결 당시와 byte-identical하다는 주장은 아니다.

희귀 다툼0회, 훈련 추위 비노출3세계, held-out 비노출 BASE15/새 두뇌13, fingerprint 계약 한계를 유지한다. Held-out outcome·why와 story 측정은 완료했고, native03 시각 검증은 명시한14PNG 범위 PASS이고 성능은 유효 FAIL, 게시는 C3로 BLOCKED이며 p8 전체 gate는42/45·exit1이다.

ISS336/[#642](https://github.com/hyunlord/new-world/issues/642)는 **로컬 deferred / GitHub OPEN**이다(D21). 잘못된 persisted mind chronology에서 retained/tile 선택의 ID·tick 또는 비공개 history high-water가 엔진 allocator/시계를 앞서는 경우 로드가 완전히 거부하지 못하고 다음 선택에서 늦게 실패할 수 있다. 정상 생산 기록은 ID를 할당하고 현재 tick을 사용하며, 이를 깨는 유효 생산 저장 경로는 발견하지 못했다. 이번 리뷰는 malformed-save 실행 재현을 수행하지 않았다. 완전한 검증에는 동결 sim-core의 명시적 chronology API가 필요해 이월한다. 보존 record만 검사하는 것으로 빈 history의 private high-water까지 검증했다고 주장하지 않으며, 직렬화 내부/가짜 삽입 우회도 하지 않는다. 이는 일반 gameplay 실패나 수정 완료 주장이 아니다.

### 오너 결정 필요

별도 baseline crate는 목표의 선택 항목으로 D20에서 제외했다. 필수 디렉터리·등록·import 경계 검증은 계속 수행하며 crate 분리를 완료했다고 주장하지 않는다. 최종 native 시각 수용과 오너 플레이 확인은 별도다. 새로운 목표 완화나 성능 면제를 요청하거나 추정하지 않는다.

### Gate 결과

| 단계 | 확인된 상태 |
|---|---|
| Part1 점유율/갈래 계측 | PASS 범위 위에 명시 |
| 최종 그림자4/훈련 후보4 | PASS, 전체 목표 수용 아님 |
| 전환 집중 검사 | switch10/save41/registration4/worldgen6/shadow3 PASS |
| fixture 수정 후 검사 | systems310/commands11/full-save7/jobs9/flee12/engine_mind_ffi3 및 두 번째 fmt/Clippy PASS |
| p6 전환 후 캡처 | 58세계 안전성/coverage 및 baseline 동일성 PASS, 경고21행 보존 |
| p7 기준선 최종 분리 | 지원 corpus identity 및 집중 isolation 검사 PASS; p8 전체 gate42/45·exit1 |
| held-out outcome·why | PASS, authoritative byte equality 및 전체 캡처/검사 chain exit0 |
| 이야기 수확량 | MEASURED 참고용, 수치 막대 없음 |
| native03 UI | 14PNG pixel/integrity PASS, 실제 후보1개·2–3줄 wrap 범위 |
| data-only 최종 gate43 | 실제 대상 도착·결정 기록 시험1 PASS |
| 최종 성능 | 유효 FAIL, 필수 C3 시간 상한 위반; ISS339/#645 OPEN |
| p8 전체 gate | 42/45·exit1; 실패25 bench,41 stall,45 core_evaluation |
| baseline_isolation42 / seed0 shadow44 | 각각 exit0 PASS; 새 seed0 OFF251664/251676·ON210955/211127, 각23범주 |
| 검증 커밋·작업트리 | p8 `44e0a0266c085d409c16c77f4a2a0181b8a2a750`; 작업트리·증거 보존 |
| 게시 | C3·stall 실패로 BLOCKED; push 없음, fixed 이슈도 원격 OPEN 유지 |

게이트는2026-10-27 04:02:02–05:58:54 KST, **7012초** 실행했다. 전체 목표의 정확한 시작 영수증은 확인하지 않았으므로 이 시간을 총 작업 시간으로 쓰지 않는다. 원본 `full-gate-p8-1/gate.txt` SHA-256 `f437bd3e64a96af0382f5806d8c314fbe8ad70e60bc50fc8c1fc8af76237a394`.

stall은 양 배율 모두 캡처를 완료했다. 1×는 준비 뒤300.000826초·35,755프레임 중>50ms2회·최대91.652ms로 FAIL, ½×는300.006963초·34,902프레임·>50ms0회·최대49.522ms로 PASS다. 시작 지연3775.521/4378.179ms도 보존한다. report `complete=true/bar_pass=false`, wrapper receipt `complete=false/exit1`은 수용 실패를 반영하며 실행 미완료를 뜻하지 않는다. `diagnostic_complete=false`이므로 원인 귀속 완료도 주장하지 않는다. [ISS340/#646](https://github.com/hyunlord/new-world/issues/646)은 OPEN이다. 두 지연은34.466878초의50.194ms와218.325537초의91.652ms이며 후자는 실행 tick0·pre/post draw85.418ms여서 mind가 원인이라고 단정하지 않는다. 근거와 SHA는 D24에 있다. p8 gate의 실패를 후속 문서 커밋 PASS로 바꾸어 쓰지 않는다.

### 오너가 할 일 (실행 한 줄: /Users/rexxa/work/new-world-play2-dev/tools/play.sh main)

기존 main 실행 명령이다. 이번 M-CORE는 게시하지 않았으며 이 명령으로 새 두뇌가 실행된다고 주장하지 않는다. 작업트리와 실패 증거는 보존한다. 양 원격 ref는 `git ls-remote` 재조회로 BASE `9855bd082ad88a4c49296410f206f148d98c4df9` 그대로임을 확인했다.

```sh
/Users/rexxa/work/new-world-play2-dev/tools/play.sh main
```

완료 authoritative 결과: `heldout-final-1/outcomes.json` SHA-256 `42e9de5b1649ced049ca37a9741f15f9b900baaa61ce128eb7f84d8d5781ebed`, `why.json` SHA-256 `552ccf8b9932e23d1243b185c6fb4a35759df21a10d364f6e633430ce618a6a8`. `session-terminal.json`의 전체 완료는 최종 gate/publication 승인을 대신하지 않는다.

측정/캡처 HEAD는 각 영수증의 p7 또는 명시된 p5+patch에 그대로 귀속한다. 이후 문서 descendant에서 결과를 재사용할 때도 해당 Rust source identity와 고정 binary/controller 입력을 확인해야 하며, 나중 HEAD를 실제 측정 HEAD로 바꾸어 쓰지 않는다.

최종 원격 조회(미게시):

```text
9855bd082ad88a4c49296410f206f148d98c4df9 refs/heads/v8/main
9855bd082ad88a4c49296410f206f148d98c4df9 refs/heads/v8/wip
```
