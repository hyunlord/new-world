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

직접 Cargo 실행과 기록 실행의 전체 native·doctest 사례를 대조하는 순수 집계를 추가했다. 원래 단계4/5 명령·전체 실행 선언·패키지 manifest의 표준 harness·실제 산출물·기록 실행 이후의 독립 전체/ignored 목록을 연결한다. 모든 section과 바이트가 맞기 전에는 소스별 결과를 배정하지 않으며, 양쪽 실패가 같아도 실행 결과는 FAILED로 유지한다. 루트37개가3.254초에 통과했고 엄격 타입 검사30·Ruff 및 독립 v3 검토도 통과했다. 직접 자식의 PID·환경·숫자 종료 코드는 관찰되지 않았고, populated jobserver FD·실제 workspace 실행·세계 안전성과 전체 gate는 미검증이다(`stage0/workspace-cases/v3/INDEPENDENT_REVIEW_V3.md`, patch SHA256 `06d64cdf0843b4d90db71eddcfc641e2478d538a052700fb25ccc9467ab0b860`).

벤치의 보존 결과를 원래6개 교차 실행 계획과 실제 시작·spawn·종료 기록에 다시 대조하는 검증기를 추가했다. 기존 수치·안전성 검증기를 그대로 호출해126개 자식 산출물과63쌍 안전성 결과를 재계산하고, 전체 산출물 목록·바이트·순서·부하·스크립트·CHANGELOG를 확인한다. 루트 독립44개가10.795초에 통과했고 표준 루트36개, 설정된 엄격 타입 범위·Ruff도 통과했다. 테스트 파일의 기존 타입 진단81개는 유지하며 전체 clean을 주장하지 않는다. COMPONENT_PASS는 빌드 출처·자식 종료 권한을 주지 않고, 실제 벤치·조정기 연결·ledger·전체 gate는 미완료다(`stage0/ordered-dispatch/BENCH_AUTH_ROOT_REVIEW_V1.md`).

전체 순서 조정기에 GDScript 단계8·46을 연결하는 명시적 소유 진입점을 추가했다. 실제 lane descriptor는 자식 종료 불명확·제어 EOF·중단 동안 유지하며, 단계46 실패 뒤의 같은 단계 검사는 실행하지 않는다. 독립 검토에서 재현한 이전 검사 prefix 대조 오류와 process 링크를 지워 강제 종료 기록을 숨기는 경로를 수정했다. 과거 결과 검증은 실제 잠금·프로세스 조회 없이 원래 선행 기록과 보존 산출물을 대조한다. 루트77개가58.057초에 통과했고 엄격 타입·Ruff 및 독립 v2 검토도 통과했다. 원래 Godot 실행·전체 gate·자동 복구 권한은 아직 없으며, UNKNOWN은 최종 inventory나 정상 실패로 바뀌지 않는다(`stage0/ordered-dispatch/STRICT_COORDINATOR_INDEPENDENT_REVIEW_V2.md`).

원래16개 oracle shell 검사를 위한 별도 helper handshake를 추가했다. 시작 전 준비·고정 request 파일·작은 알림·실제 native wait·merged log 범위를 연결하며 원래 순서와300/120초 한도를 유지한다. 신호로 끝난 상위 shell과 진단 문자열·클래스명 접근 중 예외가 허위 SETTLED가 되던 경로는 독립 재현 후 수정했다. 루트 helper34개(13.843초), 구조19개(27.387초), stall13개(0.857초)와 Ruff·Bash 검사가 통과했다. 구조 helper의 기존 동작 AST는 같고 검토한 두 해시만 갱신했다. 생산 코드는 Python3.11 엄격 타입 범위를 유지하고, typing.override를 쓰는 새 테스트는 실제 실행 도구인 Python3.14로 별도 검사해 모두0진단이다. 기존3.11 설정의 테스트 진단6개는 보존하며3.11 테스트 호환성을 주장하지 않는다. 실제 Godot oracle suite·상위 호출자 연결·전체 gate는 미완료다(`stage0/ordered-dispatch/SUITE_HANDSHAKE_INDEPENDENT_V3_REVIEW.md`, `STRUCTURAL_SUITE_V3_REBIND_REVIEW.md`).

벤치 registry의 scripted_suffix 오기를 수정했다. 실제 BASE0와 현재 READ·bench.sh가 쓰는 bench_script.json(60개 나무·8×8 비축장·4개 오두막)을 기록하며, u_script.json을 쓰는 colony·replay fixture는 그대로다. 두 스크립트의 현재 바이트가 BASE0와 같은지 확인했고 registry14개 검사(0.143초)가 통과했다. 실행 명령이나 게임 동작 변경이 아니며 벤치 backlog의 NOT_RUN·미해결 상태도 유지한다(`stage0/ordered-dispatch/BENCH25_OWNED_COORDINATOR_PLAN.md`).

벤치 실행에 명시적인 소유 상태 진입점을 추가했다. 기존 API·6개 교차 실행·수치 및 안전성 검사는 유지한다. 실제 aggregate 실행에 들어간 뒤 정상 PASS·exit0·양의 PID·원래 영수증 경로가 확인된 경우에만 종료 상태로 전환하며, 실패하거나 반환되지 않은 실행은 UNKNOWN으로 남겨 호출자가 실제 잠금을 유지해야 한다. 실행 사이 또는 모든 실행 종료 후의 검증 실패는 별도 종료된 실패로 구분한다. 진단 자체가 실패해도 원래 예외를 보존한다. 독립 검토와 루트52개(13.887초), 생산 코드 엄격 타입·Ruff가 통과했다. 테스트의 기존 타입 진단81개는 유지한다. 조정기 연결·실제 벤치·빌드 출처·전체 gate·ledger는 아직 미완료다(`stage0/ordered-dispatch/BENCH25_OWNED_LIFECYCLE_INDEPENDENT_REVIEW_V1.md`).

고정 소스 `b25149c6fd78178cb9b0991bbeebbe4ce58e51f1`의 개발용 전체 Python discovery는1,511개를849.791초에 실행해 실패4·오류15·exit1로 종료했다. 모두 구조 검사 어댑터에 집중됐으며, 나중에 로드된 `core_ui_successor_perf_validate`를 초기 모듈 스냅샷에 없다는 이유로 거부한 사전 검증과 그 후속 실패를 확인했다. 실행 후 기록한395개 입력 해시는 종료 시 동일했다. 이 기록은 정식 단계16이나 전체 gate 통과가 아니며 원본 실패 로그를 보존하고 로딩 순서 수리를 진행한다(`stage0/integrated-python-31.log`, SHA256 `079a72483b5c3755e33e6bfa7b2426ee3e6ad899a22e9d157f49cfdb9accdbb5`).

단계14 문서 검사 어댑터는 원래 `bash tools/v8/check_docs.sh`와 system-table 생성을 그대로 실행하도록 추가했다. 문서·skill 연결·Cargo 설정의 존재와 내용·원래 검사기의 경로 조회를 연결하며, 현재 제품의 세계 분류와 빌드 출처는 호출자 검증이 필요하다고 명시한다. 독립 검토에서 저장소 내부 Cargo 설정을 외부 입력으로 요구해 실행 전에 거부되는 v1 결함을 재현했고, v2는 내부 설정을 제품 입력에 넣고 실제 외부 설정만 외부 입력으로 유지한다. 원래 프로세스 경계는 완화하지 않았다. 루트58개(6.809초), 생산 코드 엄격 타입·Ruff와 독립 v2 검토가 통과했다. 임시 프로세스 입력 검증이며 원래 Cargo/system-table 실행·조정기 연결·전체 gate는 미검증이다(`stage0/ordered-dispatch/docs14-adapter-draft-v2/INDEPENDENT_REVIEW_V2.md`).

벤치 조정기를 원래 단계25 위치에 연결했다. 선택적 명시 소유 context가 있을 때만 원래6회 실행을 수행하며, BASE와 후보의 입력·스크립트·출처와 registry를 대조한다. 기존 실제 descriptor 소유자를 재사용해 잠금 하나를 유지하고, 소스가 종료를 확인한 뒤에만 보존 결과 검증을 수행한다. UNKNOWN은 최종 산출물 확정이나 다음 단계로 진행하지 않는다. 종료가 확인된 실패·PENDING 등 비통과 결과는 이후 실행과 아직 연결하지 않은 모든 의무를 NOT_RUN·blocked_by로 남긴다. 루트85개(63.265초), 엄격 타입·Ruff와 독립 검토가 통과했다. 실제 벤치·컴파일 출처 승인·전체 gate·ledger는 미완료이며 기존 strict 호출의 API와 동작은 유지한다(`stage0/ordered-dispatch/bench-coordinator-draft/INDEPENDENT_REVIEW_V1.md`).

전체 discovery에서 확인한 구조 검사 import 순서 결함을 수정했다. 실제 사용하는 eager 모듈26개의 객체·경로·초기 해시를 한 번 고정하고, 사용하지 않는 함수 내부 import가 나중에 로드돼도 이 집합이 바뀌지 않게 했다. 원래 전체36개 파일의 seal은 유지하므로 제외된 lazy 파일의 변경도 거부한다. 정확한 기존 거부를 RED로 재현했고 양쪽 import 순서·사용 모듈 누락·객체 교체·경로·바이트·seal 누락 검사를 추가했다. 루트22개(30.159초), 엄격 타입·Ruff와 독립 검토가 통과했다. 앞선1,511개 discovery 실패는 아직 전체 재실행으로 대체되지 않았으며 실제 Godot·전체 gate도 미검증이다(`stage0/ordered-dispatch/STRUCTURAL_DISCOVERY_INDEPENDENT_REVIEW_V1.md`).

단계38 원래16개 oracle shell·checker의 호출자를 추가했다. 실제 shell·helper 채널·잠금 descriptor를 보존하고, Popen 의도 이후 handle이 반환되지 않거나 종료 처리 중 오류가 나면 typed UNKNOWN으로 live 객체를 유지한다. 원래 순서·300/120초 한도·checker 중단 규칙을 바꾸지 않으며 helper 종료 확인 뒤에만 전체 로그와 산출물을 검증한다. 독립 검토에서 같은 ambient import 순서 결함을 재현해 v2에서 실제 eager 출처 고정으로 수정했다. 수정된 구조 모듈과의 별도 조합 검사 및 루트23개(29.742초), Ruff가 통과했다. 생산 코드는3.11 엄격 타입, override를 사용하는 테스트는 실제3.14 타입 검사에서0진단이며 잘못 적용한3.11 테스트 설정의6진단은 원본 로그에 보존한다. 실제 Godot·제품 빌드 출처·GPU·상위 조정기 연결·전체 gate는 미검증이다(`stage0/ordered-dispatch/ORACLES_RUN_INDEPENDENT_REVIEW_V2.md`, `ORACLES_RUN_COMBINED_STRUCTURAL_REVIEW.md`).

문서 검사에도 소스가 소유하는 종료 상태 API를 분리했다. 원래 API는 그대로이며, owned 실행은 실제 PASS·exit0과 연결된 프로세스 기록을 확인한 뒤에만 종료 상태가 된다. 양의 오류 종료도 Cargo 내부 wait 실패를 배제하지 못하므로 UNKNOWN으로 남겨 산출물 확정과 잠금 해제를 허용하지 않는다. 실행 전 거부와 종료 후 문서·출처 검증 실패는 별도 종료된 실패로 남기고, 진단·중단 오류가 원래 실패를 가리지 않게 했다. 루트71개(4.850초), 엄격 타입·Ruff 및 독립 검토가 통과했다. 상위 조정기 연결과 실제 원래 문서 검사 실행은 아직 미검증이다(`stage0/ordered-dispatch/DOCS14_OWNED_INDEPENDENT_REVIEW_V1.md`).


고정 소스 `64df8ed208f9ed48d8c47fb4e81e7f9244843de1`의 전체 Python discovery 재실행은1,583개를916.857초에 실행해 모두 통과했다. 실행 전에 기록한397개 도구 입력 해시는 종료 시 모두 같았다. 앞선1,511개 실패 기록은 보존하며, 이번 결과는 개발 회귀 검사로서 정식 단계16·전체 gate·BASE 자기 비교를 대체하지 않는다(`stage0/integrated-python-32-terminal.json`, 로그 SHA256 `591dbb0e591891915c333f6ccc9dbaa0baf1e46c83d7c0146f4ae08e374ee30f`).

단계49의 원래 정확 일치 필터에서 빠진 `colony::`를 명시적으로 수정했다. 보존된 Stage−1 final-d-03의 전용 검사는 exit0이지만0개를 실행했으며, 같은 실행의 전체 workspace에서 해당 테스트가 통과한 사실은 이 전용 호출의 누락을 대신하지 않는다. 과거 로그·게시 분류는 변경하지 않는다. release·패키지·lib·cwd·기본 스레드·capture 정책은 유지했다. 정확한 전체 명령과0개·잘못된 이름 거부 회귀를 포함한 루트28개(0.096초), Bash 구문·Ruff·독립 검토가 통과했다. 기존 테스트 타입 진단12개는 그대로이며 신규0이다. 아직 구현되지 않은 generic 단계49 호출을 지원한다고 선언하지 않으며, 수정된 실제 native 실행·독립 목록·정확히1개 성공의 전체 연결은 미검증이다(`stage0/ordered-dispatch/step49-filter-review.md`, 과거 로그 SHA256 `ed6489047d2306ba4bfc309eb82d12a7a7c6bbef9e06f0e0c720e5b15503d661`).


문서 검사를 선택적 owned context로 원래 단계14 위치에 연결했다. 기존 소유자를 재사용하고 UNKNOWN이면 다음 단계·산출물 확정·잠금 해제를 막는다. 종료가 확인된 문서 검증 실패는 FAIL을 유지한 채 원래 단계15로 진행할 수 있다. 독립 검토에서 원래/보조 오류 소실과 외부 FAIL 아래의 native PASS 입력 대조 누락을 재현해 v2에서 수정했다. 이전 정적 검사 prefix와 실제 프로세스 입력·도구·외부 파일·로그를 다시 대조하며 bench의 별도 중단 규칙을 유지한다. 루트98개(77.003초), 생산 코드 엄격 타입·Ruff·독립 검토가 통과했다. 실제 원래 문서 검사와 현재 제품의 세계 분류·빌드 출처는 아직 호출자 검증이 필요하며 전체 gate 승인을 뜻하지 않는다(`stage0/ordered-dispatch/docs14-coordinator-review/REVIEW_V2.md`, patch SHA256 `b290f2ddde918262f2b768c76e294ba40fae0d48db5929717fb0b041113e5c87`).


Oracle 호출자에 소스가 보장하는 종료된 실패 API와 보존 결과 검증기를 추가했다. 원래 launch·16개 checker·명령·순서·한도·출처 pin은 유지하며 UNKNOWN 예외와 실제 live 객체를 그대로 전달한다. 재검증은 전체 계획·shell·wrapper·native 기록·환경·로그 범위·산출물·원래 checker를 연결하고, 중간 실패 뒤 검사를 계속한 불가능한 prefix를 거부한다. 과거 파일 검증에 실제 잠금 해제나 빌드 승인 권한은 없다. 루트34개(47.159초), 생산3.11·테스트3.14 엄격 타입·Ruff·독립34개가 통과했다. 상위 단계38 조정기·실제 Godot·전체 gate는 미완료다(`stage0/ordered-dispatch/ORACLES_A2_INDEPENDENT_REVIEW_V1.md`, patch SHA256 `43659d6797c17b8666af3d7a068632657cae3ab2d4e01612c221e81a178f13ff`).


