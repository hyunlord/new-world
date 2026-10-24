# ART-TREES 보고서

## 판정: 오너 판단 대기

`oak`, `pine`, `birch`를 시뮬레이션 목재 재료 이름 그대로 키로 사용하는 아트 데이터와 순수 해석기를 작성했다. 여름 원본 12장, 낙엽수 겨울 형태 4장, 종별 그루터기 3장과 256×256 수관 덩어리를 제작했다. **게임 적용이나 시각적 승인 완료를 뜻하지 않는다.** 특히 작은 낙엽수의 여름·겨울 줄기 실루엣 차이는 아래 제한사항과 오너 결정 항목에 남긴다.

- 기준: `v8/art-polish2`의 `c8ed86087c90ed5429716713e930cc5f6a9e1332`.
- 작업 브랜치: `v8/art-trees`. 목표 원문 커밋: `83393ac2b`.
- 핵심 나무 생성 호출 **28회**, 관목 **5회**, 합계 **33회 / 상한 120회**. 대상별 최대 3회 제한을 유지했다.
- 데이터-only 증명: **`b78ed83c3`**, 파일 3개에 `.py` 변경 없음 확인.
- 푸시 대상: `origin/v8/art-trees`. 아래 게시 검증 절차로 원격 SHA를 확인한다.
- 범위: `art/**`, `docs/v8/arttrees/**`. 게임 코드 수정 및 머지 없음.

## 스키마와 해석기

[species.json](../../../art/flora/species.json)의 `species` 키, 각 `id`, 나무의 `log_material`은 동일하다. 실제 시뮬레이션의 `rust/crates/sim-core/data/material/wood/{oak,pine,birch}.ron` 이름과 대조하는 테스트가 있다. `habit`은 `deciduous`, `evergreen`, `shrub`이며, 크기별 `variants`와 `bare` 배열, 각 프레임의 경로·캔버스·앵커·마스크, 그루터기, 사계절 기록, `map_canopy` 참조를 담는다. `test_alder`는 생산 목록에 넣지 않는다.

[habit_defaults.json](../../../art/flora/habit_defaults.json)은 습성별 기본 계절과 대체 종을 제공한다. 알 수 없는 종은 호출자가 전달한 `habit`으로 대체하며, 습성도 없으면 데이터의 `unknown_habit`을 따른다. 이때 대체 종의 개별 계절값이 아니라 **습성 기본값**을 쓴다. 대체 종과 습성이 어긋난 데이터도 파싱 단계에서 거절한다.

`read_catalog(root)`가 파일 읽기와 검증을 맡고, `resolve_flora(species_id, size, season, variant_hash, catalog=catalog, habit=None)`는 I/O·난수 소비·입력 변경 없이 프레임 경로, 마스크, 틴트, 앵커, 캔버스와 재료 정보를 반환한다. 해석기에는 종 이름 분기가 없다. 음수 해시도 modulo로 변형을 안정적으로 고른다. JSON Schema는 문서 구조를, Pydantic 파싱은 키·재료·크기·계절·대체 참조 관계를 검증한다.

새 종 추가 절차는 [ADDING_A_SPECIES.md](../../../art/flora/ADDING_A_SPECIES.md)에 정리했다.

## 생성: 종·크기·변형별 시도와 검사

내장 imagegen에 Route I 인물 시트와 구조물 스타일 기준을 첨부했다. 반환된 고해상도 원본을 외부에 보존하고, 전체 실루엣의 균일 축소와 투명 패딩만 적용했다. 부분 늘리기, 가지 이동, 분리 픽셀 삭제는 하지 않았다. 생산 규격은 검사 전에 고정했다.

| 대상 | 캔버스 | 앵커 | 목표 높이 |
|---|---:|---:|---:|
| large | 192×192 | (96, 180) | 160px |
| small | 128×112 | (64, 100) | 80px |
| stump | 64×48 | (32, 40) | 20px |

