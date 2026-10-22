## 구현 완료 보고 — ART-TRIAL3

**판정: PARTIAL / A 최소 세트 NOT_COMPLETE, B 도구 미확보, R1 NOT_VERIFIED.** 아래는 시도 상한을 지킨 실험 결과이며 전체 목표 또는 제작 경로의 승인 완료를 뜻하지 않는다. 게임 코드 변경·병합 없음. 작업 브랜치 `v8/art-trial3`, 시작점 `2433a6128e89fde30a4dd9e6995fbfa75049ba30`.

### 구현 의도

부분 이미지가 처음부터 맞물리도록 공통 좌표·불투명 레이어·고정 상체·자동 검사를 먼저 만들고, 실제로 생성한 자산의 조합과 동작을 검증했다. 구조 통과와 미술·가독성·성능 승인을 분리했다. 생성 실패를 지우거나 네 번째 시도로 상한을 넘지 않았다.

### 템플릿·레이어 계약·구조 검사기 (좌표 요약, 검사 규칙, 단위 테스트 5종 결과)

`art/character/TEMPLATE.md`, `LAYER_CONTRACT.md`, 아동/성인/노인 × S/E/N 가이드9개. 작업64×96, 생성256×384, 발y88. 성인 머리(20,32)–(44,50), 몸통(18,50)–(46,70), S손(14,66)/(50,66), 어깨(32,52). 아동·노인 좌표는 별도 명세를 사용한다. 수면은96×64, 머리 왼쪽, 중심(48,36).

검사기는 캔버스·좌표·8연결 성분·12px² 초과 닫힌 구멍·상의95% 몸통 덮음·마스크 완전성·상체 픽셀 불변·실제 조합의 몸통 비침·소품 앵커·잘림을 검사한다. 요구된 정상/빈 몸통/떠 있는 팔/어긋난 발/마스크 누락5종과 후속 회귀를 포함하여 **22개 테스트 통과**. 초기 독립 감사에서 발견한 부분 집합 PASS 및 수면 정렬 누락을 수정했다. 얇은 머리 돌기처럼 의미상 잘못된 머리를 bbox 검사만으로 검출할 수 없다는 한계는 남긴다.

최종 채택 레이어 **87/87**, 상체 불변 및 조합 **603/603** 통과. 조합은 직립600+맨몸 수면3이다. 최소 인벤토리는 성인·아동·노인의 S 털망토 누락 때문에 실패한다. `CHECKS_A.md/.json`의 `passed=false`, `subset_passed=true`, `status=NOT_COMPLETE`를 유지했다.

### 경로 A (레이어별 시도·통과, 편집 모드가 못 한 것, 마스크 처리)

| 묶음 | 실제 생성 | 채택 원본 | 결과 |
|---|---:|---:|---|
| S 몸체5프레임 | 6 | 5 | 통과 |
| E 몸체5프레임 | 9 | 5 | 통과 |
| N 몸체5프레임 | 9 | 5 | 통과 |
| 수면 | 2 | 1 | 통과 |
| 상의6종 | 17 | 5 | S 털망토3회 실패 |
| 머리6종 | 6 | 6 | 통과 |
| 도끼·통나무 | 3 | 2 | 통과 |
| 합계 | **52** | **29/30** | **미완성** |

레이어별 정확한 시도는 `COST.json.attempts_by_layer`, 실패·미사용까지 포함한52회 원본은 `FAILURE_INDEX.md`, 방향별 처리 기록은 `A_*_PROGRESS.md`에 있다. 거절22개와 미사용1개를 실패 보드23타일로 보존했다. 미사용 N 가죽옷3차도 비용에서 제외하지 않았다.

