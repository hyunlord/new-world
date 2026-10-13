# Art Pass 7: 다중 마을 UI 감사

Part 1 정적 조사. 아래 위치는 Art Pass 6 렌더링 코드의 수정 전 줄 번호다. `scripts/**`, `scenes/**`, `ingame_qa*.py`, `measure_p6.py`의 settlement / formation / hearth / village 참조와 관련 호출부를 조사했다. `graft ask --source`는 Rust 중심 결과만 반환했고 `graft callers ... --in scripts/`는 인덱스 범위 없음으로 실패하여 실제 GDScript를 읽었다. 계획된 수정은 검증 완료를 뜻하지 않는다. Part 9의 테스트·화면·A30 결과가 최종 판정이다.

| ID | 수정 전 위치 | 2~3개 마을에서의 동작 / 문제 | 처리 및 근거 |
|---|---|---|---|
| U01 | `scripts/ui/wood_visuals.gd:6` | 화덕 타일에서 가장 가까운 formation을 고른다. 버려진 화덕도 다른 마을의 장작·불 상태를 물려받는다. | Part 1 필수 수정: formation 타일이 같은 행만 반환, 미일치 `{}`. |
| U02 | `scripts/ui/wood_visuals.gd:30`, `:33`, `:43` | 빈 상태는 cold_ash=false, 켜진 화덕 애니메이션과 야간 광원으로 해석된다. 정확 매칭만 고쳐서는 꺼지지 않는다. | Part 1: 미일치 상태는 재·불 없음·광원 0. 부분 열 누락도 다른 마을로 추측하지 않는다. |
| U03 | `scripts/ui/building_renderer.gd:136`, `:311`, `:357` | 각 화덕을 순회하지만 U01을 통해 상태를 가져온다. `:321`의 fire_lit 기본값 true도 잘못된 연기의 원인이다. `:357`은 화덕 장작 더미 상태다. | Part 1: 정확 타일 매칭 전파, 기본 fire_lit=false. 장식 fallback 한계는 U39. Part 4: retired hearth/건물 표현. |
| U04 | `scripts/ui/daylight_controller.gd:54`, `:62` | 모든 화덕에 개별 광원은 있으나 U01의 이웃 마을 상태를 쓴다. | Part 1 공통 helper 수정으로 해결. 각 화덕 광원 상태를 `test_every_hearth`로 확인. |
| U05 | `scripts/ui/world_effects.gd:108`, `:114` | 모든 건물 순회, 화덕 메타 fire_lit 기본 true여서 미소속 화덕도 연기·불티 가능. | Part 1: false 기본값과 U03 메타 전파. 연기 없음은 생략 불가. |
| U06 | `scripts/ui/camera_controller.gd:180` | 함수 이름은 first지만 member_count 최대, 동률이면 낮은 ID를 선택한다. | 유지 가능: 최초 정착/리셋 때만 초기 카메라 중심을 잡는 명시적 탐색 기본값. `_initial_center_pending=false` 후 새 마을 출현이 카메라를 강제로 빼앗지 않는다. Part 0 촬영 target 선택은 별도. |
| U07 | `scripts/ui/nature_layout.gd:19` | 모든 건물·공사·formation 타일을 clearings에 넣는다. 첫 마을만 비우지 않는다. | 무해: 모든 마을을 순회하며 버려진 건물도 빈 공간을 유지한다. |
| U08 | `scripts/ui/terrain_renderer.gd:133` | formation 배열이 fingerprint에 들어가므로 창립/소멸 때 전체 decal을 제거하고 재생성한다. | 유지 근거: 정착 이벤트 시 갱신이며 동일 fingerprint 프레임은 즉시 반환. 성능 무해함은 아직 미확인, Part 8 A30 실제 측정 필요. |
| U09 | `scripts/ui/nature_renderer.gd:90` | formation 변경 시 지도 전체 타일을 다시 순회·reconcile한다. 개별 노드는 키로 재사용한다. | U08과 같은 이벤트성 경로. 마을 누락은 없고 성능은 A30에 남긴다. |
| U10 | `scripts/ui/settlement_overview_renderer.gd:42`, `:64`, `:94` | 모든 formation에 고정 반경 원은 있지만 색이 모두 동일하고 이름/선택 없다. | Part 4: 번호 색과 이름을 label layer에서 전달. getter는 settlement snapshot만, OVERVIEW_ALPHA=0.28 유지. |
| U11 | `scripts/ui/zoom_lod_controller.gd:1` | 개별 마을이 아닌 렌더 계층 전체의 가시성을 zoom tier로 바꾼다. | 무해: 모든 마을에 동일 줌 정책. Part 4 라벨은 독립적으로 모든 줌에서 표시. |
| U12 | `scripts/ui/panels/hud_topbar.gd:115`, `:145` | settlement snapshot의 전체 ids 수를 센다. 한 마을도 항상 표시하며 요구하는 새 문구 없음. | Part 4: HUD_SETTLEMENTS_FMT, 두 마을 이상에서만 표시. |
| U13 | `scripts/ui/panels/hud_topbar.gd:160` | 식량·장작은 모든 마을의 stock을 합산한다. | 무해: 세계 전체 자원 합계다. 주민·건물·추위 수도 전체 snapshot 순회. |
| U14 | `scripts/ui/panels/hud_topbar.gd:194` | 네 사망 원인만 합산, 노출 사망 누락. | Part 2: 다섯 키 명시 목록, prefix 스캔 금지. |
| U15 | `scripts/ui/panels/hud_status_panel.gd:74`, `:106` | 모든 ID는 추적하지만 원시 SettlementId로 formed/dissolved를 알리며 분열 과정은 말하지 않는다. | Part 5: 최초 formed, daughter founded, departed/cancelled/abandoned를 사회 로그에서 중복 없이 처리, UI 번호 사용. |
| U16 | `scripts/ui/panels/hud_status_panel.gd:100`, `:103` | 마을별 식량 알림은 sid 자체를 문자열 인자로 보낸다. | Part 4~5: 동일 UI 번호 매핑 사용. 저장량 추적 딕셔너리 키는 원시 ID 유지. |
| U17 | `scripts/ui/panels/hud_status_panel.gd:40`, `:166`, `:250` | 노출 이유 4를 버리고 화면 4줄 이외 기록은 남지 않는다. | Part 2: exposure key, reset만 지우는 feed_log. 4줄 한도 유지. Part 0 동기 advance마다 ingest하여 사망 보존 시간 누락 방지. |
| U18 | `scripts/ui/death_viz_renderer.gd:101` | 미지 이유/노출을 전투 빨강으로 그림. | Part 2: exposure 얼음색, unknown 회색 및 한 번 로그. |
| U19 | `scripts/ui/panels/agent_inspector_panel.gd:142` | 주민 상세에는 소속 마을 행이 없다. | Part 3 AgentId join, Part 4 마을 이름 표시. entity bits와 AgentId 혼동 금지. |
| U20 | `scripts/ui/agent_renderer.gd:421`, `:451`, `:486`; `scripts/ui/agent_pose_moments.gd:25`; `scripts/ui/camp_seats.gd:52` | 모든 화덕 주변에서 활동·식사·자리 배치를 계산한다. 지리적 근접성이지 소속 표시가 아니다. | 기존 연출은 무해: 단일 화덕으로 제한하지 않음. Part 4 소속 outline은 사회 snapshot으로 별도 join. Part 5 founder 짐은 실제 적재/식사/수면 우선. |
| U21 | `scripts/ui/building_renderer.gd:210`, `:231` | 모든 마을 stock을 각 formation에서 가장 가까운 stockpile에 할당·합산한다. 공유 anchor도 합산한다. | 유지: ID를 소유권으로 추측하지 않는 시각적 자원 anchor 정책. 어느 마을의 것인지 나타내는 라벨로 사용하지 않는다. |
| U22 | `scripts/ui/building_renderer.gd:471` | wood_target은 anchor에서 가장 가까운 formation 목표만 선택. 공유 stockpile에서는 stock 합계/단일 목표가 될 수 있다. | 기존 정책 유지하되 한계 기록: 실제 저장소 소유권 getter 없음. 다중 마을 identity와 혼용 금지. A30/자원 검증에서 공유 anchor 관측 시 불일치를 숨기지 않는다. |
| U23 | `scripts/ui/panels/god_hand_panel.gd:92`, `:104` | 항상 -1 대상, 제목·toast에 대상 마을 없음. | Part 4: 선택 ID(미선택 -1)와 일치하는 제목/toast, Part 8 A29 실제 rain ids 확인. |
| U24 | `scripts/rendering/art/god_hand_effects.gd:62` | rain_settlement_ids와 모든 settlement ids를 맞춰 각 지역 강우를 그림. | 무해: 이미 다중 마을 지원. 선택 UI와 실제 cast 결과가 맞아야 함. |
| U25 | `scripts/rendering/art/god_hand_effects.gd:148` | gift/spoil의 저장고 flash는 무조건 가장 낮은 ID 주변을 고름. | Part 4 수정 대상: 실제 시전 대상 ID를 기록해서 flash anchor에 사용. 이후 선택 변경으로 시전 대상이 바뀌면 안 됨. |
| U26 | `scenes/main.tscn:1`, `:11` | overview 노드는 한 개지만 그 노드가 모든 formation을 그림. load_steps=21 정적 핀. | 무해: renderer 노드 수와 마을 수는 무관. Part 4~5 새 레이어는 코드로 생성해 scene 핀 유지. |
| U27 | `scripts/test/capture_framings.gd:49`, `:98`, `:251` | 한 기본 세계, 최대 마을 framing, society 계획 없음. | Part 0 RESET, target orig/daughter/leader/all, 두 프레임 대기. Part 7 FD/FS/FV 측정 tick 사용. |
| U28 | `scripts/test/capture_framings.gd:171`, `:209`, `:243`, `:355`, `:369` | 시전 target=-1 기록, 자연/개입 두 분류만 있으며 소속·이름·trail 메타 없음. | Part 0/7: 실제 선택 ID, 사회 시드 scenario 3개 출력 지점, society·selection·feed·death metadata. |
| U29 | `scripts/test/capture_framings.gd:313`, `:447` | advance chunk 사이 feed ingest 없음. 프레임 측정은 art, zoom 1만. | Part 0 ingest; Part 8 같은 부하·FV 줌에서 1/3마을 300프레임 back-to-back. |
| U30 | `scripts/test/art/capture_metadata.gd:39` | 선택한 village 위치에 가까운 deadwood를 선택. | 무해: 호출자가 target village를 명시하면 그 주변을 잡는다. 전역 소속 판정 아님. |
| U31 | `tools/art_pipeline/capture_art.sh:8`, `:95`, `:106` | whitelist에 society 없음. art_god 외 계획은 combined manifest 덮어씀. | Part 0 society whitelist 및 combined 제외. 자연 live 자료와 사회/개입 자료 분리. |
| U32 | `tools/art_pipeline/ingame_qa_wood_p6.py:73`; `ingame_qa_wood_p4.py:68` | 검증 코드도 화덕을 가장 가까운 마을에 연결하여 같은 오판을 통과시킴. | Part 8 `_p7` 복사본에서 정확 타일 매칭. 미일치는 fire=false·재·연기/광원 없음. 옛 파일 변경 금지. |
| U33 | `tools/art_pipeline/ingame_qa_wood_p6.py:90`, `:116`, `:128` | stockpile 할당은 모든 마을 합계, 목표는 최근접, HUD는 전체 합계. | U21/U22 정책과 동일하게 검증. 매칭 규칙을 바꾸면 검증도 같은 의미로 갱신. |
| U34 | `tools/art_pipeline/ingame_qa_observed_p6.py:84`; `ingame_qa_pixels.py:135`; `ingame_qa_wood_p6.py:17` | 화덕별 ID/상태를 검사하며 모든 visible hearth를 순회. | 다중성 자체는 무해. `_p7` 관측본에서 미소속 재도 포함하고 사회/개입을 live 판정에서 제외. |
| U35 | `tools/art_pipeline/ingame_qa_cold_p6.py:60` | 전체 target 존재로 검사를 켜고 각 보이는 장작 라벨의 자체 target 검사. | 무해: 첫 마을 값만 읽지 않음. `_p7` 복사본으로 실행. |
| U36 | `tools/art_pipeline/ingame_qa_atlas.py:78`; `ingame_qa_camp_p3.py:23` | settlement domain의 모든 formation 좌표로 coverage 판단. | 무해: 첫 행 선택 없음. 이전 자료는 읽기만. |
| U37 | `tools/art_pipeline/measure_p6.py:34`, `:46`; `ingame_qa_p6.py:167`; `ingame_qa_observed_p6.py:23` | production6 쓰기 경로/관측 경로와 manifest 전체 취급. society를 섞으면 자연 A1~A25 오염. | Part 8 `_p7`와 capture contract 복사, production7만 출력, live natural만 A1~A25, society는 A26~A30 및 별도 A18. |
| U38 | `scripts/test/art/test_structure_states.gd:42`, `:54`, `:62` | 최근접·빈 state의 켜짐/glow를 명시 기대. | Part 1 기존 기대를 정확 매칭/꺼짐으로 변경. `test_every_hearth`로 서로 다른 두 상태와 미소속 세 번째 확인. |
| U39 | `scripts/ui/building_renderer.gd:357`, `:378` | 장작 열 없는 화덕은 stage=-1의 장식 장작 fallback을 둔다. 추가 장식은 전체 shelters의 첫 건물을 기준으로 둔다. | 잔존 장작/장식은 재·불 상태와 별개이며 abandoned 건물이 남는 규칙과 양립한다. 최초 쉼터 장식은 소속이나 자원량을 뜻하지 않으므로 유지한다. 모든 마을의 필수 구조물/이름표 대체 수단으로 쓰지 않는다. |