| 종 | large 여름 v1 / v2 | small 여름 v1 / v2 | large / small 겨울 | 그루터기 |
|---|---:|---:|---:|---:|
| oak | 1 / 1회 | 1 / 1회 | 2 / 3회 | 1회 |
| pine | 1 / 1회 | 1 / 1회 | 해당 없음 | 1회 |
| birch | 1 / 1회 | 2 / 2회 | 2 / 3회 | 1회 |

최신 [CHECKS.json](CHECKS.json)에서 핵심 **19/19장**이 선언된 캔버스, 투영 수치, 단일 성분, 잘림 방지, 해당 마스크·여름색 검사를 통과했다. `projection_check.py`의 앵커·높이·광원 수치를 사용하며 small은 목표의 64–96px 밴드, large는 128–192px 밴드로 검사한다. 단일 성분의 판정 알파는 128, 가장자리 잘림은 알파 16이다. 알파 16 연결성은 보조 진단이며 실패 기준을 바꾸는 데 쓰지 않았다.

첫 시도의 올리브색 수관과 일부 미세 가지의 연결성 실패를 삭제하지 않았다. [ATTEMPT_CHECKS.json](ATTEMPT_CHECKS.json), [GENERATION.json](GENERATION.json)에 시도별 검사, 선택 원본·출력 해시와 변환을 기록했다. 원본·실행 데이터는 `/Users/rexxa/work/arttrees-data/`에 있다.

judge v2의 기존 나무 계열 이름은 `plants`다. painterly·woodcut·photo 음성 대조군 결과는 각각 통과·거절·통과로, **3개 모두 거절 조건을 만족하지 못했다. 따라서 스타일 점수는 보고용이며 게이트로 쓰지 않았다.** 수치 투영 검사 역시 front 3/4 시점이나 스타일의 오너 승인을 대신하지 않는다.

관목은 기존 full/bare가 수직 평면도임을 재확인해 재생성했다. full은 3회, 열매 없는 depleted는 2회 생성했으며 두 최종형 모두 32px 높이·밑동 앵커 80의 투영과 구조 검사를 통과했다. 줄기 박스 마스크 비율은 각각 0.603% / 0.948%, 군집 커버리지 100%, hue 102.16° / 102.24°, 채도 0.586 / 0.597이다. 기존 bare는 잎 없는 겨울형이 아니라 열매가 없는 상태라는 의미를 보존한다. `berry` 습성 기본은 새 96×96 프레임과 (48,80) 앵커를 사용한다. 실패 시도와 생산 대체 경로는 [BUSH_RECHECK.json](BUSH_RECHECK.json)에 보존했으며 시각 판정은 오너 판단 대기다.

## 앙상한 겨울 형태

oak·birch의 각 크기마다 여름 v1을 참조해 잎 없는 형태를 생성했다. pine은 별도 bare가 없다. 겨울형은 짝 여름 v1의 **동일한 전체 이미지 변환**을 적용해 위치 기준을 유지했다. 겨울에는 두 여름 변형 모두 해당 크기의 한 bare 프레임으로 교체된다.

그러나 동일한 변환은 생성 모델이 줄기 윤곽까지 정확히 보존했다는 증명이 아니다. [BARE_CHECKS.json](BARE_CHECKS.json)의 하단 30% 알파≥128 실루엣 비교는 다음과 같다.

| 짝 | 실루엣 IoU | 여름 대비 밑동 행 차이 |
|---|---:|---:|
| oak large | 0.9596 | +1px |
| oak small | 0.7419 | 0px |
| birch large | 0.9791 | 0px |
| birch small | 0.6454 | 0px |

