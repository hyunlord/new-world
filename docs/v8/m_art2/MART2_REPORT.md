## 구현 완료 보고 — M-ART2

> D19 오너/플래너 색 구분 수정: GPU 119개·UI 42개 검사 통과, 같은 저장 시점 14장 재캡처.
> 물·지면 최소 휘도 차이 0.1346, 색상 거리 0.1499회전. 나무는 황록색 05 원본을 보존했으며
> 원본 대조 렌더와 채도 차이 0이다. 아래 34단계 게이트와 FPS 수치는 `32cd61a7`의 기록이다.
> D19 수정 렌더러의 새 게이트/FPS 통과로 해석하지 않는다. 지면 상한은 0.38/0.30으로 변경되며,
> 물 대비·½×/¼× 지형 색·나무 원본 채도 검사를 추가했다. 상세 근거는 `MART2_DECISIONS.md` D19 및
> `D19_EVIDENCE.json`. D19 전체 게이트·FPS·오너 시각 승인은 아직 주장하지 않는다.
> 독립 시각 판정은 88/100, REVISE: 지면·물 구분은 개선됐지만 원본 그대로인 황록색 수관은
> 요청한 초록 숲과 다르다. 나뭇잎 색상 보정 여부는 별도 오너 선택으로 남긴다.

### 구현 의도
오너가 채택한 아트(경로 I 사람, ART-STRUCT의 한 투영, ART-POLISH2의 재료 체계·벽 윗면 (b)·얼굴 변형·초상, ART-POLISH의 아이템·늑대·사슴)를 진짜 게임에 넣었다. 한 칸 64픽셀, 바닥·물·경계·언덕 면은 코드로 굽고, 벽·문·바닥은 재료 문서로 해석하고, 사람은 사람마다 정해진 모습과 얼굴 변형으로 그린다. 1배 아래에서는 사람이 상의 색 표지로 바뀌고, 낮·밤·겨울은 톤으로 표현한다. 모든 그림 설정은 `assets/v8art/render_params.json` 한 파일에 있고 F12(디버그 실행)로 바로 다시 읽는다. 시뮬레이션은 바꾸지 않았다(브리지 읽기만 추가, 정체성 시험 바이트 동일). 이번 패스가 확장성 계약(ADR 0033)을 처음 따른다: 렌더러 코드에는 내용(아이템·재료·모습·동물) 이름이 없고, 모두 매니페스트·바인딩·재료 해석기를 거친다.

### Part 0 기록 (ADR 0033 확장성 계약, 아트 채택 기록, M5.5 결정 기록)
- ADR 0033(채택): 종류는 데이터, 동작은 코드. 레지스트리·해석기·범주 기본값, 새 내용 enum 금지(불가피하면 M-EXT 빚으로 기록), 가족마다 "데이터만으로 추가" 시험, 리뷰 점검 항목. AGENTS.md 첫 화면, `worldsim-architecture` 확장성 점검표, `worldsim-godot` "렌더러는 매니페스트로 해석"에 반영. 결과 기록(이번 패스의 grep·데이터만으로 벽 시험, 남은 빚)을 ADR에 추가했다.
- ADR 0018 채택 기록: 경로 I(ART-PROD3), 한 투영(ART-STRUCT), 재료 체계와 벽 윗면 (b)·얼굴 변형 v1·초상(ART-POLISH2), 아이템·늑대·사슴(ART-POLISH) — 모두 `v8/art-polish2` `c8ed8608`에서 가져오고 아트 브랜치는 합치지 않았다.
- ADR 0034(렌더러 데이터 흐름: 스냅샷 → 해석 → 청크와 스프라이트 → 톤)는 Part 0 초안, Part 7에 측정값과 함께 채택.
- M5.5 오너 결정(수용 시 인구 상한, 교역 물건 규칙, 고기를 화덕 옆에, F4·F9 막대 산수, 이주민 거절 −5 유지)은 `MART2_DECISIONS.md` D3에 M-EXT 몫으로 기록. ROADMAP에 M-EXT 행.
- 결정 기록 D1–D18 (`docs/v8/m_art2/MART2_DECISIONS.md`).