모델은 정확한 좌표, 몸통 고정, 의복 단독 출력에 일관되게 성공하지 못했다. 프레임은 허용된 idle 머리·몸통 복사, 일부 상의는 명시된 dressed mask-box 추출을 사용했다. 축소 후 상의 alpha248–254를255로 정리했고 RGB와 투명 외곽은 유지했다. 색 양자화·윤곽 강제·이미지 차분 추출은 사용하지 않았다. 피부1, 머리2, 상의3/4, 기본 들보와 소품5로 마스크를 기록했다.

아동29개·노인29개는 새 생성 없이 파생했다. 독립 재현 감사에서 원본29/29와 파생58/58 픽셀이 동일하게 재생성됐고 출처 해시87/87이 맞았다(`FINAL_AUDIT.md`, `PROVENANCE_AUDIT.md`). 원본 로그와 큰 이미지들은 `/Users/rexxa/work/arttrial3-data/`에 보존한다.

### 경로 B (모델·리깅·클립 출처, 렌더 설정, 패스 분리, Blender 수정 수, 크레딧)

**STOPPED_TOOL_UNAVAILABLE.** 공식 Higgsfield CLI1.2.2 설치 후 계정은 workspace 미선택, workspace 조회는 응답 없이 auth-login 안내로 종료됐다. 인증/네트워크의 정확한 원인은 단정하지 않았다. 환경변수와 명시된 파일 위치에서 Meshy 키도 확보하지 못했다. 생성 제출0, 생성 크레딧0. GLB·리깅·클립·렌더 설정·패스 분리 증거는 없고 Blender 수정 수는 N/A다. 3D 생성 선행조건이 없어 Blender 설치는 보류했다. `ROUTE_B_PREFLIGHT.md`, `CHECKS_B.md` 참조. B를 미술적 실패나0% 구조 통과로 계산하지 않았다.

### 캐릭터 랩과 GIF (보드·GIF 경로)

`art/proto/scenes/character_lab.tscn`, `CharacterLab.gd`, `art/pipeline3/capture_lab.py`. 게임 브리지와 무관한 실제 Godot4.6 창에서 초원·숲 바닥,1×/2×, 모든 검증된 조합을 표시한다. 불완전 세트는 명시적 `--allow-incomplete`에서만 진단용으로 열린다. B 자산이 없을 때 가짜 그림이나 성공한 route-toggle 증거를 만들지 않는다.

- 비교 보드: `art/character/boards/compare_AB_1x.png`, `compare_AB_2x.png`. 동일12개 GIF의 첫 프레임이며 B는 UNAVAILABLE 빈칸이다.
- 움직이는 파일: `art/character/gifs/a_01_1x.gif`부터 `a_12_2x.gif`, 총24개. 원래1×/2× 픽셀 크기 유지.
- GIF 시간:120/130ms 교대로8샘플/1초. 동일 수면 샘플은1프레임1000ms로 무손실 병합된다.
- 실패 보드: `art/character/boards/failures_a.png`. 실패 레이어는 정상 보드에 넣지 않았다.
- 전체 페이지·원본 프레임·실행 로그는 외부 raw 경로에, 저장소에는 대표 페이지와 전체 페이지 SHA/인덱스를 둔다. 세부 경로는 `LAB_RESULTS.md`, `BOARD_INDEX.md`.

초기 큰 창이 macOS 높이 제한에 잘린 결과와 가려진 창의 렌더 정지 결과를 버리지 않고 무효 측정으로 보존했다. 페이지별 실제 창 크기·조합 인덱스를 검사하고, 가시 창을 유지하여 군중 측정을 다시 수행했다.

### 검사 결과 (S1·S2·R1·T1·P1·K1 표, 리뷰어 원문 요약, 실패 보드)

