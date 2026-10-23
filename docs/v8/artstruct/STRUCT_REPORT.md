# ART-STRUCT 보고서

**판정: 오너 판단 대기. 구현·합성 결과를 제시하며, 아트 승인 또는 전 자산 수락을 주장하지 않는다.**

기준은 `v8/art-env1`의 `429a18000266483ee40d756840d68b08840313c5`, 작업 브랜치는 `v8/art-struct`다. 게임 코드 변경·merge 없이 `art/**`, `docs/v8/artstruct/**`에만 작업했다. 원시 생성물·실측 로그·전체 프레임은 `/Users/rexxa/work/artstruct-data/`에 보존했다. 미관 리뷰어는 사용하지 않았다.

## 투영 규칙

[PROJECTION.md](../../../art/style05/PROJECTION.md)의 수치를 고정했다. ground는64×64 top-down, standing은 route I와 같은 약20° front3/4·상좌광, base는 `(tile_left+32,tile_top+43)`이다. base y→x로 깊이 정렬하며 지붕은 그리지 않는다.

| 대상 | 1× 기준 |
|---|---|
| 벽·문 | cap16 + face44 =60px; 생성 문58–62px |
| 침구 | 높이14px(12–16),1×2타일; 선택본은 가로 방향2×1 |
| 작업대 |30–40px|
| 건조대 |48px(46–50)|
| 화덕 |본체20px, 불꽃 포함 전체20–40px; 본체 랜드마크는 별도 읽기|
| 바위·부싯돌·점토 |24–36 /20±2 /16±2px|
| 나무 |128–192px|
| 성인·아이 |56 /40px|
| 사슴·늑대 |어깨40 /28px; 전체 alpha높이는 어깨 증거가 아님|

`projection_check.py`는 alpha≥128 최하단과 선언anchor의 오차±2, 높이 밴드, opaque 내부 평균 휘도 gradient의180°<angle<270°를 검사한다. 평탄/표본 없음은 실패다. 상좌광 수치는 실루엣과 잉크에도 영향받는 대용 측정이며 카메라 각도의 증명은 아니다. normalization은 원본crop+균일resize+padding과 anchor를 기록하고, 검사는 그 출력에 수행했다. 원본부터 자동으로 올바른 투영이었다는 주장이 아니다.

## 벽 시스템·이음새

`wall_tiles`는 world x×64 mod256 위치에서 cap/face를 잘라 이웃의 텍스처 위상을 이어 준다. 남쪽이 벽이면 face를 생략하고 T/십자는 cap만 남긴다. 동서끝에는 end piece, 안쪽 코너에는16×16 cap을 둔다. 세로 연결은16px cap 행을 반복하여64px 지면 간격을 연결한다. 이는 코드의 선언된 반복이며 생성 원본 수선이 아니다. 문은 자기 셀을 통째로 교체하여 열린 부분의 alpha를 보존한다.

4회 타일링의 세 접합 열 MAD가 내부 열차이p95×1.15 이하인지 검사했다. 같은 strip 반복이라 세 접합 값은 같다. 실패한 strip을 복제/보정해 통과로 만들지 않았다.

| 텍스처 | 선택차수 | 접합 MAD(세 곳 동일) | 내부p95 | 한도1.15×p95 | 결과 |
|log_cap x|3|18.562500|10.385417|11.943229|FAIL|
|log_face x|1|8.424242|14.717424|16.925038|PASS|
|rock_face x|2|10.742424|16.888636|19.421932|PASS|
|stone_cap x|1|8.166667|13.689583|15.743021|PASS|
|stone_face x|1|18.803030|19.763636|22.728182|PASS|
|water x|3|7.414062|8.919141|10.257012|PASS|
|water y|3|12.915365|11.441797|13.158066|PASS|

**log_cap은3회 모두FAIL**이다. 1차3.000000>2.259271,2차13.687500>12.944688,3차18.562500>11.943229. 상한에 도달하여3차를 실패 상태의 보드용 후보로 남겼다. 합성 벽이 이어진다는 기능 검사와 텍스처 이음새 수락은 다르다. 정확한 모든 시도는 [asset_checks.json](asset_checks.json)에 있다.

직선/L/U/T/십자/문 있는 방의 alpha coverage, world-phase 연속성, 열린 문, south-face 제한은 합성 테스트로 검증한다. face는 자기base행까지44px만 차지하고 cap16px는 별도다. 타일 한 칸 북쪽에 선 인물이 반드시 가려진다는 뜻은 아니며, 가까운 북쪽 부분셀 위치에서의 실제 가림을 별도 검사한다.