단계32 resolver parity의 원래300초 wrapper와 Python checker를 그대로 호출하는 별도 어댑터를 추가했다. 선택된 소스의 세계 미생성 분류와 실제 bridge 로드·빌드 출처 검증은 구분하며 후자는 PENDING이다. 실제 상위 shell 종료만 기록하고 native PID를 만들어내지 않는다. 실패·반환되지 않은 실행은 UNKNOWN으로 유지하고 원래/보조 예외를 보존한다. 독립 검토에서 부모 디렉터리와 자식 파일을 함께 inventory에 전달해 실제 입력 검증이 실패하는 결함을 재현했으며, v2는 필수 경로를 먼저 확인한 뒤 중복 범위만 줄인다. 단계49 수정에 따른 gate.sh pin 변경도 해당 한 줄 diff를 별도로 검토했다. 루트18개(2.602초), 생산3.11·테스트3.14 엄격 타입·Ruff·독립 검토가 통과했다. override 테스트에3.11 설정을 잘못 적용한 초기4진단과 설정 수정은 보존한다. 실제 Godot·상위 단계32 연결·전체 gate는 미검증이다(`stage0/ordered-dispatch/resolver32-draft-v2/INDEPENDENT_REVIEW_V2.md`, patch SHA256 `f3fa8e10186f48282a8b161ef696b612b0b5a00ea6ad801a7ca2fa3e5a301bf8`).


단계48 구조 검사의 소유 API를 보강했다. 실제 자식 종료가 확인된 뒤 보고서 작성이 실패하면 원래 예외를 보존하고 소유자가 잠금을 해제하며, descriptor 종료 자체가 불명확하면 실제 소유 객체를 유지하고 임의 재시도하지 않는다. 정상 결과는 해제 전에 terminal 해시와 전체 산출물 목록을 고정한다. 과거 결과 검증은 기록된 소유자·원래 child chain·두 군 계획·renderer oracle·전체 산출물을 대조하고 현재 PID나 파일만으로 live 잠금을 해제하지 않는다. 기존 API·명령·p39 import 규칙은 유지했다. 루트30개(50.762초), 새 구조 코드와 Oracle A2 조합34개(40.022초), 생산3.11·override 테스트3.14 엄격 타입·Ruff·독립 검토가 통과했다. 실제 renderer·상위 단계48 연결·전체 gate는 아직 미검증이다(`stage0/ordered-dispatch/structural48-prerequisite-draft/INDEPENDENT_REVIEW_V1.md`, patch SHA256 `3d54db60ddf186aeea4fa13e07aac5af59a697843d8d921ce31cb744383c2156`).


단계38의 원래16개 oracle 검사를 선택적 owned context로 전체 순서에 연결했다. 실제 live handle은 진단 오류와 별도로 보존하며 UNKNOWN이면 이후 단계·종료 이벤트·산출물 확정을 막는다. 소스가 종료를 확인한 실패는 FAIL을 유지한 채 원래 단계39로 진행하고, 이후 실행과 최종 기록 전에 보존 결과를 다시 인증한다. 루트106개(71.411초), 생산3.11·테스트3.14 엄격 타입·Ruff·독립 검토가 통과했다. override 테스트를 기존3.11 범위에 넣어 발생한 초기4진단과 설정 수정은 보존했다. 실제 Godot oracle 실행·제품 빌드 출처·전체 gate는 아직 미검증이며 조정기의 최종 상태는 계속 FAIL 또는 INCOMPLETE다(`stage0/ordered-dispatch/ORACLES38_COORDINATOR_INDEPENDENT_REVIEW_V2.md`, patch SHA256 `2a9260d9b8d9c38d1f359a668ea05e422180b0f231b47c68d9f501aa8534aeaf`).


단계17의 원래 M1 명령 연결을 별도 소스 계약으로 고정했다. 첫 Cargo 호출의4개 integration target, 후속3개 library filter의 기본 스레드 정책, && 실패 연결, Locale의120초·원래 grep·합계 요약을 그대로 대조한다. 독립 검토가 발견한 쉘 인용 제거에 따른 `$LOG` 확장 의미 누락을 수정했고 원래 실패 사례를 보존했다. 루트2개(0.003초, 변조7종 포함), 엄격 타입·Ruff·독립 재검토가 통과했다. 이 모듈은 명령 설명과 소스 대조만 제공하며 실제 빌드·자식 실행·전체 단계17 연결이나 세계 안전성을 증명하지 않는다(`stage0/ordered-dispatch/step17-command-review/REVIEW_V2.md`).


단계32 resolver parity를 선택적 owned context로 원래 순서에 연결했다. 실제 원래300초 wrapper와 checker를 실행하는 기존 어댑터만 사용하고, UNKNOWN에서는 원래 예외·잠금을 보존해 단계33과 최종 기록으로 진행하지 않는다. 종료된 실패는 원래/보조 오류를 기록한 FAIL로 유지하며 이후와 최종 기록 전에 보존 결과를 다시 인증한다. 루트113개(73.712초), 생산3.11·테스트3.14 엄격 타입·Ruff·독립15개 조합 검사가 통과했다. 실제 Godot·빌드 출처·전체 gate는 미검증이며 구조48과의 후속 통합 검사는 별도로 필요하다(`stage0/ordered-dispatch/RESOLVER32_COORDINATOR_INDEPENDENT_REVIEW_V1.md`, patch SHA256 `cfd462094bcf3495303f175297d73fbd79091abc10a71cf060ecb02ce69f2bc1`).


실제 workspace debug V6 실행 중 `probe::runner::tests::the_baseline_scene_takes_both_twin_checks`가 실패했다(기대1·실제0). 관찰자 오류 처리 closure를 넣은 `886ff0e5`에서 실제 들여쓰기가 바뀌었는데 기존 include_str 검사는12칸 공백을 그대로 요구했다. 현재 후보의 테스트만 각 줄 공백을 제거한 연속3줄 guard 검사로 수정했으며, 두 determinism guard 개수2와 허용 scene 검사는 유지한다. 독립 소스 검토와 공백 변경·guard 삭제·순서 변경의 텍스트 검사는 통과했으나 Rust 컴파일·native 재검증은 아직 NOT_RUN이다. 실행 중인 고정 ff512 소스와 V6 실패 증거는 수정하지 않는다. V7의 같은 ff512 준비본도 이 실패를 포함하므로 실행하지 않는다(`stage0/workspace-runtime/workspace-v6-live-diagnostic/twin-test-fix-offline-review.json`, SHA256 `411afdee14c96b98803148312ad93470d993787a4a66a487793f788c7b92267a`).


단계48 구조 검사를 기존 data-only 비교 뒤에 연결했다. 직전 direct48의 실제 인증된 PASS가 있어야 실행하고, 구조 실행기가 원래 잠금을 소유하므로 외부 잠금을 중복 획득하지 않는다. 종료된 실패만 후속 단계로 진행하며 분류되지 않은 예외는 보존하고 종료 이벤트·산출물 확정·후속 호출을 막는다. resolver32와 oracle38의 인자·순서·입력 기록·보존 결과 재검증을 유지한 통합본의 루트121개(77.383초), 엄격 타입·Ruff·독립23개가 통과했다. 아직 지원하지 않는 단계49 의무를 지원한다고 바꾸지 않았으며 실제 renderer·빌드 출처·전체 gate는 미검증이다(`stage0/ordered-dispatch/structural48-coordinator-draft/v2/INDEPENDENT_REVIEW_V2.md`, patch SHA256 `866915c9e7985bbefd6f3a86e72ac027c4efbe1a721d8934bdddb7dbca2e049b`).


단계42·43·49의 library 검사 명령과 선택 사례 집계를 분리했다. 기존 전체 workspace의 filtered0 계약을 바꾸지 않고, 독립 전체/ignored 목록에서 실제 필터와 제외 수를 계산해 정확히4·1·1개 사례를 요구한다. 단계43은 테스트1개 안의 두 메커니즘 분기로 구분하며 과거 단계49의0개 실행은 거부한다. 루트10개(0.013초), 엄격 타입·Ruff·독립 검토가 통과했다. 실제 빌드·wrapper·Cargo capture·소유권·메커니즘 관찰의 출처는 아직 미검증이다. 이 순수 집계의 PASS는 native 실행이나 전체 단계 지원을 뜻하지 않으며 dispatch는 NOT_RUN이다(`stage0/ordered-dispatch/LIBRARY_SIBLINGS_INDEPENDENT_REVIEW_V1.md`, patch SHA256 `9235f5692aa297ba9c55e18706de3e0426bfffc65a7806be7eed81f8bdeae476`).


원래 단계2·3·6의 빌드·두 번 import·clippy 준비 계약과 단계4·5 사이의 공유 target 순서를 고정했다. 현재 소스·설정·도구·환경·별도 lock 출처 기록을 대조하되 실행하거나 빌드 증명을 만들지 않는다. 독립 검토에서 중복·추가 플랫폼 selector와 잘못된 초기 checkpoint를 수락하는 결함을 재현했고, v3는 검토된8개 library 매핑 및 경로·SHA·초기 조건을 명시적으로 확인한다. 이전 실패를 보존했다. 루트15개(4.999초), 엄격 타입·Ruff·독립15개가 통과했다. 준비 상태는 PREPARED_ONLY이며 실제 producer 검증·원래 실행 순서·target 변경 출처·소유권·전체 gate는 미완료다(`stage0/ordered-dispatch/setup-prepare-draft-v3/INDEPENDENT_REVIEW_V3.md`, patch SHA256 `7bf26c9b32a4cd8b5e32a24c69811067f6c6382ba9dc244d400b75ba3930d4c0`).


고정 소스 `850cda05d644f3cadb22bd033efd66a5c936a542`의 개발용 전체 Python discovery는1,685개를979.552초에 실행해 모두 통과했다. 실행 전후405개 도구 입력 해시와 HEAD가 같았다. 정식 단계16·전체 gate·BASE 자기 비교 통과를 대신하지 않는다(`stage0/integrated-python-33-terminal.json`, 로그 SHA256 `a054e916a4250cc2c238454e383ab3b7859100b9bd8187bdb69b020ecd689149`).

workspace V6의 원래 Cargo 실행은3254.415초 후101, 드라이버는1로 종료했다. 최종 실패 target은 `sim-test --lib` 하나이며 p50에서 원인을 수리한 source-string guard 실패다. 원시180개 summary의 합계2,900통과·1실패·13제외에는 중첩 출력이 있어 고유 사례 수나 target 수로 주장하지 않는다. 입력15,807개 중 기존 bridge 산출물3개가 바뀌었고 deps에12,628개가 추가됐으며 삭제는0이다. 독립 검토는 모두 기록된 compiler 출력 계열과 연결했고 소스·도구·runtime 불변 및 실제 native/driver wait를 확인했다. FAIL·프로토콜 UNKNOWN·미관찰 EOF는 보존하고, 별도 판단으로 제가 시작한 대기 소유자만 SIGTERM해 실제143과 잠금 해제를 기록했다. SETTLEMENT나 ACK를 만들지 않았다. 수정된 p50 제품의 V8 검증은 사전 점검0 후 별도 실행 중이며 아직 통과를 주장하지 않는다(`stage0/workspace-runtime/v6-terminal-actual-audit-20261103-01/INDEPENDENT_REVIEW.md`, `v6-administrative-cessation-01/TERMINAL.json`).

원래 단계2 bridge 빌드를 실제 호출하는 owned 어댑터를 추가했다. 기존 준비 계약·Bash 명령·소스/도구/version 기록·검토된 정상 Cargo wait 경로를 대조하고 실제 호출 결과를 보존한다. 정상0만 종료가 확인된 경로로 취급하며 nonzero·중단·입력 불일치는 호출자의 실제 잠금을 유지하는 UNKNOWN이다. 성공도 endpoint 해시 확인 범위이며 BuildResult·후속 import/shared-target/clippy·전체 gate 권한은 없다. 루트10개와 엄격 타입·Ruff·독립10개가 통과했다. 실제 Cargo 실행은 미검증이다(`stage0/ordered-dispatch/setup-bridge-draft/INDEPENDENT_REVIEW_V1.md`, patch SHA256 `b6c727bf58988d78933e990209d67b755bc321c6e0243583234228a90e0e7fd6`).


원래 단계21 launcher의 여섯 경우를 구분하는 기본 비활성 plan 선택 경로를 추가했다. 원래 인자·순서·3/2/4/0/0/0 기대값은 유지하고 각 경우의 별도 plan 해시와 실제 wrapper/parent PID 관찰을 남긴다. 독립 검토가 재현한 `/`·`.` 경로 정규화로 인한 단일/suite 설정 동시 수락을 수정했고 초기 실패는 보존했다. 루트21개(0.306초), 엄격 타입·Ruff·Bash 구문·독립 검토가 통과했다. 실제 launcher를 호출하는 기존 검사1개는 이번 focused 검사에서 제외했으며 실제 역사 버전 실행·aggregate 소유권·세계 관찰·전체 단계21 통과는 아직 미검증이다(`stage0/play-run/case-selector-draft/v2/INDEPENDENT_REVIEW_V2.md`, patch SHA256 `4f36f846abba24da9e442ad3c1fdc5b7cbfead64cd0beee9f1d4fa4d4ffbd808`).


단계36 native 의무의 이름을 실제 원래 명령에 맞게 `data_only_ext.debug`로 바로잡았다. 원래 `data_only_ext.sh`는 `--release` 없이 debug·threads1로 실행하며 UI 검사는 그 뒤에 온다. 실행 명령이나 지원 범위는 변경하지 않았고 아직 미구현인 native 의무를 PASS로 바꾸지 않았다. 순서 조정기11개(8.374초), 엄격 타입·Ruff가 통과했다(`stage0/ordered-dispatch/IMPLEMENTATION_PRIORITY_AUDIT_P53.md`).

단계49의 원래 Bash·Cargo·정확 일치 libtest 호출을 연결하는 owned 실행 어댑터를 추가했다. Cargo와 libtest의 실제 부모 관계·종료·로그·입력과 독립 전체/ignored 목록을 대조해 정확히1개 선택을 요구하며 UNKNOWN은 호출자의 실제 잠금을 유지한다. 검토에서 목록 생성 중 추가 파일을 놓치는 결함을 재현해 전체 파일·디렉터리 집합을 고정했고, 통합 타입 검사가 발견한 순환 import는 library 전용 연결을 실행 모듈로 옮겨 제거했다. 이전 실패 증거는 보존했다. 루트67개(48개12.510초와19개3.484초), 생산3.11·테스트3.14 엄격 타입·Ruff·독립 V3 검토가 통과했다. 첫 루트 호출의 잘못된 cwd로 발생한 import 오류도 별도 로그로 남겼다. 실제 native 실행·소스/빌드 승인·direct 대비 관찰자 검증·상위 조정기 연결·전체 gate는 아직 미완료다(`stage0/ordered-dispatch/LIBRARY_RUNTIME49_INDEPENDENT_REVIEW_V3.md`, patch SHA256 `3c329baa86dff941908797f0dd4cb45edb16e7ba805803cbd9802b655f91f4db`).

