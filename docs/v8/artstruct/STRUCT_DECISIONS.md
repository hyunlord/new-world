# ART-STRUCT 실행 결정

- 기준 `429a18000266483ee40d756840d68b08840313c5` 확인 후 `v8/art-struct` 생성. 목표 원문을 먼저 커밋했다. 다른 작업의 `.gitignore`, `.ignore`, `.omo/`, `docs/art/`는 보존한다.
- 실행 범위는 `art/**`, `docs/v8/artstruct/**`와 외부 원시 증거 `/Users/rexxa/work/artstruct-data/`다. 게임 코드 변경·merge 없음.
- 사용자 제공 Part 0–5는 실행 승인된 계획이다. 추가 계획 승인 절차 없이 구현한다. 벽/언덕, 경계, 투영 검사 모듈은 독립 실행자로 병렬 구현하며 미관 리뷰어는 사용하지 않는다.
- 순서: 투영/검사 → 생성 원본과 재시도 기록 → 벽/경계 수치 시험 → 기존 ENV1 합성기 연결 → 세 장면 여섯 출력과 세 보드 → 실제 Godot 60초 → 한국어 보고/범위 검증/명시 push와 원격 SHA 확인.
- 생성은 built-in imagegen, 최대120회·자산당3회. 원본 무수정 보존. 투명 crop/균일 resize/anchor placement는 선언된 결정론적 정규화이며 국소 픽셀 수선·양자화 금지. 벽 strip은 full-frame resize; seam 실패를 교정 복제로 숨기지 않는다.
- 시작 시각 및 모든 생성은 외부 ledger에 기록한다. 시간 상한/드롭 순서는 원문 그대로 적용한다. 최종 판정은 오너 판단 대기다.

- 생성/판정 분리: judge-v2는 고정 ENV1 방식대로 원본에 적용하고 projection은 정규화 출력에 적용한다. 미관 reviewer 없음으로 model score는MISSING이며 numeric_pass만 보고한다. 오너 승인과 분리한다.
- 선택은 직접 읽기 탈락이 수치보다 우선한다. 승인된 듯한 `passed`를 만들지 않고 `READABLE_NUMERIC_FAIL`·`VISUAL_FAIL`을 보존한다. 침구1/2차와halo/금속문은 수치가 좋아도 재선택하지 않는다.
- 생성 도구의 거절도 시도수에 포함한다. 빈건조대 첫 요청 거절을 포함해3회 상한이다. 로그cap은3회 seamFAIL을 보존하며 수정복제하지 않는다.
- 정규화는 원본alpha crop+단일비율scale+padding(standing), full-frame resize(strip), declared exact component resize다. 침구3차는2×1방향98×14px이며 두타일128px를 채우지 못하는 한계를 유지한다.
- 수치 경계PASS인데 검은 강둑이 계단처럼 남는 실제보드 관찰로 고정선 위치를 seeded0–4px 육지contour로 바꾸고 코너중복darkening을max로 제거했다.10pxfalloff·2pxstroke·24pxrun기준은 그대로다. 강topology 자연스러움은 승인하지 않는다.
- 필수6개 fullframe의밤1×PNG가1.5MB를 초과해 무손실압축만 시도했다. 양자화/화질손실 없이 fullframe은 외부raw에 보존하고repo에는SHA와규정크기boards를 둔다. 조건board는같은worldrectangle의native1×/½×crop이다.
- 실제3개scene에는완료hut이0개다. hut경로는syntheticfixture와3×3bakedFPSroom으로 확인하며 실관측으로 쓰지 않는다. Closeup의집안사람/작업자3명은기존ID의명시적illustrative복제이며canonicaloverlay를변경하지 않는다.
- FPS는별도Godotart프로젝트의실제창60초완료rendercount다. bridge/gamecode를쓰지않으므로게임bridgebuild/fullgamegate를대체하거나실행했다고주장하지않는다. OSscreenshot거절과viewportPNG성공을분리해기록했다.
