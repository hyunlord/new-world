## 구현 완료 보고

**제품 빌드 동결: `5ee5e1ee98df24a16756360b8b125d582f68fdd0`, 2026-10-16 15:25:03.021297 UTC.** Part 0–4의 절차는 종료됐고 최적화 채택은 0건이다. H7은 NOT_VERIFIED, H8은 FAIL이다. 전체 결과는 **TIMEBOXED_PARTIAL**이다. 최종 matrix는 완료됐지만 후 구간 타이머는 마감 전 측정에 진입하지 못해 NOT_MEASURED다. Part 5의 전체 gate1·리뷰·정리는 완료했다. 최종 커밋의 게이트 대상 소스 동일성 및 원격·PR 결과는 본문 커밋 후 출판 영수증에 기록한다. 문서 커밋 목록은 `cbc4cbd0`까지 반영했다.

### 구현 의도

이름·가족·생애·사망 기록으로 개인의 역사를 읽게 하고, 일곱 낮 활동과 마을 요약의 가독성을 독립 판독으로 검증한다. 500명 화면의 프레임 예산은 release 실측과 인구 유지 조건으로 판단한다.

### 구현 내용

시뮬레이션 Pass 18–19를 병합해 game_v24와 사망 뒤에도 남는 person/chronicle을 받았다. 전달된 getter 5종은 모두 존재하며, `found` 추가 필드, person에 partner 없음, AgentId와 entity bits의 구분, 최대 512행 페이지를 실제 계약으로 기록했다.
이름·가족 링크·역할별 Life·고인 기록·C 연대기 패널을 구현했다. EN/KO 신규 키 36개를 Fluent 2파일·compiled 2파일·registry 총 5파일에 추가하고 기존 제목·가족 키를 재사용했다. 일곱 활동 아이콘은 기존 Craft U1_04와 절차 생성 6종이며 이미지 생성은 0회다. zoom<2에서는 도착한 활동의 상위 3종을 마을별로 표시한다.
측정에서 가장 큰 declump 구간을 대상으로 완전한 입력값 캐시를 시험했다. 후보는 동등성 검사를 통과했지만 속도가 개선되지 않아 기각·원복했다. 화면 밖 생략은 구현하지 않았고, 과거 tick-cache 실패는 진단만 했으며 재채택하지 않았다. Art Pass 10의 선택적 debug 사후 측정은 필수 최종 측정 시간을 확보하기 위해 DROPPED다.

### 구현 방법

measure 작업트리에서 release bridge를 만들고 debug 로드 경로와 SHA 일치를 강제했다. 각 실행 블록은 reset→advance→재프레이밍으로 새로 시작했다. 종료 인구가 시작의 90% 미만이면 표시하고 H4 통계에서 제외한다. UI 촬영은 debug이며 성능 증거로 사용하지 않는다.
인물 원시 결과를 getter·AgentId·tick별로 캐시하고 언어 변경 시 문장만 다시 만든다. 120개 고정 tick 프레임에서 getter당 최대 1회 호출을 확인했고, 프레임별 `get_agent_detail`은 제외했다. Born의 부모 관점, Paired의 역방향과 원인별 문장은 전달 계약에 맞춰 생성한다.
블라인드는 P10 원래 crop 크기와 14개·10개 이상 정답·어느 활동도 0/2 금지 기준을 유지했다. 후보의 여섯 정지 프레이밍은 전체 draw clock 및 무마스크 RGBA로, 네 실행·팬 사례는 지정된 수동 tick 일정으로 비교했다. frozen 제품은 P2와 같은 소스로 복귀했다. 후보에서 측정한 H5와 최종 빌드의 소스 동일성은 서로 다른 근거다.

### 기능 설명

가족·생애 링크는 생존자를 선택하거나 고인 기록을 연다. 고인은 이름·사망 원인·가족·기질·생애를 표시하고 생존자용 초상·욕구·위치를 숨긴다. 생애가 12줄을 넘으면 펼칠 수 있다. C 연대기와 인과 창은 서로 닫히며, 큰 tick 점프 뒤 멈춘 연대기는 backlog가 남을 수 있다. 활동 아이콘은 zoom≥2, 마을 요약은 zoom<2에서 표시한다.

### 변경된 파일 목록

31개 커밋의 전체 변경 경로·상태·시각은 [commit-files.md](commit-files.md)와 [commit-files.json](commit-files.json)에 있다. simulation 병합 파일과 기각 후보의 추가·삭제도 보존했다. 이 본문 자체의 커밋 해시와 후속 출판 결과는 본문 커밋 후 출판 영수증에 기록한다.

