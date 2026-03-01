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

## Commit 109
- `stats_detail_panel_legacy`의 정적 포맷 호출(`UI_STAT_CURRENT_FMT`, `UI_STAT_GENDER_FMT`, `UI_STAT_COUPLES_FMT`, `UI_STAT_POP_FMT`, `UI_TECH_COUNT_FMT`)을 `Locale.trf1/trf2/trf4` 경량 경로로 전환.
- 출력 의미를 유지하면서 draw 경로 임시 params Dictionary 생성을 줄임.
- Legacy stats 패널 렌더 경로의 미세 오버헤드를 완화.

## Commit 110
- `map_editor/brush_palette`의 `UI_MAP_SPAWN_TOTAL` 호출 2곳을 `Locale.trf1` 경량 포맷 경로로 전환.
- 출력 의미를 유지하면서 스폰 합계 라벨 갱신 경로의 임시 params Dictionary 생성을 줄임.
- Map editor HUD 라벨 갱신의 미세 오버헤드를 완화.

## Commit 111
- `hud.gd`의 건물 저장소/시뮬레이션 이벤트/기술 상태 알림 포맷 호출을 `Locale.trf1/trf2/trf3` 경량 경로로 전환.
- `UI_NOTIF_WORLDSIM_STARTED_FMT`, `UI_FOLLOWING_FMT`를 `trf1`로 치환해 HUD 라벨 갱신의 params Dictionary 생성을 줄임.
- 출력 의미를 유지하면서 HUD 알림/라벨 갱신 경로의 미세 오버헤드를 완화.

## Commit 112
- `Locale`에 `trf5` 경량 포맷 API를 추가해 5-placeholder 치환의 params Dictionary 생성을 줄이는 경로를 확장.
- `game_calendar`의 날짜/시간/나이 포맷 호출을 `trf1~trf5`로 전환해 반복 포맷 경로를 경량화.
- `pause_menu`의 `UI_SLOT_FORMAT`을 `trf5`로 치환해 슬롯 버튼 텍스트 갱신의 미세 오버헤드를 완화.

## Commit 113
- `contagion_system`의 `CONTAGION_SPIRAL_WARNING` 포맷을 `Locale.trf2`로 전환.
- `attachment_system`의 `ATTACHMENT_FORMED` 포맷을 `Locale.trf2`로 전환.
- `coping_system`의 `COPING_ACQUIRED`/`COPING_UPGRADED` 포맷을 `Locale.trf1`로 전환해 이벤트 로그 경로의 임시 params Dictionary 생성을 줄임.

## Commit 114
- `parenting_system`의 `ADULTHOOD_TRANSITION` 설명 문자열 생성을 `Locale.trf1`로 전환.
- `child_stress_processor`의 `SHRP_OVERRIDE` 설명 문자열 생성을 `Locale.trf1`로 전환.
- `ace_tracker`의 `ACE_EVENT_RECORDED`/`HEXACO_CAP_MODIFIED` 설명 문자열 생성을 `Locale.trf2`/`trf1`로 전환하면서 chronicle 메타의 `params` 구조는 유지.

## Commit 115
- `chronicle_panel`의 이벤트 설명 렌더 경로에서 `l10n_params`를 기본 참조로 사용하고, `cause_id` 보정이 필요한 경우에만 `duplicate()`하도록 변경.
- `entity_detail_panel`의 life event 렌더 경로에서 `l10n_params` 매 프레임 복사를 제거.
- 동적 로컬라이징 표시 의미를 유지하면서 이벤트 목록 렌더의 dictionary 복사 오버헤드를 완화.

## Commit 116
- `chronicle_panel`의 `UI_SHORT_DATE` 호출에서 `month/day`를 문자열로 선변환하지 않고 정수로 직접 전달.
- `stats_detail_panel_legacy`의 `UI_TECH_COUNT_FMT` 호출에서 `known/forgotten` 선 문자열 변환을 제거.
- 출력 의미를 유지하면서 경량 포맷 호출 전 중복 문자열 변환 비용을 완화.

## Commit 117
- `Locale.trf`에 빈 파라미터 조기 반환 경로(`params.is_empty()`)를 추가.
- key-id 캐시 및 fallback 로직을 유지하면서 placeholder가 없는 동적 호출의 치환 루프를 생략.
- 출력 의미를 유지하면서 일반 `trf` 호출 경로의 미세 오버헤드를 완화.

## Commit 118
- `chronicle_system.log_event`의 `l10n` 저장 경로를 정규화해 `l10n_key`가 비어있으면 저장하지 않도록 변경.
- `l10n_params`가 비어있는 경우 엔트리에 기록하지 않도록 변경.
- 렌더 fallback 의미를 유지하면서 chronicle 이벤트 엔트리 payload 크기를 소폭 축소.

## Commit 119
- `tech_maintenance_system`의 chronicle params에서 `settlement` 값을 문자열 선변환 없이 정수로 직접 전달.
- `tech_propagation_system`의 teaching/imported 이벤트 params(`student`, `teacher`, `level`, `settlement`, `source`, `carrier`) 선 문자열 변환 제거.
- 출력 의미를 유지하면서 chronicle params 생성 시 불필요한 문자열 할당을 줄임.

## Commit 120
- `chronicle_system`에 DeceasedRegistry 노드 캐시(`_deceased_registry`)와 조회 헬퍼(`_get_deceased_registry`)를 추가.
- `_get_entity_name`에서 반복 `has_node/get_node` 탐색 대신 캐시 조회 경로로 전환.
- 이름 조회 의미를 유지하면서 chronicle 이벤트 기록 시 노드 탐색 오버헤드를 완화.

## Commit 121
- `chronicle_panel` 이벤트 렌더 루프에서 DeceasedRegistry를 루프 외부 1회 조회해 related entity 이름 조회 시 재사용.
- related entity 처리에서 반복 `get_node_or_null` 호출을 제거.
- 날짜 fallback 분기(`else`) 정렬을 정리해 동일 동작을 유지하면서 코드 안정성을 보강.

## Commit 122
- `chronicle_panel`에 이벤트 설명 문자열 캐시(`_desc_cache`, `_desc_cache_signature`)를 도입.
- locale 변경/필터 변경 시 캐시 무효화하고, 이벤트 집합 시그니처가 바뀔 때만 설명 문자열(`Locale.trf` 포함) 재생성.
- 표시 의미를 유지하면서 패널 redraw 루프에서 동적 로컬라이징/문자열 조립 오버헤드를 완화.

## Commit 123
- `entity_detail_panel` deceased life_events 렌더 경로에 설명 문자열 캐시(`_life_event_desc_cache`)를 도입.
- entity/locale 전환 시 캐시 무효화하고, 이벤트 시그니처 변경 시에만 `Locale.trf` 기반 설명 문자열을 재생성.
- 출력 의미를 유지하면서 Entity Detail 패널 redraw 루프의 life event 문자열 계산 오버헤드를 완화.

## Commit 124
- `sim-systems`에 body age curve 수식 모듈(`body::compute_age_curve`)을 추가하고 unit test로 검증.
- `sim-bridge`/`SimBridge`에 `body_compute_age_curve` API를 추가해 GDScript에서 Rust 연산 호출 경로를 노출.
- `body_attributes.compute_age_curve`가 Rust 우선 + GDScript fallback 구조로 전환되어 신체 나이 커브 계산의 네이티브 실행 경로를 확장.

## Commit 125
- `sim-systems::body`에 `calc_training_gain` 수식을 추가하고 관련 unit test를 확장.
- `sim-bridge`/`SimBridge`에 `body_calc_training_gain` API를 추가해 BodyAttributes 훈련 gain 계산의 Rust 호출 경로를 연결.
- `body_attributes.calc_training_gain`이 Rust 우선 + GDScript fallback 구조로 전환되어 신체 훈련 gain 계산 네이티브화 범위를 확장.

## Commit 126
- `sim-systems::body`에 `age_trainability_modifier` 분기표를 추가하고 unit test를 확장.
- `sim-bridge`/`SimBridge`에 `body_age_trainability_modifier` API를 추가해 BodyAttributes 나이 기반 훈련 효율 계산의 Rust 호출 경로를 연결.
- `body_attributes.get_age_trainability_modifier`가 Rust 우선 + GDScript fallback 구조로 전환되어 신체 훈련 관련 수학 경로 네이티브화를 확대.

## Commit 127
- `sim-systems::body`에 `compute_age_curves(age_years)` 배치 API를 추가하고, 인덱스 순서(`str/agi/end/tou/rec/dr`) 일치 테스트를 확장.
- `sim-bridge`/`SimBridge`에 `body_compute_age_curves`를 추가해 6축 나이 커브를 단일 bridge 호출로 수신하는 경로를 제공.
- `age_system`과 `entity_manager`의 realized 계산 루프가 배치 커브 결과를 재사용하도록 전환되어 body age curve hot path의 경계 호출 수를 축소.

