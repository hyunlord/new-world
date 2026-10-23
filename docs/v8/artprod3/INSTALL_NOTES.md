# M-ART2 설치 노트 — ART-PROD3 채택

**route I 채택: 전체34/36·성인16/18·어린이18/18.** 2026-10-23 planner/owner 결정 D4가 성인≥17/18 채택 기준을 잔여 성인2룩에 한해 면제했다. 원래 S1 기준의 FAIL과 결손은 보존하며 아래 fallback을 사용한다. PROD2 R1·P1·D1은 유지한다. 캐릭터 표준은 `art/character/TEMPLATE_V3.md`, 자산 원천은 `art/character/route_i/`와 그 `MANIFEST.json`이다. 역사 route·진단은 `art/character/archive/`로 이동했다. 이 문서는 renderer 인계 규격이며 게임 코드 설치·merge는 수행하지 않는다. 최종 근거는 [PROD3_REPORT.md](PROD3_REPORT.md), 상세 인계와 오너 결정은 [HANDOVER.md](HANDOVER.md)를 따른다.

PROD3 가시 bbox는 alpha≥16이며 캔버스 적합성·발 기준·bands에 적용한다. 배치 후 캔버스 밖 alpha<16만 버리고 수량·최대 alpha를 프레임별 기록한다. 내부 RGBA는 유지하며, 이 규칙이 픽셀 지우기·덧칠을 허가하지 않는다. 새 생성 예산과 실행은 0회다.

기존26룩 재실행은26룩 모두 통과했으나 PNG260개 중202개(26룩)의 바이트가 달랐고, 캔버스 밖 폐기 픽셀은0개였다. 배포된 기존26룩의546파일은 그대로 보존했으며 sidecar는 PROD2 기하 규칙의 역사 기록이다. 보존 PNG260개의 새 alpha≥16 검사는 모두 통과했다. 재실행 산출물의 바이트 동일성을 주장하지 않는다. 비교 근거는 [RESLICE_RESULTS.json](RESLICE_RESULTS.json)에 있다.

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

## 잔여 결손2룩 fallback

아래2룩은 보존 best 원본 old1 재절단 후에도 MISSING이다. `adult_M_fur_cloak_long_tied`는 E/work_a panel judge 실패다. `adult_M_hide_wrap_long_tied`는 sleep은 통과했으나 E/idle 가시 높이가60px로 상한58px를 넘는다(alpha≥128 높이는58px). 새 생성 없이 실패를 보존하며 fallback을 별도 수락 룩으로 세지 않는다.

| MISSING look | 현재 수락 fallback | 속성 차이 |
|---|---|---|
| `adult_M_fur_cloak_long_tied` | `adult_M_fur_cloak_bun` | top 유지; hair 대체; age/sex 유지 |
| `adult_M_hide_wrap_long_tied` | `adult_M_hide_wrap_bun` | top 유지; hair 대체; age/sex 유지 |
