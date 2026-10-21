## 구현 완료 보고 — ART-CONCEPT

**판정: PARTIAL / NOT_COMPLETE.** 8개 컨셉 정의, 114회 생성 기록, 12개 비교 보드와 가능한 입력의 검사는 확보했다. 그러나 카메라 부적합 주제 4개, 내용 불일치 컷아웃 2개, 합성 불가능 컷아웃 1개 때문에 Part 0~5의 원래 동일조건 비교 목표를 전부 달성했다고 보고하지 않는다. 아래는 실패를 포함한 실행 결과다.

### 구현 의도

같은 석기시대 정착지의 사람·건물·지면·장면·초상을 서로 다른 여덟 화풍으로 비교해 오너가 다음 ART-TRIAL2의 방향을 고를 수 있게 했다. 스타일 선정은 오너에게 남긴다. 게임 코드·게임 자산 적용·병합은 이 패스의 결과가 아니다.

### 공통 조건 (카메라, 내용, 분위기 기준, 주제 8개)

지도 주제는 수직 탑다운이며 큰 물체의 약한 앞면 표현만 허용했다. 초상만 정면 3/4 시점이다. 재료는 가죽·나무껍질·통나무·갈대·강돌·이엉·뼈·황토·숯이며 금속과 염색 천을 제외했다. 강가와 숲 가장자리의 작은 개척 캠프, 촘촘함·흙과 돌·화덕 불빛을 공통 기준으로 삼았다.

주제는 S1 열매 바구니를 든 채집 여성, S2 돌도끼와 통나무 3개를 든 남성, S3 지팡이 노인과 아이, S4 지붕 없는 통나무집(침낭 4개·화덕·남쪽 문), S5 쉼터·화덕·장작·열매덤불, S6 강변·숲 바닥·초원·바위의 네 지면, S7 8명이 일하는 전체 캠프, S8 벌목꾼 얼굴·어깨 초상이다. [COMMON_BRIEF.md](../../../art/concepts/COMMON_BRIEF.md)에 원문을 고정했다. 고정 조건의 선언과 실제 생성 충족 여부는 구분했다.

### 컨셉 8개 (슬러그, 한 줄 피치, 참고 이미지 확정까지 시도 수)

| 슬러그 | 한 줄 피치 | S1 확정까지 |
|---|---|---:|
| 01_earth_and_ash | 흙과 재. 어둡고 차분한 유화풍, 화덕만 뜨겁다. | 2회 |
| 02_folk_woodcut | 민화·목판. 굵은 선, 평면 색, 따뜻함. | 1회 |
| 03_watercolor_field | 수채 들판. 맑고 부드럽고 밝다. | 1회 |
| 04_diorama | 디오라마. 작은 모형 세상. | 2회 |
| 05_ink_and_tone | 잉크 선화. 옛 도감 같은 정밀함. | 2회 |
| 06_cave_ochre | 벽화. 황토와 숯의 원시성. | 2회 |
| 07_storybook_gouache | 동화 과슈. 두껍고 둥근 형태. | 2회 |
| 08_cold_north | 북방의 추위. 푸른 그림자, 거친 질감. | 2회 |

컨셉 교체나 시간 부족에 따른 주제 드롭은 없었다. 실패 칸은 성공 이미지로 위장하지 않고 보드에 상태를 표시했다.

### 보드 (경로 목록)

아래 12개 PNG를 실제로 열어 검사했다. 개별 최대 1,474,339바이트이며, 검사 당시 `art/concepts/` 전체 16,188,257바이트로 규격 이내다. 원본을 보정하지 않고 비율을 유지한 다운스케일과 조립을 수행했다. 자세한 육안 결과는 [BOARD_QA.md](BOARD_QA.md)에 있다.