| 지표 | A 결과 | B 결과 | 판정/범위 |
|---|---|---|---|
| S1 | 채택87/87=100%; 원본29/30 | N/A0/0 | 채택 구조 PASS, 최소 세트 NOT_COMPLETE |
| S2 | 52/29=1.793회 | 생성설정0 | A≤2.0 충족; 파생으로 분모 부풀리지 않음 |
| R1 | 실제 시간 재생 검토 NOT_VERIFIED | NOT_VERIFIED | 블라인드 디코드 순서 판독은 대체 승인 아님 |
| T1 | 6장 평균2.5417/5; judge v2 수치0/6 | NOT_MEASURED | 선호4.0 미달, 보고용 |
| P1 | 200명60.009588초,4,734렌더=78.8874FPS | NOT_MEASURED | 관측 FPS≥60; 불완전 세트라 전체 경로 P1 승인 불가 |
| K1 | 기록 생성시간1.105–1.294분/채택 원본; 크레딧 UNKNOWN | 제출0·생성크레딧0 | 노동·총 제작시간·과금 추정 아님 |

두 명의 새 블라인드 리뷰어는 라우트·정답 없이 GIF의 전체164 디코드 프레임과 시간을 확인했다. 실제 시간에 따른 재생은 도구로 관찰하지 못했다. 따라서 **R1은 NOT_VERIFIED**다. 순서 판독 참고 점수는 리뷰어1:1×/2× 활동9/12·속성32/36, 리뷰어2:각10/12·34/36. 리뷰어1의2× 활동은 원래10/12 기준에도 미달한다. 도끼질이 idle로 읽히고 통나무가 머리를 가리는 문제가 반복됐다. `BLIND_REVIEW_1/2.*`, `READABILITY.md`, `READABILITY_RESULTS.json`에 답변과 정답 비교를 보존했다. 수면은 실제 맨몸·대머리 상태를 평가했으며 수면 의복·머리 다양성은 검증하지 못했다.

스타일 리뷰는 같은 person-family 참조2개를 실제 확인했다. 두 참조는 한 작품의 겹치는 crop이므로 독립 미술 표본2개는 아니다. 흐린 윤곽과 회색 워시가 v2의 진한 잉크·따뜻한 황토색에 못 미쳤다. 기존 judge를 바꾸지 않았으며 보정12/12, 후보 수치0/6, 전체 점수는 `STYLE_REVIEW.md`, `STYLE_SCORES.json`, `style/`에 있다.

군중은 Apple M4 Max, Godot4.6 OpenGL-over-Metal 실제 창,200명,1×에서 측정했다.61회 가시성 표본 모두 drawable, 숨겨진 처리 프레임0, 시작/종료 소스 SHA 동일,200명 전체 캔버스 경계 안이다. 기존 camp.tscn 전체 장면 대신 같은 초원·캠프 소품을 쓰는 랩의 군중 격자다. 시뮬레이션·브리지·기존 울타리/동반자 부하가 없는 수치로 게임이나 기존 camp.tscn 전체 성능을 뜻하지 않는다.

비용 투영: 요청 조합 공간10,368개, 공유 가정별 고유 레이어136–196개, 관측 생성구간150.3–253.7분. 준비·검사·리뷰·노동·과금·실패 재설계 시간은 포함하지 않는다. 완료 납기 보장이 아니며 상세 산식은 `COST.md/.json`.

### 판정 (규칙 적용 결과와 권고; 선택은 오너)

**viable 판정을 받은 경로는 없다.** A는 최소 세트 누락과 R1 미검증이 남고 B는 실행 전 도구 조건에서 중단됐다. 구조적 호환성이 확보된87개가 있어도 전체 제작 경로의 승인으로 바꾸지 않는다. 사전 규칙에 따라 **이 템플릿·레이어 계약에 맞춰2D 작가에게 기본 인물 제작을 의뢰하고, 생성은 환경 자산에 유지**하는 방향을 권고한다. 선택은 오너에게 남긴다.

### 변경된 파일 목록 (커밋별)

