# Commit 259 - verify 리포트에 호스트 메타데이터 추가

## 커밋 요약
- `migration_verify_report.json`에 실행 호스트 정보(`os`, `kernel_release`, `arch`, `cpu_count`)를 추가해 결과 재현성 추적을 강화.

## 상세 변경
- `tools/migration_verify.sh`
  - verify report 생성 시 호스트 정보 수집 추가:
    - `uname -s` -> `host.os`
    - `uname -r` -> `host.kernel_release`
    - `uname -m` -> `host.arch`
    - CPU count 탐지(`getconf _NPROCESSORS_ONLN` -> `nproc` -> `sysctl -n hw.ncpu`) -> `host.cpu_count`
  - 수집 실패 시 `null`로 직렬화되도록 기존 JSON 변환 헬퍼 재사용.
  - report 루트에 `host` 객체 추가.

## 기능 영향
- 동일 커밋이라도 실행 머신 차이(아키텍처/코어 수/커널)로 발생하는 벤치 편차를 report만으로 빠르게 구분 가능.
- CI/로컬 결과 비교 시 환경 컨텍스트 확인 비용 감소.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts21 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts21/migration_verify_report.json`에서:
  - `host.os=Darwin`
  - `host.kernel_release=25.0.0`
  - `host.arch=arm64`
  - `host.cpu_count=14`
  반영 확인.