단계41 stall 검사를 선택적 owned context로 원래 순서에 연결했다. 실제 bounded wrapper와 native 종료 연결이 확인된 뒤에만 소스가 종료 토큰을 만들고, 이후 검증 실패는 원래/보조 예외를 보존한다. UNKNOWN은 실제 잠금을 유지해 후속 실행과 최종 기록을 막는다. 원래 strict1·complete=false 기록은 유지하며 timing STRICT_PASS 또는 WAIVER_ELIGIBLE도 생존/KnowledgeLost와 나머지 단계의 최종 결합 전에는 PENDING이다. 루트171개(217.645초), 조정기 생산/테스트 엄격 타입·Ruff·독립13개 검토가 통과했다. 기존 capture 파일의175개 타입 진단과2개 EXE001은 그대로이며 신규 진단은0이다. 실제 GPU·빌드 출처·세계 안전성·전체 gate는 미검증이다(`stage0/ordered-dispatch/stall41-coordinator-draft/v2/INDEPENDENT_REVIEW_V2.md`, patch SHA256 `79229f2cafdcd80e784f59a81a62fd75dd886ee174ccfa70df576197533cf32a`).

단계45용 평가기는 직전44의 실제 생산자 반환 시 고정한 영수증·전체 파일 해시를 받아7개 등록 family·9개 짝 비교를 다시 검증한다. 원래18개 프로세스의 명령·입력·로그, 보호된 baseline 원시 identity, normalizer·선언 및 생존/KnowledgeLost를 재계산하며 기존 실패를 PASS로 바꾸지 않는다. 과거 CORE held-out 과학 평가와 새 일반 fixture 비교를 구분하고 소모된 seed를 실행하지 않는다. 루트10개(7.494초), 엄격 타입·Ruff·독립10개가 통과했다. 성공도 범위 내 평가만 통과한 PENDING이며 실제 소스/빌드·성능·전체 coverage·상위45 연결·전체 gate는 미완료다(`stage0/ordered-dispatch/core45-evaluation-draft/INDEPENDENT_REVIEW_V1.md`, patch SHA256 `46f89729f72486e5ff470cb5015f8eb12fefd507616e84179151d573af02d27d`).

단계49 실행기를 원래48과50 사이에 선택적 owned context로 연결했다. 실제 소유 descriptor와 실행기 반환 객체를 사용하고 runtime PASS도 소스/빌드·관찰자 검증 전에는 PENDING으로 유지한다. UNKNOWN은 후속50·최종 기록·잠금 해제를 막으며 종료가 확인된 실패는 FAIL로 보존한다. 독립 검토에서 반환 객체 없는 종료된 실패의 빈 디렉터리 변경을 놓치는 결함을 재현해 모든 고정된49 결과의 파일·디렉터리 구성을 다시 대조하도록 수정했다. V1 전체135개, 루트 V2 통합25개(13.052초), 엄격 타입·Ruff·독립8개가 통과했다. 실제 native49와 전체 gate는 아직 미검증이다(`stage0/ordered-dispatch/LIBRARY49_COORDINATOR_INDEPENDENT_REVIEW_V2.md`, patch SHA256 `e9d802f96f2c168cce3c03e8f5bc6cab87f23fb0f43846895c4a6b9d8b4a0586`).

원래 launcher의6개 경우와16개 자식 호출을 실제 request/grant·native wait·case wait·outer wait/EOF로 연결하는 지속 소유자를 추가했다. 원래 명령과3/2/4 기대 오류는 유지하고, UNKNOWN은 실제 Popen·채널·복제한 잠금 descriptor를 보존한다. 독립 검토에서 이미 끝난 외부 native 증거 폴더에 추가된 파일을 허용하는 결함을 재현해 파일·디렉터리 종류와 전체 구성을 매 후속 검사 및 최종 종료까지 고정했다. 루트29개(11.541초), 타입 검사·Ruff·Bash 구문과 독립 변조6종이 통과했다. 실제 루트에서 발생한 import 정렬3건은 기록 후 수정했으며 독립 AST 검토가 본문 불변을 확인했다. 실제 launcher를 호출하는 기존 테스트1개는 제외했다. 역사 source/build·native/world·단계21 qualification은 INCOMPLETE이며 이 소유자가 게시 권한을 만들지 않는다(`stage0/play-run/owner-review-metadata-supplemental-v2/REVIEW.json`, `IMPORT_SUPPLEMENT.json`).

단계49 실행 환경은 호출자가 명시한 빈 `RUSTC_WRAPPER`·`RUSTC_WORKSPACE_WRAPPER`만 보존하도록 확장했다. 생략과 빈 값은 구분하며 외부 환경을 가져오지 않고 비어 있지 않은 값은 실행 전에 거부한다. V6의 build/실행 환경 불일치가 근거이며, 캐시 재사용이나 build 적합성의 증명은 아니다. 루트 실행15개(15.819초)·coordinator8개(6.139초), 타입 검사·Ruff 및 독립 검토가 통과했다. 실제 native qualification은 여전히 진행 중이다(`stage0/ordered-dispatch/library49-empty-wrappers-review-metadata/REVIEW.json`).

원래 단계3의 import 두 호출을 실제 단계2 반환 영수증에 연결했다. 첫 호출 실패 뒤 두 번째 호출을 수행하는 세미콜론 동작과 raw 종료값을 보존하며, 각 native 종료 및 최종 게시 전후의 target·cache·settings 상태를 마지막 인증 checkpoint와 비교한다. 독립 검토에서 발견한 최종 상태 재기준화 결함은 V2에서 차단했다. 루트 관련138개 검사(139.256초), 타입·Ruff 및 독립 변조 검토가 통과했다. 기존 oracle/structural 소비자의 helper 해시도 같은 코드로 고정했다. 이는 실제 Godot import 실행 또는 전체 gate 통과 증거가 아니다(`stage0/ordered-dispatch/setup-import-draft-v2/INDEPENDENT_REVIEW_V2.md`).

단계4/5 workspace 실행기는 원래 Cargo 인자·cwd·무제한 실행과 실제 native/Rustdoc capture 및 전체·ignored 목록을 연결한다. 목록별 파일 해시와 파일/디렉터리 구성을 마지막 재검사까지 고정하며, native 실패와 소유권 UNKNOWN을 구분한다. 루트94개(18.063초), 타입·Ruff와 독립 검토가 통과했다. 반환은 `OBSERVED_CASES_ONLY`이며 source/build 인계, 원래 direct 대조, world 안전성과 전체 gate 권한은 없다. 호출자는 UNKNOWN에서 실제 lane 소유권을 유지해야 한다(`stage0/ordered-dispatch/workspace-runtime-review-v2/INDEPENDENT_REVIEW_V2.md`).

단계6 Clippy 실행은 원래 Bash 명령을 유지하고 고정한 Cargo/Clippy 소스·도구와 실제 정상 종료를 연결한다. 명시적인 setup 환경값을 보존하며 주변 환경을 가져오지 않는다. 루트 준비/bridge/Clippy37개(16.747초), 타입·Ruff와 독립 검토가 통과했다. 정상 종료는 해당 endpoint만 증명하고, 이전 단계4/5의 실행 순서와 생성물 인계는 아직 별도 검증이 필요하다. 실제 Clippy 실행은 NOT_RUN이며 비정상 또는 미확인 반환에서 caller lane을 유지한다(`stage0/ordered-dispatch/setup-clippy-review-v2/INDEPENDENT_REVIEW_V2.md`).

단계45 재평가기를 owned 순서의44 직후에 연결했다. 실제44 반환값과 당시 전체 증거 파일을 소비하며, 실패를 유지하고 source/build·안전성 전체 판정은 PENDING으로 남긴다. 루트18개(14.705초), 타입·Ruff와 독립 검토가 통과했다. 외부153개 통합 실행은 기존 순서 기대값1건이 실패한 기록을 보존했고, V2에서 정확한45 기대값을 추가한 제한 재검사가 통과했다. 실제 native 실행이나 전체 gate 통과를 뜻하지 않는다(`stage0/ordered-dispatch/core45-coordinator-review/REVIEW_V2.md`).

단계42/43 실행은 원래 source scan·Cargo·tee·grep 명령과 실제 자식 반환·독립 목록을 연결한다. 42의4개,43의1개 테스트를 계수하고 43 내부 두 분기의 관측은 아직 NOT_RECORDED다. 독립 검토가 찾은 빈 decision 추가·baseline 삭제 누락을 V2에서 모든 원래 scanner 조건과 Rust 입력 재검사로 차단했다. 루트 runtime/coordinator31개(39.971초), 타입·Ruff 및 독립 재현이 통과했다. 49 경로는 유지하며 실제 source/build·observer qualification과42/43 coordinator 연결은 아직 미완료다(`stage0/ordered-dispatch/library4243-review-metadata-v2/REVIEW.json`).

실제 import→workspace 연결 fixture가 드러낸 빈 wrapper 파싱 결함을 수정했다. import profile에서 명시한 두 compiler-wrapper 키만 빈 문자열을 허용하고 일반 oracle 및 다른 환경키의 검증은 유지한다. 실제 소형 두 import의 값 보존·생략·잘못된 값 거부, 독립 normal oracle 검토와 최종 공유 소비자114개(126.674초), 타입·Ruff가 통과했다. 수정 중 구 버전 pin을 로드한 예비 실행의9실패27오류와 잘못된 테스트 출력경로 실패는 보존하며 최종 통과로 재분류하지 않는다. 실제 Godot/native qualification은 여전히 미완료다(`stage0/ordered-dispatch/import-empty-wrappers-review-metadata/REVIEW.json`).

## Stage 0–3

Stage −1 게시 SHA가 BASE0가 된다. 이후 각 stage를 설계·구현·평가·전체 gate·게시 순서로 완료한다. Stage 1은 농경과 함께 생산 상한, 도구 전용 생존 규칙, 가르침 카드를 담당한다. 1680–1743은 이미 소모했고 Stage 1의 1520–1551은 보존한다.

저장 호환성: 실제 BASE format14 저장을 불러온 뒤 day1까지 이어 간 fingerprint와 현재 형식 재저장·재로딩 연속 실행이 BASE와 일치했다. 형식15를 분리하고 과거 집사 필드 배치를 고정했다. 잘못 format14로 기록된 미게시 LOOP 파일은 추측하여 읽지 않고 오류로 남기며 원본을 보존한다.

기존 저장 제한: tick 사이에서 플레이어 명령을 수락한 직후 저장하면 엔진 tick과 자원 tick이 한 칸 달라 `mind fact source missing, rejected or in future`로 실패한다. 동일한 진단을 실제 BASE와 형식15 이행 후보에서 실행하여 둘 다 재현했다(`save-boundary-base.log`, `save-boundary-candidate-offline.log`). 이는 저장 성공 검증이 아니라 기존 결함 재현이다. 실패 시 원시 명령 영수증과 오류를 남기며, 다음 tick으로 진행한 상태를 실패 시점 저장인 것처럼 대체하지 않는다. Stage −1에서는 이 엔진 동작을 변경하지 않는다.

벤치 출처 표기 사전 수리: 동일한 후보 SHA 환경변수가 BASE 실행에도 전달될 수 있음을 정적 검토로 발견했다. BASE 실행 파일 해시 검증과 별도로 각 군의 전체 SHA를 환경·명령 인자에 지정하고 출력의 commit과 대조한다. 회귀 검사 실패를 보존했고 수리 후 READ 검증기 단위 검사24개가 통과했다. 취소한 p4 gate는 벤치를 실행하지 않았으며 실제 성능 결과는 아직 없다.


단계21의 기존 play 소유자에 선택적 역사 버전 출처 검증을 연결했다. 실제 checkout의 commit·tracked manifest·lock·Git/Cargo/rustc·설정 존재 여부와 실제 build 반환의 선택 bridge를 대조한다. 독립 검토에서 재현한 child PATH의 다른 Git 허용, 활성 config.worktree 누락, v7 tag 객체와 commit 혼동을 V2에서 수정했다. v7 tag `7db2907651b069f2db9dd843599abe1872e581b2`와 peeled commit `b1ca032f8fa84cee87d68a185e471b8e2c2bddce`를 구분하며 기존 잘못된 기록은 보존한다. 루트19개(35.298초), 생산3.11·테스트3.14 엄격 타입·Ruff·독립11개 검사가 통과했다. 루트 Ruff의 테스트 import 순서1건은 정렬 후 AST 구성 동일성을 확인하고19개를 재실행했다. 역사 버전 실제 launcher/build/Godot·관찰자·세계 검증·상위 단계21 연결은 아직 미완료이며 선택적 검증을 끈 기존 호출도 INCOMPLETE다(`stage0/play-run/source-grants-review-metadata-v2/REVIEW.json`, SHA256 `1023a2a70014b910c64859aa2ea42a5dd14c9acefa5f7a4f123cd5e01007af90`; patch `fc97492b7d33a09b98af6d51cef43c45bdd8070381e5b7e1f5ee525c092cf26b`).


단계42/43 실행기를 선택적 owned context로 원래36→42→43→37 순서에 연결했다. 단계49의 기존 소유 경로를 재사용하며 실제 request·반환 결과·로그와 디렉터리 목록을 재검증한다. 성공은 출처·관찰자 검증 전 PENDING으로 유지하고 FAIL은 보존하며 UNKNOWN에서는 실제 descriptor와 원래 예외를 유지해 이후 실행을 막는다. 잘못된 정확 필터는 실행 전에 거부한다. 루트21개(25.835초, CORE45 조합 포함), 생산3.11·테스트3.14 엄격 타입·Ruff·독립14개 검사가 통과했다. 외부 V1 전체141개 통과와 V2 제한 재검사를 구분해 보존한다. 실제 Cargo 실행·소스/빌드·관찰자·43 내부 분기 관측·전체 gate는 미완료다(`stage0/ordered-dispatch/LIBRARY4243_COORDINATOR_INDEPENDENT_REVIEW_V2.md`, SHA256 `c2eccc6906ec1d268a125ae8ea06bc5db6fc58aef2936a4479be40d31ba1073f`; patch `c94c5acbef22095bfe094df764bd65d094f5a21255251e05d3b41b387d42b7e9`).


준비 단계의 실제 ImportOwner/result→debug4→release5 연결을 추가했다. 각 producer 전후 target·Godot cache·설정 상태와 이전 원본 증거를 고정하며, 뒤 producer가 바꾼 target을 과거 값과 다시 같다고 요구하지 않는다. 실행 사이와 목록 검사·최종 기록 중 변경은 거부하고 기존 보호 binary 검사는 유지한다. 독립 검토에서 최종 기록의 predecessor 해시 변조가 통과하는 결함을 재현해 V2에서 실제 선행 digest와 chronology를 대조했다. 두 handoff에서 잘못된 두 필드를 거부한다. 루트26회 실행(서로 다른15개,76.511초), 생산3.11·테스트3.14 엄격 타입·Ruff·독립 재검토가 통과했다. V1 실패 기록은 보존한다. 이 연결은 같은 프로세스의 단계 일관성만 증명하며 BuildResult·실제 Cargo/Godot·원래 전체 shell 순서·관찰자·세계 안전성·전체 gate 승인을 뜻하지 않는다(`stage0/ordered-dispatch/workspace-endpoint-review/REVIEW_V2.md`, SHA256 `7fe58581405f961b280fc4daad3458013a9252761e9f81d41f83a1fedeed2f83`; patch `90ddb1c38635cded371ee04ca4bbfbf6988b44ba2498d327de777e560f486b23`).


