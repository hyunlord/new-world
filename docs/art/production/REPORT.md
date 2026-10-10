## 구현 완료 보고

**기술 설치 및 증거 납품 완료, 시각 품질은 부분 납품(REVISE).** 전체 코드 게이트19개 항목이 통과했다. 실제 창92장에 대한 A1–A12 중10개는 통과했고 A5 대비·가림 및 A11 활동 식별은 실패했다. 변환기 §6.1 원시 JPEG·배음 복원도 미충족이다. 실패 기준을 완화하지 않았으며, 이 보고서의 제목은 아트 품질 전체 승인이라는 뜻이 아니다.

### 구현 의도

눌리고 흐릿한 주민, 복잡한 지면, 읽기 어려운 HUD를 D3 원본 비율의 픽셀 아트와 차분한 지면, 한국어 인터페이스로 바꾸되 시뮬레이션 사실은 변경하지 않는다.

### 구현 내용

| Part | 구현 및 증거 | 현재 판정 |
|---|---|---|
| 0 | 별도 `new-world-art-prod` 복제, 고정 기준 `0941e1b76ee250a017295fe923f7c0841dafaaa7`, 방향 커밋 `b5ead380c3eed047f6689224763d3713f7e5b02c` 병합, 제한시간·창 잠금·게이트 도구 | 기준 전체 게이트 통과 |
| 1 | 격자 변환기 v3, 116색 생산 팔레트, D3 16셀 재측정 및 설치 | 당시 테스트 통과. 아래 §6.1 요구와 시험 사이의 차이는 미해결 |
| 2 | 원본 크기 Sprite2D, 공통 발 기준 y 정렬, 정수 줌, 한국어·Galmuri, 캡처 도구 이식 | 전체 게이트 13개 항목 통과, 실제 창 10장 캡처 |
| 3 | 16종 시트 생성·셀별 수용 기록, 영역 아틀라스 준비 | 42회 생성 종료, 고유 수용 13셀·절차적 대체 243셀 설치 및 패킹 커밋 |
| 4 | `terrain_renderer.gd`의 6개 이중 격자 레이어와 지면 데칼, `nature_renderer.gd`의 결정적 자연물 배치 | 실제 창에 설치 확인; 수관 가림으로 A5 실패 |
| 5 | 활동·방향 추론, 프레임 선택, 팔레트 영역 셰이더, 구조물·광원·욕구 표시 | A7 애니메이션·A6 조명 통과, A11 식별 실패 |
| 6 | 상단 수치·아이콘, 하단 왼쪽 최근 4개 소식, 동일 색상 주민 초상, 욕구 막대, 인과 패널 | A9 실제 90개 장면의 한국어·글꼴·배치 통과 |
| 7–8 | 정량 QA, 식별 실험, 비교 그림·GIF, 변이 검사, 호환 시험, 최종 게이트·보고·초안 PR | 92장 캡처·A 검사 10/12 통과, A5·A11 실패. 변이474개 사멸, 호환 검증 및 코드 게이트19개 통과 |

### 구현 방법

변환기 v3는 셀의 격자 주기·위상을 추정하고, 네이티브 표본을 팔레트에 양자화한다. 충실도는 공통 불투명 픽셀에서 측정하며 색키 제거 수를 별도로 기록한다. D3 의미 검사는 열매 픽셀 수 79→18→7→0, 불꽃 33픽셀, 어린이 및 바구니 판정을 기록했다. 단일 포즈 D3 셀에 애니메이션 일관성 검사를 적용했다고 주장하지 않는다.

차분한 지면은 D3의 지정 Lab 변환과 채도 배율 0.72를 사용한다. 주민 색은 지면 변환에서 제외한다. 공유 WorldObjects 계층의 발·바닥 앵커로 겹침을 정렬하고, 그림자는 별도 계층에 둔다. 팔레트 변형은 영역 인덱스와 램프 텍스처로 피부 4×머리 5×옷 6 조합을 지정하며 색 곱셈 방식으로 주민을 물들이지 않는다. 활동 추론은 실제 스냅샷을 읽고 선택적 `target_kinds`·`life_stages` 유무를 구분한다.

