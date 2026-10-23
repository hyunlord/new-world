# ART-ENV1 동물 제작·검증

**판정: CANDIDATE_NOT_ADOPTED / 동물 계열 수치 음성 대조 보정 FAIL / 최종 독립 가독성: 사슴 PASS·늑대 FAIL.**
늑대와 사슴의 전체 도형 시트 및20개 프레임을 제작했다. 원본은 외부 데이터
경로에 보존했고 픽셀 수정·양자화·부위 조립은 하지 않았다. 오너 승인은 아니다.

## 제작과 검사

| 종 | 생성 시도 | 현재 후보 | 구조 | native-panel v2 | image-model medium/line/light/palette | 한 줄 읽기 |
|---|---:|---:|---:|---:|---|---|
| wolf | 3/3 | try3 | 10/10 | 10/10 | 5/5/5/5 | 공격·누운 자세는 읽히지만 D가 서기를 앉기로, 정면 걷기를 서기로 읽어 자세 FAIL이다. |
| deer | 1/3 | try1 | 10/10 | 10/10 | 4/4/4/5 | 가지뿔·긴 다리의 사슴이며 풀뜯기는 땅으로 내린 목과 머리로 구분한다. |

늑대try1은 S/dead의 주둥이 불투명 픽셀이 원본 오른쪽 경계에 닿아 탈락했다.
try2는 구조를 통과했지만 round0에서 자세를 읽기 어려워 마지막try3를 생성했다.
현재20개 전체 프레임 구조 검사를 통과했다. 늑대 재시도는3/3 소진했으며 사슴2회가
남았지만 사슴은 바꾸지 않았다. 자산 생성4회+스타일/음성대조 scorecard5회=실제 image_gen9회.
[시도 원장](animal_attempts.json)에 원본·prompt·고정 첨부2장의 SHA256을 기록했다.

실제 이미지 모델이 각 후보 시트를 고정 person+scene 원본과 비교해4개 점수와
문장을 scorecard로 출력했다. 실행자가 실제 카드의 문자를 JSON으로 전사했다.
[최종 늑대 카드](../../../art/animal/boards/round1/wolf_try3_style_scorecard.png),
[사슴 카드](../../../art/animal/boards/deer_style_scorecard.png)는 열람용½축소본이며
원본 카드와 해시는 원장에 보존된다. 별도 사람 리뷰나 임의 점수로 대체하지 않았다.

## 동물 family 보정의 한계

GOAL은 animal family를 요구하지만 기존 동물 anchor 경로는 제시하지 않는다.
따라서 새 두 종의 S/stand 원본 패널을 각각 고정하고, 서로를 제외한 실제
leave-one-out으로 검증했다. `art/pipeline2/judge.py`의 측정·허용치는 변경하지
않았다. 두 held-out2/2 및20개 원본 패널 수치 검사는 통과했다. try3는 wolftry2+deertry1
보정에 대해 평가했다. 보정 JSON의 SHA256은 round0과 동일하며 새 후보로 범위를
다시 맞추지 않았다.

그러나 기존 painterly/photo 음성 대조가 새 animal 범위 안에 들어갔고 woodcut만
거절됐다. **음성 대조1/3거절, 보정 FAIL**이다. 패널 JSON의 수치 통과는 이 실패를
지우지 않는다. 문턱을 조정하거나 현재 후보를 완전 수락으로 승격하지 않는다.
동물 전용 음성 대조가 아닌 기존 통제 이미지라는 한계도 함께 남긴다.

추가 실제 image_gen 점수카드는 painterly2/2/3/3, photo2/3/3/3으로 둘 다
스타일에서 거절했다. 따라서 기존 woodcut의 수치 거절까지 포함하면 수치AND모델
결합은 세 음성 대조를 거절한다. 이 보충 결과가 원래 수치 보정 FAIL을 철회하거나
동물 전용 통제 검증을 대신하지 않는다. [실제 카드·해시·전사](animal_negative_model_review.json).

- [보정 원본](animal_judge_calibration.json)
- [20패널·음성대조·모델 증거](animal_style_evidence.json)
- [규격·좌표·시트 계약](../../../art/animal/TEMPLATE_A1.md)
- 재현: `python3 -m art.pipeline_env1.animal_normalize`
- 수치·보드: `uv run --with scikit-image --with pillow --with scipy python -m art.pipeline_env1.animal_evidence`

## 독립 가독성 리뷰

익명 혼합20프레임 보드는 종·활동 정답 없이 번호만 제공한다.
[round0 1×](../../../art/animal/boards/round0/blind_frames_1x.png),
[round0 2×](../../../art/animal/boards/round0/blind_frames_2x.png).
정답은 `/Users/rexxa/work/artenv1-data/animals/review_answer_key.json`에 분리했다.
리뷰어마다 종별10프레임·배율별 종≥9/10, 자세≥8/10을 각각 요구한다.
두 신규 독립 리뷰어의 round0 응답을 [고정 채점 규칙](ANIMAL_REVIEW_PROTOCOL.md)에
따라 채점했다. [round0 80개 ID별 대응·해시·시간](animal_reviews_round0.json), 원문 바이트 그대로
복사한 [A 응답](reviews/animals/a.json), [B 응답](reviews/animals/b.json)을 보존한다.
각 행의 분모는10이며 종≥9·자세≥8을 각각 적용했다.

