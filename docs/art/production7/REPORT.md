## 구현 완료 보고

### 구현 의도

여러 마을과 분열·노출 사망이 생긴 game_v17에서 화면이 단일 마을과 네 가지 사망만 설명하던 문제를 바로잡았다. **Part 0–9 수행, 화면 기준25/30 PASS·5 FAIL, 전체 게이트 RED**이며 초록 브랜치는 이동하지 않는다.

### 구현 내용

- Part0: 시뮬레이션 Pass9–11 정확한 커밋을 무충돌 병합하고 두 시드의120일·실제 FFI 측정을 수행했다.
- Part1: UI의 단일 마을 가정39항목 감사와 화덕의 정확한 형성 타일 대응.
- Part2: 명시적5종 사망 합계·색·영구 알림 로그. 화면 알림은4줄 유지.
- Part3: Rust 읽기 전용 `get_society_snapshot()`와24필드 수집·변환·자체 테스트 추가.
- Part4: 생애 고정 마을 번호·부모 표시·색·선택·소속 테두리·신의 손 대상·폐촌 표현.
- Part5:13명 이주 무리의 출발/창설 알림, 점선·목적지 깃발·짐 표시.
- Part6: 출입구 입김과 바람 장작 효과를 각각 한 번 수정했다. 군집 가림·아침 떨림은 그대로 재측정했다.
- Part7: 최종 소스28f58e56에서 ART94·ART_LONG27·SOCIETY17·GOD12·CALIBRATION4장, 총154쌍을 촬영·해시 검증했다.
- Part8: 독립 판독·블라인드·원본 크기 보드·A1–A30 측정. 측정 코드 검토2건을 고치고10개 회귀 테스트를 통과했다.
- Part9: 전체 게이트·남은 오류·용량·호환성을 확인하고 보고서와 PR 공개 증거를 정리했다.

### 구현 방법

화덕은 가장 가까운 마을 대신 형성 타일이 정확히 같은 마을만 참조한다. 대응이 없는 화덕은 꺼진 불·재·연기 없음으로 처리한다. 사망은 기아/탈수/전투/노화/노출의5개 키를 명시하고, 노출 색은 `(0.70,0.84,0.97)`, 미지 사유는 회색이다. 동일 위치·tick의 복수 사망도 발생 순번으로 구별한다.

`SettlementSelection`이 번호·계보·선택을 공유한다. 이름표는12화면px·어두운 배경판·정적 지붕 회피를 사용하고, 선택된 주민만1px 테두리를 얻는다. 신의 손은 선택 id, 미선택 시 기존-1을 전달하며 적용 시점 대상을 토스트에 고정한다. 이주자는 실제 Founding 상태와 로그를 읽어 점선·깃발·짐을 그린다. 실제 운반·식사·수면과 겹치는 짐은 숨긴다. 시뮬레이션 규칙과 데이터를 꾸며 넣지 않았다.

폐촌 화덕은 retired 타일로 정확히 식별한다. 주변 건물의 회색 표현은 소유권 필드 부재 때문에5타일 이내 유일하게 가까운 역사적 형성 타일을 이용하는 제한된 휴리스틱이다. 폐촌 알림 tick은 실제 사건 시점이 아닌 최초 관측 시점이다.

### 실제 게임 화면 검증

최종 [수치 원문](measurements.json), [30행 요약](measurement_summary.json), [관찰 기록](visual_observations.json)을 연결했다. A1–A25는 기본 시드 자연 장면을 기준으로 하고, A24는 기본 시드 개입, A26은 시나리오별 사망, A27–A30은 사회 시나리오를 검사한다. A18 사회 결과는 별도로 유지했다.