## Commit 128
- `sim-systems::body`에 `age_trainability_modifiers(age_years)` 배치 API를 추가하고, 배치 인덱스 순서가 단건 호출과 일치하는 테스트를 확장.
- `sim-bridge`/`SimBridge`에 `body_age_trainability_modifiers`를 추가해 연령 기반 훈련 효율 5축을 단일 bridge 호출로 수신하는 경로를 제공.
- `gathering_system`과 `construction_system`의 XP 누적 경로가 배치 trainability 결과를 재사용하도록 전환되어 body trainability hot path의 경계 호출 수를 축소.

## Commit 129
- `sim-systems::body`에 `calc_training_gains(...)` 배치 API를 추가하고, 배치 결과가 단건 `calc_training_gain`과 동일함을 테스트로 검증.
- `sim-bridge`/`SimBridge`에 `body_calc_training_gains`를 추가해 training gain 5축을 단일 bridge 호출로 계산하는 경로를 제공.
- `body_attributes.calc_training_gain_batch`와 `age_system` 연간 재계산 루프가 배치 gain 결과를 재사용하도록 전환되어 body training gain hot path의 경계 호출 수를 축소.

## Commit 130
- `sim-systems::body`에 `calc_realized_values(...)`를 추가해 연간 body realized 6축(`str/agi/end/tou/rec/dr`) 계산을 단일 수학 경로로 통합하고, `calc_training_gains`에 `trainability < 0` sentinel(0 gain) 처리를 보강.
- `sim-bridge`/`SimBridge`에 `body_calc_realized_values`를 추가해 GDScript가 연간 realized를 1회 bridge 호출로 수신하는 경로를 제공.
- `body_attributes.calc_realized_values_batch`와 `age_system`이 단일 realized 배치 결과를 재사용하도록 전환되어 연간 body 재계산 hot path의 경계 호출 수를 추가 축소.

## Commit 131
- `entity_manager`의 스폰 초기 `body.realized` 계산이 `calc_realized_values_batch` 단일 호출 경로로 전환되어 연간 갱신 경로와 실행 구조를 일치시킴.
- 초기화 루프에서 개별 age-curve 기반 수동 계산을 제거하고, 배치 결과(`str/agi/end/tou/rec/dr`)를 직접 반영하도록 정리.
- bridge 지원 시 스폰 경로의 body 수학 호출이 단일 realized 배치 호출 중심으로 수렴.

## Commit 132
- `body_attributes`에 `calc_realized_values_packed`를 추가해 realized 6축 배치 결과를 `PackedInt32Array`로 직접 처리하는 경로를 제공하고, 기존 Dictionary API는 wrapper로 유지.
- `age_system`과 `entity_manager`가 realized 반영 시 packed 인덱스 접근을 사용하도록 전환되어 딕셔너리 생성/키 조회 오버헤드를 축소.
- 계산 의미와 clamp 범위는 유지한 채 연간/스폰 body realized 갱신 경로의 메모리 churn을 완화.

## Commit 133
- `body_attributes`에 `get_age_trainability_modifier_packed`를 추가해 trainability 5축 배치 결과를 `PackedFloat32Array`로 직접 처리하는 경로를 제공하고, 기존 Dictionary API는 wrapper로 유지.
- `gathering_system`과 `construction_system`이 trainability 조회 시 packed 인덱스 접근을 사용하도록 전환되어 딕셔너리 할당/키 조회 오버헤드를 축소.
- trainability 수치 의미를 유지하면서 작업 XP 누적 경로의 메모리 churn을 완화.

## Commit 134
- `body_attributes`에 `calc_training_gain_packed`를 추가해 training gain 5축 배치 결과를 `PackedInt32Array`로 직접 처리하는 경로를 제공하고, 기존 Dictionary API는 wrapper로 유지.
- `calc_realized_values_packed` fallback이 gain 조회 시 packed 인덱스 접근을 사용하도록 전환되어 Dictionary 기반 lookup 오버헤드를 추가로 축소.
- 계산 의미는 유지하면서 bridge 미지원 경로의 body 수학 처리 메모리 churn을 완화.

## Commit 135
- `sim-systems::body`에 `action_energy_cost`/`rest_energy_recovery`를 추가해 needs 시스템의 에너지 소모/회복 수식을 Rust 함수로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `BodyAttributes` 헬퍼(`compute_action_energy_cost`, `compute_rest_energy_recovery`)로 Rust 우선 + GDScript fallback 경로를 제공.
- `needs_system`이 해당 헬퍼를 사용하도록 전환되어 needs tick의 body 에너지 수학 연산 네이티브화 범위를 확장.

## Commit 136
- `sim-systems::body`에 `thirst_decay`/`warmth_decay`를 추가해 needs 시스템의 갈증/체온 소모 수식을 Rust 함수로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `needs_system`이 Rust 우선 + GDScript fallback으로 호출하도록 전환.
- 온도 기반 소모 의미를 유지하면서 needs tick의 환경 소모 계산 네이티브화 범위를 확장.

## Commit 137
- `sim-systems::body`에 `needs_temp_decay_step`를 추가해 갈증/체온 온도 소모를 단일 함수로 통합하고, 기존 개별 함수 재사용으로 수식 의미를 유지.
- `sim-bridge`/`SimBridge`에 `body_needs_temp_decay_step`를 추가해 needs 시스템이 온도 소모 값을 1회 bridge 호출로 수신하도록 확장.
- `needs_system`이 단일 결과를 재사용하도록 전환되어 온도 소모 계산 경로의 bridge round-trip을 추가 축소.

## Commit 138
- `sim-systems::body`에 `needs_base_decay_step`를 추가해 hunger/energy/social + (옵션) thirst/warmth 기본 소모를 단일 함수로 통합.
- `sim-bridge`/`SimBridge`에 `body_needs_base_decay_step`를 추가하고, Godot 파라미터 제한을 피하기 위해 packed scalar/flag 인코딩 경로로 노출.
- `needs_system`이 배치 결과를 재사용하도록 전환되어 needs 기본 소모 계산 경로의 bridge round-trip과 중복 계산을 추가 축소.

## Commit 139
- `SimBridge`에 `body_needs_base_decay_step_packed`를 추가하고 기존 scalar wrapper는 packed 호출로 위임하도록 정리.
- `needs_system`이 base decay 입력을 scratch `PackedFloat32Array`/`PackedByteArray`로 재사용하도록 전환되어 엔티티 루프의 배열 생성/append 오버헤드를 축소.
- base decay 계산 의미를 유지하면서 needs tick 경로의 메모리 churn을 추가 완화.

## Commit 140
- `needs_base_decay_step` 출력을 6축으로 확장해 safety decay를 base step에 포함하고, `needs_expansion_enabled` 조건에서만 안전감 소모가 반영되도록 통합.
- `sim-bridge`/`SimBridge` packed scalar 인코딩 순서를 safety 포함 구조로 조정.
- `needs_system`이 base step 결과에서 safety를 직접 반영하도록 전환되어 기본 소모 경로의 분리 safety 감산 블록을 제거하고 실행 경로를 단순화.

## Commit 141
- `sim-systems::body`에 `critical_severity`와 `needs_critical_severity_step`를 추가해 갈증/체온/안전감 임계치 severity 계산을 Rust 배치 함수로 이관.
- `sim-bridge`/`SimBridge`에 `body_needs_critical_severity_step`를 추가하고, `needs_system`이 단일 결과를 재사용하도록 전환.
- needs stressor 구간의 severity 계산 의미를 유지하면서 bridge 호출/중복 계산 경로를 정리.

## Commit 142
- `needs_base_decay_step` 통합 이후 중복이 된 `needs_temp_decay_step`(sim-systems/sim-bridge/SimBridge)를 제거해 API 표면과 코드 복잡도를 축소.
- `needs_system`이 temp-decay 계산에서 중복 bridge 재호출 없이 base decay 결과 재사용 또는 GDScript fallback만 사용하도록 정리.
- 계산 의미를 유지하면서 needs 소모 경로의 중복 분기와 유지보수 비용을 완화.

## Commit 143
- `sim-bridge`/`SimBridge`에 `body_needs_critical_severity_step_packed`를 추가하고 기존 scalar API는 packed 경로로 위임하도록 정리.
- `needs_system`이 임계치 severity 입력을 scratch `PackedFloat32Array`로 재사용하도록 전환되어 엔티티 루프의 인자 구성 할당 오버헤드를 축소.
- severity 계산 의미를 유지하면서 needs stressor 경로의 메모리 churn을 완화.

## Commit 144
- `sim-bridge`/`SimBridge`에 `body_age_trainability_modifier_rec` 전용 API를 추가해 휴식 경로의 REC trainability 조회를 축 문자열 전달 없이 처리.
- `body_attributes`에 `get_rec_age_trainability_modifier` 헬퍼를 추가하고 Rust 우선 + 기존 축 기반 함수 fallback 구조를 제공.
- `needs_system` 휴식 XP 경로가 REC 전용 helper를 사용하도록 전환되어 미세 호출 오버헤드를 완화.