## 언덕·오두막

`hill_tiles`는 남쪽이 passable인 undug hill에만 rock_face44px와 hill texture cap16px를 표시한다. 내부는 top-down rock으로 유지하고 파낸 pocket 바닥은 기존 overlay 좌표를 쓴다.

오두막은3×3 통나무 ring·문·흙바닥으로 치환한다. 실제 세 장면에는 완료 hut이0개이므로 실제 장면에 오두막이 있었다고 주장하지 않는다. 합성 fixture와 FPS의 세 번째 room chunk에서 이 경로를 실행한다. 쉼터는 별도 front3/4 sprite이며2차 수치PASS다.

## 물·지형·구역

물은256×256 원본을 world offset으로 반복한다. 강둑은10px noise-dither alpha falloff와 육지 쪽2px 잉크선이다. 초기의 타일축 고정 검은선은 경계 수치PASS에도 눈에 띄어, 같은 seed의0–4px 육지쪽 contour변위와 코너 max결합으로 수정했다. 선폭과10px 기준은 유지했다. 계단형 강의 타일 topology 자체는 여전히 남는다. 외부의 `river_same_texture_comparison.png`는 같은 텍스처로 비교한 증거다.

지형은 고정seed value noise의12px blend, 코너max, 벽 인접 바닥은 crisp를 유지한다. 구역은8% tint와 모서리tick이며 내부 격자선은 없다. 여울은 원본4돌sheet를 네 영역으로 분리·균일scale해 코드 배치한다. `ford_assets.json`에 provenance가 있고 통짜 sheet의 스타일 실패를 돌 개별 수락으로 바꾸지 않았다.

연속 지형경계의 동일한 **비영(非零) 휘도step** run≤24px를 측정한다. 이는 등록된 수치 계기이며 자연스러움/계단형태 제거를 보증하지 않는다. 최종 장면별 측정은 아래 합성 결과와 manifest에 기록한다.

## 다시 그린 구조물·나무

원본 Route I sheet와05props/scene anchor를 첨부했다. judge-v2의 고정 family 보정으로 **원본**을 측정하고, projection은 정규화본을 측정했다. 미관 리뷰어를 쓰지 않으므로 judge의 model score는 `MISSING`, full `passed`는 false다. 아래 family PASS는 `numeric_pass`를 뜻하며 모델 심사 통과로 해석하지 않는다. 수치+투영+직접 한 줄 읽기를 만족하는 후보도 오너 승인 전이다.