| 항목 | 결과 | 수치와 범위 |
|---|---|---|
| A1 | PASS | 자연117/117 통과. 전체121장 진단은 보정1장 휘도0.0394<0.04로 실패 |
| A2 | PASS | 4×·6× 두 보정 화면 혼합 픽셀0 |
| A3 | PASS | 그리기 영역1,309개 오류0; 성인 정면 idle4프레임 |
| A4 | PASS | 4×·6× 북/남 4조건의 픽셀 일치율 각각100% |
| A5 | PASS | 대비 중앙값0.3454/0.3107; 측정 가능68.18%/85.19%; 이전 진단은 실패 보존 |
| A6 | PASS | 낮/밤 휘도0.3124/0.1560; 켜진 화덕1개의 주변0.2554 |
| A7 | PASS | 이동 표본231개, 변화율100%; 연속3.967초 |
| A8 | PASS | 116장 검사, 오류0 |
| A9 | PASS | 117장 검사, 오류0 |
| A10 | PASS | 실시간 시뮬레이션300프레임: 중앙값8.228ms, p95 9.502ms |
| A11 | PASS | 두 관찰자 각각22/24(91.67%), 4×·2× 동일; 운반3/3. 보정 장면 한정 |
| A12 | PASS | 주민22명/외형18종; 동일 인접0/6 |
| A13 | PASS | 24개 체형·동작 조합; 최저 실루엣 차이0.1566 |
| A14 | PASS | 숲221/873타일(25.315%); 중복0, 검사322개 |
| A15 | PASS | 배치546개·데칼13종 검사 |
| A16 | PASS | 겨울/여름 청적 비율1.1678≥1.03; 노인6명 머리색 일치. 구 규칙 진단 실패 보존 |
| A17 | PASS | 화덕74·고사목855·HUD117; 실제 운반6표본/5명, fallback0 |
| A18 | FAIL | 자연23/117장 실패, 최악21/29명 가림(72.41%). 사회10/17장 실패, 최악27/36(75%) |
| A19 | PASS | 표시 냉기5명·전체 주기2개·목표 라벨72개; 오류0 |
| A20 | PASS | 설치 프레임74개·애니메이션20그룹; 오류0 |
| A21 | PASS | 겨울14장 검사 오류0; 원본 주기13개 중 독립 관찰9개에서 반복 입김 확인 |
| A22 | FAIL | 아침3장 중2장 불일치; 독립 식별 떨림 주민0명 |
| A23 | PASS | HUD117장 검사 오류0 |
| A24 | FAIL | 개입8장 중 식량 썩히기·장작 주기2장 세계 효과 미식별; 패널 확인만 보임 |
| A25 | PASS | 9장 검사 오류0 |
| A26 | PASS | 146장 합계·알림·색 오류0; 화면 안 노출사망 표식3개 |
| A27 | PASS | 다중 마을12장 오류0; 두 관찰자 각각4/4 |
| A28 | FAIL | 이동5장 중3장 목적지 표식이 화면 밖. 짐 유지, 두 관찰자 목적 판독 각각2/2 |
| A29 | PASS | 딸 마을 비4장·대상 토스트1장·같은 화면 화덕 대조1장; 오류0 |
| A30 | FAIL | 300+300프레임 중앙값9.7695→15.828ms(+62.01%, 허용+15%); 감사37/39종결,2미해결 |

기본 자연117장(시드20261010), 사회 자연17장(20261013), 개입12장(기본8·사회4), 일반 보정8장을 섞지 않았다. 별도의5사유 보정은 실제 production 렌더러·HUD·알림에 합성 입력을 준 창 촬영이며, 화면에 SYNTHETIC을 명시했다. 자연 발생5종을 보았다는 뜻이 아니다. 자연 사망 census는 기본36명(노화34·노출2), 사회44명(노화41·노출3)이며 개입 세계와도 따로 센다.

A11은 원래 동결된24개 질문 이미지와 최종 픽셀이 동일함을 검증했다. A27/A28은 새 질문·정답 해시를 먼저 동결하고 새 관찰자 D/E가 각각4/4·2/2를 맞혔다. 형식 예시로 오염 우려가 있던 첫 사회 관찰자 답은 점수에서 제외했다. 최종 독립 리뷰 B/C는 마을·이주·딸 마을 추운 밤·5색 보정 모두 **ACCEPT**였다. 첫 회 이름표 **NEEDS WORK**는 그대로 보존하고 한 번 수정 후 재검토만 했다.