| 리뷰어 | 종 | 배율 | 종 정답/10 | 자세 정답/10 | 판정 |
|---|---|---|---:|---:|---|
| A | deer | 1× | 10 | 8 | PASS |
| A | deer | 2× | 10 | 9 | PASS |
| B | deer | 1× | 10 | 10 | PASS |
| B | deer | 2× | 10 | 10 | PASS |
| A | wolf | 1× | 0 | 4 | FAIL |
| A | wolf | 2× | 10 | 7 | FAIL |
| B | wolf | 1× | 10 | 5 | FAIL |
| B | wolf | 2× | 10 | 6 | FAIL |

round0의 엄격한 어휘 목록 적용은 역사 기록으로 보존하며 그 한계도 명시한다. A1×의 `Wolf-like canid`는
의도한 늑대를 유보적으로 인식한 응답이다. 종을 전혀 못 알아봤다는 뜻이 아니며
별도 qualified-recognition 민감도에서는10/10이다. 고정 엄격 채점은 명시적 `wolf/grey wolf` 확정과 같게 처리하지
않았다. `bounding/striding/trotting`은 고정 동의어 목록 밖이며, 뒤의 `or running`
등으로 첫 답을 교체하지 않았다. 따라서 A의 사슴 자세2개도 bounding 때문에
감점됐다. 이는 이미지 내용을 전혀 알아보지 못했다는 뜻이 아니다. 반면 이 표현들을
넓게 인정해도 늑대의 공격·사체 혼동은 남으며 기존 FAIL을 수락으로 바꾸지 않는다.
이는 프로토콜의 의미 범주를 닫힌 단어 목록으로 해석한 한계다. round1은 동일한
종≥9/10·자세≥8/10 문턱과 첫 관찰 원칙을 유지하면서 일상적인 의미로 채점했다.
따라서 두 라운드의 수치 차이를 같은 채점법으로 측정한 개선량으로 주장하지 않는다.

늑대의 S/E attack은 낮은 달리기·crouch로, dead는 압축된 bound·crouch·fall로
읽혔다. 작은 S정면 stand/walk/run도 서로 겹친다. 사슴 graze와 달리기 윤곽은
상대적으로 명확하다. 사체의 옆으로 누운 자세를 맞혀도 사망과 수면·휴식의
의미 구분을 입증한 것은 아니다.

A는1×→2×순서로49초·42초, B는2×→1×순서로54초·64초를 기록했다.
이는 모델/도구의 관측 경과 시간이다. A는 파일 쓰기 시간을 제외했고 B는 응답
기록을 포함했으므로 서로 같은 사람 반응시간 지표로 비교하지 않는다. UTC구간
차이·양수 시간·각20개ID·40개 원본 프레임해시와 복사본 바이트 동일성을 검사했다.
프로토콜 파일 수정시각은 두 응답 JSON 파일 작성시각보다 빠르다.

round0 이후 마지막try3에서 큰 벌린 입·구부린 앞발의 공격, 완전히 옆으로 누운
사체, 분리된 정면 보행/달리기 자세를 강조했다. 이 생성 단계에서 사슴의 원본·프레임·sidecar·MANIFEST 전체는
바이트 동일하게 유지했다. 최종 리뷰 후 두 종의 sidecar·MANIFEST에는 대기 대신
확정된 후보 판정만 기록했다. 모든 PNG와 사슴 원본은 변경하지 않았다.
음성 대조 보정 실패는 별도의 미해결 조건으로 유지한다.

### 최종 늑대 try3, round1

공격의 입과 앞발을 크게 읽게 하고 사체를 낮은 옆모습으로 만들었다. S어깨의
수동 기준은y252→474, 28/222공통배율이다. 사체는 기존56px몸길이 규칙을 그대로
사용했다. 최종10개 프레임 구조·고정 범위 수치통과, 실제 모델5/5/5/5다.
새 보드는 [round1 1×](../../../art/animal/boards/round1/blind_frames_1x.png),
[round1 2×](../../../art/animal/boards/round1/blind_frames_2x.png)이며 ID는
`R1-1X-*`/`R1-2X-*`다. 별도 키는
`/Users/rexxa/work/artenv1-data/animals/review_answer_key_round1.json`에 둔다.
이전 A/B와 다른 **새** 리뷰어 C/D가 정답 없이 평가했다.
[round1 80개 ID별 대응·원문·판정 근거·해시](animal_reviews_round1.json),
[C 원문](reviews/animals/c.json), [D 원문](reviews/animals/d.json)을 보존한다.
종과 자세를 각 집단의 10개 분모로 검사했으며 평균으로 실패를 상쇄하지 않았다.

