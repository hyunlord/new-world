# ENV1 초상화 제작 기록

상태: **동결 / CANDIDATE_CALIBRATION_FAIL / 승인 0/34**. 채택된 route I 34룩의 후보를 모두 보존한다. 오너 승인이나 게임 설치를 뜻하지 않는다.

## 제작 전 고정 규칙

- 원천: `art/character/route_i/MANIFEST.json`의 `accepted_looks` 34개. 결손 2룩과 fallback용 추가 초상은 만들지 않는다.
- 매 호출에 해당 룩의 실제 `S/idle.png`, 잠긴 `art/style05/anchors/portrait.png`, `scene.png`를 첨부한다.
- 정면 3/4 어깨 위 흉상. 기존 머리와 상의의 색·형태를 유지하며 중립 담채 배경. 머리 전체와 머리장식이 프레임 안에 있어야 한다.
- 정본 128×128, 64×64 표시 확인. 원본은 외부 데이터 루트에 보관한다. 종횡비 보존 resize와 중앙 배치만 허용하며 색 양자화·국소 픽셀 수정은 하지 않는다.
- **고정 얼굴 타원**: 128px 좌표에서 중심 `(64,61)`, 반지름 `(44,52)`. 얼굴 피부 영역의 수동 관찰 경계점을 이 타원에 대입한다. 머리카락은 얼굴 타원 대상이 아니며 별도 머리 bbox가 네 변에서 2px 이상 안쪽인지 확인한다. 배경 alpha나 캔버스 bbox를 얼굴 검사의 대용으로 쓰지 않는다.
- 판정 경계는 생성 전에 고정한다. 숫자 판정은 기존 judge v2의 8개 특징 및 `bounds()` 공식을 그대로 쓴다. 잠긴 초상 원본의 전체·흉상·머리와 어깨 crop을 가족 기준으로 사용하고 leave-one-out 검사를 보존한다. crop은 원본 좌표 `(0,0,1254,1254)`, `(150,20,1254,1254)`, `(250,20,1050,1150)`이다. 같은 한 원본에서 나온 crop이므로 독립된 작품 3개의 검증으로 해석하지 않는다.
- 기존 painterly/woodcut/photo 음성 대조군을 그대로 재검사하며 실패를 숨기거나 기준을 조정하지 않는다.
- 실제 이미지 모델의 medium/line/light/palette 4점수가 모두 4/5 이상이어야 한다. 수치·이미지 모델·구조·identity 독해를 모두 통과해야 `accepted=true`.
- 룩당 최대 2회. 이 lane은 scorecard를 포함해 최대 78호출. 실패한 후보와 수락되지 않은 이유를 보존한다.

## 외부 증거

원본·프롬프트·scorecard·호출 ledger: `/Users/rexxa/work/artenv1-data/portraits/`.

## 결과

| 항목 | 결과 |
|---|---|
| 실제 이미지 생성 호출 | 36회: 최초 34 + 재시도 2 |
| 실제 이미지 모델 scorecard 호출 | 7회: 후보 batch 5 + 재시도 1 + 음성 대조 1 |
| 총 호출 / 실패 호출 | **43 / 0** (배정 78 이하) |
| 룩별 시도 | `adult_F_fur_cloak_long_tied`, `adult_F_loincloth_bun` 각 2회, 나머지 32룩 각 1회 |
| 전체 시도 수치 / 모델 통과 | 34/36 / 31/36 |
| 선택된 34장 수치 / 모델 통과 | 33/34 / 30/34 |
| 선택된 34장 구조 / 머리·상의 identity 통과 | 30/34 / 33/34 |
| 최종 수락 | **0/34**, calibration FAIL로 전체 후보 유지 |

가족 crop leave-one-out은 3/3 통과했지만 같은 초상 원본의 세 crop이다. 독립 작품에 대한 일반화 증거가 아니다. 기존 음성 대조군은 수치와 이미지 모델 모두 woodcut 1/3만 거절하고 painterly와 photo를 잘못 통과시켰다. 실제 모델은 photo 대조군에도 4/4/4/4를 주며 사진을 잉크 담채로 잘 변환했다고 설명했다. 이 오판을 그대로 보존했고 기준·대조군·점수를 바꾸지 않았다. 따라서 후보별 좋은 점수는 스타일 수락 증거로 승격하지 않는다.

선택본의 수치 실패는 `adult_M_fur_cloak_short`의 wash_flatness다. 이미지 모델 실패는 `child_F_hide_wrap_bun`, `child_F_loincloth_long_tied`의 medium 3점과 `child_M_hide_wrap_bun`, `child_M_hide_wrap_short`의 medium/line 3점이다. 첫 `adult_F_loincloth_bun`은 수치(edge_density, wash_flatness)와 모델(3/3/3/4) 실패였으며 2차 시도는 두 판정을 통과했다. 두 시도를 모두 남겼다.

이미 계획한 두 재시도를 끝낸 뒤 선택 lane을 동결했다. 남은 실패를 더 생성하거나 판정 기준을 완화하지 않았다. calibration 실패가 해소되지 않은 상태이므로 호출 한도가 남았다는 이유만으로 후보를 수락하지 않는다.