단계28의 colony_knowledge release·threads8과 단계36의 data_only_ext debug·threads1 native prefix 실행기를 추가했다. 원래 shell 명령·36 subshell·로그 전달·9개 중첩 child 구간을 대조하며, 관측하지 못한 중간 PID·child 환경은 null로 남긴다. F10·UI 후속 부분은 NOT_RUN이며 prefix 성공으로 전체 단계를 통과시키지 않는다. 독립 검토에서 완료된 listing과 최종 callback 뒤 파일/빈 디렉터리 추가가 통과하는 결함을 재현해 V2에서 각 반환 직후부터 최종 기록까지 목록·바이트를 고정했다. 루트12개(19.391초, 변조8종 포함), 생산3.11·테스트3.14 엄격 타입·Ruff·독립 재검토가 통과했다. 실패 뒤 남은 result.json만으로 성공을 인증할 수 없고 실제 반환/예외와 전체 증거를 소비해야 한다. 실제 Cargo·빌드/소스·관찰자·세계 안전성·상위 연결·전체 gate는 미완료다(`stage0/ordered-dispatch/integration-prefix-review-metadata-v2/REVIEW.json`, SHA256 `e9ced293f969c0bb4fc1437d1e69648784e7ba305daea04c0e120bf83a79b6b8`; patch `78c48dbf80e7a36f72144c014c21678e0f130e98c13fbcd07494ce339ff9535e`).


실제 workspace V8의 직접 실행은 native0으로 종료했다(원본180개 요약 합계2901통과·0실패·13ignored, 고유 case 수 아님). 이후 manifest 준비는 테스트 실행 파일154개와 구분되지 않은 non-test compiled dependency9개 때문에 실패했고 관찰자 경유 실행은 NOT_RUN이다. 전후 의존성 목록은 동일했다. fresh146114-file 연속성 검사와 실제 direct0/driver1 종료를 확인한 뒤 대기 owner9637만 관리상 종료했다(session91861 exit143). 프로토콜 SETTLEMENT·EOF 관측이나 전체 검증 성공으로 바꾸지 않는다. 추가 소스 감사에서 실제 테스트가 쓰는 상위 worldsim_probe 경로가 직접 실행 전 해시에 없음을 확인해 V8 직접 실행 재사용은 거부했다. 새 직접 실행은 해당 실제 compiler output도 먼저 고정해야 한다. 증거와 결정은 `D0-workspace-compiled-dependencies`, `stage0/workspace-runtime/workspace-debug-qualification-v9-draft/DIRECT_REUSE_SOURCE_AUDIT.md`(SHA256 `6ae992c5b8ddc3515d15bb2fdecd1e15476f0888923e76db6a5b3796ebbf78f1`)에 보존한다. V8 전체 결과와 엄격 BASE 자기 비교는 계속 FAIL이다.


단계17/37의 선택 실행을 위해 default·substring·ignored-only·exact-ignored 네 모드의 순수 case 집계를 추가했다. 실제 독립 전체/ignored 목록에서 선택·필터 제외 집합을 계산하고 기존 parser의 이름별 결과와 숫자 합계를 재사용한다. 선택된 ignored 테스트를 일괄 거부하지 않으며 실패 수와 원본 로그 해시를 유지한다. 루트 공유 parser 조합45개(0.105초), 생산3.11·테스트3.14 엄격 타입·Ruff와 독립17개 및 잘못된 로그9종 거부가 통과했다. 실제 argv·프로세스 종료·목록·빌드·세계 검증은 호출자가 별도로 인증해야 하며 이 함수에 실행 승인 권한은 없다(`stage0/selected-cases-draft/INDEPENDENT_REVIEW_V1.md`, SHA256 `44e4de1efcdacf3d8dfebc9e87888ccb2321d1897b0ac77e49b4b12bfb68d5d7`; patch `d3cec787c6056324461f955b4e9500d5a48044f0384376d558adabf16288e728`).


단계21의 여섯 launcher 경우를 원래21→22 순서의 선택적 owned 호출로 연결했다. 실제 역사 버전 출처·후보 launcher 입력·원래3/2/4/0/0/0 반환·16개 자식 산출물을 대조하고 성공도 검증 전 PENDING으로 남긴다. UNKNOWN은 실제 owner·descriptor·예외를 유지하며 settled FAIL은 뒤 단계를 계속한다. 독립 V1 검토에서 거부된 외부 산출물의 예상 밖 파일 바이트 변조가 통과하는 결함을 재현해 V2에서 전체 파일 해시와 디렉터리 종류를 함께 고정했다. 실패 기록은 보존한다. 루트150개(287.761초), 독립8개(28.825초), 생산3.11·테스트3.14 엄격 타입·Ruff가 통과했다. 테스트 파일 공백 한 줄은 AST 동일성을 확인해 정리했다. 실제 역사 버전 빌드·Godot·세계 관찰·전체 gate는 아직 미완료다(`stage0/ordered-dispatch/play21-coordinator-review-metadata-v2/REVIEW.json`, SHA256 `68eede5475c75a85599f85f056b88519b9de6e0ab0bcf69581ec34c4a918bc83`; patch `ba2f91bb8947f9af027d03642ae06d875a5bc6f41db792f09003b39e02821ab7`).


workspace 의존성에 테스트154개와 별도의 schema3 compiled 입력 범주를 추가했다. 정확한 입력 해시·provenance 파일·reader 소스와 닫힌 검색 목록이 필요하며 실행 대상 수에 더하거나 불명 실행 파일을 허용하지 않는다. provenance 내용은 빌드 증명이 아니다. 독립 V1 검토가 생성 stdout을 사전 입력으로 인정하는 결함을 재현했고 V2는 실제 ProcessSpec 입력만 대조하며 생성 증거는 별도로 유지한다. 루트 관련141개(22.230초), 엄격 타입·Ruff, 독립42개와 원래 실패 재현의 거부가 통과했다. V1 실패·외부160개 호환성 검사와 최종42개 재검사 기록은 구분해 보존한다. 실제 V9는 상위 probe를 포함한 모든 compiled output을 직접 실행 전에 고정하고 새로운 direct/wrapped 쌍을 실행해야 한다. native·세계 안전성·전체 gate는 아직 미완료다(`stage0/workspace-runtime/schema3-independent-review-v2/REVIEW.md`, SHA256 `f4123d79832d4250f5f10be318ee718c17f7d378ced42f90c048579459c4a5b7`; patch `317c4257c14c09035ecafb2fc292a9251b62c43299ca129eefca33076427eab0`).


단계28/36 native prefix를 기존 F10/UI 호출 앞에 선택적으로 연결했다. 실제 prefix 성공만 같은 단계의 후속 호출을 허용하며 settled FAIL은 해당 후속만 건너뛰고 외부 순서를 계속한다. UNKNOWN은 실제 요청·owner·잠금을 유지한다. 성공도 전체 단계 승인이 아닌 PENDING이다. 독립 V1 검토에서 최종 기록 callback 뒤 빈 디렉터리 추가가 검증을 우회하는 결함을 재현해 V2에서 마지막 입력 callback과 기록 뒤 원래 바이트·종류·목록을 재검사한다. 변조가 있으면 원래 terminal을 보존하고 해당 경로/해시를 참조하는 별도 FAIL 영수증을 반환하므로 호출자는 반환 경로를 소비해야 한다. 루트30개(66.457초, play21·library42/43/49 조합), 독립8개·4개 변조 조합, 엄격 타입·Ruff가 통과했다. 실제 native·세계 안전성·전체 gate는 미완료다(`stage0/ordered-dispatch/integration-prefix-coordinator-review-v2/REVIEW.md`, SHA256 `ba39329b83897d7ec6ee9038dd155c96b25cb4c0ea11d3531fa5550708798ce5`; patch `4251ee90e059c5a8390192825f960bf69abe31718f917508840b5905a2cbd067`).


단계17 원래 shell 전체를 한 번 실행하는 owned runtime을 추가했다. 원래4개 Cargo 명령·7개 native 실행·120초 locale wrapper/Godot의13개 실제 완료와14개 독립 목록을 연결하며 && 중단과 locale marker 실패를 native 종료와 구분한다. UNKNOWN은 실제 worker Thread·요청·FD·원래 예외를 유지한다. 독립 V1 검토가 첫 인증 전에 늦게 추가된 산출물을 기준값으로 채택하는 결함4종을 재현해 V2는 producer·outer·listing 반환 직후 인증 전 목록/바이트를 고정한다. V3는 테스트의 실패 지점·이유를 정확히 확인하고 테스트 소유 ACK/worker를 정리한다. 과거 tests06은 OK 출력 후에도 프로세스가 남아 있었으므로 무개입 terminal PASS가 아니며, 검증된 소유 leaf만 종료한 실제 정리 기록과 초기 원인 UNKNOWN을 보존했다. 최종 루트29개(55.524초)는 실제 exit0, 생산3.11·테스트3.14 엄격 타입·Ruff가 통과했다. 실제 source/build·관찰자·세계·locale 자원·상위 연결·전체 gate는 PENDING이다(`stage0/ordered-dispatch/m1-runtime-review-metadata-v2/REVIEW.json`, SHA256 `78aadf3fab1ff95a3716b0a5e1a2fca7c2058c75e9ca007bd9b193617603630b`; test-only V3 review `6fe8f1e258d66572fc1c2e077b104cd08559f40af14acd3085ea3c4e3375a258`; patch `c2b0a3ee113413c4b8afd6a1c5b099fae35409d033b32fd0415af6f7e20d1872`).


단계17 전체 runtime을 원래17→18 순서에 선택적으로 연결하고 기존 부분 호출의 중복 실행을 제거했다. 실제8개 의무·전체 산출물 바이트/종류/목록과 입력을 callback 및 최종 기록 뒤에도 대조한다. 성공은 PENDING, settled FAIL은18로 계속하며 UNKNOWN은 실제 요청·Thread·Live·FD를 유지한다. 최종 기록 후 변조는 원본을 보존한 별도 FAIL 반환 경로로 전달한다. 루트22개(77.970초, prefix·play 조합), 독립14개(57.186초), 생산3.11·테스트3.14 엄격 타입·Ruff가 통과했다. 실제 M1 native·소스/빌드·관찰자·세계 안전성·전체 gate는 미완료다(`stage0/ordered-dispatch/m1-coordinator-review-metadata-v1/REVIEW.json`, SHA256 `80527737cf8b31a9e3ffc96c65bfd985a3f8dcd5d3b484cc729c21ea206a9297`; patch `f1aad8857c3b799455544b5865385756469f065d182dc7fb105a8bbb1f9e38e9`).


실제 단계5 반환 endpoint를 선택적으로 단계6 Clippy 요청에 연결했다. 같은 준비 객체·실제 predecessor·생성 target/cache/settings를 확인하고 producer 반환 직후부터 인증 및 최종 기록까지 바이트·종류·전체 목록을 고정한다. Clippy의 target 변경은 실제 실행 구간에서만 허용하며 cache/settings와 역사 증거는 유지한다. 기존 네 인자 API와 비정상 반환의 UNKNOWN 소유권 규칙은 보존한다. 루트27개(89.044초, workspace endpoint 조합), 독립13개 메서드(기존12개와 새13상황을 포함한1개) 및 실제 소형 bridge/import/debug/release→Clippy 인계가 통과했다. 소형 통합은 실제 역사 endpoint 검증을 사용하며 source wait qualifier만 명시적으로 모의했다. 엄격 타입·Ruff가 통과했고 trailing whitespace 한 줄 제거는 AST 동일성을 확인했다. 실제 Cargo/Godot·전체 준비 순서 qualification·세계 안전성·전체 gate는 미완료다(`stage0/ordered-dispatch/clippy-endpoint-review-v1/REVIEW.md`, SHA256 `86520634ec0fa21bc949d5f4275389c61ee5ca1592488bbf6d9ca024d7c01634`; patch `f1b81ef8596096d4669c93c3a27609dc3004f7a0fcee8dd71dc30fad165396f4`).


같은 소스의 debug→release workspace 선행 빌드를 실제 두 반환 객체로 묶었다. 기존 fresh build API는 유지하고 debug 증거와 target을 보호하며 release producer의 새 release 경로만 허용한다. V1의 producer 게시 중 변조와 V2의 최종 chain 영수증/빈 디렉터리 변조를 독립 재현해 V3에서 각 게시 전 바이트·종류·모드·목록 및 의도한 JSON을 고정했다. 실패한 원본 chain은 보존하고 별도 failure 경로를 반환한다. 실제 반환 객체의 유지된 anchor를 재검증하는 API는 후속 source/tool/config/양쪽 증거/target 변조와 영수증만 복원한 객체를 거부한다. 루트 V3 86개(84.482초), 독립17개(56.017초)·282해시·5객체 대체 거부, 엄격 타입·Ruff가 통과했다. V2 루트122회 검사 통과는 최종 게시 결함을 해소한 증거가 아니며 철회된 승인과 함께 보존한다. 결과는 pristine 소스의 BUILD_ANTECEDENTS_ONLY이고 원래 단계2–6 실행을 대체하지 않는다. observer overlay·warm 준비·환경 동등성·Rustdoc·실제 Cargo·세계 안전성·전체 gate는 별도 검증이다(`stage0/ordered-dispatch/setup-chain-build-review-v3/REVIEW.md`, SHA256 `01c78e8a974bf7b227c4d8597e3963d445580d09afd2107fea2b3e20c731aa42`; full patch `090bacc0505983822eab0abf2ac37e27244abe159ddfdd5ed20fb6f90d1296b8`).


기존 fresh 준비 규칙을 유지하면서 실제 유지된 workspace build pair만 받는 warm 준비를 추가했다. 같은 source/root/tool/config/target과 양쪽 실제 프로세스 환경을 대조하고 present-empty wrapper·명시적 jobs·locale을 고정한다. HOME/PATH 등 차이는 존재 여부를 포함해 기록하며 환경 동등성을 주장하지 않는다. recipe 선언 전에도 실제 pair를 재검증하고 새 target 기준값으로 갱신하지 않는다. 루트28개(20.867초, fresh·bridge 조합), 독립18개(15.450초), 엄격 타입·Ruff가 통과했다. 이는 pristine 빌드 선행물의 준비일 뿐 실행·observer·Rustdoc·세계 검증 권한이 아니다. 원래 단계2의 started 기록 뒤 실행 전 재검증 결함은 별도 bridge 수정 대상으로 확인돼 있으며, 해당 수정과 실제 단계2–6 검증 전 전체 준비 경로를 승인하지 않는다(`stage0/ordered-dispatch/warm-prepare-review-v1/REVIEW.md`, SHA256 `02f07c9f0a7ce12d5a57a5ce40f98633b48a6b9fdac816fd0c5f32c7681ad852`; patch `24d48a2f61d088f350b9acd430f375f272fb00ffea5c0cd1c3df951248471407`).