**변환기 §6.1은 미충족이다.** `converter_literal_audit.md/json`에 기록된 9개 배율·위상 조합(5.3·5.9·6.4 × 0·1.7·3.1)은 PNG와 JPEG 모두 20×28 크기를 복원했다. PNG 색 복원은 ΔE76<3에서 100% 통과했다. 그러나 품질 88·4:2:0 JPEG의 원시 색 복원은 **35.71–60.00%**로 요구된 90%에 미달한다. 알려진 원본 팔레트로 양자화한 뒤 100%가 되는 기존 시험은 더 강한 입력 조건의 정규화 시험이며 원시 JPEG 복원 증거가 아니다.

4px 기본 주기에 강한 교대 경계를 넣은 반례는 주기 **7.9**, 네이티브 **10×15**를 반환해 요구된 4.0·20×28을 복원하지 못했다. 지정 argmax 점수는 강한 2s 배음을 일반적으로 배제하지 못한다. 추가 추정 팔레트나 임의 선택 규칙을 생산 코드에 넣어 성공으로 위장하지 않았다. 생산 크기 약 5.9px 시험의 통과 역시 이 반례를 해소하지 않는다. 독립 literal audit 명령은 현재 의도적으로 종료 코드 1을 반환한다. 일반 회귀 시험이 통과하더라도 이 아트 QA 실패는 남는다.

### 기능 설명

주민과 건물이 원본 픽셀 비율을 유지하고 공통 y 정렬에서 서로 가려진다. 줌은 1·2·3·4·6·8배를 사용한다. 한국어 HUD는 시간·인구·정착지·거처·공사·식량·열매·출생·사망을 실제 스냅샷에서 표시한다. 주민 정보창은 선택된 주민의 현재 프레임과 팔레트, 위치·상태·목표·욕구·운반 식량을 보여준다. 소식 패널은 기존 사건 수집·식량 변화 누적 로직을 유지한다. 생산 슬롯은 수용 셀과 절차적 대체로 채웠지만 활동별 애니메이션과 장식 생물의 시각적 완성도는 아직 확정하지 않았다.

### 생산 결과

생성 호출은 **42/60회 시작·완료**했다. 256개 생산 슬롯은 **고유 생성 수용 13셀 + 절차적 대체 243셀**, 기존 D3 16셀까지 합한 아틀라스는 **272프레임**이다. 시도 보고의 accepted 합계 26은 신규 13 + 이전 수용 보존 13이다. 미충족 슬롯 거절은 646회이며 보존 슬롯에서 새 후보가 실패한 경우까지 포함한 후보 실패는 659회다. [생산 결과](production_results.md)와 [셀별 기록](production_results.json)을 구분해 집계했다.

| 시트 | 호출 | 고유 생성 수용 | 절차적 대체 |
|---|---:|---:|---:|
| `V_a1` | 3 | 0 | 16 |
| `V_a2` | 3 | 0 | 16 |
| `V_a3` | 3 | 0 | 16 |
| `V_b1` | 3 | 2 | 14 |
| `V_b2` | 3 | 0 | 16 |
| `V_b3` | 3 | 0 | 16 |
| `V_c1` | 3 | 0 | 16 |
| `V_c2` | 3 | 0 | 16 |
| `S1` | 3 | 5 | 11 |
| `S2` | 3 | 0 | 16 |
| `N1` | 2 | 0 | 16 |
| `N2` | 2 | 3 | 13 |
| `N3` | 2 | 0 | 16 |
| `F1` | 2 | 2 | 14 |
| `F2` | 2 | 1 | 15 |
| `U1` | 2 | 0 | 16 |

