# WorldSim Rust Migration - Detailed Summary

## 문서 정보
- 작성일: 2026-03-02
- 기준 브랜치: `lead/main`
- 기준 로그: `migration_commit_logs/commit-001.md` ~ `migration_commit_logs/commit-326.md`
- 누적 원로그: `migration_commit_logs/cumulative.md`

## 1) 전체 진행 상태
- Rust 마이그레이션 로그 수: **326개 커밋 단위**
- 코어 전환 축(브리지 대상 기준): **56/56 완료**
- 데이터 로더 축(R-1 core loader set): **9/9 완료**
- 현재 상태: 코어 전환은 종료, 이후는 성능 미세 최적화(Tier-2) 단계

## 2) 핵심 결과 요약
- GDScript hot path 계산 로직을 `rust/crates/sim-systems`로 점진 이관하고, `rust/crates/sim-bridge`의 GDExtension 메서드로 노출.
- 모든 전환은 기본적으로 **Rust-first + GDScript fallback** 구조로 적용되어, 브리지/네이티브 이슈 시에도 기존 동작 유지.
- GPU 옵션 구조는 pathfinding 축에서 `auto/cpu/gpu` 모드 + capability 체크 + fallback으로 정리.
- localization은 `manifest + compile + generated key` 파이프라인으로 전환되어 key-first 운영 체계 확립.

## 3) 아키텍처 변화 상세

### 3-1. Rust 런타임/브리지 레이어
- `sim-systems`:
  - pathfinding, stat curve, body(핵심 수식 집합) 확장
  - 테스트 기반으로 수학/분기 로직 회귀 방지
- `sim-bridge`:
  - Godot 바인딩 `#[func]` 메서드 지속 확장
  - 현재 `#[func]` 메서드 수: **186**
- `sim-systems/body.rs`:
  - `pub fn` 기준 함수 수: **133**

### 3-2. GDScript 실행 경로
- 주요 시스템은 `_SIM_BRIDGE_*` 상수와 bridge 캐시 패턴으로 호출 비용/안정성 균형 유지.
- 공통 패턴:
  1. `_get_sim_bridge()`에서 메서드 존재 확인
  2. Rust 호출 성공 시 결과 사용
  3. 실패 시 기존 GDScript 수식 fallback

### 3-3. GPU/백엔드 옵션 구조
- Pathfinding에 대해 `ComputeBackend`와 브리지 backend 모드 동기화 구조 적용.
- `auto/cpu/gpu` 모드 + capability 확인 + CPU fallback으로 안정 경로 보장.
- GPU 메서드 미지원 빌드에서도 기존 경로 동작 유지.

### 3-4. Localization 파이프라인 전환
- 적용 파일/도구:
  - `tools/localization_audit.py`
  - `tools/localization_compile.py`
  - `tools/data_localization_extract.py`
  - `tools/migration_verify.sh`
  - `localization/manifest.json`
  - `localization/compiled/en.json`, `localization/compiled/ko.json`
- 인라인 다국어 필드 -> `*_key` 기반 접근으로 점진/후방호환 전환 후 strict 감사 기준 충족.

## 4) 단계별 마이그레이션 타임라인

### Phase A - 기반 구축 (Commit 001~010)
- Rust workspace/빌드 안정화
- pathfinding Rust 모듈 및 bridge 구축
- SimBridge autoload + backend 모드 기반 설계
- GPU capability/fallback 기초 경로 확보

### Phase B - 데이터/로케일 전환 (Commit 006, 009, 011~027 중심)
- localization manifest/compile/audit 체계 도입
- data inline locale를 generated key 구조로 전환
- `Locale` 로더를 compiled 우선 구조로 정리

### Phase C - 시스템 수식 대량 이관 (중반부 커밋)
- stat curve/needs/stress/social/cognition/world 계열 수식을 Rust 함수로 대량 이전
- 시스템별로 bridge 메서드 추가 + fallback 유지

### Phase D - 코어 잔여 7->0 종결 (Commit 317~323)
- `ace_tracker`, `childcare`, `population`, `chronicle`, `psychology_coordinator`, `coping`, `emotion` 순서로 마무리
- Commit 323 시점에서 코어 전환 축 **56/56 완료**

### Phase E - 심화(Tier-2) 최적화 (Commit 324~326)
- emotion 내부 반복 수식(half-life/baseline/habituation/contagion) 추가 Rust 이관
- appraisal->8감정 impulse 단건 및 batch 경로 Rust화
- 브리지 호출 횟수 최적화(배치화)