| §14 직접 확인 | 결과와 근거 |
|---|---|
| 1. game_v17 시작·콘솔 | 정확한 시뮬레이션 병합, 최종5계획 로그 오류0, smoke 통과 |
| 2. 노출 사망 합계·알림·색 | A26 통과, 실제 화면 표식3개.5색 전체는 합성 보정 범위 |
| 3. 이주·점선·깃발·짐·창설 | 출발13명/창설 로그 일치, 짐 유지. A28은3개 리더 근접 화면의 화면 밖 목적지로 실패 |
| 4. 마을 이름·색·선택 | A27 통과, 선택 전/원래 마을/딸 마을 보드와 클릭·테두리 장면 테스트 |
| 5. 딸 마을에만 비 | A29 통과. 같은 프레임에서 딸 화덕의 재와 원래 마을의 불을 대조 |
| 6. 폐촌 회색·연기 없음 | 고아 화덕·폐촌 합성 테스트 통과. 자연120일에 폐촌0건이므로 자연 화면 수용은 미확인 |
| 7. v17 겨울·아침·가을 | 측정된 날짜로 장기27장 재촬영, [MEASURED_DAYS](MEASURED_DAYS.md) |
| 8. 오류·3마을 성능 | 최종 촬영 SCRIPT ERROR0. A30 성능은 실패 |

보드: [마을](villages.png), [이주·창설](founding.png), [사망](deaths.png), [기본 전후](before-after-live.png), [사회 전후](before-after-society.png). 각 보드는 원본1× 픽셀을 유지하며 [출처·해시](artboard_sources.json)를 제공한다. [리뷰 보드 목록](review/board_sources.json), [HUD 보드](review/hud_batches.json), [냉기 주기 보드](review/cycle_contacts.json), [B 최종 리뷰](review/independent_b_round2.json), [C 최종 리뷰](review/independent_c_round2.json).

### 시뮬레이션 작업과의 호환

병합한 시뮬레이션은 **31559232e0a24af12d29f490d440d1c5ee5f3661** 하나이며 이후 시뮬레이션은 가져오지 않았다. Rust 변경은 `world_node.rs`210줄 추가·삭제0줄의 읽기 전용 getter/수집기/변환기/테스트뿐이다. 다음 작업은 `Settlement.parent/founded_at/member_agents`, `Founding`, `fission_log`, `retired_hearths`와 새24필드 FFI 계약을 보존해야 한다. 로스터 id는 정렬하고 기존 signed i64 전송 규칙을 유지한다.

기본 시드20261010은120일에 분열0회다. 냉한 밤은28·56·84일과 이어지는29·57·85일, 가을 실제 운반은15·16일에 확인했다. 노출 사망은57일03:00와85일04:00이며1분 뒤를 촬영했다. 사회 시드20261013은13명씩 두 번 분열한다:29일06:00 출발→08:09 창설(tick40200→40329),85일06:00→08:41(tick120840→121001). 첫 딸 마을 겨울51일은 온화해84일 추운 밤도 별도로 촬영했다. 취소·폐촌·합병은 자연120일 관측0건이다.

최종 촬영 소스는28f58e56, 전체 게이트 소스는847f96b9, 계획 테스트 후속 소스는90d8e6dc다. 그 사이 production 렌더링·시뮬레이션 코드는 동일하며 후속은 테스트/문서만 바뀌었다. [소스 동등성](gates/final/source_equivalence.json)을 남겼다.

### 변경된 파일 목록

커밋별 정확한 파일 경로는 [CHANGE_INVENTORY.md](CHANGE_INVENTORY.md)에 있다. 핵심은 `scripts/rendering/`, `scripts/ui/`, `scripts/test/art/`, en/ko 지역화, 읽기 전용 `world_node.rs`, `tools/art_pipeline/*_p7.py`, 새 `docs/art/production7/`과 TEST_LEDGER다. `scenes/main.tscn`은 바이트 동일하며 load_steps21이다.