|자산|선택/총 성공시도|높이px / base오차px / 광향°|family 수치|직접 읽기|
|`bedroll`|3/3|14 / 0 / 254.24|FAIL: wash_flatness|긴 옆면과 솟은 베개가 있는 입상 침구로 읽힘. 불투명98x14px, 논리2x1(원래1x2의90도 회전). 스타일 수치 실패는 유지.|
|`birch_large`|3/3|175 / 0 / 211.40|PASS|흰 줄기와 높은 성긴 수관의 자작나무로 읽힌다.|
|`birch_small`|2/2|136 / 0 / 206.14|PASS|좁은 줄기와 잎 덩어리가 구분되는 어린 자작나무다.|
|`boulder_large`|1/1|36 / 0 / 234.39|PASS|바위 앞면이 넓게 보여 높이가 있는 큰 바위로 읽힘.|
|`boulder_small`|1/1|24 / 0 / 222.46|PASS|윗면과 전면이 분리된 낮은 작은 바위.|
|`claybank_depleted`|1/1|16 / 0 / 229.20|PASS|파낸 움푹한 점토 둑과 남은 낮은 테두리가 보임.|
|`claybank_full`|1/1|16 / 0 / 222.83|PASS|긁힌 점토 절벽 전면과 낮은 상면이 보임.|
|`door_log_closed`|1/1|60 / 0 / 212.71|PASS|나무 판자와 밧줄 가로대로 닫힌 문이 읽힘.|
|`door_log_open`|3/3|60 / 0 / 215.02|FAIL: wash_flatness|통나무 문틀과 열린 판자문, 끈 경첩과 끈 손잡이로 읽힌다.|
|`door_stone_closed`|1/1|60 / 0 / 218.62|PASS|돌 문틀 안에 밧줄로 묶인 나무 문이 읽힘.|
|`door_stone_open`|2/2|60 / 0 / 219.50|PASS|돌 문틀과 열린 판자문, 끈 경첩과 손잡이가 식별된다.|
|`drying_rack_empty`|3/2|48 / 0 / 243.21|FAIL: wash_flatness|빈 직립 나무틀과 앞뒤 지지대가 보인다. 색면 수치 실패는 유지한다.|
|`drying_rack_hide`|2/2|48 / 0 / 218.52|PASS|나무틀에 가죽을 매단 직립 건조대로 읽힌다.|
|`flint_outcrop`|1/3|20 / 0 / 229.80|FAIL: wash_flatness|밝은 모암에 어두운 부싯돌 덩이가 솟아 있음.|
|`grinding_stone`|1/1|32 / 0 / 255.36|PASS|두꺼운 석재 받침의 홈과 손돌이 식별된다.|
|`hearth_1`|3/3|33 / 0 / 249.27|FAIL: wash_flatness|낮은 돌 테두리와 그 위 불꽃의 화덕이다. 색면 수치 실패는 유지한다.|
|`hearth_pots`|1/1|36 / 0 / 239.48|PASS|돌 테두리 안에 도기 세 개와 불꽃이 있는 소성 화덕이다.|
|`hearth_unlit`|1/1|20 / 0 / 248.48|PASS|돌 테두리와 재, 장작이 있는 꺼진 화덕이다.|
|`knapping_spot`|2/2|34 / 0 / 230.52|PASS|두 돌 받침 위 작업판과 석기가 놓인 작업대다.|
|`lean_to`|2/2|64 / 0 / 196.34|PASS|기둥 두 개와 뒤로 기운 가죽 차양, 사람이 들어갈 열린 앞면의 쉼터로 읽힌다.|
|`oak_large`|1/1|177 / 0 / 214.56|PASS|굵은 줄기와 넓은 수관이 있는 큰 참나무다.|
|`oak_small`|1/1|136 / 0 / 215.17|PASS|가는 줄기와 성긴 수관이 있는 작은 참나무다.|
|`pine_large`|3/3|176 / 0 / 223.25|FAIL: stroke_q90|줄기와 가지 층이 드러나는 큰 소나무다. 수치 실패는 별도 유지한다.|
|`pine_small`|1/1|135 / 0 / 210.72|PASS|좁은 원추 수관과 줄기가 있는 작은 소나무다.|
|`pottery_bench`|1/1|36 / 0 / 225.83|PASS|다리 달린 작업대 위 세 도기가 식별된다.|
|`stump`|1/1|20 / 0 / 250.54|PASS|줄기 껍질 전면과 타원형 절단면이 분리됨.|
|`woodworking_bench`|2/2|36 / 0 / 247.58|PASS|굵은 다리와 상판, 도구가 있는 목공 작업대다.|

선택된 standing27개 모두 projection PASS, 그중21개 family수치PASS다. 나머지6개는 침구·열린통나무문·빈건조대·부싯돌·불붙은화덕의wash_flatness와 큰소나무stroke_q90 실패다. 침구는98×14px로 읽히는3차를 선택했지만 목표 두타일128px 길이를 다 채우지 못하며 비등방 늘이기로 숨기지 않았다. 부싯돌2·3차의halo, 자작나무1·2차halo, 통나무문2차금속 경첩은 직접 탈락시켰다. 수치가 더 좋아도 해당 후보로 되돌리지 않는다. 빈건조대는1차 도구거절도 시도수에 포함하므로 총3회다.

추가 아이템12종은 각각1회 재각도했고, `ground_axe`, `flint_nodule`, `stone_pile`3종만 family수치+직접 읽기를 만족하여 후보 교체했다. loose item은 오너 지정 높이 밴드가 없어 projection 수락은N/A로 남겼으며, 제작용4–24px 검사 수치를 승인으로 쓰지 않았다. 나머지9종은halo 또는수치 실패로 기존 자산을 유지했다. 기존 수량버킷도 선택된 새 원본에서만 결정론적으로 재구성한다. 구체적 시도/실패/경로는 `item_assets.json`, `newitem_readings.json`을 확인한다.

기존 나무6개는 위에서 본 수관으로 읽혀 모두 재각도했다. 기존 아이템22기본+58파생 및 사체4개 감사는 [EXISTING_ASSET_AUDIT.md](EXISTING_ASSET_AUDIT.md)에 있다. 기존 사체가 있어 별도 carcass 생성 조건은 발생하지 않았으나 기존 동물의 미채택/실패 상태는 유지한다.

## 늑대 재시도

