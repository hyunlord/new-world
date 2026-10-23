# ART-PROD3 독립 코드·증거 검토

판정: 검토 범위에서 차단 결함 없음. **자산 채택은 실패**다. 재절단 결과는 전체 34/36, 성인 16/18, 어린이 18/18이며 전체 채택과 성인 전용 채택 모두 성인 최소 17/18에 미달한다. 따라서 이 패스에서 역사 route 아카이브 이동이나 정식 자산 소스 채택을 해서는 안 된다. 원격 push·최종 보고서 정합성은 이 검토 뒤 리더가 별도 확인한다.

## 검토한 계약과 구현

`PROD3_GOAL.md` 전문, `sheet_geometry.py`, `whole_checks.py`, `prod3_test_visibility.py`, `prod3_reslice.py`, 기존 `prod1_process.py`의 evaluate/sidecar 흐름과 `prod1_boards.py`를 읽었다.

- 정규화는 upright/sleep 모두 alpha≥16 bbox로 fit과 발 기준선을 결정한다. 검사기는 같은 기준으로 크기 bands·contact·canvas edge를 판단한다. 원본 인체 배율 산정과 연결성의 alpha≥128 기준은 별도 지표로 유지된다.
- 정규화된 전체 이미지를 mask 없는 paste로 배치하여 캔버스 안 RGBA를 보존한다. 캔버스 밖 비투명 픽셀 수와 최대 alpha를 프레임마다 기록한다. alpha≥16이 밖으로 나가는 배치는 사전에 실패한다.
- 결손 10룩의 원본 선택은 PROD2_REPORT의 최종 best 표와 일치한다. 9룩 old1, `child_F_fur_cloak_bun`만 new1이다. evaluate를 직접 호출하며 생성이나 reserve 경로를 실행하지 않는다.
- 신규 publish는 기존 결손 중 structure와 numeric panel judge가 모두 통과한 룩에만 적용한다. 기존 26룩은 외부 비교 디렉터리에서 재평가하고 원래 자산 트리를 교체하지 않는다.
- 전체≥32 및 성인≥17이면 전체 채택, 전체가 부족하고 성인≥17이면 성인 전용 채택이라는 분기가 구현되어 있다. 실제 결과에서는 두 분기 모두 false다.

## 독립 실행으로 확인한 증거

`python3 -m pytest art/pipeline5/prod3_test_visibility.py -q`: **16 passed**. 6px·alpha 3/12/15 halo의 정상 crop, alpha 16/128/255 실제 넘침 거절, 기준선/bands의 alpha16 사용, 내부 RGBA 보존과 discarded accounting을 포함한다.

`RESLICE_RESULTS.json`과 외부 `ACCEPTED_BEFORE.json`/`SOURCES_BEFORE.json`을 읽고 현재 파일 SHA-256을 다시 계산했다.

| 항목 | 독립 확인 |
|---|---|
| 보존 원본 raw/prompt | 72개 파일 해시 일치 |
| 기존 수락 26룩 원본 트리 | 546개 파일 해시 및 파일 집합 일치 |
| 평가·정규화 기록 | 36룩×10프레임=360개 |
| 외부 crop | 결손10룩의 11프레임, 총112픽셀, 최대 alpha1 |
| 기존26룩 재평가 crop | 260프레임 모두 discarded=0 |
| 기존26룩 비교 PNG | 202/260 바이트 차이, 26룩 모두 차이 포함 |
| 기존 자산 자체 새 구조 검사 | 260/260 통과 |
| 기존26룩 재평가 | 26/26 룩 통과 |

기존 파일이 그대로라는 결과와 재평가 PNG가 동일하다는 결과는 다르다. 기존 파일은 보존됐지만 정규화의 bbox·기준선 변경으로 재평가 PNG는 202개가 다르므로 “26룩 재평가 바이트 동일”이라는 주장은 금지한다. 목표가 허용한 diff 목록이 `RESLICE_RESULTS.json.accepted_png_diffs`에 before/after SHA-256과 함께 있다.

## 남은 결손과 판정 제한

- `adult_M_fur_cloak_long_tied`: E/work_a numeric panel judge 실패. 9/10.
- `adult_M_hide_wrap_long_tied`: E/idle의 alpha≥16 높이60px가 성인 band [52,58]을 초과. 9/10.

나머지 8룩은 10/10으로 추가 수락됐다. 성인 부족을 어린이 전용 부족으로 해석하거나 band를 늘려 채택을 만들면 원 계약 위반이다. 이 두 결손과 fallback을 남기고 신규 생성 없이 미채택 보고하는 것이 현재 증거에 맞는다. PROD2 R1/P1/D1 인용은 이번 패스의 신규 재측정이나 게임 통합 검증을 의미하지 않는다.
