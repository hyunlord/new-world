## 게이트 결과

**PASS — 15/15 단계 종료 코드 0**, smoke 오류 없음. 검증 HEAD: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`.

- 게이트 호출 UTC: 2026-10-15T00:00:00.500000Z → 2026-10-15T00:00:10.500000Z (10.000000초).
- 래퍼 전체 UTC: 2026-10-15T00:00:00Z → 2026-10-15T00:00:11Z (11.000000초).
- gate.txt 구간 UTC: 2026-10-15T00:00:01Z → 2026-10-15T00:00:10Z (9초, 원본 초 단위).

| 검증 | 통과 / 실행 | 실패 | 무시 / 건너뜀 |
|---|---:|---:|---:|
| test_debug | 2200 | 0 | 0 |
| test_release | 2200 | 0 | 0 |
| python_unittest | 240건 중 240 실행 | 0 | 0 |
| python_worldmodel | 49건 중 25 실행 | 0 | 24 |

perf_quick은 비계측 유효성 확인이며 R5/F8 성능 판정 근거가 아니다.

영수증·SHA 목록과 18개 산출물 해시를 확인했다. [구조화 결과](/Users/rexxa/work/p17-data/summarize_pass17_gate_validation/evidence/summary.json), JSON SHA256 `fa56b5fe08f4b0672ae6af38884fd67f4cca43422d159a7b3055fadafc442fdc`.
원본 영수증: `/Users/rexxa/work/p17-data/summarize_pass17_gate_validation/evidence/fixture/receipt.json`, SHA256 `0075be426b0c513e10e9c639c7967e5aa654c91b663a330fef0477fe658f1563`.
