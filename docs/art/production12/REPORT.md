## 구현 완료 보고

> **최종 전체 게이트 102/102 PASS · 수용 기준 PARTIAL.** H1–6은 NOT_VERIFIED(contended), H8·H9는 FAIL, H10은 PASS입니다. 동결 제품의 기능·검증·증거 정리를 마쳤으며 성능 개선이나 완전한 판독 성공은 주장하지 않습니다. [PR #511](https://github.com/hyunlord/new-world/pull/511) · [게이트 증거](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production12/gates/final/terminal.json).
>
> 아래 `E/`는 `/Users/rexxa/work/art-production12-evidence/`, `R/`는 `/Users/rexxa/work/new-world-art-prod/`입니다. 출처가 없는 수치는 쓰지 않았습니다. attempt2 판독은 완성된 `E/p3/reading-score-attempt2.{md,json}`을 반영했습니다.

### 구현 의도

game_v25의 실제 실행 프레임 비용을 검증하고, 이름·생애·군중 선택을 더 정확히 읽을 수 있게 하며, 마을 간 식량·장작 운반과 관계 변화를 기존 화면에 표시하려 했습니다. 구현, 동일성, 성능, 실제 입력, 독립 판독의 합격 여부를 서로 대체하지 않습니다.

### 구현 내용

- Part0: 시뮬레이션 Pass20 `6e1446b4d6af41e13e2b221b17609751510f046f`를 병합했습니다. 관계·짐꾼 packed 배열의 ID, 방향, 출발 수량, 실제 적재량 계약을 기록했습니다. 유효 세션 심사와 release 전용 측정 작업트리, 실행 중 구간 계측·노드 제거 진단 도구를 준비했습니다.
- Part1: Rust declump 커널을 시도하고 수치·실제 고정자료·정지 및 실행 동일성을 검증했습니다. 속도 세션은 모두 경합으로 불허되어 **기본 경로에는 켜지 않았습니다**. 선택 후보(c), 이어서(b)를 중단했고, (b)의 입력·출력·독자 목록만 보존했습니다.
- Part2: 표시 이름과 마을 이름에 WORD JOINER를 적용하고 기존 Galmuri11-Bold를 코드로 지정했습니다. 가족 상태 줄바꿈, 생애/연대기 끝 여백과 경계 줄 배치, EN/KO 전환 즉시 창고 표제 갱신을 고쳤습니다. 겹친 인물 클릭 순환, 선택 인물·아이콘 표시 순서와 이름 링크 hover를 구현했습니다. 실제 입력 및 아이콘 포함 블라인드 재시험을 실행했습니다.
- Part3: 짐꾼·관계 getter를 tick 단위로 읽고, 실제 적재량에 따른 짐 운반 자세, 식량/장작 절차적 pack glyph, 복귀 투명도, 저배율 목적지 점선과 마을 관계선을 추가했습니다. 연대기 17~21의 역할별 문장과 묶음을 구현했습니다. 첫 판독·리뷰에서 드러난 짐꾼의 일반 목표 문구 모순을 수정했습니다.

### 구현 방법

release 성능은 전용 측정 작업트리와 각 세션의 소스·라이브러리 SHA로 식별합니다. 동일성용 dev opt-level=2 결과를 release 성능으로 쓰지 않습니다. 부하/외부 CPU 경합 심사에 실패한 세션은 측정값으로 채택하지 않고 원래 재시도 상한과 실패 원본을 보존했습니다. 인구 감소 블록은 원자료에 남기되 pooled 집계에서는 제외하도록 수집기를 보정했습니다.

실제 Godot 4.6 single precision(double=false)의 Vector2/Rect2 연산과 signed zero 등을 관찰하고 Rust 수치 테스트에 고정했습니다. 실제 renderer 호출 2,000개, 두 겹침 모드의 4,000개 레코드를 비트 단위로 재생했습니다. 후보(b)는 시간·타이머·선택·언어·reset 입력까지 조사했지만 구현하지 않았습니다. 출처: `R/docs/art/production12/perf/declump-kernel/verification.md`, `memo-inventory.md`.

표시 이름은 원래 이름과 분리하고 복사 시 WORD JOINER를 제거합니다. glyph probe는 실제 생애 글자 크기24에서 조이너 전후 픽셀 차이0을 확인했습니다. 클릭 순환은 화면에 그린 위치를 기준으로 테스트했습니다. 블라인드 크롭은 아이콘을 포함하는 기하를 먼저 정하고 제외·답안 고정 순서를 지켰으며 원본 픽셀을 확대·재표본화하지 않았습니다.

짐꾼은 AgentId나 행 번호 대신 signed entity bits로 합칩니다. 출발 수량은 상태 문장용이고 body/load 선택은 `carried_foods`/`carried_woods`를 사용합니다. 기존 식사 우선순위와 Cold.pose 동작을 유지합니다. 기존 cold 동작은 정지 idle/sit/hop에만 crossed를 적용하며 walk/carry를 새로 바꾸지 않습니다. 장작은 승인된 원래 어깨 짐 그림과 승인된 동일주기 파생 프레임을 그대로 사용해 중복 prop을 붙이지 않습니다. 현재 목적지 hearth는 live settlement_ids와 formation_xs/ys로 찾습니다.

### 기능 설명

이름 링크에서 인물·기록을 열고, 겹친 사람을 반복 클릭해 순환 선택할 수 있습니다. 가족 상태는 말줄임 없이 표시하고 EN/KO 전환은 멈춘 세계의 창고 표제에도 즉시 반영됩니다. 짐꾼은 종류별 주황/갈색 pack glyph를 표시하며 복귀 시 alpha0.6입니다. zoom<2에서는 glyph2배와 현재 목적지까지 화면1px 점선을 표시합니다. 마을 관계선과 연대기 사건은 현재 관계와 과거 사건을 각각 표시합니다.

### 변경된 파일 목록

각 커밋의 모든 변경 경로·상태는 [커밋별 변경 파일](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production12/changed-files-by-commit.tsv)에 기록했습니다. 병합 커밋은 첫 부모 대비 차이입니다. 아래는 기준 HEAD까지의 Art12 첫 부모 커밋 전체입니다.

| Part | 커밋 | 목적 |
|---|---|---|
| 0 | `bf85dac60579dd4a9135dd9f6f92a3139c8d1f6e` | Pass20 병합 |
| 0 | `51ac34dff554e851e65f95a7b63b11e650859516` | 실행 계약 보존 |
| 0 | `9934f78513cd1657a5df8fa2f7854b9df9e02e95` | release 유효성·동일성 도구 |
| 0 | `77002d6f2733898f436d64d4b3a4b6735e53899a` | 캡처 출처 변화 검출 |
| 0 | `a4901cb12e29afb74783039ed66aef4a74014148` | 인구 블록 집계·실행 진단 보정 |
| 0 | `c8f092db4aa5bf4ec21c1663a0df92bf0633c974` | 동일성·경합 증거 보존 |
| 1 | `37be1b855aa5f8da93c35db3d735b5babed5eced` | declump 수치·고정자료 검증 |
| 1 | `d55c94f5815bc2a9d3eb5465df53b638d5f70c24` | 미측정 커널 기본 비활성화 |
| 2 | `0b2c2fcd8578901a2c6affa9d6a01d933b084dee` | 이름·생애·군중 선택 |
| 2 | `5d462f3de722f348b3a42b15fe68a6ddb6654e64` | 실패 판독 기준 보존·hover |
| 3 | `33b8f7c7e03e658da20074f1eb56969f40fbaffb` | 짐꾼·관계·연대기 표시 |
| 3 | `c46e66bd128aa271772caba5f9317de860d222db` | 짐꾼 목적지 모순 수정·검증 경로 보정 |
| 3 | `c08076dd67ee0a54470f16fc6b993c3172752320` | 수정된 마을 표시 소스 동결 |
| 4 | `9eba38ffae8095b5f108722cd120a1a71e87aa59` | 경합으로 불허된 after 결과 보존 |

표와 TSV는 `9eba38ff`까지입니다. 최종 문서 커밋은 자기 SHA를 자기 본문에 넣을 수 없으므로 최종 응답의 publication receipt에 추가합니다. 시뮬레이션 반입 head는 위에 별도 명시했으며 그 내부 커밋들을 Art12 자체 변경으로 재분류하지 않았습니다.

### 측정 결과

| 항목 | 기준 | Before | After / 현재 판정 | 배수 |
|---|---|---|---|---|
| H1 | W0 정지 Z4 median≤6ms | NOT_MEASURED | NOT_VERIFIED(contended) | 계산 불가 |
| H2 | W1 실행2× Z4 p99≤16.67ms | NOT_MEASURED | NOT_VERIFIED(contended) | 계산 불가 |
| H3 | W200 실행2× median≤16.67ms | NOT_MEASURED | NOT_VERIFIED(contended) | 계산 불가 |
| H4 | W200 정지 median≤10ms | NOT_MEASURED | NOT_VERIFIED(contended) | 계산 불가 |
| H5 | W500 정지 median≤16.67ms | NOT_MEASURED | NOT_VERIFIED(contended) | 계산 불가 |
| H6 | W500·W-multi 실행 보고 | NOT_MEASURED | NOT_VERIFIED(contended); W500 실행은 합격/실패 판정 대상 아님 | 계산 불가 |
| H7 | 채택 후보 동일성 모두 통과 | P0 대조군 PASS | N/A — 채택된 최적화 없음. 후보(a) 동일성 PASS이나 속도 미검증·기본off | 해당 없음 |
| H8 | 활동≥10/14, 종류별0/2 없음; 요약≥5/6 | 과거 결과로 대체 안 함 | **FAIL**: 진단3/10, 4개 누락. 마을요약 NOT_RUN. pose oracle355개 불일치0 | 해당 없음 |
| H9 | 생애 모든 사실 정확·군중0%가림/3개·실제입력 | 첫 생애 판독 FAIL | **FAIL**: attempt2 이름오류12건·건설수7건 미확인. 화면/패널 활동대응3/3, 실제입력13단계PASS | 해당 없음 |
| H10 | 연대기17~21·짐꾼 실제화면 | 해당 없음 | **PASS**: 전용테스트와 native 짐꾼 glyph/운반자세·관계선 확인. live18/21 부재는 제한으로 유지 | 해당 없음 |

**세계·조건별 release 수치** — 아래 모든 Before 값은 중앙값/p90/p99/최대·틱/프레임·측정 인구 시계열 모두 NOT_MEASURED입니다. After도 모두 NOT_MEASURED이며 배수는 계산 불가입니다. census 인구를 측정 블록 인구로 쓰지 않습니다.

| 세계 | seed/tick | census 인구/마을 | 정지 Z4 Before→After | 실행2× Z4 Before→After |
|---|---|---|---|---|
| W0 | 20261010 / 40560 | 29 / 1 | NOT_MEASURED→NOT_MEASURED | NOT_MEASURED→NOT_MEASURED |
| W1 | 20261013 / 126994 | 66 / 2 | NOT_MEASURED→NOT_MEASURED | NOT_MEASURED→NOT_MEASURED |
| W200 | 15100 / 1440 | 200 / 1 | NOT_MEASURED→NOT_MEASURED | NOT_MEASURED→NOT_MEASURED |
| W500 | 15100 / 1440 | 500 / 1 | NOT_MEASURED→NOT_MEASURED | NOT_MEASURED→NOT_MEASURED |
| W-multi | 19400 / 121080 | 65 / 2 | NOT_MEASURED→NOT_MEASURED | NOT_MEASURED→NOT_MEASURED |
P0의 다섯 세계 matrix, W500 정지 구간, W200 실행 구간, W200 실행 node-ablation은 각각 세 번의 시도 상한을 소진했고 허용 세션0개입니다. W0의 중단된 pre-admission 시도도 상한에 포함했습니다. 채택 가능한 표본·인구 블록 자체가 없습니다. 외부 프로세스 CPU 경합 원본: `R/docs/art/production12/perf/admission-index.json`, `baseline.md`, `holds.md`, `E/p0-measurements/`.
Part4는 동결 `c08076dd`를 release로 빌드·import한 뒤 21/21 admission hold가 exit75로 불허되어 Godot 측정이 한 번도 시작되지 않았습니다. 다섯 세계×3회와 W500 정지/W200 실행 구간×3회이며 허용0개입니다. 초기 구간 명령의 profile 누락 오류6개는 admission 이전 오류로 원본을 보존했고, 수정 후 원래 상한을 지켰습니다. `R/docs/art/production12/perf/after.md`, `after-admission-index.json`, `after-build-provenance.json`.
실행 비용≥1ms 순위, AgentRenderer 내외 비용, getter 호출/시간, 틱·인구 계열과 W500 정지/W200 실행 구간 타이머는 **전후 모두 NOT_MEASURED**입니다. 인구 감소로 표시하거나 합산할 raw frame block도 없습니다. 실행 진단의 after 비용 순위를 만들지 않았습니다. 출처: `R/docs/art/production12/perf/running-diagnosis.md`, `after.md`.

| 후보 | 동일성·수치 | 속도/세션 | 판정 |
|---|---|---|---|
| (a) Rust declump | 4,000 비트정확 레코드, 6개 framing RGBA차이0, W0/W200 각600frame 및 W500 정지/실행 pan 각240frame PASS | 두 workload×3회 모두 admission 불허, 측정0 | NOT_VERIFIED(contended), 기본off, 속도 개선 주장 없음 |
| (b) 변경 없는 에이전트 생략 | 전체 입력·독자 목록 작성; 구현 안 함 | 없음 | NOT_IMPLEMENTED / 범위 축소 |
| (c) 선택 후보 | 구현 안 함 | 없음 | 먼저 중단 |
출처: `R/docs/art/production12/perf/declump-kernel/verification.md`, `pixel-comparison.json`, `running-comparison.json`, `speed-admissions/index.json`. 첫 running collector SHA 불일치 실패는 보존했고 같은 collector로 reference를 다시 캡처한 뒤 비교했습니다. P1 동일성을 이후 의도적으로 바뀐 UI의 픽셀 동일성으로 주장하지 않습니다.

| 활동 | 정답 | 오답 | 누락 |
|---|---:|---:|---:|
| TendFire | 1 | 1 | 0 |
| Craft | 1 | 1 | 0 |
| Repair | 0 | 1 | 1 |
| HearthCircle | 0 | 2 | 0 |
| Explore | 0 | 0 | 2 |
| Play | 1 | 0 | 1 |
| SitByFire | 0 | 2 | 0 |

활동 총3/10, 7오답, 4누락으로 원래14개 기준 FAIL입니다. 허용 보충에도 Explore 없음, Repair/Play 각각 한 명 부족을 보존했습니다. 임의 세계·동일 인물 복제·응답 기반 재시험은 하지 않았습니다. 마을요약은 유효2쌍/필요6쌍, top-two 동률 제외2건으로 정답 공개 전에 NOT_RUN 처리했습니다. 출처: `R/docs/art/production12/activity/evidence-summary.md`, `blind-result.json`.

생애 attempt1은 140개 페이지 관찰·요약을 대조했고, 잘못 단정한 주장30개와 미확인 주장30개가 남았습니다. 이름 오독과 짐꾼의 모순된 일반 목표가 포함되어 **FAIL**입니다. 반복 이름 주장은 각각의 기록된 주장이지 새 사건 수가 아닙니다. 미확인을 자동 오답 또는 정답으로 바꾸지 않았습니다. 화면/패널에 표시된 활동 대응은3/3PASS, 각 선택 body/icon 가림0%입니다. 정확히 어느 sprite인지 사람 판독으로 확정했다는 뜻은 아닙니다. 출처: `E/p3/reading-score-attempt1.{md,json}`, `reference-audit/factual-ledger.json`. attempt2 역시 **H9 FAIL**입니다. 140개 관찰·서술을 대조해 이름오류12건과 근거 없는 건설수0 주장7건을 기록했습니다. 화면/패널 활동대응3/3과 짐꾼 목적지 수정은 확인했지만 이름오류를 상쇄하지 않습니다. 열매 수는 원래 capture-time resource 자료가 아니라 별도 동일소스 재생과 독립감사로만 보강했습니다. `E/p3/reading-score-attempt2.{md,json}`, `attempt2-audit/resource-replay{,-audit}.json`. attempt1과 답안 불확실성은 보존했습니다.

### 환경

macOS Apple M4 Max, Godot4.6 stable official(`89cea143987d564363e15d207438530651d943ac`), single precision. Windowed Metal 캡처와 headless 테스트를 구분했습니다. live reset은 game_v25, stress reset은 game_v19입니다. P0 release bridge SHA256 `3b38f12812cea65c3c2c9e5d5d126dfb359c5dc0bc3860a976a554a304ca03c2`; P2/P3 입력 증거의 opt2 dev bridge `ed600ac38228c72962d4459a7ea7b9fcbd63a0a0be4bd75e46403690b2b66a27`. attempt2 캡처 bridge SHA256 `65be186ddffe90bba99a491432d7e41bc63abb431f4350b09561b6420ea19a3c`; after release/loaded debug-path 동일 SHA256 `1e6f76187e9e0d87e87426fe2b0aa510c3c797efd61b9cb14fed383531d2bf91`.

### 독립 리뷰

Round1 기능/설계 리뷰 A는 **REVISE**: 짐꾼 상태는 마을1 목적지인데 일반 목표는 다른 주민으로 표시하는 모순을 지적했습니다. `c46e66bd`에서 제한적으로 수정했습니다. `E/p3/visual-review-A.md`.

Round1 시각/CJK 리뷰 B는 **PASS(관찰한 범위)**: 57개 PNG를 직접 보았으며 저배율 관계 캡션 대비는 선택 개선사항입니다. 이는 독립 생애 판독 FAIL을 취소하지 않습니다. 또한 중간 모든 페이지·OS clipboard round trip·성능까지 검증한 판정이 아닙니다. `E/p3/visual-review-B.md`.

Round2는 검토한 구현과 정직한 증거 분류를 **제한적 ACCEPT**, 전체 요구 기준은 **REVISE / UNFINISHED**로 판정했습니다. 새로운 구현 blocker는 없으며 짐꾼 일반목표 모순은 실제 EN/KO 이미지에서 해소되었습니다. H1–6 미검증·H8/H9 실패와 실패 판정은 최종 게이트 통과 뒤에도 그대로입니다. 코드·출처·커널·실제입력·블라인드 절차 및 native7장 검토 범위이지 모든 페이지의 가독성 합격이 아닙니다. `E/final-review-round2.md`.

### 자율 결정 목록

- 일반 lead/main 초기화 지침 대신 이번 GOAL의 art WIP/production 분기 계약을 따랐습니다. 기존 사용자 변경을 보존했습니다.
- 시뮬레이션을 바꾸거나 외부 고CPU 프로세스를 종료하지 않았습니다. 경합으로 측정이 막혀도 상한을 새로 부여하거나 과거 P11 수치를 채우지 않았습니다.
- 기본dev census가 오래 걸린 뒤 identity만 dev opt2로 전환하고 출처를 분리했습니다.
- 수집기 결함 수정 전 실패·중단 증거와 이후 재빌드·재캡처를 모두 보존했습니다.
- 속도 미검증 커널은 기본off. 선택 후보(c)→(b) 순서로 중단해 필수 UI/입력/판독/짐꾼을 수행했습니다.
- procedural16px 짐꾼 glyph를 사용했으며 생성형 이미지 호출은 사용하지 않았습니다.
- Part3 커밋 `33b8f7c7`, `c46e66bd`는 Lore 본문·트레일러가 있으나 요구된 `art-p12(p3):` 제목 접두사를 누락했습니다. 게시된 이력을 amend/rewrite하지 않고 형식 위반으로 기록하며 이후 커밋은 접두사를 사용합니다.
- 실제 입력 통과, visual 리뷰 PASS, 수치 동일성 PASS를 활동·생애 판독 성공으로 바꾸지 않았습니다.

근거: `R/docs/art/production12/DECISIONS.md`.

### 실행 기록

시작09:32:00Z, 기록된 휴식 없음. 아래 시간은 `R/docs/art/production12/TASKS.md` 기준이며 Part0/1 사이 짧은 공백을 임의로 휴식으로 세지 않았습니다.

| Part | UTC 시작 | UTC 종료 | 소요 |
|---|---|---|---|
| 0 | 09:32:00 | 10:32:55.172844 | 1시간00분55초 |
| 1 | 10:34:37.871766 | 11:03:58.842474 | 29분21초 |
| 2 | 11:03:58.842474 | 11:20:04.857372 | 16분06초 |
| 3 | 11:20:04.857372 | 11:47:23.770552 | 27분19초 |
| 4 | 11:47:23.770552 | 11:50:41.582635 | 3분18초 |
| 5 | 11:50:41.582635 | 12:08:45 검증·보고 마감 | 18분04초 + 게시·정리 영수증 |

게시 대상은 `codex/art-production-2026-10-wip`와 fast-forward 전용 `codex/art-production-2026-10`입니다. [PR #511](https://github.com/hyunlord/new-world/pull/511)은 기존 내용을 보존하고 Art Pass12 절을 추가하며 draft를 유지합니다. 보고서 자체의 커밋 SHA, 게시 뒤 `git ls-remote`로 확인한 두 HEAD, 작업트리 제거 및 최종 종료 시각은 최종 응답의 발행 영수증에 함께 기록합니다. 원본 실행 영수증은 `E/publication-receipt.json`에 보존합니다.

### 확인된 제한사항

전후 모두 유효한 release 측정이 없어 개선 배수와 몇 명까지 실행 프레임을 버티는지 확정할 수 없습니다. 커널은 기본off이며 성능 개선으로 채택하지 않았습니다. 활동 인식 기준과 생애 attempt1·attempt2가 모두 실패했습니다. 이름을 통째로 보이게 하고 조이너 픽셀 차이가 없다는 사실만으로 모든 이름이 정확히 읽힌다고 할 수 없습니다. 마을요약6쌍 시험은 시행하지 못했습니다. live 연대기18/21 및 더 넓은 cause3 Life 표본은 부족하므로 단위테스트를 실제화면 증거로 대체하지 않습니다. clipboard의 실제 OS 왕복과 모든 세계·모든 중간 스크롤 상태는 전수 검증을 주장하지 않습니다. Part3 두 커밋의 필수 제목 접두사 누락도 남은 기록 형식 위반이며, 기존 이력은 보존합니다.

### 게이트 결과

- P0 clean 전체 게이트: 88/88 PASS (`R/docs/art/production12/gates/p0-clean/`). 최초87/88 실패와 경로 수정 기록은 남겼습니다.
- P1: sim-bridge52개 테스트, 실제4,000레코드, spatial594비교 및 동일성 증거 PASS. 성능 세션은 불허.
- P2/P3: 실제입력 `Input.parse_input_event` windowed13단계 PASS (`E/p3/real-input-final/result.json`); font/layout 및 porter 개별 회귀 증거 있음.
- P3 pre-freeze 게이트는 100/102였고, 읽기 경로 정적 검사와 과거 링 두께 기대값이 실패했습니다. 독립 P9 원시 오라클·SimReads 경로와 요구된 Z4 1.5px 기대값으로 바로잡고 실패 원본을 보존했습니다. 프로세스 exit만 보고 stderr/개별 실패를 PASS로 덮지 않습니다.
- 최종 전체 cargo test/clippy, GDScript strict, Python/tooling, windowed input 및 오류 스캔: **102/102 PASS, exit0** — 실행 SHA `9eba38ffae8095b5f108722cd120a1a71e87aa59`. `GATE_OUTPUT_DIR=docs/art/production12/gates/final CARGO_BUILD_JOBS=6 CARGO_PROFILE_DEV_OPT_LEVEL=2 CARGO_PROFILE_TEST_OPT_LEVEL=2 bash tools/art_pipeline/gate.sh final`; 스크립트·파싱 오류 검색0건. `gates/final/terminal.json` 및 `results.tsv`. 게시 커밋의 후속 변경은 문서·증거이며 동결 제품 소스 동일성을 별도로 확인했습니다.

### 정직한 판단

마을 하나를 2배속으로 돌릴 때 이제 버티는지, 정지/실행에서 몇 명까지 버티는지는 아직 유효한 release 전후 측정으로 답할 수 없습니다. 사람의 일생을 틀리지 않고 읽을 수 있다는 기준은 두 판독 시도에서 실패했습니다. 세 장면의 선택대상 가림0%, 화면/패널 활동대응3/3, 실제입력13단계는 확인했지만 정확한 선택 sprite 식별을 보장하지 않으며 활동 블라인드는3/10으로 부족합니다. 짐꾼의 적재·표식·현재 목적지와 마을 관계는 native 화면에서 확인되어 H10은 PASS이나 live18/21 표본 부재는 남습니다. 전체 요구 기준은 미완료입니다.

다음 아트 과제는 판독 오류가 난 이름의 실제 글자 구분과 낮은 활동 식별력을 증거에 따라 개선하는 일입니다. 프레임 예산 과제는 먼저 유효한 실행 세션으로 병목과 after를 확보해야 합니다. 실패한 첫 시도와 미확인 사실도 그대로 보존했습니다.

원본 PNG 140장과 참조·매핑은 [검증된 ZIP](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production12/reading/part3-attempt2/raw-captures.zip)에, 최종 생애 판독 대조는 [판독표](https://github.com/hyunlord/new-world/blob/codex/art-production-2026-10/docs/art/production12/reading/part3-attempt2/reading-score.md)에 있습니다.
