# Art Pass 2 — 자율 결정 기록

- 시작 / Part 0 시작: 2026-10-10T13:29:26Z. 8시간 경계: 2026-10-10T21:29:26Z.
- ART2_BASE_SHA: f3f2cafc950811ba7e798eb9cbf6e007e5d35dc7.
- 환경: Darwin hyunlordui-MacBookPro.local 27.0.0 arm64; Mac Aqua; rexxa.
- 최초 상태: 기존 WIP와 origin의 차이 0/0. 보존할 로컬 전용 커밋 없음. 새 GOAL.md 외 깨끗함.
- 범위: 이번 GOAL의 명시적 checkout/branch/merge 제한이 기존 lead/main 초기화 지시보다 우선한다.
- CPU: CARGO_BUILD_JOBS=6, Rust test threads=6. 창 캡처는 공유 lock으로 직렬화.
- graft: 이 checkout의 첫 질의는 빈 그래프 결과. 다른 checkout의 그래프를 쓰지 않고 직접 소스를 확인한다. 확인된 절약 0.
- 실행 계획: 사용자 지정 Part 0–8 순서를 유지한다. 독립 구현은 기준선 통과 후 분담하며 git commit/push/atlas 통합과 최종 gate는 리더가 직렬 수행한다.
- Python: 사용자 지정 Pillow/NumPy와 표준 라이브러리만 사용; 새 의존성 설치 없음.
- Part 0: 공유 window lock의 나이만으로 삭제하던 분기를 제거했다. Pass 4의 살아 있는 캡처 lock을 빼앗거나 이전 Production 1 기록을 수정하지 않고, 소유자가 해제할 때까지 기다린다.
- 실행 계획 스킬 적용: 제공된 9개 Part가 실행 계획이며, 사용자 명시 지시에 따라 별도 승인 대기나 추가 worktree 생성은 하지 않는다.

- Part 0 종료 / Part 1 시작: 2026-10-10T13:37:22Z. P0 full gate 19/19 PASS, Rust 1850 passed / 0 failed, Python 57, Godot scene 12. 원본 42/42 readable. Pass 3 merge a9763bf7 충돌 없음.
- Part 1 U1 해석: §6.2가 모든 크기를 outline 전 native로 정의하므로 U1 native 출력은 16×16 정확 캔버스이며 계수도 16 목표로 계산한다. 기존 outline의 별도 렌더 캔버스는 18×18이 될 수 있다. §6.4 테스트는 native 출력 16×16을 검사하고 렌더 캔버스 크기를 별도 기록한다. 임의로 14 목표로 바꾸지 않는다.
- Part 1 N-carry: 원래 시트에 두 프레임만 있으므로 실제 존재하는 두 프레임 사이 아래 40%의 차이 ≥4와 head-top ≤1을 검사하고 `two_frame_cycle` 경고를 기록한다. 존재하지 않는 4프레임 증거를 만들지 않는다.
- 범위 확인: 기존 child 시트 V_c1/V_c2에는 carry/gather/build/drink 슬롯이 없지만 A13은 세 체형 모두 8가지 활동을 요구한다. Part 3에서 기존 256 슬롯 집계와 별개인 필요한 child 활동 보충 슬롯을 만들며, 18회 생성 예산 안에서 채우거나 같은 child 원본 기반 대체로 명시한다. 기존 Production 1 SHEETS/기록은 바꾸지 않는다.

- Part 1 종료 / Part 2 시작: 2026-10-10T13:42:55Z. Python 전체 77/77 PASS; v3+v4 14/14, 별도 계약 14/14. physical origin 보존으로 동일 난수 fixture 일치율 66.964%→84.375%; 80% 기준 유지. `phi-p`를 보존하며 환산 sampling phase와 원점을 모두 기록한다.
- v4에서는 spill repair로 생길 수 있는 비팔레트 색만 다시 팔레트에 투영하고 변경 texel 수를 기록한다. 처방된 계수 후에도 목표보다 큰 후보는 추가 축소·자르기로 숨기지 않고 geometry 거절한다.
- Part 2 재검토: §7.2의 한 animation group=한 attempt 규칙을 유지한다. §7.3의 다음 후보 교체는 character의 경우 전체 group을 다음 attempt로 바꾸어 다시 검토하거나 같은 group의 승인된 프레임에서 파생한다. 서로 다른 attempt의 프레임을 몰래 섞지 않는다. S-walk 교체로 reference idle이 바뀌면 두 번째 검토 시 그 체형의 참조를 함께 갱신한다.

## Part 2: frozen review and one correction round

- All 16 round-1 review sheets and three body references were frozen with SHA256s before four fresh-context reviewers received only assigned images and the rubric. Reviewers wrote per-slot arrays before acceptance was computed. Two object candidate-ID transcription errors were corrected against the labelled cards; the unvalidated array is retained.
- Suspected display-resampling discrepancies were resolved against original-resolution or explicitly nearest-neighbour crops. Actual blue source texels remain artefact rejections; no criterion was relaxed.
- Region-map objections expose a classifier defect: the unconditional dark upper-body rule can overwrite skin overrides, and shared clothing colors can recolor held props. Apply one bounded classifier correction with view/activity context, then review all affected villagers again. Color sprites are not repainted. Remaining objections after the second review become Part 3 gaps.
- Preserve both rounds of images, mappings and verdicts. A verdict applies only to the exact candidate and reviewed image hash; changed regions require another visual verdict even when the color candidate ID stays the same.