| Part / commit | 변경 묶음 |
|---|---|
| [240fd9c3](https://github.com/hyunlord/new-world/commit/240fd9c3e9ce7dc404a6c0ca68f9863f541ce050) | v8-arttrial3(p0): prevent incompatible layers before judging character art; 21 files |
| [73e82bcc](https://github.com/hyunlord/new-world/commit/73e82bcc04805203ef63b238c6ca8b7c54d635de) | v8-arttrial3(p0): prevent diagnostic subsets from claiming route acceptance; 10 files |
| [d096a2d2](https://github.com/hyunlord/new-world/commit/d096a2d2a19fa1f1f29289de9b253bb94c7add1a) | v8-arttrial3(p2): preserve unavailable 3D route without invented render evidence; 2 files |
| [c17dce17](https://github.com/hyunlord/new-world/commit/c17dce170adfd3c2fc12249b8be09460ba07a44b) | v8-arttrial3(p1): preserve compatible layers without hiding the exhausted cloak variant; 278 files |
| [af6fd40e](https://github.com/hyunlord/new-world/commit/af6fd40e367fc88dbfdc2ce32326923ff0e6ecf2) | v8-arttrial3(p1): keep generated checker evidence whitespace-clean; 3 files |
| [b198a031](https://github.com/hyunlord/new-world/commit/b198a031186e97fab127cf9b654463d2763e0822) | v8-arttrial3(p3): make incomplete-route evidence visible without granting acceptance; 73 files |
| [51dbbe7a](https://github.com/hyunlord/new-world/commit/51dbbe7ae8ea2da826f4288daf021ec2153f6e3c) | v8-arttrial3(p4): prevent partial evidence from becoming a viable production claim; 7 files |

Part5는 이 보고서·생산 지침·최종 검증/파일 색인을 출판한다. 파일 전체 목록은 `COMMIT_FILES.md`에 커밋별로 기록했다. 추가·변경된 배포용 art 파일 합계8,152,608바이트로40MB 이내, PNG 최대1,313,374바이트, GIF 최대260,157바이트다. 임시 캐시는 이 배포 집계에서 제외했다.

### 제한사항 (드롭한 것, 도구가 없어서 못 한 것, 다음 패스가 풀 것)

1. S 털망토는3회 상한 소진. 임의 대체·재명명·추가 생성으로 채우지 않았다. 따라서 요구된 한 경로의 완전한 최소 세트 end-to-end 조건은 달성하지 못했다.
2. 실제 시간 재생을 보는 두 리뷰어 검증은 제공 도구로 완료하지 못했다. 디코드 프레임 평가를 통과로 대체하지 않았다.
3. B GLB·리깅·클립·Blender 렌더·GIF·성능은 없다. N 방향은 A에서 유지했고 소품2개도 유지했다.
4. 수면 옷·머리 조합, semantic head 검출, 큰 통나무에 가려지는 머리와 도끼질 가독성은 미해결이다. 59px 통나무 폭은 요청44px와 다르며 기록했다.
5. 전체 게임 gate는 범위 밖 게임 코드에 손대지 않았으므로 실행하지 않았다. 격리 파이프라인 테스트22개·Ruff 치명 오류 검사·타입 오류0개·Godot 실제 창을 검증했다. basedpyright 경고는 남아 있으므로 strict-clean이라 하지 않는다.
6. 기존 `.gitignore`, `.ignore`, `docs/art/`는 보존했다. 새 의존성은 프로젝트에 추가하지 않았다. 공식 CLI 설치와 기존 도구의 임시 실행 환경은 별도로 기록했다.

git ls-remote 결과 (보고서 기록 직전 Part4 출판 검증):

```text
51dbbe7ae8ea2da826f4288daf021ec2153f6e3c	refs/heads/v8/art-trial3
local HEAD 51dbbe7ae8ea2da826f4288daf021ec2153f6e3c
```

로컬과 원격 SHA 일치. 이후 Part5 보고서 커밋도 같은 브랜치로 push하고 다시 비교한다. 최종 출판 SHA는 대화 최종 응답에 기록한다. 다른 브랜치 push·강제 push·병합 없음.
