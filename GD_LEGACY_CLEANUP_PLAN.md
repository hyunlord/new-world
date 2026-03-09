# GDScript 레거시 삭제 계획

기준: `CODEBASE_AUDIT.md` Step 0 감사 + 추가 참조 스캔  
범위: 삭제 실행 없음. 참조 관계와 삭제 순서만 정리.

## 요약

- 시뮬레이션 성격 `.gd` 파일은 여전히 광범위하게 남아 있다.
- 실제 병목은 `.tscn`보다 `scenes/main/main.gd`, `project.godot` autoload, UI/debug preloads다.
- 즉시 삭제 가능한 파일은 사실상 1개뿐이다.

## 즉시 삭제 가능 (참조 없음)

- `scripts/core/simulation/runtime_shadow_reporter.gd`

## 참조 제거 후 삭제

### 1. `scenes/main/main.gd`에서 직접 preload 되는 레거시 시스템

- `scripts/ai/behavior_system.gd`
- `scripts/systems/biology/age_system.gd`
- `scripts/systems/biology/mortality_system.gd`
- `scripts/systems/biology/population_system.gd`
- `scripts/systems/cognition/intelligence_system.gd`
- `scripts/systems/development/childcare_system.gd`
- `scripts/systems/development/child_stress_processor.gd`
- `scripts/systems/development/intergenerational_system.gd`
- `scripts/systems/development/parenting_system.gd`
- `scripts/systems/psychology/contagion_system.gd`
- `scripts/systems/psychology/coping_system.gd`
- `scripts/systems/psychology/emotion_system.gd`
- `scripts/systems/psychology/mental_break_system.gd`
- `scripts/systems/psychology/morale_system.gd`
- `scripts/systems/psychology/needs_system.gd`
- `scripts/systems/psychology/personality_maturation_system.gd`
- `scripts/systems/psychology/stress_system.gd`
- `scripts/systems/psychology/trait_violation_system.gd`
- `scripts/systems/psychology/trauma_scar_system.gd`
- `scripts/systems/psychology/upper_needs_system.gd`
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
- `scripts/systems/social/social_event_system.gd`
- `scripts/systems/social/stratification_monitor.gd`
- `scripts/systems/social/title_system.gd`
- `scripts/systems/work/building_effect_system.gd`
- `scripts/systems/work/construction_system.gd`
- `scripts/systems/work/gathering_system.gd`
- `scripts/systems/work/job_assignment_system.gd`
- `scripts/systems/world/migration_system.gd`
- `scripts/systems/world/movement_system.gd`
- `scripts/systems/world/resource_regen_system.gd`
- `scripts/systems/world/steering_system.gd`
- `scripts/systems/world/tech_discovery_system.gd`
- `scripts/systems/world/tech_maintenance_system.gd`
- `scripts/systems/world/tech_propagation_system.gd`
- `scripts/systems/world/tech_utilization_system.gd`
- `scripts/systems/world/tension_system.gd`

### 2. 다른 레거시 스크립트나 테스트가 참조하는 파일

- `scripts/systems/biology/personality_generator.gd`
- `scripts/systems/cognition/intelligence_generator.gd`
- `scripts/systems/cognition/intelligence_curves.gd`
- `scripts/systems/development/attachment_system.gd`
- `scripts/systems/psychology/personality_maturation.gd`
- `scripts/systems/psychology/trait_system.gd`
- `scripts/systems/psychology/trait_effect_cache.gd`
- `scripts/systems/social/value_system.gd`
- `scripts/systems/social/settlement_culture.gd`
- `scripts/systems/record/chronicle_system.gd`

### 3. 테스트/호환성 경계 때문에 바로 못 지우는 파일

- `tests/test_stage1.gd`는 `job_assignment_system.gd`, `needs_system.gd`, `age_system.gd`, `mortality_system.gd`를 아직 이름으로 참조한다.
- `tests/test_upper_needs.gd`는 `upper_needs_system.gd`를 preload 한다.
- Rust bridge/runtime는 `age_system`, `needs_system`, `job_assignment_system`, `chronicle_system` 같은 legacy system key를 계속 사용한다.

## 보류 (UI 경계 / 혼합 파일)

### `project.godot` autoload 경계

- `scripts/core/simulation/game_config.gd`
- `scripts/core/simulation/simulation_bus.gd`
- `scripts/core/simulation/simulation_bus_v2.gd`
- `scripts/core/simulation/event_logger.gd`
- `scripts/core/entity/deceased_registry.gd`
- `scripts/core/social/name_generator.gd`
- `scripts/core/simulation/sim_bridge.gd`
- `scripts/systems/record/chronicle_system.gd`

### main bootstrap / core runtime 경계

- `scripts/core/simulation/simulation_engine.gd`
- `scripts/core/simulation/simulation_system.gd`
- `scripts/core/entity/entity_manager.gd`
- `scripts/core/settlement/settlement_manager.gd`
- `scripts/core/social/relationship_manager.gd`
- `scripts/core/world/world_data.gd`
- `scripts/core/world/preset_map_generator.gd`
- `scripts/core/world/chunk_index.gd`
- `scripts/core/tech/tech_tree_manager.gd`
- `scripts/core/tech/civ_tech_state.gd`
- `scripts/core/stats/stat_cache.gd`
- `scripts/core/stats/stat_curve.gd`
- `scripts/core/simulation/game_calendar.gd`

### UI/debug 혼합 경계

- `scripts/rendering/snapshot_decoder.gd`
- `scripts/debug/debug_overlay.gd`
- `scripts/debug/debug_data_provider.gd`
- `scripts/debug/panels/event_panel.gd`
- `scripts/debug/panels/ffi_panel.gd`
- `scripts/debug/panels/guard_panel.gd`
- `scripts/debug/panels/perf_panel.gd`
- `scripts/debug/panels/system_panel.gd`
- `scripts/debug/panels/world_panel.gd`

## 삭제 순서

1. `scripts/core/simulation/runtime_shadow_reporter.gd` 같은 무참조 파일부터 삭제
2. `scenes/main/main.gd`의 legacy system preload 제거
3. 테스트 파일의 legacy system preload/name reference 제거
4. `project.godot` autoload에서 old GDScript sim 경로 제거
5. `scripts/systems/`의 독립 레거시 시스템 삭제
6. `scripts/core/`의 mixed runtime 파일을 UI/bridge/core data로 분리한 뒤 삭제

## 핵심 리스크

- `.tscn` 직접 참조는 거의 없지만, `main.gd`가 실질적인 레거시 허브 역할을 한다.
- `project.godot` autoload가 남아 있는 한 core simulation scripts는 조기 삭제가 어렵다.
- `snapshot_decoder.gd`, `game_calendar.gd`, debug overlay 계층은 UI와 섞여 있으므로 삭제보다 분리가 먼저다.
- `parenting_system.gd`와 `attachment_system.gd`는 동적 로딩 관계가 있어 묶어서 제거해야 한다.

## 권장 실행 원칙

- 삭제보다 먼저 preload/autoload 참조를 제거한다.
- Rust 쪽 legacy system key 호환이 끝나기 전까지는 `scripts/systems/` 대량 삭제를 시작하지 않는다.
- UI/debug에서 읽기 전용으로 계속 필요한 파일은 “삭제 후보”가 아니라 “분리 후보”로 본다.