| Part | 전체 커밋 해시 | 커밋 시각(+09:00) | 의도 |
|---|---|---|---|
| 0 | `5cdf233409f11afdfde1134565730e21e8807a39` | 2026-10-16T16:52:28+09:00 | art-p11(p0): bring in simulation passes 18 and 19 |
| 0 | `ecf512d886d87ba7f9ca5fa7db92a48300dc1c68` | 2026-10-16T16:53:35+09:00 | art-p11(p0): preserve the execution contract and launch provenance |
| 0 | `fe79480544133ebbecac72a23c9383df00f16762` | 2026-10-16T18:59:31+09:00 | art-p11(p0): bind release baselines to fresh worlds and verified binaries |
| 0 | `e68b851b26869671446b9542b72b077f2c01a1ef` | 2026-10-16T18:59:39+09:00 | art-p11(p0): keep the measurement runner whitespace-clean |
| 0 | `b94bfef736f24f6f9e0567c39bb5d7ce5950916f` | 2026-10-16T19:03:46+09:00 | art-p11(p0): bound quiet waits by the original task deadline |
| 0 | `a4bfd35b96a8cf18a2b7f3dcbdc2096c7e7d32a2` | 2026-10-16T19:09:28+09:00 | art-p11(p0): retain the green merged gate and live capture cohort |
| 1 | `01fa24dd4494f32b5eb0c9006700edd0d059b292` | 2026-10-16T19:20:15+09:00 | art-p11(p1): make each person and their surviving history readable |
| 1 | `327239189218223c0564bd9389c0d7fa7339a935` | 2026-10-16T19:26:46+09:00 | art-p11(p1): validate captures against the absolute bridge clock |
| 2 | `5686e4dc355bf8a83e17ef52473d9df786c2ab57` | 2026-10-16T19:36:42+09:00 | art-p11(p2): distinguish daily work at close and village scales |
| 2 | `94cc19609f27af7cda20e7f67ba38ded5354cfed` | 2026-10-16T19:40:48+09:00 | art-p11(p2): retain pose-oracle inputs in the new capture driver |
| 1 | `5da8229140aebd45d3a4e9d87e3a90beecf9e293` | 2026-10-16T19:48:47+09:00 | art-p11(p1): preserve the independently read life story and its failed bar |
| 2 | `a27f60369037d586d00d38139f4feef4386b4c15` | 2026-10-16T19:54:55+09:00 | art-p11(p2): count foreground world sprites before selecting blind crops |
| 2 | `f20de87b79f65131f6cc4286edc0d23a8c520918` | 2026-10-16T19:58:52+09:00 | art-p11(p2): distinguish village identities across capture worlds |
| 2 | `08ef05d11db2c9c23796f59734dae240086ae625` | 2026-10-16T20:14:14+09:00 | art-p11(p2): preregister each supplemental blind-test capture |
| 2 | `cca7469929e2926dd399760e27ce893873ac0a9a` | 2026-10-16T21:00:42+09:00 | art-p11(p2): preserve blind-test bars when eligible samples are scarce |
| 2 | `f0d04f8e5bb698385b4751f0e0ea9f401ebef5ac` | 2026-10-16T21:36:13+09:00 | art-p11(p2): retain incomplete activity evidence without lowering the bar |
| 3 | `b72365aaed1118fad489ef7b17881cd82c262d76` | 2026-10-16T21:41:08+09:00 | art-p11(p3): control render steps before judging unchanged pixels |
| 1 | `d275f2e7bb90875f594394f35134d42617cd2578` | 2026-10-16T21:54:14+09:00 | art-p11(p1): preserve inspector checks after adding the chronicle |
| 3 | `1aaad0225d6e72000fd42b1495c4d317eba52d0a` | 2026-10-16T21:54:48+09:00 | art-p11(p3): account for every renderer step in identity captures |
| 1 | `90045db04807850be0c5c5c2cb792251c2575da8` | 2026-10-16T22:02:57+09:00 | art-p11(p1): retain execution proof for unchanged static contracts |
| 3 | `dad5c5e092163de32c1e2810122ea07921f5ed6b` | 2026-10-16T22:06:58+09:00 | art-p11(p3): require the stress reference to reproduce exactly |
| 3 | `6a3a2ee0865a39a5333bd08e9b2f230a226094b2` | 2026-10-16T22:21:55+09:00 | art-p11(p3): include exact stress checks in gate discovery |
| 3 | `d74f740f5de78617967c912627e84b1b758ce799` | 2026-10-16T22:42:20+09:00 | art-p11(p3): establish reproducible unmasked identity controls |
| 0 | `0635f950204b67d186017e3c07196e019ba03d37` | 2026-10-16T23:11:20+09:00 | art-p11(p0): anchor optimization to the complete release baseline |
| 3 | `1dc641b5cd161cf9683349f367194935e8213b3f` | 2026-10-16T23:15:15+09:00 | art-p11(p3): avoid recomputing identical declump inputs |
| 3 | `5ee5e1ee98df24a16756360b8b125d582f68fdd0` | 2026-10-17T00:24:36+09:00 | art-p11(p3): reject slower cache before freezing measurements |
| 3 | `d8cd2a300de87c4245c030f42c6c98e9377ff32f` | 2026-10-17T00:26:31+09:00 | art-p11(p3): preserve rejected speed result and complete identity evidence |
| 4 | `068a97adddc9540ff816f47cea68bfb204d3c531` | 2026-10-17T00:27:03+09:00 | art-p11(p4): freeze proven runtime before the final release matrix |
| 4 | `519d3916a1c7cd12857f33d6d1442d203a2f1735` | 2026-10-17T02:15:13+09:00 | art-p11(p4): preserve historical baselines and interrupted measurement provenance |
| 4 | `382d9c3e5e0ffc74ca884487195778512926c253` | 2026-10-17T05:15:40+09:00 | art-p11(p4): retain final measurements without hiding missed bars |
| 5 | `cbc4cbd05242de67e671116bf01c552c879c2d47` | 2026-10-17T05:15:59+09:00 | art-p11(p5): preserve independent review and the green full gate |

