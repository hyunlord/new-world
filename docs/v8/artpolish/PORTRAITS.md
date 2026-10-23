# ART-POLISH 초상화 — 오너 판단 대기

**34장 후보를 유지한다. 구조 34/34, 동일인 매칭 A 34/34 · B 34/34. 오너 채택 0/34.** 새 생성은 지정된 9룩만 각각 1회이며, 나머지 25장은 ENV1 선택본의 SHA-256과 바이트가 같다. 동일인 매칭은 같은 나이·성별 안에서 머리와 상의가 다른 네 스프라이트를 구별하는 시험이다. 저해상도 스프라이트가 개인 고유의 얼굴을 보증한다는 뜻은 아니다.

## 재생성과 보존

| 룩 | 재생성 사유 | POLISH 시도 | 새 수치 진단 실패 항목 |
|---|---|---:|---|
| adult_F_fur_cloak_long_tied | 머리 여백 | 1/2 | wash_flatness |
| adult_F_hide_wrap_bun | 머리 여백 | 1/2 | edge_density |
| child_M_fur_cloak_bun | 머리 여백 | 1/2 | wash_flatness |
| child_M_loincloth_bun | 머리 여백 | 1/2 | edge_density |
| adult_F_fur_cloak_bun | 원본에 없는 머리핀 | 1/2 | wash_flatness |
| child_F_hide_wrap_bun | medium 점수 | 1/2 | edge_density |
| child_F_loincloth_long_tied | medium 점수 | 1/2 | edge_density |
| child_M_hide_wrap_bun | medium/line 점수 | 1/2 | edge_density |
| child_M_hide_wrap_short | medium/line 점수 | 1/2 | edge_density |

각 요청에 실제 `art/character/route_i/<look>/S/idle.png`, 그 그림에서 머리·목 부분을 자른 **8× nearest** 참조, 잠긴 `art/style05/anchors/portrait.png`를 첨부했다. 이전 ENV1의 `identity_enlarged` 파일은 전신 bbox 확대이므로 재사용하지 않았다. 새 crop은 adult에서 alpha bbox 상단 40%, child에서 상단 55%를 사용해 머리 전체가 들어오는지 직접 확인했다. crop 좌표·배율·참조 SHA-256은 호출 원장에 남겼다. 이 crop은 참조용이며 납품 초상화에 붙이거나 합성하지 않았다.

원본 전체 정사각형을 128×128 LANCZOS로 균일 축소했다. 정본에 국소 수정, 팔레트 양자화, 색 보정, 부분 조립은 없다. 원본과 호출 도구의 저장본도 모두 보존했다. 원본·참조·scorecard는 `/Users/rexxa/work/artpolish-data/portraits/`에 있다.

## D1 스타일 진단

- 전체 선택본 judge v2 수치 결과는 **24/34**다. 새 9장은 위 표의 항목이 실패했고, 보존한 `adult_M_fur_cloak_short`는 기존 wash_flatness 실패를 유지한다.
- 새 9장 실제 이미지 모델 scorecard는 모두 네 항목 4점 이상이다. P02는 4/5/5/5, 나머지 8장은 5/5/5/5다. 보존한 25장의 기존 모델 결과도 모두 4점 이상이다.
- 모델에는 원본을 300px로 균일 축소한 9장 contact sheet와 초상 anchor를 넣었다. 수치 판정은 새 원본 해상도에 적용했다. 모델의 높은 점수와 수치 실패를 서로 덮어쓰지 않는다.
- **두 스타일 진단은 D1에 따라 수락 게이트가 아니다.** 원래 실패 근거는 `docs/v8/artenv1/portrait_negative_controls.json`, `portrait_model_scores.json`, `PORTRAITS.md`다. photo/painterly 대조군을 통과시켰던 사실과 원래 임계값을 유지한다.

## 구조와 동일인 매칭

ENV1과 같은 고정 얼굴 타원, 중심 `(64,61)`·반지름 `(44,52)`을 적용했다. 얼굴 피부·귀의 수동 관찰 bbox 네 모서리가 타원 안에 있는지 확인하고, 머리카락·잔머리까지 포함한 별도 머리 bbox에서 사방 여백 2px 이상을 검사했다. 새 9장은 원본과 정본의 3× nearest 보기를 직접 보고 약 ±2px 관찰 불확실성을 포함하는 보수적 경계를 기록했다. 기존 25장의 실제 구조 기록과 파일 해시도 유지했다. 이는 정밀 segmentation 결과가 아니다.

