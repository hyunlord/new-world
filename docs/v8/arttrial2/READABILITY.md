# ART-TRIAL2 가독성 판정

**판정: FAIL.** 최종 캡처4장 × 독립 이미지 전용 리뷰어2명 × 인물12명=96개 응답을 실제 정답과 대조했다. 두 리뷰어 모두 어느 캡처에서도 GOAL6.3 기준을 충족하지 못했다. 과거 첫 리뷰는 합산하지 않았다.

## 채점 규칙

- 동작 AND 운반 여부를 모두 맞혀야1점. 1×는8/12,2×는10/12 이상이어야 한다.
- standing=idle, walking=walk, lying down=sleep. 명시적인 carrying 표현을carry로 인정한다. `or`로 서로 다른 동작 가능성을 제시하면 정답을 골라 주지 않는다. 단순 sitting은knap, holding a tool은chop으로 인정하지 않는다.
- 운반에는 실제 손에 든 돌도끼·통나무 묶음·바구니·노인 지팡이를 포함한다. 노인은held=none이어도 composer가 지팡이를 표시하므로 운반=yes이다.
- 첫8명 성별·연령·물품 각1점,20/24 기준이다. 실제 성별 키가NOT_RENDERED_SINGLE_ADULT_BODY이므로0/8 처리한다. unknown을 정답으로 인정하거나 분모를16으로 줄이지 않는다.
- axe-like/log-like/basket-like는 해당 물품 범주로 인정한다. 일반적인 검은 형태나 도구, 서로 다른 후보를or로 제시한 물품 답변은 오답이다. 정확한 문자열 정규화표와96개 개별 판정은READABILITY.json에 보존했다.
- 원문과 캡처·정답 파일 SHA256을 보존했다. 보조 묘사를 사용하여 불명확한 동작 답변을 정답으로 바꾸지 않았다.

## 결과

| 리뷰어 | 바탕·배율 | 동작 AND 운반 | 기준 | 성별 | 연령 | 물품 | 속성 합계 | 판정 |
|---|---|---:|---:|---:|---:|---:|---:|---|
| blind_final_a | meadow-1 | 6/12 | 8/12 | 0/8 | 8/8 | 6/8 | 14/24 | FAIL |
| blind_final_a | meadow-2 | 6/12 | 10/12 | 0/8 | 8/8 | 8/8 | 16/24 | FAIL |
| blind_final_a | forest_floor-1 | 6/12 | 8/12 | 0/8 | 8/8 | 6/8 | 14/24 | FAIL |
| blind_final_a | forest_floor-2 | 6/12 | 10/12 | 0/8 | 8/8 | 8/8 | 16/24 | FAIL |
| blind_final_b | meadow-1 | 4/12 | 8/12 | 0/8 | 8/8 | 5/8 | 13/24 | FAIL |
| blind_final_b | meadow-2 | 7/12 | 10/12 | 0/8 | 8/8 | 6/8 | 14/24 | FAIL |
| blind_final_b | forest_floor-1 | 4/12 | 8/12 | 0/8 | 8/8 | 5/8 | 13/24 | FAIL |
| blind_final_b | forest_floor-2 | 7/12 | 10/12 | 0/8 | 8/8 | 6/8 | 14/24 | FAIL |

성별 미구현만으로 속성 점수의 최대값이16/24이므로20/24 통과는 불가능하다. 이를 구현 한계로 유지한다. 아래96개 원문 전체를 그대로 보존했다.

## 증거 식별

- `proto-final-people-meadow-1/capture.png` capture SHA256 `1e64622a1fb226be567f9359bc4cafc3b7a8cdf00f8359ae5fcd33d1ed3300a3`; answer key SHA256 `7b158099be15839858bdeec39a108a3046afe7055d07445d115031473ce26497`.
- `proto-final-people-meadow-2/capture.png` capture SHA256 `0a644b8365854bbbbbc3fbe18ff95bb30aef18c5bf15d5dd67122ad251fce577`; answer key SHA256 `7b158099be15839858bdeec39a108a3046afe7055d07445d115031473ce26497`.
- `proto-final-people-forest_floor-1/capture.png` capture SHA256 `48f41d78ecd675eeda20708305f31b6761622b8792e38df5bb4686fb52ac8096`; answer key SHA256 `7b158099be15839858bdeec39a108a3046afe7055d07445d115031473ce26497`.
- `proto-final-people-forest_floor-2/capture.png` capture SHA256 `dfd5fdf20f23e6caf5111283fb02f8fd1da116e27250316616198f17a8e43aab`; answer key SHA256 `7b158099be15839858bdeec39a108a3046afe7055d07445d115031473ce26497`.

