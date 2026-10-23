# POLISH Part 0 — 벽과 판 굴 검증

판정: 합성기 구현과 보드 제작 검증 완료, **오너 밝기 선택 대기**.
게임 설치·실행·성능·오너 미술 승인을 의미하지 않는다.

## 넘길 수치와 규칙

- 실행 설정: `art/compositor/wall_params.json`. 기본 `cap_style="dark"`.
- 1× 윗면: 9px 상자 흐림(반지름 4px, 가로 주기 경계 포함), HSV V×0.45, S×0.40.
- 재료별 별도 무늬를 없애기 위해 `stone_cap.png` 한 스트립을 공통으로 사용한다. `palette_05.json`의 log 인덱스 1 `(137,103,63)`, stone 인덱스 23 `(108,110,93)`에서 **색상 H만** 취한다. V/S와 픽셀 무늬는 두 재료가 공유한다. 팔레트 양자화가 아니다.
- 비벽 타일과 닿는 윗면 경계에 2px `#141210`, 알파 0.80. 겹치는 모서리는 한 번만 합성한다.
- 남쪽에 면이 매달릴 때만 윗면 마지막 1px에 V+0.08. 하이라이트는 잉크를 칠하기 전의 윗면 색에서 계산한다.
- 끝 조각은 원본의 윗면 16px를 제외한 면 부분만 사용한다. 옆 타일이 벽이라도 면이 없는 T자 연결부·문이면 현재 면의 끝이다. 윗면과 안쪽 코너에 끝/코너 조각을 얹지 않는다.
- `cap_style="struct"`는 기존 윗면·끝·안쪽 코너 조각을 그대로 재현하는 비교용 경로다.
- 미굴착 언덕의 남쪽이 통행 가능할 때 그 **언덕 타일**만 바위 면을 그린다. 북쪽이 언덕인 굴착 타일에는 면을 추가하지 않는다.
- 굴착 바닥이 이전 언덕 표시보다 우선하도록 입력 순서와 무관하게 처리한다. `floor_dug_rock`의 12px 혼합은 다른 지면과만 수행하고 언덕/벽 경계에서는 수행하지 않는다.

## 실행 및 증거

작업 데이터는 `/Users/rexxa/work/artpolish-data/walls/`에 있다.

| 검사 | 결과 | 증거 |
|---|---|---|
| 최초 신규 시험 | 10 실패 / 1 통과 — 수정 전 결함 확인 | `red.log` |
| 연결된 cap-only 이웃의 면 끝 시험 | 수정 전 1 실패 / 12 통과 | `face_ends_red.log` |
| 전체 합성기 시험 | **76 통과** | `tests.log` |
| ruff 변경 Python 9개 파일 | 통과 | `lint.log` |
| basedpyright, 실제 `/opt/homebrew/bin/python3` 지정 | **0 errors / 10 warnings** | `typecheck.log` |
| PNG 7개 용량 | 모두 1,500,000 bytes 미만 | `WALL_BOARDS.json`, `WALL_VERIFICATION.json` |
| 보드 17개 패널 | 원본 crop의 1× 또는 정수 nearest 2×와 픽셀 바이트 일치 | `WALL_VERIFICATION.json` |
| 코드/설정 SHA | 최종 렌더 이후 일치 | `WALL_BOARDS.json`, `WALL_VERIFICATION.json` |
| 직접 이미지 검토 | 비교판·6개 closeup·3단계 밝기 strip 확인, presentation 92/100 | `walls_visual_verdict.json` |
| `git diff --check` | 통과 | 최종 작업 트리 확인 |

재실행 명령:

