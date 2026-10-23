# ART-STRUCT 실행 결정

- 기준 `429a18000266483ee40d756840d68b08840313c5` 확인 후 `v8/art-struct` 생성. 목표 원문을 먼저 커밋했다. 다른 작업의 `.gitignore`, `.ignore`, `.omo/`, `docs/art/`는 보존한다.
- 실행 범위는 `art/**`, `docs/v8/artstruct/**`와 외부 원시 증거 `/Users/rexxa/work/artstruct-data/`다. 게임 코드 변경·merge 없음.
- 사용자 제공 Part 0–5는 실행 승인된 계획이다. 추가 계획 승인 절차 없이 구현한다. 벽/언덕, 경계, 투영 검사 모듈은 독립 실행자로 병렬 구현하며 미관 리뷰어는 사용하지 않는다.
- 순서: 투영/검사 → 생성 원본과 재시도 기록 → 벽/경계 수치 시험 → 기존 ENV1 합성기 연결 → 세 장면 여섯 출력과 세 보드 → 실제 Godot 60초 → 한국어 보고/범위 검증/명시 push와 원격 SHA 확인.
- 생성은 built-in imagegen, 최대120회·자산당3회. 원본 무수정 보존. 투명 crop/균일 resize/anchor placement는 선언된 결정론적 정규화이며 국소 픽셀 수선·양자화 금지. 벽 strip은 full-frame resize; seam 실패를 교정 복제로 숨기지 않는다.
- 시작 시각 및 모든 생성은 외부 ledger에 기록한다. 시간 상한/드롭 순서는 원문 그대로 적용한다. 최종 판정은 오너 판단 대기다.