## Commit 145
- `sim-test`에 `--bench-needs-math` 모드를 추가해 body/needs Rust 수학 경로(`age_curve/trainability/training_gain/realized/needs_decay/critical_severity`)를 독립적으로 마이크로벤치 가능하게 확장.
- 벤치 반복 횟수 파싱 로직을 `parse_bench_iterations`로 공통화해 `--bench-stress-math`와 `--bench-needs-math`가 동일한 `--iters` 인터페이스를 사용.
- 기존 stress 벤치 동작은 유지한 채, needs/body 경로의 성능 회귀 추적 기준점(`checksum=29719684.00000` @ 10k iters)을 추가.

## Commit 146
- `sim-systems::body`에 `erg_frustration_step` 배치 함수를 추가해 성장/관계 좌절 누적 tick 갱신과 회귀 전이(started) 판정을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 `body_erg_frustration_step_packed` 경로를 추가하고, `needs_system`이 scratch packed 입력을 재사용해 엔티티당 1회 호출로 상태를 반영하도록 전환.
- bridge 미지원 시 기존 GDScript fallback 계산을 유지하면서, 스트레스 주입/`erg_regression_started` 이벤트 조건은 started 플래그 기준으로 동일 의미를 유지.

## Commit 147
- `sim-systems::body`에 `anxious_attachment_stress_delta`를 추가해 anxious attachment 조건의 사회욕구 기반 스트레스 증가 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 `body_anxious_attachment_stress_delta` 경로를 추가하고, `needs_system` 분기가 Rust 우선 + 기존 비교 fallback 구조를 사용하도록 전환.
- 스트레스 증가 조건(`social < threshold`)과 clamp 의미는 유지하면서 needs tick 수학 경로의 네이티브화 범위를 확장.

## Commit 148
- `needs_system`에서 base decay/critical severity 처리 시 임시 `PackedFloat32Array`(`base_decay_step`, `rust_temp_decay`, `severity_step`) 생성 경로를 제거.
- Rust 반환 packed 값을 즉시 스칼라로 디코드해 갈증/체온/안전감 소모 및 severity 계산에 재사용하도록 전환.
- 계산 의미는 유지하면서 needs tick 루프의 메모리 할당 churn을 추가 완화.

## Commit 149
- `sim-systems::body`에 upper-needs 관련 Rust 수학 함수(`upper_needs_best_skill_normalized`, `upper_needs_job_alignment`, `upper_needs_step`)를 추가하고 unit test를 확장.
- `sim-bridge`/`SimBridge`에 upper-needs 전용 bridge API 3종을 추가해 GDScript가 packed 입력으로 상위욕구 통합 스텝을 1회 호출할 수 있도록 확장.
- `upper_needs_system`이 Rust 통합 스텝 우선 + 기존 `_apply_decay/_apply_fulfillment/_clamp_upper_needs` fallback 구조로 전환되었고, scratch packed 버퍼 재사용으로 루프 할당을 억제.
- `sim-test --bench-needs-math`에 upper-needs 수학 호출을 포함해 회귀 추적 범위를 확장(신규 checksum 기준: `29743414.00000` @ 10k iters).

## Commit 150
- `localization_compile`에 append-only key registry(`key_registry.json`)를 도입해 기존 key ID 순서를 유지하면서 신규 키만 뒤에 추가하는 구조로 확장.
- `manifest.json`에 `key_registry_path`/`preserve_key_ids` 옵션을 추가했고, compiled locale meta에 `active_key_count` 및 registry 관련 메타를 반영.
- 런타임 번역 값은 유지하면서, 장기 확장/마이그레이션 시 key ID 안정성과 추적성을 강화.

## Commit 151
- `localization_compile`에 `embed_keys` 옵션을 추가해 compiled locale에서 중복 `keys` 배열을 제거할 수 있도록 확장(현재 `false`).
- `Locale` 런타임이 compiled JSON에 `keys`가 없을 때 `key_registry.json`의 키 순서를 읽어 key-id 인덱스를 재구성하도록 개선.
- 번역 결과는 유지하면서 compiled payload 중복을 줄이고, registry 기반 안정 key-id 체계를 런타임 경로까지 연결.

## Commit 152
- `localization_compile`에 `_write_json_if_changed`를 도입해 내용 동일 시 compiled/registry 파일 쓰기를 생략하도록 개선.
- 컴파일 로그에 `updated` 플래그를 추가해 실제 파일 갱신 여부를 즉시 확인 가능하게 확장.
- 반복 검증 시 불필요한 localization 산출물 재작성 및 git noise를 감소.

## Commit 153
- localization manifest에 `max_duplicate_key_count`를 도입해 중복 키 개수 기준선을 명시(현재 248).
- `localization_compile`이 로케일별 최대 중복 개수를 계산해 기준 초과 시 실패하도록 회귀 가드를 추가.
- 향후 확장 시 중복 키 증가를 컴파일 단계에서 조기 차단하도록 품질 게이트를 강화.

## Commit 154
- `upper_needs_system`의 Rust 우선 경로에서 fallback(`_get_best_skill_normalized`, `_get_job_value_alignment`) 선계산을 제거.
- bridge 결과가 없을 때만 fallback을 계산하도록 변경해 Rust 활성 환경의 불필요한 GDScript 연산을 절감.
- 수치 의미는 유지하면서 상위욕구 tick 경로의 호출당 비용을 미세 최적화.

## Commit 155
- `sim-systems::body`에 `child_parent_stress_transfer`를 추가해 부모→아동 스트레스 전이(attachment/버퍼/contagion 결합) 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 `body_child_parent_stress_transfer` 경로를 추가하고, `child_stress_processor`가 Rust 우선 호출 + 기존 함수 fallback 구조를 사용하도록 전환.
- child stress tick hot path에서 전이 수식의 스크립트 연산을 줄이면서 기존 의미와 fallback 호환성을 유지.

## Commit 156
- `sim-systems::body`에 `child_simultaneous_ace_step`를 추가해 동시 ACE severity 집계의 burst/residual/kindling 계산을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 `body_child_simultaneous_ace_step` 경로를 추가하고, `child_stress_processor`가 scratch 버퍼 기반 Rust 우선 처리 + 기존 fallback 구조를 사용하도록 전환.
- child stress의 동시 ACE 처리 의미를 유지하면서 해당 수학 경로의 스크립트 연산 비중을 추가로 축소.

## Commit 157
- `sim-test --bench-needs-math`에 `child_parent_stress_transfer`와 `child_simultaneous_ace_step` 호출을 추가해 child stress Rust 수식까지 회귀 추적 범위를 확장.
- 벤치 checksum 기준을 child 수식 포함 값(`29781070.00000` @ 10k iters)으로 갱신.
- 런타임 코드 경로는 변경하지 않고 성능/회귀 관측 지표를 강화.

## Commit 158
- `sim-systems::body`에 `child_social_buffered_intensity`를 추가해 child stress social-buffer 감쇠 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `child_stress_processor._apply_social_buffer`가 Rust 우선 + 기존 fallback 구조를 사용하도록 전환.
- child stress 경로의 순수 감쇠 수학을 네이티브화하면서 기존 의미와 호환성을 유지.

## Commit 159
- `sim-systems::body`에 `child_shrp_step`, `child_stress_type_code`를 추가해 child stress의 SHRP/분류 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `child_stress_processor`가 Rust 우선 + 기존 fallback 구조를 사용하도록 전환.
- toxic onset Chronicle 이벤트 부작용은 GDScript에 유지해 기능 의미를 보존하면서 계산 경로의 네이티브화를 확장.

## Commit 160
- `sim-systems::body`에 `child_stress_apply_step`를 추가해 child stress positive/tolerable/toxic 상태 업데이트 수식을 Rust로 통합 이관.
- `sim-bridge`/`SimBridge`에 `body_child_shrp_step`, `body_child_stress_type_code`, `body_child_stress_apply_step` 경로를 추가하고 `child_stress_processor`가 Rust 우선 + fallback 구조를 사용하도록 전환.
- `sim-test --bench-needs-math`에 child apply 관련 수식 호출을 추가해 회귀 추적 범위를 확장(신규 checksum 기준: `33378700.00000`).

## Commit 161
- `sim-systems::body`에 `stress_support_score`를 추가해 strongest tie + weak ties 포화 결합 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 `body_stress_support_score` 경로를 추가하고, `stress_system._calc_support_score`가 Rust 우선 + fallback 구조를 사용하도록 전환.
- `sim-test --bench-stress-math`에 support score 호출을 포함해 회귀 추적 범위를 확장(신규 checksum 기준: `13767388.00000` @ 10k iters).

## Commit 162
- `sim-systems::body`에 `child_parent_transfer_apply_step`, `child_deprivation_damage_step`를 추가해 child stress의 잔여 단순 수식(전이 반영, deprivation 누적)을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `child_stress_processor`가 Rust 우선 + fallback 구조로 해당 분기를 처리하도록 전환.
- `sim-test --bench-needs-math`에 두 수식 호출을 추가해 회귀 추적 범위를 확장(신규 checksum 기준: `38434752.00000` @ 10k iters).