| Part | 커밋 | 변경 목적 |
|---|---|---|
| 0 | `e1e894ef217350be8f1c456f2665836199a7358f` | bring in simulation passes 9 to 11 |
| 0 | `7efbf61b045393c67d54eb463e0f760c073b9fd0` | preserve the execution contract and merge provenance |
| 0 | `07e148dfe8d1960d16076209b40e91ed3bf0d926` | make scenario captures preserve seed and settlement identity |
| 0 | `164e93f7a99c4424826bbc67dcafe2e56757926c` | bind v17 measurements and untouched-screen baselines |
| 1 | `6b013bfd26fcbeb9c97ba78b973020f33bce1a74` | prevent abandoned hearths from borrowing a neighbouring fire |
| 2 | `cff5691b820d8fbf7e8537fd750e9ba2cdfd6aca` | count and explain every death without losing simultaneous victims |
| 3 | `deef001bc7420efd8e45e03de32350e4d9138f0f` | expose village lineage without changing simulation state |
| 4 | `b2b28db8c9581d59dfa42d93113eec7b0994f338` | make village identity and intervention targets visible |
| 4 | `c84981b22dc8ba5860a04f0ba9674135e3c468bc` | hide the single-village counter without hiding the clock |
| 5 | `c4a253b2d9f8b404f10c175a596c09e6ce1cb5a9` | show why a group leaves and when its village is founded |
| 6 | `509262294e56957b25251a5dcd6efbd27b0a316d` | give doorway breath and falling wood one clearer visual attempt |
| 7 | `0f634fec92381315538d865c53ff44b88998c157` | bind captures to measured society and death evidence |
| 7 | `eb690d377ec2fe3fc69c06ede4843c0c9fa93005` | measure exposure closeup cycles without missing evidence |
| 7 | `28f58e56a2289bde129946b7f630192cd94496b2` | make village lineage readable after independent review |
| 7 | `369f2200406233e6c32454a4389b5296f4ea8276` | preserve final screen evidence and rejected capture cases |
| 8 | `847f96b928352cd48d7e5e49bf3c748ee8cfffdc` | distinguish visible acceptance from measured failures |
| 9 | `90d8e6dc85153b842b48e315603f36f77948f2a5` | align inherited plan checks with measured v17 worlds |

이 보고서·최종 게이트·공개 확인을 담는 마지막 문서 커밋의 해시는 최종 응답과 원격 WIP HEAD에 기록한다. 기존 로컬 `.gitignore`·`.ignore` 설정은 보존하고 작업 커밋에 넣지 않았다.

### 자율 결정 목록

[DECISIONS.md](DECISIONS.md)에 D01–D27을 기록했다. 핵심 결정은 정확한 시뮬레이션만 병합, 허용 밖 Rust 픽스처를 수정하지 않기, 불가능한 출발+4시간 요청을 실패로 보존, 자연/사회/개입/합성 보정 분리, 폐촌 건물 휴리스틱과 최초 관측 tick 명시, 원래 실패 판정 보존, 이름표 한 번 수정·한 번 재검토, 블라인드 오염 답 제외, 측정 검증 강화, 용량 감소 시 원본 보존이다. 이미지 생성은 **0/4회**다.

### 확인된 제한사항

