# Art Pass 8 — A18 군중 원인 진단

## 범위와 결론

Art Pass 7 자연 장면 117개 중 기존 A18 FAIL 23개를 전부 조사했다. 기존 판정은 수정하지 않았다. 가려짐 ≥ 0.5인 주민 관측 97건 중 **86건(88.7%)이 기존 CampSeats 기준 보행**, 11건이 비보행이다. 21개 실패 프레임에서 보행자가 가려진 주민의 과반수이고, 1개는 동률, 1개는 비보행 우세다. 따라서 허용된 수정 (a), 겹친 보행자의 진행 방향 수직 렌더 오프셋을 우선 적용하는 근거가 있다.

긴 겨울 후 아침 3장에서는 42건 중 36건이 보행이다. 그러나 전체 표에서는 낮의 운반·보행과 저녁 귀가 장면도 다수다. 아침에만 적용하는 조건으로 문제를 제한하면 안 된다.

## 자료와 재현 방법

- 원 판정: `docs/art/production7/measurements.json` → code A18 → detail.frames. actor.fraction ≥ 0.5인 id만 분석했다. 한 프레임 허용 최대 1명 그대로다.
- 원본 위치는 `docs/art/production7/shots/manifest_art.json` 및 `manifest_art_long.json`의 raw_entities_file을 직접 따랐다. 전 23개 raw_entities_sha256 일치를 확인했다. 사용 경로는 `/Users/rexxa/work/art-production7-evidence/final_capture/shots/{art,art_long}/`이다. 비슷한 이름의 `shots/`, `recapture_natural/`은 최종 자료와 다르므로 사용하지 않았다.
- strip/evening 19개의 원본 sidecar도 모두 읽었다. 동반 JSON에 파일 경로·SHA-256·해당 주민 id·가려짐 비율·렌더 위치를 남겼다.
- 보행 정의는 `CampSeats._walking`: moving=true **또는** activity가 walk/run/carry/carry_wood. 캡처에서 moving=false여도 멈춘 보행 애니메이션을 비보행으로 재분류하지 않았다.
- 아래 activity와 좌표는 **가려짐 ≥ 0.5인 주민만** 집계한다. 좌표는 simulation tile_x/tile_y이며 이동을 제안하는 좌표가 아니다. 시간은 time_snapshot에서 구한 표시 날짜·시각이다.

## 실패 23프레임 전수 표

| 프레임 | 표시 날짜·시각 | 가려짐/보임 | 보행/비보행 | 활동별 수 | 위치 (tile x,y × 수) |
|---|---|---:|---:|---|---|
| a_d1_0830_F2 | d1 08:30 | 4/25 | 2/2 | build 2, walk 2 | (21,18)×2; (23,24)×1; (23,25)×1 |
| a_strip_02 | d2 12:06 | 2/10 | 2/0 | carry 1, walk 1 | (32,26)×1; (34,26)×1 |
| a_strip_35 | d2 13:46 | 3/8 | 2/1 | carry 1, idle 1, walk 1 | (20,27)×1; (21,32)×1; (26,28)×1 |
| a_strip_36 | d2 13:49 | 6/10 | 6/0 | carry 3, walk 3 | (20,27)×2; (21,26)×2; (22,31)×1; (26,28)×1 |
| a_strip_37 | d2 13:52 | 4/10 | 4/0 | carry 2, walk 2 | (20,27)×2; (21,26)×1; (26,28)×1 |
| a_strip_38 | d2 13:55 | 4/10 | 4/0 | carry 2, walk 2 | (20,27)×1; (21,26)×1; (21,27)×1; (25,28)×1 |
| a_strip_39 | d2 13:58 | 2/11 | 2/0 | walk 1, carry 1 | (20,27)×1; (21,26)×1 |
| a_evening_02 | d2 19:36 | 2/20 | 2/0 | walk 2 | (31,26)×1; (34,26)×1 |
| a_evening_03 | d2 19:41 | 2/20 | 2/0 | walk 2 | (32,26)×1; (35,26)×1 |
| a_evening_04 | d2 19:44 | 2/20 | 2/0 | walk 2 | (35,26)×2 |
| a_evening_15 | d2 20:17 | 2/21 | 2/0 | walk 2 | (20,18)×1; (22,26)×1 |
| a_evening_17 | d2 20:22 | 2/21 | 2/0 | walk 2 | (21,20)×1; (22,25)×1 |
| a_evening_18 | d2 20:26 | 3/21 | 3/0 | walk 3 | (21,19)×1; (22,25)×2 |
| a_evening_19 | d2 20:29 | 3/21 | 3/0 | walk 3 | (21,19)×1; (22,25)×2 |
| a_evening_20 | d2 20:32 | 2/21 | 2/0 | walk 2 | (22,25)×2 |
| a_evening_21 | d2 20:35 | 2/21 | 2/0 | walk 2 | (22,24)×2 |
| a_evening_22 | d2 20:38 | 2/21 | 2/0 | walk 2 | (22,24)×2 |
| a_evening_25 | d2 20:47 | 3/21 | 3/0 | walk 3 | (20,19)×1; (22,23)×1; (22,24)×1 |
| a_evening_26 | d2 20:50 | 3/21 | 1/2 | walk 1, idle 2 | (21,20)×1; (22,24)×1; (23,22)×1 |
| a_evening_29 | d2 20:59 | 2/20 | 2/0 | walk 2 | (22,23)×1; (24,21)×1 |
| l_d27_0715_F2 | d27 07:15 | 7/24 | 6/1 | idle 1, walk 6 | (18,18)×4; (19,20)×3 |
| l_d29_0630_F2 | d29 06:30 | 21/29 | 19/2 | walk 19, talk 2 | (18,19)×4; (19,20)×1; (19,22)×4; (21,15)×7; (21,19)×4; (23,19)×1 |
| l_d29_0715_F2 | d29 07:15 | 14/28 | 11/3 | idle 3, walk 11 | (18,19)×1; (18,20)×4; (19,18)×3; (19,20)×1; (26,20)×5 |

