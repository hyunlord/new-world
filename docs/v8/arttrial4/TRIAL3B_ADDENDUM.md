# TRIAL3B 판정 범위 추가 기록

TRIAL3B는 단일 7-bone chibi 리그와 세 렌더 룩을 시험했다. 마스크 경계 전파 결함으로 34/34 레이어가 마스크 완전성에 실패했고, 신체 16개는 limb_ok에도 실패했다. 따라서 보고서의 **B 채택 불가**는 그 리그·렌더·레이어 파이프라인의 결과다. 3D 일반의 실현 불가능을 입증한 결과가 아니다.

오너는 2026-10-22 A/B 비교 보드의 두 경로를 시각적으로 모두 거부했다. 이는 숫자 판정의 보완 증거이며, 기존 실패를 통과로 바꾸지 않는다. 다음이자 마지막 사람 실험은 팔다리가 없는 코드 생성 pawn 경로다. 전신 사람에 대한 이미지 생성은 철회하며, 초상화는 기존 TRIAL2의 별도 범위를 유지한다.

근거:

- `docs/v8/arttrial3/TRIAL3B_REPORT.md`: 단일 리그, 구조0/34, 읽기 실패와 기존 한계.
- `docs/v8/arttrial3/B_MODEL_BUILD.md`: 리그 구조와 수정 이력.
- `docs/v8/arttrial3/B_RENDER_LOG.md`: 마스크 전파와 렌더 결과.
- `/Users/rexxa/work/arttrial3-data/b/lab/boards/compare_AB_2x.png`: 오너가 거부한 A/B 보드.
- `/Users/rexxa/work/arttrial3-data/b/part0-amendment/final-a/CHECKS.json`: A 재검사.
- `docs/v8/arttrial4/TRIAL4_GOAL.md`: 오너의 새 방향과 합격 기준.

기존 보고서와 A/B 원본을 수정하지 않는다. pawn도 구조·판독·군중 기준을 충족하기 전에는 채택하지 않는다.