## Commit 163
- `sim-systems::body`에 `stress_rebound_apply_step`, `stress_shaken_countdown_step`를 추가해 stress 후처리(rebound 반영/hidden 감소, shaken 카운트다운) 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `stress_system`이 해당 구간을 Rust 우선 + fallback 구조로 처리하도록 전환.
- `sim-test --bench-stress-math`에 rebound/shaken step 호출을 추가해 회귀 추적 범위를 확장(신규 checksum 기준: `20039734.00000` @ 10k iters).

## Commit 164
- `sim-systems::body`에 `child_stage_code_from_age_ticks`를 추가해 child stage 연령 구간 판정 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `child_stress_processor`가 stage cutoff 캐시 기반으로 Rust 우선 판정을 사용하도록 전환.
- `sim-test --bench-needs-math`에 stage code 호출을 추가해 회귀 추적 범위를 확장(신규 checksum 기준: `38457848.00000` @ 10k iters).

## Commit 165
- `sim-systems::body`에 `stress_injection_apply_step`를 추가해 stress 이벤트 주입 후처리(즉시 clamp + trace append 판정) 수식을 Rust로 이관.
- `sim-bridge`/`SimBridge`에 대응 API를 추가하고, `stress_system`의 `inject_stress_event`/`inject_event`가 Rust 우선 + fallback 구조를 사용하도록 전환.
- `sim-test --bench-stress-math`에 injection apply step 호출을 추가해 회귀 추적 범위를 확장(신규 checksum 기준: `24032652.00000` @ 10k iters).

## Commit 166
- `stress_system`의 rebound queue 처리에 packed cache(`rebound_queue_amounts`, `rebound_queue_delays`)를 도입해 tick당 딕셔너리 파싱 비용을 완화.
- `schedule_rebound`/`_process_rebound_queue`가 legacy `rebound_queue`와 packed cache를 동기화하도록 확장해 기존 호환성을 유지.
- 벤치 checksum은 유지(`stress=24032652.00000`, `needs=38457848.00000` @ 10k iters)하며 런타임 처리 경로 효율을 개선.

## Commit 167
- `localization_compile`에 `max_missing_key_fill_count` 회귀 게이트를 추가해 locale 누락 채움 수 증가를 컴파일 단계에서 차단.
- `localization/manifest.json`에 기준선(`max_missing_key_fill_count: 0`)을 추가해 확장 시 번역 누락 회귀를 조기 검출.
- 런타임 번역 동작은 유지하면서 localization 품질 게이트를 강화.

## Commit 168
- `Locale.ltr()`에 key-id 캐시(`_ltr_key_id_cache`)를 도입해 반복 키 조회를 id 기반 경로(`ltr_id`)로 통일.
- locale 재로딩 시 캐시를 초기화해 key index 변경과의 정합성을 유지.
- 번역 의미는 유지하면서 런타임 localization 조회 경로를 추가 최적화.

## Commit 169
- `SimBridge`에 pathfinding backend 제어/조회 공개 API(`set/get/resolve/has_gpu`)를 추가해 GPU 옵션 구조를 스크립트 레벨에서 직접 다룰 수 있게 확장.
- `ComputeBackend`가 `_ready` 및 `set_mode` 시점에 pathfinding backend 선호 모드를 bridge와 즉시 동기화하도록 연결.
- GPU 미구현 환경에서도 기존 CPU fallback 의미를 유지하면서, 향후 GPU 경로 구현을 위한 옵션-전달 구조를 고정.

## Commit 170
- `pathfinder.gd` 배치 Rust 경로탐색에서 `PackedInt32Array`/`PackedVector2Array` scratch 버퍼를 재사용하도록 전환.
- 호출마다 임시 배열을 새로 만들던 경로를 `resize + index write` 방식으로 바꿔 allocation churn을 감소.
- 결과 수치 의미는 유지하면서 대규모 배치 pathfinding 경로의 런타임 효율을 개선.

## Commit 171
- `pathfinder`에 packed XY 전용 batch API를 추가하고, movement 시스템이 recalc 요청을 Dictionary 배열 대신 packed 좌표 버퍼로 구성하도록 전환.
- 배치 경로 계산에서 Rust batch-xy 경로를 우선 사용하고 vec2/GDScript fallback을 유지.
- 경로 의미는 유지하면서 movement tick의 요청 구성 오버헤드를 완화.

## Commit 172
- `movement_system`의 `path_entities`/`recalc_entities`를 scratch Array로 전환해 tick 루프에서 임시 Array 할당을 제거.
- 기존 packed XY recalc 버퍼 재사용 경로와 결합해 movement hot path의 메모리 churn을 추가 완화.
- 이동/경로 의미는 유지하면서 실행 경로 효율을 개선.

## Commit 173
- `movement_system`의 path recalc 배치 호출을 엔티티 스캔 이후 1회 처리로 정렬해 loop 내부 중복 호출을 제거.
- `execute_tick`의 `_pathfinder` 분기 들여쓰기/스코프를 정리해 recalc 적용 로직(`recalc_count`)을 안정화.
- 이동 의미는 유지하면서 movement hot path에서 불필요한 경로탐색 호출 비용을 완화.

## Commit 174
- `pathfinder`에 bridge 메서드 capability 캐시를 추가해 hot path의 반복 `has_method` 체크를 1회 캐시로 전환.
- `_find_path_rust`/`_find_paths_rust_batch`/`_find_paths_rust_batch_xy`가 캐시 기반 분기를 사용하도록 정리.
- 경로 계산 의미는 유지하면서 pathfinding 호출 경로의 분기 오버헤드를 미세 완화.

## Commit 175
- `movement_system` greedy fallback 이동에서 후보 `Array[Vector2i]` 생성/순회를 제거하고 분기 기반 이동 시도로 전환.
- 공통 이동 처리를 `_try_move_candidate`로 추출해 walkable 체크/이동 이벤트 emit 경로를 단순화.
- 이동 우선순위 의미는 유지하면서 fallback 경로의 메모리 할당 churn을 완화.

## Commit 176
- `sim_bridge`에 pathfinding backend resolve 캐시를 도입해 경로탐색 호출 시 반복 resolve bridge call을 축소.
- `resolve_pathfinding_backend`/`_prefer_gpu`가 공통 캐시 헬퍼를 사용하도록 정리하고, mode sync/bridge 교체 시 캐시 무효화 규칙을 추가.
- backend 선택 의미는 유지하면서 GPU 옵션 경로의 런타임 분기 오버헤드를 완화.

## Commit 177
- `movement_system`에서 periodic path recalc 판정(`50 tick`)을 tick당 1회 계산해 엔티티 루프에서 재사용.
- `_needs_path_recalc`가 modulo 연산 대신 전달된 bool을 사용하도록 정리하고 호출부 시그니처를 일치화.
- recalc 의미는 유지하면서 movement hot path의 반복 연산 비용을 미세 완화.

## Commit 178
- `movement_system`에 `_clear_cached_path`를 도입해 빈 경로 처리 시 `cached_path` 배열을 재사용하고 재할당을 최소화.
- action 완료/재계산 실패/path blocked 구간의 경로 초기화를 공통 헬퍼로 통일.
- 이동 의미는 유지하면서 movement hot path의 빈 경로 처리 메모리 churn을 완화.

## Commit 179
- `pathfinder` path 정규화 함수들이 `append` 누적 대신 `resize + index write`와 `write_idx` compaction을 사용하도록 전환.
- `PackedInt32Array`/`PackedVector2Array` 및 `Array` 입력 모두에서 결과 좌표 의미를 유지하며 배열 생성 churn을 완화.
- pathfinding hot path 정규화 단계의 미세 성능 최적화를 반영.

## Commit 180
- `pathfinder` batch 결과 정규화 배열(`xy_normalized`, `normalized`)을 `append` 누적에서 `resize + index write`로 전환.
- `_find_paths_rust_batch`와 `_find_paths_rust_batch_xy` 모두에 동일 패턴을 적용해 그룹 포장 단계의 동적 확장 비용을 완화.
- path 결과 의미는 유지하면서 batch pathfinding 후처리 오버헤드를 미세 최적화.

## Commit 181
- `pathfinder` fallback batch API(`find_paths_batch`, `find_paths_batch_xy`)의 결과 배열 생성을 `append`에서 `resize + index write`로 전환.
- Rust 경로가 없는 환경에서도 fallback 결과 포장 오버헤드를 줄이도록 정렬.
- path 결과 의미는 유지하면서 fallback 경로의 메모리 churn을 미세 완화.

## Commit 182
- `pathfinder` 내부 Rust 경로 반환을 `{used, path(s)}` 딕셔너리 래퍼에서 `null/Array` 반환으로 정리.
- `find_path`/`find_paths_batch`/`find_paths_batch_xy` 호출부가 `Variant` null 체크 기반 분기를 사용하도록 갱신.
- path 계산 의미는 유지하면서 hot path 딕셔너리 할당/키 조회 비용을 완화.