### 가져오기 (자산 수·용량·출처 커밋·매니페스트)
- `tools/v8/art_import.py` + `art_import.json`: 명시 목록, git 객체에서 읽어 바이트 그대로 복사(`c8ed8608`), 재실행해도 같음, `--check`, `--verify`(게이트 31단계).
- `assets/v8art/`: 매니페스트 664파일 13.9 MB — 경로 I 프레임 340(36 모습, 두 모습은 대체), 초상 204(34 모습 × 6 변형, 128픽셀), 재료 텍스처 29와 문서·스키마, 바닥 텍스처 6, 물, 눈 덮개, 구조물 23, 아이템 23, 늑대·사슴 프레임 20, 나무 2, 징검돌 4, 얼굴 변형·변환 기록, 상의 색. 그 밖에 손으로 쓴 `bindings.json`(시뮬레이션 코드 → 아트 id, 내용 추가는 여기 한 곳)과 `render_params.json`, `README.md`(새 아트 커밋에서 다시 가져오는 법), `ADDING_ART.md`, `LICENSE.md`, 고친 피부·머리 마스크 340(`people/masks/`).
- 시험 `test_art_import.py` 10개(모습 격자 = `appearance.ron` 포함, 실패 사례 포함).

### 브리지와 외모 (성별·재료 열, 외모 함수, 텍스트 렌더러 일치)
- `get_agent_snapshot`에 `sexes`, `look_ids`(나이 × 성별 × 상의 × 머리, 36 모습), `variants`(얼굴 0–5, 노인 3·4) — 순수 함수 `collect_appearance`, 규칙은 `sim-bridge/data/appearance.ron`. 머리 모양과 얼굴은 AgentId의 순수 함수(저장하지 않음).
- `get_space`에 벽·바닥의 재료(u32 비트, 없음 −1), 방식(청사진 이름), 범주(재료 파일을 브리지에 내장).
- 텍스트 렌더러: `worldsim_probe --people-json`이 같은 외모를 쓴다. 게이트 23단계(렌더러 일치)가 이제 칸뿐 아니라 사람마다 모습·얼굴 변형까지 AgentId로 비교한다: PASS n = 10.
- 스냅샷 스키마 v7, 브리지 `#[func]` 목록은 그대로 97개. 정체성 시험: M5.5 기준과 바이트 동일(Part 2).