단계2 bridge producer의 상태를 최초 인증 전에 고정하고 최종 기록까지 재검증했다. 독립 검토가 warm 준비의 started 기록 중 target/pair 증거 변경 뒤 실행에 도달하는 결함을 재현해 V2에서 실행 직전 source·wait·유지된 pair·초기 target/cache/settings·started 바이트와 출력 구성을 다시 확인한다. 원래 fresh/warm producer의 실제 생성 효과는 허용하되 실행 밖 변경은 거부한다. 11개 실행 전 변조는 producer를 시작하지 않고 거부하며, 실행 후 미확인 상태는 caller FD를 유지한다. 루트35개(82.670초, import 조합), 독립19개(55.360초)와 원래 warm 재현2종의 실행 전 거부, 엄격 타입·Ruff가 통과했다. V1에서 거부 시점이 늦었던 raw 변조1종과 false PASS10종, V2 이전의 prelaunch 실패를 구분해 보존한다. 실제 Cargo/Godot·전체 준비 순서·세계·전체 gate 검증은 미완료다(`stage0/ordered-dispatch/bridge-endpoint-integrity-review-v2/REVIEW.md`, SHA256 `e64961b256b266a54094cf1c308fc78c14eff1e6dd4e99459425e6a9552d1d67`; patch `6e52c79b3c61fd5f3475dc931f076a2f38dc75d2fb4ee477376bbf9848ab107c`).


workspace 선행 빌드 pair가 기존에 인증한 observer overlay도 처리하도록 확장했다. debug에서만 한 번 적용하고 release는 원래 clean baseline·index·허용된 postimage를 유지해 실제 파생 소스를 다시 검증한다. 새 허용 경로를 추가하거나 dirty 소스를 clean으로 취급하지 않는다. 두 receipt는 원래 baseline과 실제 effective map, 이번 빌드의 patch 적용 여부를 구분하고 의미적 동등성은 NOT_RUN으로 남긴다. 루트96개(75.021초, 기존 build·pristine warm 조합), 독립93개(65.019초) 및 최종 게시 중 overlay patch/manifest/postimage/index/mode 변조5종 거부, 엄격 타입·Ruff가 통과했다. 실제 Cargo·관찰자 동등성·파생 소스 warm 준비·전체 gate는 별도 검증이다(`stage0/ordered-dispatch/setup-chain-draft/build-overlay-review-v1/REVIEW.md`, SHA256 `65c1efabd071bed3eb03d19a1e4d08560358701443712cba6baeff69f52ce0c8`; patch `46f9cedf831f0d4562e9b58ed88a04222989adfcd02c156d6251b8b3d6e53ad0`).


단계37 defence 전체 실행기를 추가했다. 원래 shell의 Cargo4개·native10개·validator1개와 독립 목록20개를 연결하며 실제 Popen·PID 관계·caller FD를 유지한다. 독립 V1 검토에서 spawn 기록의 사후 변경과 첫 인증 전 capture 빈 디렉터리 추가를 새 기준값으로 받아들이는 결함을 재현했다. V2는 게시 전 spawn 바이트를 유지하고 완료된 capture를 첫 검증 전에 고정한다. 루트 독립16개 중11개 통과(80.570초), 통합33개 중28개 통과(68.499초), 각각 canonical-lock 검사5개는 V9 실행 중이므로 미실행이다. 실제 두 세션 exit0, 엄격 타입·Ruff가 통과했다. 양성 fixture는 복사한 잠금 경로와 소형 실행 파일을 명시적으로 사용하며 원본 잠금·실제 Cargo·진짜 과거 저장·source/build·observer·세계 안전성·상위 coordinator·전체 gate는 미완료다. V1 실패 기록을 보존한다(`stage0/ordered-dispatch/mux2-runtime-review-v1/REVIEW.md`, SHA256 `4142a3c5e497a3b9aca734ab8e109d0c6bd955b10937597c1c59074cb3608aff`; V2 patch `ad0e9056cc2a17b67c77fd4095628d7baae0fa803b8ed3208c84c5dc24ec3336`).


원래 단계10/11/12/18의 내장 Cargo build 명령을 실제 predecessor endpoint에서 실행하는 소유 경로를 추가했다. 단계10의 ROOT 지역 변수와 subshell, 나머지 원래 package/bin 명령을 보존한다. V1의 started 내용/목록 및 마지막 callback 뒤 변경 허용을 V2에서 게시 전 의도한 바이트·초기 목록과 유지된 전체 역사 anchor로 차단했다. 독립 원래 공격3종 거부·8개 통과(38.608초), 루트 Clippy 인계 조합9개 통과(46.638초), 엄격 타입·Ruff가 통과했다. 초기 source/wait·predecessor 인증은 소형 fixture에서 명시적으로 모의했으며 실제 이전 단계의 native 이력·Cargo·BuildResult·중간 workload 인계·전체 wrapper 순서·전체 gate는 미완료다. 성공 endpoint만으로 이를 대체하지 않는다(`stage0/ordered-dispatch/embedded-build-review-v2/REVIEW.md`, SHA256 `d3b42784939ee2e98d9892368d0ebf921d8976808d69972c6a97c273c21e83aa`; patch `a1466166fc28ff69999b14639c926756adf201bc1fcd53ef7c28d82ab6d3394b`).


observer 파생 소스의 warm 준비를 연결했다. 최초에는 실제 workspace pair 전체가 온전해야 하며 이후 source-only proof는 원래 baseline/index·허용 postimage·도구/설정·양쪽 producer와 pair의 역사 증거를 유지한다. 후속 producer가 바꾼 현재 target/실행 파일은 이 source 검사에서 제외하되, 초기 recipe/full-pair 및 각 producer의 생성물 검증은 유지한다. dirty 소스를 pristine으로 바꾸거나 serialized receipt로 권한을 복원하지 않는다. 루트 관련148개 검사(241.445초), 독립76개(137.201초)와 추가 source/history 변조4종 거부, 엄격 타입·Ruff가 통과했다. 외부 mirror의 누락 shell로 발생한 초기39오류는 보존했다. 실제 파생 bridge/import/workspace 전체 실행·관찰자 동등성·세계 안전성·전체 gate는 미완료다(`stage0/ordered-dispatch/setup-chain-draft/warm-derived-review-v1/REVIEW.json`, SHA256 `f2f4d6ec7f45b0f301fe51951aa49707f4d53a5ba711caad0bc2b7ee53dd43a9`; patch `84383033fcf16684731767d67cbaabce1fc88bafa8d1f6f270b04901cb266cc3`).


단계37 전체 실행기를42/43 뒤·38 앞의 owned 순서에 연결하고 기존 부분37 호출을 제거했다. 원래11개 의무를 한 번 처리하며 실제 반환만 인증한다. 정상 성공은 PENDING, settled FAIL은 보존한 채38로 계속하고 UNKNOWN은 실제 요청·Popen을 가진 Live·FD·예외를 유지한다. 첫 인증 전과 최종/실패 기록까지 증거 목록·종류·바이트 및 입력을 재검사한다. 루트23개(88.255초), 독립8개(43.583초), 외부 영향받은36개 호환 검사(96.578초), 엄격 타입·Ruff가 통과했다. 외부156개 실행의9실패26오류는 복사 fixture19개 누락으로 남겨 두며 전체156개가 재통과했다고 쓰지 않는다. 실제 원본 잠금·Cargo·과거 저장·source/build·관찰자·세계·전체 gate는 미완료다(`stage0/ordered-dispatch/mux2-coordinator-review-v1/REVIEW.md`, SHA256 `53c7ce5ea0fa65ba78b193c0327a9301f9af7a2f86f392b184121493bd89046c`; patch `dae5996f94d34ea49eb35a6cb93e9937d8c1fcffaa5aa6f22054c06840af833c`).


원래 준비2→3(두 import)→4→5→6을 단계1 뒤에 연결했다. 실제 준비·predecessor·workspace factory·반환 endpoint와 FD를 유지하고 settled 실패와 UNKNOWN을 구분한다. V1의 최종 target 변경 누락은 V2에서 전체 producer 종료 직후의 현재 target/cache/settings를 고정해 최종 검증·기록 뒤 대조하도록 수정했다. 뒤 producer의 정상 변경을 과거 단계6 값과 비교하지 않는다. 독립 원래 재현은 명시적 FAIL로 바뀌었고 추가6변조와 정상 후속 변경이 검증됐다. 루트 setup/MUX2/M1 조합20개(213.536초), 두 선택 경로를 함께 켠 순서·중복 검사1개(0.003초), 엄격 타입·Ruff가 통과했다. 새 순서 검사의 초기 oracle 플래그 누락과 수정 명령의 경로 오류는 별도 실패 로그로 보존했다. 별도 소스 검토는 병합한21개 helper가 원본 그대로이며26개 positional field가 일치함을 확인했다. 초기 외부 fixture의 /bin/bash unlink 시도는 OS가 거부했고 파일 해시는 그대로였으며, V2는 fixture 경계·symlink 거부를 유지한다. 실제 native 준비/관찰자/세계/전체 gate는 미완료다(`stage0/ordered-dispatch/setup-coordinator-review-v2/REVIEW.md`, SHA256 `e8846f6be80b0b9c239e7a4dba7f47e23508e98a7300cfd0000d272cd8297d58`; source merge `f611ec5b8d50a8b0fc62c34852aa03cab4701de36ed406aef93c29b4cf23564a`).


실제 workspace build pair의 normal-bin 산출물 인계를 추가했다. 선택 profile의 원본 metadata/compiler stream과 package·target·source·manifest·정확 경로를 대조하며 test harness와 proc-macro를 구분한다. 기록이 없으면 새 빌드를 추측하지 않고 실패한다. V1의 마지막 pair 검증 callback 뒤 target 빈 디렉터리·소스·미선택 binary·선택 executable mode 변경 허용을 독립 재현해 V2에서 원래 유지된 pair 전체를 마지막에 검사한다. 루트79개(131.230초), 독립7개(50.524초)와 원래4공격 거부, 엄격 타입·Ruff가 통과했다. 현재 pair가 온전할 때만 유효하며 후속 setup 변경 뒤 source-only proof로 새 target을 채택하지 않는다. 실제 Cargo·후속 endpoint 인계·런타임/세계·전체 gate는 미완료다(`stage0/ordered-dispatch/normal-artifact-review-v2/REVIEW.md`, SHA256 `e8b1d68689c26ff179970f43f57d086feb15610a999dad20a51af668ed4c3e58`; patch `c8393aa0ea933d7ba3b67c7798dda5e471f250e93a2615974d7067bec0db010c`).


CLI의 실제 선행 빌드 준비 경로를 추가했다. 독립 BASE/후보와 pristine harness, 전체 committed registry·선언·소스·설정·정확 환경을 overlay 전에 고정하고 기존 build owner로 BASE 다음 후보의 debug/release pair를 실제 실행한다. BASE 실패는 후보 NOT_RUN으로 남기고 예외는 임의 settled 상태로 바꾸지 않는다. V1의 마지막 인증 callback 뒤 harness/양쪽 원본 출력 변경 허용을 V2에서 원래 소스·target·증거·종류/모드·부재 설정의 전체 유지된 closure로 차단했다. public recheck는 실제 객체 identity와 같은 closure를 요구하며 후속 setup 뒤 새 기준값을 채택하지 않는다. 루트25개(125.286초), 독립18개(75.912초)와 원래3공격 거부, 엄격 타입·Ruff가 통과했다. 공개 SHA 인자는 관측된 선택값이며 이 함수 자체가 remote 상태를 조회했다는 뜻은 아니다. 실제 native·런타임 context 구성·전체 CLI dispatch·전체 gate는 미완료다(`stage0/ordered-dispatch/cli-prepare-review-v2/REVIEW.md`, SHA256 `fc722926ecf131f22633bcbf343476b63b3ce091a5c81b885a5106906641ab2a`; patch `a8f7c744ca13ac98dafdf98264737871b9f3f7264c2ee8835237c8d1b75662fc`).

### S0-p92 — 실제 normal 산출물에서 CLI 실행 descriptor 구성 (전체 gate 아님)

실제 양쪽 build pair의 release compiler 기록에서 probe/perf를 선택하고 source-only 입력과 별도 target 입력을 구분한다. perf sidecar는 원본 build/process를 가리키는 projection이며 새 빌드 증거가 아니다. V1의 마지막 callback 뒤 source/target/실행 권한 변경과 출력 권한 변경 허용을 원래 유지된 CLI closure 및 미리 정한 directory0700/file0644로 차단했다. upstream 변경은 기존 raw 비교를 private 메서드로 추출한 것뿐이며 실제 객체 identity 검사는 유지한다. 루트가 실행한 동결 mirror11개(195.043초), 독립9개(183.462초)와 원래5공격 거부, 엄격 타입·Ruff가 통과했고 통합한 세 파일의 바이트 일치를 확인했다. setup 이후에는 이 pre-setup descriptor를 새 기준값으로 재인증하지 않는다. 실제 runtime 입력 소비·setup successor·전체 CLI/gate는 별도 작업이다. 증거: `stage0/ordered-dispatch/cli-products-review-v2/REVIEW.md` SHA256 `b31f7719d8783e8f8cfc9e2939d7afb6632c227d4a3b342f27f68db21f2e384e`; patch `d65bf80e5bef6c36a970b6056db908ab63cfd3c98e44a7c1851b7bf250f8f91e`.

### S0-p93 — perf/bench 실제 프로세스에 runtime 입력 결합 (전체 gate 아님)

`Product.input_files`의 source-only 의미를 유지하고 별도 `runtime_inputs`를 실제 perf/bench ProcessSpec과 retained 인증, standalone safety 검사 및 ordered dependency 기록에 결합했다. 경로·중복·충돌·symlink·해시 변경은 거부하며 기본 빈 map의 기존 기록 형식은 유지한다. 실행 중 실제 작은 자식 프로세스가 runtime 파일을 바꾼 경우 native exit0이어도 process FAIL로 보존한다. 루트 관련81개(20.164초)와 dependency/routing5개(0.037초), 독립81개(21.371초)·추가12개(6.917초)·dependency9개(0.179초), 엄격 타입·Ruff가 통과했다. standalone safety 누락 RED와 루트 테스트 이름 오타로 인한 첫 loader 오류는 외부 기록에 남겼다. map 자체는 setup/build provenance가 아니며 실제 endpoint에서의 구성과 후속 producer 이후 역사적 인증은 미완료다. 증거: `stage0/ordered-dispatch/perf-runtime-inputs-review-v1/REVIEW.md` SHA256 `c63bfc693203465bb739885a1fe33abbfa8b3ade5af4647fcc24ce8808db8c1c`, `runtime-dependencies-review-v1/REVIEW.md` SHA256 `5eb3aa995b6eec2fd95018deab506465a8437bddca1fc558686b7aa70cdc1c37`.

### S0-p94 — 원래 build와 실제 setup successor 연결 (전체 gate 아님)

