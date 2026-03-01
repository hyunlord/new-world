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