### 렌더러 (바닥 청크·경계·물·벽 재료 해석·언덕 면·사람·변형 틴트와 피부 마스크 수정·동물·물건·LOD·빛과 계절·설정 파일)
- **바닥:** 16×16칸 청크(1,024² RGBA8)를 CPU에서 Godot 기본 Image 연산으로만 굽는다. 같은 바닥이 이어진 줄은 한 번에 복사, 경계는 12픽셀 노이즈 디더 섞기(네모 경계 없음), 물가는 10픽셀 디더 둑과 2픽셀 잉크, 물은 청크의 투명 부분 아래 셰이더가 흐름과 함께 그린다. 보이는 곳만 굽고 LRU 48개, 1배 아래에서는 ½·¼ 해상도 텍스처.
- **톤:** ENV1 상한(평균 ≤ 0.32, 표준편차 ≤ 사람의 0.25배, 채도 ≤ 0.22)을 텍스처를 읽을 때 적용. 언덕 꼭대기는 `ground.value_by_ground`로 어둡게 해서 바위 덩어리로 읽힌다(리뷰 후, D18).
- **벽:** 재료 해석기(art line `resolve.py` 이식: 방식 정확 일치 → 청사진 → 범주 기본 → 문서 기본; 색조는 재료 → 범주 → 방식)로 텍스처를 고르고, walls.py (b) 방식으로 줄마다 띠를 굽는다: 벽 방향으로 누운 윗면, 남쪽이 열린 곳에만 앞면, 앞면 끝에 마구리, 열린 가장자리에 잉크. 띠는 y 정렬된 `ColonyView/World`에 사람과 함께 선다. 문은 서 있는 스프라이트.
- **언덕 면:** 남쪽이 지나갈 수 있는 언덕 칸에만 16픽셀 언덕 윗면 + 바위 면. 판 굴 바닥은 다진 흙(리뷰 후).
- **사람:** 발 위치에 놓인 노드, 경로 I 프레임 계획(서기·걷기 번갈아, 일, 나르기, 잠), 동·서는 뒤집기, 북 = 남(N 그림 없음). 얼굴 변형은 셰이더가 마스크 픽셀마다 art line과 같은 Lab 이동을 적용(잉크 보호), 노인은 회색 머리 목표.
- **피부 마스크 수정:** `tools/v8/art_masks.py`가 art line 마스크에서 옷 색 군집·옷/소품 다각형·영역 밖 픽셀을 뺀다(더하지 않음). 찾은 것: `c8ed8608`의 v3 마스크는 이미 털망토를 피했고, 지운 것은 경계의 어두운 피부 177 + 영역 밖 3, 머리 54픽셀(피부 99.72 % 유지). 시험 9개, 340 프레임 전부.
- **동물·물건:** 늑대·사슴 프레임과 자세, 물건은 개수 구간(1 / 2–8 / 9+), 사냥감 시체는 그 동물의 죽은 프레임, 구조물·작업장·화덕 상태·침상·자원 노드·구역·표시·청사진 유령·방문자·방 — 모두 바인딩에서.
- **LOD:** 1배 아래에서 사람 몸을 그리지 않고 MultiMesh 한 번으로 상의 색 원(½× 10px, ¼× 6px, 잉크 테두리)과 ½×의 활동 아이콘(도끼·통나무·zz·!)을 그린다.
- **빛과 계절:** 밤 = CanvasModulate 하나(값 × 0.55, 푸른 기운), 불 켜진 화덕마다 5칸 안을 되살리는 가산 빛. 불에서 먼 사람에게 옅은 1픽셀 테두리(작업 규칙 5, D17) — 밤의 사람/바닥 대비 중앙값 0.047 → 0.204(기준 0.15). 겨울 = 바닥 채도 35 % + 눈 덮개(≤ 60 %), 가뭄 = 풀색을 황토 쪽으로.
- **UI:** 인스펙터에 그 사람의 모습·얼굴 변형 초상(128px)과 "가죽 랩 · 묶은 머리 · 얼굴 5" 같은 말, 위 막대의 "확대 1×", 교역 창·작업장 주문 버튼의 32px 물건 아이콘, 명령 막대 아이콘, 64픽셀 청사진 미리보기. 소집한 사람은 빨간 2픽셀 테두리.
- **설정 파일:** 카메라·바닥·물·벽·사람·LOD·빛·계절·구역·현장·이름표·디버그. F12로 다시 읽으면 "렌더 설정 다시 읽음"(벽·언덕 윗면까지 다시 굽는다 — 리뷰에서 찾은 결함을 고침).