## 5) 도메인별 브리지 적용 현황 (코드 스캔 기준)

### 5-1. 브리지 연동 시스템 파일 수
- `scripts/systems` 내 브리지 연동 파일: **52개**
- 도메인 분포:
  - biology: 4
  - cognition: 3
  - development: 6
  - psychology: 12
  - record: 5
  - social: 12
  - work: 2
  - world: 8

### 5-2. 브리지 연동 시스템 파일 목록
- `scripts/systems/biology/age_system.gd`
- `scripts/systems/biology/mortality_system.gd`
- `scripts/systems/biology/personality_generator.gd`
- `scripts/systems/biology/population_system.gd`
- `scripts/systems/cognition/intelligence_curves.gd`
- `scripts/systems/cognition/intelligence_generator.gd`
- `scripts/systems/cognition/intelligence_system.gd`
- `scripts/systems/development/ace_tracker.gd`
- `scripts/systems/development/attachment_system.gd`
- `scripts/systems/development/child_stress_processor.gd`
- `scripts/systems/development/childcare_system.gd`
- `scripts/systems/development/intergenerational_system.gd`
- `scripts/systems/development/parenting_system.gd`
- `scripts/systems/psychology/contagion_system.gd`
- `scripts/systems/psychology/coping_system.gd`
- `scripts/systems/psychology/emotion_system.gd`
- `scripts/systems/psychology/mental_break_system.gd`
- `scripts/systems/psychology/morale_system.gd`
- `scripts/systems/psychology/needs_system.gd`
- `scripts/systems/psychology/personality_maturation.gd`
- `scripts/systems/psychology/psychology_coordinator.gd`
- `scripts/systems/psychology/stress_system.gd`
- `scripts/systems/psychology/trait_violation_system.gd`
- `scripts/systems/psychology/trauma_scar_system.gd`
- `scripts/systems/psychology/upper_needs_system.gd`
- `scripts/systems/record/chronicle_system.gd`
- `scripts/systems/record/memory_system.gd`
- `scripts/systems/record/stat_sync_system.gd`
- `scripts/systems/record/stat_threshold_system.gd`
- `scripts/systems/record/stats_recorder.gd`
- `scripts/systems/social/economic_tendency_system.gd`
- `scripts/systems/social/family_system.gd`
- `scripts/systems/social/job_satisfaction_system.gd`
- `scripts/systems/social/leader_system.gd`
- `scripts/systems/social/network_system.gd`
- `scripts/systems/social/occupation_system.gd`
- `scripts/systems/social/reputation_system.gd`
- `scripts/systems/social/settlement_culture.gd`
- `scripts/systems/social/social_event_system.gd`
- `scripts/systems/social/stratification_monitor.gd`
- `scripts/systems/social/title_system.gd`
- `scripts/systems/social/value_system.gd`
- `scripts/systems/work/building_effect_system.gd`
- `scripts/systems/work/job_assignment_system.gd`
- `scripts/systems/world/migration_system.gd`
- `scripts/systems/world/movement_system.gd`
- `scripts/systems/world/resource_regen_system.gd`
- `scripts/systems/world/tech_discovery_system.gd`
- `scripts/systems/world/tech_maintenance_system.gd`
- `scripts/systems/world/tech_propagation_system.gd`
- `scripts/systems/world/tech_utilization_system.gd`
- `scripts/systems/world/tension_system.gd`

### 5-3. 코어 보조 레이어(시스템 외)
- `scripts/core/world/pathfinder.gd`
- `scripts/core/stats/stat_curve.gd`
- `scripts/core/simulation/sim_bridge.gd`
- `scripts/core/simulation/compute_backend.gd`

## 6) 최신 검증 상태
- 마지막 검증 커맨드:
  - `cd rust && cargo test -q`
  - `cd rust && cargo run -q -p sim-test`
- 최근 결과:
  - Rust 테스트 전체 통과
  - `sim-test PASS`

## 7) 참고 문서
- 전체 커밋 원문 요약: `migration_commit_logs/cumulative.md`
- 개별 상세 로그: `migration_commit_logs/commit-001.md` ~ `migration_commit_logs/commit-326.md`
- 최신 심화 로그:
  - `migration_commit_logs/commit-324.md`
  - `migration_commit_logs/commit-325.md`
  - `migration_commit_logs/commit-326.md`

