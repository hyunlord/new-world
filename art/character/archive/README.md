# 캐릭터 역사 자료 아카이브

2026-10-23 planner/owner의 route I 채택 결정에 따라 이동했다. 현재 표준은
[TEMPLATE_V3](../TEMPLATE_V3.md), 자산 원천은 [route_i/MANIFEST.json](../route_i/MANIFEST.json)이다.
여기의 실패·후보·리뷰 이미지는 현재 자산으로 섞지 않는다. 이동 전후 파일 내용은 SHA-256으로 비교했다.

| 이전 위치 (`art/character/` 기준) | 보존 위치 (이 README 기준) | 실험·보고서 |
|---|---|---|
| route_p/ | [route_p/](route_p/) | TRIAL4 절차적 pawn · [TRIAL4_REPORT](../../../docs/v8/arttrial4/TRIAL4_REPORT.md) |
| route_a/ | [route_a/](route_a/) | TRIAL3 Route A · [TRIAL3_REPORT](../../../docs/v8/arttrial3/TRIAL3_REPORT.md) |
| route_b/ | 없음 | 기준 트리에 부재, 이동·생성하지 않음 · [TRIAL3B_REPORT](../../../docs/v8/arttrial3/TRIAL3B_REPORT.md) |
| candidates/ | [candidates/](candidates/) | TRIAL5 Part0 · [TRIAL5_PART0_REPORT](../../../docs/v8/arttrial5/TRIAL5_PART0_REPORT.md) |
| trial5_diagnostics/ | [trial5_diagnostics/](trial5_diagnostics/) | TRIAL5 · [TRIAL5_REPORT](../../../docs/v8/arttrial5/TRIAL5_REPORT.md) |
| route_i_lab/prod2/ | [route_i_lab/prod2/](route_i_lab/prod2/) | ART-PROD2 · [PROD2_REPORT](../../../docs/v8/artprod2/PROD2_REPORT.md) |

`route_i_lab/prod*`에 존재했던 유일한 디렉터리는 prod2다. `art/whole_figure_lab/`의 랩 도구는 그대로 유지한다.
PROD2 랩 도구는 route와 출력 디렉터리를 인자로 받는다. 새 자산 입력은 route_i를 사용하며, 보존 리뷰는 위 archive 경로에서 읽는다.

sidecar·manifest·과거 보고서의 옛 절대/상대 경로는 당시 provenance이므로 바이트를 고치지 않았다.
역사 자료를 열 때 위 이전→보존 위치 표로 해석한다. 옛 route 경로가 하드코딩된
TRIAL3/4/5 생산 도구는 역사 재현용이며 현재 생산 진입점이 아니다.
원시 생성·리뷰 증거의 외부 `/Users/rexxa/work/artprod*-data/` 경로는 그대로다.
결정·fallback·검증은 [PROD3_REPORT](../../../docs/v8/artprod3/PROD3_REPORT.md)와
[HANDOVER](../../../docs/v8/artprod3/HANDOVER.md)를 따른다.