1회10포즈sheet를 생성했다. family수치10/10이지만 projection2/10이며 기존 animal 보정의 음성대조FAIL도 남는다. 서기/걷기는 더 명확하나 Srun은 pounce와 가깝고 dead는 sleep과 모호하다. `DIAGNOSTIC_CANDIDATE_NOT_ADOPTED`; 원래 동물은 교체하지 않았다. [wolf_retry.json](wolf_retry.json), `art/style05/struct/wolf_retry_sheet_1x.png` 참조.

## 합성·세 보드

원래ENV1의 day6noon·day6 02:00·winter noonscene/HUD와 동일한 declared overlay를 읽었다. 필수1×/½×6장과 보충¼×3장·day6noon 기반 drought1×/½×/¼×3장, 총12장을 생성했다. drought는 관측된 별도 가뭄simulation이 아니라 기존scene의선언된 tone조건이다.¼×는 기존LOD를 적용하고½×HUD를 재사용한 보충출력이다. 전체input snapshot불변을 확인했으며 상세SHA는 [render_manifest.json](render_manifest.json)에 있다.

|렌더 파일|측정 경계수|최대 동일step run px|≤24|분류|
|---|---:|---:|---|---|
|scene_day6_noon_day_1x_figures.png|140|2|PASS|필수|
|scene_day6_noon_day_half_lod.png|466|2|PASS|필수|
|scene_day6_noon_day_quarter_lod.png|1500|3|PASS|보충|
|scene_day6_night_night_1x_figures.png|140|2|PASS|필수|
|scene_day6_night_night_half_lod.png|466|2|PASS|필수|
|scene_day6_night_night_quarter_lod.png|1500|4|PASS|보충|
|scene_winter_noon_winter_1x_figures.png|140|3|PASS|필수|
|scene_winter_noon_winter_half_lod.png|466|2|PASS|필수|
|scene_winter_noon_winter_quarter_lod.png|1500|2|PASS|보충|
|scene_day6_noon_drought_1x_figures.png|140|2|PASS|보충|
|scene_day6_noon_drought_half_lod.png|466|2|PASS|보충|
|scene_day6_noon_drought_quarter_lod.png|1500|3|PASS|보충|

- [보드1: 현재·ENV1·STRUCT](../../../art/boards/struct_owner_board.png):같은day6noon,같은crop,1×세로/가로scale변경 없음.
- [보드2:2×확대](../../../art/boards/struct_closeups_2x.png):집·창고·파낸언덕·여울·작업자3명·명시적3×3hut시연. 집의자는인물은침구base보다3px위에놓아 침구opaque와교차하는지검사했다. 실인구증감이나실제잠자기배치관측이아닌기존ID복제시연이다.
- [보드3: 밤·겨울](../../../art/boards/struct_conditions.png):각조건의같은world영역을native1×/½×로잘랐다. 전체frame을축소해용량을맞추지않았다.

세최종보드를 직접열어 읽었다. STRUCT의벽은수평face와cap이구분되고, 쉼터는식탁형태보다기둥과기울어진차양으로읽힌다.2×에서침구아래두께와자는인물, 열린문, 파낸언덕의앞면, 여울돌을구분할수있다. 다만벽의넓은cap반복, 강의계단topology, 겨울바닥의강한무늬, 낮은침구는오너판단이필요하다. 밤은낮보다어둡고½×는기존LOD의사람점표시를유지한다. 자연스러움이나승인은주장하지않는다.

보드PNG는각각1,076,276 /821,601 /1,435,799bytes로1.5MB이하다. sidecar는원본crop·scale·SHA를기록한다.

## Crowd FPS

**PASS:304.108106FPS**,60.001689초, 완료18,247프레임과Engine18,247프레임 일치. 실제macOS 창/Godot4.6 Compatibility OpenGL4.1 Metal/AppleM4Max,1600×900·1×·vsync off·cap0.60표본 모두200명+600환경+방3개,invalid0,423입력SHA 불변이다. 환경은필수27종을 반복하며 추가아이템/늑대 진단본은 포함하지 않았다. 벽은 `walls.py`의방단위bake 결과를3chunk로 그리므로bake비용은60초에 포함하지 않는다.

실제viewport PNG를 직접 열어 확인했다. OS 창표시는 CoreGraphics로 확인됐으나OS screenshot은 공유상태0으로 실패했다. Godotviewport 시작/끝 PNG는 성공했다. 개별sprite의viewport bounds 검사는 했지만 모든sprite가 겹침 없이 보인다는 뜻은 아니다. 물리scanout·게임simulation·생산 환경performance 승인으로 확대하지 않는다.

측정JSON SHA256:`f7a384460316c8eff30e0f82b16e18b4ab99648f581afd5f56d1b6409fa571f6`. raw:`/Users/rexxa/work/artstruct-data/fps/final/`. 상세계약·실패경로검증·재현명령은 [FPS.md](FPS.md).