초기 D3 16셀은 7개 수용·9개 임시 설치 상태다. 건축가 20×28, 노인 19×28, 가득 찬 덤불 25×22는 크기 기준을 초과하며 일부 셀은 ΔE 기준을 초과한다. 이를 정식 수용으로 세지 않는다. 생성 시트의 다수 거절은 크기·충실도 기준에 따른 것이며 시트별 상세 이유는 `sheets/*_attempt_*.json`에 있다. 요구한 1024×1024 대신 1254×1254가 반환된 초기 호출은 원본 그대로 보존하고 실제 격자를 측정했다.

설치 아틀라스는 2048×2048 `assets/art/atlas/world_0.png` 한 페이지에 272프레임을 포함한다. `fallback_install.json`은 대체 243셀별 규칙과 제한을 기록한다. 동일 방향의 수용 idle이 없는 주민은 D3 정면 기반 대체이므로 북·동 방향의 정체성은 미검증이다. 생성 수용으로 절차적 대체를 계산하지 않는다. `report_artifacts.json` 측정 시 자산은 1,431,470바이트, 보고 디렉터리는 38,619,319바이트로 각각 25MB·40MB 이내였다. 이후 보고·게이트 파일 증가분은 최종 한도 재확인 대상이다. 낮 GIF 1,662,108바이트, 저녁 GIF 2,777,023바이트는 각각 8MB 이내이며 각 40프레임·128색이다. 원본은 `/Users/rexxa/work/art-production-raw/`에 보존한다.

생산 native PNG 256개는 크기 예산 위반 0개다. 전체 PNG 538개와 manifest 538건에 대해 미등록·누락·중복 경로·비이진 알파·기록 크기 불일치가 모두 0이었다. 이는 목록·형식 검사이며 시각적 수용이 아니다. [런타임 자산 검토](asset_runtime_review.md)가 발견한 성인 수면 막대와 분리된 앉기 다리는 `253c71c6023e81ba5d52bdbdb9b791932be1cf36`에서 수정됐다. 수면 2셀은 수용된 `V_b1_16`의 13×28 북향 걷기 원형을 정확히 90도 회전한 28×14 모피 캔버스이며, 앉기·웅크림 7셀은 다리 두 행을 몸통 바로 아래에 연결했다. 성인 A의 체형·원래 방향 대체, 여러 방향의 동일 실루엣, 북향 걷기 폭 2텍셀 변화, 일부 걸음 발의 앵커 위 1텍셀 들림은 별도 제한으로 남는다.

### 변경된 파일 목록

발행 직전까지의 모든 `art-prod` 커밋을 전체 SHA로 기록했다. 실제 전체 변경 경로는 changed_files.txt와 각 커밋의 git show --stat에 있다. 최종 보고 발행 커밋 자체는 PR와 사용자 메시지에서 식별한다.