**생성 겨울형의 줄기·뿌리 실루엣은 근사이며, 정확히 동일하다고 주장하지 않는다.** 특히 small 두 장은 차이가 크다. 임의 IoU 합격선을 만들지 않았고, 픽셀 수술이나 4번째 시도로 이를 덮지 않았다. 해당 차이를 허용할지는 오너 판단이 필요하다. 계절 틴트의 비마스크·알파 보존과, 서로 독립적으로 생성한 여름/겨울 이미지의 일치 여부는 별개의 검증이다.

## 수관 마스크 품질

Lab 공간에서 기존 올리브 수관색과 새 여름 팔레트의 최근접 군집을 찾고, 나무껍질·잉크 군집과 `L*≤24`의 어두운 잉크를 제외한다. 줄기 박스를 강제로 비워 통과시키지 않는다. 빈 겨울형·그루터기 마스크는 계절 색 변환 대상이 아니다.

핵심 프레임의 줄기 박스 마스크 비율은 최대 **1.172%**, 기준은 **2% 이하**다. 수관 군집 커버리지는 기준 **85% 이상**을 검사한다. 이 커버리지는 같은 군집 정의에서의 추출 충실도이며, 사람이 표시한 실제 잎 영역에 대한 독립 정확도 점수가 아니다. 가지와 잎의 색이 비슷한 경계에서는 오너의 확대 확인이 필요하다.

여름 원본의 최초 평균 hue가 팔레트 밖이어서 마스크 영역에만 Lab 색차를 적용했다. 최종 여름 12장의 평균 hue는 **102.47–106.11°**, 채도는 **0.576–0.643**이며, 팔레트 hue 범위 **88.89–112.50°**, 채도 **0.30 이상**을 만족한다. 보정 전 실패값, 실제 ΔLab, 마스크·입출력 해시와 보존 검사는 [SUMMER_TRANSFORMS.json](SUMMER_TRANSFORMS.json)에 있다.

## 계절 틴트 표

| 계절 | oak | birch | pine |
|---|---|---|---|
| 봄 | ΔLab (+6, 0, +6) | 동일 | 동일 |
| 여름 | 항등 (0, 0, 0) | 동일 | 동일 |
| 가을 | (−8, +35, +15), 황토·갈색 방향 | (+8, +10, +32), 노란색 방향 | 항등 |
| 겨울 | 해당 크기 bare로 교체 | 해당 크기 bare로 교체 | (−8, 0, 0) + 상단 눈 |

`seasons.py`는 sRGB↔D65 Lab 변환 후 마스크의 RGB에만 이동값을 적용한다. 여름 항등은 색 공간 왕복도 하지 않는다. 계절 합성의 비마스크 픽셀과 원본 알파 보존은 테스트로 확인한다. pine 눈은 수관 마스크의 열별 최상단에서 3px 깊이의 띠를 만들며, 흰색 혼합 알파는 최대 `178/255`로 70%를 넘지 않는다. 이는 생성 이미지에 구워 넣은 눈이 아니라 코드 합성이다.

## ¼× 수관 덩어리

[canopy_mass.png](../../../art/flora/textures/canopy_mass.png)는 256×256 불투명한 여름 수관 텍스처다. 전체 프레임 축소와 허용된 마스크 Lab 보정 후 hue **101.69°**, 채도 **0.572**를 기록했다. X 이음 MAD **15.125 / 제한 20.083**, Y **17.760 / 제한 19.689**로 양축 이음 수치가 통과했다. 외부 2×2 반복 그림에서 직선 이음이 보이지 않는다는 에이전트 관찰도 [CANOPY_TEXTURE.json](CANOPY_TEXTURE.json)에 기록했으며 오너 판정은 별도다.

¼× 합성기는 인접 나무가 둘 이상인 밀집 셀에 세계 좌표로 텍스처를 샘플링한다. 셀마다 무늬를 처음부터 반복하지 않는다. 성긴 나무는 개별 실루엣을 유지한다. 같은 계절 기록으로 수관 덩어리도 틴트하고, 겨울 bare 선택 셀은 잎 덩어리 대신 가지 스프라이트로 남긴다. 이 경로는 오프라인 시연이며 게임 청크 렌더러 적용은 후속 작업이다.

