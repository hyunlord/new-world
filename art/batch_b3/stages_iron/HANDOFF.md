# B3 철기 건설·손상 단계 인계

9개 계열의 54개 상태를 모두 판정했다. 신규 23개 채택, A2 재사용 11개, 루트 blueprint 표시 레시피 9개, 해당 없음 4개, 기초 3회 제한 탈락 5개, 반건설 2회 후 제한 검토 탈락 2개다. 원본 생성은 총47회이며, 채택되지 않은 원본은24개다. 보류 상태는 없다.

기초 탈락: bridge·reservoir는 고정 스타일 수치 또는 상태 의미를 함께 통과하지 못했다. aqueduct·forge·town_gate는 원본 지지점/폭을 유지하지 못했으며 마지막3회차도 탈락했다. aqueduct·town_gate 반건설은2회차까지 지지점 위치가 달라져 제외했다. aqueduct2회차는 빈 수로 조건과 달리 파란 내부도 생겼다. 두 반건설 상태의 세 번째 시도는 사용하지 않았다. 수치나 원본 형상 조건을 완화하지 않았다.

rotary_quern·sundial은 이동 가능한 소형 도구로서 foundation/half_built가 해당 없다. built9개와 watchtower/aqueduct damaged2개는 A2 `227529c26307bc3a854eb7bf819e88d02af39046`의 경로와 SHA256을 coverage에 고정했다. 루트가 blueprint 표시 레시피를 담당한다.

자체 검토는 약한6개 상태(전체30개 물리 신규 상태의20%)를 골라 재검토했고 추가 수치/의미 오류도 바로 수정했다. 독립 검토에서 발견한 원본 지지점 오류5개 상태는 위 제한 안에서 수정 시도 후 제외했다. 채택23개는 고정 A2 수치와 자체·독립 시각 검토를 통과했다. 독립 검토의 최초 판정 및 반건설 추가 판정은 `reviews/`에 보존했다. 수치 판정기의 과거 모델 심사는 미제공이므로 모델 심사 통과라고 주장하지 않는다.

- `manifest.json`: 전체47회 원본 판정 및 채택23개 경로·해시·크기·앵커·풋프린트·선택 시도.
- `coverage.json`: family/state/status/source/attempts/reasons 54행, 재사용 소스 커밋과 해시 포함.
- `assets.json`, `art_import.draft.json`, `bindings.draft.json`: 실제 적용하지 않은 통합 초안. 최종 소스 커밋 고정은 루트가 담당한다. lifecycle selector와 building code 연결은 C2/C3 후속 작업이다.
- `boards/*_{1x,2x}.png`: 채택 상태와 원본 built, 기존 성인 스프라이트를 native1x 및 nearest2x로 비교. 명암 배경과 탈락/N/A 명시.
- 외부 증거 `/Users/rexxa/work/art-batch/B3/stages_iron/`: records의 최종 프롬프트/입력, 불변 raw47개, normalized47개, 과거 보드, 최종 보드, 리뷰와 알파128 bbox 기록.

검증: `python3 /Users/rexxa/work/art-batch/B3/stages_iron/verify_final.py` → PASS. 이미지·원본·정규화 SHA256, RGBA 크기·앵커·풋프린트, 채택 수치, 재사용 해시,3회 제한,54개 고유 상태, 보류 없음, draft 필드,18개 보드를 확인했다. `final-check.json`에 결과를 남겼다.

런타임/게임 상태 선택기와 Godot import·play 검증은 아트 전용 범위 밖이므로 실행하지 않았다. 런타임 코드를 변경하지 않았고 sprite 픽셀의 수작업 보정·비균등 확대·gate 완화도 하지 않았다. 저장은 built-in imagegen으로 생성한 뒤 고정 A2 정규화만 적용했다.
