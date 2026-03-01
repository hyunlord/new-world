# Commit 258 - verify 리포트에 툴체인 메타데이터 추가

## 커밋 요약
- `migration_verify_report.json`에 실행 환경의 핵심 툴체인 버전(`python3`, `cargo`, `rustc`)을 기록하도록 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - verify report 생성 시 툴 버전 캡처를 추가:
    - `python3 --version`
    - `cargo --version`
    - `rustc --version`
  - report 루트에 `toolchain` 객체 추가:
    - `toolchain.python3`
    - `toolchain.cargo`
    - `toolchain.rustc`
  - 각 필드는 명령 실패/미존재 시 `null`로 안전하게 직렬화.

## 기능 영향
- 검증 리포트만으로 실행에 사용된 Python/Rust 툴체인 버전을 추적 가능.
- CI/로컬 환경 간 결과 차이 분석 시 환경 요인 식별이 쉬워짐.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts20 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts20/migration_verify_report.json`에서:
  - `toolchain.python3`
  - `toolchain.cargo`
  - `toolchain.rustc`
  필드 존재 및 값 출력 확인.