### 확장성 검사 (리터럴 grep, 데이터만으로 벽 추가 시험, 해석기 일치)
- `tools/v8/test_mart2_extensibility.py`(게이트 33단계): `scripts/world/**/*.gd`의 문자열 리터럴이 매니페스트 id·모습·재료·방식·바인딩의 종류 이름과 같으면 실패, 칸 크기 자리의 `16`이 있으면 실패. 둘 다 심은 줄로 실패를 확인. 한계(작은따옴표, 이어 붙인 문자열, 셰이더·UI 스크립트는 안 봄)와 명령 막대의 `"hut"` 같은 청사진 명령 이름은 M-EXT 빚으로 D16에 기록.
- 데이터만으로 벽: `tools/v8/fixtures/data_only_wall.json`(새 재료 `test_river_mud` + 새 청사진 → 방식 `mud_brick`)이 코드 수정 없이 해석되고 그려진다(게이트 34단계).
- 해석기 일치: `material_resolver.gd` = art line `resolve.py`, 39 사례 0 차이(게이트 32단계). 리뷰 후 30 → 39 사례로 늘려 "정확 일치가 종류를 무시" "호출자 범주가 재료 범주를 덮음" 변이를 잡는 것을 확인(15·5 차이).

### 성능 (장면·줌별 FPS, 청크 재굽기 시간, 텍스처 메모리, 틱 비용 불변)
측정: `tools/v8/mart2_fps.sh`, 창 1920×1080, vsync 끔, 릴리스 라이브러리, 30초 예열 + 60초, 1분 부하 2.9–5.7(경합 아님), 커밋 `bab5ec4a`(리뷰 수정은 그리기 경로를 바꾸지 않음).

| 장면 | 줌 | FPS | 기준 | 텍스처 메모리 |
|---|---|---|---|---|
| 시드 0, 20일째(12명) | 1× | 328.6 | ≥ 120 | 265 MB |
| 〃 | ½× | 238.2 | ≥ 120 | 176 MB |
| 〃 | ¼× | 120.65 | ≥ 120 | 87 MB |
| 1,000명 세계 | ½× (LOD) | 186.8 | ≥ 60 | 82 MB |
| 〃 | ¼× (LOD) | 173.5 | ≥ 60 | 62 MB |

- 청크 굽기(20일째 세계 144청크): 새로 굽기 중앙값 1.8 ms, 90번째 2.4 ms, 최대 5.3 ms(144개 중 1개가 4 ms 넘음); 다시 굽기 중앙값 2.1 ms, 최대 3.7 ms(기준 4 ms 통과); 파기 한 칸 = 공간 적용과 다시 굽기 1.4 ms. 불러온 직후 첫 굽기는 한 번 17.5 ms.
- 처음 고친 것: 1,000명 ½×에서 사람 원을 하나씩 그려 27 FPS → MultiMesh로 211 FPS; ¼× 텍스처 546 MB → 축소 텍스처로 87 MB; 첫 청크 굽기 258 ms → 18 ms.
- 시뮬레이션 틱 비용은 그대로(Rust 동작 변경 없음; 게이트의 정체성·벤치 단계).

### 화면 비교 (옛 렌더러 vs 새 렌더러, 장면별)
같은 저장 파일을 옛 렌더러(M5.5 기준 `29f720aa`, 16픽셀)와 새 렌더러로 같은 땅 넓이에 찍었다: `docs/images/mart2_compare_sheet.png`(6쌍), 새 화면 12장 `docs/images/mart2_contact_sheet.png`, 14장 전체의 틱·위치·해시 `docs/v8/m_art2/screens/SCREENS.md`.
- 6일째 정오 1×: 옛 화면은 풀·숲 바닥을 칸마다 다른 초록으로 칠하고 사람이 색 상자다. 새 화면은 바닥이 차분한 한 톤이고 사람·물건·오두막·작업장이 잉크 그림이며, 사람마다 모습과 얼굴이 다르다.
- 02:00 밤: 둘 다 어둡고 화덕 둘레만 밝다. 새 화면은 벽 집 안의 잠든 사람들이 옅은 테두리로 보인다.
- 첫 겨울 정오: 옛 화면은 계절이 바닥에 안 보였다. 새 화면은 바닥이 바래고 눈 얼룩이 덮인다(art line 겨울 판과 같은 모습). 나무·물건은 제 색.
- 창고 2×: 통나무 벽 윗면이 벽 방향으로 눕고 남쪽에만 앞면이 있다. 안의 물건 더미가 종류별 그림.
- 작업장 2×: 일하는 세 사람이 도끼질·나르기 프레임으로 보인다.
- ¼×: 옛 화면은 칸이 작은 점이 된다. 새 화면은 숲이 덩어리, 사람은 상의 색 점.

