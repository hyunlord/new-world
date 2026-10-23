# ART-POLISH 아이템 결과

**판정: FAIL · 오너 판단 대기.** 허용된 한 차례 블라인드 보정을 마쳤지만 수치 스타일과 식별 기준을 통과하지 못했다. 게임 설치·승인을 주장하지 않는다.

## 설계와 생성 범위

`art/style05/ITEMS.md`에 목표의17종 구별 특징을 첫 호출 전에 고정했다. 최초13종을 재설계하고 기존9종을 보존했다. 첫 시험 후 D6의 “오독 종류 한 차례 재생성” 규칙에 따라 기존 berry_basket, log_pile_small, stone_axe도 최종 보정에 포함했다. 최종 선택은 새16종+기존6종이며, 원본·실패·첫 패킷을 모두 보존했다.

총44회 = 아이템37회 + 실제 이미지 모델 점수표7회. 도구 실패0회. ID별1–3회이고 추가 보정은 하지 않는다. 최종16개 선택은 생성 전에 `selection_round1.json`에 고정했으며, 옛 수치 통과작으로 되돌려 의미 보정을 무효화하지 않았다.

## 독립 블라인드 시험

22종을 각각 실제32px/64px로 같은 조용한 초원 위에 표시했다. 패킷에는 무작위 Q번호만 있으며 정답표는 외부에 숨겼다. 라운드마다 새 검토자2명, 최종 라운드는32px 답안을 먼저 저장한 뒤64px를 열었다. 첫 이름만 채점하며 대안·notes로 보충하지 않았다. ID 오타 교정은 별도 기록에 보존했다.

| 라운드 | 검토자 | 32px | 64px | 32px 실패 쌍 |
|---|---|---:|---:|---|
| 0 | A | 9/22 | 14/22 | log/firewood, stone/ground axe, hide/cured hide, raw/fired pot, wrap/cloak garments |
| 0 | B | 11/22 | 15/22 | log/firewood, stone/ground axe, hide/cured hide, raw/fired pot, wrap/cloak garments |
| 1 | A | 9/22 | 11/22 | log/firewood, stone/ground axe, raw/fired pot, wrap/cloak garments |
| 1 | B | 9/22 | 12/22 | stone/ground axe, hide/cured hide, raw/fired pot, wrap/cloak garments |

기준은32px 각17/22와 필수5쌍 모두 양쪽 정답,64px 각20/22다. 최종 FAIL이다. “basket”만으로 berries를 식별했다고 보지 않으며 첫 이름에서 내용물이 명시되지 않으면 BerryBasket 점수를 주지 않는다. 엄격 채점 확정치는 위 표이며 초기 잠정 해석은 외부에 보존했다.

## 스타일와 남은 실패

고정 judge v2 아이템 family를 유지했다. calibration SHA256 `7218ac77979065ec6398c77cb6156bce5bd523979ed48732945f2ab642a5d5fd`. 임계값 재조정 없음. 최종 수치/결합 스타일 통과11/22: 새6/16 + 기존5/6. 실제 모델 점수표는 새16/16 모두 네 축≥4였으나 수치 실패를 덮지 않는다.

| ID | 최종 시도 | 수치 | 실패 지표 |
|---|---:|---|---|
| firewood | 3 | FAIL | wash_flatness |
| ground_axe | 3 | PASS | — |
| spear | ENV1 | FAIL | 기존 spear 실패 보존 |
| cured_hide | 2 | FAIL | edge_density |
| hide_wrap | 2 | FAIL | wash_flatness |
| fur_cloak | 2 | FAIL | wash_flatness |
| wolf_hide | 2 | PASS | — |
| clay | 3 | FAIL | saturation_q25 |
| raw_pot | 2 | PASS | — |
| clay_pot_empty | ENV1 | PASS | — |
| clay_pot_filled | ENV1 | PASS | — |
| meat | ENV1 | PASS | — |
| flint_nodule | 3 | FAIL | saturation_q25, wash_flatness |
| log_pile_small | 1 | PASS | — |
| log_pile_medium | 3 | PASS | — |
| log_pile_large | 3 | PASS | — |
| stone_pile | ENV1 | PASS | — |
| flint_pile | 3 | FAIL | wash_flatness |
| flint_blade | 3 | FAIL | saturation_q25, saturation_q75, wash_flatness |
| stone_axe | 1 | FAIL | saturation_q25, wash_flatness |
| hide | ENV1 | PASS | — |
| berry_basket | 1 | FAIL | wash_flatness |

Spear는 명시된 범위에 따라 ENV1 원본을 유지했다.64px 알파≥128 전체 실루엣 최장거리70.007px, 최종 StoneAxe59.816px/ GroundAxe54.626px. 비율1.170x/1.282x로 요구2x에 미달한다. 창 전체 길이는 자루 길이의 상한이므로 자루 요건도 FAIL이다. 첫 라운드 비율은 `items_length_check.json`의 round0에 남겼다.

## 아티팩트와 검증

- 오너 보드: `art/boards/items_owner.png` —22종 이름과32/64px. 오너 승인 PENDING.
- 경로표: `items_manifest.json`; 모든 시도·전체 프롬프트·참조/도구/원본 해시·시각: `items_generation_ledger.json`.
- 실제 점수표와 원문 전사: `items_model_reviews.json`, `art/style05/polish/items/model*_scorecard*.png`.
- 원본·숨긴 정답·답안·패킷: `/Users/rexxa/work/artpolish-data/items/`의 call_N.json, round0/, round1/.
- `items_integrity.json`:44쌍 도구/원본 해시, 모든 참조, 두 패킷과 소스의 불변성 검사. 새 PNG 최대1,085,460바이트, 모두≤1.5MB.
- 정규화는 기존 ENV1 함수 그대로 알파 경계 crop·비율 유지58px fit·64px 중앙 배치. 픽셀 수술·양자화 없음.
- 5개 회귀 시험 통과: 명시적 최종 선택 우선, 배포된 패킷 덮어쓰기 거부, 보드/정답/소스 변조 검출. basedpyright0errors/0warnings, scoped Ruff와 no-excuse 검사 통과.

M-ART2는 hover 라벨과 inspector에서 항상 아이템 이름을 표시해야 한다. 이 시험은 식민지 화면에서 바로 구분되는지 검증하며, 라벨 존재가 실패를 면제하지 않는다. 최종 설치 후보와 실패 상태를 함께 인계한다.
