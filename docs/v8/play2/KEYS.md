# V8-PLAY2 입력·렌더 설정 코드 대조표

대상 게임 소스: `f0b759fe29d1f8c16f4ae6a994807036933b50df` (M-ART2.1). `scripts/**`의 `KEY_`, `MOUSE_BUTTON_`, `InputEventKey`, `InputEventMouse`, `is_key_pressed`, `is_action_`와 `project.godot` 입력 맵을 검색하고 해당 핸들러를 읽었다. Graft 인덱스는 GDScript 결과를 누락하여 `rg`로 보완했다. 아래는 **코드 확인**이며 실제 키·화면 확인 결과는 별도의 [PLAY2_REPORT.md](PLAY2_REPORT.md)에 기록한다.

파일 위치는 저장소 루트 기준, 줄 번호는 위 게임 커밋 기준이다.

| 입력 | 코드상 동작·조건 | 파일:줄 |
|---|---|---|
| Space / Pause | 일시정지·재개 | `scripts/world/camera_controller.gd:24`, `:92` |
| 1 / 2 / 3 / 4 | 1× / 2× / 3× / 4× 속도; 메뉴 열림 시 선택 우선 | `scripts/world/camera_controller.gd:19`, `:104`; `scripts/ui/hud/order_bar.gd:478` |
| 휠 위 / 아래 | 확대 / 축소 | `scripts/world/camera_controller.gd:76`, `:79` |
| `-` / 숫자패드 `-` | 축소 | `scripts/world/camera_controller.gd:95` |
| `=` / 숫자패드 `+` | 확대 | `scripts/world/camera_controller.gd:98` |
| 가운데 버튼 드래그 | 카메라 이동 | `scripts/world/camera_controller.gd:82` |
| A / ←, D / →, W / ↑, S / ↓ | 카메라 이동; 살아 있는 사람 선택 시 D 이동 비활성 | `scripts/world/camera_controller.gd:180` |
| H / Home | 캠프 중심 복귀 | `scripts/world/camera_controller.gd:21`, `:101` |
| F1 | 벌목 구역 | `scripts/ui/hud/order_bar.gd:39` |
| F2 | 비축 구역 | `scripts/ui/hud/order_bar.gd:40` |
| F3 | 건설 메뉴 전환 | `scripts/ui/hud/order_bar.gd:41`, `:510` |
| F4 | 구역 제거 | `scripts/ui/hud/order_bar.gd:42` |
| F5 | `user://saves/<seed>-<tick>.wsave` 저장 | `scripts/ui/hud/order_bar.gd:69`, `:429`, `:498` |
| F6 | 명령·청사진·사냥 구역·사슴 표식 취소 | `scripts/ui/hud/order_bar.gd:43`; `scripts/world/colony/order_tool.gd:241` |
| F7 | 채집/사냥 하위 메뉴 열기 | `scripts/ui/hud/order_bar.gd:44`, `:514` |
| F8 | 언덕 파기 | `scripts/ui/hud/order_bar.gd:45` |
| F9 | 이번 실행의 마지막 저장, 없으면 가장 최근 `.wsave` 불러오기 | `scripts/ui/hud/order_bar.gd:70`, `:446`, `:502` |
| F10 | 해체 구역 | `scripts/ui/hud/order_bar.gd:46` |
| F12 | debug 빌드일 때 렌더 JSON 재읽기·토스트 | `scripts/world/camera_controller.gd:107`, `:114` |
| C | 연대기 전환 | `scripts/ui/panels/chronicle_panel.gd:269` |
| Q | 타일 인과 기록 패널 전환 | `scripts/ui/panels/causal_panel.gd:243` |
| P | 작업 우선순위 전환 | `scripts/ui/panels/priority_panel.gd:22`, `:117` |
| T | 기술 지도 전환 | `scripts/ui/panels/tech_panel.gd:25`, `:173` |
| K | 가르치기 전환; 지도 선택 기술→위험 기술→첫 기술 | `scripts/ui/panels/teach_panel.gd:21`, `:137`, `:157` |
| N | 이웃 패널 전환 | `scripts/ui/panels/neighbours_panel.gd:13`, `:95` |
| R | 방 오버레이 전환; 방 상세 자체의 단축키는 아님 | `scripts/world/colony/room_layer.gd:25`, `:85` |
| D | 상세 패널에서 선택한 살아 있는 사람 징집·해제 | `scripts/world/colony/order_tool.gd:333`, `:409` |
| M | 기분 오버레이 전환 | `scripts/world/colony/agent_layer.gd:44`, `:634` |
| L | 한국어·영어 전환 | `scripts/ui/hud/hud_topbar.gd:177` |
| 왼쪽 클릭 | 명령 없을 때 교역 무리→이주민→사람→방(R 켬)→작업장→타일 기록 순으로 대상 판정 | `scripts/world/colony/order_tool.gd:402`, `:556` |
| 왼쪽 드래그 | 구역 지정; 벽·바닥은 선 건설 | `scripts/world/colony/order_tool.gd:437`, `:456` |
| Shift+드래그 | 벽은 속 빈 사각형, 바닥은 채운 사각형 | `scripts/world/colony/order_tool.gd:176`, `:185`, `:411` |
| 오른쪽 클릭 | 명령 모드면 해제; 기본 모드면 징집자 이동·동물/다른 사람 공격 | `scripts/world/colony/order_tool.gd:396`, `:286` |
| Esc | 명령 모드 종료; 건설/채집 메뉴 닫기 | `scripts/world/colony/order_tool.gd:406`; `scripts/ui/hud/order_bar.gd:484`, `:494` |
| Esc | 사람·방·작업장·왜?·이주민·교역·기술·가르치기·우선순위·이웃 패널 닫기 | `scripts/ui/panels/agent_inspector_panel.gd:824`, `room_panel.gd:113`, `workshop_panel.gd:118`, `why_panel.gd:101`, `migrant_panel.gd:106`, `trade_panel.gd:174`, `tech_panel.gd:176`, `teach_panel.gd:160`, `priority_panel.gd:120`, `neighbours_panel.gd:98` |
| 이름 텍스트 복사 | 선택 텍스트가 있을 때 `ui_copy`; 게임은 플랫폼별 키 조합을 지정하지 않음 | `scripts/ui/person_names.gd:67` |

