# TRIAL4 결정 기록

## D1 — 별도 실험 브랜치와 범위

목표의 구체 지시를 적용하여 `v8/art-trial4`에서 작업한다. 기준 HEAD는 `7c767aae5b4c5ef2a1b8eb575bffa251dea0263f`. 공통 v8/wip 규칙보다 이번 명시적 실험 브랜치 지시가 우선한다. 변경은 art/** 및 docs/v8/arttrial4/**만 허용한다. 공통 CHANGELOG/ROADMAP/HISTORY/스킬은 범위 밖이므로 이번 기록에 해당 내용을 남긴다. 게임 실행·전체 게임 gate의 통과를 주장하지 않는다.

## D2 — v1 해부학 계약에서 v2 pawn으로

TRIAL3B의 단일 리그 및 마스크 전파 결함은 3D 전체에 대한 부정 증거가 아니다. 오너의 A/B 시각 거부와 팔다리 없는 새 목표에 따라 v2는 head/body/hair/top/item을 사용한다. limb_ok와 손·발 최소 크기/손 앵커는 v2에서 폐기한다. 기존 v1 검사·실패 기록은 역사 재현을 위해 버전 분기로 유지한다.

근거: `docs/v8/arttrial3/TRIAL3B_REPORT.md`, `docs/v8/arttrial3/B_RENDER_LOG.md`, `/Users/rexxa/work/arttrial3-data/b/lab/boards/compare_AB_2x.png`, `/Users/rexxa/work/arttrial3-data/b/part0-amendment/final-a/CHECKS.json`.

목표의 좌표에는 두 모순이 있다. 중심(32,42) 높이20의 머리 하단52와 높이34/하단88 몸통 상단54는 2px 떨어져 있어 직접 4px 겹칠 수 없다. 명시 크기·위치를 유지하고 BODY에 목 연결부 x28..36/y48..58을 두어 몸통과 4px 겹친다. child는 머리 폭 내에 연결부를 유지한다. 이는 별도 팔다리 레이어가 아니다. 수면은 화면 좌표에서 시계방향90°이면 머리가 오른쪽으로 간다. **머리 왼쪽** 결과를 우선하여 반시계방향90°로 동일 레이어/마스크를 회전한다. 원본 목표는 수정하지 않는다.

## D3 — 초기 작업트리 보존 및 UID

기존 `.gitignore` 변경은 *.uid 무시 추가/일부 예외, docs 이미지 무시 제거, graft 캐시 무시 추가다. `.ignore`는 로컬 무시 파일이고 `docs/art/`는 기존 비추적 문서 디렉터리다. 세 항목과 기존 `.omo/`는 이번 작업에서 수정·스테이징하지 않는다. UID는 이미 존재하는 CharacterLab/wash_tint의 값을 보존하고 명시 경로만 `git add -f`한다. scene 헤더에는 고유 UID를 추가한다.

사전 `bash tools/v8/check_docs.sh` 종료코드 **1**: 위 두 tracked UID와 camp/character_lab scene UID 총4개 누락. 원시 로그 `/Users/rexxa/work/arttrial4-data/part0/docs-before.log`.

## D4 — 공유 오버레이와 체형 등록

성인 남성26px/여성24px 몸통에 동일24개 TOP 파일을 사용하면서 각각 상부70%를 덮으려면 체형 등록이 필요하다. 남성26px 기준 idle TOP을 x32 중심으로 실제 몸통 폭에 맞춰 변환한다(여성/노인24, 어린이18). 어린이 HAIR는 머리 폭18/22 비율로 등록한다. 이 변환도 spec 데이터에 두고 checker와 lab이 동일하게 적용한다. 새 의복 조합 파일을 만들거나 좁은 공통 영역만 검사하여 coverage를 완화하지 않는다. bob/lean은 오버레이에 동일 적용하고, sleep은 등록 후 전체 레이어를 함께 회전한다. 랩과 검사기에 픽셀 손수정 경로는 없다.

## D5 — 사전 문서 검사 복구

기존 두 UID 파일을 명시적으로 추적하고 두 scene 헤더 UID를 추가한 뒤 `bash tools/v8/check_docs.sh` 종료코드는 **0**이다. 전후 로그는 `/Users/rexxa/work/arttrial4-data/part0/docs-{before,after}.log`. 이는 문서/UID 검사 결과이며 전체 게임 gate 또는 pawn 구조·시각 합격을 뜻하지 않는다.

## D6 — 시각 검증과 실행 표면

`visual-verdict` 스킬을 적용하여 최초 미리보기와05 인물 기준을 직접 확인한다. 단, 일반 스킬의90점 반복 기준은 이번 목표의 최대2회 spec 수정과 새 리뷰어 기준을 대체하지 않는다. 원시 판독 JSON은 목표의 외부 데이터 규칙에 따라 arttrial4-data에 보관하며 `.omx/`에는 새 상태를 쓰지 않는다. 초기 미완성 후보의 계약 불일치는 spec 고정 전 바로잡고, 첫 완성 후보를 고정한 뒤 변경을 라운드로 계산한다.

Godot 실행 대상은 독립 `art/proto/project.godot`이며 브리지·게임 스크립트를 로드하지 않는다. 이 실험의 랩 렌더/군중 FPS는 실제 창으로 측정하고, 파서 검사나 headless 실행을 화면/FPS 근거로 사용하지 않는다. 게임 성능 또는 시뮬레이션 동일성은 측정 범위 밖이다.

## D7 — 차분한 환경 배경의 실제 경로

현재 `art/style05/env/ground/meadow.png`는 세부 묘사가 많은 원본이고, 기존 차분한 지면은 `art/style05/ground_v2/meadow.png`와 `forest_floor.png`다. 목표의 'calm meadow, not busy forest floor' 조건을 우선하여 이 차분한05 자산을 사용하고 랩 provenance에 원본 경로와 SHA256을 기록한다. 기존 A/B 보드의 배경을 그대로 붙이지 않고, 보존된 캐릭터 레이어를 같은 meadow에 다시 합성한다.

## D8 — 반투명 경계와 의복 가림

상부70%의 실제 BODY alpha255 픽셀은 TOP도255여야 한다. 안티앨리어싱 경계에서 BODY alpha가1..254이면 TOP alpha가 BODY 이상이어야 한다. 사각형 좌표 영역만으로 실제 몸통 밖에도 완전 불투명을 요구하지 않는다. 체형별 실제 alpha로 검사하며, alpha를255로 양자화하는 출력 픽셀 보정은 도입하지 않는다. 이는 경계 정의이며 검사 통과를 위한 coverage 축소가 아니다.

## D9 — 48 조합 표본의 커버리지

기존 TRIAL3 검사기는 조합 전수 열거이며 재사용 가능한48개 sampler 함수는 없다. 따라서 v2는48개의 서로 다른 결정론적 조합을 선택하고 모든7상태·3방향·양성·4피부·각 머리/의복/소품 variant의 포함을 회귀 검사한다. 균등 index 간격은7프레임 주기와 겹쳐 일부 상태만 선택하는 문제가 확인되어 고정 stride17로 수정했다.48개 표본 검사는 전수 조합 검사라고 표기하지 않는다. 전체 레이어 및 모든 프레임의 오버레이 적합 검사는 별도로 유지한다.
