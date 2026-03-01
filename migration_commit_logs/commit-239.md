# Commit 239 - Owner-policy 카테고리 분포 지표 추가

## 커밋 요약
- `localization_audit` 리포트에 owner-policy 카테고리 분포(`category -> key count`)를 추가해, 정책 확장 시 카테고리 편중/누락을 빠르게 확인할 수 있도록 개선.

## 상세 변경
- `tools/localization_audit.py`
  - `run_audit`에서 owner policy map 기반 카테고리별 카운트 집계 추가.
  - 리포트 필드 확장:
    - `owner_policy_category_count`
    - `owner_policy_category_counts`
  - 콘솔 출력 확장:
    - `owner_policy_categories` 요약 라인 추가.
  - owner-policy markdown 출력 확장:
    - `Owner Category Distribution` 표 섹션 추가(`Category`, `Keys`).
    - 요약 헤더에 category count 추가.

## 기능 영향
- owner 정책이 특정 카테고리에 과도하게 몰리는지 조기 관찰 가능.
- 대규모 키 추가/정리 작업 시 정책 분포를 문서 기반으로 비교하기 쉬워짐.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts2 tools/migration_verify.sh` 통과.
  - 콘솔에 `owner_policy_categories: 2` 출력 확인.
  - `/tmp/worldsim_audit_artifacts2/audit.json`에
    - `owner_policy_category_count`
    - `owner_policy_category_counts`
    필드 생성 확인.
  - `/tmp/worldsim_audit_artifacts2/owner_policy.md`에 `Owner Category Distribution` 표 생성 확인.