### 측정 결과

네 세계 matrix가 완료되어 원본 표본으로 전후 값을 독립 재검증했다. 후측정 구간 타이머는 원래19:50Z 마감 전에 quiet hold에 진입하지 못했다. session exit124, summary invalid·frames 없음, driver exit1을 보존하며 NOT_MEASURED로 확정한다. 네 세계 matrix의 유효 결과와 구간 수집 실패를 분리한다. H5의 PASS는 기각 후보1dc의 측정 결과다. frozen5ee는 P2 제품 소스와 동일하지만 5ee에서 B를 다시 촬영했다고 주장하지 않는다.

| H | 전 | 후 | 배수(전÷후) | 판정·한계 |
|---|---|---|---|---|
| H1 W0 정지 Z4 |2.8750ms |3.3705ms |0.8530× |PASS, median≤6ms; 후 세션 contended |
| H2 W200 정지 Z4 |16.2800ms |16.3175ms, p99 17.5150ms |0.9977× |FAIL, median12ms·p99 16.67ms 초과 |
| H3 W500 정지 Z4 |28.9180ms |28.9335ms |0.9995× |FAIL, median16.67ms 초과 |
| H4 W500 실행2× |적격0/3 |적격0/3 |N/A |NOT_VERIFIED, 인구 감소 표시 블록 제외 |
| H5 동등성 |P2 A/A′ |후보1dc 정지6·실행/팬4 PASS |N/A |측정 후보 PASS; frozen5ee 제품=P2 소스, 새5ee 캡처 아님 |
| H6 W500 AgentRenderer |24.8000ms |24.7665ms |1.0014× |REPORT_ONLY; 별도 합격 기준 없음 |
| H7 활동 |C12/14 확보 |진단0/12→1/12;D2/6→5/6 |N/A |NOT_VERIFIED; 원래 C 기준 유지 |
| H8 생애 |기능 테스트 통과 |44/45 정확 |N/A |FAIL; 모든 직접 문장 정확 조건 미달 |

Art Pass 10 debug 사후 실험은 14:00Z에 제외했다. 과거 total 구간 47.2085ms는 debug·contended이며, alternating 29.04ms도 미실행 최종 matrix를 대체하지 않는다. TIMEBOXED_PARTIAL과 과거 tick-cache REJECTED는 유지한다.

| Art Pass10(debug) | 전 중앙값 ms | 후 | 배수 | 판정 |
|---|---:|---|---|---|
| H1 W0 frozen Z4 | 4.2845 | NOT_MEASURED | N/A | NOT_VERIFIED |
| H1 W0 frozen Z1 | 4.4150 | NOT_MEASURED | N/A | NOT_VERIFIED |
| H3 W500 frozen Z4 | 51.3875 | NOT_MEASURED | N/A | NOT_VERIFIED |
| H3 W500 frozen Z1 | 51.6745 | NOT_MEASURED | N/A | NOT_VERIFIED |
| H6 W500 AgentRenderer ablation | 45.8745 | NOT_MEASURED | N/A | NOT_MEASURED |

**과거 before는 존재하며 after/final만0세션이다.** `docs/art/production10/perf/baseline.json`의 원시 표본을 독립 재계산해 baseline.md와 세계별 요약을 확인했다. debug commit `0caabf5fd1c62b9d3c99845daab1805e79799db7`, bridge SHA `f41bd469ac5d2b67c7f50375189e306a7872513566140f096bf8716c513faa1c`다. W0은 contended=true(load6.26→3.48), W500은 false(2.45→2.73)였다. 조건별900표본의 process_ms+render_cpu_ms를 집계했고, H6은 인접 all-on300표본 median51.2510−off150표본 median5.3765=45.8745ms다. 세부 백분위·출처는 `perf/art10_after.md`에 보존했다. 과거 문서와 verdict는 변경하지 않았다.

네 세계 release 전후 측정: primary=process_ms+render_cpu_ms, 단위 ms, 각 조건 3×300프레임, nearest-rank percentile. 기준선 원본은 `perf/baseline-raw-evidence.zip`, 요약은 `perf/baseline.md`다. 최종 패키지는 [perf/final-evidence/README.md](perf/final-evidence/README.md)와 [원본 ZIP](perf/final-evidence/raw-evidence.zip)에 보존하며 원시·집계 일치와 payload 해시를 검증하고 `382d9c3e`에 커밋했다. 선택 세계 W1000은 미측정이다.