실제 CLI 산출물·warm preparation·bridge/import/debug/release/Clippy2→6의 동일 객체와 원본 증거를 연결한다. 이후 소스·harness·도구·설정·이전 증거는 원래 closure로 유지하고 후보 target/.godot만 실제 생성 단계 결과와 따로 검증한다. 선택된 normal 실행 파일과 bridge가 바뀌면 거부한다. V1의 마지막 callback 뒤 소스 변경 및 harness 권한/빈 디렉터리 누락, V2의 발급 객체/검증 범위 대체 허용을 V3에서 차단했다. 새 immutable projection은 원래 실제 CliBuilds와 원래 anchor·전체 범위를 요구하며 매 consumer 경계에서 발급 객체를 확인한다. 과거 import에 current-target snapshot을 재호출하지 않고 실제 종료한 owner와 원래 terminal을 확인한다. 루트21개(200.713초), 독립2개(49.362초), 엄격 타입·Ruff·범위 문서 검사가 통과했다. 앞선 V2 루트21개 통과는 독립 결함을 덮지 않으며 모두 보존했다. 실제 제품 native·실행 CLI 연결·전체 gate는 미완료다. 증거: `stage0/ordered-dispatch/cli-setup-handoff-review-v3/REVIEW.md` SHA256 `c8ded9801370d6063810d0cdf8a7b2a18d7cb8096a2b4b42211e60f3df70d2f2`; patch `a2dad47f709e347eb113b1c2fe31e6f691c968fb90a4cad17ac3345c0686feda`.

### S0-p95 — 깨끗한 체크아웃의 structural 테스트 준비 수정

동결 p91 전체 Python 검사는 2,091개 실행, 실패12·오류46·skip5, 2,039.811초, 실제 exit1로 종료했다. 검사한 tools451개 해시는 유지됐다. 58개 실패 블록 모두 테스트 fixture가 ignored `.godot/global_script_class_cache.cfg`를 요구한 경로로 확인했으며, stderr가 숨겨졌던 한 자식도 원본 fixture로 재현했다. 원래 FAIL은 보존한다. tracked resolver 테스트에 이미 있던 정확한 synthetic cache literal을 structural fixture 자체에 넣어 생성된 체크아웃 캐시 의존을 제거했다. production hash·resource-resolution 검사와 기존 음성 검사는 바꾸지 않았다. 루트38개(55.007초), 독립38개(53.243초)·깨끗한 mirror3개(1.735초), 테스트 타입0오류·Ruff가 통과했다. 전체2,091개 재실행은 아직 하지 않았으며 실제 Godot cache 생성 증거가 아니다. 증거: `stage0/python-regression-p91-terminal.json`, `stage0/ordered-dispatch/structural-cache-fixture-review-v1/REVIEW.md` SHA256 `ad06af46656d5c39dc6786a83deed9e2c9c788cee18fadd0443cc0823927e413`; patch `b572fab9db20573c34140f98c48b08c2aaf365c536683375a6d464a266272755`.

### S0-p96 — 실제 Cargo 인자를 보존하는 doctest 정책과 schema4 수집

실제 workspace metadata·원본 compiler stream·Cargo.lock·pinned toolchain에서 다섯 doctest 의무를 구성하고, 원래 Cargo가 한 번 생성한 인자를 닫힌 규칙으로 검증한 뒤 그대로 전달한다. 사전 전체 direct 실행으로 인자를 학습하지 않는다. live Cargo 부모의 실제 start/spawn·환경·입력과 원본 의무 집합을 실행 전·transient·terminal에서 연결하며, 나중 목록은 인증한 실제 intent에서 유도한다. schema1–3은 유지한다. A V1의 모드/마지막 callback/의무 축소, V2의 검증 callable 대체를 실제 발급 객체와 원본 closure로 차단했고, B V1의 마지막 부모 검사 중 entry/intent/빈 디렉터리 변경은 exec 직전 원래 게시 의도와 다시 대조한다. Python private module을 임의 수정하는 공격까지 격리한다는 뜻은 아니다.

통합 후 발견한 import cycle4건은 순수 인자 검사 모듈을 추출해 없앴다. constructor와 wrapper는 이 새 모듈의 실제 해시를 각각 결합하며, 옛 explicit manifest에 자동으로 누락 입력을 채워 넣지 않는다. 루트180개(29.741초), EOF 빈 줄만 제거한 최종 파일의20개(8.006초), production/test 전체 타입0오류·Ruff가 통과했다. 독립 A3·B2 및 추출 검토가 승인됐고 최종 추출 독립20개(8.224초)가 통과했다. 앞선179개 통과와 타입4오류는 그대로 보존했다. 이는 작은 실제 자식/모의 도구 fixture 검증이며 실제 Cargo argument equivalence·full bind chain·전체 CLI·세계 안전성·full gate는 아직 별도 검증이다. 증거: `stage0/ordered-dispatch/workspace-policy-review-v4/REVIEW.md` SHA256 `48bb8ca0344939b1c27db3c28550a9ed66bb96f33f6b0a88a0d3df743eb6af12`; `workspace-schema4-review-v2/REVIEW.md` SHA256 `ea889e90b97ee1d7eb37ab58a6b07b48cb5e90f6d4b9a99752f46fa4d642785c`; 최종 파일/실행 기록 `workspace-policy-schema4-root-v4-validation.json`.

### S0-p97 — BASE와 후보의 실제 Godot 준비 인계

원래 양쪽 빌드의 발급 객체와 소스 기록을 유지한 채 BASE 보조 bridge/import와 후보2→6을 각각 연결한다. BASE 준비는 후보1 전에 수행하는 별도 선행 작업이며 후보의4/5/6을 BASE에 복제하지 않는다. Godot descriptor는 실제 양쪽 생성 상태에서 유도하고, BASE probe witness는 후보의 후속 빌드를 허용하면서 양쪽 불변 소스·과거 증거와 BASE 산출물을 계속 검증한다. 기존 후보 전용 기본 경로는 유지한다.

V1/V2에서 마지막 검증 callback 뒤 소스 변경이 누락되는 경로를 재현했고, V3는 발급 당시 유지한 소스 검사와 원본 도구·산출물 모드/해시·증거·생성 상태를 마지막에 직접 대조한다. 새 기준값을 채택하지 않는다. 루트23개(353.047초), production/test 전체 타입0오류·Ruff·문서 부분 검사가 통과했고 네 파일 해시가 검사 전후 동일했다. 독립 원래 재현과 여섯 source/history/mode 변경 검사가 같은 호출에서 거부됨을 확인했다. V1/V2와 재현 harness의 초기 키워드 오류는 보존했다. 이는 작은 실제 프로세스 fixture 검증이며 실제 Godot smoke·제품 빌드·세계 안전성·전체 gate 통과 증거가 아니다. 증거: `stage0/ordered-dispatch/godot-products-review-v3/REVIEW.md` SHA256 `e7c40b5449dbadd728b180d9c8cc988b9a1af79758620d1ad5275744a9bce8e9`; patch `80a834dbc29a0a75ffa59aa3e7eb325725b70524fdf560131a4021b36cb1bff0`; `godot-products-root-v3-validation.json`.

### S0-p98 — 승인된 단계49 수정과 static 소스 고정값 일치

CLI fixture09에서 static1/7/8이 실행 전에 거부됐다. 고정된 gate.sh 해시는 p42 이전 값이었고, 현재 파일과의 전체 차이는 이미 승인된 단계49 exact filter의 `colony::` 접두사 하나뿐이었다. 독립 Git/blob 대조로 static13개 경로가 그대로임을 확인한 뒤 고정값만 현재 `ba949251efa089536c44b9883515d017c7254122b4167756833dfd7070d8c523`으로 갱신했다. 임의 버전 허용이나 검증 우회는 추가하지 않았다. 실제 저장소 파일과 상수를 비교하는 회귀 검사는 옛 값에서 실패하고 새 값에서 통과했다. 루트 static72개(5.582초), production/test 전체 타입0오류·Ruff가 통과했다. fixture09 실패와 과거 static 실행 기록은 원래 소스 기준으로 보존한다. 증거: `stage0/ordered-dispatch/static-gate-pin-audit/REPORT.md` SHA256 `07caaad547bfb89dfcd20a715dfb9f046ac01331ba4cd5a9179206fa3cc7053a`; patch `e45f84211eea6d98b3f2de8abfea5f34e523b6a247a9bd87470983f6ea5a2db0`; `static-gate-pin-root-v1-validation.json`. 전체 gate 통과를 뜻하지 않는다.

### S0-p99 — 저장소 루트 discovery에서 순수 모듈 검사 실행

p96 고정 worktree 전체 discovery는2,137개·2,538.273초 뒤 실패1개/건너뜀5개로 종료됐다. 추적한 tools 입력461개는 바뀌지 않았다. 유일한 실패는 순수 matcher import 검사 자식이 저장소 루트 cwd를 상속해 `generic_workspace_arguments`를 찾지 못한 것이었다. 루트에서 같은 실패를 재현한 뒤 자식 cwd를 실제 로드한 matcher 모듈 디렉터리로 명시했다. import 순환 방지 assertion은 그대로다. 저장소 루트 discovery의 해당 모듈11개(6.693초)와 tools/v8에서 단일 검사, 테스트 전체 타입0오류·Ruff가 통과했다. p96 전체 실패 기록은 보존하며 전체2,137개 재통과를 주장하지 않는다. 원래 잠금5개 건너뜀도 실제 실행으로 바꾸어 계산하지 않는다. 증거: `stage0/python-regression-p96-terminal.json`, 로그 SHA256 `33b5f7b893a104b5c03de687af09c11d4a0e80b97737b6c3f1c8c3f5d5f86aee`; `workspace-policy-cwd-red.log`, `workspace-policy-cwd-green.log`.

### S0-p100 — 실제 준비1→8을 수행하는 닫힌 CLI

`--execute-setup-prefix`가 실제 양쪽 선행 빌드, 원본 compiler/policy, 준비2→6, static1/7/8을 연결한다. 아직 생성되지 않은 runtime product를 가정하지 않고 실제 반환 객체와 소유 FD를 유지한다. 전체51단계113의무 중 이후 단계는 NOT_RUN이며 성공한 prefix도 INCOMPLETE/종료2·게시 권한 없음으로 남긴다. 이전 UNKNOWN/실패 시도01–09는 보존했다. 명시적 test-only 도구/잠금 치환을 사용한 실제 C/Python fixture10은1→8을 끝냈고 static1/7/8은0/PASS, setup2→6은PENDING이었다. 실제 제품 Cargo/Godot 자격 검증은 아니다.

V1에서 후속 callback이 앞 단계 receipt 옆에 빈 디렉터리를 추가하면 마지막 snapshot이 이를 채택하는 문제가 재현됐다. V2는 각 완료 단계의 전체 컨테이너와 원래 모드, 허용된 prefix 항목을 다음 producer 전과 게시 후 대조한다. 루트24개(160.668초), 독립17개(13.275초), production/test 전체 타입0오류·Ruff가 통과했다. 독립 네 가지 경로/모드 변경이 거부됐고 정상 mock coordinator는 INCOMPLETE를 유지했다. fixture10은 V1 당시 실제 실행 기록이며 V2 실제 fixture 재실행으로 바꾸어 적지 않는다. 현재 static pin은 현재 후보 gate.sh 계약만 검증한다. BASE0 자체 gate.sh 해시는 `dd3d52db1905af1211d207dcd39796de150d6c2c6eba2595624cef3afbec7938`이며 self-proof 원본 계약 및 기존 KnowledgeLost 실패는 별도 미해결이다. 증거: `stage0/ordered-dispatch/cli-execute-review-v2/REVIEW.md` SHA256 `90960f11db37cb1dc031933672ad17a054c77431ee889f54313a8a1809a0a019`; patch `e88d9796312c6721b5570d9e871573588b7c3e46e0dac3436800ba7d124a2883`; `cli-execute-root-v2-validation.json`.

### S0-p101 — 실제 후보 빌드 뒤 등록된 단계11 workload 인계

BASE는 원래 실제 normal probe와 보조 준비 witness를 재사용하고 후보만 원래 embedded build를 한 번 수행한다. 뒤따르는 양쪽 determinism 실행의 실제 ProcessSpec/ProcessResult와 종료 출력을 즉시 유지한다. 기본 fixture runner는 그대로이며 선택적 실제 반환 보존 경로만 추가했다. 정상 nonzero는 FAIL, 반환·소유권 불명은 원래 예외와 소유자를 유지한다. 후속 후보 target 변경 뒤 과거 증거를 검증할 때 현재 binary를 새 기준값으로 채택하지 않는다.

V2의 마지막 source callback 뒤 빈 디렉터리 누락과 V3의 외부 Godot 도구 권한 변경 누락을 각각 재현했다. V4는 원래 발급된 양쪽 source closure와 시작 시 검증된 외부 도구 해시·모드를 최종 검증까지 유지한다. 루트21개(168.749초), 작성자20개(180.951초), 독립 원래 mode 재현 거부, production/test 전체 타입0오류·Ruff가 통과했다. V3 전체26개 통과 및 추가 결함은 함께 보존하며 V4 전체27개 재실행으로 기록하지 않는다. 원래 shell/cwd/echo, smoke 순서, CLI 연결, 실제 native/세계·전체 gate 자격은 아직 미완료다. 증거: `stage0/ordered-dispatch/embedded-workload-review-v4/REVIEW.md` SHA256 `0100261f37ccf9ff814e1b162d2ac838acacc9eb40b90421d6062b91effc322d`; patch `dcf13fe69bb57e888c5bbe17a35548cfc25ae1e63e24d0bdde156cb92dcb8911`; `embedded-workload-root-v4-validation.json`.

### V9 실제 workspace 자격 검증 — FAIL 보존

고정 제품 `40f3feb81`/harness `71820a6a4`의 direct와 wrapped는 각각 실제 종료0이며154개 capture의 무결성 PASS·완전성 COMPLETE였다. 그러나 독립 listing 대조에서 sim_core가 거부돼 자격 검증은 FAIL이다. 실제 목록491개와 관측된 고유 이름491개는 같지만, 소스에 선언된 내부 child7회가 출력에 섞여 outcome498개·summary8개가 되었고 기존 단일-summary parser가 이를 거부했다. 전체 direct/wrapped case parity 및 세계 안전성은 완료되지 않았다. 실패 RESULT SHA256 `2e862c2cdd1fa639230ce499afd9adc5d1eebd3628bfeab9510ea7cf3cebea67`, 독립 원인 감사 `workspace-runtime/v9-static-listing-failure-audit-01/REVIEW.md` SHA256 `daa8eb51c8ba7dbb9bb75afc0adaa6cfb920d31618340ab4493506475037f4ff`를 보존한다.

Driver98042의 실제 종료1 후 전용 owner94208은 완료되지 않은 settlement를 계속 검사했다. 두 프로세스 목록·정확 시작/명령/cwd/FD·자식 없음과 원래 결과 해시를 확인한 뒤 이 owner에만 SIGTERM을 보냈고 session44604 실제 종료143을 얻었다. 기존 잠금 inode가 그대로 사용 가능함을 확인했지만 원래 FAIL과 protocol UNSETTLED, SETTLEMENT/UNKNOWN 파일 부재는 바꾸지 않았다. 상세: `workspace-runtime/v9-administrative-recovery-audit/ROOT_TERMINAL.json`. 후속 parser 수정·과거 결과 대조는 별도 원본 입력 검증과 실제 listing 실행을 요구하며 V9를 재분류하지 않는다.

### S0-p102 — 원래 문서 검사14의 실제 산출물 인계