## 검증 경계

이 문서는 수정 전 조사와 처리 배정이다. U08/U09 성능, U22 공유 stockpile 목표 의미, 실제 딸 마을 강우 및 화면 가독성은 코드 읽기만으로 PASS로 판정할 수 없다. Parts 7~9의 metadata·테스트·동일 부하 측정·블라인드 판정을 붙여야 한다. 카메라 초기 중심, 세계 합계 HUD, 모든 타일 clearing, renderer 노드 한 개는 단일 마을 가정으로 오해하지 않고 위 근거로 유지한다.

## Part 1 구현·검증 결과

U01~U05와 U38의 Part 1 범위를 구현했다. `wood_visuals.settlement`는 정확한 formation 좌표만 고르며 미일치는 `{}`다. 불 상태가 없는 화덕은 계절·시간에 상관없이 S1_16 재, cold_ash=true, 광원 0으로 처리한다. `building_renderer`의 fire_lit 메타와 `world_effects`의 메타 누락 기본값은 false다. 미소속 화덕의 장식 장작 fallback은 유지되며 실제 재고를 나타내지 않는다(U39). `test_structure_states.gd`의 최근접/빈 상태 광원 기대와 `test_cold_hearth.gd`의 빈 상태 재 기대를 새로운 계약에 맞게 수정했다. 사용처가 테스트뿐인 `Structures.hearth_frame` 위임 함수는 그대로 두었으며 상태 없는 호출의 반환은 재로 바뀐다. 실제 lit 상태의 4개 애니메이션 프레임 검증은 유지한다.