| 조건 | 전 중앙값/p90/p99/최대 ms | 후 중앙값/p90/p99/최대 ms |
|---|---|---|
| W0/frozen/Z4/0x | 2.8750 / 3.1540 / 3.7550 / 4.1150 | 3.3705 / 3.6130 / 4.2610 / 4.5730 |
| W0/frozen/Z1/0x | 2.8395 / 3.0810 / 3.4440 / 5.1470 | 3.4605 / 3.7070 / 4.4510 / 4.6080 |
| W0/running/Z4/2x | 5.1480 / 5.5580 / 5.8850 / 6.1990 | 5.5600 / 5.8710 / 6.1560 / 6.6740 |
| W0/running/Z4/4x | 5.4685 / 5.8300 / 6.1300 / 6.6080 | 5.8615 / 6.3920 / 6.7860 / 7.7180 |
| W1/frozen/Z4/0x | 5.8210 / 6.1750 / 6.7640 / 7.2030 | 6.7770 / 7.7480 / 9.2220 / 10.2510 |
| W1/frozen/Z1/0x | 5.9700 / 6.4200 / 6.8570 / 7.3600 | 6.5745 / 7.6820 / 9.0910 / 9.8620 |
| W1/running/Z4/2x | 10.1760 / 10.7670 / 11.1570 / 11.4380 | 12.0290 / 16.9330 / 19.3370 / 20.7550 |
| W1/running/Z4/4x | 10.7480 / 11.5290 / 12.3990 / 13.8020 | 11.3625 / 13.0060 / 14.7430 / 15.9440 |
| W200/frozen/Z4/0x | 16.2800 / 16.9130 / 17.8140 / 19.4240 | 16.3175 / 16.8840 / 17.5150 / 17.9130 |
| W200/frozen/Z1/0x | 16.5115 / 17.0440 / 17.4980 / 18.1130 | 16.4460 / 16.9760 / 17.6460 / 18.2850 |
| W200/running/Z4/2x | 26.5125 / 28.3060 / 30.0540 / 63.8910 | 26.5225 / 28.5020 / 30.0010 / 63.5710 |
| W200/running/Z4/4x | 26.6315 / 28.6220 / 30.1880 / 30.9850 | 26.7880 / 29.0770 / 30.5140 / 31.5480 |
| W500/frozen/Z4/0x | 28.9180 / 29.8670 / 30.6750 / 31.4120 | 28.9335 / 29.9630 / 31.0020 / 31.7940 |
| W500/frozen/Z1/0x | 28.8775 / 29.5920 / 30.5180 / 31.6140 | 28.7550 / 29.5290 / 30.2450 / 31.1890 |
| W500/running/Z4/2x | 81.5925 / 93.8730 / 111.3990 / 116.9210 | 82.2805 / 94.5110 / 110.8990 / 117.7510 |
| W500/running/Z4/4x | 81.6680 / 93.7670 / 112.2460 / 118.1990 | 82.3270 / 93.9550 / 113.1740 / 117.9400 |

실행 블록은 새 세계에서 시작했다. 표의 각 행은0/1/2의 세 블록이며, 다른 값은 같은 순서의 슬래시 배열로 보존한다. 정지 블록은 새 reset 대상이 아니다. 후 W0·W1은 contended, 후 W200·W500은 uncontended다. 세션 간 부하 차이와 후보 미채택을 고려해 배수를 코드 최적화 효과로 해석하지 않는다.

| 세계·블록(각0/1/2) | 인구(전 / 후, 각각 시작→끝) | 전/후 표시 | 새 설정 | 전 ticks/frame | 후 ticks/frame | 전/후 tick 프레임 |
|---|---|---|---|---|---|---|
| W0/frozen_Z4 | 29→29 / 29→29 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W0/frozen_Z1 | 29→29 / 29→29 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W0/running_2x | 29→29 / 29→29 | False/False | True/True | 1.0000 | 1.0000 | 300/300 |
| W0/running_4x | 29→29 / 29→29 | False/False | True/True | 2.0000 | 2.0000 | 300/300 |
| W500/frozen_Z4 | 500→500 / 500→500 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W500/frozen_Z1 | 500→500 / 500→500 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W500/running_2x | 500→392 / 500→392 | True/True | True/True | 4.8800/4.8933/4.9067 | 4.9100/4.9100/4.8900 | 300/300 |
| W500/running_4x | 500→363 / 500→363 | True/True | True/True | 5.0000 | 5.0000 | 300/300 |
| W1/frozen_Z4 | 66→66 / 66→66 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W1/frozen_Z1 | 66→66 / 66→66 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W1/running_2x | 66→66 / 66→66 | False/False | True/True | 1.0000 | 1.0233/1.0567/1.0000 | 전[300,300,300] / 후[298,298,300] |
| W1/running_4x | 66→66 / 66→66 | False/False | True/True | 2.0000 | 2.0000/2.0000/1.9967 | 300/300 |
| W200/frozen_Z4 | 200→200 / 200→200 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W200/frozen_Z1 | 200→200 / 200→200 | False/False | False/False | 0.0000 | 0.0000 | 0/0 |
| W200/running_2x | 200→200 / 200→200 | False/False | True/True | 1.6933/1.6967/1.6933 | 1.7000/1.6967/1.7000 | 300/300 |
| W200/running_4x | 200→200 / 200→200 | False/False | True/True | 3.3767/3.3867/3.3867 | 3.4000/3.4167/3.4167 | 300/300 |

