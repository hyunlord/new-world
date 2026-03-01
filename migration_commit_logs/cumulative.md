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
