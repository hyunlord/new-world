# Commit 118 - Chronicle l10n payload 정규화

## 커밋 요약
- `ChronicleSystem.log_event`에서 `l10n` 메타 저장을 정규화해 빈 키/빈 params 저장을 생략.

## 상세 변경
- `scripts/systems/record/chronicle_system.gd`
  - `l10n` 처리 시:
    - `key`를 `String`으로 추출한 뒤 비어있지 않을 때만 `entry["l10n_key"]` 저장.
    - `params`가 비어있지 않을 때만 `entry["l10n_params"]` 저장.
  - 기존 이벤트 본문(`description`) 저장 및 개인 이벤트/월드 이벤트 기록 동작은 유지.

## 기능 영향
- Chronicle 렌더/로딩 동작은 동일 (`evt.get("l10n_params", {})` fallback 유지).
- 빈 로컬라이즈 메타 저장을 줄여 이벤트 엔트리 메모리 사용량 및 세이브 payload를 소폭 축소.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=463.1`, `checksum=13761358.00000`
