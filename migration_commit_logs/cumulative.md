# Rust Migration Commit Cumulative Log

## Commit 001
- Rust 워크스페이스 컴파일/테스트 안정화.
- `sim-systems`에 A* pathfinding 모듈 추가.
- `sim-bridge`에 pathfinding 브리지 API + Godot 클래스 노출(`WorldSimBridge`) 추가.
- `sim-bridge`를 `cdylib`로 빌드 가능하게 구성하고 `.gdextension` 파일 추가.
- 데이터 로더와 enum/serde 테스트 호환성 문제 수정.

## Commit 002
- `Pathfinder`에 Rust 우선 + GDScript fallback 경로탐색 분기 추가.
- `SimBridge` autoload shim 추가 및 `project.godot`에 등록.
- `Locale.ltr()`를 flat dictionary lookup으로 최적화하고 `tech` 카테고리 로딩 추가.

## Commit 003
- Rust bridge에 `pathfind_grid_batch` 계열 배치 API 추가.
- GDScript `Pathfinder`/`SimBridge`에 배치 프록시 경로 추가.
- `MovementSystem`이 재계산 엔티티를 모아 배치 호출하도록 최적화.

## Commit 004
- `ComputeBackend` autoload 추가(`cpu/gpu_auto/gpu_force`).
- `SimBridge`에 GPU 메서드 우선 탐색 경로 추가(없으면 CPU fallback).
- `project.godot` autoload에 `ComputeBackend` 등록.

## Commit 005
- Rust `WorldSimBridge`에 `pathfind_grid_gpu` / `pathfind_grid_gpu_batch` 메서드 추가.
- 현재 구현은 CPU 경로 재사용 fallback으로 안정 동작 보장.

## Commit 006
- `tools/localization_audit.py` 추가.
- en/ko parity, 중복 키, data 내 inline localized field 자동 감사 가능.

## Commit 007
- `sim-systems::stat_curve` 모듈 신규 추가.
- 기존 GDScript stat curve 수학 함수들을 Rust 순수 함수로 1차 이관.
- 단위 테스트 추가로 수학 로직 회귀 방지.

## Commit 008
- Rust `sim-bridge`에 stat curve API 10종 노출.
- `SimBridge`/`StatCurve`를 Rust 우선 + GDScript fallback 구조로 연결.
- 기존 수학 로직 보존하면서 hot path를 점진적으로 Rust 실행 경로로 이전.

## Commit 009
- `localization/manifest.json` + `tools/localization_compile.py` 도입.
- locale 카테고리 파일을 `compiled/<locale>.json` 단일 인덱스로 빌드/운영하는 경로 추가.
- `Locale` 로더를 compiled 우선 + legacy fallback 구조로 개편.

## Commit 010
- `sim-bridge`에 `gpu` feature 플래그와 `has_gpu_pathfinding()` capability API 추가.
- `SimBridge` GPU 선택 로직을 모드 + 네이티브 capability 이중 체크로 강화.
- GPU 미지원 빌드에서 CPU fallback 경로를 더 명시적으로 보장.

## Commit 011
- `tools/data_localization_extract.py`로 data inline 다국어 -> locale key 자동 추출 경로 추가.
- `data_generated` 카테고리를 manifest/컴파일/런타임 로더에 통합.
- `Locale.tr_data()`를 `*_key` 일반 패턴까지 확장해 점진 이관 호환성 강화.

## Commit 012
- `data_localization_extract`에 `--apply-key-fields` 추가.
- 인라인 다국어 포함 data 4개 파일에 `*_key` 참조 자동 주입.
- 인라인 원문을 유지해 점진 이관/하위호환을 동시에 보장.

## Commit 013
- `localization_audit`를 inline->key 전환 커버리지 관점으로 확장.
- `inline_groups_with_key/without_key` 지표로 이관 진행률을 수치화.
- 생성 산출물 스캔 노이즈를 제거해 감사 신뢰도 개선.

## Commit 014
- `StatQuery` XP 계산을 수동 수식에서 `StatCurveScript` 호출로 통합.
- `_compute_level_from_xp`가 Rust-backed `xp_to_level` 경로를 사용하도록 변경.
- breakpoint multiplier 중복 구현 제거로 수식 단일화.

## Commit 015
- Rust `sim-bridge`에 `stat_skill_xp_progress` 단일 호출 API 추가.
- `StatCurve`에 progress 헬퍼를 추가해 Rust 우선 + GDScript fallback 유지.
- `StatQuery.get_skill_xp_info`를 단일 호출 기반으로 전환해 FFI 호출 횟수 최적화.

## Commit 016
- inline localization 감사 지표를 keyable(문자열) / non-keyable(객체/배열 등)로 분리.
- strict 기준을 keyable 누락 중심으로 재정의해 진행률 평가 정확도 향상.
- 현재 keyable 그룹 기준 `*_key` 누락 0 달성.

## Commit 017
- `Locale.tr_data()` deprecated 경고를 1회 출력으로 제한.
- key-first 조회는 유지하면서 반복 호출 시 로그/성능 오버헤드 완화.

## Commit 018
- `StatCurve` LOG_DIMINISHING 경로에 XP 파라미터 PackedArray 캐시 추가.
- 동일 곡선 파라미터 재사용 시 브리지 인자 변환 오버헤드 감소.
- 캐시 상한 기반 단순 bounded 메모리 전략 적용.

