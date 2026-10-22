# ART-TRIAL3B 구현·실험 보고 — 2026-10-22

**판정: B 채택 불가. S1 구조 FAIL, R1 속성 판독 FAIL. 결제·크레딧 소비 0.**
무료 모델 경로의 제작과 제한된 실험을 수행했으며, 사용할 수 있는 캐릭터 세트를 완성했다는 뜻은 아니다. 모델1/3, 렌더3/3에서 종료한다. 기존 2D 작가 제작 권고를 유지한다.

구현 커밋: [f2c7fb83](https://github.com/hyunlord/new-world/commit/f2c7fb83370e21502612c5bb093d863d03c6fc67).
[오너 변경 계약](TRIAL3B_AMENDMENT_20261022.md), [이전 도구·가격 차단 역사](TRIAL3B_REPORT_PREFLIGHT_HISTORY.md). 이전 보고서의 미실행 상태는 당시 기록이며 현재 결과를 대체하지 않는다.

## 구현 의도

Higgsfield 결제 없이 무료 CC0 리깅 모델을 템플릿에 맞추고, 실제 3D 렌더가 구조·가독성 요구를 만족하는지 검사했다. 선택 사항인 Higgsfield one-shot은 생략했다. 따라서 3D 실제 호출의 가격 답을 얻었다고 주장하지 않는다. 유료 프롬프트·참조 이미지 생성·Mixamo 로그인 모두 0건이다.

## 템플릿·레이어 계약·구조 검사기

Part 0를 먼저 완료한 뒤 모델을 내려받았다. 그림자 중심(32,88), 불투명 최하단86..88, 연속 겹침3px, 손6px·발8px·팔다리4px, 머리 박스70%를 검사한다. 수면 머리 면적도 고정 템플릿 기준이며 입력 head_box를 작게 선언해 우회할 수 없다. 해부학 ID는 실제 모델 폴리곤 분할과 네이티브 IndexOB에서 유래하고, 검사 입력 해시에 포함된다.

B만 성인 세트와 움직이는 몸을 허용하고, 의복·머리 오버레이는 방향별 idle 한 장을 고정 사용한다. A의 기존 세트·상체 고정 규칙은 유지했다. 회귀 검사 포함 **45 tests PASS**. [Part 0 상세](B_PART0_CHECKS.md), [독립 구현 검토](B_IMPLEMENTATION_REVIEW.md).

## A 재검사 결과

| 항목 | 결과 |
|---|---:|
| 기존 레이어 | 87 |
| 새 기준 통과 / 실패 | 39 / 48 |
| limb_ok 실패 | 48 |
| contact_ok 실패 | 22 (위 실패와 중복) |
| 원본 SHA256 동일 | 265/265 |

A 이미지·마스크·sidecar·기존 보고서를 고치지 않았다. 최종 재검사 `b/part0-amendment/final-a/CHECKS.json`도 동일하다. 과거87/87은 과거 기준의 결과로 보존했다.

## 경로 B — 모델·클립·렌더

[Kenney Mini Characters](https://kenney.nl/assets/mini-characters)의 `character-male-a.glb`, [CC0](https://creativecommons.org/publicdomain/zero/1.0/)를 사용했다. 익명 다운로드 URL·License.txt·해시는 [B_MODEL_LOG](B_MODEL_LOG.md)에 기록했다. 후보 선택은 공개 미리보기 기반이며 다른 모든 모델의 수치적 최적성을 입증한 것은 아니다.

원래7개 뼈와 몸·머리 폴리곤을 유지하고 머리 비율48.87%를30%로 조정했다. 두 의복·두 머리·도끼·통나무를 별도 메시로 만들었다. 다섯 클립은 외부 생성 없이 Blender에서 작성했으며 각각 최대5개 시점의 키 포즈다. .blend 재로딩과 rigged GLB 재수입은 통과했다. 무릎·팔꿈치 관절이 없는 리그, 원래 옷/신발의 잔여 형태, 제거된 얼굴 디테일은 명시적 한계다. Blender UI 수동 편집은 없고 모든 수정은 스크립트로 남겼다. [모델 상세](B_MODEL_BUILD.md).

Blender4.5.14에서 S/E/N,20도 정사영,4× 렌더 후 Lanczos1×를 만들었다. 세 룩은 순서대로 실제 still을 검사하고 judge v2를 실행한 뒤 진행했다. 회색 잉크1.50 → 따뜻한3밴드2.50 → 8% 종이 질감2.50; 세 룩 모두 수치 검사 FAIL. 세 번째를 고정했다. 이후 카메라·셰이더 재시도나 픽셀 보정은 하지 않았다.

34개 패스: 몸16, 의복6, 머리6, 방향별 도끼3·통나무3. 색상은 IndexMA, 해부학은 IndexOB에서 파생했다. RGB/alpha 원본을 고치지 않았으며 마스크 가장자리 전파를 실제 alpha 지지 영역과 제한 거리 안으로 묶었다. 네이티브 근거 없는 섬은0으로 남겨 검사에서 실패시켰다. 소품 앵커는 실제 grip/어깨 부착점의 투영이다. 잘못된 초기 앵커 기록도 `render/history/`에 보존했다.

**구조 최종0/34, 채택 조합0.** 34개 모두 마스크 완전성 실패, 몸16개 모두 limb_ok 실패. 추가로 분리된 실루엣·구멍·머리 정렬·의복 범위/덮음률이 실패했다. 접지의 새 불투명 조건은 직립15개 모두 통과했지만 기존 발의 alpha 경계 검사는12개에서 실패했다. 이는 서로 다른 alpha 기준의 측정이며 접지만으로 레이어 통과를 주장하지 않는다. [렌더 상세](B_RENDER_LOG.md).

## 캐릭터 랩과 GIF

정상 CharacterLab의 통과 세트 입장 조건은 우회하지 않았다. 별도 **DIAGNOSTIC_REJECTED** 자료로 전체48조합을 meadow/forest와1×/2×에 합성했다. 24GIF·원본192PNG·A/B 동일 표본 비교 보드가 있다. 의복/머리/소품은 원래 렌더 좌표에 합성했으며 정상 앵커 재배치의 통과 증거가 아니다. idle은 계약의1px bob, walk/work/carry는 실제 프레임, 수면은 단일 정지 포즈다.

실제 Chromium GIF 재생을8Hz 목표로3초씩 캡처했다. **576화면·72연속 시트, 평균125.131ms, 최대159.181ms, JS 오류0**. sleep4개를 제외한20GIF에서 변화가 관측됐다. GIF 디코딩 이미지를 브라우저 재생 증거로 대신하지 않았다. 새 리뷰어 두 명은 정답·경로명을 제외한 패킷만 보고 각각68장·62장의 연속 시트를 판독했다. [랩 방법](B_LAB_DESIGN.md).

주요 로컬 산출물(모두 외부 원본):

- [1× A/B 비교](/Users/rexxa/work/arttrial3-data/b/lab/boards/compare_AB_1x.png), [2× A/B 비교 첫 페이지](/Users/rexxa/work/arttrial3-data/b/lab/boards/compare_AB_2x.png); 전체 페이지 목록 `lab/boards/pages.json`.
- [B 실패 보드](/Users/rexxa/work/arttrial3-data/b/checks/failures.png), [실제 브라우저 화면](/Users/rexxa/work/arttrial3-data/b/browser/overview/1280.png).
- GIF `b/lab/gifs/b_01..12_{1x,2x}.gif`; 전체 조합 보드 `b/lab/boards/`.
- 브라우저 원본 `b/browser/playback/`, 시간·해시 `b/browser/playback.json`.
- 리뷰어 원문 `/Users/rexxa/work/arttrial3-data/reviewer-{one,two}.json`; 채점 `b/readability.json`, 재현 스크립트 `b/score_reviews.py`.

## 검사 결과

| 지표 | 실측 결과 | 판정 |
|---|---|---|
| S1 | 채택0, 제출34; 유효 완성 세트 없음 | FAIL, 빈 집합을100%로 표시하지 않음 |
| S2 | 무료 기본 모델1/3, 렌더 룩3/3 | 상한 준수, 렌더 소진 |
| R1 | 아래 표; 속성20~24/36 | FAIL |
| T1 | 여섯 still 모델 평균2.46/5; numeric0/6 | 선호4.0 미달 |
| P1 | 별도 진단200명,60.001초,489.807 render-loop FPS | NOT_VERIFIED for accepted lab; 적격 아님 |
| K1 | credits0; 채택 레이어당 분 산출 불가 | 채택 분모0, 생산량 투영 불가 |

| 리뷰어 | 배율 | 동작 /12 | 소품·의복·머리 /36 | 판정 |
|---|---:|---:|---:|---|
| one | 1× | 10 | 22 | FAIL |
| one | 2× | 10 | 24 | FAIL |
| two | 1× | 10 | 20 | FAIL |
| two | 2× | 10 | 22 | FAIL |

동작 기준1×8/12·2×10/12는 충족했지만 속성30/36을 모두 미달했다. 옆모습의 의복 차이와 소품·머리 구분이 불확실했으며 unknown은 오답으로 유지했다. 답변을 채점 뒤 수정하지 않았다. 허용 렌더3회를 이미 소진해 별도 룩 수정/리뷰 재시도는 하지 않았다.

스타일 독립 모델 점수: medium2.00, line2.83, light2.00, palette3.00. 따뜻한 색과 외곽선은 연결되지만 평면적인 옷·얼굴과 각진 몸이05 기준의 질감·입체감을 잃는다는 판독이다. 투명 합성 still6장을 judge v2에 입력했으며 배경으로 점수를 부풀리지 않았다. 원문·수치 결과 `b/style/`.

실제 Godot 창의 별도 군중 진단은200개 원본 RGBA 캐릭터를1×,walk_a/b로60초 재생했다. 60개 가시성 표본 모두200명, 숨김0, 소스 변경0. 공유 호스트 부하29 정도, vsync 비활성화의 렌더 제출 FPS이며 물리적 주사율·게임 시뮬레이션 성능이 아니다. 정상 랩의 구조 적격성이 없으므로 P1 통과로 계산하지 않는다. [군중 상세·캡처 제한](B_CROWD.md), [최종 화면](/Users/rexxa/work/arttrial3-data/b/crowd-diagnostic/crowd_final.png).

K1 시간은 durable 변경 계약 파일 생성19:25:11~최종 지표 집계19:53:39 KST의 **관측 벽시계28.47분 이상**이다. 앞선 탐색·마지막 문서/게시 시간과 병렬 에이전트 인시를 포함한 전체 노력값은 아니다. 네이티브 룩 렌더 합0.338초, 전체 패스2.231초는 렌더 호출 시간뿐이다. 채택0이므로 분/채택레이어와4몸체×전체 의상 생산 투영의 유한 범위는 산출할 수 없다. 실패 산출물당 평균을 생산 비용으로 바꾸지 않았다. `b/timing.json`에 기준을 기록했다.

## 판정

S1과R1이 실패하고 정상 P1도 적격 증거가 없다. **B viable 아님.** 이번 무료 리그·세 룩으로 요구 품질을 입증하지 못했으며, 템플릿에 맞춘2D 작가 제작 권고를 유지한다. 무료3D 전반의 불가능을 증명한 결과는 아니다. 남은 모델 후보 슬롯을 소진하려고 렌더 상한을 새로 열지 않았다.

## 변경된 파일과 검증

- `1a9d55f6`, `607c13b6`: 템플릿·해부학 검사·A 재검사, B 계약 구분.
- `f2c7fb83`: `art/pipeline3/build_b_model.py`, `b_model_*.py`, `render_b.py`, `b_render_*.py`, 브라우저/합성 도구와 회귀 검사; `art/proto/Trial3BCrowd.gd` 및 UID; 상세 기록.
- 이 보고 커밋: 최종 판정, 초기 보고 보존, 재현 가능한 수치 요약.

단순화: 무제한 마스크 전파를 제한된 근거 파생으로 교체하고, 편의상 픽셀에 맞추던 앵커를 실제 모델 투영으로 교체했다. 새 프로젝트 의존성은 추가하지 않았다. 게임 코드·A 자산은 변경하지 않았다.

검증: Python45 tests, Ruff E9/F, JS 문법, Blender 실행/GLB 재수입, 실제 브라우저·Godot 표면 검증, diff 검사 통과. Blender RNA 전체 strict 정적 타입 검증과 전체 게임 gate는 미실행이다. 기존 `check_docs.sh`는 CharacterLab/wash_tint의 tracked UID 및 두 기존 scene header UID 누락으로 계속 실패한다. 해당 기존 문제를 이번 미술 실험 통과로 숨기지 않는다. 경로 제한에 따라 공통 CHANGELOG/ROADMAP/HISTORY·스킬은 수정하지 않았다.

## 제한과 종료

최종 산출물은 검토용 불합격 자료이며 게임에 설치하지 않았다. 정상 통과 랩 대신 진단 랩을 쓴 차이, 단일 수면 포즈, 얼굴/관절 부족, mask fringe 실패, 공유 호스트 FPS를 보존한다. 시간 상한으로 N/두 번째 의복/군중을 버리지는 않았다. 렌더 상한 도달로 제작 재시도는 종료한다. 원격 게시 후 SHA 일치는 별도 최종 실행 증거로 확인한다.