| Part | 전체 SHA | 의도 |
|---|---|---|
| 0 | `5a20542f2c32c94a2ffdb05ae22b63d3667d4b0a` | art-prod(p0): bring in the direction work |
| 0 | `9425778ded8d75d43813b16ce3322c6504c5bbd3` | art-prod(p0): establish isolated production baseline and bounded gates |
| 1 | `563b0f6c922ce78ecea5cb7df24f4aaeafbf01c0` | art-prod(p1): measure native sprites and derive the D3 production palette |
| 2 | `35401db7e4ee72d9d45f2a940f2d26171bcf1bb3` | art-prod(p2): install the native D3 world and verify its first real capture |
| 3 | `b350f91977c7d1011df04014de41dfdeebeadae1` | art-prod(p3): audit the first two character sheets without relaxing acceptance |
| 3 | `74cf74fdbe267c306277e31dc95fbc88ae4c3866` | art-prod(p3): retain accepted structures and establish paired palette regions |
| 3 | `784fdc0ac7d683507027918577612e555f4653cd` | art-prod(p3): audit the nature sheets and preserve the accepted flint cell |
| 3 | `21ab30e8a0be7f52334198aa0ccf8ea4ad378802` | art-prod(p3): record rejected carry and work sheets for bounded retries |
| 3 | `bb9cfcfdd27c6b691412a0c7f02c92aa2477e106` | art-prod(p3): audit camp and interface sheets within the evidence size budget |
| 3 | `0f155bf0e61589db57a51c2c6d7401c9292c5d10` | art-prod(p3): record effect and adult rest sheet quality results |
| 3 | `8786daabf3f01f52e9fd69f0304d85d7a776db23` | art-prod(p3): preserve adult and child rest sheet measurements |
| 3 | `cda2cd1f5e271ebd3dfddaebf52f1e48e1cbdd84` | art-prod(p3): measure decal and ambient creature candidates |
| 3 | `4957612d21647c3e15ba4b1d183ca9a6c67a00b2` | art-prod(p3): retain very-small interface and effect retries |
| 3 | `332b0fac6f78d49d1ef62abc45c6d66612f34864` | art-prod(p3): retain tiny decal and ambient retries |
| 3 | `1cdf5716a8ec47e126300a440d3657381d3481c5` | art-prod(p3): apply checkpoint prompt and expose converter audit gaps |
| 3 | `207ea67bc26315d5b8223888dd33360bddecb2b0` | art-prod(p3): close adult locomotion generation attempts |
| 4 | `fb0f461bc3d7789a58bdd37ab2bfcc54dd4528a0` | art-prod(p4): connect muted terrain and deterministic nature |
| 5 | `ab697f377b9f3bfd6690e403fe24bde1644c4671` | art-prod(p5): make resident activity and camp effects follow snapshots |
| 6 | `198e71764a093cbec25eb0d264a9f6de45af7478` | art-prod(p6): organize Korean pixel HUD and preserve source contracts |
| 3 | `9a5c191dc85a0952f0716afa9103c6fcf38bf681` | art-prod(p3): record child locomotion and structure retries |
| 3 | `43a4956de77c6db1a17d0c60caece62c9ddb1dab` | art-prod(p3): record nature retries without relaxing native limits |
| 7 | `0189eead423c0496bda04d93b2e2da57a3efea27` | art-prod(p7): prepare complete capture and assertion audit evidence |
| 3 | `6c35aa00c0ab7bd68523fec0ea40b112a672e5e8` | art-prod(p3): record work pose retries against accepted view requirements |
| 3 | `3dc751e32d85349e11d45cc5fcdc0ce8d78fb60f` | art-prod(p3): retain camp and adult rest rejection evidence |
| 3 | `eee63e9e260081fb4ae5eb8c6dab045f4795ea4c` | art-prod(p3): retain remaining rest and play retries |
| 3 | `86f16401e45feebbd81f268ce8ad5195899f1146` | art-prod(p3): close child locomotion and structure generation |
| 3 | `f85bfa55b9c80c7d101ee23df3367ba5463f933d` | art-prod(p3): close work pose generation at the cell attempt limit |
| 3 | `92e9e5caeac1f84d6cd64a9e6705020550fdf267` | art-prod(p3): apply forty-call fallback checkpoint |
| 3 | `678cfb05e73a280b2b45a1944f1b27c44c4d07e1` | art-prod(p3): close bounded character retries with honest rejection evidence |
| 3 | `9b5751730a56f23aa852f4aefcf980670c9d3c9c` | art-prod(p3): fill exhausted production slots with explicit procedural provenance |
| 3 | `253c71c6023e81ba5d52bdbdb9b791932be1cf36` | art-prod(p3): retain recognizable native bodies in rest fallbacks |
| 7 | `dc55c9a663320b57047c1ba50a6298e8164a06d0` | art-prod(p7): preserve complete ledger mutation and integration evidence |
| 7 | `a4c168fb684bd17e743d03bb152acfa71b8c558f` | art-prod(p7): preserve real panel non-overlap through the new layout |
| 7 | `d149611ff38bc3decf3682ab131007ae446affd4` | art-prod(p7): publish real captures and retain failed visual acceptance results |
| 7 | `73972b0fcb5b72ab612254170c729dc33bc50363` | art-prod(p7): account for scoped test repairs and independent visual review |
| 8 | `64db8021f9e9cae4b8aabb0725a5d047bcab4ef1` | art-prod(p8): permit exact-head verification without rewriting committed evidence |

