# Commit 018 - StatCurve XP 파라미터 PackedArray 캐시 도입

## 커밋 요약
- `StatCurve`의 LOG_DIMINISHING 경로에서 breakpoints/multipliers PackedArray 재생성을 캐시로 대체.
- 동일 파라미터 반복 호출 시 브리지 입력 준비 비용(할당/변환)을 줄임.

## 상세 변경
- `scripts/core/stats/stat_curve.gd`
  - `_XP_CURVE_CACHE_MAX` 상수 및 `_xp_curve_cache` 추가.
  - 신규 헬퍼:
    - `_xp_curve_cache_key(bp, bm)`
    - `_get_xp_curve_meta(params)`
  - `log_xp_required`, `xp_to_level`, `skill_xp_progress`가 위 캐시 메타를 공통 사용.
  - 캐시 용량 상한 도달 시 clear(간단한 bounded 전략).

## 기능 영향
- 계산 결과는 동일.
- XP 커브 관련 빈번한 호출에서 PackedArray 변환/할당 오버헤드 감소.
- Rust 브리지 우선 경로의 호출 준비 비용을 줄여 전체 tick 부하 완화에 기여.

## 검증
- `cd rust && cargo test -q` 통과
