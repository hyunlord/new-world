# ENV1 현재 게임 화면 기준선

**캡처 검증 완료.** 동결된 `origin/v8/wip`의 실제 Godot 렌더러로 동일한 네이티브 저장 파일 세 개를 읽어 6장을 촬영했다. 게임 코드나 시뮬레이션 객체를 바꾸지 않았다. 화면 비교의 기준은 [baseline_manifest.json](reference/baseline_manifest.json)이며, 각 이미지와 HUD의 SHA-256, 저장 파일·브리지·하네스 해시, 인물 ID/좌표/행동, 방·동물의 실제 상태를 포함한다.

| 조건 | 정확한 시각 | 1×, 64px 타일 | ½×, 32px 타일 | 전체 인구 / 맵에 중심점이 보이는 인물 | 저장고 방 / 늑대 |
|---|---|---|---|---|---|
| 봄 6일 낮 | T7440, 12:00 | [현재 화면](reference/current_day6_noon_1x.png) · [HUD](reference/current_day6_noon_1x_hud.png) | [현재 화면](reference/current_day6_noon_half.png) · [HUD](reference/current_day6_noon_half_hud.png) | 10 / 8 | 0 / 0 |
| 봄 6일 밤 | T6840, 02:00 | [현재 화면](reference/current_day6_night_1x.png) · [HUD](reference/current_day6_night_1x_hud.png) | [현재 화면](reference/current_day6_night_half.png) · [HUD](reference/current_day6_night_half_hud.png) | 10 / 10, 모두 수면 | 0 / 0 |
| 첫 겨울 낮 | T26160, 절대 19일·겨울 1일, 12:00 | [현재 화면](reference/current_winter_noon_1x.png) · [HUD](reference/current_winter_noon_1x_hud.png) | [현재 화면](reference/current_winter_noon_half.png) · [HUD](reference/current_winter_noon_half_hud.png) | 10 / 8 | 0 / 0 |

## 출처와 화면 정렬

- 시뮬레이션/게임 소스: `3a3b236c125c0c25e9b74261be88536473fc01e9`, 작업 트리 `/Users/rexxa/work/artenv1-data/probe-worktree`. 캡처 후 추적 파일 변경 없음.
- 아트 기준 커밋: `159ef0f9a1cfb7ee9e6005e85d5388d677e07fa8`. 저장 파일의 네이티브 ASCII·인물 multiset 일치는 [SCENE_SOURCE.md](SCENE_SOURCE.md)와 [source_manifest.json](scenes/source_manifest.json)에 연결된다.
- Godot `4.6.stable.official.89cea1439`, 실제 창, Compatibility/OpenGL 4.1 Metal, Apple M4 Max. 헤드리스 이미지가 아니다.
- 뷰포트는 **1600×900**. HUD는 각 실제 화면의 `(0,0,1600,112)` 영역을 그대로 잘랐다. 글자·수치·아이콘을 재작성하거나 크기를 바꾸지 않았다. HUD 아래 맵 영역은 `(0,112,1600,788)`이다.
- 캠프 타일 `(91,101)`의 중심은 **화면 `(800,506)`**. 게임 타일 16px에 카메라 zoom4/2를 적용해 64/32px가 된다. 좌표식은 `screen = (800,506) + (tile - (91,101)) * tile_pixels`이다.
- 인물 좌표는 이 타일식만으로 만들지 않았다. 현재 `AgentLayer.drawn_position`에 반 타일을 더한 실제 렌더러 중심점을 사용하여 수면 중 겹침 분산까지 반영했다. `agent_id`와 `entity_bits`를 저장하고 `get_activity_snapshot.ids`로 활동을 결합했다.
- `in_map_view`는 인물 중심점이 HUD 아래 맵 사각형 안에 있는지 나타낸다. 픽셀 단위 가림이나 전신 노출 판정은 아니다. 봄 낮의 ID6·7, 겨울 낮의 ID4·8은 1×에서 화면 밖이며 ½×에서는 `(384,58)`·`(352,58)` 근처, 즉 HUD 뒤에 있다.

## 재현

이미 빌드·임포트된 동결 체크아웃과 `scenes/rich_{day6_noon,day6_night,winter_noon}.save`를 사용한다. 하네스는 작업 트리 HEAD를 확인하고 180초 제한과 `caffeinate`로 실행한다.

```bash
bash art/compositor/baseline_capture.sh /Users/rexxa/work/artenv1-data/baseline
```

[baseline_capture.gd](../../../art/compositor/baseline_capture.gd)는 동결 체크아웃의 `res://tools/v8/capture.gd`를 상속하고 실제 `res://scenes/main.tscn`을 연다. `WorldSim` 처리를 첫 대기 프레임 전에 끄고 `SimReads.mutate_raw("load_world", ...)`로 저장 파일을 읽는다. `advance_ticks`, 건설 명령, 객체 추가는 호출하지 않는다. 렌더러가 갱신된 후 각 촬영 시각과 캠프 좌표를 검증한다.

