# M-ART2 설치 노트 — R1 통과·S1 실패, 채택 아님

이 문서는 renderer 인계 규격이며 게임 코드 설치·merge를 수행하지 않는다. R1은 통과했지만 전체26/36·성인13/18로 S1이 실패하여 채택하지 않는다. P1도524.831FPS로 통과했으나 S1 실패를 대신하지 않는다. 최종 증거는 PROD2_REPORT.md를 따른다.

## 좌표와 표시

- upright 캔버스 80×112px, 발 기준 `(40,104)`; sleep 캔버스 112×80px, 기준 `(56,72)`.
- 원본 크기 그대로 1px=1world-px, nearest filtering. 2×는 정수 확대다. 캐릭터 캔버스를 64×64 tile에 늘이거나 찌그러뜨리지 않는다.
- 64px 타일에서 화면상 발 위치를 `(tile_left+32, tile_top+43)`으로 잡는다(타일 아래쪽 1/3의 시작). upright 좌상단은 발 위치에서 `(40,104)`를 빼고, sleep은 `(56,72)`를 뺀다. 타일 중심/투영 규약이 달라지는 M-ART2에서는 동일 발 위치를 renderer 월드 좌표로 변환한다.
- lab contact shadow는 발 위치 중심의 28×8px 타원이다. PNG에 그림자를 굽지 않고 renderer가 별도로 그린다. 최종 게임의 겹침/지면 가독성은 M-ART2에서 확인한다.
- S와 E가 실제 제작 방향이다. W는 E를 수평 반전하며 발 anchor도 같이 보존한다. N은 이번 패스에서 S 대체다. 실제 N 제작 여부는 오너 결정이다. 수면 원본은 머리-왼쪽 canonical이며 W 반전 시 머리가 오른쪽이 된다.

## 크기와 프레임

- adult 서 있는 인체 높이56px, child40px. idle S에서 구한 공통 배율을 idle/walk_a/work_a/carry에 적용한다. 도끼·통나무 전체를 인체 높이로 강제하지 않는다.
- sleep은 원본 alpha>=128 bbox 긴 축으로 독립 배율을 계산한다. adult 길이52–60px/높이≤36, child37–43px/높이≤28. 각 PNG sidecar의 `scale`은 실제 배율이며 `standing_scale`, `sleep_scale`과 MANIFEST의 `sleep_scales.S/E`를 참조한다. runtime에서는 이미 정규화된 PNG를 다시 나이별 축소하지 않는다.
- idle은 idle; walk는 idle/walk_a 교대; work는 work_a/idle 교대; carry는 carry; sleep은 sleep. 랩은 8fps이며 walk의 전체 sprite 이동은 합성 연출이다. 시뮬레이션 속도·보행 주기는 게임 통합 시 별도 결정한다.

## look과 entity 속성

`<age>_<sex>_<top>_<hair>`가 look ID다. MANIFEST의 필드를 읽고 경로 문자열을 임의 번역하지 않는다.

- sex: entity의 성별 M/F. 어린이 sex는 선택 속성이지만40px 가독성 채점 대상은 아니다.
- age: entity life-stage를 child/adult로 매핑한다. 이 패스는 새로운 나이 임계값을 정의하지 않는다. 노인 전용 그림은 없다.
- top: 착용 apparel에서 loincloth/hide_wrap/fur_cloak로 매핑. 의복이 없는 상태·신규 apparel의 규칙은 M-ART2 데이터에서 명시한다.
- hair: identity의 short/long_tied/bun을 사용한다. 작업·방향에 따라 바꾸지 않는다. 머리색 변형은 아직 제작하지 않았다.
- 결손 fallback은 같은 age와 sex 안에서 top 일치 우선, 다음 hair 일치, 동률은 look ID 사전순이다. 후보가 없으면 같은 age에서 동일 우선순위를 적용하고 성별 대체를 명시한다. child를 adult 크기로 대체하지 않는다. 해당 age 자체가 없으면 MISSING을 유지한다.

## 현재 결손10룩의 fallback

최종 생성27회·라운드0 R1 통과로 고정된 수락26룩 기준이다. 리뷰 교정은 발생하지 않았다. 전체26/36·성인13/18로 S1 실패이므로 이 표는 인계 규칙이며 설치 승인이나 자동 채택이 아니다. 후속 별도 패스에서 수락 세트가 바뀌면 같은 우선순위로 다시 계산한다. 원래 MISSING 상태와 실패 기록은 보존하고 fallback을 별도 수락 룩으로 세지 않는다.

| MISSING look | 현재 수락 fallback | 속성 차이 |
|---|---|---|
| `adult_F_hide_wrap_bun` | `adult_F_hide_wrap_long_tied` | top 유지; hair 대체; age/sex 유지 |
| `adult_M_fur_cloak_long_tied` | `adult_M_fur_cloak_bun` | top 유지; hair 대체; age/sex 유지 |
| `adult_M_fur_cloak_short` | `adult_M_fur_cloak_bun` | top 유지; hair 대체; age/sex 유지 |
| `adult_M_hide_wrap_long_tied` | `adult_M_hide_wrap_bun` | top 유지; hair 대체; age/sex 유지 |
| `adult_M_loincloth_bun` | `adult_M_loincloth_long_tied` | top 유지; hair 대체; age/sex 유지 |
| `child_F_fur_cloak_bun` | `child_F_fur_cloak_long_tied` | top 유지; hair 대체; age/sex 유지 |
| `child_F_fur_cloak_short` | `child_F_fur_cloak_long_tied` | top 유지; hair 대체; age/sex 유지 |
| `child_F_loincloth_bun` | `child_F_hide_wrap_bun` | top 대체; hair 유지; age/sex 유지 |
| `child_F_loincloth_long_tied` | `child_F_fur_cloak_long_tied` | top 대체; hair 유지; age/sex 유지 |
| `child_F_loincloth_short` | `child_F_hide_wrap_short` | top 대체; hair 유지; age/sex 유지 |
