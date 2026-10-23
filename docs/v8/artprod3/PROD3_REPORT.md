# ART-PROD3 보고서 — route I 채택

**판정: route I 채택.** S1은34/36룩(성인16/18·어린이18/18)이며 원래 성인≥17 기준의 FAIL은 보존한다. 2026-10-23 planner/owner가 잔여 성인2룩의 채택 기준을 면제했다. PROD2 R1·P1·D1은 유지한다. 신규 생성 **0회**, 게임 코드 변경·merge 없음. 기준은 `c63eb72b198f7cd213be5bb5dfd0fe4cc2339b73`; [v8/art-prod1](https://github.com/hyunlord/new-world/tree/v8/art-prod1)에 게시하며 최종 원격 일치 SHA는 최종 응답과 외부 `ADOPTION_PUBLICATION.json`에 기록한다.

## 채택

> Planner/owner decision 2026-10-23: route I is ADOPTED at S1 34/36 (adults 16/18) on top of PROD2's R1, P1 and D1 — the 17/18 bar was the planner's and is waived for the two adult_M_*_long_tied looks, which fall back per HANDOVER.md.

[결정 D4](PROD3_DECISIONS.md)는 최초 미채택(D3)의 처분만 대체한다. S1 수치·실패·26룩 재실행 바이트 차이는 그대로다. `TEMPLATE_V3`를 캐릭터 표준으로, `route_i/`와 `MANIFEST.json`을 자산 원천으로 채택한다. 두 결손은 같은 age/sex/top의 bun으로 대체하며 수락 수에 더하지 않는다.

## 임계값·근거와 재절단

[결정 D1](PROD3_DECISIONS.md)에 따라 모든 프레임의 가시 bbox를 **alpha≥16**으로 통일하고 fit·발 기준선·크기 band에 적용했다. 배율·회전·연결성의 기존 alpha≥128 기준은 유지한다. 캔버스 안 RGBA는 유지하고 밖의 alpha<16만 잘라 프레임별 수량·최대 alpha를 기록한다. 결손10룩 전체에서 11프레임·112픽셀, 최대 alpha1이 잘렸으며 그중 1픽셀은 최종 실패 룩의 sleep이다. 승인8룩에서는 111픽셀이다. 6px alpha3–12 halo 통과, alpha16/128/255 실제 넘침 거절, 내부 RGBA 보존·버린 수량·발 기준·band 테스트를 포함한다.

PROD2 표에 지정된 best 원본만 사용했다. 아래 `old1`은 보존 PROD1 시도1, `new1`은 이미 보존된 PROD2 시도1이며 이번 생성이 아니다. 통과는 같은 시트의 구조와 panel judge 모두 10/10이다.

| 결손 룩 | 원본 | 구조 / judge | 결과 |
|---|---|---|---|
| adult_F_hide_wrap_bun | old1 | 10/10 · 10/10 | 승격 |
| adult_M_fur_cloak_long_tied | old1 | 10/10 · 9/10 | MISSING: E/work_a judge 실패 |
| adult_M_fur_cloak_short | old1 | 10/10 · 10/10 | 승격 |
| adult_M_hide_wrap_long_tied | old1 | 9/10 · 10/10 | MISSING: E/idle 가시 높이60 > 상한58 |
| adult_M_loincloth_bun | old1 | 10/10 · 10/10 | 승격 |
| child_F_fur_cloak_bun | new1 | 10/10 · 10/10 | 승격 |
| child_F_fur_cloak_short | old1 | 10/10 · 10/10 | 승격 |
| child_F_loincloth_bun | old1 | 10/10 · 10/10 | 승격 |
| child_F_loincloth_long_tied | old1 | 10/10 · 10/10 | 승격 |
| child_F_loincloth_short | old1 | 10/10 · 10/10 | 승격 |

`adult_M_hide_wrap_long_tied`의 sleep은 해결됐지만 E/idle의 opaque 높이58과 visible 높이60이 다르다. 목표대로 band에도 alpha16을 적용해 실패를 유지했으며 배율·상한을 완화하지 않았다. 상세 원본 해시·프레임·crop 로그: [RESLICE_RESULTS.json](RESLICE_RESULTS.json); 원시 증거 `/Users/rexxa/work/artprod3-data/reslice/`.

## 26룩 불변 확인과 유지 게이트

**기존 게시26룩의 546개 파일은 바이트 불변이다. 재실행 PNG의 바이트 동일성은 FAIL이다.** 독립 재절단26룩 모두 구조·judge 10/10이지만, 가시 bbox 중심·발 기준 변경으로 260PNG 중202개가 달라졌다(26룩 전체에 차이). `accepted_png_diffs`에 프레임별 전후 SHA를 전부 기록했다. 재실행에서 이26룩의 canvas 밖 잘림은 0이었다. 새 PNG로 교체하지 않았으며, 보존된260PNG도 새 alpha16 구조 검사를 모두 통과했다. 원본·프롬프트36쌍의 해시도 불변이다.

PROD2 [R1](../artprod2/review_round0/scores.json)·[P1](../artprod2/GODOT_RESULTS.json)·[재현성 D1](../artprod2/D1_RESULTS.json)은 재측정 없이 인용한다: 두 리뷰어 양 배율 활동12/12, item+top+hair31–32/36, 성인sex+age20–21/22; Godot 실제 창200명 524.831FPS; 123원본 두 실행 재현성 PASS. planner D1은 이번 임계값 수정에 신규 리뷰를 요구하지 않는다. 원래 S1 실패를 통과로 바꾸지 않으며 채택은 위 오너 결정에 따른다. 이번 전체133테스트·lint·타입검사(오류/경고0)·docs 검사와 현재340프레임 구조 검사는 통과했다. 명령·해시·범위는 [VERIFICATION.json](VERIFICATION.json), 독립 검토는 [CODE_REVIEW.md](CODE_REVIEW.md)에 기록한다.

## 아카이브·인수인계·오너 결정

`art/character/archive/` 아래 `route_p`(TRIAL4), `route_a`(TRIAL3), `candidates`(TRIAL5 Part0), `trial5_diagnostics`(TRIAL5), `route_i_lab/prod2`(PROD2)를 이동했다. `route_b`는 기준 트리에 없어 이동 대상이 없고, 랩 도구는 원위치에 보존했다. [archive README](../../../art/character/archive/README.md)에 각 trial과 보고서를 명시했다. `route_i/MANIFEST.json`과 1×/2× 전체 보드는 기존34룩을 유지한다. [채택 검증](ADOPTION_VERIFICATION.json): 이동2,320파일·자산714파일 바이트 보존, manifest34개·PNG/sidecar680개 해시 일치. 기존 VERIFICATION의 미채택 판정은 면제 전 역사 기록이다.

[HANDOVER.md](HANDOVER.md)에 경로·canvas/anchor·프레임·방향·나이별 배율·그림자·속성·리뷰 증거를 인계했다. [INSTALL_NOTES.md](INSTALL_NOTES.md)의 잔여 fallback은 `adult_M_fur_cloak_long_tied → adult_M_fur_cloak_bun`, `adult_M_hide_wrap_long_tied → adult_M_hide_wrap_bun`이며 결손을 수락으로 세지 않는다. 남은 오너 결정은 **실제 N 방향**, **노인/머리색 tint**, **추가 상의** 3건이다. 게임 통합·새 성능 측정·새 가독성 평가는 수행하지 않았다.