W500의 2× 3블록은 모두500→392, 4× 3블록은 모두500→363으로 감소했다. 표시된 값을 숨기지 않고 H4 적격 통계에서 제외했다. 적격 2× 블록은0/3이며 기준선 H4는 NOT_VERIFIED다. 최종 W500 역시 적격0/3이어서 H4는 NOT_VERIFIED다.
W500 구간 타이머는 release·frozen Z4의300표본이다. 세션은14:09:44–14:09:58Z, uncontended, exit0이다. 단위는 μs이며 별도 founding-group callback은 이 표에 포함되지 않는다. 아래 후 값 NOT_MEASURED는 마감 전 수집 미진입에 따른 결측이며0이 아니다. 구간 전후 배수도 계산하지 않는다.
후측정 구간은 전부 NOT_MEASURED다. 아래는 전 중앙값/p90/p99/최대(μs)를 빠짐없이 보존한다.

| 구간 | 전 값 | 구간 | 전 값 |
|---|---|---|---|
| activity_context_usec | 83.0000 / 90 / 96 / 100 | agent_setup_usec | 590.0000 / 621 / 654 / 681 |
| blocked_usec | 20.0000 / 22 / 24 / 25 | causal_usec | 2.0000 / 2 / 3 / 3 |
| checksum_talk_usec | 31.0000 / 35 / 37 / 39 | children_usec | 1312.0000 / 1341 / 1366 / 1387 |
| chill_ingest_usec | 201.0000 / 208 / 216 / 217 | declump_usec | 6676.5000 / 6837 / 6967 / 6997 |
| draw_rows_usec | 546.0000 / 561 / 575 / 577 | easing_usec | 912.0000 / 1005 / 1120 / 1203 |
| footprints_usec | 683.5000 / 745 / 827 / 904 | frame_choice_usec | 1985.0000 / 2041 / 2081 / 2128 |
| look_usec | 1229.0000 / 1261 / 1288 / 1312 | meta_usec | 1264.0000 / 1296 / 1332 / 1402 |
| other_usec | 69.0000 / 77 / 87 / 90 | pose_usec | 2550.0000 / 2596 / 2631 / 2713 |
| post_loop_usec | 2112.0000 / 2198 / 2273 / 2346 | reads_usec | 50.0000 / 55 / 60 / 66 |
| retire_usec | 110.0000 / 133 / 142 / 149 | seats_usec | 657.5000 / 729 / 827 / 929 |
| selection_usec | 6.0000 / 7 / 8 / 8 | tile_contexts_usec | 647.0000 / 667 / 678 / 691 |
| total_usec | 24923.0000 / 25502 / 26172 / 26240 | variant_usec | 1002.0000 / 1027 / 1048 / 1051 |
| walker_apply_usec | 257.0000 / 298 / 391 / 505 | walker_offsets_usec | 1894.5000 / 1969 / 2042 / 2094 |

노드 비용은 인접 all-on 300프레임 median−off 150프레임 median이다. 음수를 그대로 보존하며 구간 타이머와 혼동하지 않는다.

| 노드/그룹 | W0 전→후 ms | W500 전→후 ms |
|---|---:|---:|
| ActivityTrailRenderer | 0.0540→0.1400 | 0.5990→0.4685 |
| AgentRenderer | 1.5225→1.6545 | 24.8000→24.7665 |
| AgentRenderer/FoundingGroups | 0.0095→0.0880 | 0.1430→0.0190 |
| BuildingRenderer | 0.1000→0.0415 | 0.2305→0.0950 |
| BuildingRenderer/NightChillLayer | -0.0470→0.0060 | 0.1430→-0.1410 |
| DaylightController | 0.0150→-0.0125 | 0.0705→0.1820 |
| DeathVizRenderer | -0.0335→-0.1105 | -0.0665→-0.0490 |
| GodHandEffects | 0.1410→0.0970 | 0.0355→0.1105 |
| NatureRenderer | 0.3555→0.4450 | 1.6030→1.3090 |
| NeedBarRenderer | 0.0610→-0.0825 | 0.1660→0.3790 |
| SeekVizRenderer | 0.0190→-0.0035 | -0.0925→0.1125 |
| SettlementLabels | 0.4550→0.4220 | 0.5885→0.5665 |
| SettlementOverviewRenderer | 0.0210→0.0965 | -0.0835→-0.0770 |
| Shadows | -0.0370→0.1240 | -0.2675→0.3040 |
| TerrainRenderer | -0.0000→-0.0245 | 0.1960→0.0515 |
| UI/AgentInspectorPanel | -0.3425→-0.1795 | 0.0440→-0.0325 |
| UI/ChroniclePanel | NOT_MEASURED→0.0215 | NOT_MEASURED→0.1070 |
| UI/GodHandPanel | 0.0205→-0.1680 | 0.1560→-0.0920 |
| UI/HudStatusPanel | -0.1335→0.0280 | 0.1675→-0.0730 |
| UI/HudTopbar | 0.3215→0.0035 | 0.2235→0.1035 |
| WorldEffects | 0.0610→-0.0120 | 0.8890→0.6260 |
| WorldObjects | 0.0650→0.0825 | 0.1025→0.3140 |
| WorldRenderer | 0.0175→0.0680 | -0.1860→0.0505 |
| ZoomLodController | 0.1795→-0.0670 | 0.0975→0.1675 |