### 지운 것 (M1 시트 경로 등)
- `scripts/world/colony/terrain_layer.gd`(16픽셀 지형).
- M1 시트 경로: `tools/v8/gen_tiles.py`, `tools/v8/test_gen_tiles.py`, `assets/v8/tiles.png`, `assets/v8/tiles.json`, `economy_text.gd`의 `ITEM_SPRITES`, `porter_presentation.gd`의 16픽셀 짐 아이콘과 경로 그리기(부르는 곳 없음). ADR 0015는 대체됨으로 표시.
- `docs/images/m55_contact_sheet.png`(M5.5 시트 → `mart2_contact_sheet.png`).
- 남긴 것: `art_sprite_library.gd`와 `assets/art/atlas`는 위 막대 32px 아이콘과 `wood_visuals.gd`가 아직 쓴다(D16, M-EXT에서 매니페스트로 옮김).

### 변경된 파일 목록 (커밋별)
- `a872345d` (p0): MART2_GOAL·DECISIONS, ADR 0033·0034·0018, ROADMAP, HISTORY, AGENTS.md, 스킬 둘.
- `fbff5a54`, `3ce3909e` (p1 1–2/5): `assets/v8art/people/frames/` 어른·아이 모습 340 PNG.
- `ef6e0845`, `93f6d2db` (p1 3–4/5): `assets/v8art/people/portraits/` 204 PNG.
- `388eae12` (p1 5/5): `tools/v8/art_import.py/.json`, `test_art_import.py`, `assets/v8art/` 재료·구조물·아이템·동물·바닥, `manifest.json`, `bindings.json`, `render_params.json`, `LICENSE.md`.
- `6da9dd26` (p2): `sim-bridge`(`ffi/appearance.rs`, `material_category.rs`, `space.rs`, `node.rs`, `mod.rs`, `build.rs`, `data/appearance.ron`, Cargo.toml), `sim-test`(`probe/ascii.rs`, `engine_appearance_ffi.rs`, `engine_space_ffi.rs`), `snapshot_schema.md`, ARCHITECTURE, CHANGELOG.
- `cbaa8894` (p3): `scripts/world/art/`(art_data, material_resolver, ground_baker, wall_baker, tone), `ground_layer.gd`, `view.gd`, `camera_controller.gd`, `shaders/ground_chunk.gdshader`, `scenes/main.tscn`, 지역화(ko/en), `terrain_layer.gd` 삭제, `tools/v8/`(resolver_parity.py/.gd/fixture/expected, mart2_ground_test.gd, mart2_chunk_hashes.json, test_mart2_extensibility.py, fixtures/data_only_wall.json), `assets/v8art/README.md`·`ADDING_ART.md`.
- `202d5a9a` (p4): `agent_layer.gd`, `lod_markers.gd`, `shaders/person_variant.gdshader`, `daylight_controller.gd`, tree/building/overlay/room/animal/visitor 레이어, `site_ghost.gd`, `tools/v8/art_masks.py`, `test_art_masks.py`, `mart2_people_test.gd`, M1–M5 UI 시험과 `smoke.gd`.
- `5054a583`, `37fa5c3b` (p4 마스크): `assets/v8art/people/masks/` 340 PNG + `MASKS.json`, `skin_regions.json`.
- `7ad5872d` (p5 일부): 인스펙터, 위 막대, 명령 막대, `item_icons.gd`, 교역 창, 작업장 창, `order_tool.gd`.
- `498a4905` (p6 일부): M1 시트 경로 삭제, `gate.sh`(31–34단계), `renderer_agreement.sh/.gd`, `capture.gd`, `mart2_shots.gd`, `mart2_old_shots.gd`, `mart2_fps.gd/.sh`, LOD MultiMesh.
- `98a1f21d` (p5): `mart2_ui_test.gd`, 밤 테두리(`agent_layer.gd`, `daylight_controller.gd`, params), 작업장 아이콘 누수 수정, D15–D17.
- `bab5ec4a` (p6): 화면 시트 둘, `SCREENS.md`, README, CHANGELOG, ARCHITECTURE 14절, ADR 0015·0033·0034, AGENTS.md, 스킬 다섯, `renderer_agreement.gd` 머리말.
- `32cd61a7` (p7 리뷰): `ground_layer.gd`, `agent_layer.gd`, params·bindings, 해석기 사례, `mart2_ui_test.gd`, `mart2_shots.gd`, 화면 다시 찍기, D16·D18.
- (보고 커밋): 이 보고서, ROADMAP, HISTORY, PERF_LEDGER(M-ART2 줄), ARCHITECTURE(스키마 v7 줄).