```sh
python3 -m pytest art/compositor -q
python3 -m art.compositor.polish_boards
uvx --offline ruff check art/compositor/wall_style.py art/compositor/polish_boards.py art/compositor/test_polish_walls.py art/compositor/walls.py art/compositor/edges.py art/compositor/struct_ground.py art/compositor/struct_layers.py art/compositor/struct_render.py art/compositor/struct_boards.py
uvx --offline basedpyright --pythonpath /opt/homebrew/bin/python3 art/compositor/wall_style.py art/compositor/polish_boards.py art/compositor/test_polish_walls.py art/compositor/walls.py art/compositor/edges.py art/compositor/struct_ground.py art/compositor/struct_layers.py art/compositor/struct_render.py art/compositor/struct_boards.py
```

정적 스킬 검사에서 `struct_layers.py`의 기존 if/elif 분기와 `struct_render.py`의 match 뒤 `assert_never` 배치, 2건이 남는다. 두 건은 기준 커밋 `b6baa73f1f64983fee6aea08ed783a038ba2a4ce`에서도 같은 검사로 재현했다. 범위를 넓히는 기존 코드 재작성은 하지 않았다. 초기 basedpyright 실행은 격리된 uvx의 Python 선택 때문에 의존성을 찾지 못했으며, 실제 실행 Python을 지정한 결과가 위의 최종 기록이다.

## 보드

| 파일 (`art/boards/`) | 내용 | bytes |
|---|---|---:|
| `polish_walls_owner.png` | 같은 상세 crop `(0,140,448,500)`, STRUCT/어두운 cap 각각 1×·2× | 652022 |
| `polish_closeups_2x.png` | 집·창고·판 굴·강/여울·작업장·3×3 오두막, 기존과 동일 crop | 682353 |
| `polish_cap_darkness_strip.png` | V×0.35 / 0.45 / 0.60, 같은 crop 2× | 327567 |
| `polish_walls_owner.struct_1x.png` | STRUCT board 1 전체 crop `(0,140,960,740)` | 680566 |
| `polish_walls_owner.struct_2x.png` | 같은 전체 crop, 정확한 nearest 2× | 870350 |
| `polish_walls_owner.dark_1x.png` | 같은 전체 crop, 어두운 cap | 599086 |
| `polish_walls_owner.dark_2x.png` | 같은 전체 crop, 어두운 cap 2× | 775176 |

전체 장면 네 패널을 한 PNG에 넣은 첫 시도는 `pngcrush -brute` 후에도 1,850,232 bytes였다. 따라서 메인 판에는 같은 상세 crop을 네 번 제시하고, **원래 전체 장면은 1×·2× 모두 별도 판으로 보존**했다. 축소·양자화·손수 픽셀 수정 없이 파일당 상한을 지킨다. 비교 양쪽에 동일한 판 굴 수정이 적용되어 윗면 차이를 비교할 수 있다.

## 변경 파일과 단순화

- 신규: `wall_params.json`, `wall_style.py`, `polish_boards.py`, `test_polish_walls.py`.
- 수정: `walls.py`, `edges.py`, `struct_layers.py`, `struct_ground.py`, `struct_render.py`, `struct_boards.py`, `test_walls.py`.
- `art/boards/`의 위 PNG 7개와 각각의 crop/SHA sidecar, `docs/v8/artpolish/WALL_*` 및 시각 검토 기록.
- 어두운 cap 경로에서 재료별 cap 무늬와 안쪽 코너 덧조각을 제거했다. 기존 장면·보드 조립기를 재사용하여 별도 장면 복제 코드 없이 비교판을 만든다.

## 제한

- 낮의 동결된 ENV1 장면을 오프라인 Python 합성기로 다시 그린 결과다. Godot 또는 M-ART2 런타임에는 설치하지 않았다.
- 집의 잠든 인물, 작업장 인물, 3×3 오두막은 이전 STRUCT와 같은 명시적 시연 fixture다. 실측 인구나 실제 시뮬레이션 사건으로 해석하지 않는다.
- 9px 흐림 후 약한 무늬는 남을 수 있다. 수치 구현 검증과 주인의 미술 선택은 별개이며, 0.35/0.45/0.60 중 어느 것도 채택되었다고 표시하지 않는다.