활동 판독은 두 라운드 모두 유지한다. C의 원래 기준은14개 중10개 이상이면서 어느 활동도0/2가 아니어야 한다. 실제 확보는12개뿐이고, 충분히 확보한 여러 활동도0/2다. 부족한 TendFire·Explore 각1개를 조작하거나 분모를 환산하지 않았다.

| 활동 | 확보/필요 | R1 정답 | R2 정답 |
|---|---:|---:|---:|
| TendFire | 1/2 | 0 | 0 |
| Craft | 2/2 | 0 | 0 |
| Repair | 2/2 | 0 | 0 |
| HearthCircle | 2/2 | 0 | 0 |
| Explore | 1/2 | 0 | 1 |
| Play | 2/2 | 0 | 0 |
| SitByFire | 2/2 | 0 | 0 |
| C 진단합계 | 12/14 | 0/12 | 1/12 |
| D 마을요약 | 6/6 | 2/6 FAIL | 5/6 PASS |

15개 유효 캡처의503 holder에서 pose 불일치는0이었다. C의129개 제외는 crop 전에 기록했고 원래 크기를 유지했다. D는 고정4시점을 사전등록해6쌍을 확보했다. 가림 판정은 상위 world Sprite2D 사각형의 합집합이며 alpha·UI 가림의 완전한 증명이 아니다. D의 두 번째 PASS는 C의 부족과 실패를 해소하지 않는다.

| 생애 판독 | 값/판정 |
|---|---|
| 실제 화면 | seed20261010,60–90 일 31 일,EN/KO16 장 |
| 직접문장 | 44PASS /1FAIL /0NOT_VERIFIED |
| 불일치 | P3-S07 오후디 vs 원본 subject54 오후티 |
| DepartCause0–4 | 실제 Departed0 건,다섯모두 N/A |
| 단위검증 | 문장 249/249,생애 72/72,기존개인 63/63;이름·패널실패 0 |
| 이름충돌 EN/KO 동일 | 500 명 5 초과(1.00%);10000 명 1889 초과(18.89%) |

생애의 모든 직접 문장이 정확해야 하므로44/45도 H8 FAIL이다. 이름 충돌은 EN/KO가 같고, ID282의 금칙 접두사는 마지막 음절46회 소진 후 앞 음절로 carry하도록 첫 촬영 전에 기록했다. 실제60–90일에 Departed가 없어 다섯 원인은 모두 N/A다.
기각 후보는 W500 frozen Z4에서150프레임씩 세 교대쌍을 측정했다. 대조·후보 각각450표본이다.

| 후보 release 교대 | 대조 median ms | 후보 median ms | 결과 |
|---|---:|---:|---|
| pair1 | 39.1445 | 37.8680 | 단일 쌍만 개선 |
| pair2 | 63.8485 | 65.8330 | 악화 |
| pair3 | 64.7850 | 65.9935 | 악화 |
| pooled | 62.8585000098 | 63.9470 | **0.9829780914×, REJECTED** |

| 후보 pooled primary CPU(ms) | 대조 | 후보 |
|---|---:|---:|
| 중앙값 | 62.8585000098 | 63.9470000000 |
| p90 | 67.5499999943 | 70.0980000224 |
| p99 | 83.7340000000 | 86.5870000391 |
| 최대 | 119.2989999723 | 133.3989999666 |

전체 배수0.9829780913846646×로 개선되지 않았다. 15:20:34–15:21:51Z hold는 load111.32→51.64의 contended 상태였다. 조용한 호스트의 속도 추정으로 일반화하지 않고 후보를 REJECTED로 남겼다. [기각 실험 기록](/Users/rexxa/work/new-world-art-prod/docs/art/production11/perf/declump-input-cache-rejected/README.md)에 실제 파일 위치와 원본 ZIP을 보존했다. 보존 ZIP은 7,126,337바이트이며 SHA256은 `df49b17e75becac0e1251ebf73276375a17993821d19bde63de89326f66b701a`다. 독립 속도 validator의 `pixel_identity:NOT_VERIFIED`는 identity를 입력받지 않는 필드이며, 실제 identity 결과를 덮어쓰지 않았다.

| 후보 H5 검사 | 측정 결과 |
|---|---|
| P1–P6 A/A′ 및 A/B | 모든 프레이밍 RGBA 차이0·최대채널차이0·전체 draw clock 일치 |
| W0/W200 실행 | 각각600프레임 PASS |
| W500 정지/2× 팬 | 각각240프레임 PASS |
| 최종 frozen5ee 제품 소스 | 기준P2 f0d04f8e와 byte-identical; 5ee 재촬영 결과가 아님 |

### 환경