### 리뷰 결과 (코드·화면)
보고서 파일: `/Users/rexxa/work/mart2-data/reviews/code_review.md`, `visual_review.md`. 처리 내역은 D18.
- **코드(판정 "변경 요청"):** 막힘 1 — `HALO_STEPS: float = 16.0`이 16픽셀 grep에 걸려 게이트 16·33단계가 `bab5ec4a`에서 빨강 → 정수로 고침. 고칠 것 5 — F12가 벽을 지움(S1), 계절·F12 뒤 언덕 윗면이 옛 톤(S2), 청크 모서리의 벽 마구리가 안 바뀜(S3): 셋 다 고치고 S1·S2는 UI 시험에 넣어 옛 코드에서 실패하는 것 확인; grep 한계(S4)는 빚으로 기록; 해석기 사례 부족(S5)은 39 사례로 늘림(art line 데이터에 없는 두 경우는 남음). 확인된 건전함: sim-core·systems·engine 변경 없음, 브리지에 `unwrap` 없음, 마스크는 줄기만 함, 서 있는 것은 모두 y 정렬 노드.
- **화면(판정 "받아들일 만함", 막힘 없음):** 한 투영·(b) 벽·디더 경계·물가 잉크·옷 안 물듦·겨울·LOD가 판과 맞음. 고칠 것 3 — 언덕이 바닥처럼 보임(꼭대기를 어둡게), 판 굴이 굴처럼 안 보임(바닥을 다진 흙으로), 소집 빨간 테두리가 1×에서 약함(2픽셀); 이주민 장면 구도(대화창 아래로) — 모두 고치고 화면을 다시 찍음.

### 확인된 제한사항
- ¼× FPS 여유가 얇다(120.65 / 120): 지도 전체의 서 있는 그림 2,252번 그리기. 미달하면 계획의 3단계(¼×에서 나무를 청크에 굽기)를 한다.
- 멀리 줌아웃해 처음 보는 곳이 열리면 새 청크를 한 프레임에 다 굽는다(어림 40청크 × 1.8 ms ≈ 70 ms 한 번; 측정 안 함). 불러온 직후 첫 굽기 17.5 ms.
- 사람 N(북) 그림이 없어 북쪽으로 걸으면 남쪽 그림. 상의는 세 가지.
- 노인 초상에는 회색 머리가 없다(art line 초상 파일 그대로); 지도에서 노인 회색이 푸른 회색으로 보인다는 화면 리뷰 의견.
- LOD 표지는 대부분 같은 황갈색 상의라 비슷해 보이고, ½× 아이콘이 작다(약 12×7 px).
- 위 막대 아이콘은 아직 옛 아틀라스; `economy_text.gd`의 물건 이름 키, 명령 막대의 청사진 명령 이름, grep의 한계는 M-EXT 빚.
- 지붕 디버그 토글(버릴 수 있는 목록)은 하지 않았다. 나무 왕관이 사람을 가릴 때 흐리게 하는 것도 없다.
- `mart2_ui_test.gd`(창 필요)는 게이트 밖에서 손으로 돌린다: 42 검사 통과.
- `tools/v8/verify_v8_0.gd`(M1 화면 검사, 게이트 밖)는 16픽셀 기준 그대로(M1 기록).