- [concept_01_earth_and_ash.png](../../../art/concepts/boards/concept_01_earth_and_ash.png)
- [concept_02_folk_woodcut.png](../../../art/concepts/boards/concept_02_folk_woodcut.png)
- [concept_03_watercolor_field.png](../../../art/concepts/boards/concept_03_watercolor_field.png)
- [concept_04_diorama.png](../../../art/concepts/boards/concept_04_diorama.png)
- [concept_05_ink_and_tone.png](../../../art/concepts/boards/concept_05_ink_and_tone.png)
- [concept_06_cave_ochre.png](../../../art/concepts/boards/concept_06_cave_ochre.png)
- [concept_07_storybook_gouache.png](../../../art/concepts/boards/concept_07_storybook_gouache.png)
- [concept_08_cold_north.png](../../../art/concepts/boards/concept_08_cold_north.png)
- [overview_people.png](../../../art/concepts/boards/overview_people.png)
- [overview_scene.png](../../../art/concepts/boards/overview_scene.png)
- [overview_hut.png](../../../art/concepts/boards/overview_hut.png)
- [readability.png](../../../art/concepts/boards/readability.png)

### 검사 결과 (6.1 가독성 점수표, 6.2 분위기·시청 순위 원문, 6.3 일관성 위험, 6.4 제작 적합 메모)

**6.1 가독성.** 숲 바닥과 초원 각각 640×360, 타일 64px, 수동 정수리–발 높이를 기준으로 인물 약 56px를 적용했다. 4×4 타일 패치를 반복해 화면을 덮었다. 새 검토자가 이름을 가린 14개 크롭을 원래 크기로 읽었다. 실제 답변 전체는 [BLIND_REVIEW.md](BLIND_REVIEW.md)에 보존했다. 인원 수·두 활동·지면 범주의 세 항목을 각각 1점으로 채점했다. 모든 답변이 2명, 바구니/통나무 운반, 숲 바닥 또는 풀밭을 식별했다. 열매 종류나 도끼의 세부 식별까지 성공했다는 뜻은 아니다.

| 컨셉 | 숲 바닥 /3 | 초원 /3 | 해석 제한 | 일관성 위험 |
|---|---:|---:|---|---|
| 01 | 3 | 3 | 손·소품 세부 및 어두운 배경 분리 한계 | low |
| 02 | 3 | 3 | 바구니 안 붉은 내용물의 정확한 정체는 불명 | low |
| 03 | 3 | 3 | 팔다리 일부가 배경과 섞임 | medium |
| 04 | 3 | 3 | S5/S7 카메라 실패는 별도 유지 | medium |
| 05 | 3 | 3 | C2 통나무 4개, 동일내용 비교 부적합 진단 점수 | medium |
| 06 | 3 | 3 | C2 통나무 4개, 동일내용 비교 부적합 진단 점수 | medium |
| 07 | 3 | 3 | S7 카메라 실패는 별도 유지 | medium |
| 08 | NOT_VERIFIED | NOT_VERIFIED | C2에 마젠타 배경이 없어 두 크롭을 만들지 않음 | low |

**6.2 분위기·시청 순위 원문.** 검토자에게 이름을 가린 S7을 제공했다. 아래 두 줄은 원문이며 각 순위의 개별 이유도 [BLIND_REVIEW.md](BLIND_REVIEW.md)에 그대로 있다.

> H > A > G > E > D > F > C > B

> H > G > C > F > E > B > A > D

첫 줄은 ‘촘촘함, 흙과 돌, 화덕 불빛’, 둘째 줄은 ‘한 시간 동안 지켜보고 싶다’ 순서다. ID 대응은 A=06, B=03, C=08, D=02, E=07, F=01, G=05, H=04다.

| 컨셉 | 분위기 순위 | 시청 선호 순위 | 장면의 조건 한계 |
|---|---:|---:|---|
| 01 | 6 | 4 | 인물이 약 64px보다 큼 |
| 02 | 5 | 8 | 인물이 약 64px보다 큼 |
| 03 | 8 | 6 | 밝은 식생이 불빛보다 우세 |
| 04 | 1 | 1 | REJECTED_CAMERA 진단 장면 |
| 05 | 3 | 2 | 최종 장면 개선, KEPT_SCALE_LIMITATION 유지 |
| 06 | 2 | 7 | REJECTED_CAMERA 진단 장면 |
| 07 | 4 | 5 | REJECTED_CAMERA 진단 장면 |
| 08 | 7 | 3 | 본체 약 58–70px 확인 |