Mac Apple M4 Max14CPU·36GiB, macOS27.0.1(26A434), Godot4.6 stable89cea143, windowed1920×1080. cargo jobs6, test opt-level2를 사용했다. release 기준선과 동결 빌드의 양쪽 dylib SHA는 `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`다. UI 촬영 debug SHA는 `aa6da070ff17861f8dc21d8f6203643dec3d8c44a8d94b90525c388eb89cdd8a`다. GPU 보고0을 GPU 비용0으로 해석하지 않는다.
실제 freeze는15:25:03.021297Z이며 TASKS15:24:41은 앞선 결정 시각이다. release build/import는 각각exit0이다. `perf/frozen-build/source-equivalence.json`은 scripts·scenes·shaders·assets·localization·project·data·rust·gdextension 전체를 비교했다. 제품 변경 경로는0개이고 차이는 진단 스크립트5개·Rust 정적 테스트5개뿐이다. 넓은 diff는 비어 있지 않다는 사실도 보존했다.

### 독립 리뷰

리뷰 반복은 최대2라운드를 지켰다. 활동 R1의 C0/12·D2/6 실패와 R2의 C1/12·D5/6 결과를 모두 보존했다. 부족한14개 기준을 줄이거나 세 번째 판독으로 교체하지 않았다. 생애44/45 실패와 QA의 NEEDS WORK도 유지한다.

| 리뷰 | 결과 | 한계 |
|---|---|---|
| 생애 구현 R1 | 실제 계약·번역36키·역할·캐시·고인 기록 검토 PASS | H8 FAIL 유지; 실제 현재 HEAD 런타임 재실행 아님 |
| 블라인드 절차 R1 | archive268개 hash, 제외129개, crop12개 픽셀·두 라운드 점수 재확인 | 암호화된 초기 task 본문 전체의 무키 내용은 NOT_VERIFIED; 관찰된 누출 없음 |
| 보안 R1 | 경로·simulation 불변·argv·숫자 링크 PASS, critical/high 없음 | S1 LOW: identity runner 전체 lock 대기 제한 불완전; 다른 wrapper 수정으로 소급 해소하지 않음 |
| 보안 후보 R2 | 완전 입력 키·복사·원래 알고리즘·후속 갱신 검토 PASS | 성능 합격 아님; 후보는 이후 기각됨 |
| 시각 QA R1 |11장 원본 동일성·보이는 UI 확인 | NEEDS WORK; 실제 클릭·키 입력 검증 아님 |
| 시각 QA R2 |18개 A/A′/B 전체RGBA와 시계 독립 비교, 여섯 exact0 PASS | 전체 가독성 NEEDS WORK; 제3의 blind 라운드 아님 |
| quiet 구간 감사 |네 기준선 hold에서 보존된 기록상 task-owned runtime 중첩 미발견 | 일부 OS exit 시각·연속 감시 없음; 감사 당시 대기 중이던 이후 세션으로 확대하지 않음 |

리뷰 원본은 [reviews/manifest.json](reviews/manifest.json)에 SHA와 함께 보존했다. 두 번째 최종 증거 리뷰는 네 세계의 원시·집계 일치, 보고서 16조건 수치, ZIP 55개 payload의 해시를 확인하고 측정·출처·보고 범위에서 종료했다. 게이트·출판은 별도 검증하며 작업트리 정리는 아래 영수증으로 확인했다. 초기 생애 raw 누락7개 항목은 같은 bridge/seed/tick 재현으로 보완했지만 그 재현에는 자체HEAD stamp가 없다. B의 generic manifest git_sha도 비어 있어 후보 귀속은 주변 source snapshot·실행 기록을 따른다. A의 보조 provenance는 도중 기록이며 모든 launch 전후 감사가 아니다.

### 자율 결정 목록

- ID282의 종료 불가능한 금칙 접두사는46회 suffix 순환 뒤 carry로 처리했다. 금칙어를 줄이지 않았다.
- 최대512행·tick당1회 규율을 유지했다. 큰 점프 후 멈춘 feed의 backlog와 표시 부재를 제한으로 남겼다.
- P0 기준선 소스를 고정하고 UI 준비를 병행했다. quiet hold에서는 runtime을 별도 운영했고 남은 감시 한계도 공개했다.
- 고정 crop 위로 아이콘 대부분이 잘리는 기하를 사전 기록했지만 crop이나 판정 막대를 바꾸지 않았다. D 추가 시점은 사전등록했고 C 부족은 NOT_VERIFIED로 남겼다.
- 최대 구간 declump를 첫 후보로 선택했으나 기각했다. 선택한 첫 후보와 채택한 최적화를 혼동하지 않는다. 추가 최적화는 하지 않았다.
- 필수 최종 측정을 위해 선택 항목 Art10 after를 제외했다. 과거 verdict는 변경하지 않았다.

### 실행 기록

| Part | UTC 시작 | UTC 종료 | 상태·경과 |
|---|---|---|---|
| 0 |07:50 |14:09:58 |완료·6시간19분58초 |
| 1 |약10:17 |약10:47 |완료·약30분; H8 FAIL |
| 2 |10:35 |12:36 |완료·2시간1분; H7 NOT_VERIFIED |
| 3 |12:37 |15:24:41 종료 결정 |결정까지2시간47분41초; 후보 기각·채택0건. 제품 빌드 동결은22.021297초 뒤인15:25:03.021297 |
| 4 |15:24:41 결정 /15:25:03.021297 실제freeze |19:50 |실제 freeze부터4시간24분57초; matrix 완료·후 구간 NOT_MEASURED |
| 5 |19:50 공식 시작; 리뷰 준비는 앞서 병행 |본문 커밋 후 출판 영수증에 실제 종료 기록 |gate1 88/88·리뷰·정리 완료; 최종 소스 동일성·출판 결과는 본문 커밋 뒤 기록 |