## Commit 183
- `movement_system`의 `_needs_path_recalc`, `_apply_recalculated_path`에서 path size 조회를 지역 변수로 통일해 중복 호출/분기를 정리.
- 경로 재계산/적용 의미는 유지하면서 movement hot path의 미세 계산 비용을 완화.

## Commit 184
- `sim_bridge`에 GPU pathfinding capability probe 캐시를 도입해 `has_gpu_pathfinding`/`_prefer_gpu`의 반복 메서드 확인 및 호출 비용을 축소.
- native bridge 교체 시 capability 캐시를 함께 무효화하도록 초기화 경로를 보강.
- GPU 가능 여부 의미를 유지하면서 pathfinding backend 분기 hot path를 미세 최적화.

## Commit 185
- `sim_bridge`의 `_resolve_pathfinding_backend_cached`가 cache hit 시 즉시 반환하도록 순서를 조정해 매 호출 sync 비용을 줄임.
- cache miss에서만 sync를 수행하고, sync 후 cache 설정 가능성을 고려한 재확인 분기를 추가.
- backend 결정 의미를 유지하면서 `_prefer_gpu` hot path 분기 오버헤드를 미세 최적화.

## Commit 186
- `world_data`에 `terrain_revision`을 도입하고 `set_tile` 실변경 시 revision을 증가시켜 지형 변경 추적을 명시화.
- `pathfinder` 캐시 재빌드 조건에 revision 비교를 추가해 동적 지형 변경 후 stale walkable/move_cost 캐시 사용을 방지.
- 캐시 재사용 효율은 유지하면서 pathfinding 정합성을 강화.

## Commit 187
- `world_data`에 타일 배치 업데이트 API(`begin_tile_update`/`end_tile_update`)를 추가해 대량 `set_tile` 구간의 revision 증가를 1회로 coalescing.
- `world_generator`와 `preset_map_generator`가 생성 루프를 배치 업데이트로 감싸 생성 중 revision churn을 크게 감소.
- 지형 결과 의미를 유지하면서 revision 기반 cache invalidation 경로의 운영 비용을 완화.

## Commit 188
- `sim-bridge` batch pathfinding 경로에서 월드 그리드를 요청마다 재구성하던 구조를 제거하고, 배치당 1회 grid 빌드 후 재사용하도록 개선.
- 길이 검증 + grid 생성 로직을 `build_grid_cost_map`으로 공통화해 `pathfind_grid_bytes`/`pathfind_grid_batch_bytes` 경로를 단순화.
- path 의미/에러 의미는 유지하면서 batch pathfinding의 핵심 오버헤드를 완화.

## Commit 189
- `sim-bridge`에 packed XY 슬라이스 직접 순회 경로(`pathfind_grid_batch_xy_bytes`)를 추가해 `pathfind_grid_batch_xy`의 tuple 디코딩 중간 할당을 제거.
- 기존 `decode_xy_pairs`를 제거하고 XY batch 브리지 호출이 곧바로 슬라이스 기반 batch pathfinding을 사용하도록 전환.
- path 의미를 유지하면서 XY batch 경로의 메모리/CPU 오버헤드를 완화.

## Commit 190
- `sim-bridge` Vec2 batch pathfinding 경로에 슬라이스 직접 순회 헬퍼(`pathfind_grid_batch_vec2_bytes`)를 추가해 tuple 디코딩 벡터 할당을 제거.
- `pathfind_grid_batch`가 중간 좌표 변환 벡터 없이 배치당 1회 grid 재사용 + 직접 순회 경로를 사용하도록 전환.
- path 의미를 유지하면서 Vec2 batch 경로의 변환 오버헤드를 완화.

## Commit 191
- `sim-bridge` path 출력 인코딩에 `encode_path_vec2`/`encode_path_groups_vec2`를 도입해 `Vec<Vector2>` 중간 collect를 제거.
- `pathfind_grid`/`pathfind_grid_batch`가 Packed 배열 직접 인코딩 경로를 사용하도록 정리.
- path 의미를 유지하면서 bridge 결과 포장 단계의 메모리/CPU 오버헤드를 완화.

## Commit 192
- `sim-test`에 `--bench-pathfind-bridge` 모드를 추가해 Rust bridge batch pathfinding(`tuple`, `packed XY`) 경로를 직접 벤치 가능하게 확장.
- 벤치 경로에서 장거리 탐색이 빈 결과로 수렴하지 않도록 `max_steps`를 맵 셀 수 기반으로 조정하고, 기본 반복 수를 `1000`으로 낮춰 실사용 가능한 실행 시간을 확보.
- 기존 stress/needs 체크섬(`24032652.00000`, `38457848.00000`)은 유지하면서 pathfinding 벤치 checksum 기준(`708000.00000` @ 1k, `7080000.00000` @ 10k)을 추가.

## Commit 193
- `tools/migration_verify.sh`에 `--with-benches` 옵션을 추가해 기본 4단계 검증은 유지하면서 선택적으로 Rust 벤치 체크섬 회귀까지 자동 검증할 수 있게 확장.
- `run_bench_and_check` 헬퍼를 도입해 벤치 출력 `checksum` 파싱/비교를 표준화하고 mismatch 시 즉시 실패하도록 강화.
- 벤치 기준선으로 `pathfind-bridge=70800.00000`(@100), `stress=24032652.00000`(@10k), `needs=38457848.00000`(@10k)를 검증 파이프라인에 연결.

## Commit 194
- `sim-bridge` 테스트에 tuple/packed XY/Vec2 batch pathfinding 결과 동치성 검증을 추가해 최근 batch 최적화 경로의 회귀 탐지 범위를 확장.
- XY packed 입력 홀수 길이 오류 케이스를 명시적으로 테스트해 입력 검증 경계를 강화.
- 런타임 로직 변경 없이 브리지 API 안정성을 테스트 레벨에서 보강.

## Commit 195
- `localization_audit.py`에 중복 키 상세 분석(`duplicate_details`)을 추가해 파일별 값과 `value_conflict` 여부를 추적하도록 확장.
- 출력 요약에 `duplicate_conflicts`/`duplicate_consistent`를 추가하고, 충돌 키 상위 목록을 즉시 노출하도록 개선.
- `--strict-duplicate-conflicts`, `--report-json`, `--duplicate-report-json` 옵션을 추가해 중복 충돌을 선택적으로 품질 게이트/산출물화 가능하게 정리.

## Commit 196
- `localization_compile.py`가 duplicate key 중 값 충돌(`duplicate_conflict_keys`)을 별도 계산하도록 확장되고, locale meta/log에 `duplicate_conflict_count`를 노출.
- manifest에 `max_duplicate_conflict_count` 기준선을 추가해 값 충돌 중복의 증가를 컴파일 단계에서 자동 차단.
- compiled locale(`en/ko`) meta에도 충돌 수를 기록해 산출물 기준으로 중복 위험도를 추적 가능하게 정리.

## Commit 197
- `localization_audit.py`의 duplicate 집계를 `supported_locales` 전체로 확장하고 로케일별 충돌 요약(`duplicate_locale_summary`)을 추가.
- 충돌 최대 로케일을 `duplicate_report_locale`로 선택해 top-level `duplicate_conflicts`와 상세 충돌 목록 기준을 compile 게이트 관점과 일치시킴.
- 결과적으로 audit/compile가 동일하게 `ko=35` 충돌 기준을 보고하도록 정렬.

## Commit 198
- `sim-bridge`에 `validate_grid_inputs`를 추가해 grid pathfinding API의 차원/버퍼 길이 검증을 공통화하고 `InvalidDimensions`를 명시적으로 반환하도록 보강.
- 단일 경로 API(`pathfind_grid_bytes`)에서 `from==to` stationary 케이스를 검증 후 즉시 반환해 불필요한 그리드 빌드 오버헤드를 제거.
- 관련 경계 테스트(차원 오류/stationary 반환)를 추가해 입력 검증과 fast-path 의미를 고정.

## Commit 199
- `sim-bridge` batch pathfinding(tuple/XY/Vec2)에 stationary 질의 fast-path를 추가해 `from==to` 항목에서 A* 호출을 생략하고 singleton path를 즉시 반환하도록 최적화.
- 전체 배치가 stationary인 경우 grid 구성 자체를 건너뛰는 early-return 경로를 추가해 batch 오버헤드를 추가 절감.
- tuple/XY/Vec2 모두에 대해 stationary 반환 의미를 테스트로 고정.

## Commit 200
- `migration_verify.sh --with-benches`에 벤치 반복 수 환경변수(`MIGRATION_BENCH_*_ITERS`)를 추가하고 입력 유효성 검사를 도입.
- 기본 반복 수일 때는 기존 checksum 기준선 검증을 유지하고, 비기본 반복 수일 때는 관측 모드로 자동 전환해 파이프라인 유연성을 확장.
- 벤치 단계가 현재 반복 수 설정을 명시적으로 출력하도록 개선해 실행 컨텍스트 추적성을 강화.

