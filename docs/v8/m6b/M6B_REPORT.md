# M6b 보고서

## Stage −1 게시 확정 기록 (Stage 0 작업 중 추가)

2026-11-02 10:45:18 UTC에 [2f6993009ddb600e9249c28f9768a1a5252c8821](https://github.com/hyunlord/new-world/commit/2f6993009ddb600e9249c28f9768a1a5252c8821)을 `v8/wip` 다음 `v8/main`에 fast-forward하고 두 ref와 review ref의 같은 SHA를 확인했다. **BASE0는 이 SHA**다. 아래 p8·이전 후보 기록은 당시 상태 그대로 보존한다. 이 후속 문서가 포함될 Stage 0 커밋은 별도 검증 대상이다.

- 최종 D의 전체 gate03은 필수49단계 exit0, metadata 감사v3과 후속 queue도 exit0이다. 원래 queue exit2, 감사v1·v2 exit1, strict stall FAIL은 보존했다. 최신 오너가 허용한 LOOP gate의 waiver로 게시했다. zoom1의 >50ms는52.701·58.127ms, zoom0.5는58.176ms, 모두60ms 미만이다. >33ms 분당 수는 각각 `[1,5,1,2,0]`, `[1,0,3,0,1]`로 남긴다.
- debug/release 각각2,853통과·0실패·13ignored. M1은229개 통과 실행(222개 고유 이름과 중복7회)이다. 원시 awk의 `inf`/228 집계는 보존하고 별도 이름별·숫자별 감사를 첨부했다.
- baseline identity와 전체 colony fixture는 BASE와 동일하다. 128개 원시 파일 쌍,9개 CORE/READ 정규화 쌍, 일반3개와F10, 실제 format14→15 저장 호환3개 검사를 구분해 보존했다. 일반 CORE 집사OFF의 BASE 지식 손실은 남아 있으며 Stage0 자기 비교의 무손실 기준과 충돌한다.
- 최종 D 첫10분 이미지 판독은 두 배율4/4, 같은 프로토콜 BASE1/4. 집중420·입력62 검사,158PNG 중 별도 정적 검토36장이다. 전체 종류 카드296검사·12종·17경로와16세계×48일 자연 실행에서950응답·29건 native 카드 수락·카드 거절0·KnowledgeLost0을 확인했다. 직접 사람의 플레이·연속 애니메이션 평가는 NOT_RUN이다.
- 성능 mean/p99 비율: n200 `1.008790/1.016270`, n1000 `1.008982/0.982165`, n2000 `1.000848/1.006368`, script `1.003925/0.995846`, side64 `0.986017/0.969118`, wolves `1.004362/1.026251`. 이전 gate02 side64 p99 실패1.121212와 원인 UNKNOWN을 보존한다.
- #650/#663은 이미 닫혀 있음을 확인했다. 미완료 집사 작업은 [ISS-361 / #684](https://github.com/hyunlord/new-world/issues/684)에 LOOP held-out 실패와 함께 이관했다. LOOP-R1 승인 무효와1680–1743 소모 상태는 변하지 않는다.

게시 영수증: `/Users/rexxa/work/m6b-data/minus1/STAGE_MINUS1_PUBLICATION.json`. 독립 감사: 같은 디렉터리의 `FINAL_D_03_PUBLICATION_AUDIT.md/json`. unrelated `.gitignore`로 인한 `worktree_clean=false`와 별도 product source clean 증거를 함께 보존했다. 실제 gate의 bridge 재빌드도 별도 출처로 묶었다. Stage0는 이 예외를 일반화하지 않고 깨끗한 별도 제품 checkout을 사용한다.

## Stage −1: BASE 집사 동작으로 프레젠테이션 게시

상태: 진행 중, 미게시. BASE0 미정. 기준 동작은 `fbc03bddb8f5b68f43fba1b2dad283982e5cf436`, 인수한 증거 분기는 `d88fca82cd7d5501d9cb5a68f0761d40b0e9bf36`이다. 오너 결정 D-minus1에 따라 M-LOOP 게시 대기는 종료했다. LOOP-R1은 승인 근거로 무효이며 실패 기록은 보존한다.

기본 집사 데이터는 BASE의 여덟 규칙으로 복원했다. 새 코드와 명령은 유지하고 기본 cap/grace는 None, extended는 false다. 카드 만료는 none이며 자동 명령을 보내지 않는다. 역할 선택은 기존 우선순위 표에서 표시하고 플레이어가 선택한 경우에만 해당 사람의 명시된 작업 우선순위를 바꾼다.

복원 전 회귀 테스트 2개가 실패했고 복원 후 통과했다. 집사 테스트는 11 통과, 외부 v6 fixture를 요구하는 기존 테스트 1개는 무시 상태다. 규칙 파싱 6개 및 UI 관찰자 테스트 9개가 통과했다. 통합 Python 비교기 테스트 30개가 통과했다. 이는 전체 gate 또는 fixture 동일성 증거를 대체하지 않는다.

검증 소스: `3f21c8f72d05bbf72073f1778774661cafdabb45` (p8). 아래 수치는 p8 사전 검증이며 이 문서를 포함한 최종 커밋의 결과가 아니다. 최종 커밋은 별도 전체 gate와 필요한 수용 증거를 완료한 뒤 v8/wip → v8/main에 fast-forward하고 외부 게시 영수증으로 확인한다. 그 SHA만 BASE0가 된다. 문서 작성 시점에는 미게시·BASE0 미정이며, 게시 결과를 쓰려고 검증 뒤 소스를 바꾸지 않는다.

- p8 커밋 전 집중 검사: 카메라417(원시 저장 바이트·tick·명령 로그 불변 포함), 순수 배치12, Python544, 기본 Why53, 한국어·영어 Why 각64, GPU8장면0실패. 이 수치는 동결 커밋의 전체 수용 결과가 아니며 그 검증을 대신하지 않는다.
동결 p8 화면·첫10분·카드 검증:

- 고정 소스 `3f21c8f72d05bbf72073f1778774661cafdabb45`의 새 실제 화면 캡처: 집중419·입력62 검사 통과, 캡처 프로세스 exit0·소스 변경 없음. 첫10분 캡처는1.0에서71장,1.25에서68장이고, 각0→14400tick과 BASE의 시드·카메라·입력 드라이버·배율·시계 프로토콜이 일치한다.
- 새로운 이미지 전용 비교 판독62장: BASE1/4 대 p8 4/4, 두 배율 모두. 현재 수확4명·제작2명·운반 준비1명, 상인 선택지, 올해 겨울 목표3/4, 계절 교역 일정에 따른 상인 방문 원인을 화면에서 읽었다. 준비를 실행으로,3/4를 완료로 해석하지 않는다.
- 별도 정적 검토36/161장: 차단할 글자 잘림·의도하지 않은 패널 겹침 없음.1.25배의4/명 등 어색한 한국어 줄바꿈과 영어 화면의 한국어 native 콘텐츠는 남아 있다. 나머지125장은 이 정적 검토에서 보지 않았다.
- 실제 OS 제스처·연속 시간 품질·전체 번역·오너 플레이는 미검증이다. 입력62 검사는 Godot 이벤트 경로 검사이며 물리 장치 제스처 검사가 아니다. native Player 출처 및 카드 응답→연대기의 인과 연결은 별도 미검증으로 남는다.
- 화면 모음은 실제 p8 화면6장을50%로 배치한 후보 기록이다. p8 사전 검증과 이 보고서를 포함한 최종 D의 전체 gate·게시 영수증은 별도로 확인한다.

- 자연 카드 검증은 같은 p8의 응답/무응답 8쌍, 총16세계×48일이다.961응답·1170시도·28 native 수락 영수증, 거절·검증 실패0. 모든 쌍의 생존자/사망 수가 같고,16세계 모두 KnowledgeLost0이었다. 이는 fbc03bdd fixture 동등성 검사를 대신하지 않는다.
- 자연 관측/응답은10종이고 storage·sick_person은 자연 실행에서 관측되지 않았다. 별도 전 종류 fixture는12종·17경로,296검사 통과: native 카드 명령12건과 로컬 동작9건, 준비 명령2건을 구분한다. 관측/control120tick 후 저장1,278,404바이트는 동일했다.
- 카드 실행은 release bridge를 사용하고 원래 debug bridge 복원을 확인했다. native 명령 로그에 Player/source 메타데이터가 없어 동기 응답 구간을 통한 귀속까지만 검증했으며, 연대기의 동시 사건을 응답의 원인으로 주장하지 않는다.

- p8 gate의 debug 단계:2,853통과·0실패·13ignored, exit0. release 단계도 exit0이며 개별 이름별 결과는2,853통과·13ignored로 이전 p7과 일치한다. 다만 자식 프로세스 출력이 요약3줄에 섞여 원본 gate의 요약 집계는2,850으로 남아 있다. `p8-release-test-accounting.json`이180요약 중177개만 파싱되는 원인과 개별2,866결과 대조를 보존한다. 단계별 집계와 전체 gate 결과를 구분하며, 원본 집계를 수정하지 않는다.

- p8 전체 gate는49개 등록 단계 종료 코드0이다. strict stall 캡처는53.209ms 프레임1개로 원래exit1을 남겼고, 다른 모든 단계가 통과하여 기존 waiver가 성립했다. 후속 외부 감사기의 정상 무출력·명령 echo 오탐으로 거절된 최초·r2 시도도 보존한다. 이 수정은 제품 코드·fixture 정규화·허용 차이를 변경하지 않았다. 최종 후속 검증은 별도 영수증으로 확인한다.

- 후속 r3는2026-11-02 04:28:00UTC에 exit0, 정확한 p8 SHA의 POSTGATE PASS로 종료했다. 9개 CORE/READ fixture 쌍과32시드×120일 baseline의128개 원시·연대기·관계 파일 쌍, 일반3 fixture, 의도적 손실 F10을 별도로 검사했다. BASE 형식14의 집사ON/OFF 실제 저장을 읽고 계속 실행·재저장하는3개 release migration 테스트가 통과했다.
- 일반 fixture의 CORE 집사OFF seed0×20일은 BASE와 후보 모두 alive0, 기아9·전투1 사망 및 tick28560 clay_digging KnowledgeLost를 유지한다. 이는 Stage −1 동등성 통과이며 Stage0의 일반 세계 무손실 기준 통과가 아니다. F10은 tick1800 고의 사망 후 doorway 손실을 확인하고, tick1799 생존은 NOT_OBSERVED로 남긴다.
- 이전 p7 `007731c7ccdd317d6f70380b98a1bb7d8dfd346c` 전체 gate는49단계를 실행하고39·40·46에서 실패했다. 후속 감사가 수용을 거절했고 보충 fixture는 실행하지 않았다. p7 첫10분4/4·카드0거절 및 성공한 성능 하위 검사를 전체 gate 통과로 바꾸지 않는다.
- 사전 전체 gate: PASS49/49, source `3f21c8f72d05bbf72073f1778774661cafdabb45`, audit `/Users/rexxa/work/m6b-data/minus1/postgate-p8-01-r3/gate-audit.json`. 전체 동일성·일반 fixture·F10 등 보충 증거: p8 fullgate49/49 and postgate-p8-01-r3 exit0: strict baseline32×120d/128 paired artifacts;9 CORE/READ pairs;ordinary3 fixtures;injected-loss F10 8×4d;save migration3tests. No changed fixtures. CORE OFF BASE loss retained, F10 pre-injection survival NOT_OBSERVED. Final documentation SHA still requires its own acceptance. 원시 증거와 해시는 `/Users/rexxa/work/m6b-data/minus1/publication-p8/admission.json`에 기록한다.
- 네이티브 Player 출처 및 응답→연대기의 인과 연결은 UNVERIFIED다. 같은 tick 사건을 인과 증명으로 쓰지 않는다.
- 성능: 아래 값은 p8 사전 gate의 실제 BASE/new3회 측정이다. 최종 문서 커밋의 측정으로 바꾸어 부르지 않는다.

### p8 실제 성능: step25 독립 원본 감사

측정일2026-11-02. 후보 `3f21c8f72d05bbf72073f1778774661cafdabb45`, BASE `fbc03bddb8f5b68f43fba1b2dad283982e5cf436`. p8-01의6개라운드 JSON과126개원시 child를 새로 읽고 검산했다. **수치·부하·C3 감사 PASS**, 실패 막대 없음. `bench.log`는 READ_BENCH PASS다. 이 성능 감사는 전체 gate 종료 전에 수행했다. 이후 전체 gate·후속 검증 결과는 위의 p8 검증 기록에서 별도로 확인하며, 최종 문서 커밋의 게시를 주장하지 않는다. p7 숫자를 복사하지 않았다.

실제 admission 시간 UTC `2026-11-02T03:46:23+00:00`–`2026-11-02T03:52:56+00:00`(KST+09:00, 마지막은 마지막 child 시작시각). 로그 step25 시작12:45:23로컬. release, 시드15100, warmup1440, measured2880틱;6장면×3라운드×2군=36timed child,3장면×5반복×6실행=90RSS build-only child. 각 라운드의 timed repeat는1개이며 세 라운드 중앙값을 비교한다.

#### 평균·p99 중앙값과 비율

각 통계의 중앙값을 별도로 구했다. 후보/BASE≤1.10, 예외 없음. 표시는 반올림이고 판정은 원본 정밀도다.

| 장면 | BASE 평균µs | p8 평균µs | 평균비율 | BASE p99µs | p8 p99µs | p99비율 |
|---|---:|---:|---:|---:|---:|---:|
| n200 | 433.226041667 | 432.574305556 | 0.998495621 | 806 | 830 | 1.029776675 |
| n1000 | 2019.648263889 | 2015.791319444 | 0.998090289 | 3939 | 3936 | 0.999238385 |
| n2000 | 2997.011111111 | 2987.126041667 | 0.996701691 | 7061 | 7025 | 0.994901572 |
| n200_script | 512.615972222 | 505.133333333 | 0.985403032 | 986 | 964 | 0.977687627 |
| n200_side64 | 373.457638889 | 374.102083333 | 1.001725616 | 677 | 659 | 0.973412112 |
| n1000_wolves | 2246.364583333 | 2223.204861111 | 0.989690132 | 4828 | 4797 | 0.993579122 |

#### 각 반복의 C3

n1000평균≤5000µs, n2000평균≤10000µs, allocator/agent≤4096B, allocator/tile≤96B, RSS별도5반복중앙값/agent≤5120B·/tile≤128B. allocator는 timed raw의 after_build 차분/1800 또는/32768; RSS는 build-only raw의 after_build KiB 차분×1024/1800 또는/32768로 재계산했다. 후보3반복 각각6/6PASS; BASE도 보조 확인했다.

| 군/라운드 | n1000평균µs | n2000평균µs | alloc/agent B | alloc/tile B | RSS/agent 중앙값 B | RSS/tile 중앙값 B | C3 |
|---|---:|---:|---:|---:|---:|---:|---|
| base-r1 | 2019.648263889 | 2997.011111111 | 1863.393333333 | 68.795288086 | 2157.226666667 | 97.5 | 6/6PASS |
| new-r1 | 2026.143055556 | 2987.126041667 | 1863.393333333 | 68.795288086 | 2184.533333333 | 93.0 | 6/6PASS |
| base-r2 | 1981.028472222 | 2996.694791667 | 1863.393333333 | 68.795288086 | 2129.920000000 | 97.0 | 6/6PASS |
| new-r2 | 2015.791319444 | 2986.804166667 | 1863.393333333 | 68.795288086 | 2129.920000000 | 95.0 | 6/6PASS |
| base-r3 | 2021.251388889 | 3019.090625000 | 1863.393333333 | 68.795288086 | 2111.715555556 | 93.0 | 6/6PASS |
| new-r3 | 2009.452430556 | 3026.645486111 | 1863.393333333 | 68.795288086 | 1993.386666667 | 104.0 | 6/6PASS |

| 군/라운드 | 원시 RSS/agent 5개 B | 원시 RSS/tile 5개 B |
|---|---|---|
| base-r1 | 2157.226666667, 2093.511111111, 2248.248888889, 2111.715555556, 2621.440000000 | 98.5, 97.5, 93.0, 97.0, 101.5 |
| new-r1 | 2184.533333333, 2184.533333333, 2148.124444444, 2202.737777778, 2193.635555556 | 93.0, 89.0, 94.5, 103.5, 90.0 |
| base-r2 | 2129.920000000, 2166.328888889, 2129.920000000, 1419.946666667, 2211.840000000 | 95.0, 96.5, 98.0, 134.0, 97.0 |
| new-r2 | 1447.253333333, 1847.751111111, 2193.635555556, 2129.920000000, 2384.782222222 | 131.5, 108.5, 91.0, 95.0, 90.0 |
| base-r3 | 2111.715555556, 2075.306666667, 2102.613333333, 2175.431111111, 2594.133333333 | 98.0, 94.5, 93.0, 93.0, 90.0 |
| new-r3 | 1966.080000000, 2630.542222222, 1793.137777778, 2503.111111111, 1993.386666667 | 102.5, 95.5, 129.5, 104.0, 130.5 |

PERF_LEDGER의 RSS 열은 timed process의 `bytes_per_agent`·`bytes_per_tile`이며, 위 C3의 별도 build-only 5회 중앙값과 다른 관측이다. allocator 열과도 구분한다.

RSS 판정은 위5개 중앙값이다. 모든 개별값까지 상한 안인지의 추가 관측: False. 개별값 확인을 새 gate 규칙으로 해석하지 않는다.

#### 실제 실행 순서·부하·timing

순서 `base-r1 → new-r1 → base-r2 → new-r2 → base-r3 → new-r3`는 bench.log와 admission시각으로 확인했고, 각 summary완료mtime이 다음라운드 첫 admission보다 이르다. 순서 입증의 mtime 한계는 아래에 남긴다.
ADMIT126건, WAIT5건.126raw child의 시작부하1분<8, 최대7.37. 모든1/5/15값은 raw와 admission에 있으며 timed값을 아래에 표시한다.5/15분≥8은 현 시작1분 규칙의 실패가 아니다.

| 군/라운드 | 장면 | 평균µs | p99µs | maxµs | 시작load1/5/15 |
|---|---|---:|---:|---:|---|
| base-r1 | n200 | 433.326736111 | 795 | 1476 | 6.57 8.90 7.99 |
| base-r1 | n1000 | 2019.648263889 | 3977 | 5377 | 6.61 8.87 7.99 |
| base-r1 | n2000 | 2997.011111111 | 7061 | 8458 | 6.85 8.75 7.99 |
| base-r1 | n200_script | 503.257986111 | 974 | 1693 | 6.79 8.65 7.97 |
| base-r1 | n200_side64 | 373.457638889 | 677 | 928 | 6.79 8.65 7.97 |
| base-r1 | n1000_wolves | 2246.364583333 | 4830 | 5732 | 6.79 8.65 7.97 |
| new-r1 | n200 | 431.384027778 | 830 | 1435 | 7.37 8.67 7.98 |
| new-r1 | n1000 | 2026.143055556 | 4001 | 5394 | 7.37 8.67 7.98 |
| new-r1 | n2000 | 2987.126041667 | 6923 | 8020 | 6.06 8.37 8.00 |
| new-r1 | n200_script | 501.481597222 | 945 | 1744 | 5.45 8.12 7.92 |
| new-r1 | n200_side64 | 377.136111111 | 693 | 1972 | 5.45 8.12 7.92 |
| new-r1 | n1000_wolves | 2223.204861111 | 4781 | 5599 | 5.45 8.12 7.92 |
| base-r2 | n200 | 427.893055556 | 806 | 1422 | 5.26 7.95 7.86 |
| base-r2 | n1000 | 1981.028472222 | 3885 | 5407 | 5.26 7.95 7.86 |
| base-r2 | n2000 | 2996.694791667 | 7084 | 8511 | 4.75 7.75 7.79 |
| base-r2 | n200_script | 522.633333333 | 1066 | 1794 | 4.14 7.47 7.69 |
| base-r2 | n200_side64 | 393.740972222 | 759 | 1804 | 4.14 7.47 7.69 |
| base-r2 | n1000_wolves | 2260.197916667 | 4807 | 5684 | 4.14 7.47 7.69 |
| new-r2 | n200 | 449.377083333 | 916 | 1825 | 4.17 7.32 7.63 |
| new-r2 | n1000 | 2015.791319444 | 3918 | 5623 | 4.17 7.32 7.63 |
| new-r2 | n2000 | 2986.804166667 | 7025 | 8433 | 3.76 7.12 7.56 |
| new-r2 | n200_script | 505.133333333 | 964 | 1706 | 4.46 7.12 7.55 |
| new-r2 | n200_side64 | 374.102083333 | 659 | 927 | 4.46 7.12 7.55 |
| new-r2 | n1000_wolves | 2257.776388889 | 4797 | 5773 | 4.42 7.06 7.53 |
| base-r3 | n200 | 433.226041667 | 832 | 1391 | 4.20 6.93 7.47 |
| base-r3 | n1000 | 2021.251388889 | 3939 | 5481 | 4.20 6.93 7.47 |
| base-r3 | n2000 | 3019.090625000 | 6996 | 8739 | 4.09 6.82 7.43 |
| base-r3 | n200_script | 512.615972222 | 986 | 1767 | 3.72 6.60 7.34 |
| base-r3 | n200_side64 | 372.388541667 | 652 | 983 | 3.72 6.60 7.34 |
| base-r3 | n1000_wolves | 2242.127083333 | 4828 | 5874 | 3.82 6.57 7.32 |
| new-r3 | n200 | 432.574305556 | 819 | 1448 | 3.85 6.48 7.28 |
| new-r3 | n1000 | 2009.452430556 | 3936 | 5525 | 3.85 6.48 7.28 |
| new-r3 | n2000 | 3026.645486111 | 7225 | 9253 | 3.48 6.32 7.21 |
| new-r3 | n200_script | 505.955555556 | 971 | 1762 | 4.10 6.32 7.20 |
| new-r3 | n200_side64 | 370.448611111 | 658 | 886 | 4.10 6.32 7.20 |
| new-r3 | n1000_wolves | 2219.070833333 | 4825 | 5790 | 4.09 6.28 7.18 |

#### 명령·바이너리 출처와 한계

launcher의 candidate/base-source.txt는 정확한 각SHA와 빈 dirty status를 기록했고 selected-environment는 후보SHA를 기록했다. 실행된 runner와 gate 소스의 현재 해시는 input-sha256의 시작시점 해시와 일치한다. BASE 실행 파일 `/Users/rexxa/work/m6b-data/worktrees/minus1-base/rust/target/release/worldsim_perf`의 현재 해시는 config와 시작 manifest 양쪽의 `b56aed4fce6a2af09adde318e018b2341c2be1c520547395dc317bd3609fa386`에 일치한다. step12 perf.log는 release sim-test build 경로를 기록한다.

Runner의 명령은 `<arm binary> bench --out <arm-rN> --commands /Users/rexxa/work/new-world-m6b-dev/tools/v8/bench_script.json --repeats 1 --commit <arm full SHA>`이며 `WORLDSIM_GIT_SHA`도 각군SHA로 따로 지정한다.6summary의commit이 각군과 일치하고 원시36개모두release·동일seed/warmup/ticks, n200_script만script=true다. 후보 실행경로 `/Users/rexxa/work/new-world-m6b-dev/rust/target/release/worldsim_perf`의 **감사시점** SHA256은 `54dea8c65a651c3ef76a4460657a6444731eea09a2e3b509e1e93d053df4eb3b`, command JSON의 **감사시점** SHA256은 `cb655db4a4207c9c8fa07a3303d0db5b309e90a80221bf5aa2eb0972bd8aaa31`다. 이 둘은 launcher input manifest에 시작시점 pin이 없으므로 개별 child가 그 정확한 바이트로 실행됐다는 독립 exec-trace 증명은 없다. label을 바이너리 해시 증명으로 바꾸어 주장하지 않는다.

전체 원본 루트 `/Users/rexxa/work/m6b-data/minus1/gate-attempts/p8-01`. [감사 JSON](/Users/rexxa/work/m6b-data/minus1/publication-p8/p8-performance-audit.json) SHA256 `259f99d5e064c9f53964865113864e6fc735638bca1769b0df36f7c450c1a6af`에 원시126개·summary6개·admission6개·관련 소스/런처 증거의 해시와 검산 결과를 기록했다.

| summary/로그 | SHA256 |
|---|---|
| `fullgate/bench/base-r1/bench.json` | `052d2dcf7a7c1fd79affd43913dfde10ba903ffb9b3fc872a4f2d4c0d746346a` |
| `fullgate/bench/new-r1/bench.json` | `a4c81beeebd1bc91d0a5d79af28f75db54f901a85e5c1e344c92e10e17f856f2` |
| `fullgate/bench/base-r2/bench.json` | `7d99c758dd502cd61a0efbf1df88af83223171c2016868c83f9d3fb9e6689293` |
| `fullgate/bench/new-r2/bench.json` | `cb041bdd3b18f6dad10da8a8bb5d69a1f7d6bd64c7422aa46f94e8055e7e87fb` |
| `fullgate/bench/base-r3/bench.json` | `439511667eba8d2346dc29a96da112b7b624b12f78e388690de3a08e37203d5a` |
| `fullgate/bench/new-r3/bench.json` | `7dcaafe4aea4796c98fb7ac162d090501372b873f05805ec03839abf50eb2ef6` |
| `fullgate/bench.log` | `e844a8a859f2e540d07e3ea924aab04f1f892fbae2202f1d86acb7a0b8cc2eca` |

남은 한계: 개별 실행의 바이너리/command 시작시점 해시 인증과 독립 프로세스 목록은 이 감사에 없다. 단일 측정 작업 소유는 리더의 실행 영수증에 의존한다. 장면별상대비율은 timing중앙값 규칙이고 allocator상대비율 검사를 READ validator가 수행한다고 주장하지 않는다. 이 오프라인 감사는 fullgate의 완료·동일성·UI·게시 검증을 대신하지 않는다. 장부 append는 전체 gate가 GREEN인 뒤에만 가능하며 이번 작업은 수행하지 않았다. 저장소 편집·git·빌드·테스트·새 세계·GPU 실행은 하지 않았다.

과거 검증 상태(아래 p1–p7의 FAIL·중단·미검증은 그대로 보존):

검증 상태:

- BASE 장기 identity: 캡처 완료, 기준 자료 확보.
- 후보 baseline/전체 colony/CORE/READ 동일성: p4 전체 gate는 화면 결함 수리를 위해 중단했으며 완료 판정은 없다. p4의 일반 no-beasts soak·CORE 집사 OFF/ON 및 별도 knowledge F10 8세계는 BASE 대비 정규화 바이트가 일치했다. 전체 동일성 증거를 대체하지 않는다.
- 첫 10분 새 이미지 검토: BASE 2/4, p2 3/4, p3의 마지막 day10 화면은 4/4(두 배율 동일). p3은 실제 PNG 60장을 새로 읽은 결과이며 과거 LOOP 4/4를 재사용하지 않았다. 모든 중간 사건의 원인이 설명된다는 뜻은 아니다. 화면 겹침은 별도 실패로 남아 있다.
- 기존 UI 회귀 검사: 하네스의 scene-tree/locale 오류를 수정한 재실행은 201/201 통과, 엔진 오류 0건이다. 이전 199개 assertion 통과 기록은 오류가 함께 있어 승인 증거에서 제외하고 보존한다. p1(`6edbc7355`)의 전체 12종·17경로 검사와 자연 발생 카드 실행에서 거절 명령은 0건이었다. 그러나 seed5에서 무응답 사망0명 대비 권장 선택 사망1명(추위)이 발생하여 카드 생존 검증은 FAIL이다. 기존 하네스가 허용하던 +1 사망 판정은 승인 근거로 쓰지 않는다.
- 정확한 커밋 전체 gate: p4 실행 중단(CANCELLED), PASS 아님. 수정 커밋에서 첫 단계부터 다시 실행한다.
- v8/wip, v8/main 게시 및 이슈 처리: NOT_RUN.

비교 제외 경로는 MINUS1_NORMALISATION.md에 한정한다. 변경 fixture 선언은 없다. 원시 로그는 `/Users/rexxa/work/m6b-data/minus1/`에 보존한다. 프레젠테이션이 복원된 기본 동작에 명령이나 상태 변화를 누출하면 원인을 고친 후 다시 비교한다.

추가 사전 검증: 창을 띄운 입력 검사에서 다섯 패널의 휠 캡처, 지도 휠 확대, 목록 교체, 실제 인물의 원인 팝업과 지상 아이템 팝업 교체·바깥 클릭 닫기, 두 글자 크기의 역할 선택을 통과했다. 입력 전후 tick과 명령 로그는 같았다. 우선순위 표를 가리던 카드·지도·도움말은 목록이 전경에 있는 동안 표시만 보류하고 닫으면 현재 상태로 복원한다. 관련 회귀 25개가 통과했으며 카드 만료와 내용은 바뀌지 않았다. 이 결과들은 커밋 전 진단이며 최종 SHA 검증은 별도로 수행한다.

카드 영수증은 실제 네이티브 명령 로그의 인덱스·종류·수락 여부·tick과 대조하고 연대기와 KnowledgeLost 관찰값을 기록한다. 기존 네이티브 로그에는 Player 출처 필드가 없으므로 이를 추정해 PASS로 쓰지 않는다. 같은 tick의 연대기 항목 역시 시간적 일치이며 인과 연결을 증명하지 않는다. 자동 명령 부재는 별도의 수동 UI/만료 및 미래 저장 바이트 동일성 검사로 검증한다.

추가 수리의 제한된 검증: 교역 카드는 기존 조건 창을 여는 표시 동작으로 바꿨다. 실제 상인 세계에서 권장 버튼으로 보낸 마우스 입력은 1.0·1.25 배율 모두 해당 창을 열었고 tick·명령 로그가 같았다(30개 검사). 잘못된 카드 판정의 독립 음성 사례 17개가 수리 전 실패, 수리 후 통과했다. 이 결과는 수정 빌드의 8세계 생존 재실행이나 전체 gate를 대체하지 않는다.

p2(`b12df27ee`) 자연 카드 재실행은 seed5 회귀를 해소했으나 seed3 권장 선택 사망5명 대 무응답2명(추위)으로 FAIL이다. 12종·17경로와 16세계에서 거절 명령은 0건이며 KnowledgeLost는 0이었다. 이주 카드 단독 진단에서도 5 대 2를 재현했다. 기존 권고는 도착 인원과 추가 침상·연료 수요를 무시했다. p3 수정은 실제 도착 인원의 초기 식량 수요를 읽기 전용으로 제공하고, 실제로 걸어갈 수 있는 완성된 지붕 방의 침상 여유·현재 연료 목표·도착 인원을 포함한 5일 식량을 확인한다. 누락되거나 불명확한 관찰값은 거절 권고로 처리하며 직접 수락 선택은 유지한다. 이는 미래 추위나 부패를 예측하지 않는다. 첫 수정의 집중 검사 79개와 실제 이주 세계 19개가 통과했다. 이후 경로 도달 가능성 누락을 독립 검토에서 발견하여 추가 회귀 14개 실패를 재현했고, 네이티브 침상 관찰값을 사용한 집중 검사 75개와 Rust 이웃 FFI 검사 5개가 통과했다. 지붕 있는 침상만 세는 보수적인 권고이며 오두막·쉼터 전체 수용력을 주장하지 않는다. 도달 가능성 수정 후 실제 이주 세계 19개 검사와 clippy도 통과했다. 정지 상태 ko→en→ko 전환 24개 검사는 tick·명령·카드 상태 보존을 확인했다. p3(`bc6dc2ebc`)의 12종·17경로와 자연 8쌍 세계 시험은 PASS다. 모든 세계에서 사망 수와 마지막 생존 인원이 무응답 대조군과 같았고, 거절 명령 및 KnowledgeLost는 0이었다. 같은 SHA의 입력 검사 62개와 양 배율 0→14400 tick 캡처도 완료했다. 다만 1.0 화면에서 알림이 작은 지도 위로 넘치고, 1.25 화면에서는 상인 안내가 알림 제목을 덮어 화면 승인에는 실패했다. 해당 배치 수리와 새 SHA 검증이 필요하다. 우선순위 표에서 스크롤 후 열 이름이 사라지는 문제도 수정하여 실제 스크롤·표적 셀 클릭 검사 80개가 통과했다. 빈 화자 콜론과 식량 요청의 반복 인과 표현 수정은 언어 전환 검사와 함께 33개가 통과했다.

상인 도착 문구에는 실제 생성 규칙인 이웃의 교역 가능 여부와 계절 방문 일정을 설명한다. 기록되지 않은 과거 호감도나 특정 행동을 원인으로 꾸미지 않는다. 카드 넘침은 글자 크기를 줄이지 않고 버튼 줄바꿈과 스크롤로 다룬다. 상단 버튼과 목표 문구의 겹침을 추가로 수정했으며 배치 검사 124개와 실제 화면에서 확인했다. 이 값은 최종 커밋의 전체 화면 평가를 대신하지 않는다.

p4 배치 사전 검증: 방문자 안내와 경고의 자리를 따로 확보하고 긴 경고·소집 목록만 스크롤한다. 교역·이주 목록도 기존 스크롤 영역을 줄여 선택 버튼과 요청한 글자 크기를 유지한다. 실제 방문자에 합성 경고를 더한 검사 5367개, 방문자 단독 1372개, 이주 991개, 동적 알림 298개가 오류 없이 통과했다. 초기 스크롤 위치에서 안내가 온전히 보이고 아래 행도 스크롤로 접근 가능함을 화면으로 확인했다. 이것은 p4 커밋의 첫10분 재검토·카드 생존·전체 게이트를 대체하지 않는다.

추가 UI 회귀에서 과거 메뉴 수·연대기 종류 수·기본 필터·교역 거절 방식에 묶인 오래된 검사 가정을 발견했다. 실제 표시와 명령 유무를 검사하도록 고쳤으며 실패 로그를 보존한다. 별개의 실제 결함인 연대기 묶음의 숫자 코드와 잘못된 원인 요약은 원래 사건 문장으로 바꿨다. 묶음을 펼치면 각 사건의 실제 원인 링크를 볼 수 있고 지도 강조도 내용 변경에 맞춰 갱신한다. 두 언어·두 배율에서 실제 마우스 클릭을 포함한 165개 검사가 통과했으며 tick·전체 명령 로그·네이티브 저장 바이트는 동일했다. M-READ 32개, 수정된 M5 43개·M5.5 36개와 M-LOOP 201개도 통과했다. 변경된 GDScript 16개와 FFI 호출 73개의 엄격 검사 및 문서 검사도 통과했다. 최종 SHA의 통합 검증은 별도로 기록한다.

p4(`73ae254fd99078ff438762f7b84fbe814e748fcc`) 최종 카드 실행: 자연 발생 48일 × 16세계에서 검증 실패·거절 명령·KnowledgeLost가 모두 0이었다. 951회 응답 중 실제 명령 영수증은 28건이며 전부 수락됐다. 시드별 생존 인원은 두 군 모두 `[9,11,11,11,10,13,11,12]`, 사망은 `[1,0,0,2,0,0,0,0]`였다. 자연 발생으로 관측한 카드는 10종이고 storage는 미관측이다. 별도 전체 종류 검사는 12종·17경로에서 거절 0건을 확인했다. 관찰자 수동 동작의 미래 명령 로그와 저장 바이트 동일성도 별도로 통과했다. 이 결과는 모든 플레이 선택의 안전성이나 기록되지 않은 명령 출처를 증명하지 않는다.

p4 첫10분 캡처는 두 배율 모두 tick0→14400을 정확히 실행했다. 새 검토자가 실제 PNG 58장을 읽어 최종 시점 4/4를 부여했다(BASE 새 검토 2/4). 초기 기술 발견·방 생성의 원인 설명 누락은 남아 있다. 독립 배치 검토 48장에서는 배율1.25 역할창 하단 드롭다운을 상태 알림이 가리는 결함으로 FAIL이다. 같은 후보의 전체 gate `p4-01`은 fmt·bridge build·Godot import 통과 후 디버그 테스트 중 중단했다. 시작한 프로세스 그룹만 종료했으며 실제 종료143과 원시 로그를 보존했다. 런처 EXIT trap의 원시 종료 기록0은 성공 증거가 아니며 `cancellation-terminal.json`에 차이를 기록했다. 전체 gate 완료·성능·게시 판정은 없다.

후속 배치 수리: 상태 알림은 전경 창 옆 여백에서 줄바꿈하고, 공간이 없으면 전용 줄을 확보한다. 우선순위·기술 지도는 기존 스크롤 영역을 사용한다. 실제 ‘왜?’→화덕 클릭→기술 지도 입력에서도 원인 팝업이 목록을 덮는 문제를 확인해 별도 자리를 확보했다. 알림의 내용·수명·대기열과 원인 팝업의 기존 위치 선택 규칙은 유지했다. 최종 집중 검사2934개가 통과했고 두 언어·두 배율에서 실제 역할 클릭, 연속 프레임 배치, 긴 알림, 팝업 공존, 닫은 뒤 원상복구, tick·전체 명령 로그·저장 바이트 동일성을 확인했다. 첫 GREEN의 긴 알림 글자 크기 실패와 합성 원인 이벤트의 잘못된 타입은 별도 실패 로그로 보존한다. 이는 아직 새 커밋의 전체 gate나 최종 독립 화면 승인이 아니다.

같은 최종 수리 소스에서 우선순위 헤더 회귀80개와 실제 상인·경고·소집 스트레스5367개도 실패·오류 없이 통과했다. 변경된 GDScript5개와 FFI73개 엄격 검사, 문서 검사도 통과했다.

게시 전 최신 검증 기록: p5(`23255d2d`) 자연 카드16세계는960회 응답·27건 수락 영수증, 거절·검증 실패·KnowledgeLost0 및 시드별 동일 생존 결과였다. 독립 배치 검토46장은 통과했지만 첫10분 판독52장은 양 배율3/4였다. 과거 집사의 집 계획 알림은 주민의 **현재** 행동을 설명하지 못했다. p5 전체 gate는 디버그2853개 통과 뒤 릴리스 컴파일 중 취소143으로 보존했으며, 성능·최종 동일성·게시 성공을 뜻하지 않는다.

명판에 기존 읽기 스냅샷의 현재 행동별 인원과 나머지 수를 추가했다. 이동·식사·수면·일·활동을 구분하고, 소집 대기를 작업으로 오인하거나 과거 계획을 현재 의도로 표시하지 않는다. 상호작용 종류가 불분명하면 불명으로 남긴다. 독립 검토가 발견한 일시정지 명령 뒤 캐시 문제는 기존 읽기 계층의 무효화 알림으로 고쳤다. 최종 집중 검사252개와 엄격 검사4개 스크립트/FFI73개가 통과했다. 수동 표시 전후 저장 바이트·tick·명령 로그 동일성과 실제154틱 행동 전환을 확인했고, 별도 의도적 소집 명령 한 건은 같은 tick의 화면에 반영되며 추가 명령이 없었다. 이 결과는 새 커밋의 첫10분·카드·전체 gate를 대신하지 않으며 아직 미게시다.

p6(`67b1ba2e1`)는 정적 화면62장과 카드16세계·48일을 통과했다(949응답,27수락 영수증, 거절·검증 실패·KnowledgeLost0, 시드별 사망 증가 없음). 별도 전체 종류 검사는12종·17경로이며 실제 카드 선택21건과 명시적 준비 명령2건을 구분한다. 그러나 새 첫10분 판독32장은 두 배율3/4였다. “일하기6명·이동 중4명”은 작업 목적을 설명하지 못하므로 실패를 유지하고 p6 전체 gate는 실행하지 않았다.

후속 수정은 현재 상태와 연결된 기존 작업 코드를 이용해 “벌목 작업”, “건설 재료 운반 준비” 등을 표시한다. 교사 대기는 학습 진전으로 단정하지 않고, 유휴 상태의 배정·소집·불명확한 상호작용은 분리한다. 회귀63개 실패를 재현한 뒤 집중420개가 통과했다. 실제 native 작업별 인원과 화면, 행동 전환, 수동 표시 전후 저장 바이트·tick·명령 로그 동일성을 확인했다. 새 첫10분 이해도와 커밋별 카드·전체 fixture gate는 아직 필요하며, 미게시 상태다.

## Stage 0 개발 검증 중간 기록

Stage 0은 미게시이며 전체 generic gate·BASE 자기 비교·고의 결함 후보 증명은 아직 완료하지 않았다. 새 관찰자는 기본 비활성이고, 원시 결과와 자식 프로세스·실행 파일·입력 해시를 묶는 도구를 추가 중이다. 기존 gate의 성공이나 단위 검사 결과를 generic gate 통과로 대신하지 않는다.

개발 중 foundation 빌드의 identity는 공유 값228,748개에서 차이0, chronicle64파일과 샘플64파일 일치, seed0–3 determinism을 통과했다(`stage0/development-identity-01.log`). 이 실행 중 후속 M6a companion 코드가 추가되었으므로 현재 전체 소스나 새 커밋의 identity 증거로 사용하지 않는다. 당시 workspace check·clippy, 집중 Rust30개와 perf observer7개는 통과했다. Python 통합256개는 `integrated-python-03.log`에 기록하며 이후 추가된 실행 어댑터는 별도 재검증 대상이다.

M6a 실제 selected-pack 주 실행은 tick1409에 성공했고, 별도 고정 종료 companion은 tick1410까지 진행했다. 두 실행 모두 시작10명·종료10명, KnowledgeLost0·사망0이다. 첫 실행은 테스트 자식의 콘텐츠 초기화 누락으로 실패했으며 로그를 보존했다. 생산 초기화 경로를 호출한 수정 후 두 집중 release 검사가 통과했다(`stage0/m6a-companion/native-20261102-r2/`). 이는 BASE 대 후보의 source-bound 증명이나 전체 gate가 아니다.

후속 동결 개발 검사 `stage0/development-foundation-02/`는 workspace 전체 대상 check·clippy, release probe 빌드 및 전체 identity를 통과했다. 실행 전후 Rust·RON·Cargo 소스와 기준·하네스 해시가 같았으며, 공유 값228,748개 차이0, chronicle64파일19,683항목·원시 샘플64파일 일치, seed0–3 결정성을 확인했다. 이후 벤치 관찰 JSON 할당을 capacity companion 뒤로 옮겼으므로 이 identity 결과를 수정 후 전체 소스에 소급하지 않는다. 해당 순서 수정의 집중 Rust10개는 통과했고 실제 이중 옵션 할당량 측정은 NOT_RUN이다.

`stage0/integrated-python-06/`은 검사 대상 소스가 전후 동일한 상태에서382개를 통과했다. 앞선05 실행은 작성 중 모듈과 겹쳐10오류가 발생한 무효 통합 시도로 보존한다. 이후 검토에서 원시 산출물의 늦은 변조, 이전 벤치 해시 덮어쓰기, CRLF 정규화로 인한 바이트 비교 누락을 재현하여 수정했다. 이 후속 수정과 새 저장 이력·UI 실행 모듈은 다시 통합 검증해야 한다. 단위 검사 통과를 실제 전체 세계 실행이나 게시 근거로 사용하지 않는다.

Godot 인증된 smoke 비교에서 boot는 tick80, labels는 tick83의 별도 실행과 일치했다. 네 프로세스 모두 종료0, 생존10명·KnowledgeLost0, 명령16개의 전체 메타데이터·인원·이력이 같았다. labels는592개·raw key0을 확인했다(`stage0/ui-companion-native/RESULT.json`). 실행 파일·입력 해시는 인증했으나 빌드 출처는 UNVERIFIED이므로 전체 gate나 exact-SHA 제품 검증으로 사용하지 않는다. 첫 인증 시도는 빈 argv 기록과 정상 native 집사 명령을 금지한 어댑터 가정 때문에 거절되었다. teardown 가설은 캐시 후에도 빈 argv였으므로 기각했다. Godot이 소비하는 엔진 인자와 내부 노출 인자를 구분하고 전체 프로세스 argv는 별도로 검증한다. 정상 초기 SetSteward와 이후 native Steward 기록은 원래·별도 실행 사이에서 전체 로그를 대조한다. 실패 기록과 원래 native 집사 동작을 유지한다.

완료된 작업 디렉터리 정리는 원본·WIP·ignored 결과를 보관하고 독립 Git 복원 증명 뒤 수행했다. loop/core/mux2/m6a/q4 및 등록된 하위18개를 제거했고, 활성 프로세스가 있던 m6a-r2는 보존했다. 상세 목록과 제외 이유는 `minus1/archives/cleanup-20261102T1050Z/FINAL_SUMMARY.json`에 있다.

후속 Python 통합07은 검사 소스가 전후 동일한 상태에서420개를 통과했다. 고정 UI 실행02는 toast·layout·ground·people·trees 다섯 native 검사 모두 종료0·빈 stderr로 통과했다. 첫 UI 실행의 layout은244개 도메인 검사를 통과했어도 shader cache ERROR 때문에 FAIL로 남겼다. 원인은 실행 환경에서 HOME을 제거하여 Godot 캐시 경로가 상대 경로가 된 것으로 별도 실제 프로세스에서 재현했고, 실제 계정 HOME만 명시적으로 전달해 수정했다. 앱 설정은 계속 새 출력 디렉터리에 격리한다. 증거는 `stage0/ui-fixed-native/`에 있으며, 두 제품의 독립 빌드·비교를 대신하지 않는다.

렌더러·가시성·Why·네이티브 테스트 연결부의 독립 검토 후 통합09는512개 검사와 소스 전후 동일성을 통과했다. generic 생산 모듈 전체의 엄격 타입 검사10은 오류0·경고0이며 Ruff 검사도 통과했다. ASCII 관찰 경로는 기본 렌더링 출력·저장 바이트 보존을 포함한 집중7개와 기존 외형 검사1개를 실제 native 실행으로 통과했다. 테스트 바이너리는 `cargo test --no-run`의 실제 compiler-artifact에서 선택하고 종류·프로필·새 빌드 여부·실행 파일 경로와 해시를 검증한다. BASE 관찰자 패치의 허용 경로에는 ASCII 인자 기본값 추가가 필요한 기존 외형 테스트 한 파일만 추가했다. 관련 경로 거절 회귀를 재현한 뒤 빌드 계약30개가 통과했다. 이 기록은 전체 generic gate나 독립 BASE/후보 실행의 완료를 뜻하지 않는다.

실제 save_load 바이너리는7개 검사와46개 세계 이력(새 세계29·불러온 세계17)을 모두 통과했다(`stage0/native-save-history/run-01/`). 별도 새 release 빌드의 colony_world·colony_commands는17개 검사와23개 세계 이력을 통과했다(`stage0/native-named-history/run-01/`). 후자는 유효 입력1107개의 해시·HEAD·작업 상태가 실행 전후 같았다. 두 실행 모두 누락 이력은 없으며 한 개발 제품의 관찰자 검증이다. BASE 대비 생존·독립 두 제품·전체 gate 승인으로 승격하지 않는다.

Stage 0 자기 비교의 알려진 충돌은 유지한다. 공개 BASE0의 기존 `core.steward_off` seed0·20일 세계는 창립자10명에서 생존0명, 굶주림 사망9·전투 사망1이며 tick28560에 `clay_digging` KnowledgeLost(보유자9)가 발생했다. 실제 진단과 엄격한 B/B 판정은 FAIL이다(`stage0/core-off-native-01/`, `design/S0_BASE_SAFETY_READINESS.md`). 원래 fixture의 기간·명령·분류나 절대 KnowledgeLost0 기준을 바꾸어 자기 비교를 통과시킨 것으로 기록하지 않는다.

후속 foundation03은 workspace 전체 대상 check·clippy, release probe 빌드와 원래32세계×120일 identity를 통과했다. 공유 값228,748개 차이0·한쪽 전용 경로0, chronicle/relation64파일19,683항목 및 원시 샘플64파일 일치, seed0–3 결정성을 확인했다. native 소스·기준 자료·identity 하네스의 전후 해시는 같았다. 실행 중 Python 연결부와 Why 도구는 추가되었으므로 전체 작업 트리 동결이나 최종 커밋의 gate로 주장하지 않는다(`stage0/development-foundation-03/RESULT.md`).

실제 가시성 첫 실행은 Godot JSON의 정수→실수 표현 차이를 연결부가 거절하여 FAIL로 보존했다. 가시성 manifest 경계에서만 안전한 정수를 Godot 표현으로 맞추고 값·경로·해시 변경 거절 회귀를 검증한 뒤 새 run02를 실행했다. 원래 네 저장 시점, 실제10세계·저장 연결4개·GPU8장면을 통과했고 모든 세계 생존10명·KnowledgeLost0이었다. 소스1470개와 runtime 입력6418개의 해시가 실행 전후 같았다(`stage0/ui-visibility-native/RESULT-run02.md`).

렌더러 실제 run01은 원래 생성기·ASCII·Godot·비교기를 모두 exit0으로 종료했다. 사람10명의 위치·외형, 타일3개, tick900 일치 및 실제4세계 이력을 확인했다(`stage0/renderer-native/run-01-summary.md`). Why run01은 bridge 빌드·부모/locale strictcheck 후 기본53·영어64·한국어64 검사를 통과했다. 세 실행의 실제 prefix→terminal tick은 각각7→8·8→9·7→8이며 모두 생존10명·KnowledgeLost0, exit0·빈 stderr였다. 직접 건물 이벤트 주입은 기제 검사로 명시하며 자연 발생 사건이나 명령 로그로 바꾸어 분류하지 않는다. 설정 복원과 전체 입력·출력 해시 재검증도 통과했다(`stage0/ui-why-native/run-01/README.md`). 이 세 실행은 개발 제품 한쪽의 연결부 검증이고, 독립 BASE/후보 비교·동일 종료 조건·전체 gate·사람의 시각 수용 검증은 아니다.

최신 Python 검증10은 generic468개와 기존 요약/READ/정규화61개를 모두 통과했다. generic 실행 전후 Python 소스 해시 변화는 없으며, 생산 모듈 엄격 타입 검사12도 오류0·경고0이었다. 검토한 작업 경로145개만 Stage0 리뷰용 변경으로 보존하며, unrelated `.gitignore`와 로컬 Godot 로그는 포함하지 않는다. 원래51단계 전체 연결·workspace 테스트 세계의 완전한 열거·독립 제품 및 관찰자 중립성 증명은 아직 미완료다.

선택 콘텐츠 팩 UI의 실제 run01은 bridge 빌드와 원래 headless 실행을 exit0으로 마쳤다. 보고서41항목 중39개 단언이 통과했고 렌더링은 NOT_RUN_HEADLESS다. 한 연속 세계의 실제 tick0→1, 창립자/생존자10명·KnowledgeLost0 및 전체5체크포인트를 검증했다. 소스1471개·runtime 입력6419개와 실행 파일·HEAD·작업 상태는 전후 같았다(`stage0/ui-pack-native/RESULT.md`). Rust 기제 검사·양쪽 제품 비교·장기 생존·전체 gate를 대신하지 않는다.

리뷰 커밋 `886ff0e5ee58fadda44fc653d962c3f5d6697d74`의 native 파일902개는 foundation03 입력 바이트와 일치했다. 나머지 입력 `rust/Cargo.lock`은 기존 ignore 규칙에 의해 Git 밖에 있으며, 당시와 현재 해시가 같다. 따라서 커밋 SHA만으로 의존성 잠금 파일의 출처까지 증명하지 않는다. 독립 빌드는 이 파일의 바이트와 출처를 별도 입력으로 고정해야 한다(`stage0/foundation-lockfile-qualification.json`).

후속 smoke run01은 입력 목록에서 기존 `smoke.sh`·`smoke_base_counts.txt`가 빠져 세계 실행 전에 FAIL했고, 네 실행은 NOT_RUN으로 남겼다. 목록을 수리한 새 run02는 원래 두900프레임 검사와 각 동일 제품 companion을 모두 exit0으로 마쳤다. boot tick80·labels tick82가 각각 일치했고 모든 세계 생존10명·KnowledgeLost0, 오류·raw key0이었다. 소스·설정 복원·입출력 해시의 최종 검증도 통과했다(`stage0/ui-smoke-native/run-02/README.md`). 독립 BASE/후보 비교나 전체 gate는 아니다.

테스트 실행 식별 연결은 기존 네 가족의 기본 호출을 유지하며 debug/release·step17·step24를 서로 다른 실행으로 묶는다. 같은 명령행이라도 기록된 실행 환경의 식별자가 다르면 재사용을 거절한다. 기본 `coverage` 문자열의 호환성을 복원한 회귀와 실제 빌드 프로필·원래 스레드 수 검증을 포함한 집중10개가 통과했다. 모든 workspace/step17/37 검사를 구현한 것은 아니다. 별도 Python 조정기는 원래51단계의 필수 목록을 고정하고 현재12개 구성요소를 호출하지만, 빠진 의무 때문에 항상 INCOMPLETE 또는 FAIL이며 게시 승인을 내리지 않는다. 독립 검토에서 찾은 반환 상태/파일 상태 불일치, 실패 출력의 변경 누락, 실행 순서 표기 문제를 수리한14개 검사가 통과했다. smoke의 세 경계 회귀를 포함한15개와 전체 생산 모듈 엄격 타입 검사13·Ruff 후속 검사도 통과했다. 셸 gate의 실행 경로는 아직 이 조정기로 전환하지 않았다.

추가 검토에서 원래 Cargo가 테스트 바이너리를 패키지 디렉터리에서 실행함을 확인했다. 이름 붙인 gate04/05/17/24 호출은 이를 따르도록 수정하고 작업 디렉터리가 다른 영수증을 거절한다. 기존 development 호출은 이전 디렉터리를 유지하며 원래 Cargo 실행의 자격 증명으로 사용하지 않는다고 명시했다. 별도 workspace 구조 파서는 패키지·target 종류·프로필·실행 파일 위치를 대조한다. 실제 pinned Cargo 메타데이터의5패키지·156target 중154개 test 선택과5개 doctest 의무를 읽었지만, 이는 컴파일·실행·세계 검사 수가 아니다. 활성 bench/example/staticlib 및 비활성·조건부 target의 회귀9개가 통과했고, 전체 세계 범위와 Cargo 실행 환경·doctest 실행의 증명은 미완료로 유지한다.

통합 Python11은 generic·요약·READ·정규화569개 검사를214.780초에 통과했고 실행 전후 Python 소스 해시는 같았다. 전체 생산 모듈의 엄격 타입 검사14는 오류0·경고0, Ruff06도 통과했다. 이는 원래 gate 전체의 Python 단계나 native workspace 전체 실행을 대신하지 않는다.

독립 관찰자 검증 run02는 공개 BASE0 `2f6993009ddb600e9249c28f9768a1a5252c8821`에서 새로 만든 두 제품을 사용했다. 한 제품에는 검토한 probe 관찰 경로7개만 추가했고, 하네스는 리뷰 커밋 `886ff0e5ee58fadda44fc653d962c3f5d6697d74`였다. 원본 비활성·파생 비활성·파생 활성의9개 프로세스, 총51세계 실행에서 baseline120일·U10·CORE-off20일의 여섯 비교가 모두 일치했다. 활성 관찰 이력17개와 원래 출력의 연결도 통과했다. 사전 고정한 timing12경로 템플릿(실제 확장 경로 보존)·colony 측정2필드 및 원시 요약과 대조한 Markdown 시간 행만 측정값으로 분리했고, 알 수 없는 시뮬레이션 키는 제거하지 않았다(`stage0/observer-qualification/RUN02_SUMMARY.md`, 원시 결과 SHA256 `e3a42bdbe512b1ee73063a5122bbb7ecb0b8271999aba35ddc9ed9e9fb4c5406`).

같은 실행의 ordinary safety는 **FAIL**이다. 세 CORE-off 원시 요약은 바이트가 같고 종료0명과 tick28560의 `clay_digging` 상실을 동일하게 기록했다. 창립자10명은 파생 활성 제품의 sidecar에서 관측했으며 원래 colony 요약에는 초기 인원 필드가 없다. runtime의 git_sha는 원래 명시 환경에서 `unknown`이며 제품 출처는 별도 source/build·실행 파일 해시로 인증했다. 실제 전체 명령 순서는 NOT_AVAILABLE이고 명령·수락 집계만 비교했다. 이 중립성 증거는 CLI로 활성화한 probe 경로와 위 세 fixture에 한정한다. 전체 Stage0 자기 비교·UI·저장·mirror 모드·성능·게시 자격으로 확대하지 않는다. 첫 run01의 실행 전 파서 실패도 보존한다.

workspace 빌드 연결부는 패키지·target·feature 축소와 배타적인 명시 모드를 추가했다. 실제 Cargo metadata와 새 workspace no-run 빌드의 프로세스·입력·출력을 묶고 패키지별 target과 실행 파일을 대조하며, 이전 metadata/version 증거가 변경되면 거절한다. 독립 검토와 루트의 기존/신규 회귀78개가 통과했다. 이 검사는 작은 합성 compiler를 사용했으며 새 모드의 실제 Cargo 빌드·테스트 실행·doctest·전체 세계 안전성은 아직 NOT_RUN 또는 INCOMPLETE다.

과거 버전 실행기에는 기본 비활성인 자식 프로세스 기록 경로를 추가했다. 원래 명령행·디렉터리·실행 파일·입력 해시를 사전에 고정하고 실제 PID·환경·원시 종료·출력 해시를 기록한다. 독립 검토에서 발견한 bounded 자식의 표준입력 EOF, 시그널 종료 코드, 타임아웃 문구 차이를 재현한 뒤 원래 래퍼와 맞췄다. 루트 회귀15개가 통과했으며 기본 실행 분기는 그대로다. 기록 경로는 출력을 종료 후 재생하므로 성능 증거가 아니다. 원래 여섯 launcher 경우의 실제 native 실행·과거 버전 빌드 출처·관찰자 자격은 아직 미완료다.

통합 Python12는623개를207.845초에 통과했고 검사 대상 Python 입력은 전후 동일했다. 생산 generic 모듈과 새 launcher helper/검사의 엄격 타입 검사15는 오류0·경고0이며 변경 파일 Ruff와 shell 구문 검사도 통과했다. 이는 원래 gate16 전체 discovery나 실제 workspace native 실행을 대신하지 않는다.

정적 단계 연결부는 원래 단계1·7·13·15·20·22·31·33의 명령을 그대로 보존한다. 명시 환경·실행 도구·전체 유효 입력과 단계15/33의 과거 Git 참조를 고정하고, 앞서 성공하거나 실패한 결과 파일도 종료 시 다시 검사한다. 독립 리뷰와 루트 합성 회귀13개가 통과했다. 실제 여덟 명령 실행과 Python/Pillow/toolchain 의존성 전체의 인증은 아직 미완료이며, 이 연결부가 전체 gate 순서나 소스 승인을 부여하지 않는다. 원래 문서 단계14는 실제 colony 엔진을 생성하므로 `--skip-system-table` 결과를 전체 단계14 통과로 바꾸어 기록하지 않는다.

Cargo 자식 기록기는 실제 전달된 명령행·작업 디렉터리·환경과 부모/기록기/자식 PID를 남기고, 두 출력 스트림을 제한된 버퍼로 동시에 전달한다. 리뷰에서 재현한 시그널 중복 전달과 예외 시 자식·출력 스레드 정리 누락을 고쳤으며, 루트 합성 프로세스 검사17개가 통과했다. 원래 자식 종료와 관찰 결과를 구분하고 기록이 완결되지 않으면 pending 증거를 유지한다. 아직 pinned Cargo의 실제 runner 호출, 전체 테스트 결과 집계, doctest 또는 세계 안전성의 증거는 아니다.

리뷰 커밋 `6ed44f4cc1d3512fcef75af10649bca8a529a501`의 독립 pristine 제품에서 새 debug workspace no-run 빌드를 실제 실행했다. Cargo metadata와 compiler가 모두 exit0이었고, 선택된 테스트 실행 파일154개를 패키지별로 인증했다. 소스·도구·설정·입출력의 최종 재검증도 통과했다(`stage0/workspace-build-native/TERMINAL_SUMMARY.json`, 결과 SHA256 `f2aec9199bb2d1ae49981ec3520cff91568d2439d14744f922618a438df17db4`). Cargo.lock은 별도 고정 입력이고 커밋된 파일로 주장하지 않는다. 실제 테스트 실행과 doctest5개 대상은 NOT_RUN, 세계 안전성은 INCOMPLETE다. 전체 workspace 결과 집계는 필요하지만 모든 기제 단위 검사의 엔진 생성자를 생존 fixture로 바꾸지 않는 기존 범위 교정을 유지한다.

stall 결과 연결부는 원래 타이밍 계산과 pristine 캡처 인증을 재사용한다. strict 통과는 native0, 완전한 waiver 후보는 원래 FAIL/native1과 정확한 실패 진단을 유지한다. 원래 설정 디렉터리·profile 환경과 직접 Godot 프로세스를 별도로 묶고, 실패한 캡처의 파일도 전체 해시 목록에 포함한다. 독립 리뷰와 합성 검사11개가 통과했다. 이 순수 검증기는 NONE 프로토콜만 지원하고 안전성은 PENDING, 게시 권한은 false로 고정한다. 실제 자식/부모 연결·lane·관찰자 제품 출처·동일 종료 시점의 생존 비교가 없으면 원래 단계41의 승인으로 사용할 수 없다.

통합 Python13은664개를476.065초에 통과했고 실행 전후 입력 해시는 같았다. 이후 새 정적 검사 테스트의 all-mode 타입 오류29개를 수리했다. 기존13개 시나리오·단언을 유지한 테스트 전용 변경이며, 독립 리뷰와 수정 후 루트13개 재검사가 통과했다. 통합 실행 이후 변경된 Python 파일은 그 테스트 한 개뿐이다. 최종 생산 모듈·새 테스트의 엄격 타입 검사17은 오류0·경고0, Ruff08도 통과했다. 이전 타입 실패 로그는 보존하며, 이 기록은 전체 native gate 통과가 아니다.

단계별 direct 실행은 전체 registry·변경 선언을 먼저 검증한 뒤 원래 단계의 완전한 fixture 묶음만 선택한다. 기존 전체 개발 실행 API는 유지하며, 단계50은 여섯 fixture를 각각 BASE→후보 순서로 새로 실행한다. 독립 검토에서 발견한 순서 오류와 실패 이후 후속 채점 실행을 고쳤다. 첫 producer 실패 뒤 나머지 producer·채점은 NOT_RUN으로 남기며 앞선 원시 결과는 재검증한다. replay도 같은 군의 생성→기록 재생→스크립트 재생과 원래 실패 의존성을 지킨다. F10 쌍은 분리하지 않고 공동 M6a 실행만 있는 단계47/48은 미구현 상태를 유지한다. 루트 집중52개와 기존 조정기 호환 검사가 통과했고 엄격 타입 검사18은 오류0·경고0, 변경 파일 Ruff도 통과했다. 전체51단계 실행·게시 권한은 여전히 없다(`stage0/ordered-dispatch/direct-draft/independent-review-v3.md`).

작은 실제 Cargo runner 검증 run01은 테스트 실행 전 no-run 빌드에서 FAIL했다. 별도 읽기 전용 fixture의 Cargo.lock에 Cargo 표준 생성 주석 두 줄이 없어 재기록하려다 권한 오류가 발생했다. 원래 직접/기록 실행과 파일 디스크립터 동등성 검증은 NOT_RUN이며 실패 입력·원시 종료·부분 산출물을 보존했다. 고정된 Cargo 소스에 근거해 주석만 추가한 v3는 오프라인10개와 독립 검토를 통과했으나 실제 복구 실행 결과로 간주하지 않는다(`stage0/workspace-runtime/cargo-hook-qualification/debug-native-run-01/RESULT.json`).

원래 단계16의 `python_unittest`는 기존 정적 명령 실행기를 확장해 연결했다. 전체 `test_*.py` discovery 명령을 유지하고 문서 입력, 실제 셸·Perl 도구, 두 아트 원본 Git 참조와 명시적 시작 환경을 검증한다. 실행된 unittest의 종료 요약에서 검사 수·skip·expected failure·failure·error를 읽으며 소스 선언 수를 실행 수로 쓰지 않는다. 기존 여덟 단계의 경로는 유지했다. 독립 리뷰와 루트 합성19개, 엄격 타입 검사19 및 Ruff가 통과했다. 새 Perl 의존성 목록의 실제 수집·동결과 원래 전체 discovery 실행은 아직 NOT_RUN이며, 기존 정적8개 증거를 단계16 완료로 재사용하지 않는다(`stage0/ordered-dispatch/step16-draft/`).

리뷰 커밋 `7b360944ff49eb763c4b413ee2d57e7e752e6b6d`의 독립 pristine 제품·하네스에서 원래 정적8개 명령을 실제 실행해 모두 exit0으로 마쳤다. 단계33의 원래 세 모듈 unittest는29개를6.435초에 통과했고 ResourceWarning 두 건을 원시 stderr에 보존했다. 독립 감사는 여덟 영수증 연결, 프로세스 산출물40개, 두 소스의 tracked3037개·유효 입력2738개를 재검증했다. 최종 결과는 PASS_COMPONENTS_ONLY이며 원래 전체 gate 순서·ordinary safety·게시 승인은 아니다(`stage0/static-qualification/INDEPENDENT_RUN01_AUDIT.md`, 결과 SHA256 `0e9d6f8b440627e92aada0c8e971f73ffff5ec20ffadb88158f837a85ac2a043`). 이후 리뷰 커밋에 소급 적용하지 않는다.

전체 단계 조정기에 원래 순서의 지원 가능한 부분을 연결했다. 공유 direct 단계 목록, 정적8개, smoke와 지원 UI를 각 위치에서 실행하고, 앞선 실패·산출물 변경·lane 종료 오류를 보존한다. 단계50 결과는 새 실행으로 만들며 단계34/46의 실패 뒤 후속 형제 실행은 멈춘다. 단계16·전체 workspace·성능·stall·분리되지 않은 M6a 등 미구현 의무는 남기고 결과는 FAIL 또는 INCOMPLETE만 가능하다. 루트 합성27개, 엄격 타입 검사20, Ruff 및 독립 리뷰가 통과했다. 실제 순서 실행·소스 빌드 자격·전체 gate CLI 전환은 아직 하지 않았다(`stage0/ordered-dispatch/coordinator-draft/independent-review-v1.md`).

작은 Cargo 검증 run02도 FAIL로 보존했다. v3 잠금 파일로 no-run 빌드는 통과했고 원래 직접/기록 Cargo 명령을 모두 실행했으며 계획된 네 native 파일은 COMPLETE 영수증을 남겼다. 그러나 실제 host runner는 rustdoc의 다섯 번째 임시 doctest 실행 파일에도 적용되었다. 기록기가 비정규 경로·미등록 파일을 거절해 직접 실행에서 통과한 doctest가 기록 실행에서 실패했다. 실제 관찰이 과거의 별도 doctest hook 가정을 반박하므로, 해당 실행을 제외하거나 성공으로 인정하지 않는다. 전체 환경·파일 디스크립터 동등성 판정은 아직 완료하지 못했다(`stage0/workspace-runtime/cargo-hook-qualification/debug-native-run-02/RESULT.json`, SHA256 `5c4df09a56d7a36257bf50d74b58dfba472aedb200b0d1ece62042341f681679`).

Cargo 기록의 순수 대조기는 원래 부모 명령·manifest·패키지별 실행 파일·순번 claim·시작/종료 기록과 출력 해시를 연결한다. 프로세스를 새로 실행하지 않고, 비정상 종료·부분 기록·미등록 doctest를 누락시키지 않는다. 기록 완결성과 native 성공을 분리하며 PID 재사용을 전역 중복 오류로 취급하지 않는다. 독립 리뷰와 루트 합성10개·엄격 타입 검사21이 통과했다. 루트 Ruff에서 발견한 테스트 import 순서 한 건을 고친 뒤10개 재검사와 Ruff도 통과했다. 실제 빌드 출처 승인, Cargo 환경·파일 디스크립터 동등성, 개별 testcase·doctest 집계와 세계 안전성은 여전히 미완료다(`stage0/workspace-runtime/INDEPENDENT_CAPTURE_V1_REVIEW.md`).

stall의 기존 960초 wrapper에 기본 비활성인 직접 자식 기록 경로를 연결했다. 원래 stdin EOF·병합 실시간 출력·명령을 유지하고, 시작 승인 파이프와 부모 연결 감시로 기록 실패·중단 시 소유한 자식만 종료·회수한다. 실제 부모/자식 환경, 입력 해시, 출력 바이트 구간과 시간·종료 기록을 대조하며 일반 프로세스 영수증으로 위장하지 않는다. 루트 검사에서 임시 제품에 helper 의존 모듈이 없어3개가 실패한 기록을 보존하고, 필요한5개 모듈을 복사하며 PYTHONPATH를 제거한 뒤13개를 통과했다. 결과 검증12개·기존 명령9개·play child15개와 엄격 타입 검사22도 통과했다. 기존 mux2_gate_receipts의 Ruff3건 및 타입33건은 이전 소스와 동일하며 새 오류로 숨기거나 전체 lint 통과로 기록하지 않는다. 독립 검토는 자식 회수·환경 연결·영수증 검증을 승인했지만 실제 Godot stall, 양쪽 세계 안전성 및 관찰 비용은 아직 미검증이다(`stage0/stall-run/CHILD_CHAIN_INDEPENDENT_REVIEW_V2.md`).

고정 리뷰 커밋 `09c0a3c5020f11962d1e0bbbfef15e5262f33269`의 독립 pristine 제품·하네스에서 단계16 원래 전체 discovery를 실행했다. 실제1,199개가571.933초에 OK·exit0으로 끝났고 failure/error/skip/expected failure/unexpected success는0이었다. ResourceWarning 두 건과 거절 테스트 진단은 원시 stderr에 보존했다. 소스·런타임·참조·산출물 최종 검사 및 driver 종료0을 확인했고 독립 감사가 정확한 명령·환경·작업 디렉터리와 영수증 연결을 검증했다. 결과는 PASS_COMPONENTS_ONLY이며 이후 커밋이나 전체 gate·세계 안전성·게시 승인으로 확장하지 않는다(`stage0/python-discovery-qualification/INDEPENDENT_RUN01_AUDIT.md`, 결과 SHA256 `672bfe065392ddaf5d844b25fa3a9c24f701ca19dd353adc68091662ad134b00`).

M6a 실행기는 기존 공동 API를 보존하면서 단계47의 네72일 primary·두 scorer와 단계48의 두 selected primary·실제 후보 종료 tick에 맞춘 새 BASE companion을 분리했다. 전체 fixture 정의·정확한 군/seed/순서와 fresh 출력 경로를 검증하고, 실패 산출물 및 scorer/companion의 첫 해시를 마지막까지 대조한다. 독립 검토와 루트20개 검사를 통과했다. direct/coordinator 연결과 실제 native qualification은 미완료이며, 단계48의 별도 식물 구조 렌더러 검사도 아직 필수 미구현 의무로 남는다(`stage0/ordered-dispatch/m6a-split-draft/independent-review-v1.md`).

Cargo가 임시 doctest 파일에도 runner를 적용한 실제 run02 실패에 대응해, 선택적으로 같은 PID의 rustdoc exec-intent와 실제 직계 부모 실행 파일을 연결하는 기록 경로를 추가했다. 원래 임시 파일 경로를 실행하고 정규 경로·파일 식별·보존 복사본을 별도 기록한다. 검토에서 재현한 joined dependency 인자 우회와 검증 도중 intent 변경 수락을 수리했으며, 지원하지 않는 인자 문법은 실행 인자를 고치지 않고 거절한다. 루트33개 합성 검사·엄격 타입 검사23·Ruff 및 독립 검토가 통과했다. 실제 Cargo/rustdoc 호출 인자 고정·live probe·환경/FD 동등성·최종 부모 영수증 연결·doctest 집계는 아직 미검증이다. 원래 run02 FAIL과 미등록 transient를 거절하는 workspace 대조기의 제한은 유지한다(`stage0/workspace-runtime/INDEPENDENT_DOCTEST_V2_REVIEW.md`).

분리한 M6a47/48을 direct와 원래 순서 조정기에 각각 연결했다. 선택 전에 전체 registry·선언을 검증하고, 단계·fixture·scorer·companion의 산출물 해시를 정확히 묶어 다른 단계 영수증 재사용을 거절한다. 단계48의 `m6a.plant_art_structural`은 별도 필수 미구현 의무로 명시했다. 연결 전 조정기 회귀3개 실패를 보존한 뒤 direct33개·조정기27개가 통과했으며 엄격 타입·Ruff도 통과했다. 이 결과는 오프라인 연결 검증이고 전체 gate 결과는 여전히 INCOMPLETE/FAIL만 가능하다. 실제 분리 실행 및 구조 렌더러 검증은 미완료다(`stage0/ordered-dispatch/m6a-direct-draft/independent-review-v1.md`).

관찰자 overlay 소스의 읽기 전용 대조기를 추가했다. 원래 commit tree와 현재 index, 외부에서 고정한 manifest의 pre/postimage, 허용 경로 및 실제 작업 파일을 대조하고 입력을 다시 확인한다. patch를 적용·되돌리거나 index를 변경하지 않는다. 동일 경로의 다른 hunk가 있어도 이 좁은 대조만으로 patch 적용을 증명할 수 없음을 회귀로 고정하고 `patch_application_provenance=NOT_VERIFIED`를 명시했다. 빌드·runtime은 PENDING, 관찰자 의미 검증은 NOT_RUN이다. 독립 검토와11개 작은 Git 저장소 검사·엄격 타입·Ruff를 통과했으며 pristine capture의 clean 요구나 실제 stall 실행 경로는 바꾸지 않았다(`stage0/stall-run/source-draft/INDEPENDENT_REVIEW_V2.md`).

원래 Python 전체 discovery를 조정기의 단계16에 연결했다. 기존 정적8개와 문서·추가 runtime 입력이 필요한 discovery의 환경을 분리하고, 첫 작업 전부터 원래 도구/PATH/시작 정책을 검증한다. 실제 프로세스·raw unittest 요약·시작/spawn/완료 기록과 이전 FAIL 영수증을 대조하며, null/0 PID나0 duration을 PASS로 수락했던 회귀도 거절하도록 고쳤다. 루트 조정기43개·정적19개, 엄격 타입 검사25·Ruff·문서 검사를 통과했다. p7의 과거1,199개 결과를 이 변경의 실행 증거로 재사용하지 않으며, 실제 순서 실행·완전한 runtime admission·전체 gate는 여전히 미완료다(`stage0/ordered-dispatch/step16-coordinator-draft/independent-review-v2.md`, `independent-root-duration-review.md`).

작은 Cargo 실제 run03은 수정된 실행기의 사전 검증 후 수행했지만 FAIL했다. 직접 실행의 doctest는 통과했고 두 Cargo 부모는 각각101로 끝났으나, 기록 실행은 네 static 실행 파일이 소스 입력에도 포함되어 기존 경계 검증에 거절됐다. rustdoc 인자 배열은 선언과 일치했지만 실제 compiler 작업 디렉터리는 workspace 루트이고 선언은 패키지 디렉터리여서 역시 거절됐다. 후처리의 `wrapped-snapshots: empty or duplicate input scope` 오류와 원시 process 종료101·phase STARTED 표기의 차이도 별도로 보존한다. 모든 소유 프로세스14개의 종료와 최종 입력·산출물 무결성을 확인했으며, 같은 외부 종료 코드만으로 동등성을 인정하지 않는다(`stage0/workspace-runtime/cargo-hook-qualification/debug-native-run-03/RESULT.json`, SHA256 `b6cf7359280f5f305059afcd04254731c1708d9e43e1610deff9832e6ed69834`).

M6a 단계48의 원래 식물 구조 검사를 실행하는 별도 어댑터를 추가했다. 원래 windowed 명령·통합 출력·검증기를 유지하고 실제로 import한 조정기 소스와 Godot extension/class/UID/import 경로를 고정 입력에 대조한다. 독립 검토에서 재현한 다른 검증기 import와 미등록 extension 수락은 모두 거절되며, 루트12개 검사(12.594초)·엄격 타입 검사26·Ruff를 통과했다. Godot PID는 기록되지 않아 null이고 시각 평가는 NOT_EVALUATED, 실제 renderer 실행은 NOT_RUN이다. 자식 종료가 불명확한 경우에는 INCOMPLETE로 남기고 후속 군 실행·최종 inventory를 하지 않는다. 기존 조정기는 이 상태에서의 소유 프로세스·lane 유지가 아직 해결되지 않아 연결하지 않았다. 독립 검토는 이 제한된 컴포넌트만 승인했다(`stage0/ordered-dispatch/M6A_STRUCTURAL_INDEPENDENT_REVIEW_V2.md`).

Cargo 기록 정책v2는 컴파일 산출물을 소스 입력과 분리하고 전체 dependency 디렉터리의 파일·하위 디렉터리·해시를 대조한다. 실행하지 않은 참조 산출물도 각 자식의 시작·종료에 확인하며, rustdoc compiler cwd와 native run cwd를 별도로 검증·기록한다. v1/default 경로는 유지했다. 독립 검토에서 발견한 잘못된 정책의 종료 기록 누락, 중복 joined run-directory 인자 수락, 저장된 admission의 cwd 누락을 회귀와 함께 고쳤다. 루트24+19개 검사·엄격 타입·Ruff 및 독립43개 검사를 통과했다. 실제 compiler/metadata 출처 대조와 마지막 외부 closure 검사는 새 실행기에서 연결해야 하며, 실제 재실행·전체 workspace 검증은 NOT_RUN이다. run03 FAIL을 변경하지 않는다(`stage0/workspace-runtime/doctest-policy-v2-draft/INDEPENDENT_REVIEW_REVISION2.md`).

단계51의 커밋된 진단 결과 검사를 정적 어댑터에 추가했다. 원래 Bash/Python 본문과 truthiness·첫 Q1/Q5 선택·5개 이상 통과 조건을 그대로 유지하고, 해당 JSON만 별도 입력 범위로 포함한다. 이는 과거 결과 파일 검사이며 새 UX·시각·월드 평가가 아니다. 독립 검토에서 BASH_ENV가 Python assertion을 꺼 0/6도 통과시키는 우회를 재현했고, 단계51만 허용 환경 키를 제한해 실행 전에 거절하도록 수리했다. 루트28개 검사(1.939초)·엄격 타입·Ruff와 독립28개 검사가 통과했다. 정적8개와 전체 discovery의 범위는 유지하며 원래 순서 조정기 연결·커밋 소스 실제 실행·전체 gate는 아직 완료하지 않았다(`stage0/ordered-dispatch/STEP51_INDEPENDENT_REVIEW_V2.md`).

단계51을 원래 순서의 마지막 정적 호출로 연결했다. 정적8개·전체 discovery·진단 JSON의 입력과 환경은 세 context로 분리하고, 시작 전·각 경계에서 같은 후보 소스와 정확한 범위를 확인한다. 세 환경의 이전 PASS/FAIL 영수증을 모두 전달하며, 단계16/51의 공통 프로세스 대조는 실제 Python/Bash 시작·spawn·종료·출력 해시에 결합했다. 실패한 실행에 PASS marker나 가짜 unittest 요약을 요구하지 않는다. 루트 조정기50개(9.784초)와 앞선 동일 정적 어댑터28개, 독립 통합78개·엄격 타입·Ruff를 통과했다. 실제 전체 순서 실행과 full gate는 미완료이며 알려진 BASE COREoff 지식 손실 FAIL도 유지한다(`stage0/ordered-dispatch/STEP51_COORDINATOR_INDEPENDENT_REVIEW_V1.md`).

Stall 관찰자 patch의 실제 변환과 공급된 빌드 증거 대조를 분리했다. 외부의 새 scratch에 고정한 원래 blob을 펼쳐 Git patch를 적용하고 정확한 postimage를 확인하며, 제품 소스·index는 바꾸지 않는다. debug sim-bridge의 도구·프로세스·compiler package/target/source·artifact를 대조하되 과거 실행 권한은 CALLER_DRIVER_EVIDENCE_REQUIRED, runtime은 PENDING으로 남긴다. 독립 검토의 외부 패키지 artifact·0 duration 수락, 추가 무결성 오류가 있는 예상 Git128을 로컬 변환 PASS로 처리한 문제를 회귀와 함께 고쳤다. 루트16개 검사(26.835초)·엄격 타입 검사27·Ruff와 독립 검토를 통과했다. 실제 빌드·stall capture 연결·성능/안전 admission은 아직 없다(`stage0/stall-run/build-draft/INDEPENDENT_REVIEW_V3.md`).

실제 Cargo run04도 FAIL로 보존한다. 네 일반 자식은 COMPLETE와 원래 종료0/7/0/0을 기록했으나, alpha doctest는 직접 실행 PASS와 달리 기록 실행에서125로 실패했다. 실제 요청은 `/var/folders/.../T/rustdoctestGIBDro/rust_out`, 선언한 임시 루트는 Python이 선택한 `/private/tmp`여서 생성 파일을 실행하기 전에 차단됐다. pinned Rust의 Darwin 사용자 임시 디렉터리 선택과 Python 기본값의 차이를 확인했으며 환경 변경이나 허용 경로 확대 없이 실행기 선언을 수리할 예정이다. 산출물592개·바이너리4개·sealed 파일72개를 재검증하고 기록된 자식/launcher PID18개의 종료 및 외부 실행기 exit1을 확인했다. 원시 Cargo101 두 개만으로 동등성을 인정하지 않는다(`stage0/workspace-runtime/cargo-hook-qualification/INDEPENDENT_RUN04_AUDIT.md`, RESULT SHA256 `01e42405a78bd2e833404d0926155a8903458b26905983f79b054d8a602d707d`).

실제 Cargo run05는 작은 debug 검증 프로젝트에 한해 PASS다. pinned Rust의 Darwin `confstr` 임시 디렉터리 선택을 실행기 선언에 반영했으며 TMPDIR 주입이나 helper·원래 테스트 명령 변경은 없었다. 일반 자식4개는 원래 종료0/7/0/0, 임시 doctest 자식1개는 종료0으로 모두 COMPLETE였고 직접/기록 실행의 alpha doctest PASS·beta 0개 및 의도한 Cargo101이 일치했다. 임시 루트·의존성 최종 검사와 독립 감사도 통과했다. 산출물599개·native 바이너리4개·sealed 입력88개의 해시를 재검증했고 기록된 PID19개 부재와 외부 실행기 exit0을 확인했다. 실제 jobserver FD 관측은 양쪽 모두 빈 목록이므로 활성 FD 상속 증명은 아니다. workspace 전체 집계는 INCOMPLETE, release·반복 실행은 NOT_RUN이며 세계 실행·전체 gate·게시 승인을 뜻하지 않는다. run01–04 실패는 그대로 보존한다(`stage0/workspace-runtime/cargo-hook-qualification/INDEPENDENT_RUN05_AUDIT.md`, RESULT SHA256 `060b193bc6d8970014132abadedd9f3805dcda59efe8e92d79c87bf7cff16d23`).

원래 단계24의 저장·복원 검사를 전체 단계 조정기에 연결했다. 각 군의 release `save_load` 제품·실제 실행 기록·원래7개 검사·46개 저장 이력과 쌍별 안전성을 다시 대조하고, 실행기가 단독으로 lane을 소유하도록 중복 잠금을 피한다. 실패 산출물과 실행되지 않은 후보를 보존하며 전체 gate 상태는 여전히 FAIL/INCOMPLETE다. 독립 리뷰에서 찾은 순차 PID 재사용 오검출은 각 실행의 별도 인증 기록으로 식별하도록 고쳤다. 루트 통합60개 검사가56.232초에 통과했고 엄격 타입 검사·Ruff도 통과했다. 실제 양쪽 release 실행 및 원래 셸/Cargo 준비 순서의 증명은 아직 미완료이며, 기존 빌드 준비와 직접 native 실행의 분리 연결을 전체 셸 실행으로 주장하지 않는다(`stage0/ordered-dispatch/GATE24_INDEPENDENT_IMPLEMENTATION_V2_REVIEW.md`).

구조 렌더러용 자식 프로세스 소유 helper를 기본 비활성 경로로 추가했다. 원래180초 명령을 유지하며 부모가 실제 wrapper PID를 기록한 뒤 실행을 허용하고, 실제 native PID·원시 종료·합쳐진 로그 범위를 연결한다. 이미 관측 가능한 제어 파이프 단절은 실행 전에 거절하며, 이후 단절과 실행의 경쟁은 기존 supervisor가 종료·회수한다. 종료 증거가 불완전하면 UNKNOWN을 반환하며 lane 해제나 완료를 주장하지 않는다. 루트의 새 helper 및 기존 stall/play 호환39개 검사가2.508초에 통과했고 타입·Ruff·셸 구문 및 독립 검토도 통과했다. timeout 회귀는 합성 복사본의 대기 한도를0으로 주입한 회수 검사이며 실제180초 또는 Godot 실행 증거가 아니다. 실제 렌더러·suite·전체 조정기 연결과 UNKNOWN 상태의 lane 소유·복구는 아직 미완료다(`stage0/ordered-dispatch/ORACLE_CHILD_INDEPENDENT_REVIEW_V1.md`).

workspace 기록 대조기는 기존 schema1 일반 자식 경로를 유지하고 policy2 Rustdoc 실행 의도와 임시 자식을 별도로 연결한다. 실행되지 않은 doctest 의무·부분 기록·알 수 없는 기록도 남기며, 사라진 임시 실행 파일을 다시 조사하거나 재생하지 않는다. 루트 검토에서 재현한 다른 UID 및 부모와 다른 자식 환경의 허위 통과를 수정했다. 동일 사용자/runtime의 보존 소유자와 인증된 Rustdoc 실행 환경을 정확히 대조하며, 이 한계를 다른 사용자 환경으로 옮길 수 있는 증명으로 확대하지 않는다. 최종 루트 합성26개가1.269초에 통과했고 엄격 타입·Ruff도 통과했다. 처음 통합 시 테스트 import 순서 Ruff 오류1개를 고친 뒤 재검사했으며 이전 로그를 보존했다. 전체 테스트 사례·doctest·세계 안전성은 여전히 INCOMPLETE이고 실제 workspace 실행은 아직 NOT_RUN이다(`stage0/workspace-capture-v2/v2/HANDOFF.md`, 원래 오검출 증거 `stage0/workspace-capture-v2/owner-review/RED.log`).

perf/nav 실행기의 보존 결과를 검증하는 source-owned 함수를 추가했다. 실제 계획된 세계·프로세스·sidecar·부하 승인 기록·원래 요약기와 안전성 결과를 다시 연결하며 실행 순서·명령·잠금·시간 기준은 바꾸지 않는다. 독립 검토에서 완료된 자식 산출물을 목록에서 누락해도 PENDING으로 인정하던 결함을 재현하고, 보존 산출물과 완료 prefix를 대조하도록 고쳤다. 최종 루트 합성19개가1.928초에 통과했고 엄격 타입·Ruff 및 독립 v2 검토도 통과했다. 실제 성능 측정·관찰자 비용·원래 빌드 순서·전체 조정기12/27 연결은 아직 미완료이며, COMPONENT_PASS/FAIL/PENDING 원시 상태를 바꾸지 않는다(`stage0/ordered-dispatch/PERF_AUTH_INDEPENDENT_REVIEW_V2.md`).

원래 단계12·27의 perf/nav 실행을 전체 순서 조정기에 연결했다. 각 군의 별도 제품·입력·빌드 영수증과 nav 스크립트, 유한 deadline·목적을 명시적으로 대조하고 기존 실행기만 lane을 소유한다. 보존 결과를 source-owned 검증기로 재확인하며 COMPONENT_PASS/FAIL/PENDING을 그대로 유지한다. 루트 조정기65개가57.984초에 통과했고 엄격 타입·Ruff 및 독립 검토도 통과했다. 이후 strict 어댑터와의 호환65개도56.498초에 통과했다. 원래 빌드 준비 순서, 실제 측정, 관찰자 비용, 동적 companion·ledger 및 전체 gate 승인은 아직 미완료다(`stage0/ordered-dispatch/PERF_COORDINATOR_INDEPENDENT_V1_REVIEW.md`).

Stall 파생 capture는 실제 빌드 증거 참조·원래 bounded 자식 연결·처음 보존한 원시 산출물을 검증한다. 호출자가 이미 소유한 lane을 요구하고 종료 불명확 상태에서 해제하지 않는다. 독립 검토와 루트 재현으로 발견한 검증 중단·예상 밖 예외의 허위 VERIFIED를 수정했으며, 검증 완료 조건과 오류 기록 후 재전파를 분리했다. 원래 strict 종료1은 잠정 진단으로만 남고 PASS·waiver가 아니다. 루트 capture37개(11.611초)와 stall53개(489.663초), 설정된 엄격 타입 범위 검사를 통과했다. 전체 변경 파일의 기존 타입 진단은206→205로 신규0이며 전체 clean을 뜻하지 않는다. Ruff의 기존 EXE001 두 건은 유지했고 파일 끝 빈 줄1개만 정리했다. 실제 빌드 실행 권한·native capture·관찰자 중립성·안전성·전체 gate는 아직 미검증이다(`stage0/stall-run/DERIVED_CAPTURE_SLICE_B_INDEPENDENT_V4_REVIEW.md`).

원래 GDScript strict 단계8·46의 별도 어댑터를 추가했다. 단계8은 원래 전체 scripts 선택을, 단계46은 원래4개 파일과 전체 FFI 검사를 유지하며 준비된 Godot 입력·명시적 실행 파일·호출자 소유 lane을 요구한다. 실행이나 진단 기록 중 예외·중단은 StrictOwnershipUnknown으로 남아 최종 산출물 고정과 정상 실패 처리를 막고, project.godot의 바이트뿐 아니라 권한도 대조한다. 독립 리뷰의 예외 종류 누락과 같은 바이트의 권한 변경 허위 통과를 수정했다. 루트42개(4.302초), 조정기 호환65개(56.498초), 구조 helper 변경 후42개(3.331초), 엄격 타입·Ruff를 통과했다. 실제 Godot 검사·전체 순서 연결과 불명확한 자식의 상위 소유·복구 통합은 아직 미완료다(`stage0/ordered-dispatch/GDSCRIPT_STRICT_INDEPENDENT_REVIEW_V3.md`).

구조 렌더러의 별도 live owner는 실제 lane descriptor를 소유하며 UNKNOWN·제어 파이프 EOF·중단 중에도 소유 프레임을 유지한다. 원래 wrapper/native 종료 증거가 있을 때만 동일 시도의 복구를 허용하고 복구 결과도 FAIL로 남긴다. 독립 검토에서 예외 종류 및 예외 문자열 변환으로 소유 프레임이 풀리는 결함을 재현해 수정했다. 루트 표준 discovery의 초기10개 하위 사례 실패는 테스트 자식의 모듈 검색 경로 누락으로 확인했고, 명시적인 실제 모듈 경로와 시작 stderr 기록을 추가했다. PYTHONPATH를 해제한 최종 루트19개가26.466초에 통과했고 타입·Ruff도 통과했다. 원래180초 명령을 유지하지만 timeout 검사는 합성 대기 한도이며 실제 Godot·전체 단계 연결·프로세스 강제 종료나 시스템 손실 후 보장은 없다(`stage0/ordered-dispatch/M6A_STRUCTURAL_OWNER_INDEPENDENT_REVIEW_V3.md`, `STRUCTURAL_OWNER_TEST_IMPORT_FIX.md`).

workspace doctest 집계는 기록된 Rustdoc 실행·독립적인 전체/ignored 목록·원래 stdout/stderr를 연결하는 순수 검증기로 분리했다. 고정한 edition2021 직렬 실행의 전체 section 수와 모든 바이트가 맞을 때만 선언 순서대로 배정하며, 누락 시 일치해 보이는 앞부분도 특정 소스 결과로 인정하지 않는다. 목록 실행의 실제 시작·spawn·종료·duration·입력·로그도 대조한다. 독립 검토에서 앞선 실행 누락의 오배정과 종료 필드가 없는 목록 영수증 수락을 재현해 수정했다. 루트22개가1.164초에 통과했고 엄격 타입 검사30·Ruff도 통과했다. 루트 import 순서 오류2개를 고친 기록을 보존한다. 직접 실행 및 native 프로세스 집계, 실제 목록 실행, 전체 workspace·세계 안전성은 아직 미완료다(`stage0/workspace-cases/v2/INDEPENDENT_REVIEW_V2.md`).

## Stage 0–3

Stage −1 게시 SHA가 BASE0가 된다. 이후 각 stage를 설계·구현·평가·전체 gate·게시 순서로 완료한다. Stage 1은 농경과 함께 생산 상한, 도구 전용 생존 규칙, 가르침 카드를 담당한다. 1680–1743은 이미 소모했고 Stage 1의 1520–1551은 보존한다.

저장 호환성: 실제 BASE format14 저장을 불러온 뒤 day1까지 이어 간 fingerprint와 현재 형식 재저장·재로딩 연속 실행이 BASE와 일치했다. 형식15를 분리하고 과거 집사 필드 배치를 고정했다. 잘못 format14로 기록된 미게시 LOOP 파일은 추측하여 읽지 않고 오류로 남기며 원본을 보존한다.

기존 저장 제한: tick 사이에서 플레이어 명령을 수락한 직후 저장하면 엔진 tick과 자원 tick이 한 칸 달라 `mind fact source missing, rejected or in future`로 실패한다. 동일한 진단을 실제 BASE와 형식15 이행 후보에서 실행하여 둘 다 재현했다(`save-boundary-base.log`, `save-boundary-candidate-offline.log`). 이는 저장 성공 검증이 아니라 기존 결함 재현이다. 실패 시 원시 명령 영수증과 오류를 남기며, 다음 tick으로 진행한 상태를 실패 시점 저장인 것처럼 대체하지 않는다. Stage −1에서는 이 엔진 동작을 변경하지 않는다.

벤치 출처 표기 사전 수리: 동일한 후보 SHA 환경변수가 BASE 실행에도 전달될 수 있음을 정적 검토로 발견했다. BASE 실행 파일 해시 검증과 별도로 각 군의 전체 SHA를 환경·명령 인자에 지정하고 출력의 commit과 대조한다. 회귀 검사 실패를 보존했고 수리 후 READ 검증기 단위 검사24개가 통과했다. 취소한 p4 gate는 벤치를 실행하지 않았으며 실제 성능 결과는 아직 없다.