원본·상세 덤프·로그는 `/Users/rexxa/work/artenv1-data/baseline/`에 남긴다. 최종 화면 6장은 저장소 사본과 바이트가 같고, HUD 6장은 원본의 정확한 상단 112px이다. 가장 큰 PNG는 218,326바이트로 1.5MB 제한 이하다. 이전 캠프 중심 y450 시도의 파일은 `superseded_camp_y450/`에 보존했다. 로그에는 `ENV1_BASELINE_CAPTURE_DONE 6`이 있으며 `SCRIPT ERROR`, `Parse Error`, `ERROR:`가 없다.

## 동일 조건 비교의 한계

세 네이티브 저장 파일에는 침실 방 하나(침상 4개), 적치장 구역 두 개, 미완성 통나무벽 현장 21개가 있다. **완성된 저장고 역할의 방은 없지만, 미완성 벽 둘레 안에 지정된 적치장이 실제로 있다.** `get_colony_overlay.zone_*`의 zone2는 타일 x101–106, y95–100의 36칸이다. x100–107, y94–101의 계획된 8×8 둘레에는 완성 통나무벽 6칸, 완성 문 1칸 `(104,101)`, 진행 0/40인 통나무벽 현장 21칸이 있다. `get_space.wall_*`와 `get_colony_overlay.site_*`를 대조했으며 세 조건에서 같다. `get_rooms`의 Storehouse 역할 수 0과 모순되지 않는다.

| 실제 적치장 평가 대상 | 1× 화면 경계 `[왼쪽,위,오른쪽,아래]` | ½× 화면 경계 | 화면에서 지정 가능한 점 |
|---|---|---|---|
| zone2 계획 벽 둘레 | `[1344,26,1856,538]`; 맵에 보이는 교집합 `[1344,112,1600,538]` | `[1072,266,1328,522]` | 1× `(1504,282)`, ½× `(1200,394)` |
| zone2 적치장 내부 | `[1408,90,1792,474]`; 맵에 보이는 교집합 `[1408,112,1600,474]` | `[1104,298,1296,490]` | 위와 같음 |
| 별도의 야외 zone1, x94–99·y101–106 | `[960,474,1344,858]` | `[880,490,1072,682]` | zone2와 구분 |

따라서 위치 찾기 과제는 **‘미완성 벽 둘레의 적치장’**을 기준선의 정답 대상으로 미리 정하면 평가할 수 있다. 이를 완성된 `RoomRole::Storehouse`로 바꾸어 부르지 않는다. 후속 합성 화면의 완성 저장고와는 건설 상태·위치·노출 면적이 달라 순수한 동일 상태 비교는 아니다. 1×에서 zone2의 오른쪽 부분은 화면 밖이므로 채점에는 실제 보이는 교집합을 쓴다. 이 정보는 manifest의 각 캡처 `designated_stockpiles`와 `storage_readability_target`에 기록했다.

사슴은 전체 맵에 9마리이고 늑대는 0마리다. ½× 밤 화면에 사슴 한 마리만 들어온다. 늑대 존재 과제의 기준선 정답은 ‘없음’이며, 늑대가 배치된 후속 장면과의 상태 차이는 유지한다.

세 시각의 모든 인물은 작업 `job=0`, 운반 `count=0`, `kind=-1`이다. 낮에는 실제 routine 활동(불 돌보기·제작·보수·화롯가·탐색)이 있지만 현재 렌더러는 서 있는 프레임으로 보인다. 따라서 후속 장면에 선언된 운반·작업 자세나 의상 대체가 포함되면 그 차이는 순수한 동일 상태 아트 교체 결과와 구분해야 한다.

겨울 화면은 실제 첫 겨울 낮이며 보통 겨울이다. 현재 렌더러의 녹색·갈색 지형을 그대로 보존했으므로 눈이나 한파 효과의 전후 증거는 아니다. 실캡처 6장의 낮/밤 조명과 HUD는 그대로 비교할 수 있다.

기존 `reference/m5_01_wolves_banner.png`, `m5_05_cold_snap_banner.png`, `m5_07_hunted_deer.png`는 다른 사건·시각·카메라의 M5 참고 화면으로 유지한다. 기존 93px HUD 참고 잘라내기는 이 기준선의 112px HUD를 대체하지 않는다. 후속 합성 장면이 저장고·늑대·완공 건물·특정 행동을 별도로 배치했다면 해당 선언과 평가 범위를 함께 보여야 한다. 이 캡처는 성능, 게임 통합, 전체 상태 커버리지의 통과 증거가 아니다.