카메라 실패 장면이 섞인 탐색 순위이므로 공정한 동일조건 8개 순위로 확정할 수 없다. 실제 한 시간 플레이나 애니메이션 경험을 측정하지 않았다.

**6.3 일관성.** 리더가 실제 S1/S2/S4 비교에서 매체·광원·선 무게를 판독했다. 01은 유화 붓질과 명도 경계, 02는 굵은 선과 황토색 면, 08은 청회 그림자와 거친 붓질이 유지됐다. 03은 건축이 더 단단하고 세밀해졌고, 04는 모형 조명은 유지되지만 다른 주제의 카메라 재현이 약했다. 05는 건물 해칭과 그림자가 강해졌고, 06은 건물의 사실적 구조가 늘었으며, 07은 건축에서 둥근 단순화가 약해졌다. 표의 low/medium은 이 실제 최종 판독이며 초기 생성 로그의 가설을 대체한다.

**6.4 제작 적합 메모.** 01은 부품 붓질·명도와 반복 붓자국, 02는 관절 외곽선 중복과 판화 결 연결, 03은 반투명 관절 겹침과 종이 밝기, 04는 회전 부품의 조명과 수직 카메라, 05는 얇은 관절선과 지면 해칭, 06은 사지·도구 실루엣과 광물 반점, 07은 몸 비율과 넓은 색면, 08은 부품 그림자와 컷아웃 재현을 해결해야 한다. 각 컨셉의 머리·몸통·팔·다리·도구, 지면 이음, 200개 자산 통제 조건은 [CHECKS.md](CHECKS.md)에 구체화했다. 아직 제작 검증은 아니다.

### 비용 (총 생성 수, 컨셉별 시도·채택, 소요 시간)

총 **114회 생성**, 엄격 채택 **73개(주제 60+컷아웃 13)**, 비채택 41개다. 컨셉별 시도/채택은 01 15/10, 02 13/10, 03 14/10, 04 15/8, 05 14/9, 06 16/8, 07 15/9, 08 12/9이다.

호출별 기록 시간 합 **3899.304초(64.99분)**는 병렬 시간을 더한 값이다. 2026-10-22 02:07:39 KST 집계 마감까지 전체 경과는 01:28 KST 시작 기준 **약 40분**이다. 최종 문서·push까지의 전체 경과는 아직 포함하지 않는다. 금전 청구액은 NOT_MEASURED. 자세한 표와 정의는 [COST.md](COST.md)에 있다.

### 추천 결선 2개와 이유 (선택은 오너)

**잠정 결선은 02 folk_woodcut와 05 ink_and_tone이다.** 02는 다른 컨셉과 명확하게 구분되는 굵은 윤곽과 제한 색면을 유지하며 작은 화면에서 두 활동을 읽혔다. 시청 선호 8위라는 약점은 숨기지 않으며, 장기 관찰의 세부 밀도를 다음 패스에서 보완해야 한다.

05는 개선한 캠프 장면이 분위기 3위·시청 선호 2위였고, 가는 선으로 작업과 재료를 설명하는 방향이 뚜렷하다. 다만 해칭의 축소와 반복 뿌리 무늬, 네 통나무 컷아웃 실패를 해결해야 한다. 04는 두 순위 모두 1위였지만 S5/S7의 카메라 실패 때문에 잠정 결선에서 제외했다. 추천은 생산 승인이나 최종 선택이 아니다.

### 변경된 파일 목록 (커밋별)

