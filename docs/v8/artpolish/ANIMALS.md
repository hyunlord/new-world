# ART-POLISH 동물 — 오너 판단 대기

최종 후보는 늑대 try2와 사슴 try2다. ENV1 원본을 덮어쓰지 않았고 게임에 설치하지 않았다. 구조 검사 20/20, 최종 블라인드 기준은 두 리뷰어 모두 PASS다. **공격과 도약의 혼동은 남았으며 오너 채택은 대기**한다. 스타일 수치·점수표는 D1에 따라 보고만 한다.

## 시도와 보존

| 종·시도 | 구조 | native-panel numeric v2 (보고만) | medium / line / light / palette (보고만) | 처리 |
|---|---:|---:|---|---|
| wolf try1 | 10/10 | 10/10 | 4 / 4 / 4 / 5 | 라운드 0 보존 |
| deer try1 | 9/10 | 10/10 | 4 / 4 / 4 / 5 | S/dead 불투명 성분 2개: 본체 737픽셀 + 뿔 끝 1픽셀. 실패 보존 |
| wolf try2 | 10/10 | 10/10 | 5 / 5 / 4 / 5 | 공격 lunge·열린 턱과 옆으로 누운 사체 실루엣 강화, 최종 후보 |
| deer try2 | 10/10 | 10/10 | 5 / 5 / 4 / 5 | 작고 굵게 연결된 뿔 가지를 생성 요청, 최종 후보 |

이미지 호출은 **6회 = 자산 4 + 실제 이미지 모델 점수표 2**, 실패 호출 0회다. 늑대 2/3회, 사슴 2/2회, lane 총량 6/9회다. 남은 배정은 자동 추가 생성 권한으로 사용하지 않는다. 각 호출은 [animal_generation_ledger.json](animal_generation_ledger.json)에 사전 예약, 전체 프롬프트, 입력 참조 해시, tool 원본·raw 해시, 관측 시각이 있다. 호출 내부 스케줄 시각은 알 수 없으며 시작 시각은 준비/예약 경계다. 원본은 `/Users/rexxa/work/artpolish-data/animals/`에 보존했다.

## 방향·기하

S 프롬프트는 **정면에서 관찰자 왼쪽으로 35° 회전한 front 3/4**다. 육안으로 새 시트의 S가 viewer-left 방향이고 E가 오른쪽 측면인 것은 확인했다. 사슴의 이전 head-on S는 이번에 다시 생성했다. 정밀한 카메라 파라미터가 없는 생성 래스터에서 35°를 수치 실측했다는 주장은 하지 않는다. 시트별·포즈별 회전 차이는 오너 보드에서 판단할 잔여 한계다.

| 종·시도 | 원본 S/stand 어깨 y → 발 y | upright 균일 배율 | 목표 어깨 |
|---|---|---:|---:|
| wolf try1 | 247 → 454 | 28/207 | 28 px |
| deer try1 | 250 → 486 | 40/236 | 40 px |
| wolf try2 | 236 → 440 | 28/204 | 28 px |
| deer try2 | 258 → 486 | 40/228 | 40 px |

TEMPLATE_A1의 RGBA ownership 분할, upright 96×80/y72, dead 96×64/y56와 몸 길이 56/64 px를 유지했다. 정규화는 전체 패널의 균일 Lanczos 크기 변경·배치만 한다. 보이는 alpha≥16은 버리지 않았고, 캔버스 밖 alpha<16의 수·최댓값은 프레임 JSON에 남겼다. 픽셀 수술·양자화·재색칠·포즈별 fit 축소는 하지 않았다.

## 블라인드 라운드

| 라운드 | 리뷰어 | 1× 종 / 자세 | 2× 종 / 자세 | 기준 판정 |
|---|---|---|---|---|
| 0 | A | 20/20 · 17/20 | 20/20 · 17/20 | PASS |
| 0 | B | 20/20 · 18/20 | 20/20 · 18/20 | PASS |
| 1 | 새로운 A | 20/20 · 18/20 | 20/20 · 18/20 | PASS |
| 1 | 새로운 B | 20/20 · 18/20 | 20/20 · 18/20 | PASS |