새 `scripts/test/art/test_every_hearth.gd`는 실제 building renderer를 snapshot double로 구동해 서로 다른 두 마을의 장작·불 메타/스프라이트, 인접한 미소속 화덕의 재, 개별 광원 계산, 실제 world_effects의 연기, 기존 lit 마을 소멸 뒤 연기 제거를 검증한다. 세 테스트는 각각 `run_bounded.sh 120`의 headless Godot로 실행했고 exit=0이며 SCRIPT ERROR가 없었다.

| 검증 | 결과 | 로그 |
|---|---|---|
| 두 마을·미소속·소멸 화덕 | EVERY_HEARTH failures=0 | `gates/p1/every_hearth.log` |
| 기존 건물·장작 표현 회귀 | ART STRUCTURE STATES PASS | `gates/p1/structure_states.log` |
| 겨울 재·연기 회귀 | COLD_HEARTH failures=0 | `gates/p1/cold_hearth.log` |
| 변경 파일 whitespace | git diff --check exit=0 | 명령 실행 결과 |

이 결과는 렌더러 동작 테스트이며 창 화면/블라인드 가독성 판정이 아니다. U03의 abandoned 건물 외형과 Parts 2~8 항목은 여전히 담당 Part의 검증이 필요하다.

## Parts 1~6 이후 처리 상태 (Part 7 시작 시 재검토)