Part 6 주요 파일은 `scripts/ui/panels/{hud_topbar,hud_status_panel,agent_inspector_panel,causal_panel,art_ui}.gd`, `assets/ui/worldsim_theme.tres`다. 추가 키는 원문과 동일한 `HUD_NEWS_TITLE`, `INSPECTOR_CARRYING_FMT`, `HUD_SPEED_FMT`이며 영어·한국어 FTL과 `localization/compiled/{en,ko}.json`, `localization/key_registry.json`을 기존 컴파일러로 갱신했다. 주민 제목은 기존 `INSPECTOR_AGENT_FMT`를 재사용한다. 기존 키 ID는 보존한다.

### 실제 게임 화면 검증

[실제 창 QA](ingame_qa.md)는 원시 windowed 프레임버퍼와 실제 노드 증거로 **92/92장**을 대조했다. A1–A12 중 **10개 PASS, A5·A11 FAIL**이다. 추가 CAPTURE_SET·RAMP 검사는 PASS이며 12개 A 항목의 분모에 더하지 않는다.

| 검사 | 수치·판정 |
|---|---|
| A1 | PASS: 92프레임, 표준편차·색 수 실패 없음 |
| A2 | PASS: 4×·6× 원시 체커에서 혼합 픽셀 0 |
| A3 | PASS: 실제 그리기 사각형 1,164개, 크기 오류 0; 성인 S-idle 4프레임 비율 오류 0 |
| A4 | PASS: 4×·6× 북/남 가림 4사례 픽셀 일치율 모두 1.0 |
| A5 | **FAIL**: 대비 중앙값 0.2378, 최솟값 0.0548; 주민 0은 0.0884로 두 주민이 최소 0.10 미달. 겹침으로 8명 측정 불가, 통과로 세지 않음 |
| A6 | PASS: 낮/밤 세계 밝기 0.2951/0.1523; 화덕 48px 평균 0.2524 |
| A7 | PASS: 이동 표본 40개, 완전한 이동 1초 구간별 프레임 수 5·4·4, 변경 비율 1.0; 3.956초, 간격 0.054–0.117초 |
| A8 | PASS: 89장, 표시 규칙 실패 없음. 원거리 정착지 디스크 제거까지 증명하는 검사는 아님 |
| A9 | PASS: 90장, 한국어·글꼴·라벨·패널 검사 실패 없음 |
| A10 | PASS: live simulation 300프레임, 중앙값 **5.929ms**, p95 **15.266ms** |
| A11 | **FAIL**: 4×·2× 모두 **4/24=16.67%**. 건설·음수·수면·어린이는 관찰 집합에 없어 미검증 |
| A12 | PASS: 주민 22명에 22개 조합; 동일 조합 이웃 0/13 |

RAMP는 12개 알려진 변형을 4×·6×에서 대조해 모두 실제 프레임버퍼 일치율 1.0을 기록했다. 기술적 색상 출력과 활동을 알아보는 능력은 별도다. 눈 가리기 관찰 답안 SHA-256은 `b32d76447e14b10675e1f0262d4d9f94161ef55e192112b93a00403b131cafd4`이며 정답 공개 전 저장됐다. 두 배율의 혼동 행렬은 동일했다.

| 실제 활동 | gather로 응답 | carry로 응답 | walk로 응답 | 식별 불가 | 합계 |
|---|---:|---:|---:|---:|---:|
| talk | 2 | 0 | 0 | 0 | 2 |
| sit_or_idle | 0 | 1 | 0 | 5 | 6 |
| gather | 3 | 0 | 0 | 0 | 3 |
| carry | 1 | 0 | 0 | 5 | 6 |
| walk | 0 | 4 | 1 | 2 | 7 |