- 첫 마을이 생기기 전 몇 틱 동안 캠프 화덕이 찬 재로 보임
- A18 군집 가림, A22 아침 떨림, A24 식량/장작 개입의 세계 효과, A28 화면 밖 목적지, A30 성능은 실패다.
- 감사39항목은 fixed24·justified13·pending2(U08/U09)다. 측정 파이프라인 완료가 성능 문제 해결을 뜻하지 않는다.
- A30은 동일 실행·zoom1·동결 시뮬레이션에서 연속300+300프레임이다. 호스트1분 부하가176.70→36.20으로 변했으므로 통제된 동일 부하 비교가 아니며, 증가분을 UI나 시뮬레이션 한쪽 탓으로 단정하지 않는다. 실시간 시뮬레이션 비용의 증거도 아니다.
- 지정된+4시간 FD는 이미 창설 후라 살아 있는 founder가 없다는 명확한 오류로 종료했다. 실제+1시간 장면을 보충했으며 존재하지 않는 이주 상태를 만들지 않았다.
- A11은 보정 활동 판독,5색 판정은 합성 보정이다. 자연 기아·탈수·전투 사망이나 자연 폐촌을 확인한 것으로 확대하지 않는다.
- 이름표는 원본1920×1080에서 수용됐으며 더 작은 화면의 수용을 주장하지 않는다. 지붕 회피는 모든 카메라 위치의 무겹침 보장이 아니다.
- 숨겨진 F2A 죽음 그리기 캐시에 잠재적 오래된 값 위험이 있지만 이번 F2A는 사망0건이어서 현재 근거를 오염시키지 않았다. 일부 실시간 census 간격은11tick으로,8tick이라고 주장하지 않았다.

### Harness 결과

**전체 게이트 RED.** [원본53행 결과](gates/final/results.tsv)는50 PASS/3 FAIL이다. Rust `cargo test --workspace`와 `clippy --workspace --all-targets -- -D warnings`는 기존 `InterventionRecord.soaked_wood` 누락으로 각각101 종료했다. 테스트 실행 전 컴파일 실패라 이 전체 실행의 Rust 통과 수를 만들지 않았다. 앞서 실행한 sim-bridge 제외 보충 검사는140개 테스트 묶음·2,057개 통과/0실패였으나 최종 전체 게이트 대체 증거가 아니다.

strict·production bridge·smoke·smoke 오류 검사·Python unittest275개는 통과했다. Godot 장면46개 중 최초45개 통과/1개 실패였다. 오래된21장 기대값을 정확한27장·RESET 계약으로 수정하고 **커밋90d8e6dc에서 해당 장면과 scoped strict를 재실행해 통과**했다. [후속 로그](gates/final/followup/results.tsv)를 원래 실패와 분리했다. 측정 도구 pytest10개도 독립 재실행 통과다. 남은 오류는 Rust2건이며 전체 게이트가 초록이라고 보고하지 않는다.

용량은 [budget.json](budget.json)에서 최종 바이트를 확인한다: assets/art25MB 이하·production7 30MB 이하. 공개 GIF는 시간 표본만 절반으로 줄이고 선택된 픽셀과 전체 재생 시간을 그대로 유지했다. 원본 측정 주기는 외부 근거 폴더에 남아 있다.

WIP의 문서 공개 전 확인 HEAD는90d8e6dc85153b842b48e315603f36f77948f2a5, 초록 브랜치는cd998d3807358521ead77b0a7314e93fd8283878이다. 각 push 뒤 `git ls-remote` 일치를 확인했다. Rust RED 때문에 초록 브랜치를 이동하지 않았다. PR #511의 head는 초록 브랜치이므로 Part7 WIP 변경이 PR 코드 차이에 실렸다고 주장하지 않는다.

[PR #511](https://github.com/hyunlord/new-world/pull/511) 설명에 Art Pass7을 추가했고 다시 조회해 본문 일치·기존 본문 보존·head/base·draft 유지를 확인했다. [공개 확인](PR_PUBLICATION.json)을 남겼다.

### 정직한 판단

처음 보는 두 관찰자는 화면만 보고 마을 수와 계보, 무리가 새 마을을 세우러 이동한다는 목적을 모두 맞혔다. 최종 화면 검토도 두 명 모두 ACCEPT였다. 따라서 이 핵심 전달은 확인됐다. 그러나 근접 화면에서 목적지까지 항상 보이거나, 혼잡한 주민을 구별하거나, 떨림과 모든 개입 효과를 읽거나, 세 마을의 성능 기준을 만족하는 단계는 아니다. **25개 통과와5개 실패, Rust 게이트 RED를 함께 남기는 부분 수용 결과**다.