원래 문서 검사와 소유 프로세스를 한 번 실행하고, 단계12의 실제 endpoint에서 단계14가 만든 target 변경을 후속 단계에 넘긴다. 기존 API와 기본 실행은 유지하며 실제 ProcessSpec/ProcessResult, 원래 양쪽 source closure, 외부 도구 바이트·권한, 이전 증거 전체를 보존한다. 정상 oracle FAIL은 FAIL로 남기고 독립적으로 검증된 산출물 인계만 허용한다. 소유권 불명은 실제 실행과 FD를 유지하며 후속 endpoint를 발급하지 않는다.

V2는 마지막 generated callback이 source 빈 디렉터리를 추가하는 독립 재현에서 실패했다. V3는 의미 검사 뒤 원래 target/cache/settings를 직접 비교해 같은 호출에서 이를 차단했다. 독립490개 해시 대조·원래 재현 및 외부 도구 권한/닫힌 FD 검사, 추가 회귀1개22.801초가 통과했다. 루트 신규·static27개(397.613초)와 기존 경로58개(6.691초), production/test 전체 타입0오류·Ruff 및 제한된 docs 검사가 실제 종료0으로 통과했다. 이전 V1/V2 실패는 보존한다. 단계15/16/17 순서 연결, BASE0 원본 gate 계약, 실제 native/세계·전체 gate 자격은 별도 미완료다. 증거: `stage0/ordered-dispatch/docs14-successor-review-v3/REVIEW.md` SHA256 `6f88ea0843f15e06f93bc8a429770248c45ac91f654c750aed34c30890793763`; patch `244c5527b0abdedc9b8e847b092247a1dcd7186b959365f9d9bfcba69df27d72`.

### S0-p103 — 단계36의 원본 소스 검사를 자기 단계로 한정

단계36이 무관한 최신 단계28 명령을 요구해 정확한 BASE0 gate를 거부하던 경로를 수정했다. 요청된 단계의 정확한 명령만 검사하며 단계36 wrapper·Godot 환경·자식9회 topology 해시와 실제 입력/FD 계약은 유지한다. 단계28 자체는 기존 최신 명령을 계속 요구하므로 BASE0 단계28 호환성을 새로 주장하지 않는다.

전체 원본 BASE0 gate 바이트로 거부를 재현하고 수정 후 단계36 admission을 확인했다. 루트14개(20.276초), 외부 postimage14개(22.385초), production/test 타입0오류·Ruff가 통과했다. 앞선 mirror12개 실행은 수정 전 fixture 검사로 별도 보존하며 수정 후 검사로 계산하지 않는다. 실제 native/전체 단계36·세계·전체 gate는 미완료다. 증거: `stage0/ordered-dispatch/step36-root-validation.json`; patch `4e84c532a513d599618720a18a89cd3c0ad4418cd4dbd5865657bcdf5fe8bbd7`.

### S0-p104 — 소스에 선언된 중첩 native 테스트 집계

sim_core 등 실제7개 target의 부모/자식 테스트 블록·guard 상수·원본 실행 입력을 확인한 경우에만 정해진 중첩 횟수를 집계한다. 원래 전체 목록과 ignored 목록, 모든 outcome·summary·filtered 수를 대조하며 원본 출력 해시를 유지한다. PID 관계는 관측하지 않았으므로 SOURCE_BOUND_AGGREGATE_ONLY로 남긴다. 직접 실행은 실제 Cargo Running 순서와 각 전체 aggregate를 검증한 뒤 범위를 나눈다. 일반 단일-summary 경로와 알 수 없는 중첩 실행 거부는 유지한다.

V1의 미등록 progress 이름 및 ok 뒤 임의 suffix 허용을 독립 재현해 V2에서 엄격한 이름/문법 검사로 닫았다. 루트가 실행한 동결 postimage58개(16.595초), 독립9개(0.237초), 통합22파일 바이트 일치, production/test 전체 타입0오류·Ruff 및 제한된 docs 검사가 통과했다. B5 ignored, 같은 이름의 재귀 child, data-only9회는 그대로다. 실제 stdout의 빈 줄은 byte identity를 위해 유지한다. 추가6개 target의 테스트 목록은 소스 유도 fixture이며 실제 listing 증거가 아니다. 실제 남은 listing 실행과 전체 direct/wrapped parity는 미완료이고 V9 FAIL/UNSETTLED는 바꾸지 않는다. 증거: `stage0/ordered-dispatch/workspace-nested-accounting-review/REVIEW_V2.md` SHA256 `2078430d587474968b1b41c554f723b070a27a897c6b0d87358c02459f034869`; patch `dcdae761a07da5d24566115eb558c500e9543efef3cb9b802fb464def15eb6e3`; `nested-accounting-root-v2-validation.json`.

### p99 고정 소스 전체 Python 회귀 종료

고정 SHA `3f6fe40edab11b9fa12b6c36b5b5fce850137968`의 전체 discovery는2,140개·2,538.536초, 실제 종료0으로 통과했다. 원래 잠금 검사5개는 기본 건너뜀으로 유지하며 실행한 검사로 계산하지 않는다. 추적 입력463개의 전후 해시가 동일하다. p96의 원래 실패와 p99 수정은 보존하고, 이 결과를 이후 HEAD 또는 formal 단계16/전체 gate 통과로 바꾸지 않는다. 증거: `stage0/python-regression-p99-terminal.json`; 로그 SHA256 `8dfc60ff6a5beca3c79b54841638015e44e2191102c456dc49b50637122c7f39`.

### S0-p105 — BASE0 자체 검증용 정적 소스 계약

정확한 BASE0 self_proof 선택, 실제 양쪽 선행 빌드와 원래 paired source closure에서만 정적 소스 계약을 발급한다. 원본 gate blob·변경 없는13개 명령·승인된 effective source를 대조하며 gate를 observer postimage로 바꾸는 것은 거부한다. 기본 현재 gate 검사는 그대로다. 후속 generated 변경 뒤에도 원래 source proof를 유지하고, 계약을 복사하거나 다른 docs predecessor에 붙이는 경로를 거부한다.

소스 receipt 조인 함수는 실행 증명이 아니다. CLI/fullgate 연결에서는 기존 실제 process·입력·oracle·전체 receipt 검사를 계속 요구하며, 이 커밋은 그 연결을 아직 포함하지 않는다. 루트 실제 작은 fixture4개(56.766초), 통합 현재 gate 검사1개, 독립 foreign closure 거부 및 production/test 타입0오류·Ruff·제한된 docs 검사가 통과했다. 작성자 전체89개는88성공과 외부 sibling gate.sh 누락 오류1개로 종료됐고, 정확한 fixture 보충 후 해당1개만 재검사해 통과했다. 이를 깨끗한89개 전체 재실행으로 기록하지 않는다. 실제 BASE0/native·전체 gate와 기존 KnowledgeLost 실패는 미해결이다. 증거: `stage0/ordered-dispatch/base0-static-independent-review/REVIEW.md` SHA256 `30ff264e8f0131eb20a7ba506352a64029b409c02511b6f2547fb8053813bce3`; patch `8d391fb44c72e8440a98a1c3e8f1cc7aaf11beaa4122b9985bdd56bb90433fcf`; `base0-static-root-validation.json`.

### S0-p106 — CLI 원래 smoke9 연결과 BASE0 정적 계약 유지

명시적인 양쪽 setup 선택을 받는 추가 CLI 경로가 BASE bridge/import를 한 번 실행한 뒤 원래 후보1~9 순서를 수행한다. 실제 smoke 자식8회의 spec/result·원래 출력과 소유 FD를 유지한다. V1에서 재현한 terminal 내용·권한·빈 디렉터리 변경 및 마지막 callback의 selector 변경을 V2에서 거부한다. 기존1~8 경로는 유지하며 성공한 prefix도 INCOMPLETE이고10~51은 NOT_RUN이다.

정확한 BASE0 self_proof에서만 p105 소스 계약을 발급하고 단계8 입력 재구성에도 같은 계약을 유지한다. 전체 process·native exit·입력·oracle 검사를 전후 소스 검증과 함께 유지하며 소스 계약을 실행 증명으로 사용하지 않는다. 두 변경의 독립 검토 후 루트에서 합친41개 검사가410.045초·실제 종료0으로 통과했고, production/test 전체 타입0오류·Ruff가 통과했다. 작은 fixture의 실제 프로세스와 명시적인 native/world 검증 대체를 구분한다. 실제 BASE0/Godot/세계 및 전체 gate는 미완료이고 기존 자기검증 FAIL은 그대로다.

증거: `stage0/ordered-dispatch/cli-smoke-static-root-p106-validation.json`; CLI V2 review SHA256 `cc87059a1471385bb3d8ea1cac8f3aa1ca2d06f588e359f487b9522db4131579`; 정적 연결 review `3a45e50457ff743b6bd69aca730009c6bca29f0ee21d5ea0d47e4ee0073fafbb`. 원래 V1 실패와 전체39개 작성자 검사·독립5개 및 정적 연결 독립2개 결과는 별도 보존한다.

### S0-p107 — 새 하네스와 제품 소스 경계로 통합 경로 수정

목표의 “new gate on BASE0 against itself”를 다시 대조했다. BASE0 제품의 동일성을 지키는 것과 과거 gate 스크립트를 그대로 실행하는 것은 다르다. 독립 검토에 따라 새 하네스 H·제품 P·명시적인 관측 패치 O를 구분하고, 원래 전체 실행 순서와 공통 단계 journal에서 실제 결과를 검증하는 경로를 채택한다. 단계마다 별도의 인계 타입을 계속 추가하는 것은 필수 조건이 아니다. 실제 세계 관측 완전성·동일성·native 종료·성능·stall 판정은 계속 요구한다.

검토된 M1 successorV2와 resolver32 준비본은 루트 합동37개(474.966초)·타입0오류·Ruff를 통과했지만 새 경로의 필수 변경으로 채택하지 않았다. 원래 외부 패치와 검증 증거를 보존한 뒤 정확한 역패치로 미커밋 루트 변경만 제거했다. BASE0 과거 M1 셸 호환 V1은 별도 마지막 callback 후 증거 변경 결함이 확인돼 CHANGES_REQUIRED로 보존하며 후속 수정을 보류했다. 이는 기존 M1V2 검토 결과와 다른 준비본이다.

V9는 원래 입력 admission과 잠금 획득 후 남은 listing314개를 실제 실행 중이다. 아직 완료/전체 집계 통과를 주장하지 않는다. 기존 V9 FAIL/UNSETTLED와 엄격한 BASE0 자기검증 KnowledgeLost FAIL은 그대로다. 결정: D0-new-harness-product-boundary. 원격 main/wip는 게시된 Stage−1 `2f6993009ddb600e9249c28f9768a1a5252c8821`이며 Stage0 게시가 아니다.

### S0-p108 — 원래 ASCII 명령의 관측 환경 연결

단계23의 원래 ASCII 명령이 일반 probe와 같은 `WORLDSIM_GATE_OBSERVER=1` 설정으로 관측을 켜도록 기존 parser에 연결했다. 기본 미설정은 꺼짐이며 CLI와 환경을 함께 지정해도 중복 flag로 처리하지 않는다. 잘못된 환경값·실제 중복 flag·출력 경로 누락은 거부한다. 기존 관측 schema와 replay 경로를 재사용하고 시뮬레이션 정책·데이터는 수정하지 않았다.

루트 기존 debug cache에서 실제 Rust ASCII 검사8개가 통과했다(실제 Cargo 종료0, 검사6.80초, 전체13.591초). 기존 U-script 렌더 출력 byte equality, full-save/fingerprint/명령 상태 동일성, 중단 endpoint와 실제 DebugKill 경계 검사도 포함한다. 제한된 rustfmt 검사와 diff 검사를 통과했다. 전체 baseline identity·전체 clippy·Godot/renderer paired qualification·전체 gate는 이 검사로 입증하지 않는다. 증거: `stage0/ordered-dispatch/whole-gate-ascii-observer-native-01/{native.log,started.json,terminal.json}`.

V9 후속 listing의 실제314개 종료 영수증은 모두 PASS/0이고 출력 해시와 일치한다. 마지막 자식은2026-11-03 05:14:45 UTC에 끝났으나 소유자 PID66725 및 실행 세션87478은 더 이상 없고 최종 RESULT/OWNER_TERMINAL/UNKNOWN·집계 보고서가 없다. 소유자 종료 코드와 종료 원인은 UNKNOWN이며 완료로 복구하지 않는다. 원래 V9 FAIL/UNSETTLED도 그대로다. 독립 감사: `stage0/workspace-runtime/v9-accounting-successor-v3/ROOT_TERMINAL_AUDIT_20261103.json` SHA256 `c045c31784fb60cd271466c66cc7b77e370b19d5ceaa10e502cdd300877aa007`. 06:20 UTC 원격 확인에서 main/wip는 Stage−1 게시 SHA 그대로이며 Stage0 자기검증 KnowledgeLost FAIL은 미해결이다.

### p108 전체 baseline identity 및 독립 준비 검증

고정 HEAD `ff617e06a1d283b64a361c8a4440a2329703dfbd`에서 기존 cache를 사용한 workspace 전체 대상 check·clippy·release probe 빌드와 표준 baseline identity가 모두 실제 종료0으로 끝났다. identity는 게시된 Stage−1 final-d-03 데이터를 기준으로228,748개 공통 값에서 차이0개·한쪽에만 있는 경로0개, chronicle64개/19,683항목 동일, 샘플64개 동일, seeds0–3 결정성을 확인했다. 표준 identity의 기존 git_sha 및 제거된0값 키 제외 규칙을 그대로 사용했다. native 소스·기준 데이터·identity 하네스의 전후 해시도 동일하다. 전체 gate나 colony 자기검증 KnowledgeLost 실패를 대체하지 않는다. 증거: `stage0/development-foundation-p108/{commands.json,result.json,identity.log}`; 실행 소유자 실제 종료0·잠금 반환 확인.

새 전체 게이트 준비본의 독립 검사에서는 A 체크포인트38개(19.802초), 기존 테스트 이관67개(21.272초), B V3 판정부13개(1.765초)가 실제 종료0으로 통과했다. 각각 별도 동결 준비본 검사이며 합쳐진 전체 gate의 실행 증거가 아니다. 증거: `stage0/ordered-dispatch/whole-gate-root-prepared-validation-v1.json`. UI·native·성능·stall의 남은 경로를 계속 연결하고 있으며 Stage0은 게시하지 않았다.


### S0-p109 — 새 전체 게이트 실행 경로 통합

새 기본 `gate.sh`가 소스가 고정된 selector를 받아 독립 BASE 제품과 후보 제품에서 원래 단계 순서를 실행하고, 실제 프로세스 종료와 관측 원본을 공통 journal로 연결하도록 통합했다. BASE 참조31단계와 후보51단계는 별개 실행이며, BASE의 참조 timing 결과와 후보 stall 예외를 구분한다. 기존 native/UI history 검증은 공통 순수 함수로 분리해 기존 호출 경로도 유지했다. 과거 gate 전체 해시를 검사하던 진단 테스트는 정확한 p107 스크립트 fixture를 사용하며 기존 소스 변조 검사를 남겼다.