## Commit 201
- `GridCostMap`에 flat bool/byte 버퍼 직생성 API(`from_flat_unchecked`, `from_flat_bytes_unchecked`)를 추가해 grid 초기화 시 per-cell setter 경로를 대체.
- `sim-bridge`가 새 직생성 API를 사용하도록 연결해 pathfinding grid 구성 오버헤드를 완화.
- move_cost clamp 의미(`max(0.0)`)를 테스트와 함께 유지해 기존 동작 호환성을 보장.

## Commit 202
- `from_flat_bytes_unchecked`가 중간 `Vec<bool>` 재복사 경로를 제거하고 최종 `GridCostMap`을 직접 구성하도록 정리.
- move_cost clamp 의미는 유지하면서 byte 기반 grid 초기화 경로의 할당/복사 오버헤드를 추가 절감.

## Commit 203
- `sim-test`에 `--bench-pathfind-bridge-split` 모드를 추가해 tuple batch와 packed XY batch 경로의 성능/체크섬을 독립적으로 관측할 수 있게 확장.
- pathfinding 벤치 입력 생성 로직을 공통 헬퍼로 분리해 split/combined 모드 간 입력 정합성을 고정.
- split 기준선 관측값: tuple `354000.00000`, xy `354000.00000` (@1000).

## Commit 204
- `migration_verify.sh`에 `MIGRATION_BENCH_PATH_SPLIT` 옵션을 추가해 path split 벤치를 선택적으로 실행하고 tuple/XY checksum 정합성을 자동 확인하도록 확장.
- split 벤치 checksum 파싱/검증 로직을 추가하고, macOS bash 3.x 호환을 위해 `mapfile` 없이 `sed` 기반 파싱으로 구현.
- 기본 검증 경로는 유지하면서 필요 시 경로별 벤치 관측을 파이프라인에 통합.

## Commit 205
- `migration_verify` strict audit 단계에 `MIGRATION_AUDIT_REPORT_JSON`/`MIGRATION_AUDIT_DUPLICATE_REPORT_JSON` 환경변수 연동을 추가해 localization audit 결과를 JSON 아티팩트로 바로 저장 가능하게 확장.
- 환경변수 미설정 시 기존 strict audit 동작을 그대로 유지해 기존 파이프라인과 호환.
- 벤치 옵션과 함께 사용해 검증 + 리포트 산출을 한 번에 수행할 수 있도록 운영성을 보강.

## Commit 206
- `migration_verify` split path 벤치에 기본 반복수(`path=100`) 전용 strict checksum 기준선 검증을 추가해 tuple/xy 회귀를 자동 차단.
- 비기본 반복수에서는 기존 관측 모드를 유지해 실험/튜닝 유연성을 보장.
- split 체크섬 기준선: tuple=`35400.00000`, xy=`35400.00000`.

## Commit 207
- `sim-systems` A* 코어를 `HashMap/HashSet`에서 인덱스 기반 `Vec` 상태 배열로 전환해 pathfinding hot path의 해시/할당 오버헤드를 제거.
- 경로 재구성도 인덱스 체인 기반으로 교체하고 시작점 out-of-bounds 경계 테스트를 추가해 의미를 고정.
- checksum은 유지하면서 `pathfind-bridge` 성능이 약 `20.1ms/iter -> 2.9ms/iter` 수준으로 크게 개선.

## Commit 208
- `sim-systems` A* open-set 선택 로직을 선형 스캔에서 `BinaryHeap` 기반 우선순위 큐로 전환해 최소 f-score 탐색 비용을 추가 절감.
- stale heap entry 스킵 + 유효 확장 노드 기준 `max_steps` 카운팅으로 의미를 유지하면서 조기 종료 리스크를 제거.
- checksum을 유지한 상태에서 `pathfind-bridge` 성능이 추가로 `~2.9ms/iter -> ~2.0ms/iter` 수준으로 개선.

## Commit 209
- `find_path` 휴리스틱 계산을 `GridPos` 임시 생성 경로에서 좌표 직접 계산(`chebyshev_xy`)으로 정리해 미세 오버헤드를 줄이고 경고를 제거.
- 목표 좌표 캐시(`to_x`, `to_y`)를 도입해 neighbor 평가 경로의 반복 필드 접근을 축소.
- checksum 유지 상태로 pathfinding 경로의 미세 최적화를 추가 반영.

## Commit 210
- `GridCostMap::from_flat_owned_unchecked`를 추가해 소유 `walkable/move_cost` 버퍼를 직접 소비하고 move_cost clamp를 in-place 처리하도록 확장.
- `sim-bridge.pathfind_from_flat`가 새 소유 버퍼 경로를 사용하도록 전환되어 flat 입력 pathfinding의 재복사를 제거.
- checksum 유지 상태로 bridge flat 입력 경로의 메모리 효율을 개선.

## Commit 211
- `localization_audit.py`에 중복 충돌 Markdown 리포트 생성기(`--duplicate-conflict-markdown`)를 추가해 conflict key/파일/값 샘플을 표 형태로 출력 가능하게 확장.
- `migration_verify`가 `MIGRATION_AUDIT_CONFLICT_MARKDOWN` 환경변수를 통해 strict audit 실행 중 Markdown 산출물을 생성하도록 연동.
- 기존 JSON 리포트 경로와 병행해 사람이 바로 검토 가능한 충돌 요약 아티팩트 생산성을 강화.

## Commit 212
- `sim-bridge` pathfinding 테스트에 시작점 out-of-bounds 케이스(단건/배치)를 추가해 OOB 시작 질의의 빈 경로 반환 의미를 고정.
- batch 테스트에서 OOB/정상 질의를 함께 검증해 부분 실패 시나리오의 동작 안정성을 보강.
- 런타임 변경 없이 경계 조건 회귀 탐지 범위를 확장.

## Commit 213
- `sim-systems`에 재사용 가능한 `PathfindWorkspace`와 `find_path_with_workspace`를 추가해 pathfinding scratch 버퍼를 호출 간 재사용 가능하게 확장.
- `sim-bridge` batch pathfinding(tuple/xy/vec2)이 batch당 1회 workspace를 생성해 모든 질의에서 재사용하도록 연결.
- checksum 유지 상태로 batch 경로의 메모리 할당 churn을 추가 완화.

## Commit 214
- localization duplicate conflict Markdown 리포트에 `Canonical (Suggested)` 컬럼을 추가해 충돌 키 통합 기준 파일을 자동 제안.
- 추천 우선순위를 `ui.json > game.json > events.json`으로 적용해 실제 정리 워크플로우 의사결정 시간을 단축.
- 기존 리포트/검증 파이프라인과 호환되며 출력 가독성을 강화.

## Commit 215
- `sim-systems` `PathfindWorkspace`를 generation stamp 기반으로 전환해 쿼리마다 발생하던 배열 전체 `fill` 초기화 비용을 제거.
- A* 본문을 `seen_gen/closed_gen` 판별과 `came_from` sentinel 체인 복원으로 정리해 재사용 워크스페이스의 hot path를 최적화.
- generation 카운터 래핑(`u32::MAX`) 회귀 테스트를 추가하고, checksum/벤치 검증(`migration_verify --with-benches`)을 통과.

## Commit 216
- `sim-bridge` pathfinding에 backend 디스패처 레이어를 추가해 `auto/cpu/gpu` 설정이 실제 실행 경로에 반영되도록 연결.
- GPU 전용 엔트리포인트 스텁(현재 CPU 폴백)과 공통 `normalize_max_steps`를 도입해 GPU 실구현 이전에도 호출 계약을 안정화.
- 디스패처 회귀 테스트(단건/배치)를 추가하고 `migration_verify --with-benches`로 checksum 유지를 검증.

## Commit 217
- `localization_audit`에 `--key-owner-policy-json`을 추가해 duplicate key canonical 소스 제안을 JSON 정책(`owners`)으로 내보낼 수 있게 확장.
- canonical 선택 로직을 공통화해 Markdown 리포트와 정책 JSON의 기준 일관성을 보장.
- `migration_verify`가 `MIGRATION_AUDIT_KEY_OWNER_POLICY` 환경변수로 key-owner 정책 아티팩트를 생성하도록 연동.

## Commit 218
- `localization_compile`에 key-owner 정책 로딩/적용을 추가해 key별 canonical category를 compile 단계에서 강제할 수 있게 확장.
- manifest에 `key_owners_path`, `max_owner_rule_miss_count`를 도입하고 `localization/key_owners.json`(248 keys) 정책 파일을 연결.
- owner miss 회귀 게이트를 추가하고 전체 검증(`migration_verify --with-benches`)에서 `owner_misses=0` 및 checksum 유지를 확인.

## Commit 219
- `localization_audit`에 key-owner 정책 비교 모드(`--compare-key-owner-policy`)를 추가해 generated owners와 저장 정책 간 drift를 자동 검출.
- 비교 결과를 `missing/extra/changed`로 출력하고 불일치 시 실패하도록 해 정책 동기화 게이트를 강화.
- `migration_verify`에 `MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY` 연동을 추가해 검증 파이프라인에서 owner 정책 일관성을 강제.

