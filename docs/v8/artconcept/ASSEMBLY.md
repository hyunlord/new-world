# 보드 조립 검증

실제 생성 원본으로 12개 보드를 조립했다. RGB PNG만 사용하고 팔레트 양자화·외곽선·블록 축소·리터칭은 하지 않았다. 먼저 각 주제의 짧은 변을 480px로 축소해 2열 4행으로 배치한 다음 PNG 1,500,000바이트를 넘으면 보드 전체의 비율을 유지하며 축소했다. 한국어 라벨은 AppleSDGothicNeo이다.

| 보드 | 최종 크기 | 바이트 |
|---|---|---:|
| `concept_01_earth_and_ash.png` | 764×1154 | 1,332,336 |
| `concept_02_folk_woodcut.png` | 764×1154 | 1,340,428 |
| `concept_03_watercolor_field.png` | 764×1154 | 1,399,796 |
| `concept_04_diorama.png` | 764×1154 | 1,411,967 |
| `concept_05_ink_and_tone.png` | 764×1154 | 1,410,029 |
| `concept_06_cave_ochre.png` | 764×1154 | 1,313,019 |
| `concept_07_storybook_gouache.png` | 764×1154 | 1,247,415 |
| `concept_08_cold_north.png` | 764×1154 | 1,296,980 |
| `overview_hut.png` | 2267×320 | 1,474,339 |
| `overview_people.png` | 2267×320 | 1,426,208 |
| `overview_scene.png` | 618×934 | 1,258,966 |
| `readability.png` | 1280×3392 | 1,003,161 |

검증 당시 art/concepts의 전체 크기는 16,188,257바이트, 보드의 최대 크기는 1,474,339바이트였다. 이 값은 후속 로그 정정으로 수백 바이트 달라질 수 있으며 최종 검사에서 다시 확인한다.

## 재현

```bash
python3 docs/v8/artconcept/build_boards.py --selection /Users/rexxa/work/artconcept-data/selection.json --body-heights /Users/rexxa/work/artconcept-data/body-heights.json --evidence /Users/rexxa/work/artconcept-data/checks-reproduce
```

원본·선택·조립 근거는 외부 데이터 디렉터리에 보존한다. `checks-final/assembly.json`에 79개 원본 SHA와 14개 본체 배율, 입력 상태, 크롭 ID 대응을 기록했다. 재현 시 기존 크롭이 없는 새 evidence 디렉터리를 사용한다. 각 컷아웃의 수동 정수리–발 높이는 480px 미리보기에서 판독해 원본 좌표로 환산한 근사값이다. 출력 본체는 약 56px이고 통나무가 위로 돌출될 수 있다. 원본 픽셀은 변경하지 않았다.

- 실제 명령 실행 완료, PNG 12개 디코딩·바이트 한도·원본 SHA 유지 확인.
- 크롭 14장은 모두 640×360이며, 08의 2장은 미생성/NOT_VERIFIED다.
- 실패한 08 C2를 키잉하면 magenta fraction 0.000, corners 0/4로 명시적 오류가 발생하는 것을 확인했다.
- 05/06 C2의 통나무 4개는 내용 불일치이며 진단용 시험으로 표시했다.
- 실제 보드 12장의 시각 검토는 [BOARD_QA.md](BOARD_QA.md), 조립 검토와 수정 내용은 [ASSEMBLY_REVIEW.md](ASSEMBLY_REVIEW.md).
- 지면 경계와 체크무늬는 생성 원본의 한계이며, 색 보정이나 이음 보정으로 감추지 않았다.