### 오너 결정 필요 (설정값 조정, N 방향, 상의 추가, 지붕 토글)
1. 설정값: 언덕 꼭대기 어둡기(`ground.value_by_ground.hill_rock` 0.6), 밤 테두리(`people.night_halo_alpha` 0.8 — 0이면 끔), 벽 윗면 밝기(`walls.top_value` 0.80), 눈 덮개(`season.snow_alpha` 0.6) — 게임 안에서 F12로 보며 고르면 된다.
2. N 방향 그림: 지금은 북쪽으로 걸어도 남쪽 그림. art line에 N 프레임을 주문할지.
3. 상의 추가: 지금 세 가지(가죽 랩·털망토·허리 두르개). 더 늘릴지 — 늘리면 바인딩·마스크만 추가하면 된다.
4. 지붕 토글: 방 안을 지붕 없이 보여 준다. 지붕을 35 % 투명하게 보이는 디버그 토글을 만들지.
5. ¼× FPS가 기준 바로 위: 나무를 ¼× 청크에 굽는 작업을 지금 할지, 미달할 때 할지.
6. 나무 왕관이 사람을 가릴 때 왕관을 흐리게 할지.

### Gate 결과
`BENCH_PASS=M-ART2 bash tools/v8/gate.sh /Users/rexxa/work/mart2-data/gate1 --base-data /Users/rexxa/work/m55-data/base/identity`, 깨끗한 커밋 `32cd61a7`(dirty 0)에서 **34단계 모두 통과**, 11:47:16–13:24:44 KST, `status 0`.
- 테스트: debug 2,024 통과 0 실패, release 2,024 통과 0 실패; clippy, 금지 수학 0, GDScript 엄격 검사, 스모크 통과.
- 정체성(M5.5 기준과): 요약 228,748 값 0 차이, 연대기 64 파일 같음, 원시 표본 64 파일 같음, 결정론 PASS — 시뮬레이션 변경 없음.
- perf: OK(+10 % 넘는 평균 없음). bench: 조용한 상태(부하 7.05 < 8), 모든 장면 ×1.016–×1.053 이내, alloc/agent 1,871 B 그대로, 상한 6개 PASS. M-ART2 줄 6개를 `docs/perf/PERF_LEDGER.md`에 추가.
- 23 renderer_agreement: PASS n = 10, 외모 10명 일치. soak, nav_bench(135.7 µs/틱), knowledge(F10 8/8), story, neighbours 통과.
- 31 art_manifest, 32 resolver_parity(39 사례 0 차이), 33 extensibility_grep, 34 mart2_render(16 검사) 통과.
- 게이트 밖: `mart2_ui_test.gd`(창) 42 검사 통과(`32cd61a7`과 같은 코드).
- 게이트 뒤 `v8/main`을 `29f720aa` → `32cd61a7`로 빨리 감기했다. 이 보고서·ROADMAP·HISTORY·PERF_LEDGER는 그 위의 문서 커밋이고, `v8/main`을 다시 그 커밋으로 빨리 감는다(코드는 `32cd61a7`과 같음).

git ls-remote 결과
게이트 통과 뒤(빨리 감기 직후):
```
32cd61a77cb26886be5f13b9924fa55b7d64cbe0	refs/heads/v8/main
32cd61a77cb26886be5f13b9924fa55b7d64cbe0	refs/heads/v8/wip
```
문서 커밋 뒤의 결과는 최종 메시지에 적는다.