## Commit 220
- `sim-bridge` pathfinding backend 제어 로직을 `pathfinding_backend.rs`로 분리해 모드 저장/파싱/해석 책임을 모듈화.
- 브리지 API/dispatch 경로는 유지하면서 backend 상태 접근을 모듈 getter/setter로 통일.
- GPU 실구현을 위한 구조적 확장 지점을 정리하고 전체 검증(`migration_verify --with-benches`)으로 checksum 유지 확인.

## Commit 221
- `localization_audit`에 `--compare-key-owner-policy-auto`를 추가해 manifest의 `key_owners_path`를 기준으로 owner 정책 drift를 자동 비교.
- `migration_verify` 기본 audit 단계에 auto compare를 포함시켜 owner 정책 일관성을 기본 게이트로 승격.
- 필요 시 `MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY`로 명시 대상 비교를 override 가능하게 유지.

## Commit 222
- `sim-bridge` GPU placeholder pathfinding 엔트리포인트를 `pathfinding_gpu.rs`로 분리해 GPU 구현 확장 지점을 모듈화.
- 기존 dispatch 인터페이스/동작은 유지하고 CPU fallback semantics를 동일하게 보장.
- 전체 검증(`migration_verify --with-benches`)으로 checksum 유지 확인.

## Commit 223
- pathfinding backend 레이어에 CPU/GPU dispatch 카운터를 추가하고 dispatch 경로에서 resolved backend 기준으로 계측.
- `WorldSimBridge`에 `get_pathfinding_backend_stats`/`reset_pathfinding_backend_stats`를 추가해 런타임 관측성을 강화.
- 병렬 테스트 환경에서도 안정적인 델타 기반 회귀 테스트를 추가하고 전체 검증(`migration_verify --with-benches`) 통과.

## Commit 224
- `localization_compile` 산출물 meta에 key-owner 정책 경로/적용 통계(seen/hit/miss/override)를 추가.
- `compiled/en.json`, `compiled/ko.json`에 신규 meta 필드를 반영해 아티팩트 자체의 관측성을 강화.
- 전체 검증(`migration_verify --with-benches`) 통과로 checksum 유지 확인.

## Commit 225
- `sim-bridge`에 backend dispatch 경유 공개 batch API(`pathfind_grid_batch_dispatch_bytes`, `pathfind_grid_batch_xy_dispatch_bytes`)를 추가.
- `sim-test` pathfinding 벤치가 dispatch 경로를 사용하고 backend dispatch 카운터(cpu/gpu/total)를 함께 출력하도록 연동.
- checksum 유지 상태에서 벤치 관측성(backend 사용량)까지 확장.

## Commit 226
- `sim-bridge`에 non-Godot backend 모드 제어/조회 공개 API(`set/get/resolve_pathfind_backend_mode`)를 추가.
- `sim-test` pathfinding 벤치에 `--backend auto|cpu|gpu` 옵션을 연결해 모드별 실행/관측을 지원.
- 기본 검증 파이프라인에서는 기존 checksum을 유지하면서 backend 관측 출력을 확장.

## Commit 227
- `migration_verify` 벤치 단계에 `MIGRATION_BENCH_PATH_BACKEND`(auto/cpu/gpu)를 추가해 pathfinding backend 모드 제어를 지원.
- pathfinding/split 벤치 호출에 `--backend` 인자를 전달하고 입력값 유효성 검사를 도입.
- `auto`, `cpu` 모드 모두에서 checksum 기준선 유지 확인.

## Commit 228
- `sim-bridge` 공개 backend helper API(`set/get/resolve`, dispatch counter, dispatch batch API)에 대한 단위 테스트 2개를 추가.
- mode roundtrip/invalid rejection과 counter 증가를 검증해 API 계약을 테스트로 고정.
- 전체 검증(`migration_verify --with-benches`) 통과로 checksum 유지 확인.

## Commit 229
- `localization_audit`에 `--refresh-key-owner-policy-auto`를 추가해 manifest의 `key_owners_path`로 owner 정책 자동 갱신을 지원.
- `migration_verify`에 `MIGRATION_AUDIT_REFRESH_KEY_OWNER_POLICY` 환경변수를 추가해 필요 시 검증 중 정책 동기화를 수행.
- 기본 게이트(비갱신 compare) 호환성을 유지하면서 로컬 운영 편의성을 확장.

## Commit 230
- `migration_verify`에 `MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND`를 추가해 pathfinding 벤치 출력의 resolved backend(cpu/gpu)를 기대값으로 강제 검증.
- 일반/split pathfinding 벤치 모두에서 resolved 파싱/불일치 실패 로직을 추가.
- checksum 기준선은 유지하면서 backend 해석 검증 정확도를 강화.

## Commit 231
- `localization_compile`에 owner policy 품질 지표(duplicate coverage missing, unused owner keys)를 추가하고 manifest 임계치 게이트로 회귀 차단.
- 컴파일 로그/compiled meta에 owner policy 요약 정보를 노출해 운영 관측성을 강화.
- `manifest`에 `max_owner_unused_count=0`, `max_duplicate_owner_missing_count=0`을 설정해 정책 정합성을 기본 강제.

## Commit 232
- `localization_audit` 리포트에 owner policy 품질 지표(entry count, duplicate coverage miss, unused keys)를 추가.
- strict audit 출력에서 owner policy 상태를 함께 노출해 검증 가시성을 강화.
- 전체 검증(`migration_verify --with-benches`)에서 owner 정책 지표와 checksum 유지를 확인.

## Commit 233
- `migration_verify` pathfinding 벤치 검증에 dispatch total 파싱/기대값 검증을 추가해 checksum + 실행 경로 계측을 함께 게이트화.
- 일반 벤치는 `total=iters*2`, split 벤치는 tuple/xy 각각 `total=iters`를 강제.
- split + resolved backend 검증 조합으로 전체 파이프라인 통과를 확인.

## Commit 234
- `sim-test`에 `--bench-pathfind-backend-smoke`를 추가해 `auto/cpu/gpu` 3모드의 pathfinding dispatch 동작을 동일 입력으로 연속 검증.
- `migration_verify`에 `MIGRATION_BENCH_PATH_BACKEND_SMOKE`/`MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS`를 추가하고, smoke 출력 파싱으로 모드 존재/체크섬 일치/dispatch total/CPU mode resolved를 강제.
- `MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 tools/migration_verify.sh --with-benches` 기준으로 smoke + 기존 checksum 게이트 동시 통과를 확인.

## Commit 235
- `migration_verify` smoke 검증에 `MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED`/`MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED`를 추가해 mode별 resolved 기대값을 명시적으로 강제할 수 있게 확장.
- smoke 파서가 `resolved_auto/resolved_gpu`를 함께 추출하고, 기대값 지정 시 불일치 즉시 실패하도록 게이트를 강화.
- CPU fallback 환경에서 `auto=cpu`, `gpu=cpu` 기대값을 적용한 전체 검증(`tools/migration_verify.sh --with-benches`) 통과를 확인.

## Commit 236
- `migration_verify` smoke 파서를 모드 라인 직접 매칭(`auto/cpu/gpu`)으로 전환해 출력 순서 의존성을 제거.
- mode별 `configured/checksum/total/resolved`를 독립 파싱·검증하고, `configured` 값이 모드와 일치하는지까지 게이트를 확장.
- `MIGRATION_BENCH_PATH_BACKEND_SMOKE=true ... tools/migration_verify.sh --with-benches` 재검증으로 smoke + 기존 checksum 게이트 동시 통과를 확인.

## Commit 237
- `localization_audit`에 owner-policy 전용 Markdown 리포트 생성 옵션(`--owner-policy-markdown`)을 추가.
- 리포트는 owner policy 요약 수치와 누락/미사용 키 목록을 분리해 출력하며, 이슈가 없을 때는 명시적으로 clean 상태를 기록.
- `migration_verify`에 `MIGRATION_AUDIT_OWNER_POLICY_MARKDOWN` 전달 경로를 연결해 검증 파이프라인에서 owner-policy 문서 아티팩트 생성을 자동화.

## Commit 238
- `migration_verify`에 `MIGRATION_AUDIT_REPORT_DIR`를 추가해 audit 산출물 경로를 디렉터리 단위로 일괄 설정 가능하게 확장.
- 개별 경로 env가 비어 있을 때만 기본 파일명(`audit.json`, `duplicate.json`, `duplicate_conflicts.md`, `key_owner_policy.generated.json`, `owner_policy.md`)을 자동 할당.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts tools/migration_verify.sh`로 아티팩트 5종 자동 생성을 검증.

## Commit 239
- `localization_audit` 리포트에 owner-policy 카테고리 분포 지표(`owner_policy_category_count`, `owner_policy_category_counts`)를 추가.
- 콘솔 요약에 `owner_policy_categories`를 포함하고, owner-policy markdown에 `Owner Category Distribution` 표를 추가해 분포 가시성을 강화.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts2 tools/migration_verify.sh`로 신규 지표가 JSON/Markdown 양쪽에 반영됨을 확인.