## 데이터만으로 종 추가 증명

`art/flora/tests/test_alder.json`은 oak의 프레임·마스크를 재사용하고 별도 계절값을 갖는 시험용 종이다. 생산 `species.json`은 변경하지 않고 임의 fixture를 읽는 일반 코드로 다음 명령을 실행한다.

```bash
python3 -m art.flora.boards --fixture art/flora/tests/test_alder.json --species-id test_alder --output art/boards/test_alder_strip.png
```

해석기 테스트는 네 계절 모두 새 종의 기록을 사용하고, 생산 카탈로그를 바꾸지 않는 것을 확인한다. 최종 증거는 코드 커밋 뒤 **fixture JSON + 렌더 스트립 + 메타데이터만** 추가한 별도 커밋이어야 한다. 렌더 메타데이터에는 실행 때의 Python 해시를 남긴다.

- 네 계절 스트립: [test_alder_strip.png](../../../art/boards/test_alder_strip.png), 실제 렌더·육안 확인 완료.
- 일반 구현 동결 커밋: `a3d5cc5bc`. 증명 커밋: `b78ed83c3`.
- `git diff-tree --no-commit-id --name-only -r b78ed83c3` 출력은 `art/flora/tests/test_alder.json`, `art/boards/test_alder_strip.png`, `art/boards/test_alder_strip.json` 3개뿐이다. **`.py` 변경 없음 확인**. 스트립 JSON의 Python 해시도 실행 시점과 연결된다.

## 오너 보드

- [trees_owner.png](../../../art/boards/trees_owner.png): 나무 3종 × 2크기 × 2변형 × 4계절 × 1배/2배, 총 **96개 표시 사례**. 차분한 단색 초지와 Route I 성인 56px를 함께 표시한다. 2배는 최근접 확대다.
- [trees_scene.png](../../../art/boards/trees_scene.png): 기존 게임의 황토색 숲과 새 여름 숲의 1배·¼배 비교. 기존 화면은 `d796fcacd23d03959e2bd1046f69581a9b1fc637`의 `docs/images/mart2_compare_sheet.png`에서 가져온 **50% 미리보기 크롭**이다.
- [trees_seasons_scene.png](../../../art/boards/trees_seasons_scene.png): 같은 day-6 입력을 봄·여름·가을·겨울로 합성한 원본 1배 영역.

장면은 `scene_day6_noon.json`의 tick 7440 관측과 기존 명시적 구조물 오버레이를 재사용한다. 숲이 드러나도록 새 아트 카메라를 캠프에서 동쪽 8타일 이동했다. **기존 게임 스크린샷과 정확히 같은 카메라·크롭의 전후 비교는 아니다.** 종 배정도 좌표 해시로 정한 아트 시연이며 시뮬레이션에 실제 종을 추가한 결과가 아니다. 계절은 동일 장면의 렌더 연습이며 실제 날짜 진행 캡처가 아니다.

1.5MB 제한에 맞춰 고정 영역을 잘라 표시하고 1배 픽셀을 축소하지 않는다. 전체 1600×900 렌더는 `/Users/rexxa/work/arttrees-data/boards/`에 보존한다. 각 PNG의 JSON에 소스·해시·패널 크롭·렌더 조건이 있다. 계절 공통 ROI는 `(784,352,1408,784)`, 원본 624×432px다. 최종 보드와 sidecar 해시 일치 및 96개 표시 사례의 소스 해시·캔버스 경계 검증을 마쳤다.

| 보드 | 픽셀 크기 | 파일 바이트 |
|---|---:|---:|
| trees_owner | 3264×4496 | 1,437,861 |
| trees_scene | 1968×1180 | 1,362,663 |
| trees_seasons_scene | 1296×964 | 1,431,742 |