얼굴과 머리 경계는 실제 그림을 보고 128px 좌표에서 수동 기록했다. 얼굴 bbox는 관찰 오차를 고려해 약 2px 확장한 보수적 경계이며, 네 모서리를 고정 타원에 대입했다. 선택 34장 모두 얼굴 bbox가 타원 안에 있고, 눈으로 확인한 머리 잘림은 없었다. 다만 생성 전에 고정한 2px 머리 여백 기준에서는 `adult_F_fur_cloak_long_tied` 2차, `adult_F_hide_wrap_bun`, `child_M_fur_cloak_bun`, `child_M_loincloth_bun`의 보수적 bbox가 실패한다. 이 4장을 구조 PASS로 표시하지 않았다. bbox는 alpha/캔버스에서 추정하지 않았다. 검사 오버레이는 외부 `structural_inspection_overlay.png`에 있으며 빨강은 머리 bbox, 초록은 얼굴 bbox, 청록은 고정 타원이다. 수동 bbox는 정밀 segmentation 증거가 아니다.

`adult_F_fur_cloak_bun`은 bun/상의 범주는 맞지만 원래 스프라이트에서 확인되지 않는 뼈 머리핀이 추가돼 identity를 실패로 기록했다. `adult_F_fur_cloak_long_tied` 첫 시도는 anchor의 bun을 복사해 실패했고 2차에서 묶은 긴 머리로 수정됐다. 첫 두 호출 이후에는 원본 `S/idle.png`와 별도로 같은 그림의 alpha bbox를 기하학적으로 자른 뒤 8배 nearest 확대해 네 번째 identity 보조 참조로 첨부했다. 원래 `S/idle.png`와 두 locked anchor는 모든 호출에 그대로 들어갔다. 저해상도 스프라이트로부터 고유한 얼굴 동일성을 입증하지는 못했으므로 전부 `NOT_ESTABLISHED_FROM_LOW_RES_SPRITE`를 보존했다.

정본은 원본 전체 프레임을 128×128 LANCZOS로 균일 축소한 결과다. 선택 보드는 각 룩을 128px와 64px로 나란히 보여 준다. 색 보정, 양자화, 국소 수정은 없다. 보드는 PNG 무손실 zopfli 재압축 후 1,483,844 bytes이며, `portrait_finalize.py`가 재압축 전 생성 픽셀과 재압축 후 디코딩 픽셀의 바이트 일치를 확인한다. 시도별 PNG도 모두 1.5 MB 이하이다. 원본과 scorecard는 크기 제한 적용 대상인 repo 납품물에 넣지 않고 외부 원본으로 남겼다.

## 파일과 재현

- 정본 36개: `art/portrait/env1/<look>_try<n>.png`. 34개 선택본 및 각 룩의 모든 시도는 `portrait_manifest.json`에 명시한다.
- 오너 보드: `art/boards/portraits_34.png`.
- `portrait_calibration.json`, `portrait_negative_controls.json`: 고정 수치 기준, held-out, 음성 대조 결과.
- `portrait_numeric.json`, `portrait_model_scores.json`, `portrait_inspections.json`: 독립된 수치 결과, 실제 scorecard 전사, 수동 구조·identity 관찰.
- `portrait_manifest.json`: 34룩과 시도 36개의 원본/정본/참조/프롬프트/scorecard SHA-256, 호출 수, 선택 상태.
- 외부 `ledger.json`, `score_ledger.json`, `<look>_try<n>.prompt.json`, `score_*.png`: 실제 생성 원본과 호출 출처.

실행은 repo 루트에서 `uv run --with pillow --with numpy --with scipy --with scikit-image --with pydantic python -m art.pipeline_env1.portrait_process` 다음 `... python -m art.pipeline_env1.portrait_finalize` 순서다. 처음부터 보드를 다시 만들면 첫 finalize는 1.5 MB 제한에서 멈출 수 있다. 그때 외부 도구 환경 `uv run --with zopfli`에서 `zopfli.png.optimize(..., num_iterations=5, lossy_transparent=False, lossy_8bit=False)`로 **보드만 무손실 재압축**하고 finalize를 다시 실행한다. pixel 일치 검사가 다르면 중단한다. 모델 심사에 사용한 기존 contact sheet는 재실행으로 덮어쓰지 않는다. 생성 완료 후 원본 복사를 확인한 다음 정규화해야 한다. 생성 도중 실행한 최초 정규화는 아직 없는 원본 경로에서 실패했고, 완료 후 순차 재실행으로 해결했다. 이는 추가 이미지 생성 호출이 아니다.

검증: 세 Python 파일 Ruff 통과, basedpyright 오류·경고 0, 실제 정규화와 manifest 작성 성공, 34룩 원천 집합 일치, 원본·정본·참조·scorecard 해시 확인, 128×128 크기 및 1.5 MB 상한 확인. 보드와 구조 오버레이를 실제로 열어 확인했다. 게임 화면·FPS·최종 오너 검토는 이 초상화 lane의 증거에 포함하지 않는다.