위 원본 조사·Part 1 기록은 역사로 유지한다. 아래는 현재 구현을 다시 읽고 추가한 처리표다. `구현`은 코드와 명시한 테스트 근거이며 A26~A30의 화면/성능 PASS를 뜻하지 않는다. `유지`는 단일 마을로 제한되지 않는 이유가 확인된 기존 동작이다. 촬영·측정 도구가 병행 수정 중인 U27~U37은 실제 Part 7~8 출력 전까지 보류한다.

| ID | 처리 상태 | 현재 근거와 남은 검증 |
|---|---|---|
| U01 | 구현 | `scripts/ui/wood_visuals.gd:6` 정확 좌표 매칭. `test_every_hearth.gd`, p1/every_hearth.log failures=0. |
| U02 | 구현 | `scripts/ui/wood_visuals.gd:27` 미소속 재, `:39` 광원 0. p1/cold_hearth.log와 every_hearth.log failures=0. |
| U03 | 구현 | `scripts/ui/building_renderer.gd:321` 기본 fire=false. `scripts/ui/settlement_labels.gd:159` abandoned shader 적용. p4/abandoned.log failures=0; 실제 자연 소멸 화면은 대기. |
| U04 | 구현 | `scripts/ui/daylight_controller.gd:57` 공통 정확 매칭 helper 소비. `test_every_hearth.gd` 개별 lit/unlit/orphan 광원 검사. 실제 밤 화면은 대기. |
| U05 | 구현 | `scripts/ui/world_effects.gd:114` 메타 누락 false. 실제 effects 인스턴스의 미소속·소멸 smoke 제거를 p1/every_hearth.log에서 확인. |
| U06 | 유지 | `scripts/ui/camera_controller.gd:180` 최초 중심만 최대 마을로 지정, 이후 pending=false. 새 마을 출현 시 카메라를 재중앙화하지 않는 탐색 기본값. |
| U07 | 유지 | `scripts/ui/nature_layout.gd:19` 모든 buildings/sites/formation을 순회한다. 첫 마을만 clearing하지 않는다. |
| U08 | 유지·성능 보류 | `scripts/ui/terrain_renderer.gd:133` formation 변화 때 decal 전면 갱신, 동일 fingerprint는 반환. 정확성 유지 근거와 성능 검증은 별개이며 A30 300프레임 비교 대기. |
| U09 | 유지·성능 보류 | `scripts/ui/nature_renderer.gd:90` formation 변화 때 전 영역 reconcile 및 키별 재사용. A30 실제 비교 대기. |
| U10 | 구현 | `scripts/ui/settlement_overview_renderer.gd:98`, `:107` 마을별 전달 색/이름 사용; getter 범위·alpha 0.28 유지. `scripts/ui/settlement_labels.gd:64` 10px 라벨. p4/labels_pair_isolated.log failures=0; A27 블라인드 대기. |
| U11 | 유지 | 기존 zoom tier는 전체 렌더 계층 정책. `scripts/ui/settlement_overview_renderer.gd:112` 코드 생성 labels와 `scripts/ui/settlement_labels.gd:94` 역줌 스케일로 라벨 분리. 모든 줌 화면 검증 대기. |
| U12 | 구현 | `scripts/ui/panels/hud_topbar.gd:153` 새 문자열/다중 마을 가시성. `test_settlement_labels.gd` 실제 row/icon 숨김과 다른 HUD 보존 확인; p4/labels_pair_isolated.log failures=0. |
| U13 | 유지 | `scripts/ui/panels/hud_topbar.gd:167` 이후 전체 식량/장작 합계는 세계 자원 의미. 소속 판단에 사용하지 않는다. |
| U14 | 구현 | `scripts/ui/panels/hud_topbar.gd:28`, `:268` 명시적 다섯 death key. `test_death_exposure.gd` 추가 deaths_unrelated=999 제외까지 확인; p2/death_exposure_green.log. |
| U15 | 구현 | `scripts/ui/panels/hud_status_panel.gd:280` fission 로그 tick으로 depart/found/cancel 복원·중복 방지, disappeared는 최초 관측 tick임을 메타에 명시. p5/founding_group_green.log failures=0. 실제 사회 장면 A28 대기. |
| U16 | 구현 | `scripts/ui/panels/hud_status_panel.gd:108`, `:111` Selection.number_for(sid). `test_founding_group.gd:68` raw42 대신 UI2 검증. |
| U17 | 구현·촬영 보류 | `scripts/ui/panels/hud_status_panel.gd:43`, `:165`, `:239` 다섯 이유·무제한 feed_log·reset 경계. p2/death_exposure_green.log. 동기 advance 캡처에서 모든 사망 수집 확인은 U29와 함께 대기. |
| U18 | 구현 | `scripts/ui/death_viz_renderer.gd:108` exposure 얼음색, unknown 회색/한 번 경고. `test_death_exposure.gd` 다섯 색/미지 이유 검증. 실제 죽음 화면 A26 대기. |
| U19 | 구현 | `scripts/ui/panels/agent_inspector_panel.gd:158` AgentId→home→마을 이름. FFI join은 p3/society-short.log와 society-natural.log failures=0. 실제 inspector 화면 검증은 별도. |
| U20 | 기존 유지·소속 구현 | 기존 모든 화덕 근접 연출 유지. `scripts/ui/agent_renderer.gd:623` 소속 outline, `scripts/ui/founding_group_renderer.gd:1` 별도 founder 레이어. p4/select.log와 p5/founding_group_green.log failures=0; A27/A28 화면 대기. |
| U21 | 유지 | `scripts/ui/building_renderer.gd:210` 모든 마을 stock을 nearest anchor로 한 번씩 합산. 저장소 소유권 표시가 아닌 자원 anchor 정책. |
| U22 | 유지·한계 명시 | `scripts/ui/building_renderer.gd:471` 공유 anchor의 목표는 최근접 마을 하나. 실제 소유권 FFI 없음. 합계 재고/단일 목표 혼합 가능성을 그대로 공개하며 정확 소속 정보로 해석하지 않는다. |
| U23 | 구현 | `scripts/ui/panels/god_hand_panel.gd:88`, `:92`, `:110` 선택 ID와 대상 이름, 시전 후 선택 변경에도 toast 대상 유지. p4/select.log failures=0; 실제 daughter rain A29 대기. |
| U24 | 유지·실증 보류 | `scripts/rendering/art/god_hand_effects.gd:74` 실제 rain ids 별 중심을 순회. stub 테스트와 P6 God hand 회귀는 통과했으나 실제 선택 daughter 및 원래 불 동시 화면은 A29 대기. |
| U25 | 구현 | `scripts/rendering/art/god_hand_effects.gd:113`, `:148`, `:229` 시전 당시 target 보존 후 해당 저장고 flash. p4/select.log의 target freeze, p6/god_hand.log 회귀 failures=0. |
| U26 | 유지 | `scenes/main.tscn:1` load_steps=21 유지; `scripts/ui/settlement_overview_renderer.gd:112` 동적 labels, `scripts/ui/agent_renderer.gd:60`이 founding 레이어 생성. scene 노드 하나는 다중 마을 순회 renderer다. |
| U27 | Part 7 결과 대기 | `scripts/test/capture_framings.gd` RESET/target/FD/FS/FV 현재 병행 수정 중. 최종 시나리오 manifest·행 성공/실패·선택 결과 확인 전 완료로 닫지 않는다. |
| U28 | Part 7 결과 대기 | 동일 capture script의 actual target/scenario/society/feed/death metadata는 최종 sidecar를 검사해야 한다. 자연·사회·개입 오염 여부 포함. |
| U29 | Part 7~8 결과 대기 | 동기 feed ingest와 A30 동일 부하·동일 FV 300프레임 결과 필요. 구현 유무만으로 사망 누락 없음/프레임 예산 PASS를 주장하지 않는다. |
| U30 | 기존 유지·촬영 보류 | `scripts/test/art/capture_metadata.gd:39` 선택된 마을 주위 deadwood를 찾는 helper는 소속 추론 아님. 최종 target 전달과 실제 이미지 대응은 촬영 결과 대기. |
| U31 | Part 7 결과 대기 | `tools/art_pipeline/capture_art.sh` society whitelist 및 combined manifest 제외의 실제 배포 결과 검사 필요. |
| U32 | Part 8 결과 대기 | `_p7` wood 검사에서 정확 formation matching과 orphan 꺼짐 검증 필요. `_p6/_p4` 원본은 역사 보존. |
| U33 | Part 8 결과 대기 | `_p7` 자원 검사는 U21/U22의 합계/목표 정책 및 알려진 한계를 구분해야 한다. |
| U34 | Part 8 결과 대기 | 화덕별 순회는 유지 가능하지만 orphan 재와 live/soc/god 증거 분리를 실제 measurement에서 확인해야 한다. |
| U35 | Part 8 결과 대기 | 기존 cold 검사의 모든 target/라벨 순회는 단일 마을 문제가 없다. `_p7` 복사본 실행과 실제 산출물은 대기. |
| U36 | 기존 유지·Part 8 보류 | atlas/camp coverage는 모든 formation 좌표 사용. 호출되는 `_p7` 측정의 시나리오 범위와 결과 확인 대기. |
| U37 | Part 8 결과 대기 | production7 출력, live natural A1~A25, 사회 A26~A30 및 별도 A18, 개입 A29의 분리된 측정 결과가 필요하다. |
| U38 | 구현 | `scripts/test/art/test_structure_states.gd:30`, `:42`, `:63` 빈 상태 재/정확 좌표/no glow로 기대 변경. p1/structure_states.log PASS. 이전 코드의 잘못된 최근접 기대는 보존 대상 동작이 아니다. |
| U39 | 유지 | `scripts/ui/building_renderer.gd:357`, `:378` 미소속 장식 fallback/첫 shelter 장식은 소속·실제 재고 수치 아님. 생존 마을 수·정체성을 표현하는 라벨을 대신하지 않는다. |

### 현재 증거로 닫을 수 없는 항목

A30의 모든 항목 판정과 프레임 비교는 아직 없다. U22의 공유 저장고 목표 제한은 수정되었다고 주장하지 않는다. abandoned의 건물 소속은 `Selection.retired_building`의 retired hearth 주변/최근접 기준인 UI 추정이며 simulation 소유권 getter가 아니다. 실제 소멸 사건이 캡처되지 않으면 abandoned 회귀 테스트와 자연 화면 확인을 구분한다. 기존 실패/재시도 로그는 삭제하지 않으며 위 표에는 현재 성공 로그만 명시적으로 연결했다.