## 검증과 변경된 파일

최종 `python3 -m pytest art/flora -q`: **42 passed**. `uvx ruff check art/flora`: **통과**. `basedpyright --pythonpath /opt/homebrew/bin/python3 art/flora`: **0 errors, 62 warnings**이며 완전 무경고 통과라고 주장하지 않는다. 경고는 NumPy/JSON 동적 타입, 미사용 반환값, 도달 불가 exhaustive 분기 등의 정적 분석 경고다. `git diff --cached --check` 통과, 명시적 허용 경로만 커밋했다. 주요 결과는 [CHECKS.json](CHECKS.json), [BUSH_RECHECK.json](BUSH_RECHECK.json), [GENERATION_ATTEMPTS.json](GENERATION_ATTEMPTS.json)에 연결된다.

| 묶음 | 경로와 역할 |
|---|---|
| 데이터 계약 | `art/flora/species.json`, `species.schema.json`, `habit_defaults.json`, `habit_defaults.schema.json`, `schema.py`, `resolve_flora.py`, `ADDING_A_SPECIES.md` |
| 생산 이미지 | `art/flora/sprites/*.png`, `masks/*.png`, `textures/canopy_mass.png`, `art/style05/palette_05.json`의 여름 수관색 추가 |
| 오프라인 도구 | `normalize.py`, `summer.py`, `seasons.py`, `check_assets.py`, `check_bare.py`, `canopy_texture.py`, `bushes.py`, `scene_board.py`, `boards.py`, `board_io.py` |
| 검증 | `art/flora/test_*.py`, `tests/test_alder.json`, `docs/v8/arttrees/*CHECKS.json`, `GENERATION.json`, `SUMMER_TRANSFORMS.json`, `CANOPY_TEXTURE.json` |
| 검토 자료 | `art/boards/trees_*.png`와 JSON, 데이터-only 스트립, 본 보고서·목표·결정 기록 |

게임 코드 경로는 건드리지 않았다. 기존 아트의 종별 분기를 늘리는 대신 JSON 카탈로그와 하나의 해석·합성 경로로 구성했다. 이 검증은 아트 데이터와 오프라인 Python 범위다. Godot 실행·게임 셰이더·성능·시뮬레이션 동작 검증을 수행했다고 주장하지 않는다.

## 제한사항

1. 작은 oak·birch의 겨울 줄기·뿌리 윤곽은 여름 v1과 정확히 같지 않다. 동일 실루엣 요구의 엄격한 충족은 미확인이다. 위 IoU와 실제 보드를 함께 판정해야 한다.
2. 스타일 판정기의 음성 대조 보정이 실패했으므로 스타일 자동 합격 주장은 없다. 원근·잎/줄기 분리·계절 색 읽힘은 오너 확인 대상이다.
3. 비교 보드는 동일 시드/day-6 데이터를 쓰지만 새 카메라 이동·크롭과 기존 화면의 50% 미리보기 때문에 픽셀 단위 전후 비교 자료가 아니다.
4. 마스크 커버리지는 Lab 군집 기준이다. 인간이 표시한 식물 부위 정답 마스크와의 정확도를 뜻하지 않는다.
5. 원본과 전체 렌더는 외부 데이터 디렉터리에 있다. 커밋된 해시·변환 기록으로 출처를 연결하지만 원본 파일 자체를 저장소에 넣지는 않는다.
6. 수관 덩어리의 숲 경계와 빈 셀은 타일 점유를 그대로 따르므로 계단 형태가 보인다. 관목은 마스크 줄기 조건을 만족시키기 위해 아래 줄기가 길게 드러나며 자연스러움은 오너 확인 대상이다.
7. 정적 타입 검사 경고 62개를 남겼다. 게임 전체 빌드나 런타임 검증 결과로 확대하지 않는다.

## 게시 검증