활동 합계: build 2, walk 76, carry 10, idle 7, talk 2. moving=true는 46건이며 보행 여부는 위의 합성 조건으로 판정했다.

## 코드 원인과 허용 범위 내 수정 권고

조사 시점 코드 위치(이후 같은 파일이 변경되면 함수명으로 재확인):

1. `scripts/ui/camp_seats.gd:85` `_eligible`은 보행자를 좌석 후보에서 제외하고, `:101` `spread_offsets`도 보행자를 건너뛴다. `:128` `declump`의 `:146` 조건 역시 `not _walking(row)`만 이동시킨다. 앞에 그려진 보행자는 다른 주민을 가리지만 자신은 어떤 겹침 해소도 받지 않는 구조다.
2. `scripts/ui/agent_renderer.gd:495` `_apply_drawn_positions`가 좌석·spread·declump를 조합하고 `:533`에서 CampSeats.declump를 부른다. `:536` 이후 target을 부드럽게 따라가고 `:543`에서 정수 world pixel로 반올림한다. 오프셋은 simulation row/ipos가 아닌 이 렌더 경로에만 적용한다.
3. 기존 `CampSeats.declump` 안에서 보행 분기를 확장한다. AgentId로 좌우 우선순위를 고정하고, 실제 보행 방향에 수직인 후보만 평가하며, 겹친 경우에만 **최대 3 screen px**까지 허용한다. 카메라 zoom을 반영해 screen px와 world px를 구분해야 한다. 현재 최종 정수 world 반올림은 zoom 4에서 1 world px = 4 screen px라 상한을 깨거나 0으로 만들 수 있으므로, 최종 화면 변환 후의 실제 변위도 검증한다. 이 제약을 피하려고 3을 world/native px로 바꾸지 않는다.
4. 기존 `:188` `_declump_allowed`의 건물·물 타일 검사 재사용. 후보가 없으면 0 이동을 유지한다. 진행 방향이 불명확한 paused walk에서는 추측성 임의 축을 쓰기 전에 기존 facing/이동 증거를 확인한다. row 순서 변화에도 동일 결과, 반대 진행, diagonal, zoom별 상한, 물/건물, 비겹침 0, simulation 불변을 검증한다.
5. `scripts/test/art/test_declump.gd:29`와 `:33`의 기존 “moving villagers never shift / paused walking animation never shifts” 기대는 새 허용 범위와 충돌한다. 기존 검사를 삭제해 통과시키지 말고 새 제한된 보행 계약으로 교체하며 기존 비보행·장애물·determinism 검사를 유지한다. 새 행동은 brief의 `test_meal_ring.gd`에도 검증할 수 있다.
6. 비보행 11건은 후속 잔여 문제다. 별도 좌석 체계를 만들지 않는다. 필요하면 기존 `_rebuild`만 확장한다. 실제 현재 `_rebuild`(`:52`)는 고정 타원 반경 104×72 world px 후보이며 brief의 설명(2타일 + 12명당 1타일)과 다르다. 현 구현을 읽고 인원 의존 후보/재빌드 조건이 필요하면 같은 체계 안에서 추가하되, 본 표의 지배 원인은 보행자임을 바꾸지 않는다.

## 탐색과 연결 관계

graft ask를 먼저 실행했다. GDScript 함수는 현재 graph에 없어서 graft grep declump, skeleton 및 callers _rebuild가 연결을 제공하지 못했다. 이후 원문에서 실제 연결을 확인했다: agent_renderer render 호출(:178) → _apply_drawn_positions(:495) → CampSeats.update/spread_offsets/declump(:511–533), 그리고 test_declump의 직접 호출. Rust 시뮬레이션 변경은 권고하지 않는다.

## 한계

이는 저장된 가려짐 측정과 검증된 sidecar의 원인 분석이며 새 화면의 A18 PASS나 사람의 셈 가능성(A18b)을 주장하지 않는다. 97건은 주민 고유 수가 아니라 프레임별 관측 건수다. 원 측정의 mask 판정은 재작성하거나 완화하지 않았다. 이번 진단에서는 GPU 캡처, 픽셀 재측정, 실제 보행 방향 추적, 테스트 실행을 하지 않았다. A18b 두 블라인드 관찰자와 12장 판정은 별도 수행해야 한다. 최대 3 screen px 제한으로 다중 완전 중첩이 모두 해결된다는 보장은 없으며 새 자연 캡처로 원 임계값을 재검증해야 한다.