| 리뷰어 | 종 | 배율 | 종 정답/10 | 자세 정답/10 | 판정 |
|---|---|---|---:|---:|---|
| C | deer | 1× | 10 | 9 | PASS |
| C | deer | 2× | 10 | 10 | PASS |
| C | wolf | 1× | 10 | 8 | PASS |
| C | wolf | 2× | 10 | 8 | PASS |
| D | deer | 1× | 10 | 9 | PASS |
| D | deer | 2× | 10 | 9 | PASS |
| D | wolf | 1× | 10 | 7 | FAIL |
| D | wolf | 2× | 10 | 7 | FAIL |

모든 종 응답은 명시적인 wolf 또는 deer/stag다. round0의 `wolf-like canid`와 같은
유보적 종 인식이 없어 qualified 민감도를 적용해도 round1 점수는 같다.
`bounding`과 방향성 있는 달리기 도약은 일상적 의미의 run으로, 공격으로 설명한
앞발 들기·타격 자세는 attack으로 채점했다. 설명의 첫 주 관찰을 보존했으며,
나중에 제시한 대안을 정답으로 골라 바꾸지 않았다. 예를 들어 C의 S/run은 주 관찰이
pounce라서 run 오답이고, D의 E/dead 사슴은 주 관찰이 airborne leap라서 dead 오답이다.
옆으로 누워 잠/휴식 중이라는 관찰은 기존 `dead (lying)` 계약에 따라 자세만 인정한다.

늑대는 D가 양쪽 배율에서 S/E stand를 sitting으로, S/walk를 standing으로 읽었다.
C도 양쪽 배율에서 S/walk를 standing으로 읽었고 S/run을 pounce로 읽었다.
이는 후지·꼬리가 앉은 윤곽처럼 보이고 정면 걸음의 다리 분리가 부족한 문제를 시사한다.
공격은 양쪽 리뷰어에게 읽히지만 사람처럼 선 모습이라는 지적이 남았다.
사체는 누운 자세와 수면/휴식을 구분하는 사망 의미까지 입증하지 않는다.
사슴은 모든 집단을 통과했으나 D의 E/dead 도약 혼동과 C1×의 S/walk 서기 혼동은 남긴다.

C는1×→2×에서48초·59초, D는2×→1×에서57초·60초를 기록했다.
이는 모델/도구 경과 시간이며 사람 반응 시간이 아니다. 프로토콜 해시는 이전과
동일하고 파일 수정시각은 두 새 리뷰 구간 시작보다 빠르다. 원문 사본의 바이트 동일성,
각20개ID·총80개 채점 대응,40개 키 프레임 해시와 네 양수 UTC시간 구간을 검증했다.
늑대는3/3 소진했으므로 추가 생성 없이 FAIL 증거를 보존한다. 사슴도 변경하지 않았다.

이전 늑대 전체는 `art/animal/archive/wolf_try2/`, 보드는 `boards/round0/`,
[엄격+qualified 민감도 점수](animal_reviews_round0.json),
[원래 스타일 판정](animal_style_evidence_round0.json),
[외부 원본 고정 해시](animal_round0_frozen.json)로 보존했다.
현재 합성기의 wolf PNG는try3 후보이므로 이전 렌더와 혼동하면 안 된다.
[두 라운드 요약·현재 판정](animal_reviews.json)은 각 고정 보고서 해시를 연결한다.
사슴의 가독성 PASS와 별개로 수치 음성 대조 보정 FAIL이 남아 두 종 모두 정식 채택하지 않는다.
현재 MANIFEST·sidecar의 status는 `CANDIDATE_NOT_ADOPTED`이며 늑대는
`READABILITY_FAIL`, 사슴은 `READABILITY_PASS`를 별도 기록했다. 최종 판정 기록 후
20개 PNG 해시가 round1 키와 동일함을 재검증했다.

## 인수인계

`art/animal/{wolf,deer}/{S,E}/{stand,walk,run,attack|graze,dead}.png`와
각 JSON, 종별 MANIFEST가 현재 후보이다. 살아있는 프레임96×80 anchor(48,72),
사체96×64 anchor(48,56). 명세의 dead64높이와y72는 동시에 성립할 수 없어
8px아래 여백을 유지하는y56으로 명시했다. 사슴 S어깨40px, 늑대28px를 수동
기준점으로 보정한 동일 배율을 양방향 모든 upright에 적용한다. 사체는 몸길이로
별도 균일 축소한다. E방향 어깨의 투시상 높이 차이는 숨기지 않으며 따로 크기
수정하지 않았다. W는 E전체반전이다. ½×이하는 동물 삼각형 LOD를 쓴다.

합성기는 후보임을 provenance에 기록하고 사용할 수 있지만 게임에 설치하거나
정식 채택을 주장하면 안 된다. 독립 가독성, calibration음성대조, 실제 합성화면,
오너판단은 각각 별도 검증이다.
