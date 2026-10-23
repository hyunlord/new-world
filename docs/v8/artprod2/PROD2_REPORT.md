# ART-PROD2 보고서 — S1 미달로 미채택, R1·P1 통과

**판정: S1 FAIL, route I 미채택.** 현재 26/36룩, 성인13/18·어린이13/18이다. 성인 기준도 실패하므로 “어린이만 실패하면 성인 세트 채택” 예외에 해당하지 않는다. R1은 두 신규 리뷰어 모두 통과했고 P1도524.831FPS로 통과했다. 기존 route_p/route_a/route_b/candidates/trial5_diagnostics를 archive로 이동하지 않는다. TEMPLATE_V3와 route_i는 채택 후보이지 승인된 게임 표준으로 선언하지 않는다.

기준 커밋: [`eb428ce232df77c720eb1cd756e43a891d9dd8a9`](https://github.com/hyunlord/new-world/commit/eb428ce232df77c720eb1cd756e43a891d9dd8a9), 브랜치 `v8/art-prod1`. 발행 대상: [v8/art-prod1](https://github.com/hyunlord/new-world/tree/v8/art-prod1). 종료 시 로컬 HEAD와 원격 ref 일치를 검증하며 정확한 SHA는 외부 `PUBLICATION.json` 및 최종 응답에 기록한다. 범위는 `art/**`, `docs/v8/artprod2/**`; 게임 코드 변경·merge 없음.

| 게이트 | 기준 | 현재 증거 | 판정 |
|---|---|---|---|
| S1 | 전체≥32/36, 성인≥17/18 | 26/36, 성인13/18, 어린이13/18 | FAIL |
| S2 | 새 생성≤40, 룩당≤2 | 최종27/40회, 룩당최대2회; 리뷰 교정0회 | PASS |
| R1 | 두 신규 리뷰어 모두 양 배율의 모든 기준 | 라운드0 A/B,1×/2× 모두 기준 충족 | PASS |
| P1 | Godot 실제 창200명1×, vsync off,60초,≥60FPS | 31,490프레임/60.000226초=524.831FPS | PASS |
| D1 | 두 실행 재절단 동일성 | 보존96+신규27=123개 원본, PNG 바이트 동일; JSON 경로만 정규화 | PASS |

여기서 게이트 D1은 재현성이며, 결정 기록 D1은 아래의 sex/age 채점 변경이다. 두 항목을 혼동하지 않는다.

## 규칙 수정과 전수 재절단

sleep만 alpha≥128 bbox의 긴 축을 adult56px/child40px에 맞추는 독립 배율을 쓴다. idle/walk_a/work_a/carry는 S idle 공통 배율을 유지한다. adult sleep 길이52–60·높이≤36, child37–43·높이≤28, 캔버스112×80·최하단y72·anchor x56이다. 머리 왼쪽 회전 규칙과 전체 RGBA 보존을 유지했다. standing_scale과 방향별 sleep_scale을 sidecar/manifest에 기록한다. 픽셀 지우기·덧칠·구멍 메우기는 수행하지 않는다.

보존96원본 전체를 두 번 처리했다. PROD1의 수락13개에서 재절단 수락22개(성인11·어린이11)가 되었다. 이전 수락 `adult_F_loincloth_long_tied`, `adult_M_hide_wrap_long_tied`는 새 규칙에서 실패해 과거 파일을 외부에 해시 검증 보존하고 수락을 철회했다. 전자는 신규 생성1회로 다시 통과했고 후자는 현재도 MISSING이다. 규칙을 고쳤다고 모든 수면 실패가 사라지지는 않았다. 불투명 몸통은 맞아도 낮은 alpha의 흔적이 전체 보존 bbox를 늘려 캔버스를 넘는 사례와 E idle 높이·panel judge 실패를 그대로 남겼다.

재절단220프레임 감사는 통과했지만 당시48개 look/facing 표본은44개뿐이어서 PART0 전체 감사는 표본 수 미달이었다. 추가 생성 후 독립 `GAP_SET_AUDIT.json`은26룩260프레임과48표본, findings0으로 통과했다. 이는 구조·수치 panel judge 감사이며 R1 가독성 통과를 뜻하지 않는다.

근거: [PART0_RESULTS.json](PART0_RESULTS.json), [D1_RESULTS.json](D1_RESULTS.json), `/Users/rexxa/work/artprod2-data/PART0_RESULTS_FULL.json`, `DETERMINISM.json`, `WITHDRAWN_ACCEPTANCE.json`, `GAP_SET_AUDIT.json`. 원시 JSON의 절대 실행 루트만 정규화했으며 PNG는 원시 바이트를 비교했다. 따라서 JSON 파일 전체를 원시 바이트 동일이라고 주장하지 않는다.

## 36룩 전후·생성·잔여 실패

`oldN`은 보존 PROD1 시도 N을 이번 규칙으로 재절단한 결과이고 `newN`은 PROD2 생성 N이다. 각 단계는 통과 패널 수 최대, 동률 최초 시도다. `구조`와 `judge`는 각각10패널 중 통과 수이며 최종 수락에는 동일 시도의10개 모두가 두 검사를 통과해야 한다. 최종 best는 재절단과 신규를 합친 최대값이다. 실패 문구의 캔버스 bbox는 원문을 유지하고 상세 scaled_size/trim_box는 연결된 JSON에 보존했다.

| 룩 | 수정 전 best | 재절단 best | 새 호출 | 현재 best/수락 | best의 실패 |
|---|---|---|---:|---|---|
| `adult_F_fur_cloak_bun` | 9/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | PASS 10/10 (new2) | 없음 |
| `adult_F_fur_cloak_long_tied` | 10/10 (old2) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `adult_F_fur_cloak_short` | 10/10 (old3) | 10/10 (old3; 구조10, judge10) | 0 | PASS 10/10 (old3) | 없음 |
| `adult_F_hide_wrap_bun` | 9/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (60, 65) offset (26, 36) would clip canvas (112, 80) |
| `adult_F_hide_wrap_long_tied` | 10/10 (old2) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `adult_F_hide_wrap_short` | 10/10 (old2) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `adult_F_loincloth_bun` | 10/10 (old3) | 10/10 (old3; 구조10, judge10) | 0 | PASS 10/10 (old3) | 없음 |
| `adult_F_loincloth_long_tied` | 10/10 (old2) | 9/10 (old1; 구조9, judge10) | 1 | PASS 10/10 (new1) | 없음 |
| `adult_F_loincloth_short` | 10/10 (old2) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `adult_M_fur_cloak_bun` | 10/10 (old2) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `adult_M_fur_cloak_long_tied` | 9/10 (old1) | 9/10 (old1; 구조10, judge9) | 2 | MISSING 9/10 (old1) | E/work_a: panel judge FAIL |
| `adult_M_fur_cloak_short` | 9/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (60, 47) offset (26, 53) would clip canvas (112, 80) |
| `adult_M_hide_wrap_bun` | 10/10 (old1) | 10/10 (old1; 구조10, judge10) | 0 | PASS 10/10 (old1) | 없음 |
| `adult_M_hide_wrap_long_tied` | 10/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (59, 41) offset (26, 54) would clip canvas (112, 80) |
| `adult_M_hide_wrap_short` | 10/10 (old2) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `adult_M_loincloth_bun` | 9/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (62, 71) offset (25, 32) would clip canvas (112, 80) |
| `adult_M_loincloth_long_tied` | 10/10 (old1) | 10/10 (old1; 구조10, judge10) | 0 | PASS 10/10 (old1) | 없음 |
| `adult_M_loincloth_short` | 10/10 (old3) | 10/10 (old3; 구조10, judge10) | 0 | PASS 10/10 (old3) | 없음 |
| `child_F_fur_cloak_bun` | 0/10 (old1) | 0/10 (old1; 구조0, judge0) | 2 | MISSING 9/10 (new1) | S/sleep: normalization: visible extent (45, 39) offset (33, 53) would clip canvas (112, 80) |
| `child_F_fur_cloak_long_tied` | 8/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | PASS 10/10 (new2) | 없음 |
| `child_F_fur_cloak_short` | 9/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (41, 31) offset (35, 60) would clip canvas (112, 80) |
| `child_F_hide_wrap_bun` | 8/10 (old1) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `child_F_hide_wrap_long_tied` | 8/10 (old1) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `child_F_hide_wrap_short` | 8/10 (old1) | 10/10 (old3; 구조10, judge10) | 0 | PASS 10/10 (old3) | 없음 |
| `child_F_loincloth_bun` | 8/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (43, 34) offset (34, 59) would clip canvas (112, 80) |
| `child_F_loincloth_long_tied` | 8/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (46, 35) offset (33, 58) would clip canvas (112, 80) |
| `child_F_loincloth_short` | 8/10 (old1) | 9/10 (old1; 구조9, judge10) | 2 | MISSING 9/10 (old1) | S/sleep: normalization: visible extent (43, 34) offset (34, 59) would clip canvas (112, 80) |
| `child_M_fur_cloak_bun` | 8/10 (old1) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `child_M_fur_cloak_long_tied` | 8/10 (old1) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `child_M_fur_cloak_short` | 8/10 (old1) | 10/10 (old3; 구조10, judge10) | 0 | PASS 10/10 (old3) | 없음 |
| `child_M_hide_wrap_bun` | 8/10 (old1) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `child_M_hide_wrap_long_tied` | 8/10 (old1) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |
| `child_M_hide_wrap_short` | 8/10 (old1) | 10/10 (old3; 구조10, judge10) | 0 | PASS 10/10 (old3) | 없음 |
| `child_M_loincloth_bun` | 8/10 (old1) | 10/10 (old1; 구조10, judge10) | 0 | PASS 10/10 (old1) | 없음 |
| `child_M_loincloth_long_tied` | 7/10 (old1) | 9/10 (old1; 구조10, judge9) | 2 | PASS 10/10 (new2) | 없음 |
| `child_M_loincloth_short` | 8/10 (old1) | 10/10 (old2; 구조10, judge10) | 0 | PASS 10/10 (old2) | 없음 |

## 추가 생성과 예산

재절단 후 결손14룩에만27회를 사용했다. 4개 새 시트가10/10 통과했고23개는 거절되었다. 최종 수락은22+4=26이며10룩이 남았다. `adult_F_loincloth_long_tied`는1회, 나머지13개 대상은각2회다. 신규 통과는 `adult_F_fur_cloak_bun`(new2), `adult_F_loincloth_long_tied`(new1), `child_F_fur_cloak_long_tied`(new2), `child_M_loincloth_long_tied`(new2)다.

각 원본·프롬프트·참조·해시·거절 이유를 [PROD2_GENERATION_LOG.json](PROD2_GENERATION_LOG.json)과 외부 `generation/`, `new-production/`에 보존했다. 40회에서 남은13회를 무조건 쓸 수 있는 것은 아니다. 리뷰 교정도 룩별2회 한도에 포함하며 이미2회 사용한 룩은 재생성할 수 없다. 라운드0 R1이 통과해 교정 생성은0회이며 최종27/40회로 종료했다. 보존96개+신규27개, D1의123원본 범위는 바뀌지 않는다.

## 인증 랩과 리뷰

원본 수락26개만 대상으로 결과·픽셀·리뷰 답변과 독립적인 고정 층화 규칙을 적용했다. 표본은 성인여5·성인남5·어린이여1·어린이남1이며, top은 loincloth5/hide_wrap4/fur_cloak3, hair는 short5/long_tied4/bun3이다. idle3/walk3/work2/carry2/sleep2, S/E, meadow/forest_floor,1×/2×를 포함한다. 리뷰 교정 후에도 ID·속성·활동은 최초 키로 고정한다.

24GIF·120텍스처, 실제 Chromium156 재생24개×24샘플=576PNG를 8Hz 목표 간격으로 캡처했다. 페이지 오류0, 각 시퀀스 마지막 캡처 종료2.893–2.931초다. 익명 패킷은 실제 관찰 contact sheet48장과 manifest/README만 포함하며 키를 포함하지 않는다. 입력 이미지+sidecar240개 및 캡처·패킷 해시를 검증했다. `formal_R1_eligible=true`는 입력·증거 적격성이고 가독성 정답률이 아니다.

- 랩: `/Users/rexxa/work/artprod2-data/lab/round0`
- 리뷰어 전용 패킷: `/Users/rexxa/work/artprod2-data/blind/round0`
- 검증: `/Users/rexxa/work/artprod2-data/lab-round0-evidence-audit.json`
- 고정 private key SHA256: `b2f5ce93a342cf72637eceb2ecab3f3486a32f2e7af88fd1d64a0b1b0adbe3d0`

결정 D1에 따라 sex는 성인만, age는12명 전체를 채점한다. 40px 아동의 성별은 읽을 수 있는 속성으로 요구하지 않는다는 planner의 근거를 그대로 따른다. 본 표본은 `(성인 sex 정답+age 정답)/22≥0.83`, 즉19/22가 필요하다. 활동1×≥8/12·2×≥10/12, item+top+hair는각배율≥30/36이며 두 리뷰어가 모두 만족해야 한다. PROD1의 기존 매핑을 그대로 import하고 미등록·모호한 답은 오답이며 notes로 정답을 보정하지 않는다. 과거 진단 표본 점수를 이번 인증 점수로 재사용하지 않는다.

**R1 PASS:** 두 신규 리뷰어의24개씩 총48개 ID/배율 응답을 기존 매핑으로 채점했고 네 reviewer/scale 조합 모두 기준을 통과했다. 아래 sex 열은 성인만 세므로 원시 전체 sex 정답 수와 구별된다.

| 라운드 | 리뷰어 | 배율 | 활동 | item | top | hair | item+top+hair | 성인sex | age | 성인sex+age | 판정 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | A | 1× | 12/12 | 12/12 | 9/12 | 11/12 | 32/36 | 8/10 | 12/12 | 20/22 | PASS |
| 0 | A | 2× | 12/12 | 12/12 | 9/12 | 11/12 | 32/36 | 8/10 | 12/12 | 20/22 | PASS |
| 0 | B | 1× | 12/12 | 12/12 | 8/12 | 11/12 | 31/36 | 8/10 | 12/12 | 20/22 | PASS |
| 0 | B | 2× | 12/12 | 12/12 | 8/12 | 11/12 | 31/36 | 9/10 | 12/12 | 21/22 | PASS |

| 라운드 | 실행 | 이유 | 추가 생성·리뷰어 |
|---|---|---|---|
| 0 | 완료 | A/B 모두 모든 기준 PASS | 최초 신규 A/B |
| 1 | NOT_RUN / NOT_TRIGGERED | 최초 R1 통과로 실패 교정 조건이 발생하지 않음 | 0회, C/D 미배정 |

개별 오답이 남아도 전체 게이트는 통과했다. 오답만을 이유로 표본을 바꾸거나 추가 동의어를 등록하거나 교정 라운드를 실행하지 않았다. S1의10개 결손도 그대로 보존한다.

원문 요약 A: 48장 전체를 original detail로 관찰했다고 기록했다. 걷기·작업은 부드러운 운동보다 두 자세의 뚜렷한 교대로 보이고 낮아진 도끼는 거의 사라진다. 통나무는 명확하지만 머리·어깨와 겹치며, 수면 자세는 의상·성별 단서를 약화시킨다. 아동 체형은 읽히지만 성별은 불명확하다. 2×는 윤곽 분리를 개선해도 이러한 모호함을 해소하지 않으며 두 지면 모두 실루엣 대비는 유지된다. 원문: “Sleeping collapses garment and sex-presentation cues.”

원문 요약 B: 48장 전체와 두 순서를 관찰했고 보이지 않은 중간 운동을 추정하지 않았다고 기록했다. 도끼를 든 작업은 읽히지만 접촉·후속 동작은 관찰되지 않았고 idle/sleep은 정적이다. 1×의 가슴띠·사선 의복·피부 음영은 밀집되어 보이며 얼굴·머리 구분은 활동·큰 의상보다 약하다. 수면 p1_05 성별은 두 배율 모두 unknown이고 p1_10은2×에서만 남성으로 읽혔다. 원문: “Facial and hair distinctions are less reliable than actions and large garments.”

원시 응답과 주관적 confidence/notes는 그대로 보존했다. confidence는 정답률이나 수용 점수가 아니다. A/B는 원문에서 키·원본·다른 리뷰를 보지 않았다고 각각 명시했다. 채점은 unknown과 미등록 답을 오답으로 유지했다.

- 점수·원문 복사·개별 판정: `/Users/rexxa/work/artprod2-data/reviews/round0/scored/scores.json`
- 원시 응답 A/B: `/Users/rexxa/work/artprod2-data/reviews/round0/a.json`, `b.json`
- 원문 시각 관찰 A/B: `/Users/rexxa/work/artprod2-data/reviews/round0/a.md`, `b.md`

## Godot FPS

**P1 PASS: 31,490 rendered frames / 60.000226초 = 524.831356468557FPS**, 기준60FPS 이상이다. 실제 macOS windowed Godot4.6 stable official `89cea143`에서 Apple M4 Max, `gl_compatibility`,1600×960 viewport,200명1×,vsync0,max_fps0으로 측정했다. `RenderingServer.frame_post_draw` 계수와 engine frames drawn delta는31,490으로 일치했고 `measurement_verified=true`, visible200, hidden0, 입력 해시 불변을 확인했다. 렌더 계수이므로 디스플레이 scanout FPS나 게임 시뮬레이션 처리량으로 해석하지 않는다.

원시 기록: `/Users/rexxa/work/artprod2-data/godot-crowd/measurement.json`. 배포 요약은 [GODOT_RESULTS.json](GODOT_RESULTS.json), 실제 화면은 [crowd_1x.png](../../../art/character/route_i_lab/prod2/crowd_1x.png)에서 확인한다. 별도 Godot 감사 결과는 최종 검증 기록에 함께 반영한다. PROD1 Chromium59.27FPS 및 이번 Chromium 캡처 수치는 P1에 사용하지 않았다. headless 검사와 짧은 smoke도60초 성능 결과를 대신하지 않는다.

## 변경 파일·검증·제한

변경 범위는 수면 정규화/검사·생산 도구(`art/pipeline5/`), 후보 규격·현재 수락 자산(`art/character/route_i/` 및 템플릿), 인증 랩·채점(`art/whole_figure_lab/prod2*.py`), 독립 Godot 랩(`art/proto/`), 이 패스 문서(`docs/v8/artprod2/`)다. 정확한 변경 경로는 [CHANGED_FILES.txt](CHANGED_FILES.txt), 검증은 [VERIFICATION.json](VERIFICATION.json), 판정은 [GATES.json](GATES.json)에 기록했다. 원시 이미지·측정 로그·브라우저 캡처는 외부 데이터 경로에 둔다.

26개 수락은 구조와 수치 panel judge 합격이고, 그중 고정12명 표본의 독립 R1 가독성도 통과했다. 이는26개 전부의 개별 가독성이나 자연스러움·동작 품질의 전면 승인을 뜻하지 않는다. 생체 부위별 limb mask가 없어 semantic limb 검증은 NOT_MEASURED다. 게임 시뮬레이션·bridge·전체 게임 gate 및 실제 정착지 화면 수용 검증은 수행 범위가 아니다. 색상·노인·N 방향 변형, 게임 내 충돌·가림·발 위치 수용 여부는 미검증이다. 재현성 PASS는 그림의 품질이나 실패 기준을 무효화하지 않는다.

전체 art 테스트117개 통과(23.71초), 마지막 테스트 타입 명시 수정 후 해당35개 재통과(8.30초). 변경 Python19파일 Ruff 통과, docs check 및 diff check 통과, 최종 세트 감사26룩260프레임·48표본·findings0이다. Basedpyright는 최종 **0오류·148경고, exit1**이다. JSON Any 경계·추론 타입·미사용 반환값 등의 경고가 남으므로 정적 분석을 완전 green으로 부르지 않는다. 테스트의 JSON shape를 명시적으로 좁히고 RGBA 전체 값을 검증해 발견된20타입 오류를 제거했으며 생산 이미지 처리 코드는 이 마지막 수정에서 바뀌지 않았다. [LAB_QA.json](LAB_QA.json)은576캡처가 GIF 상태와 일치하고48익명 이미지·384셀·192합성 프레임·96최근접2배 프레임이 재구성과 일치함을 기록한다. 새 라이브러리 의존성이나 게임 시스템은 추가하지 않았다.

Godot 시작/끝 PNG는60초가8Hz·10프레임 주기의48회 반복이어서 동일했다. 이 두 정지화면만으로 중간 애니메이션을 입증하지는 않는다. 가독성 랩의8fps 실제 관찰 캡처는 별도 증거다.

검토용 파일: [1×12명 랩](../../../art/character/route_i_lab/prod2/lab_p1_1x.png), [2×12명 랩](../../../art/character/route_i_lab/prod2/lab_p1_2x.png), [200명 Godot](../../../art/character/route_i_lab/prod2/crowd_1x.png), [24 GIF](../../../art/character/route_i_lab/prod2/), [후보 자산 manifest](../../../art/character/route_i/MANIFEST.json).

## 설치 노트 요약과 오너 결정

[INSTALL_NOTES.md](INSTALL_NOTES.md)에80×112 upright/112×80 sleep, 발 anchor,64px 타일 상대 배치,별도 contact shadow,56/40px 정규화,S/E와 W flip·N=S,frame plan,entity 속성 매핑과 결손10룩 fallback을 기록했다. 현재 S1 실패로 설치·채택·archive 이동은 수행하지 않는다. fallback은 수락 수를 늘리거나 누락 룩을 통과로 계산하는 근거가 아니다.

오너 결정 필요: 실제 N 방향을 별도 제작할지, 노인 및 머리색 변형의 범위, 상의 추가 계획. 현재의 원본 보존·재시도 한도를 유지한 채 후속 생산을 새 계약으로 진행할지는 별도 결정이다.