- Part 0 [`ff56df32`](https://github.com/hyunlord/new-world/commit/ff56df32): `art/concepts/COMMON_BRIEF.md`, `docs/v8/artconcept/CONCEPT_DECISIONS.md`, `CONCEPT_GOAL.md`, `PROGRESS.md`.
- Part 1 [`18d8211b`](https://github.com/hyunlord/new-world/commit/18d8211b): 01~08의 `art/concepts/<slug>/CONCEPT.md` 8개.
- Part 2 [`6c74d1a8`](https://github.com/hyunlord/new-world/commit/6c74d1a8): 01~08의 `art/concepts/<slug>/LOG.md` 8개, `docs/v8/artconcept/CONCEPT_DECISIONS.md`, `ORIGINALS.json`, `PROGRESS.md`.
- Part 3 [`483b5ab0`](https://github.com/hyunlord/new-world/commit/483b5ab06f35cb89be6d00a00bafc18ac416cf1e): 위 경로 목록의 `art/concepts/boards/` PNG 12개, `docs/v8/artconcept/ASSEMBLY.md`, `ASSEMBLY_REVIEW.md`, `BOARD_QA.md`, `PROGRESS.md`, `build_boards.py`.
- Part 4 [`4e0aeabf`](https://github.com/hyunlord/new-world/commit/4e0aeabf7bfd0564261dd435aa99fa6d508812b5): `CHECKS.md`, `BLIND_REVIEW.md`, `SCORES.json`, `REQUIREMENT_AUDIT.md`, `ASSEMBLY.md`, `ORIGINALS.json`, `PROGRESS.md`, `art/concepts/06_cave_ochre/LOG.md`. 검사·원문·감사 및 06 컷아웃 채택 정정.
- Part 5: **PENDING — 리더의 커밋 이후 실제 SHA·변경 경로를 대입**. 책·비용·본 보고 및 종료 기록.

### 제한사항 (카메라 규칙 위반으로 버린 수, 모델이 못 한 것, 드롭한 컨셉·주제)

- 카메라 위반이 명시된 비채택 시도는 복합 사유 포함 **27개(지도 주제 24개·초상 3개)**다. [COST.md](COST.md)에 파일별 집계를 남겼다. ‘버림’은 채택 제외이며 원본 삭제가 아니다.
- 04 S5/S7, 06 S7, 07 S7은 허용 3회 후에도 카메라 실패다. 최종 보드의 해당 칸은 진단 자료이며 유효한 동일조건 주제로 세지 않는다.
- 05/06 C2는 통나무가 4개, 08 C2는 자연 지면이 남았다. 컷아웃은 각 1회 제한을 지켰고 재시도·통나무 삭제·AI 배경 제거로 고치지 않았다. 08 가독성은 NOT_VERIFIED다.
- 일부 S7의 사람은 약 64px보다 크다. 보드를 동일 크기로 표시한 것이 원본 장면의 동일 축척을 보증하지 않는다.
- 정사각 원본은 요청한 1024²/512²가 아닌 실제 1254², 장면은 1536×1024다. 원본을 수정하지 않았고 규격 편차를 보존했다.
- 03 초원에 밝은 격자 반복, 05 숲 바닥에 반복되는 큰 뿌리가 보인다. 무봉합 타일 성능은 달성·검증하지 않았다.
- 드롭한 컨셉·주제는 없다. 부적합과 누락을 유지했으므로 ‘모든 조건 충족’도 아니다. 새 제작·실험 없이 본 시도 제한 안에서 실패를 성공으로 바꿀 수 없다.
- 게임 실행, 리그 애니메이션, 200개 자산 일관성, 성능·전체 개발 게이트, 게임 통합은 검사 범위에 없다. 문서·PNG 검사는 이 증거들을 대신하지 않는다.

git ls-remote 결과

```text
PENDING: 최종 Part 5 push 후 git ls-remote origin v8/art-concept 결과와
로컬 HEAD 일치를 리더가 실제 출력으로 대체한다. 현재 최종 원격 검증을 주장하지 않는다.
```