## blind_final_a 원문

원본 `/Users/rexxa/work/arttrial2-data/blind_final_a.json`; SHA256 `49e6e0533836f594b97ef028d90a36b72325af9b05d60aa2c0bab8b55a5fc79c`.

```json
{
  "reviewer": "blind_final_a",
  "method": "Only the four supplied PNGs viewed. No visible number labels; figures indexed left-to-right, top-to-bottom across three rows of four. Sex is not reliably legible from these images. Carrying includes visibly holding a tool.",
  "answers": [
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Upright figure with arms close to the body; no separate load is visible.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 2,
      "activity": "walking",
      "carrying": "no",
      "description": "Upright figure with angled legs and arms slightly apart.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 3,
      "activity": "standing holding an object",
      "carrying": "yes",
      "description": "Upright figure with a narrow object protruding to the right.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "long narrow tool, exact type unclear"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 4,
      "activity": "holding or lifting overhead",
      "carrying": "yes",
      "description": "Bent arms rise toward a dark elongated shape above the head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "dark elongated object"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Body is horizontal with legs extended to the left.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 6,
      "activity": "sitting with arms spread",
      "carrying": "no",
      "description": "Shortened lower body and arms extending sideways suggest sitting.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 7,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Small upright figure with slightly separated legs.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 8,
      "activity": "carrying a load",
      "carrying": "yes",
      "description": "Round colored object is beside the upper body.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "round basket-like object"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal body with head at the right and legs at the left."
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Small upright figure with arms down."
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 11,
      "activity": "holding a tool",
      "carrying": "yes",
      "description": "Upright figure with an object extending diagonally right."
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 12,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Upright figure with legs apart; no clear load."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Upright figure, straight legs, arms beside torso.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 2,
      "activity": "walking",
      "carrying": "no",
      "description": "Crossed or stepping legs and swinging arms suggest walking.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 3,
      "activity": "holding a tool",
      "carrying": "yes",
      "description": "Arm holds a diagonal wooden handle with a pale broad head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "axe-like tool"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 4,
      "activity": "carrying or lifting overhead",
      "carrying": "yes",
      "description": "Both arms raised around a dark oblong object above the head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "dark log-like object"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Whole body is horizontal, head right and legs left; sleep versus injury cannot be determined.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 6,
      "activity": "sitting with arms spread",
      "carrying": "no",
      "description": "Bent or folded lower body with both arms extended sideways.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 7,
      "activity": "walking or standing with legs apart",
      "carrying": "no",
      "description": "Smaller figure with short limbs and splayed legs.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 8,
      "activity": "carrying a basket",
      "carrying": "yes",
      "description": "Round woven-looking container with colored contents held beside shoulder.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "round basket with colored contents"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal extended body; sleep versus injury unclear."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Small figure with arms close to torso and straight legs."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 11,
      "activity": "holding a tool",
      "carrying": "yes",
      "description": "Diagonal wooden shaft with a pale broad head held to the right."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 12,
      "activity": "standing or walking",
      "carrying": "no",
      "description": "Upright figure with one leg angled outward and arms near sides."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Upright figure with arms close to the body; no separate load is visible.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 2,
      "activity": "walking",
      "carrying": "no",
      "description": "Upright figure with angled legs and arms slightly apart.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 3,
      "activity": "standing holding an object",
      "carrying": "yes",
      "description": "Upright figure with a narrow object protruding to the right.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "long narrow tool, exact type unclear"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 4,
      "activity": "holding or lifting overhead",
      "carrying": "yes",
      "description": "Bent arms rise toward a dark elongated shape above the head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "dark elongated object"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Body is horizontal with legs extended to the left.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 6,
      "activity": "sitting with arms spread",
      "carrying": "no",
      "description": "Shortened lower body and arms extending sideways suggest sitting.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 7,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Small upright figure with slightly separated legs.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 8,
      "activity": "carrying a load",
      "carrying": "yes",
      "description": "Round colored object is beside the upper body.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "round basket-like object"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal body with head at the right and legs at the left."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Small upright figure with arms down."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 11,
      "activity": "holding a tool",
      "carrying": "yes",
      "description": "Upright figure with an object extending diagonally right."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 12,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Upright figure with legs apart; no clear load."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Upright figure, straight legs, arms beside torso.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 2,
      "activity": "walking",
      "carrying": "no",
      "description": "Crossed or stepping legs and swinging arms suggest walking.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 3,
      "activity": "holding a tool",
      "carrying": "yes",
      "description": "Arm holds a diagonal wooden handle with a pale broad head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "axe-like tool"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 4,
      "activity": "carrying or lifting overhead",
      "carrying": "yes",
      "description": "Both arms raised around a dark oblong object above the head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "dark log-like object"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Whole body is horizontal, head right and legs left; sleep versus injury cannot be determined.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 6,
      "activity": "sitting with arms spread",
      "carrying": "no",
      "description": "Bent or folded lower body with both arms extended sideways.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 7,
      "activity": "walking or standing with legs apart",
      "carrying": "no",
      "description": "Smaller figure with short limbs and splayed legs.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 8,
      "activity": "carrying a basket",
      "carrying": "yes",
      "description": "Round woven-looking container with colored contents held beside shoulder.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "round basket with colored contents"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal extended body; sleep versus injury unclear."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Small figure with arms close to torso and straight legs."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 11,
      "activity": "holding a tool",
      "carrying": "yes",
      "description": "Diagonal wooden shaft with a pale broad head held to the right."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 12,
      "activity": "standing or walking",
      "carrying": "no",
      "description": "Upright figure with one leg angled outward and arms near sides."
    }
  ]
}
```

