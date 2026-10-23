# ENV1 아이템 생산·판정 기록

판정: **신규 13개 상태 이미지 중 12개 스타일 수락, 창 1개 수치 탈락. 기존 9개 이미지 재사용 확인. 32px 종류별 구별 요구는 미충족.** 이 문서의 스타일 수락은 judge v2 수치와 실제 이미지 생성 모델의 네 점수, 제작자의 한 줄 판독을 결합한 결과다. [두 신규 리뷰어 판독](ITEMS_READABILITY.md)은 대분류39/44·정확 종류19/44이며 사전 수치 문턱 없는 별도 결과다. 오너 승인을 주장하지 않는다.

## 산출물과 생산량

- 실행 입력: `ENV1_GOAL.md` 전체, `art/style05/STYLE.md`, 잠긴 props·scene 원본과 props_logs·props_hearth 가족 crop.
- 매니페스트: [items_manifest.json](items_manifest.json). 17개 `ItemKind`를 `sim_code=0..16`으로 연결한다. 빈/찬 구운 토기는 한 종류의 두 상태이며, berry basket은 별도의 legacy food 표현이다.
- 신규 PNG: `art/style05/env1/items/<id>.png`, 모두 64×64 RGBA, 중심 앵커(32,32), footprint1×1.
- 원본과 모든 탈락 시도: `/Users/rexxa/work/artenv1-data/items/`. 원본 바이트, SHA-256, 프롬프트, 참조 SHA, 도구 반환 경로, UTC 시각을 보존했다. `started_at`은 프롬프트 준비·호출 이전의 기록 시각, `completed_at`은 도구 반환 관측 시각이며 내부 생성 스케줄 시각은 공개되지 않았다.
- 생성 도구 호출 **27회**: 아이템 원본23회, 실제 이미지 모델 점수표3회, 참조 경로5개 상한을 넘겨 생성 전에 거절된 호출1회. 추가 재생성 없음. 자산별 최대3회 준수.
- 처리: 생성 alpha 보존 → alpha 경계 crop → 비율 고정 Lanczos58px fit → 64px 중심 배치. 양자화·윤곽 덧칠·부분 보정 없음.
- 점수표: [items_model_reviews.json](items_model_reviews.json), `art/style05/env1/items/model_scorecard{1,2,3}.png`. 원본 점수표는 외부 데이터에 보존하고 게시본은 가독성 있는 비율 축소본이다.
- 점수표 모델은 built-in `image_gen__imagegen`이며 구체적인 모델 버전은 도구가 공개하지 않았다. 일반 vision 리뷰 점수를 대신 기입하지 않았다. 텍스트 전용 요청과 달리 그림도 반복했지만 실제 인쇄된 네 점수와 문장이 읽혀 그대로 전사했다.

## 신규 자산

점수 순서는 medium / line / light / palette, 각5점 만점이다. `PASS`는 스타일 판정만 뜻한다.

| id / 경로 stem | 시도 | 최종 수치 | 모델 점수 | 판정과 한 줄 판독 |
|---|---:|---|---|---|
| firewood | 2 | PASS | 5/5/5/5 | PASS. 짧은 쪼갠 단면과 수피로 통나무 더미와 구별. |
| ground_axe | 2 | PASS | 5/5/5/5 | PASS. 넓고 매끈한 돌날과 짧은 자루. |
| spear | 3 | **FAIL** | 5/5/4/5 | **탈락**. 긴 자루와 창끝은 읽히나 담채 분산 초과. |
| cured_hide | 1 | PASS | 5/5/4/5 | PASS. 털 없는 네 갈래 황갈색 가죽. 생가죽과의32px 구별 검토 필요. |
| hide_wrap | 3 | PASS | 5/5/5/5 | PASS. 목·팔 구멍, 사선 여밈과 허리끈. |
| fur_cloak | 2 | PASS | 5/5/5/5 | PASS. 둥근 목구멍, 밝은 털깃과 종 모양 외곽. |
| wolf_hide | 2 | PASS | 5/5/4/5 | PASS. 긴 꼬리·네 발·머리·등줄기. |
| clay | 1 | PASS | 4/4/4/5 | PASS. 눌린 홈이 있는 부드러운 적갈색 덩어리. |
| raw_pot | 1 | PASS | 5/5/4/4 | PASS. 회색 코일 몸체와 손잡이 없는 입구. |
| clay_pot_empty | 2 | PASS | 5/5/4/5 | PASS. 주황 토기, 양쪽 손잡이, 어두운 빈 입구. |
| clay_pot_filled | 1 | PASS | 5/5/4/5 | PASS. 같은 토기 계열의 푸른 수면. |
| meat | 2 | PASS | 5/5/4/5 | PASS. 탁한 적색 덩어리와 한쪽 뼈 돌출. |
| flint_nodule | 1 | PASS | 5/5/5/5 | PASS. 단일 덩이의 껍질과 푸른 회색 박리면. |