- Scale decision: round 1 has 0 of 128 villager cells rejected for fitting damage with acceptable unfitted art (0%, below the strictly greater than 25% escape condition). Keep adult 28 and child 21; do not rescale the world. The two fitting-damage objections on objects do not trigger a villager-scale escape.

## Part 3: installation boundaries

- Retain both independent Part2 verdict rounds unchanged. During reference inspection, the leader observed a blue crown texel on adult_a reference V_a1_01@1 that those reviewers did not reject. Place that candidate on an installation hold and prioritize a replacement S idle in the first fill sheet. This is a recorded residual defect, not a third review round or retroactive alteration of evidence.
- Functional U1 icons may retain a meaningful existing glyph as an explicitly nonaccepted fallback. Generic shared-blob pixels are never a functional fallback; if no meaningful glyph exists, record the icon unresolved.
- Suppressed decoration will have atlas tombstones outside drawable sprites so renderer fallback paths can distinguish intentionally hidden content from a missing legacy ID.

- Six fill calls produced 96 distinct requested target cells. Pause generation while review finishes. The checkpoint counts unique newly approved generated targets against all 96 attempts, excluding derived frames and already accepted baseline targets; the installation-held adult_a idle is a real gap.
- Fill round 1 exposed anatomical metadata failures on otherwise readable sprites. Use the single correction/re-review loop allowed by §0.4 for image-digest mask annotations only. This changes review metadata, not the generated color candidate or its raw attempt; wrong poses and damaged artwork remain rejected rather than being repainted. Freeze a second review for all six sheets and retain both rounds. Existing Part2 masks must remain byte-identical.

- Part3 checkpoint: six calls, 15/96 newly accepted generated targets (15.625%). Stop generation under §8.1; remaining12-call allowance is unused. Final reviews and guard failures remain visible. Final installation selection is separate from immutable Part2 final_selection.json; approved fill groups replace old partial groups wholesale.

- Installer final guards recheck the assembled approved/derived v4 selection, rather than trusting historical ledger flags. Explicit §8.2 procedural or idle+icon fallbacks remain nonaccepted and may fail animation distinctness; those failures must be recorded, not promoted into acceptance or used to prevent every fallback from being installed. All drawable fallbacks still retain format, geometry, same-body, and basket restrictions. A13/A11 will expose the remaining activity limitations.

- Part3 종료 / Part4 시작: 2026-10-10T14:32:27Z. Generation6calls/15newtargets; installation266drawables/27suppressed/0unresolved, atlas282including16D3references. 612/612PNGregistered, manifest auditPASS; Python126PASS and native-sizeGodotPASS. Fullgate remains Part8. Seven carrygroup idle-icon fallbacks retain leg_difference warnings; this is not animation acceptance.

- Part4 종료 / Part5 시작: 2026-10-10T14:37:18Z. Renderer uses metadata groups and shared frame/icon helpers, no D3 catchall. Activity + 1000-ID elder headless tests PASS; strict 23files/15FFIcalls PASS. No stage2 preserves original appearance; with elders adult hair0–2 and elder3–4. Windowed visual acceptance remains Part7.

- Part5 종료 / Part6 시작: 2026-10-10T14:43:34Z. Grove gain0.55→0.65 based on actual captureseed20261010. Final rendered326/1190forest=27.395%; fresh reload identical, clearing violations0. Seven Godot tests +strictPASS. Canopyprocess afteragentinterpolation fixed actualscene one-framebug; parentbenchmark635.067us for200agents/400trees excludesGPU. Season day_of_season assumes0-based; verify againstlatestgreen inPart7. Missingkeys restorewhite. No Rustassertion changed, so no new mutation targets.

## Part 7: measured visual limits and evidence budget

- Preserve A1/A5/A11 failures; thresholds unchanged. A13 shape difference is not proof of recognizable activity, particularly idle+icon carry fallbacks.
- Public sidecars use lossless gzip with JSON manifest paths; raw original JSON/framebuffers remain under `.harness/evidence/art-production2`. This reduces docs from 52.24 MB to below 40 MB without changing frozen review evidence.
- Window capture began at Part 6 HEAD with capture-tool edits, committed as `5433649a` during the run; no rendering source changed. Calibration ran at that committed head.

## Part 7: latest-green compatibility

- Locally merged `7a8ae053059f3d3eed376a427eadec235259fecd` into art `4908036f` without conflicts (temporary merge `276d8572`); never pushed, branch deleted after10windowed captures and22headless checks passed.
- Actual bridge documents one-based `day_of_season`. Corrected HUD/tint assumptions in art-owned code after failing regression tests, then passed tests/review and observed 봄2일째.
- life_stages, season keys and deaths_old_age are present. No elder or old-age death occurred in day1–6captures; full season transition remains unit-tested rather than observed.

## Part 8: publication evidence

- Full source gate at6a30aa33 passed25/25: Rust1850, Python129, Godot18. No simulation/static Rust test changes or mutation targets.
- Publish report/evidence as a documentation-only commit, immediately push WIP, repeat the full gate on that exact commit into `.harness/evidence/art-production2/final-head-gate/`, then fast-forward green and verify both remote heads. Record final UTC/SHA/PR status in external publication.json to avoid a self-referential commit.
- Independent final report audit found only a stale byte count; replaced it with current measured budget evidence. A1/A5/A11 remain failures.
