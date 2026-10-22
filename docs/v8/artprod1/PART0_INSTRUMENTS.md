# Part 0 계측 검증

## 패널 judge

`art/pipeline5/judge_panels.py`는 소유권 추출 직후 원본 해상도 인물에 alpha>0 crop + 축별 8% 투명 여백을 적용한다. `person` family와 judge v2 calibration SHA `7218ac77979065ec6398c77cb6156bce5bd523979ed48732945f2ab642a5d5fd`를 고정한다. PROD1 수락은 정확히 10패널 모두 numeric_pass일 때만 가능하다. 모델 점수와 numeric 판정은 별개다.

교정: 원본 I-1 try2 단일 인물 1/1 통과. 사전 지정한 같은 룩 최초 5자세 시트 4/5 통과, work_a stroke_q90 실패. Part0 원본은 실제 단일 인물이므로 목표 문서에 기술된 5패널 교정은 역사 입력과 불일치한다. 수치를 조정하지 않았다. 근거: 외부 `part0-panel-calibration/preregistration.json`, `results.json`.

## 구조와 캔버스

일반 80×112, 수면 112×80. 접지선104/72 ±2; 정상화는 정확히 접지선에 고정한다. 단일 연결성, 보이는 픽셀의 캔버스 경계 비접촉, idle 높이 성인52–58/아동37–43, 수면 폭48–60을 검사한다. 구멍은 정상 음영 공간이므로 실패 항목에서 제거했다. 자세 전체는 S idle의 단일 배율을 공유한다.

수면 방향과 머리 끝 질량은 alpha≥128 경계 및 바깥30%로 판단한다. 전체 RGBA를90/180/270도 회전해 보이는 픽셀을 보존하며 sidecar에 회전을 기록한다. 희미한 alpha 꼬리로 방향이 뒤집히던 실제 오류의 RED→GREEN 회귀 테스트를 추가했다. dust는 기존 소유권 규칙을 유지하며 삭제하지 않는다.

2026-10-23 독립 재실행: `uv run --with pillow==12.2.0 --with numpy --with scipy --with scikit-image --with pytest python -m pytest art/pipeline5/test_judge_panels.py art/pipeline5/test_whole_figures.py art/whole_figure_lab/test_prod1.py -q` → **34 passed**. 이 증거는 합성 계측·인터페이스 검증이며 새 그림의 품질 승인이 아니다.

## 랩

`art.whole_figure_lab.prod1`은 수락된10프레임만 읽는다.112×112 표시 셀 안에 정규화 캔버스를 접지선104로 배치해112px 수면 프레임도 자르지 않는다.28×8 그림자를 랩에서 그린다. 최소12룩, 여성3, 아동2, 상의/머리3종을 충족하지 못하면 생성 전에 실패한다.

전체 `art/whole_figure_lab` typecheck:0 errors/0 warnings. 합성 브라우저 증거는 `/Users/rexxa/work/artprod1-data/lab-synthetic-test/`에 있고 실제 생성물의 리뷰나 FPS로 계산하지 않는다. 실제 랩·새 리뷰어·60초200명 측정은 생산 이후 진행한다.

## 재집계

최종 결과는 `TRIAL5_RECOUNT.md` 및 JSON에 기록한다. 정보용 재측정이며 TRIAL5 파일은 승격하지 않는다.
