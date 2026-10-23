# ENV1 동물 블라인드 채점 규칙

두 리뷰어에게 정답·종 목록·자세 목록·제작 이력을 주지 않는다. 각자 실제 1× 및 2× 보드의 20개 ID를 보고 종과 자세를 기록한다. A는 1×부터, B는 2×부터 보며 순서 효과를 기록한다. 각 종의 방향 두 개 × 자세 다섯 개가 배율마다 10개다.

채점 기준은 원시 응답을 읽기 전에 고정한다. 종은 deer/roe deer/stag/hart를 사슴, wolf/grey wolf를 늑대로 인정한다. dog, fox, 다른 유제류는 정답으로 대체하지 않는다. 자세는 다음 동의어를 인정한다.

| 원래 자세 | 인정하는 관찰 |
|---|---|
| stand | standing, idle, stationary, 서 있음 |
| walk | walking, stepping, strolling, 걸음 |
| run | running, galloping, sprinting, fleeing, 달림 |
| graze | grazing, feeding with head down, 풀을 먹음 |
| attack | attacking, lunging, pouncing, biting lunge, 돌진 공격 |
| dead | dead, carcass, lying on its side, 옆으로 누움 |

목표 문서의 `dead (lying)`에 따라 옆으로 누운 자세를 인정하되, 잠/휴식과 사망의 의미 혼동은 별도 한계로 남긴다. 단순히 crouching/sitting으로 본 경우는 dead 정답이 아니다. 복수 후보를 답한 경우 첫 번째 명시한 자세로 채점하고 나머지는 원문에 보존한다. 산책/달리기를 모두 제시했다고 양쪽 정답으로 처리하지 않는다.

각 리뷰어 × 각 종 × 각 배율마다 종 ≥9/10, 자세 ≥8/10을 개별 적용한다. 전체 평균으로 실패 집단을 상쇄하지 않는다. 시간은 보드 열람과 답변 사이에 관측한 모델·도구 경과 시간이며 사람 반응 시간으로 해석하지 않는다. 원본 응답은 외부 데이터 폴더에 보존하고 해시와 채점 사본을 보고서에 연결한다.