최초12개 중4개 수치 통과,8개 탈락. 8개 재시도 후 창·가죽옷2개가 계속 탈락하여 세 번째 시도를 사용했다. 별도로 단일 flint nodule1회 생성. 총23원본 중12개 수치 통과,11개 실패 기록을 모두 보존한다.

창의 `wash_flatness`: 1차0.0347, 2차0.0249, 3차0.0256. 동결된 items 범위 상한은 **0.020179014811321002**다. 세 번 모두 초과하므로 모델 스타일 점수가 좋아도 수락하지 않는다. 현재 `spear.png`는 탈락을 명시한 리허설 후보이고 M-ART2 채택 자산이 아니다. 문턱을 조절하거나 네 번째 시도를 하지 않았다.

다른 최초 탈락 원인은 firewood·ground_axe·hide_wrap·fur_cloak·wolf_hide·meat의 담채 분산, clay_pot_empty의 saturation_q25였다. 전체 정밀 수치와 각 시도 원본 해시는 매니페스트 `attempts[].numeric`에 있다.

## 기존 자산 재사용

`art/style05/env/items/`의 다음9개 PNG는 기존 JSON의 `working_sha256`과 현재 파일 SHA가 모두 같다. 기존 judge v2 수치 통과와 모델 네 점수도 확인했다. 기존 파일과 기록을 바꾸지 않았다.

| 기존 stem | 용도 |
|---|---|
| log_pile_small / log_pile_medium / log_pile_large | Log의1 / 2–8 / 9+ 구간 |
| stone_pile | Stone |
| flint_pile | Flint의 기존 더미 표현; 단일 nodule은 신규 경로 |
| flint_blade | FlintBlade |
| stone_axe | StoneAxe |
| hide | Hide |
| berry_basket | legacy food, 추가 ItemKind가 아님 |

## 수량·상태 파생

17개 종류와 토기 두 상태에 대해 `count_buckets`는 순서대로 **1 / 2–8 / 9+**다. 신규 및 기존 비통나무 sprite는 `*_count_1.png`, `*_count_2_8.png`, `*_count_9_plus.png`로 원본 전체를1·3·9회 배치한다. 단품은64px, 중간은42px, 큰 더미는29px의 원본 전체 비율 축소본을 사용한다. 외곽을 재그리거나 내용 일부를 잘라 조립하지 않는다. 통나무는 기존 소·중·대3장을 그대로 연결한다.

이 버킷은 서수형 수량 아이콘이며 기존 더미 그림 속 돌·통나무 개수를 실제 수량으로 해석하지 않는다. 현재 renderer의≤8 / ≤16 / >16 분류와 다르게 ENV1 요구에 맞춘 리허설 규칙이다. 모든 파생 파일의 SHA는 `count_bucket_sha256`에 남긴다.

고기 fresh는 `meat.png`. spoiled는 재생성 없이 전체 RGB에 `(0.55,0.70,0.45)`를 곱하고 alpha를 그대로 두는 코드 변환을 선언했다. 픽셀별 선택·칠하기는 없다. 현재 sim은 부패 시 meat item을 삭제하므로 지속하는 spoiled 그림은 선언한 보조 상태다. 찬 토기도 현재 ItemStack에 없는 선언 상태다.

## 판독 보드·검증·한계

- `art/style05/env1/items/items_64px_keyed.png`, `items_32px_keyed.png`: 실제 표시 크기 보드.
- `items_64px_blind.png`, `items_32px_blind.png`: 번호만 있는 동일 보드. 정답 순서는 매니페스트 `blind_order`에만 있다. 두 신규 리뷰어가 키 없이32px를 검토했으며 원문44답을 보존했다.
- 아이템 파이프라인을 실제 실행해 PNG·JSON·점수 결합을 검증했고 Python bytecode compile을 통과했다. LSP 진단은 basedpyright가 설치되어 있지 않고 기존 설치 거절 기록이 있어 실행할 수 없었다. 별도 설치나 의존성 추가 없음.
- `items_prepare.py` → `items_review_boards.py` → `items_acceptance.py` 순으로 재현한다. 첫 점수 입력 보드는 최초 생성 당시 Pillow 기본 BICUBIC, 나머지는 LANCZOS 비율 축소를 사용한다. 점수 보드 재현 시 이 차이를 유지한다.
- 스타일 수락은32px의 종류 구별이나 count readability 통과를 증명하지 않는다. 실제 리뷰에서 hide/cured_hide, 돌/부싯돌, 두 도끼와 의복 구별이 실패했다. 수량 자체는 질문하지 않아 NOT_VERIFIED다.
- 오너 승인 없음. M-ART2에는 수락12종·기존9파일·17kind 매핑·버킷·상태 변환을 전달하되 창은 탈락 후보로 분리한다.