| 출판·정리 검증 | 결과 |
|---|---|
| `git ls-remote origin refs/heads/codex/art-production-2026-10-wip` |본문 커밋 후 출판 영수증에 실제 조회 시각·전체 SHA 기록 |
| `git ls-remote origin refs/heads/codex/art-production-2026-10` |본문 커밋 후 출판 영수증에 실제 조회 시각·전체 SHA 기록 |
| [PR #511](https://github.com/hyunlord/new-world/pull/511) |본문 커밋 후 출판 영수증에 실제 갱신 제목·본문·시각 기록 |
| 작업트리·본 작업 소유 임시 프로세스 정리 |완료: base·measure 제거 확인. 19:54:06.869691Z 사전 점검에서 원본 보존 SHA·base manifest 일치·measure clean 확인. 작업 소유12:20 caffeinate 생존 프로세스 없음 |

정리 근거: `/Users/rexxa/work/art-production11-evidence/p5-runtime/cleanup-precheck.json`, `worktrees-after.txt`. 정리 후 목록에는 메인 작업트리와 기존 `art-production-audit`만 남는다. 기존07:51 caffeinate와 다른 소유자의 프로세스·작업트리는 건드리지 않았다.

### 확인된 제한사항

영어 고인 가족 행의 말줄임, 스크롤 가장자리 잘림, 한국어 이름 줄바꿈, 영어 화면의 한국어 자원 라벨이 남았다. 활동과 사람이 밀집해 대응이 어렵고, 실제 입력으로 링크·스크롤·Escape를 조작한 통합 검증은 없다. 속도1.0x는 UI 설정 표시만 확인했다.
후보의 deterministic 수동 tick identity는 모든 상태나 자연 실행 성능의 증명이 아니다. W500 전후 실행은 모두 인구 감소로 H4 부적격이다. 후 W0·W1의 혼잡 조건과 W500 attempt1 중단 기록을 보존한다. 동결 소스 동일성은 새 캡처가 아니며, 후보 exact0도 H7/H8 실패를 없애지 않는다. 후 구간 타이머 결측으로 renderer 내부 비용의 최종 전후 변화는 검증하지 못했다.

### 게이트 결과

P0 gate82개 단계는 모두exit0이다. 최초 cargo의 wall7695초와 UTC 불연속은 호스트 정지로 기록하고 성능 근거에서 제외했다. P1 문장249/249·생애72/72·기존개인63/63, strict48파일·FFI6호출을 확인했다. P2 활동 테스트 실패0·pose503개 불일치0이다. 재고정한 Rust 정적 테스트5개는 각각1 passed/0 failed이며 기존/수정 양쪽의 rustfmt 문제는 원본 로그로 보존했다.
후보 targeted7 checks와 comparator14 tests/5 subtests가 통과했다. 최종 gate1은 HEAD `519d3916a1c7cd12857f33d6d1442d203a2f1735`를 시작·종료 때 동일하게 확인했고, 19:50:19→20:15:01 UTC(24분42초)에 **88/88 단계 exit0**, 로그 오류 검색 0건으로 완료했다. [verification.json](gates/final/verification.json)에 원본 로그 해시와 결과를 보존했다. 최종 커밋의 게이트 대상 소스가 gate1 검증 대상과 동일한지 확인한 결과, 최종 본문 커밋 해시, 양 원격 헤드, PR 실제 갱신과 Part 5 종료 시각은 본문 커밋 후 출판 영수증에 기록한다. 현재 그 후속 절차의 완료를 주장하지 않는다. 원시 로그 두 파일과 작업트리 목록 한 파일의 EOF 빈 줄에 대한 diff-check 경고는 원본 로그와 해시를 보존하기 위해 그대로 남겼다. 테스트 성공을 가독성이나 성능 합격으로 확대하지 않는다.

### 정직한 판단

개인의 생애는45문장 중44문장을 정확히 읽었지만 H8 완전 정확 기준은 실패했다. 낮 활동은 설치됐으나 개인 활동 블라인드 인식이 부족해 H7도 완료하지 못했다. 최적화는0건 채택했고 release 기준선의 정지500명 중앙값은28.918ms다. 최종 release 정지200명은 median16.3175ms·p99 17.5150ms로 H2를 실패했고500명은28.9335ms로 H3를 실패했다. W0의29명은H1을 통과했지만 이 네 표본만으로 일반적인 지속 가능 인원 상한을 확정하지 않는다. 따라서 전체 결과는 TIMEBOXED_PARTIAL이다. 다음 아트 과제는 고정 crop에서도 구별되는 활동 형태, 밀집 화면의 인물 대응, 생애 문구의 줄바꿈·잘림 개선이다.