현재 통합 작업 트리의 집중130개(54.179초), 기존 검증기 회귀199개(127.011초), 과거 계약67개(21.660초)가 실제 종료0으로 통과했다. 첫 외부 합동 준비본124개(50.052초)와 이 결과는 서로 다른 구성이다. 별도 검토에서44→45 anchor 키, bench의 실제 단계 receipt 및 runtime 상대 경로, 변경 없는 원본 normalizer의 승인 경로 불일치를 찾아 수정했다. 루트는 실제 셸의 여섯 acceptance 출력 경로에 남은 `..` 때문에 엄격한 경로 검사가 거부하는 문제를 재현한 뒤 고쳤다. 후속 bench·경로61개(14.500초)가 실제 종료0으로 통과했다. 관측 없는 작은 벤치 fixture는 성공으로 바꾸지 않고 예상된 FAIL을 보존한다.

이 구성은 전체 제품 gate가 아니다. 실제 play21 연결·일부 native 관측·외부 저장 이력의 출처·선택된 전체 O 패치의 중립성 qualification이 남으며, source materialization 자체는 build 증명이 아니다. 제품 qualification이 없거나 관측 경로가 미완료이면 최종 PASS를 내지 않는다. 엄격한 BASE0 자기 비교에서 이미 확인한 CORE-off KnowledgeLost FAIL은 그대로이고 Stage0은 게시하지 않는다. 통합 증거는 `stage0/ordered-dispatch/whole-gate-root-integration-v1.json`, 개별 `whole-gate-root-*-01.log`, `whole-gate-root-bench-paths-02.log`, `whole-gate-combined-review.md`에 보존한다.

최종 통합에서 원본 fixture 환경의 닫힌 입력 검사를 추가했고 M1의 실제 네 replay 관측을 연결했다. 후속 source/native/bench101개(29.666초)와 엄격한 타입·Ruff 검사가 통과했다. 이 과정의 타입 import 순환과 파일 열거 순서에 의존한 negative test 실패는 별도 로그로 남긴 뒤 수정했다. native negative는 실제 `first` branch를 선택해 변조하도록 고쳤으며 생산 판정기는 약화하지 않았다. 전체 Python discovery와 실제 제품 실행은 다음 검증으로 남는다.


### S0-p110 — 기존 appearance 테스트 관측과 엄격한 저장 이력 판정

원래 appearance 테스트의 여섯 assertion 집합과 명령·진행 시간을 유지하면서 다섯 Watch endpoint와 두 renderer 관측을 연결했다. 저장 tick2000의 부모·자식 이력과 실제 kin 첫 성공 시점을 그대로 사용한다. 실제 release 테스트는 관측 꺼짐·켜짐 각각 여섯 개가 종료0으로 통과했다. 관측 파일은 실제 다섯 endpoint·두 mirror 및 원래 세 명령 checkpoint와 일치했다. 이 결과는 독립 BASE 제품의 관측 중립성이나 전체 workspace 관측 완전성을 입증하지 않는다.

외부 저장의 candidate 전체 기록에 KnowledgeLost가 있으면 출처 조회 전부터 실패하도록 판정을 연결했다. v6/v7/v9의 확인 가능한 원래 소스 출처를 묶고, v8/v10의 미확인 출처는 미완료로 남긴다. founders를 새로 만들지 않으며 후보 생존 수가 같은 horizon의 BASE 이상인 경우만 보수적인 생존 증거로 인정한다. 기존 migration fixture에 새 reader 소스 목록이 빠진 실패3건을 보존하고 fixture 입력을 보충한 뒤 기존·strict·origin 합동12개가 실제 종료0으로 통과했다.

UI projection의 무조건 import가 기존 작은 launcher fixture를 깨뜨린 문제를 관측 기능 활성 시에만 import하도록 수정했다. 기존 owner/source 및 새 projection·migration 합동31개가 실제 종료0으로 통과했다. p109 공개 진입점은 양쪽 Cargo cache가 같아 준비 단계에서 실패했다. 독립 cache와 별도 제품 root를 만든 후속 준비본을 보존했으며 실제 재실행은 별도 기록한다. 원래 실패를 성공으로 덮어쓰지 않는다. 엄격한 BASE0 CORE-off KnowledgeLost 실패와 Stage0 미게시 상태는 유지한다.

증거: `stage0/ordered-dispatch/whole-gate-root-p110-progress-01.json`, `whole-gate-appearance-native-01`, `whole-gate-root-migrations-03.log`, `whole-gate-native-draft/appearance-integration/actual-shapes.json`, `stage0/whole-gate-prepared-p109-r2`. 전체 Python 회귀·baseline identity·전체 gate 결과는 각각 실제 종료 후 별도로 기록한다.


play21은 새 전체 게이트 session에서 원래 여섯 사례와 실제 자식16회를 실행하는 owner에 연결했다. main의 원래 bridge 빌드와 관측 제품 bridge의 출처를 분리하고, m1/v7도 원래 빌드 후 명시적인 관측 export만 사용한다. Git 원본 worktree는 관측 패치로 덮어쓰지 않는다. 실행 준비 여부와 관측 자격을 구분해 `ui.play` 경로는 호출 가능하지만, 수집 결과는 미완료 coverage로만 남기며 일반 세계 안전성이나 완료 판정에 더하지 않는다. 루트 관련40개 검사(40.156초)와 엄격한 타입 검사는 실제 종료0으로 통과했다. 실제 Godot/Cargo play21 실행과 과거 관측 중립성은 미검증이다.


play 연결의 독립 검토에서 상위 제품의 Cargo target 상속, 명시적 HOME 누락, UNKNOWN 진단 파일 쓰기 실패 시 owner 종료 문제를 확인했다. 각 원본 worktree의 상대 `target`을 명시하고 세션 HOME을 검증하며, 진단 쓰기 성공 여부와 관계없이 미정리 owner·잠금을 유지하도록 고쳤다. 수정 후 루트52개 검사(52.099초)와 독립 격리4개가 통과했다. 작은 실제 파일 쓰기로 후보 제품 bridge가 덮어써지지 않음을 확인했고, 진단 OSError 주입으로 실제 FD 유지도 확인했다. 실제 Cargo/Godot 검증으로 대신 계산하지 않는다. 검토: `stage0/ordered-dispatch/whole-gate-a-draft/play-v9-independent-review.md` SHA256 `65911bc4e005cebe73dd8660856e44069879b48f7d9feef63b2f47482cf7c955`; 루트 로그 `whole-gate-root-play-v9-05.log`.


p110 검증 작업 트리에서 workspace 전체 대상 check·clippy·release probe 빌드·표준 baseline identity가 모두 실제 종료0으로 끝났다. 게시된 Stage−1과 공통228,748값 차이0·한쪽 경로0, chronicle64개/19,683항목 동일, 표준 git_sha·기존0값 제외 후 샘플64개 동일, seeds0–3 결정성이 통과했다. native 소스·기준 파일·identity 하네스 전후 해시가 동일하다. 증거: `stage0/development-foundation-p110/{commands.json,result.json,identity.log}`. 기존 cache를 사용한 개발 검증이며 전체 gate·독립 제품 관측 중립성·CORE-off 자기검증 FAIL을 대체하지 않는다. 최종 엄격한 타입0오류·Ruff·셸 문법·diff 및 제한된 docs 검사도 통과했다.


### S0-p111 — 원래 workspace 결과와 추가 관측 연결

원래 workspace 실행의154 native 대상·5 doctest 집합을 실제 aggregate 출력과 소스에 묶었다. ASCII6개 검사의8개 세계, knowledge4개와 presentation-default2개 세계를 기존 명령·assertion을 유지한 채 관측한다. 실제 첫 성공 시점이 다른 carrier 비교는 horizon을 바꾸지 않고 미완료로 남긴다. bridge API13·resolver32의 원래 검사도 별도로 연결했으며 모든40 실행 경로는 호출 가능하지만 전체 안전성·관측 중립성 입증은 아직 미완료다.

p110 공개 진입점은 새 관측 파일의 부모 디렉터리가 아직 없다는 이유로 overlay 적용 전에 실패했다. 명시적인 추가 파일에만 없는 부모를 허용하고 기존 symlink·경로 탈출 검사를 유지했다. 실패 제품과 로그를 보존했으며 후속 합동95개 검사는 실제 종료0으로 통과했다. workspace 연결 후 옛 합성 fixture 세 건이 새 입력 계약을 충족하지 못한 실패도 보존하고, 해당 테스트의 독립 seam만 보충한 후94개가 통과했다. 엄격한 타입 검사0오류다.

첫 native 검증은 관측 꺼짐18개 통과 후, 관측 켜짐 appearance에서 이전 sidecar 충돌로 실패했다. 기존 파일의 생성 시점이 이번 실행 이전임을 확인하고, 관측 켜짐 출력만 명시적 외부 실행 디렉터리로 분리했다. 수정 후 Python 관련52개가 실제 종료0으로 통과했다. 새 경로 두 곳의 native 연속 재실행·workspace check/clippy·baseline identity는 실행 중이며, 완료 전에는 성공으로 계산하지 않는다. 원래 실패와 Stage0 CORE-off KnowledgeLost FAIL은 유지한다.

증거: `stage0/development-foundation-p111/{commands.json,result.json,DEBUG_JOURNAL.md}`, 후속 `development-foundation-p111-r2`, `stage0/ordered-dispatch/whole-gate-root-p111-integration-02.log`, `whole-gate-root-overlay-parents-01.log`, `whole-gate-root-p111-isolation-python.log`.


p111-r2 실제 native 재실행은 관측 꺼짐18개·켜짐18개 두 회 모두 종료0이었다. workspace 전체 대상 check·clippy·release probe 빌드도 종료0이다. 별도 원본 관측 감사는 각 회19개 완전한 Watch 이력과 renderer2개, 서로 다른 출력 경로를 확인했다. 이 감사는 source/history 검사이며 독립 BASE 중립성 증명이 아니다. 추가 대상 세 파일의 명시적 overlay 허용 목록 누락을 실패3건으로 재현해 고친 뒤 build·workspace·ASCII·knowledge 합동95개가 종료0으로 통과했다. 이후 테스트 형식 정리 후9개도 종료0이었다.

독립 검토를 통과한 관측 중립성 reader/comparator를 선택적 증거 입력으로 연결했다. 원래 제품·동일 파생 제품의 관측 꺼짐·켜짐 세 모드에 대해 실제 build/process 영수증과 원본 산출물을 다시 읽는다. 허용된 timing 키 밖의 state 손실, boolean/integer 혼동, 실행 후 normalizer 변조를 거부한다. O61개·비교 경로51개 중22개 비교기가 구현됐고29개는 미지원으로 남는다. 불완전한 보고서나 일반 gate 성공 로그로 PRODUCT_QUALIFIED가 되지 않는다. 루트41개 검사가 종료0(5.153초)으로 통과했으며 실제 전체 qualification은 NOT_RUN이다. 증거: `whole-gate-root-qualification-v5.log`, `whole-gate-qualification-adoption-v5/ADOPTION_RESULT_V5.md`, `whole-gate-comparator-review-v4/REVIEW.md`.


workspace의 파서 전용2개·빈 Phase2/scaffold/수동 tree identity3개 대상, 원래21개 검사에 한정한 소스 분류도 추가했다. 첫 검토에서 직접 factory/FFI 소스 pin 누락을 찾아 보충한 V2가 독립 검토를 통과했다. 해당 다섯 대상만 일반 세계 미완료 목록에서 제외하며 원래 aggregate 검사와 다른 세계·doctest 의무는 유지한다. 루트 통합26개 검사 종료0(0.090초). 증거: `whole-gate-workspace-classification-review/REVIEW_V2.md`, `whole-gate-root-classification-v2.log`. 전체 native workspace 실행·안전성 PASS로 확대하지 않는다.


p111-r2의 마지막 baseline identity도 실제 종료0으로 완료됐다. Stage−1 기준 공통228,748값 차이0·한쪽 경로0, chronicle64개/19,683항목 동일, 표준 git_sha·기존0값 제외 후 샘플64개 동일, seeds0–3 결정성 통과다. native 소스·기준 자료·identity 하네스 전후 해시가 동일하고 실행 소유자 종료0·잠금 반환을 확인했다. 실제 검증한 것은 커밋 전 p111 작업 트리이며 출력의 git_sha 라벨은 당시 부모 HEAD8000f5f다. 전체 gate나 별도 제품 중립성 증명은 아니다. 최종 구성 Python74개(5.545초), import 정리 후 qualification38개, 엄격한 타입0오류·Ruff(명시적 public reexport 규칙 제외)·rustfmt·diff·제한된 docs 검사도 통과했다.

### S0-p112 — 렌더러 원본 비교와 빈 bridge 세계 분류

렌더러 원본 산출물 세 개를 제외 키 없이 비교하는 순수 비교기를 추가했다. seed100·tick900, 인물 위치와 외형, settled, 명령 수락 여부를 함께 검사한다. 독립 검토가 찾은 tick900 경계와 역순 명령의 공통 오류를 재현해 거부하도록 고쳤다. 보존된 과거 원본 fixture는 현재 native 실행 증거가 아니며 Godot·debug bridge 생산 경로 인증은 남아 있다. qualification matrix의 미지원 상태는 유지한다.

engine_bridge_ffi의 원래22개 사례는 빈 세계에서 BuildingStampSystem만 수동 실행하는 소스 분류로 연결했다. 여섯 직접 소스 pin과 정확한 사례 집합을 요구하며 일반 세계 안전성으로 계산하지 않는다. 루트 통합39개 검사 종료0(0.126초), 추가 Python 파일의 엄격한 타입0오류를 확인했다. 증거는 `stage0/ordered-dispatch/whole-gate-root-p112-integration-02.log`, `whole-gate-root-p112-types.log`, `whole-gate-native-draft/p112-renderer-review/v2-evidence.json`, `whole-gate-engine-bridge-review/CLASSIFIER_REVIEW_V2.md`다.

동결 p109 전체 Python discovery는 2,328개 실행(4,544.006초) 후 import 오류2개·skip5개로 종료1이었다. tools/v8에서 PYTHONPATH 없이 실행해 `test_mux_safety`와 `test_mux_score`가 `tools` 패키지를 찾지 못했다. 원래 FAIL을 보존하고 두 모듈의 경로 보정 검증을 별도 기록한다. p111 공개 전체 게이트는 별도 동결 H와 독립 BASE/후보 제품으로 실행 중이다. BASE bridge 빌드·Godot import는 실제 종료0이며 debug workspace는 아직 실행 중이다. Stage0은 미게시이고 엄격한 CORE-off KnowledgeLost 실패는 유지한다.

후속 검증은 같은 동결 p109에 명시적 repository PYTHONPATH를 사용해 해당 두 모듈만 실행했고19개(0.800초) 종료0이었다. 원본 입력430개 해시가 전후 동일하다. 원래 discovery의 argv/cwd/env는 원본 영수증에 없어 실행 당시 기록과 오류 로그를 구분했으며, 보충 성공을 전체 suite 성공으로 바꾸지 않는다. 증거: `stage0/ordered-dispatch/python-p109-mux-import-rerun/REVIEW.md`. p112 최종 통합39개와 변경 파일 Ruff·diff 검사는 통과했다. 이 변경은 Python 판정기·테스트·문서뿐이며 Rust/data는 p111과 동일하다. p111-r2 identity 증거를 보존하며 새 native identity 실행으로 표현하지 않는다.
