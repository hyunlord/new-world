# ART-POLISH 결정 기록

## D1 — 초상·동물 스타일 계측은 보고 전용

목표 문서가 선언한 계측 변경을 그대로 적용한다. 초상과 동물은 judge v2
수치 및 이미지 모델 스타일 점수표를 남기되 합격 여부를 결정하는 게이트로
사용하지 않는다. 구조, 각각의 블라인드 시험, 오너 보드 등재를 별도로
판정한다. 이 조건 충족도 오너의 채택을 뜻하지 않는다. 아이템은 보정된
items family의 judge v2 스타일 게이트를 유지한다.

근거는 ENV1의 `docs/v8/artenv1/portrait_negative_controls.json`,
`portrait_calibration.json`, `animal_style_evidence.json`,
`animal_negative_model_review.json`이다. 초상 사진 대조군을 모델이
4/4/4/4로 평가했고, 초상·동물 수치 계측도 부정 대조군을 통과시켰다.
동물의 추가 이미지 모델 대조군은 사진·회화풍을 거절했으므로, 동물의
수치 계측 실패와 모델 점수표 결과를 같은 실패로 뭉뚱그리지 않는다.
기존 ENV1 판정이나 임계값은 수정하지 않는다.

## D2 — 오너의 STRUCT 방향과 어두운 벽 윗면

2026-10-24 오너 결정, 목표 문서 원문:

> the STRUCT direction is adopted — ground from above, everything standing in the same front 3/4, walls/hill faces/water/edges drawn by code — with wall caps made dark and nearly patternless

이번 합성기 기본값은 cap value ×0.45, saturation ×0.40, 9px box blur,
비벽 이웃 방향의 2px `#141210` 80% 선, 면이 걸린 남쪽 1px value +0.08
하이라이트다. 0.35/0.45/0.60 세 선택지를 보드에 남긴다. M-ART2가 복사할
권위 값은 `art/compositor/wall_params.json`이다. 게임에 설치하지 않는다.

## D3 — 범위, 실행 기록과 예산

- 기준: `b6baa73f1f64983fee6aea08ed783a038ba2a4ce`, 작업/게시 브랜치
  `v8/art-polish`. 이번 명시적 목표가 상시 `v8/wip` 규칙을 이 작업에 한해
  대체한다. merge 없음.
- 수정은 `art/**`, `docs/v8/artpolish/**`만. 전역 CHANGELOG/ROADMAP/스킬
  문서의 갱신은 범위 밖이므로 이번 결정·보고서에 기록한다.
- 원본·실행 로그·블라인드 비공개 정답은 `/Users/rexxa/work/artpolish-data/`.
  기존 자산과 ENV1 결과는 보존한다. 각 커밋 파일은 1,500,000 bytes 이하.
- 생성 예산 100회: 동물 최대 9회, 초상 최대 22회, 아이템 최대 45회,
  미배정 24회. 실패한 요청·점수표도 센다. 미배정분 사용은 사유와 배정
  변경을 먼저 기록하며 자산별 시도 상한은 그대로다.
- 각 lane은 호출 전 원장에 프롬프트·입력 해시·시각을 기록하고, 수신 후
  도구 경로·원본 복사본 해시를 남긴다. 이미지 생성은 내장 image_gen.
- 시작 UTC `2026-10-23T15:38:48Z` (KST 2026-10-24 00:38:48), 목표 4시간,
  hard stop 6시간. 목표 문서의 drop order 및 never-drop 목록 유지.
- 블라인드 판독자는 새 context의 독립 에이전트다. 정답·파일명·이전
  판독·기준 점수는 보여주지 않는다. 원시 답변과 오답도 보존한다.

## D4 — 초상 대상과 동물 방향

초상 재생성은 기존 결함 대상의 합집합인 9명만, 각 최대 2회다. 다른 25명은
ENV1 선택본을 유지한다. 명단과 근거는 초상 manifest에 기록한다.
기존 사슴 S 프레임은 정면이므로 사슴도 35° viewer-left 방향으로
재생성한다(최대 2회). 늑대는 최대 3회. 최종 채택은 오너 대기다.
