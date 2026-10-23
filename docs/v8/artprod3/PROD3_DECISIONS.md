# ART-PROD3 결정 기록

## D1 — 가시성 임계값 일치

planner가 PROD2의 불일치한 임계값에 대한 책임을 명시했다. 모든 프레임의 visible_extent는 alpha ≥ 16 bbox이며 캔버스 적합성, 발 기준선, 크기 band에 이를 사용한다. 배치 후 캔버스 밖에 놓인 alpha < 16 픽셀의 잘림은 기하학적 crop이며 픽셀 수술이 아니다. 프레임마다 버린 비투명 픽셀 수와 최대 alpha를 기록하고 캔버스 내부 RGBA는 변경하지 않는다. 불투명 신체의 실제 넘침은 계속 실패해야 한다.

보존 원본과 리뷰 프레임이 변경되지 않았으므로 이 규칙으로 통과하는 룩은 신규 리뷰를 요구하지 않는다는 planner의 승인을 적용한다. PROD2 R1/P1/재현성 D1을 인용하며 이번 패스에서 재측정했다고 주장하지 않는다.

## D2 — 실행 범위와 판정

기준은 c63eb72b198f7cd213be5bb5dfd0fe4cc2339b73, 작업·push 브랜치는 v8/art-prod1이다. 변경은 art/** 및 docs/v8/artprod3/**만 허용한다. 일반 패스의 CHANGELOG/ROADMAP/ADR 갱신보다 이 명시적 범위를 우선하며 이 문서에 결정과 인계 기록을 남긴다. 게임 코드와 merge는 제외한다.

결손 10룩은 PROD2 표의 best 원본 한 개씩만 재절단한다. 기존 수락 26룩은 비교용 외부 디렉터리에서 재실행하고 원래 수락 파일을 보존한다. 동일하지 않은 재실행 결과는 숨기지 않고 목록화한다. 신규 생성 예산은 0이다.

전체 ≥32/36 및 성인 ≥17/18이면 채택한다. 어린이만 부족하면 성인 세트만 채택한다. 실패 룩은 MISSING과 fallback을 유지한다.

## D3 — 측정 결과에 따른 미채택

실제 결과는 34/36·성인16/18·어린이18/18이다. sleep fit 문제는 해결되었으나 adult_M_hide_wrap_long_tied의 E/idle visible 높이60이 상한58을 넘고, adult_M_fur_cloak_long_tied의 E/work_a panel judge 실패도 남았다. 추가 원본 탐색·생성·배율 보정·band 완화 없이 두 룩을 결손으로 유지한다. 성인 기준 미달이므로 전체·성인 단독 모두 미채택이며 조건부 archive 이동을 수행하지 않는다.

기존26룩의 재실행에서 202/260 PNG 차이가 관측되었다. 잘림이 없어도 alpha16 bbox 중심과 발 기준이 alpha128/alpha>0 혼합 규칙과 달라질 수 있다. 목표의 “or list the diffs”에 따라 전후 SHA를 목록화하고 기존546파일은 보존했다. 기존260PNG와 재실행260PNG는 모두 새 구조 검사를 통과한다. sidecar를 다시 쓰며 불변이라고 주장하지 않는다.

## D4 — planner/owner 채택 결정 (2026-10-23)

> Planner/owner decision 2026-10-23: route I is ADOPTED at S1 34/36 (adults 16/18) on top of PROD2's R1, P1 and D1 — the 17/18 bar was the planner's and is waived for the two adult_M_*_long_tied looks, which fall back per HANDOVER.md.

이 결정은 D3의 미채택·archive 미이동 처분만 대체한다. 측정값34/36·성인16/18·어린이18/18, 두 룩의 MISSING, 원래 S1 기준의 FAIL, 기존26룩 재실행202/260 PNG 차이와 게시546파일 불변은 그대로 보존한다. `adult_M_fur_cloak_long_tied`는 `adult_M_fur_cloak_bun`, `adult_M_hide_wrap_long_tied`는 `adult_M_hide_wrap_bun`으로 대체하고 별도 수락으로 세지 않는다.

TEMPLATE_V3가 캐릭터 표준이며 `art/character/route_i/`와 `MANIFEST.json`이 자산 원천이다. §2의 역사 route와 prod 진단을 `art/character/archive/` 아래로 옮기고 랩 도구는 유지한다. route_b는 기준 트리에 없다. 생성0회·게임 코드 변경 없음·merge 없음의 범위를 유지한다. 실제 N 방향, 노인/머리색 tint, 추가 상의의 세 오너 결정은 미결로 인계한다.