기준은 리뷰어마다 종≥19/20, 자세 1×≥14/20·2×≥16/20이다. [라운드 0 원문·매핑·점수](blind_animals_round0.json)를 보존한다. 양쪽 공격을 웅크림으로 읽은 공통 오류와 A의 E/dead prowling 오독이 있었다. 기준 자체가 실패한 것으로 바꾸지 않았다. 사슴 구조 실패 수정과 함께 명시된 늑대 자세 단서를 개선하는 **한 번의 조정 라운드**를 수행했다. [라운드 1 원문·매핑·점수](blind_animals_round1.json)는 이전 답을 보지 않은 새로운 두 리뷰어의 결과다. 두 리뷰어 모두 두 배율에서 S/E 공격을 `leaping`으로 읽었으며, 사슴과 동일하게 `run`으로 매핑해 **공격 정답으로 구제하지 않았다**. 이 잔여 혼동을 보존하고 6회에서 생성을 종료했다.

`lying on its side`는 ENV1의 자세 판독 규칙에 따라 dead 자세로 매핑하되, 사망인지 수면인지 확신하지 못했다는 원문 한계를 남긴다. 텍스트 판독자는 사람 사용자 표본이 아니며 사람의 자연스러운 인식·오너 채택을 증명하지 않는다.

## D1 근거와 현재 스타일 보고

`docs/v8/artenv1/animal_style_evidence.json`의 고정 numeric 음성 대조군은 painterly/photo를 거절하지 못했다. 한편 `docs/v8/artenv1/animal_negative_model_review.json`의 **추가 이미지 모델 평가는 두 대조군을 거절했다**. 두 동물 계측기가 모두 실패했다고 잘못 단순화하지 않는다. D1은 이 기록과 초상 계측기 실패를 근거로 이번 패스에서 동물·초상 스타일을 공통으로 보고 전용으로 전환한다. 기존 교정 범위·과거 실패는 바꾸지 않았다.

실제 새 점수표는 `art/animal/polish/boards/style_round0.png`, `style_round1.png`다. 전체 원본과 프롬프트는 외부 데이터에 있다. 원본 해시·참조 파일 해시가 원장에 연결되어 있다.

## M-ART2 경로

| 용도 | 경로 |
|---|---|
| 늑대 최종 10프레임 | `art/animal/polish/wolf/try2/{S,E}/{stand,walk,run,attack,dead}.png` |
| 사슴 최종 10프레임 | `art/animal/polish/deer/try2/{S,E}/{stand,walk,run,graze,dead}.png` |
| 각 프레임 구조·출처 | PNG와 같은 이름의 `.json` 및 각 try 디렉터리 `MANIFEST.json` |
| 최종 20프레임 경로·판정 연결 | `docs/v8/artpolish/animal_manifest.json` |
| 늑대·사슴 1차 후보 보존 | 위 경로의 `try1/` |
| 오너 보드 | `art/boards/animals_owner.png` |
| 판독 원본·정답표·응답 | `/Users/rexxa/work/artpolish-data/animals/round0/`, `round1/` |

오너 보드는 모든 20프레임을 meadow·forest_floor 각각의 1×와 2×로 보여 준다. 2×는 nearest-neighbor, 1×는 원래 크기다. 전체 보드를 화면에 맞춰 축소한 미리보기만으로 1× 판독을 대신하지 않는다.

## 구현·검증

추가 파일은 `art/pipeline_polish/animal_process.py`, `animal_boards.py`, `animal_tests.py`, `art/animal/polish/**`, 오너 보드와 이 pass의 animal 기록이다. 기존 ENV1 분할·정규화·judge 함수를 재사용했다. 게임 코드·ENV1 기록은 바꾸지 않았다.

- 보드 기준점·2× nearest 확대·바닥 끝까지 채움 테스트 3개 PASS.
- 세 Python 파일 basedpyright: 오류 0·경고 0, ruff PASS, no-excuse 검사 PASS.
- [animal_integrity.json](animal_integrity.json): 과거·현재 40프레임을 raw에서 재정규화한 RGBA 완전 일치, 6개 tool/raw 해시 쌍·모든 기록 참조·프롬프트 검증, 8개 리뷰어/배율 묶음의 20프레임 완전성, 모든 lane 파일 1,500,000 bytes 이하.
- 원본 이미지와 정적 합성 보드 검증이다. 런타임 애니메이션·게임 화면·사람 판독 시험은 하지 않았다.