`git push origin v8/art-trees` 후 `git ls-remote origin refs/heads/v8/art-trees`와 로컬 HEAD를 대조한다. 원격 확인 결과는 [PUSH_VERIFICATION.json](PUSH_VERIFICATION.json)에 기록한다. 자기 커밋 SHA를 문서 안에 넣는 순환을 피하기 위해 이 기록은 검증한 보고서·보드 커밋을 가리키며, 마지막 증거 커밋의 원격 HEAD도 별도로 대조한다. 오너 판단 대기 상태를 유지하며 머지하지 않는다.

## M-ART2.1에 넘길 것

가져올 데이터는 `art/flora/species.json`, `habit_defaults.json`와 두 JSON Schema, 해당 레코드가 참조하는 `sprites/`, `masks/`, `textures/canopy_mass.png`다. `palette_05.json`의 `foliage_summer_*` 값과 [ADDING_A_SPECIES.md](../../../art/flora/ADDING_A_SPECIES.md)도 함께 참조한다. `resolve_flora.py`·`schema.py`·`seasons.py`는 오프라인 동작 기준이며 게임에 Python을 그대로 탑재한다는 뜻이 아니다.

게임 어댑터는 종 id·습성·크기·안정적인 변형 해시·계절을 입력으로 삼는다. 카탈로그를 로드한 뒤 프레임과 마스크를 선택하고, 앵커 기준으로 배치한다. `map_canopy`를 `map_canopies`에서 해석해 ¼배 밀집 청크에 적용한다. 미등록 종의 요청 id와 대체 아트 id를 구분해 진단에 남긴다.

계절 셰이더 입력은 **기본 RGBA, 수관 마스크, D65 Lab ΔL/Δa/Δb, `frame` 교체 지시, `snow_cap.depth_px/alpha`**다. 마스크 밖 RGB 및 원본 알파를 유지하고 여름은 항등으로 처리한다. 겨울 낙엽수는 bare를 먼저 고르며, evergreen만 −8 명도와 상단 눈을 적용한다. 그늘은 생성 스프라이트 밖에서 코드로 그린다. 장면·스프라이트 보드와 동일한 선택·합성 결과를 재현한 뒤 실제 게임 캡처와 성능을 별도로 검증해야 한다.

## M-EXT에 넘길 것

시뮬레이션 나무에 `species_id`가 생기면 이 값이 아트 카탈로그 키로 바로 연결된다. 벌목 결과 통나무의 재료 id도 같은 종의 목재 id로 이어져야 한다. 예를 들어 `birch` 나무 → `birch` 통나무 재료 → 재료 시스템의 `birch` 틴트 → 해당 재료를 사용하는 통나무 벽 색의 경로다. 현재 모든 통나무가 oak인 시뮬레이션을 이번 아트 작업에서 바꾸지는 않았다.

`log_material == id`는 아트 계약이며, 실제 아이템 생성·건설 재료 보존·벽 렌더 바인딩은 M-EXT와 게임 어댑터의 책임이다. 누락된 아트가 대체 프레임을 골라도 시뮬레이션의 원래 재료 id까지 바꾸면 안 된다. 시험용 `test_alder`는 데이터 확장 증명용이고 실제 목재 재료 등록을 의미하지 않는다.

## 오너 결정 필요

- 1배·2배에서 종·크기·두 변형의 형태, Route I 인물과의 비례, front 3/4 시점을 수용하는가.
- 여름 숲이 초지와 구분되고, 봄·가을의 차이와 pine 겨울 눈이 자연스럽게 읽히는가.
- 작은 낙엽수 겨울형의 줄기 실루엣 차이를 수용하는가. 수용하지 않으면 이번 생성 상한 3회 결과를 유지하고 별도 수정 계약이 필요하다.
- ¼배 수관 덩어리의 밀도·반복감과 실제 게임 통합용 데이터 계약을 수용하는가.

최종 시각 판정은 **오너 판단 대기**다.