## Commit 019
- `tools/migration_verify.sh` 추가로 Rust + localization 파이프라인 검증 절차를 단일 명령으로 통합.
- 선택 옵션 `--apply-key-fields`를 통해 데이터 key 주입까지 포함한 검증 지원.
- 반복 커밋 검증의 일관성/속도 향상.

## Commit 020
- Emotion label/intensity 표시를 `EMO_*` locale key 기반으로 통합.
- `emotion_definition.json`의 object형 inline 다국어 필드 제거.
- localization audit에서 non-keyable inline 그룹 0 달성.

## Commit 021
- Pathfinding batch에 `PackedInt32Array(x,y,...)` 기반 bridge API 추가.
- `Pathfinder`가 int-packed 경로를 우선 사용하도록 연결하고 기존 vec2 경로는 fallback으로 유지.
- batch 좌표 변환/반올림 오버헤드를 줄여 이동 시스템 재계산 경로 성능 개선.

## Commit 022
- Single pathfinding에 `pathfind_grid_xy`/`pathfind_grid_gpu_xy` int-packed API 추가.
- `Pathfinder` single/batch 모두 XY 우선 + vec2 fallback 체계로 정렬.
- 브리지 래퍼 노출/네이티브 미지원 조합에서도 안전 fallback되도록 호출 순서 보강.

## Commit 023
- Rust `sim-bridge`에 pathfinding backend 모드 제어 API(`set/get/resolve_pathfinding_backend`) 추가.
- `SimBridge`가 `ComputeBackend` 모드를 Rust backend로 동기화하고 resolve 결과 기준으로 GPU 선호 판단.
- GPU 미지원 환경은 기존처럼 CPU로 안전 fallback되며, 향후 실제 GPU 경로 구현을 위한 정책 레이어 확보.

## Commit 024
- `tools/data_localization_extract.py`에 `--strip-inline-fields` 옵션 추가(`--apply-key-fields`와 함께 사용).
- key 매핑이 확정된 그룹의 inline 다국어 필드를 자동 제거해 key-first 데이터 구조 정리 자동화.
- `tools/migration_verify.sh`가 신규 옵션을 지원하도록 확장되어 반복 검증/정리 파이프라인 통합.

## Commit 025
- `data_localization_extract` 기본 동작에 기존 `data_generated` 보존(merge) 레이어를 추가.
- inline 제거 이후 재검증 시 `data_generated`가 0으로 리셋되는 문제를 방지해 key-first 운영 안정화.
- 필요 시 `--no-preserve-existing-generated` 옵션으로 강제 재생성 가능.

## Commit 026
- `trait_tooltip`, `trauma_scar_system`, `emotion_data`에서 inline locale 필드 직접 참조를 제거.
- trait/scar 표시는 `*_key` 기반 조회로 통일하고, 미번역 시 원시 키 노출 대신 안전 fallback 적용.
- data inline 제거 작업 전제(코드 경로 의존성 제거)를 위한 선행 정리 완료.

## Commit 027
- `trait_definitions`, `stressor_events`, `emotion_definition`, `species_definition`의 inline 다국어 필드 제거.
- `*_key` + `data_generated` 보존 모드 기반 key-first 데이터 구조로 전환 완료.
- strict localization audit 지표에서 inline localized fields 0 달성.

## Commit 028
- `SimBridge` pathfinding backend 모드 동기화에 last-mode 캐시를 추가해 중복 bridge 호출을 제거.
- 모드가 바뀌지 않는 일반 프레임에서 동기화 call overhead를 줄이고, 모드 변경 시 동작은 유지.

## Commit 029
- `StatCurve`의 `scurve_speed`/`step_linear`에서 PackedArray 변환 결과를 params 내 캐시로 재사용.
- 영향 계산 핫패스의 반복 할당/변환 비용을 줄여 Rust bridge 호출 경로 오버헤드 완화.

## Commit 030
- `StressSystem` 연속 입력 계산(배고픔/에너지/사회 결핍)을 Rust(`sim-systems`) 함수로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 대응 API를 추가해 GDScript는 단일 호출로 결과를 수신.
- stress breakdown 키/의미는 유지하면서 hot path 연산을 네이티브 경로로 전환.

## Commit 031
- `StressSystem`에서 `NEED_HUNGER/ENERGY/SOCIAL` 정규화 조회를 틱당 1회로 통합.
- `_calc_appraisal_scale`와 `_calc_continuous_stressors`가 전달 인자 기반으로 동작하도록 시그니처 정리.
- 수식 동일성은 유지한 채 `StatQuery` 중복 호출을 제거해 CPU 오버헤드를 절감.

## Commit 032
- `StressSystem`의 Lazarus appraisal 스케일 수식을 Rust(`sim-systems`)로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 appraisal API를 추가해 GDScript는 입력 수집 후 단일 호출.
- 수식 의미와 clamp 범위(0.7~1.9)를 유지하면서 stress 계산 hot path를 네이티브화.

## Commit 033
- `StressSystem`의 감정 기여 계산(8감정 가중치 + VA composite)을 Rust 함수로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 emotion contribution API를 추가해 GDScript는 breakdown 조립만 담당.
- stress breakdown 키 체계를 유지한 채 감정 수식 핫패스를 네이티브 경로로 전환.

