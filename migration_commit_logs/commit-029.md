# Commit 029 - StatCurve PackedArray 변환 캐시로 호출 오버헤드 절감

## 커밋 요약
- `StatCurve`의 SCURVE/STEP_LINEAR 경로에서 매 호출마다 반복되던 PackedArray 변환을 params 내 캐시로 대체.

## 상세 변경
- `scripts/core/stats/stat_curve.gd`
  - `scurve_speed(...)`
    - `phase_breakpoints/phase_speeds`의 packed 변환 결과를 params 딕셔너리에 캐시:
      - `_phase_breakpoints_packed`
      - `_phase_speeds_packed`
  - `step_linear(...)`
    - `steps` 파싱 결과 packed 배열을 params 딕셔너리에 캐시:
      - `_step_below_thresholds_packed`
      - `_step_multipliers_packed`
  - 이후 동일 params 재사용 시 packed 재생성 없이 Rust bridge 호출 수행.

## 기능 영향
- 스탯 영향 계산 핫패스에서 반복 배열 변환/할당을 줄여 프레임당 CPU 오버헤드 감소.
- 계산 결과/수식 자체는 기존과 동일.

## 검증
- `tools/migration_verify.sh` 통과
  - extraction: `entries=0`, `keys=437`, `preserved=437`
  - strict audit: inline localized fields 0 유지