검토 산출물: [전후 비교](before_after.png), [낮 40프레임 GIF](village_noon.gif), [저녁 40프레임 GIF](village_evening.gif), 전체 크기 PNG [낮 마을](shots/a_d2_1200_F2.png)·[밤 마을](shots/a_d2_2300_F2N.png)·[전체 지도](shots/a_d2_1200_F1.png). [기능 검토](visual_review_functional.md)와 [D3 일관성 검토](visual_review_fidelity.md)는 모두 **REVISE**다. 숲의 주민 가림, 반복 정면 바구니 실루엣, 어두운 HUD 아이콘, 평면적인 나무와 기존 자산의 디테일 밀도 차이를 지적했다. 검토 B는 전체 지도 정착지 디스크도 발견했다. 문서의 판정은 당시 캡처를 기준으로 보존하며 후속 수정으로 소급 PASS 처리하지 않는다.

### 정적 테스트 장부

`docs/art/TEST_LEDGER.md`의 최종 감사 범위는 **343행**이다. **338행은 사멸한 변이 표적에 연결**됐고, **5행은 단언 진단 문구만 바뀐 행**으로 `docs/art/MUTATION_AUDIT.md`에 비변이 이유가 기록돼 있다. 주요 변경 이유는 MultiMesh·UV·상태 색·크기 확대·이미지 경로·HUD 소유 위치·패널 배치의 교체다. 실제 스냅샷 읽기, 시뮬레이션 쓰기 금지, 클릭 반경, 사건 의미 계약은 유지한다.

대상 커밋 **`0189eead423c0496bda04d93b2e2da57a3efea27`**에서 **151개 시험 함수**를 사용해 변경 단언에 매핑된 변이 **467/467개를 사멸**시켰다. 생존·무효·기준 실패는 각각 0이다. 격리 worktree에서 각 기준 시험 통과 후 변이를 실행했으며 컴파일 실패·0개 선택 시험·시간 초과는 사멸로 세지 않았다. 측정 구간은 UTC 12:02:15.405–12:09:49.279, **453.874초**다. 원상복구와 깨끗한 worktree가 확인됐다.

추가 기하 감사는 **`a4c168fb684bd17e743d03bb152acfa71b8c558f`**에서 실제 패널 위치·폭·상수 연결·오른쪽 정렬·겹침 의미 변이 **7/7개를 사멸**시켰다. 추가 7행은 개별 매핑됐고 2.673초가 소요됐다. 앞선 467개 감사의 모든 대상 소스·시험은 두 SHA 사이 바이트 동일성이 확인됐다. 따라서 합계는 **두 명시적 SHA에 걸친 474개 사멸**이며, 단일 최종 HEAD에서 474개를 다시 실행했다고 주장하지 않는다. 전체 행은 336+7=343, 사멸 대응 331+7=338, 진단 문구만 5행이다.

전체 어휘 후보 758개 중 **매핑되지 않은 291개는 실행 범위 밖으로 미실행**이다. 따라서 `summary.json`의 전체 `audit_complete:false`와 `mapped-summary.json`의 `mapped_audit_complete:true`를 구분하며 758/758 통과로 보고하지 않는다. 이후 감사 대상 코드·시험이 변경되면 관련 부분 재검사가 필요하다. 이 결과는 정적 단언의 민감도를 검증하며 최종 아트 품질·시뮬레이션 정확도·기기 수용을 증명하지 않는다.

### 시뮬레이션 작업과의 호환

아트 `73972b0fcb5b72ab612254170c729dc33bc50363`와 최신 시뮬레이션 `a8611e07aabbb4043f5fac2e8dd14b375a0ada5d`를 임시 로컬 분기에 병합했다. 충돌 0건, 임시 병합 SHA `ce6d9494ba763b6f3f5689f333f60154ac7f39fb`, 브리지 빌드와 실제 창 92장 캡처 성공. 임시 분기는 삭제했고 원격에 푸시하지 않았다. WIP에 시뮬레이션 변경은 포함되지 않는다.