블라인드 문항은 128px 초상 하나와 **2× nearest 스프라이트 네 개**를 보여 준다. 정답과 같은 age/sex 안에서 다음 고정 순서로 대조군을 택했다: 같은 상의·다른 머리의 사전순 첫 룩, 같은 머리·다른 상의의 첫 룩, 둘 다 다른 첫 룩. 부족하면 남은 같은 집단에서 사전순으로 보충한다. 문항 순서와 A/B/C/D 위치는 seed `20261024`로 섞었다. 문항 파일·표시는 P001–P034와 A/B/C/D만 포함하며, 정답·룩 경로는 별도 외부 key에 둔다.

두 새 리뷰어 `/root/portrait_blind_a`, `/root/portrait_blind_b`는 생성 이력·정답 없이 각각 **34/34**를 맞혔다. 기준은 리뷰어별 ≥30/34다. 원답·문항·정답 해시와 채점은 [blind_portraits.json](blind_portraits.json)에 있다. 후보 선택이나 threshold를 결과에 맞춰 바꾸지 않았다.

## 오너 보드와 M-ART2 경로

`art/boards/portraits_owner.png`는 34장 모두를 정본 128px, BILINEAR 축소 64px, 해당 route I 스프라이트 0.5× LANCZOS로 보여 준다. 스프라이트 배율은 보드 제목과 manifest에 표시했다. 블라인드 시험의 2× 스프라이트는 그대로 동결했다. 1× 스프라이트를 쓴 외부 비교판도 데이터 폴더에 보존했다.

최종 PNG는 **1,472,492 bytes**이며 zopfli 무손실 재압축 전후 디코딩 RGB 바이트 일치를 확인했다. 모든 정본과 커밋 대상 JSON도 1,500,000 bytes 미만이다. 보드를 실제로 열어 34개 이름·128/64 보기·스프라이트를 확인했다.

M-ART2는 [portrait_manifest.json](portrait_manifest.json)의 34개 `records[].path`를 사용한다. 새 9장은 `art/portrait/polish/<look>_try1.png`, 나머지는 원래 `art/portrait/env1/<look>_try<n>.png`다. `owner_adopted=false`이며 게임 설치나 오너 승인을 뜻하지 않는다.

## 호출 수와 검증

실제 built-in imagegen **10회 = 자산 9 + scorecard 1 + 실패 0**. lane 배정 22 이하, 모든 룩의 새 시도는 1/2다. [portrait_generation_ledger.json](portrait_generation_ledger.json)은 호출 전 예약, 전체 프롬프트, 모든 입력 해시, 도구 원본/raw 쌍 해시, 완료 관찰을 보존한다. 예약 시각과 배치 수신 후 복사 시각은 관찰 경계이며 도구 내부 실행 시각이라고 주장하지 않는다.

검증: 실제 정규화·manifest 실행 성공, Python tests **5 PASS**, Ruff PASS, basedpyright **0 errors / 0 warnings**, 모든 호출의 원본/raw·입력 해시 대조, 34문항의 실제 128px 초상·네 2× 스프라이트를 원천 픽셀과 대조, 기존 25장 바이트 동일성, 최종 보드 크기 확인. 게임 코드·게임 화면·FPS는 이 lane 범위에 없다.

재현은 repo 루트에서 `portrait_prepare.py`로 요청을 준비한 뒤 실제 도구 출력 원장을 보존하고, `python -m art.pipeline_polish.portrait_packets`로 보드와 문항을 만든다. 보드는 ENV1과 같이 `zopfli.png.optimize(..., num_iterations=15, num_iterations_large=15, lossy_transparent=False, lossy_8bit=False)`로 무손실 재압축한 다음 `python -m art.pipeline_polish.portrait_finalize`를 실행한다. finalizer는 1.5MB 초과 보드를 거절한다. `uv run --with pillow --with pydantic --with pytest python -m pytest -q art/pipeline_polish/portrait_test.py`로 실제 산출물을 검사한다.

변경 파일은 `art/pipeline_polish/portrait_{prepare,packets,finalize,test}.py`, 새 초상 PNG 9개, 오너 보드, 이 문서와 `docs/v8/artpolish/portrait_*.json`이다. 기존 ENV1 자산과 기록을 수정하지 않았다.