`project.godot:34`의 사용자 입력 맵에는 Esc의 `ui_cancel`만 있다. 게임 조작은 주로 위 직접 키 핸들러에서 처리한다. C/Q 패널은 각각 C/Q로 전환하며 모든 패널에 Esc가 구현됐다고 쓰지 않는다. 기능 키의 macOS 전달 및 한글 입력 상태는 운영체제 조건이므로 위 소스만으로 성공을 판정하지 않는다.

## 메뉴가 열렸을 때

건설 키 배열은 `scripts/ui/hud/order_bar.gd:49`, 종류와 순서는 `scripts/ui/economy_text.gd:56`, `:71`, 한국어 이름은 `localization/ko/buildings.json:3`이다.

| F3 뒤 키 | 항목 |
|---|---|
| 1 | 오두막 |
| 2 | 돌깎이 자리 |
| 3 | 목공대 |
| 4 | 건조대 |
| 5 | 통나무 벽 |
| 6 | 문 |
| 7 | 다진 흙바닥 |
| 8 | 침상 |
| 9 | 돌담 |
| 0 | 토기대 |
| `-` | 갈돌 |

F7 뒤에는 **1 채석, 2 점토, 3 사슴 지정 사냥, 4 사냥 구역**이다(`scripts/ui/hud/order_bar.gd:58`, `:489`). 메뉴가 열려 있으면 이 키들은 속도·확대 대신 항목을 선택한다.

## F12와 기본 화면

- `assets/v8art/render_params.json:4`: 타일 64px. `:5`: 줌 `[0.25, 0.5, 1.0, 2.0]`, 기본 1.0.
- `scripts/world/camera_controller.gd:49`: 시작 시 `camera.default_zoom`; `:107`: `OS.is_debug_build()`일 때만 F12.
- `scripts/world/camera_controller.gd:114`: 성공·실패 토스트를 구분하고 약 2초 표시한다. 성공 한국어 문자열은 “렌더 설정 다시 읽음”. F12 뒤 `step_zoom(0)`으로 현재 줌을 새 단계 중 가장 가까운 값으로 맞춘다. 기본 줌으로 되돌리는 기능은 아니다.
- `scripts/world/art/art_data.gd:77`: JSON이 비었거나 스키마 버전이 1이 아니면 false, 기존 설정 유지. 성공하면 params 버전을 증가시켜 레이어가 재구성하게 한다.
- `scripts/world/art/wall_baker.gd:53`: `walls.top_value` / `top_saturation` / `top_blur_px`로 벽 윗면 조정.
- `scripts/world/colony/ground_layer.gd:349`: `ground.tone.*`로 지면 톤 처리. `:280`: `water.drift_px`, `water.drift_hz`; `:408`: 물/지면 명도·색상 차이 하한. `:515`: `season.snow_alpha`.
- `scripts/world/colony/agent_layer.gd:278`: `people.night_halo_alpha`를 0..1로 제한. `:310`: `lod.people_below_zoom` 미만에서 마커; `:468`: `lod.glyph_zoom`에서 활동 기호. `scripts/world/colony/animal_layer.gd:136`도 같은 줌 임계값 사용.

## 첫 플레이의 시스템 근거

- 선/사각형 건설: `scripts/world/colony/order_tool.gd:176`; 벽 사각형은 문 틈을 자동으로 만들지 않는다.
- R+방 클릭 및 역할 설정: `scripts/world/colony/order_tool.gd:580`, `scripts/ui/panels/room_panel.gd:1`, `:17`.
- 가르치기의 기술·스승·제자 선택과 주문: `scripts/ui/panels/teach_panel.gd:1`.
- 징집자는 선택한 징집자 우선, 없으면 전체 징집자에게 우클릭 명령: `scripts/world/colony/order_tool.gd:270`, `:286`.
- 이주민 기다림 시 자동 대화창, 1배속일 때만 자동 일시정지: `scripts/ui/panels/migrant_panel.gd:1`.
- 교역 무리·배너 클릭, 재고/물건 수량과 거래·선물: `scripts/ui/panels/trade_panel.gd:1`. 거래 조건은 주는 값×0.8 ≥ 받는 값×1.2이며 현재 재고·방문 상태에 따라 거절될 수 있다.