## blind_final_b 원문

원본 `/Users/rexxa/work/arttrial2-data/blind_final_b.json`; SHA256 `95aa7858240fb90462cb4ed1b9543a0823b5a2a13dbc41affcb0f185fce6d472`.

```json
{
  "reviewer": "blind_final_b",
  "method": "Image-only review of four supplied PNGs. No numeric labels were legible; figures are indexed row-major, left to right, top to bottom. Sex is unknown because these sprites provide insufficient visual evidence. Age labels are apparent relative body-size impressions.",
  "answers": [
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Upright figure with round brown head and arms low.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 2,
      "activity": "walking or standing with crossed legs",
      "carrying": "no",
      "description": "Upright gray-headed figure; legs appear crossed.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 3,
      "activity": "standing",
      "carrying": "unclear",
      "description": "Small upright figure; right-side protrusion is too small to identify confidently.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "unclear"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 4,
      "activity": "arms raised near head",
      "carrying": "unclear",
      "description": "Dark oblique shape merges with the head and raised arms.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "unclear dark shape"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal figure with head at the right.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 6,
      "activity": "sitting with arms out",
      "carrying": "no",
      "description": "Brown-headed figure has a low broad lower body and extended arms.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 7,
      "activity": "standing",
      "carrying": "no",
      "description": "Smaller upright narrow figure.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 8,
      "activity": "standing holding something near upper body",
      "carrying": "yes",
      "description": "Round dark reddish object beside head or shoulder.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "round dark reddish object"
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal pale figure with head at the right."
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Small upright narrow figure with arms down."
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 11,
      "activity": "standing with one arm out",
      "carrying": "unclear",
      "description": "Brown-headed figure with a small protruding shape at the side."
    },
    {
      "image": "proto-final-people-meadow-1/capture.png",
      "figure": 12,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Upright figure with legs separated."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Brown round-headed figure upright with arms lowered.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 2,
      "activity": "walking",
      "carrying": "no",
      "description": "Arms angled outward and legs crossing in a stepping pose.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 3,
      "activity": "standing holding a tool",
      "carrying": "yes",
      "description": "Right arm extends to a brown handle with a pale gray head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "stone-headed axe or similar tool"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 4,
      "activity": "carrying something above shoulder or head",
      "carrying": "yes",
      "description": "Both arms are raised around a large dark elongated object.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "dark log-like elongated object"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Body is horizontal with straight legs to the left and head to the right.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 6,
      "activity": "sitting with arms spread",
      "carrying": "no",
      "description": "Legs appear folded beneath a broad garment; arms extend outward.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 7,
      "activity": "standing",
      "carrying": "no",
      "description": "Short small-bodied figure with feet apart and arms lowered.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 8,
      "activity": "carrying a round container",
      "carrying": "yes",
      "description": "Round brown-rimmed object with reddish contents held beside the upper body.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "basket or round container"
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal figure with head to the right and legs to the left."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Short upright figure with arms lowered."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 11,
      "activity": "standing holding a tool",
      "carrying": "yes",
      "description": "Brown-headed figure holds a long brown handle with a pale gray head."
    },
    {
      "image": "proto-final-people-meadow-2/capture.png",
      "figure": 12,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Upright figure with one leg angled outward and hands lowered."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Upright figure with round brown head and arms low.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 2,
      "activity": "walking or standing with crossed legs",
      "carrying": "no",
      "description": "Upright gray-headed figure; legs appear crossed.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 3,
      "activity": "standing",
      "carrying": "unclear",
      "description": "Small upright figure; right-side protrusion is too small to identify confidently.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "unclear"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 4,
      "activity": "arms raised near head",
      "carrying": "unclear",
      "description": "Dark oblique shape merges with the head and raised arms.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "unclear dark shape"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal figure with head at the right.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 6,
      "activity": "sitting with arms out",
      "carrying": "no",
      "description": "Brown-headed figure has a low broad lower body and extended arms.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 7,
      "activity": "standing",
      "carrying": "no",
      "description": "Smaller upright narrow figure.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 8,
      "activity": "standing holding something near upper body",
      "carrying": "yes",
      "description": "Round dark reddish object beside head or shoulder.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "round dark reddish object"
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal pale figure with head at the right."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Small upright narrow figure with arms down."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 11,
      "activity": "standing with one arm out",
      "carrying": "unclear",
      "description": "Brown-headed figure with a small protruding shape at the side."
    },
    {
      "image": "proto-final-people-forest_floor-1/capture.png",
      "figure": 12,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Upright figure with legs separated."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 1,
      "activity": "standing",
      "carrying": "no",
      "description": "Brown round-headed figure upright with arms lowered.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 2,
      "activity": "walking",
      "carrying": "no",
      "description": "Arms angled outward and legs crossing in a stepping pose.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 3,
      "activity": "standing holding a tool",
      "carrying": "yes",
      "description": "Right arm extends to a brown handle with a pale gray head.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "stone-headed axe or similar tool"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 4,
      "activity": "carrying something above shoulder or head",
      "carrying": "yes",
      "description": "Both arms are raised around a large dark elongated object.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "dark log-like elongated object"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 5,
      "activity": "lying down",
      "carrying": "no",
      "description": "Body is horizontal with straight legs to the left and head to the right.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 6,
      "activity": "sitting with arms spread",
      "carrying": "no",
      "description": "Legs appear folded beneath a broad garment; arms extend outward.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 7,
      "activity": "standing",
      "carrying": "no",
      "description": "Short small-bodied figure with feet apart and arms lowered.",
      "perceived_sex": "unknown",
      "age_group": "child",
      "held_item": "none visible"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 8,
      "activity": "carrying a round container",
      "carrying": "yes",
      "description": "Round brown-rimmed object with reddish contents held beside the upper body.",
      "perceived_sex": "unknown",
      "age_group": "adult",
      "held_item": "basket or round container"
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 9,
      "activity": "lying down",
      "carrying": "no",
      "description": "Horizontal figure with head to the right and legs to the left."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 10,
      "activity": "standing",
      "carrying": "no",
      "description": "Short upright figure with arms lowered."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 11,
      "activity": "standing holding a tool",
      "carrying": "yes",
      "description": "Brown-headed figure holds a long brown handle with a pale gray head."
    },
    {
      "image": "proto-final-people-forest_floor-2/capture.png",
      "figure": 12,
      "activity": "standing or stepping",
      "carrying": "no",
      "description": "Upright figure with one leg angled outward and hands lowered."
    }
  ]
}
```