## Commit 034
- `StressSystem`의 회복(decay) 수식을 Rust(`sim-systems`) 함수로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve` 경유 호출로 GDScript는 입력 수집 + breakdown 처리만 수행.
- recovery 관련 상수를 Rust fallback 함수로 정리해 stress hot path 네이티브 전환 범위를 확장.

## Commit 035
- `StressSystem`의 reserve 갱신 + GAS 단계 전이 로직을 Rust step 함수(`stress_reserve_step`)로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 reserve-step API를 추가해 GDScript는 결과 반영만 담당.
- stress 파이프라인의 핵심 수학 경로(연속 입력/appraisal/emotion/recovery/reserve)를 연속적으로 네이티브화.

## Commit 036
- `StressSystem`의 allostatic load 업데이트를 Rust step 함수(`stress_allostatic_step`)로 이관.
- avoidant attachment 배율은 GDScript에서 산출하고 수치 업데이트는 Rust 경로로 위임.
- stress 파이프라인의 주요 수학 업데이트 경로가 대부분 네이티브 함수 호출 기반으로 정리됨.

## Commit 037
- `StressSystem`의 `stress_state` 판정과 stress→emotion meta 계산을 Rust snapshot 함수로 통합 이관.
- GDScript는 snapshot을 1회 호출해 상태와 meta를 반영하도록 변경되어 중복 계산/호출을 줄임.
- stress 수학 파이프라인이 snapshot/step 중심의 Rust API 구조로 정돈됨.

## Commit 038
- `StressSystem`의 trace decay/keep 처리(`_process_stress_traces`)를 Rust batch step API로 이관.
- packed 배열 기반으로 contribution 합/갱신값/active mask를 계산하고 GDScript는 결과 반영만 수행.
- stress trace 처리 경로까지 네이티브 수학 파이프라인으로 편입.

## Commit 039
- `StressSystem`의 resilience 업데이트 수식을 Rust 함수(`stress_resilience_value`)로 이관.
- HEXACO/지원도/allostatic/피로/scar 보정 입력을 받아 최종 resilience를 네이티브 경로에서 계산.
- stress 수학 업데이트 경로의 remaining hot path를 추가로 네이티브화.

## Commit 040
- `sim-test`에 `--bench-stress-math` CLI 모드를 추가해 stress 수학 Rust 경로를 마이크로벤치 가능하게 확장.
- iteration 옵션(`--iters`)과 `black_box`/`Instant` 기반 측정으로 회귀 비교에 필요한 지표(`elapsed_ms`, `ns_per_iter`, `checksum`)를 출력.
- 기존 기본 실행 경로는 유지하면서 성능 검증용 실행 경로를 분리해 지속 측정 기반 최적화 루프를 지원.

## Commit 041
- `StressSystem`의 최종 delta 계산과 C05 denial redirect 누적 처리 로직을 Rust step 함수(`stress_delta_step`)로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 delta-step API를 추가하고 GDScript는 입력 수집/결과 반영만 수행하도록 정리.
- `sim-test` stress 마이크로벤치에 delta-step 경로를 포함해 stress hot path 네이티브 성능 측정 범위를 확장.

## Commit 042
- stress 틱 후반의 reserve/GAS, allostatic, state snapshot 갱신을 단일 Rust batch step(`stress_post_update_step`)으로 통합.
- `sim-bridge`/`SimBridge`/`StatCurve`에 post-update API를 추가하고 `StressSystem`은 분리 호출 3회를 1회 호출로 치환.
- `sim-test` stress 벤치에 post-update batch step을 포함해 네이티브 핫패스 측정 커버리지를 확장.

## Commit 043
- stress 틱 초반의 Lazarus appraisal 계산과 unmet-needs continuous 입력 계산을 단일 Rust step(`stress_primary_step`)으로 통합.
- `sim-bridge`/`SimBridge`/`StatCurve`에 primary-step API를 추가하고 `StressSystem`의 분리 호출 경로를 결합 호출로 치환.
- stress breakdown 키/동작은 유지하면서 브리지 round-trip을 추가로 축소.

## Commit 044
- stress 틱 중반의 emotion contribution, recovery, delta/denial 계산을 단일 Rust step(`stress_emotion_recovery_delta_step`)으로 통합.
- `sim-bridge`는 Godot 메서드 파라미터 개수 제한을 피하기 위해 PackedFloat32Array/PackedByteArray 인코딩 경로로 노출.
- `StressSystem`은 기존 3단계 호출을 결합 step으로 치환하면서 breakdown 키(`emo_*`, `va_composite`, `recovery`)와 hidden accumulator 의미를 유지.

## Commit 045
- stress 틱의 trace decay 처리와 emotion/recovery/delta 계산을 단일 Rust step(`stress_trace_emotion_recovery_delta_step`)으로 통합.
- `StressSystem`에서 trace 처리 함수와 emotion/recovery/delta 함수의 분리 경로를 제거하고 결합 helper 경로로 치환.
- trace/emotion/recovery breakdown 의미를 유지하면서 브리지 round-trip을 추가로 축소.

## Commit 046
- stress 틱 후반의 post-update(reserve/GAS/allostatic/state)와 resilience 계산을 단일 Rust step(`stress_post_update_resilience_step`)으로 통합.
- `StressSystem`은 결합 step 결과로 reserve/gas/allostatic/resilience를 한 번에 반영하도록 변경되고 기존 `_update_resilience` 함수는 제거됨.
- stress 핵심 수학 파이프라인이 결합 step 중심으로 정리되며 브리지 호출 수가 추가로 축소됨.

## Commit 047
- stress 틱 핵심 수학 경로를 `stress_tick_step` 단일 Rust step으로 통합해 Primary/Trace+Emotion+Recovery+Delta/Post+Resilience를 1회 호출로 수렴.
- `StressSystem`은 단일 step 결과를 반영하는 구조로 재구성되고 기존 내부 helper(`_calc_primary_inputs`, `_calc_trace_emotion_recovery_delta`)는 제거.
- breakdown/trace/hidden accumulator 의미를 유지하면서 stress 파이프라인의 브리지 round-trip을 최소화.

## Commit 048
- `StressSystem.get_work_efficiency`의 Yerkes-Dodson 수식을 Rust 함수(`stress_work_efficiency`)로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 work-efficiency API를 추가하고 GDScript는 네이티브 우선 호출 경로로 전환.
- stress 관련 남은 GDScript 수식 경로를 추가 축소하면서 기존 piecewise/penalty/clamp 의미를 유지.

## Commit 049
- `StressSystem`의 `stress_tick_step` 입력 경로에서 `PackedFloat32Array`/`PackedByteArray`를 엔티티마다 새로 생성하지 않고 scratch buffer 재사용 방식으로 전환.
- trace/scalar/flag 입력 배열을 resize + 인덱스 갱신 패턴으로 채워 GDScript 할당/GC 오버헤드를 줄임.
- stress 수식/상태 업데이트/breakdown 동작 의미는 유지한 채 실행 경로의 메모리 churn을 완화.

## Commit 050
- 이벤트 기반 stress 주입 경로(`inject_stress_event`, `inject_event`)의 최종 스케일 계산을 Rust helper(`stress_event_scaled`)로 공통화.
- `sim-bridge`/`SimBridge`/`StatCurve`에 대응 API를 추가하고 GDScript의 중복 수식 경로를 네이티브 우선 호출로 치환.
- `total_scale`/`loss_mult`/`final_instant`/`final_per_tick` 의미를 유지하면서 이벤트 주입 계산 일관성과 유지보수성을 강화.

## Commit 051
- 이벤트 주입 경로의 `_calc_personality_scale` 곱셈/클램프 수식을 Rust helper(`stress_personality_scale`)로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 personality-scale API를 추가해 GDScript는 입력 수집 후 네이티브 호출 중심으로 전환.
- HEXACO 편차 방향(`high_amplifies`)과 trait multiplier 의미를 유지한 채 성격 스케일 계산 경로를 일원화.

## Commit 052
- 이벤트 주입 경로의 `_calc_relationship_scale`, `_calc_context_scale` 수식을 Rust helper(`stress_relationship_scale`, `stress_context_scale`)로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 관계/상황 scale API를 추가하고 `StressSystem`은 입력 구성 후 네이티브 호출 중심으로 전환.
- method 처리(`none`, `bond_strength`, unknown) 및 context multiplier clamp 범위를 유지하면서 이벤트 스케일링 계산 경로를 통일.

## Commit 053
- 이벤트 감정 주입 수식(`fast`/`slow` layer clamp + scale 적용)을 Rust step(`stress_emotion_inject_step`)으로 이관.
- stressor 로드 시 `emotion_inject`를 `_emo_fast`/`_emo_slow` 배열로 사전 컴파일해 이벤트 실행 시 문자열 파싱 비용을 축소.
- `StressSystem` 감정 주입 경로를 사전 컴파일 + 네이티브 step 호출 기반으로 정리하면서 기존 레이어 경계값 의미를 유지.

## Commit 054
- `stressor_events` 로드 시 personality modifier를 `_p_specs`/`_p_traits`로 사전 컴파일해 `inject_event` 런타임 파싱 비용을 축소.
- `_calc_personality_scale` 입력을 원본 Dictionary 대신 사전 컴파일된 spec 기반으로 전환해 이벤트 경로 분기/문자열 처리 오버헤드를 완화.
- 성격 스케일 계산은 기존 Rust helper(`stress_personality_scale`) 호출 구조를 유지해 수식 의미 일관성을 보존.

## Commit 055
- `stressor_events` 로드 시 relationship/context modifier를 `_r_*`/`_c_*` 필드로 사전 컴파일해 `inject_event` 런타임 dictionary 해석 비용을 축소.
- `_calc_relationship_scale`, `_calc_context_scale` 입력을 사전 컴파일 스키마 기반으로 전환해 이벤트 경로 분기/순회 오버헤드를 완화.
- 관계/상황 배수 계산은 기존 Rust helper 호출(`stress_relationship_scale`, `stress_context_scale`)을 유지해 수식 의미를 보존.

## Commit 056
- `_calc_personality_scale`의 trait multiplier 적용 경로를 trait id 맵 기반 조회로 전환해 반복 선형 탐색 비용을 축소.
- personality spec에 axis stat 문자열(`axis_stat`)을 사전 컴파일해 런타임 문자열 결합 비용을 완화.
- 성격 스케일 계산 수식 및 Rust helper 호출 구조는 유지해 동작 의미를 보존.

## Commit 057
- rebound queue 처리 루프(`_process_rebound_queue`)를 Rust batch step(`stress_rebound_queue_step`) 호출 중심으로 이관.
- `sim-bridge`/`SimBridge`/`StatCurve`에 rebound queue API를 추가해 GDScript는 배열 입력 구성/결과 반영만 담당.
- 현재 `REBOUND_DECAY_PER_TICK=0.0` 설정에서는 기존 만료/합산 의미를 유지하면서 런타임 순회 계산 경로를 네이티브화.

## Commit 058
- 이벤트 감정 주입 경로(`_inject_emotions`)에서 current fast/slow `PackedFloat32Array`를 매 호출 신규 생성하지 않고 scratch buffer 재사용 방식으로 전환.
- `stress_emotion_inject_step` 호출 구조와 결과 반영 의미를 유지한 채 GDScript 임시 할당/GC 오버헤드를 완화.

## Commit 059
- `inject_event`의 relationship/context/event scaling 계산을 단일 Rust step(`stress_event_scale_step`)으로 결합해 bridge round-trip을 축소.
- `StressSystem`은 context 활성 multiplier 수집만 수행하고 스케일 계산은 결합 step 결과를 사용하도록 재구성.
- 분리 helper(`_calc_relationship_scale`, `_calc_context_scale`)를 정리하면서 기존 최종 수식 의미를 유지.

## Commit 060
- `inject_event`의 scale 계산과 emotion layer 반영을 단일 Rust step(`stress_event_inject_step`)으로 결합해 이벤트 경로 bridge 호출을 추가로 축소.
- `StressSystem`은 현재 emotion layer snapshot 수집 후 결합 step 결과를 반영하는 구조로 재구성되고 기존 `_inject_emotions` 경로는 제거.
- 기존 최종 stress/trace/emotion 반영 의미를 유지하면서 이벤트 주입 핫패스의 GDScript 조립 비용을 완화.

## Commit 061
- `inject_event` 입력 수집(personality/context)에서 반복 생성되던 PackedArray를 scratch buffer 재사용(`resize(0)`) 방식으로 전환.
- `_calc_personality_scale`, `_collect_active_context_multipliers` 경로의 임시 할당을 줄여 이벤트 주입 핫패스의 GDScript GC 압력을 완화.

## Commit 062
- stressor personality trait modifier를 `_p_trait_ids`/`_p_trait_multipliers` Packed 배열로 사전컴파일해 런타임 dictionary 해석 비용을 축소.
- `_calc_personality_scale`를 trait 배열 인덱스 순회 기반으로 전환하고 trait id map도 scratch dictionary 재사용으로 최적화.
- 성격 스케일 수식 및 Rust helper 호출 구조는 유지해 동작 의미를 보존.

## Commit 063
- stressor personality spec(`axis/facet/weight/direction`)를 `_p_spec_*` Packed 배열로 사전컴파일해 `Array[Dictionary]` 해석 비용을 축소.
- `_calc_personality_scale`를 spec 배열 인덱스 순회 기반으로 전환해 이벤트 경로 타입 캐스팅/분기 오버헤드를 완화.
- 성격 스케일 수식 의미와 Rust helper 호출 구조는 유지.

## Commit 064
- `emotion_inject`가 실질적으로 없는 stressor에 대해 `_emo_has_values` 플래그를 도입하고 `inject_event`에서 scale-only fast path를 사용하도록 분기.
- 감정 주입이 없는 이벤트에서는 emotion snapshot/결합 step 호출을 건너뛰어 이벤트 주입 경로의 불필요한 오버헤드를 절감.

## Commit 065
- relationship method 전달을 문자열에서 method code(enum) 기반으로 확장(`*_step_code`)해 브리지 경계 문자열 오버헤드를 완화.
- `inject_event`는 `_r_method_code`를 사용해 code 기반 event scale/inject step을 호출하도록 전환.
- string 기반 기존 API는 유지해 호환성을 보존하고 신규 code 경로는 동등성 테스트로 검증.

## Commit 066
- `stress_tick_step` 결과를 packed 출력(`scalars`/`ints`)으로 수신하는 경로를 추가하고 `StressSystem`은 인덱스 기반 해석으로 전환.
- tick 핫패스의 dictionary key 조회를 줄이기 위해 state/meta 반영 경로를 packed 값 직접 적용 방식으로 재구성.
- 기존 dictionary 반환 경로는 유지하면서 `StatCurve` fallback에서 packed 변환을 제공해 호환성을 유지.

## Commit 067
- stress tick breakdown의 감정 키를 `_EMOTION_BREAKDOWN_KEYS` 상수 배열로 고정해 루프 내 문자열 포맷팅 비용을 제거.
- breakdown 키 체계(`emo_*`) 의미는 유지하면서 미세 오버헤드를 완화.

## Commit 068
- stress trace breakdown 키(`trace_<source_id>`)를 trace 데이터(`breakdown_key`)에 캐시해 tick 루프 문자열 포맷팅을 줄임.
- 기존 trace 데이터에는 키가 없을 수 있으므로 처리 중 1회 생성/저장하는 보정 경로를 추가해 호환성을 유지.

## Commit 069
- `DEBUG_STRESS_LOG` 비활성 시 stress tick의 breakdown 조립 경로(continuous/trace/emotion/recovery)를 건너뛰도록 최적화.
- trace breakdown 키 생성(`trace_<source_id>`)을 debug 활성 케이스로 제한해 문자열 처리 오버헤드를 완화.
- debug off 상태에서는 `ed.stress_breakdown`이 기존 값이 있을 때만 clear해 불필요한 tick별 dictionary 재할당을 줄임.

## Commit 070
- `DEBUG_STRESS_LOG` 비활성 틱에서 `breakdown` 빈 Dictionary 초기화를 생략하도록 초기화 시점을 조건부로 변경.
- debug off 틱에서 `_debug_log` 함수 호출 자체를 건너뛰어 호출 오버헤드를 제거.
- stress 계산/상태 업데이트 의미는 유지하면서 디버그 비활성 운영 경로의 미세 비용을 완화.

## Commit 071
- `execute_tick`에서 `DEBUG_STRESS_LOG`를 tick당 1회 계산하고 `_update_entity_stress`로 전달하도록 변경.
- `_update_entity_stress` 내부의 반복 `GameConfig.DEBUG_STRESS_LOG` 조회를 제거해 엔티티 루프 미세 오버헤드를 완화.
- stress breakdown/로그 출력 의미는 유지하면서 디버그 플래그 조회 경로를 단순화.

## Commit 072
- stress trace 유지 필터링 경로를 `next_traces` 신규 배열 append 방식에서 in-place compact(`write_idx`) 방식으로 전환.
- trace 갱신 후 `resize(write_idx)`로 비활성 tail을 제거해 tick당 trace 배열 재할당/복사 비용을 완화.
- trace 업데이트/활성 판정 의미는 유지하면서 stress tick 메모리 churn을 줄임.

## Commit 073
- localization compile 산출물에 stable key index(`keys`)와 `meta.key_count`를 추가해 key 집합 추적을 구조화.
- `Locale`에 key↔id 조회 API(`has_key`, `key_id`, `ltr_id`)를 추가하고 compiled `keys` 기준 인덱스를 런타임에 구성.
- 기존 `ltr()` 호환성을 유지하면서 Rust/GDExtension 경로의 정수 key id 기반 조회 확장을 위한 기반을 마련.

## Commit 074
- `GameCalendar`의 age 단위 텍스트(`UI_AGE_YEARS/MONTHS/DAYS`) 조회를 key-id 캐시 + `Locale.ltr_id()` 경로로 전환.
- 정적 key id 캐시를 도입하고 최초 1회 `Locale.key_id(...)` 해석 후 재사용하도록 구성.
- 결과 문자열 의미는 유지하면서 반복 포맷 경로의 locale key 문자열 조회 오버헤드를 완화.

## Commit 075
- `Locale`에 `MONTH_1..12` key id 캐시(`_month_key_ids`)를 추가하고 locale 로드 시 선계산.
- `get_month_name`이 `ltr_id` 경로를 우선 사용해 월 이름 반복 조회의 문자열 key lookup 비용을 줄임.
- key id가 없을 때는 기존 `ltr("MONTH_n")` fallback을 유지해 동작 호환성을 보장.

## Commit 076
- `EmotionData.get_intensity_label`에 key id 정적 캐시를 추가해 강도 라벨 키 해석을 1회 후 재사용.
- 라벨 조회 시 `Locale.ltr_id` 경로를 우선 사용하고, 미지원 상황은 기존 `Locale.ltr` fallback으로 호환 유지.
- 감정 강도 라벨 반복 조회의 locale key 문자열 lookup 비용을 완화.

## Commit 077
- `Locale.tr_id`에 조합 키 기반 key-id 캐시(`_tr_id_key_id_cache`)를 추가해 반복 key 해석을 1회로 축소.
- `tr_id`는 `ltr_id` 경로를 우선 사용하고, 미지원 시 기존 `ltr` + raw `id` fallback 의미를 그대로 유지.
- locale reload 시 캐시를 초기화해 locale 전환 이후에도 안전하게 동작하도록 보강.

## Commit 078
- `Locale.trf`에 포맷 키 기반 key-id 캐시(`_trf_key_id_cache`)를 추가해 반복 key 해석 비용을 축소.
- `trf`는 `ltr_id` 경로를 우선 사용하고, 미지원 상황은 기존 `ltr` fallback을 유지해 호환성을 보장.
- placeholder 치환 로직은 변경하지 않고 조회 경로만 최적화.

## Commit 079
- localization compiler에 `include_sources` 옵션을 도입하고 기본값을 `false`로 설정해 compiled 산출물에서 `sources`를 기본 제외.
- compiled meta에 `include_sources` 상태를 기록하고 필요 시에만 `sources`를 포함하도록 분기.
- 런타임 미사용 데이터 제거로 compiled 파일 크기를 크게 축소해 locale 로드 I/O/파싱 비용을 완화.

## Commit 080
- `Locale.tr_id`에 최종 결과 캐시(`_tr_id_result_cache`)를 추가해 동일 `(prefix,id)` 조회를 즉시 반환하도록 최적화.
- miss 시 기존 key-id 변환 경로로 계산한 결과(번역/미번역 fallback)를 캐시에 저장해 반복 miss 비용을 줄임.
- locale reload 시 결과 캐시를 초기화해 locale 전환 이후 정합성을 유지.

## Commit 081
- localization compile 결과를 locale별 독립 key 목록이 아닌 지원 locale union 기반 canonical key index로 고정.
- locale별 누락 키는 `en` fallback(또는 key 자체)으로 채워 모든 locale 산출물의 key 집합/순서를 일치시킴.
- locale 전환 이후에도 key-id 캐시가 동일 인덱스 의미를 유지하도록 안정성을 강화.

## Commit 082
- `StatQuery`에 `get_normalized_batch`를 추가해 다수 stat 정규화 조회를 단일 캐시 경로로 처리.
- `StressSystem`은 NEED/HEXACO 정규화 입력을 batch 호출 1회로 수집하도록 전환해 입력 준비 오버헤드를 완화.
- stress 계산 수식/결과 의미는 유지하면서 tick 핫패스의 중복 호출을 줄임.

## Commit 083
- `StatQuery`에 정규화 range 캐시(`_normalized_range_cache`)를 도입해 `get_range` 반복 호출을 축소.
- `get_normalized`/`get_normalized_batch`가 캐시된 range를 공통으로 사용하도록 정리.
- 정규화 결과 의미는 유지하면서 stat range 조회 경로의 반복 비용을 완화.

## Commit 084
- `StatQuery`에 output buffer 재사용 API(`get_normalized_batch_into`)를 추가해 batch 결과 배열 할당을 줄임.
- 기존 `get_normalized_batch`는 새 API를 호출하는 호환 래퍼로 유지.
- `StressSystem`이 scratch norm buffer를 재사용하도록 전환해 엔티티 루프 임시 할당/GC 부담을 완화.

## Commit 085
- `Locale`에 key index 버전(`key_index_version`)을 도입해 locale 로드 시 캐시 무효화 기준을 제공.
- `GameCalendar`와 `EmotionData`의 정적 key-id 캐시가 버전 변경을 감지해 자동 초기화되도록 보강.
- locale 전환/재로딩 이후에도 key-id 캐시 정합성을 유지하는 안전장치를 추가.

## Commit 086
- `StatQuery.get_normalized_batch_into`에 `assume_defined` fast-path 옵션을 추가해 반복 `has_def` 체크를 선택적으로 생략.
- `StressSystem`의 고정 stat id batch 조회는 `assume_defined=true`로 호출해 입력 수집 분기 비용을 완화.
- 기본 경로는 기존 안전 동작(`assume_defined=false`)을 유지해 호환성을 보장.

## Commit 087
- `ListPanel` entity row 생성 시 `job_display`를 선계산해 draw 루프의 `Locale.tr_id` 반복 호출을 제거.
- building list에 타입별 번역 캐시(`building_type_cache`)와 built 라벨 1회 조회를 적용해 루프 내 locale 조회를 축소.
- UI 표시 문자열 의미는 유지하면서 패널 렌더 핫루프의 번역 조회 오버헤드를 완화.

## Commit 088
- `EntityDetailPanel`의 needs/personality/derived 구간을 `StatQuery.get_normalized_batch_into` 기반으로 전환.
- 기본/상위 욕구, HEXACO 축, 파생 스탯 조회를 scratch buffer 재사용 batch 호출로 수집해 draw 루프의 개별 정규화 호출을 축소.
- 패널 표시 의미는 유지하면서 UI 상세 패널 렌더 경로의 stat 조회 오버헤드를 완화.

## Commit 089
- `ListPanel`에 locale 라벨 캐시(`tabs/columns/sort/deceased`)를 도입하고 locale 변경 시점에만 갱신하도록 구조화.
- draw 루프에서 정적 라벨의 `Locale.ltr` 호출을 캐시 문자열 사용으로 치환해 반복 조회 비용을 줄임.
- 탭/헤더/정렬 표시 동작은 유지하면서 UI 목록 패널 렌더 경로를 경량화.

## Commit 090
- `HUD` 엔티티 패널 needs 갱신을 `StatQuery.get_normalized_batch_into` 1회 호출 + scratch buffer 재사용 구조로 전환.
- `NEED_HUNGER/ENERGY/SOCIAL/THIRST/WARMTH/SAFETY` 개별 조회를 버퍼 인덱스 참조로 치환해 `StatQuery` 반복 호출을 축소.
- 저배고픔 blink 판정은 추가 조회 없이 같은 batch 결과(`hunger_norm`)를 재사용해 HUD tick 핫패스를 경량화.

## Commit 091
- `HUD._process()`에서 건물 카운트(`built/wip`)와 stockpile 자원 합계를 동일 루프에서 함께 계산하도록 통합.
- `get_all_buildings()` 결과를 1회 순회해 `UI_BLD_*`와 `UI_RES_*` 라벨 갱신 입력값을 동시에 수집.
- 중복 순회 helper(`_get_stockpile_totals`)를 제거해 HUD 프레임 업데이트 경로를 경량화.

## Commit 092
- `BuildingManager`에 `get_building(id)` 직접 조회 API를 추가해 건물 ID 조회를 dictionary lookup 경로로 노출.
- `HUD._get_building_by_id`를 전체 배열 선형 탐색에서 `get_building` 직접 호출로 전환.
- 선택 건물 패널 갱신 시 불필요한 건물 전체 순회를 제거해 조회 경로를 경량화.

## Commit 093
- `Locale`에 `trf1`/`trf2` 경량 포맷 API를 추가해 1~2 placeholder 치환에서 임시 params Dictionary 생성을 줄이는 경로를 도입.
- `HUD`의 프레임 루프 포맷 호출(`UI_POP_FMT`, `UI_BLD_*`, `UI_RES_*`, `UI_POS_FMT`, `UI_ENTITY_STATS_FMT`)을 `trf1`/`trf2`로 치환.
- 표시 의미를 유지하면서 HUD locale 포맷 핫패스의 호출 오버헤드를 완화.

## Commit 094
- `ListPanel`의 deceased status 포맷(`UI_DECEASED_STATUS_FMT`)과 footer count 포맷(`UI_ENTITIES_COUNT_FMT`)을 `Locale.trf1` 경로로 전환.
- 기존 placeholder/출력 의미는 유지하면서 draw 경로의 임시 params Dictionary 생성을 줄임.
- 리스트 패널 포맷 호출 핫패스의 미세 오버헤드를 완화.

## Commit 095
- `ChroniclePanel` draw 경로의 이벤트 개수(`UI_EVENTS_COUNT`) 포맷을 `Locale.trf1`로 전환.
- 짧은 날짜 fallback(`UI_SHORT_DATE`) 포맷을 `Locale.trf2`로 전환.
- 표시 의미를 유지하면서 Chronicle 렌더 경로의 고정 포맷 호출 오버헤드를 완화.

## Commit 096
- `HUD._update_building_panel`의 `UI_UNDER_CONSTRUCTION_FMT`, `UI_BUILDING_WIP_FMT` 포맷을 `Locale.trf1`로 전환.
- `HUD._on_speed_changed`의 `UI_SPEED_MULT_FMT` 포맷을 `Locale.trf1`로 전환.
- 표시 의미는 유지하면서 HUD 건물 상태/속도 라벨 갱신 경로의 임시 params Dictionary 생성을 줄임.

## Commit 097
- `world_stats_population_tab`에서 `UI_STAT_POP_FMT` 결과를 `total_pop_text`/`s_pop_text`로 1회 생성 후 폭 계산/행 문자열 조립에 재사용.
- `UI_STAT_GENDER_FMT` 호출을 `Locale.trf2` 경량 경로로 전환.
- draw 루프의 중복 포맷/문자열 조립 비용을 줄여 Population 탭 렌더 경로를 경량화.

## Commit 098
- `settlement_overview_tab`의 `UI_CHARISMA_FMT`, `UI_TOTAL_POP_FMT` 호출을 `Locale.trf1` 경량 포맷 경로로 전환.
- 기존 placeholder/출력 의미를 유지하면서 단순 포맷 호출의 임시 params Dictionary 생성을 줄임.
- Settlement Overview 탭 draw 경로의 미세 오버헤드를 완화.

## Commit 099
- `settlement_population_tab`의 `UI_TOTAL_POP_FMT` 호출을 `Locale.trf1` 경량 포맷 경로로 전환.
- 출력 의미를 유지하면서 draw 경로의 단일 placeholder 포맷 호출 임시 params Dictionary 생성을 줄임.
- Settlement Population 탭 렌더 미세 오버헤드를 완화.

## Commit 100
- `world_stats_tech_tab`의 `UI_TECH_COUNT_FMT` 호출을 `Locale.trf2` 경량 포맷 경로로 전환.
- 출력 의미를 유지하면서 2-파라미터 포맷 호출의 임시 params Dictionary 생성을 줄임.
- World Stats Tech 탭 렌더 경로의 미세 오버헤드를 완화.

## Commit 101
- `world_stats_panel` 헤더/푸터의 정착지 수 표시(`UI_SETTLEMENT_COUNT_FMT`)를 `Locale.trf1` 1회 생성 후 재사용 구조로 전환.
- 동일 draw 프레임에서 중복되던 locale 포맷 호출을 제거해 문자열 계산/임시 객체 생성을 줄임.
- 출력 의미를 유지하면서 World Stats 패널 렌더 경로를 경량화.

## Commit 102
- `building_detail_panel`의 `UI_STATUS_UNDER_CONSTRUCTION_FMT`, `UI_DETAIL_CAPACITY_FMT`, `UI_DETAIL_EFFECT_RADIUS_FMT` 호출을 `Locale.trf1` 경량 포맷 경로로 전환.
- 출력 의미를 유지하면서 단일 placeholder 포맷 호출의 임시 params Dictionary 생성을 줄임.
- Building Detail 패널 draw 경로의 미세 오버헤드를 완화.

## Commit 103
- `building_detail_panel`의 건물 조회를 `get_all_buildings()` 선형 탐색에서 `get_building(id)` direct lookup으로 전환.
- 패널 draw 경로에서 프레임당 건물 전체 순회를 제거해 조회 비용을 줄임.
- 표시 의미를 유지하면서 Building Detail 조회 핫패스를 경량화.

## Commit 104
- `settlement_detail_panel` 헤더 인구 라벨(`UI_STAT_POP_FMT`) 호출을 `Locale.trf1` 경량 포맷 경로로 전환.
- 출력 의미를 유지하면서 단일 placeholder 포맷 호출의 임시 params Dictionary 생성을 줄임.
- Settlement Detail 패널 draw 경로의 미세 오버헤드를 완화.

## Commit 105
- `Locale`에 `trf3`/`trf4` 경량 포맷 API를 추가해 3~4 placeholder 치환 시 임시 params Dictionary 생성을 줄이는 경로를 확장.
- `world_stats_population_tab`의 `UI_STAT_CURRENT_FMT`를 `trf4`로 전환.
- `settlement_overview_tab`의 `UI_POP_SUMMARY_FMT`를 `trf3`로 전환해 draw 경로 오버헤드를 완화.

## Commit 106
- `settlement_tech_tab`의 반복 포맷 호출을 `Locale.trf1/trf2/trf3` 경량 경로로 전환 (`UI_PRACTITIONERS_FMT`, `UI_NEEDS_MORE_FMT`, `UI_DISCOVERER_FMT`, `UI_AND_N_MORE`, `UI_STAT_POP_FMT`).
- 출력 의미를 유지하면서 draw 경로 임시 params Dictionary 생성을 줄임.
- Technology 탭 렌더 경로의 미세 오버헤드를 완화.

## Commit 107
- `settlement_overview_tab`의 `UI_ERA_PROGRESS_FMT` 호출을 `Locale.trf3` 경량 포맷 경로로 전환.
- 출력 의미를 유지하면서 3-파라미터 포맷 호출의 임시 params Dictionary 생성을 줄임.
- Settlement Overview 탭 draw 경로의 미세 오버헤드를 완화.

## Commit 108
- `pause_menu`의 단일 placeholder 포맷 호출(`UI_OVERWRITE_CONFIRM`, `UI_TIME_AGO_MINUTES/HOURS/DAYS`)을 `Locale.trf1`로 전환.
- 출력 의미를 유지하면서 메뉴 갱신 경로의 임시 params Dictionary 생성을 줄임.
- Pause 메뉴 텍스트 업데이트 경로의 미세 오버헤드를 완화.
