# M-ART2 인수인계 — ART-PROD3

**판정: S1 FAIL, route I 미채택.** 결손10룩 중8룩이 통과하여 전체34/36·성인16/18·어린이18/18이다. 전체≥32 기준은 충족하지만 성인≥17 기준이 미달하며, 어린이만 부족한 경우의 성인 단독 채택 예외도 적용되지 않는다. [PROD3_REPORT.md](PROD3_REPORT.md)를 최종 판정으로 따른다. 새 생성은0회이며 게임 코드 변경·설치·merge·archive 이동은 수행하지 않는다.

## 자산과 좌표

- 후보 규격: [TEMPLATE_V3.md](../../../art/character/TEMPLATE_V3.md). 후보 자산과 수락 이력: [route_i/MANIFEST.json](../../../art/character/route_i/MANIFEST.json). 승인된 게임 표준으로 선언하지 않는다.
- PNG 경로: `art/character/route_i/<age>_<sex>_<top>_<hair>/<S|E>/<frame>.png`; 같은 이름의 JSON sidecar와 룩별 `MANIFEST.json`에 원본 해시·검사·배율이 있다. 다른 route와 진단 이미지는 게임 자산으로 혼합하지 않는다.
- upright: 80×112px, 발 anchor `(40,104)`; sleep: 112×80px, anchor `(56,72)`. 64px 타일에서 발을 `(tile_left+32,tile_top+43)`에 놓고 해당 anchor를 빼서 PNG 좌상단을 계산한다. 투영 좌표는 M-ART2 renderer에서 변환한다.
- 1px=1world-px, nearest filtering; 2×는 정수 확대다. 캔버스를 64×64에 맞춰 늘이거나 찌그러뜨리지 않는다.
- contact shadow는 발 위치에 별도 28×8px 타원으로 그린다. PNG에 굽지 않는다. 지면·가림·밀집 상태는 게임 화면에서 별도 검증한다.

## 크기·방향·프레임

성인 idle 인체 목표 높이는 56px, 어린이는 40px다. S idle에서 얻은 공통 배율을 양 방향의 idle/walk_a/work_a/carry에 적용한다. 도끼·통나무를 인체 높이에 맞춰 줄이지 않는다. sleep은 alpha≥128 원본 bbox 긴 축으로 독립 배율을 정한다. 성인 길이 52–60px·높이≤36px, 어린이 37–43px·높이≤28px이며, 실행 시 이미 정규화된 PNG를 다시 나이별 축소하지 않는다. 실제 `scale`, `standing_scale`, `sleep_scale` 및 `sleep_scales.S/E`는 sidecar/manifest를 따른다.

PROD3의 가시 bbox는 **alpha≥16**이다. 캔버스 적합성·발 기준·bands는 이 기준을 쓴다. 배치 후 캔버스 밖의 alpha<16만 기하학적 crop으로 버리고 프레임별 수량·최대 alpha를 기록한다. 캔버스 내부 RGBA는 유지한다. 이 규칙은 내부의 낮은 alpha 픽셀 삭제나 추가 보정의 허가가 아니다.

기존26룩은 새 임계값 재실행에서26룩 모두 통과했지만, PNG260개 중202개(26룩)에 바이트 차이가 발생했다. 재실행의 캔버스 밖 폐기 픽셀은0개다. 배포된 기존26룩의546파일은 바꾸지 않았으며 sidecar는 PROD2 기하 규칙의 역사 기록이다. 보존 PNG260개를 새 alpha≥16 규칙으로 별도 검사한 결과는 모두 통과했다. 재실행 산출물이 바이트 동일하다고 주장하지 않으며 비교 근거는 [RESLICE_RESULTS.json](RESLICE_RESULTS.json)을 따른다.

| 표시 상태 | 사용하는 프레임 |
|---|---|
| idle | `idle` |
| walk | `idle` / `walk_a` 교대, 표시 시 전체 sprite 1px bob |
| work | `work_a` / `idle` 교대 |
| carry | `carry`, 표시 시 전체 sprite 1px bob |
| sleep | `sleep` 정지 |

랩은 8fps다. 게임의 보행 주기·시뮬레이션 속도는 별도 결정한다. 실제 제작 방향은 S/E이며 W는 E 전체의 수평 반전이다. 발 anchor를 같은 월드 위치에 유지한다. **N은 S 대체**이며 실제 뒷모습이 아니다. sleep 원본은 머리가 왼쪽인 canonical이고 W 반전 시 오른쪽이 된다. 부위별 반전·독립 팔다리 운동·재색칠은 이 계약에 없다.

## 속성 및 결손

룩 ID는 `<age>_<sex>_<top>_<hair>`다. entity life-stage를 child/adult, 성별을 M/F, apparel을 loincloth/hide_wrap/fur_cloak, identity hair를 short/long_tied/bun으로 매핑한다. 나이 임계값과 무의복·신규 apparel 정책은 M-ART2 데이터에서 명시한다. 어린이 성별은 선택 속성이지만 40px 가독성 채점 대상은 아니다. 전용 노인·머리색 변형은 없다. 방향·활동 변화로 identity를 바꾸지 않는다.

MISSING은 [INSTALL_NOTES.md](INSTALL_NOTES.md)의 fallback을 사용하며 수락 수에 포함하지 않는다. 같은 age/sex에서 top 일치 우선, 다음 hair 일치, 동률 ID 사전순이다. 후보가 없으면 같은 age에서 동일 우선순위를 적용하고 성별 대체를 명시한다. child를 adult 크기로 대체하지 않으며 해당 age 자체가 없으면 MISSING을 유지한다.

## 리뷰 증거와 한계

- PROD2 판정·점수·측정: [PROD2_REPORT.md](../artprod2/PROD2_REPORT.md), [GATES.json](../artprod2/GATES.json), [GODOT_RESULTS.json](../artprod2/GODOT_RESULTS.json), [D1_RESULTS.json](../artprod2/D1_RESULTS.json).
- 원시 가독성 응답: `/Users/rexxa/work/artprod2-data/reviews/round0/{a,b}.{json,md}`; 채점 `reviews/round0/scored/scores.json`; 블라인드 패킷 `blind/round0`; 실제 재생 랩 `lab/round0`(모두 같은 외부 데이터 루트).
- 현재 시각 자료: `art/character/route_i_lab/prod2/{lab_p1_1x.png,lab_p1_2x.png,crowd_1x.png}` 및24 GIF. 미채택이므로 기존 경로를 유지한다.
- 전체 룩 보드: [1×](../../../art/character/boards/route_i_sheet_1x.png), [2×](../../../art/character/boards/route_i_sheet_2x.png).

PROD2 R1은 두 리뷰어·양 배율에서 활동 12/12, item+top+hair 31–32/36, 성인 sex+age 20–21/22로 통과했다. P1은 실제 Godot 창 200명·60초에서 524.831FPS, D1은 재현성 통과다. 이 결과는 게임 시뮬레이션 성능·모든 룩의 개별 가독성·자연스러움·최종 정착지 화면 승인과 다르다. planner의 PROD3 결정 D1에 따라 원본과 리뷰 프레임이 유지되는 임계값 수정에 새 리뷰를 요구하지 않는다.

## 남은 오너 결정 3건

1. 실제 N 방향을 별도 제작할지.
2. 노인 및 머리색 tint의 범위와 제작·검증 방식.
3. 추가 상의 종류와 후속 생산 범위.