## 검증·생성 예산·변경 파일

- 생성78/120회: 원본77개+도구거절1회. 자산당최대3회. 필수/나무/물64원본, 선택아이템12원본, 늑대1원본이다. [generation_manifest.json](generation_manifest.json)에모든prompt·원본SHA·실제reference SHA·거절기록이있다.
- 테스트63PASS, RuffPASS, 전체새Python모듈typecheck0error·3warning. 경고는exhaustive Never pattern2개와NumPy Any1개이며오류를숨기지않았다. 실제NumPy타입을로드한최종검사에서발견한item cast5개는정수변환으로수정하고해당테스트/타입검사를재실행했다.
- 12렌더재실행pixel동일, input불변. 세보드13패널이source SHA와crop/nearest2×픽셀에정확히일치한다. 집시연인물과침구의opaque교차는375/393/366px다.
- 실제Godot60초FPS와headless거부경로를확인했다. 게임코드/bridge미변경이므로전체게임gate·simulationidentity는실행하지않았고그결과를주장하지않는다.
- 변경파일전체목록: [CHANGED_FILES.md](CHANGED_FILES.md). 생성별16-neighbour벽그림을늘리지않고face/cap/end/corner를코드연결하며, 기존ENV1scene/HUD/LOD를재사용했다. 기존자산원본은보존했다.
- [STRUCT_VALIDATION.json](STRUCT_VALIDATION.json)은검증요약과외부raw로그SHA다. 각커밋파일≤1,500,000bytes,scope검사와`git diff --check`를커밋전에확인한다. push대상은명시적`origin v8/art-struct`이며merge하지않는다.

## M-ART2에 넘길 것

|구현 함수/책임|참고 경로|
|---|---|
|벽 연결·cap/face/end/corner·door replacement|`art/compositor/walls.py`: `wall_tiles`, `render_walls`|
|언덕 남쪽 exposed face|동일파일 `hill_tiles`|
|world-sampled water·bank·ground blend|`art/compositor/edges.py`: `sample_texture`, `water_mask`, `render_tile`|
|구역8% tint·모서리tick·여울돌|동일파일 `zone_overlay`, `ford_overlay`|
|씬/overlay→벽/바닥/오두막|`art/compositor/struct_layers.py`: `build_layout`, `structural_tiles`, `wall_door`|
|base y→x 깊이정렬, RouteI anchor, 기존ENV1 LOD|`art/compositor/struct_render.py`, 기존 `drawing.py`/`placements.py` 연결부|
|raw crop/균일scale/선택·수치실패 보존|`art/pipeline_struct/normalize.py`, `ford.py`|

|자산|경로/manifest|
|---|---|
|벽·물·standing38종|`art/style05/struct/*.png`, `assets.json`, `asset_checks.json`, `asset_readings.json`|
|네 여울돌|`ford_stone_1.png`–`ford_stone_4.png`, `ford_assets.json`|
|기존채택인물|원래 `art/character/` 경로 유지|
|동물 후보|원래 `art/animal/` 유지; 늑대새sheet는 진단용|
|FPS baked room3개|`art/proto/struct/rooms/`, `rooms.json`; runtime bake비용 미포함|

이 인계는 게임 설치 승인이나 production-ready 패키지 선언이 아니다. 로그cap의 seamFAIL과6개 family수치FAIL, 기존동물미채택을 소비자가 그대로 확인해야 한다.

## 제한사항·오너 결정 필요

- **벽 높이 체감:**60px는56px성인과 수치상 비슷하나 큰 cap면·반복텍스처가 과도하게 느껴지는지는 보드로 판단해야 한다.
- **지붕 토글 기본값:**본 패스는 roof 없음.35%alpha debug overlay는 규칙만 문서화했고 게임 토글 구현은 범위 밖이다.
- **밤 밝기:**기존ENV1 tone/조명을 유지했다. 읽기 쉬움의 승인 여부는 밤 보드로 판단해야 한다.
- 파라미터화된alpha·높이·광향·seam·경계·FPS는 각자 범위의 증거다. 이들 PASS로 미관 승인, 모든 포즈 판독, 게임 성능 또는 실제population 행동을 주장하지 않는다.
- 생성 원본은 바꾸지 않았다. 로컬raw가 없는 다른 머신에서는 외부 절대경로 증거를 내려받아야 원본 재측정이 가능하다. 커밋된 normalized PNG·검사JSON·boards는 독립 열람 가능하다.