두 선택적 키가 실제 스냅샷에 존재했다. 정오 성인24명·어린이1명이며 어린이24번은 child 프레임을 사용하지만 나무에 가려 독립 시각 식별은 입증하지 못했다. 19:30에 주민16명이 화로에서 Chebyshev3 이내에 앉았고 어린이도 앉은 프레임을 사용했다. 23:00에는 주민26명 중23명이 쉼터 타일에서 수면 상태로 숨겨졌고3명은 이동 상태로 남았다. 개별 주민의 쉼터 소유권은 이 렌더링 증거로 검증하지 않았다. [세부 증거](compatibility/README.md), [원시 요약](compatibility/summary.json).

### 자율 결정 목록

`DECISIONS.md` 기준으로 독립 복제·WIP/green 분리, 실패를 포함한 생성 60회 제한, 창 실행 공유 잠금, D3 전용 설치, 차분한 지면 변환, Sprite2D 공유 정렬을 적용했다. 정수 줌 전환은 0.12초 보간 표본을 정수 단계로 맞춘다. 보고용 연락 시트는 JPEG85로 저장하되 원본·셀 자산은 무손실로 보존한다. 커밋 `784fdc0ac7d683507027918577612e555f4653cd` 제목의 flint는 잘못된 설명이며 실제 수용 셀은 N2_05 작은 바위다. 호출 체크포인트20·40을 보존했다. 최종 원격 SHA는 사용자 메시지, 커밋 체크포인트 시간은 아래에 기록한다. 독립 Part별 실행 시간은 병렬 작업 중 계측하지 못했으므로 커밋 간격을 실제 작업 시간으로 주장하지 않는다.


Part별 시간은 아래 **커밋 체크포인트 사이의 근사 벽시계 구간**이다. 병렬 작업이 겹치므로 합산 가능한 노동 시간도, 실제 시작 시각도 아니다. 단일 커밋 Part는 직전 통합 체크포인트부터의 구간만 표시하므로 제작 시간을 과소대표한다. 실제 측정이 있는 변이 실행 453.874초는 위에 따로 기재했다.

| Part | UTC 커밋 구간 | 측정 한계 |
|---|---|---|
| 0 | 11:06:54–11:14:51 | 커밋 간 7분 57초; 실제 작업 시간은 별도 계측하지 않음 |
| 1 | 11:15:09–11:15:09 | 단일 커밋 체크포인트; 병렬 작업의 독립 소요 시간 미계측 |
| 2 | 11:25:10–11:25:10 | 단일 커밋 체크포인트; 병렬 작업의 독립 소요 시간 미계측 |
| 3 | 11:25:29–12:23:07 | 커밋 간 57분 38초; 실제 작업 시간은 별도 계측하지 않음 |
| 4 | 11:57:19–11:57:19 | 단일 커밋 체크포인트; 병렬 작업의 독립 소요 시간 미계측 |
| 5 | 11:59:14–11:59:14 | 단일 커밋 체크포인트; 병렬 작업의 독립 소요 시간 미계측 |
| 6 | 11:59:16–11:59:16 | 단일 커밋 체크포인트; 병렬 작업의 독립 소요 시간 미계측 |
| 7 | 12:01:36–12:33:41 | 커밋 간 32분 5초; 실제 작업 시간은 별도 계측하지 않음 |
| 8 | 12:35:43–12:35:43 | 단일 커밋 체크포인트; 병렬 작업의 독립 소요 시간 미계측 |

### 확인된 제한사항

