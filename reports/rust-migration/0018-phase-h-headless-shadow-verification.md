# 0018 - Phase H headless shadow verification

## Summary
Godot headless smoke 실행 경로를 고정하고, Rust GDExtension 로딩/런타임 클래스 등록을 보강해 실제 shadow 리포트 생성 및 cutover 승인 판정을 통과시켰다.

## Files Changed
- `project.godot`
  - `[gdextension] enabled=PackedStringArray("res://rust/worldsim.gdextension")` 추가
  - 프로젝트 시작 시 Rust extension 로딩을 명시적으로 활성화
- `scripts/core/simulation/sim_bridge.gd`
  - `class_name` 지역 변수명을 `native_class_name`/`runtime_class_name`로 변경 (파서 충돌 회피)
  - `_ensure_gdextension_loaded()` 추가 및 네이티브 브리지/런타임 조회 시 extension 선로딩
- `scripts/core/simulation/simulation_engine.gd`
  - `SimBridge`, `SimulationBus` 직접 참조를 singleton/scene lookup 기반 호출로 보강
  - headless 스크립트 실행 컨텍스트에서도 runtime tick/events/command 경로가 동작하도록 동적 호출 경로 정리
- `tools/rust_shadow_smoke.gd` (신규)
  - headless smoke 실행용 도구 추가
  - autoload 노드 구성 후 `SimulationEngine`를 고정 프레임 실행
  - shadow 리포트 flush + 출력 경로 검증

## API / Signal / Schema Changes
- 공개 API 시그니처 변경 없음
- 신규 운영 도구 추가:
  - `tools/rust_shadow_smoke.gd`

## Verification
- `cd rust && cargo build && cargo build --release` : PASS
- `cd rust && cargo check -p sim-bridge` : PASS
- `Godot --headless --script tools/rust_shadow_smoke.gd` : PASS
  - `SHADOW_REPORT_PATH=/Users/rexxa/Library/Application Support/Godot/app_userdata/WorldSim/reports/rust_shadow/latest.json`
- `python3 tools/rust_shadow_cutover_check.py --report <latest.json>` : PASS
  - `approved_for_cutover=True`
  - `frames=800 mismatch_frames=0 mismatch_ratio=0.000000`
- `Godot --headless --check-only` : FAIL (기존 프로젝트 파싱 이슈)
  - `res://scripts/systems/world/tech_maintenance_system.gd` preload/resolve 오류
  - `res://scripts/ui/hud.gd:787` match pattern 파싱 오류
  - 이번 커밋 변경 범위 외 기존 이슈

## Rust Migration Progress
- Previous: 99% complete / 1% remaining
- Current: 100% complete / 0% remaining
- Delta: +1%

## Notes
- 본 검증으로 Phase H shadow 컷오버 승인 조건(리포트 생성 + 승인 판정)이 충족됐다.