## Commit 240
- `migration_verify` pathfinding backend smoke 검증에 mode별 `cpu/gpu` dispatch 카운트 파싱을 추가하고 `cpu+gpu=total` 관계를 강제.
- `resolved` 결과와 dispatch 방향 일관성(`resolved=cpu -> gpu=0`, `resolved=gpu -> cpu=0`)을 mode별(`auto/cpu/gpu`)로 검증하도록 게이트를 확장.
- `MIGRATION_BENCH_PATH_BACKEND_SMOKE=true ... tools/migration_verify.sh --with-benches` 재검증으로 checksum/dispatch/resolved 게이트 동시 통과를 확인.

## Commit 241
- `localization_audit`에 `--owner-policy-compare-report-json`을 추가해 owner-policy drift 비교 결과(missing/extra/changed)를 구조화된 JSON으로 출력 가능하게 확장.
- `migration_verify`에 `MIGRATION_AUDIT_OWNER_POLICY_COMPARE_REPORT_JSON` 전달 경로를 추가하고, `MIGRATION_AUDIT_REPORT_DIR` 사용 시 기본 아티팩트 `owner_policy_compare.json` 자동 생성을 연결.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts3 tools/migration_verify.sh`에서 compare JSON 아티팩트 생성/값(0 drift) 확인.

## Commit 242
- `migration_verify`에 `MIGRATION_BENCH_REPORT_JSON`을 추가해 bench 체크 결과(checksum/dispatch/resolved)를 JSON 아티팩트로 저장 가능하게 확장.
- `MIGRATION_AUDIT_REPORT_DIR` 사용 시 `bench_report.json` 기본 경로를 자동 할당해 audit/bench 아티팩트를 단일 디렉터리로 수집 가능.
- `--with-benches` 실행에서 `bench_report.json` 생성과 path/smoke/stress/needs 핵심 지표 기록을 검증.

## Commit 243
- `migration_verify` bench report 출력에서 선택 필드의 빈 문자열 표현을 제거하고 `null`/숫자/문자열 타입을 명확히 정규화.
- split 비활성 시 split checksum/total을 `null`로, smoke total은 숫자로 출력해 후속 파서의 타입 안정성을 개선.
- `python3 -m json.tool` 검증으로 `bench_report.json` 유효성과 타입 반영을 확인.

## Commit 244
- `localization_compile`에 `--report-json`을 추가해 compile 전역/locale 지표를 JSON 아티팩트로 출력 가능하게 확장.
- `migration_verify` compile 단계가 `MIGRATION_COMPILE_REPORT_JSON`을 지원하고, `MIGRATION_AUDIT_REPORT_DIR` 사용 시 `compile_report.json` 자동 생성을 연결.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts6 tools/migration_verify.sh`에서 compile report 생성과 JSON 유효성을 확인.

## Commit 245
- `migration_verify`에 `MIGRATION_VERIFY_REPORT_JSON`을 추가해 실행 옵션과 아티팩트 경로를 단일 메타 JSON으로 출력.
- `MIGRATION_AUDIT_REPORT_DIR` 사용 시 `migration_verify_report.json`을 기본 생성해 report dir 내 산출물 인덱스를 제공.
- verify report는 상대 경로를 절대 경로로 정규화하고, 미사용 아티팩트는 `null`로 표준화해 후처리 파서 호환성을 강화.

## Commit 246
- `sim-bridge` 공개 API에 `has_gpu_pathfind_backend()`를 추가하고, 관련 테스트에서 feature-gated capability 값을 검증.
- `sim-test` pathfind backend smoke 출력에 `has_gpu` 필드를 추가해 capability 상태를 벤치 로그로 노출.
- `migration_verify` smoke 검증이 `has_gpu` 파싱/모드 간 일치/resolve 정책 일관성을 강제하도록 확장하고, `bench_report.json`에 `path_smoke.has_gpu`를 반영.

## Commit 247
- `migration_verify`에 `MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU`를 추가해 smoke capability 기대값(`true|false`)을 명시적으로 강제 검증 가능하게 확장.
- smoke 파서가 mode별 `has_gpu` 일치 + 기대 capability 일치를 검증하고, bench 컨텍스트 로그에 `smoke_expect_has_gpu`를 노출.
- `bench_report.json`에 `path_smoke_expect_has_gpu`를 추가해 capability 기대 설정까지 아티팩트에 보존.

## Commit 248
- `migration_verify`에 `MIGRATION_VERIFY_ASSERT_ARTIFACTS`를 추가해 report 아티팩트 파일 존재를 검증 단계에서 강제 가능하게 확장.
- `assert=true`일 때 compile/audit 아티팩트(및 `WITH_BENCHES=true` 시 bench report) 존재를 체크하고 누락 시 즉시 실패.
- `migration_verify_report.json`에 `assert_artifacts` 메타 필드를 추가해 실행 시점의 아티팩트 강제 여부를 기록.

## Commit 249
- `localization_compile` summary report에 `schema_version`, `generated_at_utc` 메타를 추가해 스키마 버전 관리와 실행 시각 추적을 지원.
- `migration_verify`가 생성하는 `bench_report.json`, `migration_verify_report.json`에도 `schema_version` 메타를 추가해 아티팩트 간 형식 일관성을 강화.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts11 ... tools/migration_verify.sh --with-benches` 실행에서 compile/bench/verify 리포트의 메타 필드 존재를 확인.

## Commit 250
- `migration_verify_report.json`에 git 실행 컨텍스트(`git_branch`, `git_head`, `git_dirty`)를 추가해 검증 결과를 코드 상태와 직접 연결.
- git 정보 조회 실패 시 null 안전 처리로 보고서 생성 안정성을 유지.
- `/tmp/worldsim_audit_artifacts12/migration_verify_report.json`에서 branch/SHA/dirty 필드 반영을 확인.

## Commit 251
- `localization_compile` summary report에 `thresholds` 객체를 추가해 manifest 기반 품질 게이트 임계치들을 JSON 아티팩트에 함께 기록.
- compile 결과 지표와 임계치를 동일 리포트에서 비교할 수 있어 검증 기준 추적성과 운영 가시성을 강화.
- `/tmp/worldsim_audit_artifacts13/compile_report.json`에서 `thresholds` 6개 필드 반영을 확인.

## Commit 252
- `localization_audit` 출력 JSON(`audit.json`, `duplicate.json`)에 `schema_version`, `generated_at_utc` 메타를 추가해 아티팩트 버전/생성시각 추적을 지원.
- `run_audit` 결과와 duplicate report export payload를 함께 확장해 두 출력 간 메타 일관성을 유지.
- `/tmp/worldsim_audit_artifacts14/audit.json`, `/tmp/worldsim_audit_artifacts14/duplicate.json`에서 메타 필드 반영 확인.

## Commit 253
- `migration_verify_report.json`에 `artifact_sha256` 객체를 추가해 compile/audit/bench 산출물의 SHA-256 무결성 해시를 함께 기록.
- `shasum`(macOS)과 `sha256sum`(Linux) 양쪽을 지원하며, 파일이 없는 항목은 `null`로 표준화.
- `/tmp/worldsim_audit_artifacts15/migration_verify_report.json`에서 non-bench 실행 기준 아티팩트 해시 채움 + bench 해시 null을 확인.

## Commit 254
- `migration_verify_report.json`에 `artifact_size_bytes` 객체를 추가해 compile/audit/bench 산출물의 파일 크기(bytes)를 함께 기록.
- 파일이 없는 항목은 `null`, 존재하는 항목은 정수 바이트 크기로 출력해 파서 일관성을 유지.
- `/tmp/worldsim_audit_artifacts16/migration_verify_report.json`에서 artifact size 필드 반영과 non-bench 기준 bench size null을 확인.

## Commit 255
- `migration_verify`가 전체 실행 시간과 단계별 소요 시간(초)을 측정해 `migration_verify_report.json`의 `total_duration_seconds`, `timings_seconds`로 기록하도록 확장.
- 단계별 메트릭은 `rust_tests`, `data_localization_extract`, `localization_compile`, `localization_audit`, `rust_bench`(선택)를 포함.
- `/tmp/worldsim_audit_artifacts17/migration_verify_report.json`에서 시간 메트릭 필드 반영을 확인.

## Commit 256
- `migration_verify_report.json`에 실행 설정 스냅샷 `config`를 추가해 audit/bench 옵션 조합을 구조화된 형태로 함께 기록.
- bool/int/string/null 변환 헬퍼를 도입해 `WITH_BENCHES` on/off 모두에서 안전하게 설정값을 직렬화.
- `/tmp/worldsim_audit_artifacts18/migration_verify_report.json`에서 bench 관련 기대값(`expected_resolved_backend`, `path_backend_smoke_expect_*`)을 포함한 config 필드 반영을 확인.