변환기의 원시 JPEG 90% 및 4px 배음 반례 요구는 명시적으로 실패한다. 생산 256셀 중 생성 수용은 13셀뿐이며 243셀은 절차적 대체다. 북·동 방향용 수용 원형이 없는 주민 동작은 정면 D3 대체여서 방향 정체성을 증명하지 못한다. D3 임시 셀의 크기·충실도 실패도 별도 남는다. A5 대비·가림과 A11 활동 식별은 실제 창 증거에서 실패했다. 두 시각 검토도 REVISE다. Part 4–6 커밋 및 변경 단언 변이 감사 통과를 최종 전체 gate·시각적 수용으로 확대하지 않는다. 비교 이미지·GIF·정량 QA는 게시됐으나 호환 시험과 코드 게이트는 완료됐다. 최종 원격 SHA와 초안 PR URL은 발행 후 사용자 메시지에 기록한다.

### Harness 결과

전체 게이트 대상 `64db8021f9e9cae4b8aabb0725a5d047bcab4ef1`에서 **19/19 종료 코드0**. 이후 변경은 보고서·캡처·장부 기록뿐이다. 발행 커밋에서 같은 전체 게이트를 외부 증거 디렉터리 `.harness/evidence/art-production/final-head-gate/`로 재실행해 HEAD를 바꾸지 않고 확인한다. WIP·green의 정확한 최종 SHA와 `git ls-remote` 일치, 초안 PR URL은 최종 사용자 응답의 발행 기록에 제공한다.

| 항목 | 결과 | 소요 |
|---|---|---:|
| cargo test --workspace | 1,806개 통과, 실패0 | 331초 |
| cargo clippy --workspace --all-targets -- -D warnings | 통과 | 6초 |
| strict GDScript | 통과 | 5초 |
| sim-bridge build | 통과 | 4초 |
| headless smoke·오류 검색 | 종료0, SCRIPT ERROR/ERROR0 | 7초 |
| Python 파이프라인 | 57개 통과 | 2초 |
| Godot 아트 장면 | 12개 통과 | 합계9초 |

근거: [final/results.tsv](gates/final/results.tsv). 구형 정보창 위치·폭 고정 때문에 실패했던 최초 통합 로그도 [integrated/results.tsv](gates/integrated/results.tsv)에 보존했다. 실제 비중첩 기하 계약으로 수정한 뒤 워크스페이스 전체1806개 재검사와 추가7개 변이 사멸이 확인됐다. 원래467개 변이 대상과 최종 렌더링 소스는 변하지 않았다. 창 캡처 당시 SHA와 최종 렌더링 파일의 동일성은 [capture_source_validity.json](capture_source_validity.json)에 기록했다.

green은 코드 게이트 통과만 의미한다. A5·A11과 독립 converter literal audit의 종료1은 그대로 미달로 남는다. 초안 PR의 제목은 **Art production 1: D3 world in the game**, 대상은 `codex/revitalize-2026-10`, source는 `codex/art-production-2026-10`이며 #510 위에 쌓인다.

### 정직한 판단

현재 증거로는 처음 보는 사람이 완성도 있는 픽셀 아트 게임으로 볼 수준이라고 승인할 수 없다. 선명한 네이티브 픽셀, 한국어 인터페이스, 밤 조명, 팔레트 출력과 성능은 검증됐지만, 실제 주민 대비 최소값과 활동 식별은 실패했다. 생성 수용 13셀에 비해 대체 243셀의 비중이 높고 정면 바구니 실루엣 반복·수관 가림·디테일 밀도 불일치·어두운 아이콘이 완성도를 낮춘다. 수정 두 차례 한도를 넘어 추가 작업을 했다고 주장하지 않으며 현재 결과를 검증된 기술 설치와 미수용 시각 품질로 구분한다. 다음 개선은 실제 활동을 구별할 수 있는 방향별 인물 원형, 주민·건물 주변 숲 가독성, 아이콘 대비에 집중해야 한다. 코드 게이트가 통과했어도 A5·A11 및 변환기 §6.1 실패는 별도로 남는다.

변경 경로 전체 목록: [changed_files.txt](changed_files.txt). 최종 보고 자체의 커밋은 자기 SHA를 포함할 수 없으므로 최종 사용자 메시지와 PR의 커밋 목록이 발행 기록을 보완한다.
