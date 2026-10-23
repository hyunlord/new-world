# ART-POLISH 동물 제작·판독 규약

판정은 오너 판단 대기다. 이전 ENV1 원본·판정은 수정하지 않는다.

- 시트는 TEMPLATE_A1의 2행 × 5열, S/E 순서와 원본 RGBA ownership 분할을 그대로 사용한다. 늑대는 stand/walk/run/attack/dead, 사슴은 stand/walk/run/graze/dead다.
- S는 정면에서 관찰자 왼쪽으로 35° 돌린 front 3/4를 생성 프롬프트에 요구한다. E는 오른쪽 측면 3/4다. ENV1 사슴 S가 정면이므로 이번 패스에서 재생성한다. 결과의 왼쪽 3/4 여부는 육안으로 보고하고, 래스터에서 정확한 카메라 각도를 실측했다고 주장하지 않는다.
- 어깨 높이는 늑대 28 px, 사슴 40 px. S/stand의 실제 원본 어깨·발굽 y 좌표를 수동 기록하고 그 비율 하나를 모든 upright 포즈에 적용한다. dead만 기존 규약대로 몸 길이 56/64 px로 균일 축소한다.
- upright 캔버스 96×80, 발 기준 y72; dead 96×64, 기준 y56. 보이는 alpha≥16, 불투명 alpha≥128, 불투명 8연결 성분 정확히 하나, 경계 접촉 없음, 원본 경계 불투명 픽셀 없음. 허용한 캔버스 밖 손실은 alpha<16만이며 수와 최댓값을 기록한다.
- 프레임을 조립·수정·재색칠·양자화하지 않는다. 분리, 균일 Lanczos 크기 조정, 전체 이미지 배치만 한다. 2× 보드는 nearest-neighbor다.
- D1: 고정된 `docs/v8/artenv1/animal_judge_calibration.json`을 그대로 이용한 native-panel numeric v2와 실제 이미지 모델 점수표를 **보고만** 한다. 스타일 수치로 구조·자세 시험 또는 오너 판단을 대체하지 않는다.
- `art/boards/animals_owner.png`에는 20프레임 전부를 meadow와 forest_floor 위에 각각 1×·2×로 표시한다. 바닥은 합성기의 calm 제한(mean≤0.32, person std ratio≤0.25, saturation≤0.22)을 사용한다. 동물 색은 바꾸지 않는다.
- 각 블라인드 리뷰어는 별도 섞인 20프레임 1×와 20프레임 2× 두 장만 본다. 정답표·원본 파일명·이전 판정은 공개하지 않는다. 종과 자세를 한 개씩 답한다. 종 ≥19/20, 자세 1×≥14/20 및 2×≥16/20을 리뷰어마다 독립 적용한다. 실패하면 한 번의 재생성 라운드와 새로운 두 리뷰어를 사용하며 늑대 3회·사슴 2회 상한을 유지한다.
- 생성/점수표/도구 실패를 포함한 lane 배정은 9회다. 각 호출 전 `/Users/rexxa/work/artpolish-data/animals/generation_ledger.json`에 프롬프트·참조 해시·시각을 예약하고, 완료 후 tool/raw 경로·해시를 기록한다. 분할·보드·독립 텍스트 판독은 이미지 생성 0회다.

재현: `uv run --with pillow --with numpy --with scipy --with scikit-image --with pydantic python -m art.pipeline_polish.animal_process`, 이어 `python -m art.pipeline_polish.animal_boards`를 같은 환경에서 실행한다. 동결된 라운드 보드는 덮어쓰지 않으며 새 라운드 번호로 만든다.
