# M-UX Stage B 코드 검토 — 최종 B5 소스 갱신

**B5 집중 Rust·strict clippy·창54검사 통과 / 전체 B gate·게시 미완료.** 포착 `2026-10-25T17:06:50.527470+09:00`, HEAD `7577f8af08b233ab436d4301f88543d150b72295` + dirty source. 이 문서는 final source-refresh 후보이며 제품 완료 판정이 아니다. 변경 중인 작업 트리를 읽고 파일 SHA를 즉시 재확인했다(변경0경로). 별도 build/test/Godot/perf는 실행하지 않았다.

## 이전 초안의 stale provenance

이전 draft는 `2026-10-25T16:03:21.363891+09:00`, HEAD `7577f8af08b233ab436d4301f88543d150b72295` 기준이다. HEAD 변경 여부: **False**. HEAD가 같아도 dirty bytes는 달라질 수 있다. 이전 current manifest 대비 added0, changed1, removed0경로이며 정확한 경로·이전/현재 SHA는 새 JSON의 두 snapshot에 보존했다. 이전 UNCOMPILED current 표현과 이전 LOC/hotspot 표를 이 앞부분으로 대체한다. 이전 SHA는 당시 bytes에 대해서만 유효하다.

기존 저장소 CODE_REVIEW 앞부분의12:50/76f572f 기준·113 raw enum·최신 save14/core372 등은 역사적 실행 범위다. 현재 B5 결과로 재표시하지 말고 dated history로 보존한다. 이전 전체 조사 appendix와 원래 PASS/FAIL 기록은 삭제하지 않는다.

## 동일 방법의 owner-facing 비교

셀은 **파일 / 물리 LOC / raw enum**. GOAL base=`67a6232ab41a4bf5051b11303bcac97543b111f0`, published A=`ca9f531ca6ef700ca223d99dba14d834b94b45da`, H=`bdbd4fb2ea9c5b446648778ea7b0b2da14635049`. 역사 열은 기존 git-blob manifest를 그대로 보존했고 현재 열만 재수집했다. `3828938` initial-survey committed base는 JSON의 별도 역사 참고이며 GOAL before 값이 아니다.

| 범위 | GOAL67a6232 | 게시 A | H/pre-B5 | 현재 B5 dirty |
|---|---:|---:|---:|---:|
| core | 88 / 22,101 / 72 | 91 / 22,413 / 76 | 133 / 27,049 / 87 | 136 / 27,690 / 87 |
| engine | 1 / 1,441 / 0 | 1 / 1,446 / 0 | 2 / 1,844 / 0 | 2 / 1,844 / 0 |
| systems | 94 / 35,554 / 9 | 99 / 37,518 / 9 | 105 / 40,020 / 18 | 106 / 40,466 / 18 |
| bridge | 21 / 10,251 / 6 | 27 / 10,990 / 6 | 30 / 11,396 / 6 | 30 / 11,429 / 6 |
| sim-test | 148 / 87,600 / 4 | 152 / 89,058 / 4 | 183 / 95,991 / 8 | 185 / 96,499 / 8 |
| scripts | 51 / 15,999 / 1 | 67 / 18,841 / 1 | 70 / 19,388 / 1 | 70 / 19,391 / 1 |

LOC=`len(text.splitlines())`, 주석·빈 줄·inline test 포함. 동일 정규식 `^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+([A-Za-z_]\w*)`로 선언을 집계한다. cfg(test) 제거·매크로 확장·타입 해석 없음. raw enum은 production content enum 수가 아니다. named Rust source 범위와 scripts만 집계했고 heldout-named paths/data는 열지 않았다. 새로운 전체 correctness audit이나 의미 기반 match census가 아니다.

## B5 소스와 실행 증거 경계

`b5/focused-first-receipt.json`의 core/flora_population/tree_identity/save/b5_flora/tree_snapshot은 모두 exit0이며 core389 unit+38 integration-parent, save23, B5flora6parent+1child, snapshot3 등 범위를 명시한다. `regression-second/clippy.exit`는0이다. `B5_VERIFICATION.md:36`은 ISS301의 catalog boot 누락을 시험 준비에서 수정한 뒤 단독1·steward9/1ignored·data_only_ext2 및 child·save_load7·historical4·workspace all-targets strict clippy와 bridge build 통과를 기록한다. 이는 기존 receipt/doc를 읽은 결과이며 이 작업에서 다시 실행하지 않았다. 첫 실패는 보존한다. 이 실행들이 현재 모든 dirty SHA를 검증했다는 확대 해석은 하지 않는다.

B5는 저장된 Wood MaterialId/default+tile overrides → worldgen/felling → bridge species column → renderer 경로를 추가했다. 독립 renderer species hash만 사용하는 pre-B5 설명은 현재 소스에 맞지 않는다. size/variant 등 시각 hash와 simulation identity는 구분한다. format9 legacy reader/migration과 새 시험 소스도 포함한다. 실제 pixel acceptance·전체 gate·최종 성능은 root가 별도 해당 receipt로 확정한다. 문서의 부분 PASS를 모든 표면 PASS로 합치지 않는다.

다섯 ItemKind/BlueprintKind/RecipeKind/PressureKind/AnimalKind는 private-u32 open ID다. PressureMechanic/AnimalMechanic/blueprint·recipe 행동 분류, GoodsPolicy, legacy wire/test enum을 새 콘텐츠 enum으로 묶지 않는다. 현재 raw declaration 수가 같아도 사용처가 모두 같다는 뜻은 아니다. Top 두 저장 슬롯, Designation, knowledge32노드, fuel1:1 제약은 별도 잔여 부채다. 새로운 종 fixture는 기존 물리 pine의 flora 분포/구성원 추가이며 임의 새 물리 재료나 네 번째 species가 검증됐다는 뜻이 아니다.

## 현재 1,200 LOC 초과 소스

물리 크기 목록이며 측정된 성능 병목 순위가 아니다. JSON에 각 SHA와 enum 위치를 기록했다.

| 경로 | LOC |
|---|---:|
| `rust/crates/sim-systems/src/runtime/decision/system.rs` | 3,487 |
| `rust/crates/sim-test/tests/relations_social_system.rs` | 2,937 |
| `rust/crates/sim-test/tests/engine_influence_channels.rs` | 2,451 |
| `rust/crates/sim-test/tests/settlement_migration.rs` | 2,448 |
| `rust/crates/sim-test/tests/settlement_resource_scarcity.rs` | 2,356 |
| `rust/crates/sim-test/src/probe/colony.rs` | 2,248 |
| `rust/crates/sim-test/tests/agents_memory_system.rs` | 1,985 |
| `rust/crates/sim-test/tests/determinism.rs` | 1,797 |
| `rust/crates/sim-test/tests/settlement_system.rs` | 1,697 |
| `rust/crates/sim-test/tests/chronicle_combat.rs` | 1,616 |
| `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | 1,571 |
| `rust/crates/sim-test/tests/settlement_stockpile.rs` | 1,475 |
| `rust/crates/sim-test/tests/settlement_construction.rs` | 1,460 |
| `rust/crates/sim-engine/src/lib.rs` | 1,455 |
| `rust/crates/sim-test/tests/settlement_fission.rs` | 1,445 |
| `rust/crates/sim-bridge/src/ffi/node.rs` | 1,394 |
| `rust/crates/sim-test/src/perf.rs` | 1,390 |
| `rust/crates/sim-systems/src/runtime/jobs/assign.rs` | 1,361 |
| `rust/crates/sim-test/tests/chronicle_settlement.rs` | 1,322 |
| `rust/crates/sim-systems/src/commands.rs` | 1,293 |
| `rust/crates/sim-test/tests/relations_rule.rs` | 1,263 |
| `scripts/ui/panels/agent_inspector_panel.gd` | 1,244 |
| `rust/crates/sim-bridge/src/ffi/snapshot.rs` | 1,238 |
| `rust/crates/sim-systems/src/runtime/steward/planning.rs` | 1,234 |
| `rust/crates/sim-test/tests/agents_combat.rs` | 1,234 |
| `rust/crates/sim-test/tests/settlement_seasons.rs` | 1,233 |
| `rust/crates/sim-test/tests/agents_sleep.rs` | 1,222 |
| `rust/crates/sim-test/tests/settlement_gather.rs` | 1,220 |
| `rust/crates/sim-test/tests/colony_knowledge.rs` | 1,203 |

## 보존·후속 경계

B4 네 frozen build의 평가와 B5 구현을 혼합하지 않는다. B4 판단·D44·성능은 root의 해당 evidence를 따른다. Stage A의 과거 profile71.007724%를 현재 성능으로 옮기지 않는다. Stage C dead-code 실행 여부는 최종 소스에서 별도 확인하며, settlement/daylight·world_0 UI15bindings·palette/assets·baseline/V7/research·역사 기록은 보호한다. A8 실패, 물리 트랙패드 미검증, UI 및 ISS283 이력은 이 집계로 해결되지 않는다.

통합 시 기존 current front를 dated historical subsection으로 보존하고 이 draft를 대체 앞부분으로 사용한다. root 최종 commit/gate/binary receipt에 맞춰 다시 상태를 묶어야 한다. 이 draft는 gate 통과·게시 승인·성능 개선·행동 동일성 주장이 아니다.

실제 창 후속: 일반 벌목·운반으로 가림을 제거한 원래 그루터기, 실제 두 번째 pine벽과 oak대조를 확인했다. [B5 기록](../mux/B5_VERIFICATION.md)의 실패 이력과 통제 조건을 따른다. 구현 담당 검토자의 픽셀 검토는 소유자 승인이나 독립 맹검 판독이 아니다.

---

# 역사적 중간 검토: 12:50 KST 및 초기 조사

# 코드 검토 2026-10 — B1 조사와 Stage B 구현 중간 검증

**B1 검토 문서 통합 / Stage B IN_PROGRESS / 전체 gate·게시 미완료.** 갱신: **2026-10-25 12:50 KST**. 기준 HEAD는 안전망을 통과한 압력·동물 확장 `76f572fffeb2a2312eb823c9e04a49b9bfc45927`이며 아래 현재 집계는 그 위의 **미커밋 작업 트리**다. 이 문서는 구현 완료나 삭제 승인이 아니다.

## 현재 판정과 읽는 방법

B1의 아키텍처·콘텐츠 enum·하드코딩·baseline·dead code·중복·결정성·성능·시험·문서 검토를 아래 기존 조사 전문과 함께 보존한다. 기존 전문의 경로·행·수치는 해당 조사 시점의 근거다. **현재 상태는 이 앞부분이 우선**하며, 과거의 PENDING, 닫힌 enum, HUD→ArtSpriteLibrary 설명을 현재 사실로 재사용하지 않는다. 전체 파일을 이번에 다시 읽거나 전체 트리 compiler-certified census를 완료했다고 주장하지 않는다.

원본 조사들의 범위·SHA·제외 해소·동적 참조 한계는 뒤의 보존 전문에 있다. B1 도구 실행과 B3 일곱 family 실제 실행 증거는 확보했다. B6 두 도메인 추출의 최종 안전망도 통과했다([검증](../mux/B6_VERIFICATION.md)). 후속 행동 변경과 전체 gate는 남아 있다. 원시 로그·probe·캡처는 저장소에 복사하지 않았다. 외부 증거 루트는 `/Users/rexxa/work/mux-data/`다. 이 절의 링크는 해당 작업 환경의 원본 파일을 가리킨다.

## 실행 증거: 완료 범위와 남은 범위

| 항목 | 확인된 결과 | 근거와 한계 |
|---|---|---|
| 게시 Stage A | clean ca9f531ca gate3 35/35, 10:29:24 KST 종료; 10:30:05 KST wip/main 게시 확인 | 기존 root 기록. Stage B 전체 gate가 아님 |
| 명시 dead-code 검사 | `cargo clippy --workspace --all-targets -- -W dead_code`, exit0, 24.48초 | [B1 receipt](/Users/rexxa/work/mux-data/stage-b/review/b1-tool-receipt.json), [로그](/Users/rexxa/work/mux-data/stage-b/review/clippy-dead-code.log); clean ca9f531ca. GDScript/asset 무사용 증명 아님 |
| 현재 core clippy 재검사 | 두 번째 실행 완료, 22.65초, PASS | [core-clippy-second.log](/Users/rexxa/work/mux-data/stage-b/core-clippy-second.log); root의 성공 결과와 로그 종료행 확인. 첫 실패를 성공으로 덮어쓰지 않음 |
| cargo udeps | **NOT_RUN** | receipt: nightly 및 cargo-udeps 미설치. 의존성이 모두 사용된다는 결론 아님 |
| core / save / historical migration | core372 PASS; 최신 save14 PASS; 앞선 historical migration3 PASS | [integration receipt](/Users/rexxa/work/mux-data/stage-b/integration-test-receipt.json), [save third](/Users/rexxa/work/mux-data/stage-b/save-tests-third.log). 앞선 save15와 최신14는 서로 다른 실행 범위이며 전부 workspace 시험 수가 아님 |
| UI 기능·회귀 검토 | fifth 284 checks, failures0, script_errors0, exit0; scoped visual regression PASS | [UI receipt](/Users/rexxa/work/mux-data/stage-b/ui-fifth-receipt.json), [시각 판정](/Users/rexxa/work/mux-data/stage-b/ui-fifth-visual.json). 기존 한국어 줄바꿈·잘림 ISS274/275는 절대 품질 REVISE; 오너 승인/전체 gate가 아님 |
| core safety1 OFF | 원래 baseline identity + 9 literal OFF summary PASS, differences0, shared values228,748; chronicle64파일/19,683항목 | [receipt](/Users/rexxa/work/mux-data/stage-b/core-safety1-receipt.json), 기록 2026-10-25 11:26:14 KST. summary는 byte 비교; chronicle 기존 비교기는 git_sha만 제외 |
| core ON1 | seeds0–7 ×48일, story/neighbours/steward ON, summary와 command log8개 literal 동일 | [ON receipt](/Users/rexxa/work/mux-data/stage-b/core-on1-receipt.json). OFF9개와 같은 의미의 ON9개 실행으로 확대하지 않음 |
| 최종 lint 반영 source safety2 | **PASS**: 기준선228,748값 차이0, Chronicle64파일, OFF9요약 literal 동일 | [비교 로그](/Users/rexxa/work/mux-data/stage-b/core-safety2-compare.log). 최종 binary `bb9a10a82970bafc0f0d564ae8c88e98c7f4d0f95ac3ad08403ef54da1238845`; 이전 binary의 결과로 대체하지 않음. 최종 ON2도 요약과 명령 기록8개 literal 동일: `core-on2-receipt.json` |

압력·동물 후속 단위는 core378, save19, 기존 story/wildlife/bridge28, novel illness u32::MAX lifecycle1, 실제 과거 저장6/7/8 continuation4가 통과했다. 최신 `ecology-clippy-final.log`의 strict clippy도 통과했다. B3는 별도 프로세스의 실제 제작·건설·학습·한파 효과·사냥과 bundled-before/after, 실제 Godot 경고 클릭·학습 Next를 실행했다(headless39/0, window40/0). collector 실제 full-width code·이름·종류·burn_need·HP·사냥 가능성 assertion도 재실행 통과했다(`ecology-pack-final.log`). 새 포식자 팩은 실제 생성·이동·공격·처치·전리품·압력 종료·저장 계속 실행까지 통과했다(`predator-runtime-first.log`). 새 팩 첫 화면은 제목 잘림/미니맵 준비 부족으로 REVISE였고 둘째는 scoped PASS, 기존 ISS274 줄바꿈은 남는다. 기본 HUD284/0와 독립 회귀 시각 PASS는 [새 검토](/Users/rexxa/work/mux-data/stage-b/ecology-ui-first-visual.json)에 있다.

[런타임 정적 검토](/Users/rexxa/work/mux-data/stage-b/ecology-runtime-audit.md)는 authored 가뭄 배율 상한 결함을 찾았고 검증을 추가했다. 직접 WorldRule의 기존 과도한 배율 문제는 ISS277로 분리했다. [저장 검토](/Users/rexxa/work/mux-data/stage-b/ecology-save-audit.md)는 frozen layout/직접 이행/검증 순서에서 확정 결함을 찾지 못했지만 조작된 동시 mechanic 상태의 별도 검증 공백을 명시한다. M5 보조 UI 시험의 격리 설정 누락 사고 ISS283은 성공한 기능 검사와 별도로 보존하며, 실행 전 설정 사본이 없어 복원은 미확인이다. 외부 설정 경로 guard의 음성 검사와 격리한 실제 창43검사/0실패를 확인했고, 그 재실행 전후 공유 설정 SHA와 수정시각은 동일했다. 재질39·수종352 일치 및 배율220검사도 통과했으며 배율 화면 시각 검토는 별도다.

압력·동물 최종 단위도 원본 identity228748값 차이0·OFF9요약·ON8시드 요약/명령8개 literal 동일을 확인했다([검증](../mux/B_ECOLOGY_VERIFICATION.md)). 기능 시험·save migration·단위 safety가 함께 성공해도 B7 전체 gate를 대신하지 않는다. Stage A의 A8 실패·물리 트랙패드 미검증·ISS030 OPEN 역시 그대로 남는다.

## 현재 규모: 같은 단위로 해석할 것

2026-10-25 12:50 KST에 해당 폴더 아래 `.rs`/`.gd` 텍스트를 열어 물리 줄 수를 집계했다. 주석·빈 줄·inline test를 포함하며 generated macro expansion과 비텍스트 asset은 제외한다. enum은 줄 시작의 선택적 `pub(...)` + `enum Name` 정규식 선언 수다. **cfg(test) 분리/매크로 확장/타입 해석을 하지 않은 raw lexical count**이며 과거 production92와 같은 통계가 아니다.

| 범위 | 파일 | 물리 LOC | raw enum 선언 |
|---|---:|---:|---:|
| sim-core/src | 133 | 26,965 | 86 |
| sim-engine/src | 2 | 1,844 | 0 |
| sim-systems/src | 104 | 39,786 | 17 |
| sim-bridge/src | 30 | 11,396 | 6 |
| sim-test 전체 Rust | 159 | 91,013 | 4 |
| scripts 전체 GDScript | 70 | 19,388 | 1 |

현재 위 Rust 범위 raw 선언은113개다. frozen legacy codec 선언도 포함하며, 이번 증가를 새 콘텐츠 enum으로 해석하면 안 된다. 과거 appendix의 production92/raw95 census는 과거 조사와 범위 분류를 보존한 기록이며 113−92를 새 콘텐츠 enum 수로 해석하면 안 된다. 현재 콘텐츠 `ItemKind`, `BlueprintKind`, `RecipeKind`는 `rust/crates/sim-core/src/colony/content/ids.rs:91,115,134`의 open-ID 선언으로 이동했다. `content/defs/blueprint.rs:8–59`의 placement/completion/nav/policy/material enum과 `defs/recipe.rs:10`의 material term은 동작 분류다. `AnimalKind`와 `PressureKind`도 같은 ids.rs의 open u32 ID로 옮겼다. `PressureMechanic`/`AnimalMechanic` 등은 동작 분류이며 과거 저장 독자를 위한 frozen enum은 역사적 binary schema다. 과거 family별 match/files 표는 이행 전 영향 지도이며 현재 match 수로 재표시하지 않는다.

## 구현 갱신과 우선순위

| 우선 | 현재 문제 / 진척 | 근거 | 다음 조치·위험·행동 경계 |
|---|---|---|---|
| P1 | item/recipe/blueprint registry와 legacy save 첫 단위 커밋 완료; 후속 family 별도 검증 | `content/ids.rs`, `data/colony/items.ron`, `blueprints.ron`, `recipes.ron`; core/save receipt | core safety2/ON2 확인. 후속 단위는 새 안전망·gate 필요; 순서·코드·material·tool wear 보존 |
| P1 | pressure/animal open-ID 및 B3 일곱 family 실제 실행·최종 단위 안전망 통과, B gate 미완료 | MUX_GOAL B3: item, recipe, blueprint/style, pressure, animal, knowledge, alert/lesson | 실제 world/UI 증거와 ecology 최종 binary identity/OFF/ON 확보. 전체 B gate는 대기. resolver 등록만으로 PASS 금지. B2 시간 여유 조항으로 B3 요구를 면제하지 않음 |
| P1 | shelf cutter selector 불일치 발견 후 현재 source 수정 | [base audit](/Users/rexxa/work/mux-data/stage-b/base-data-audit.md), `runtime/jobs/craft.rs:184`는 현재 `knife_uses` | 기존 잘못된 `wildlife.knife_uses`는 bundled4 대신8회 사용. 현재 소스 수정과 해당 실행 binary의 증거를 구분; 과거 진단을 삭제하지 않음 |
| P2 | HUD는 semantic binding으로 이행했으나 원본 raster는 살아 있음 | `scripts/ui/panels/art_ui.gd:76–96`, `assets/v8art/bindings.json:547–673` | 15개 UI binding이 `assets/art/atlas/world_0.png` 사용. atlas 전체 삭제 금지 |
| P2 | 무호출 frame helper와 마지막 legacy library dependency | [current dead-code audit](/Users/rexxa/work/mux-data/stage-b/dead-code-current.md), `wood_visuals.gd:6,30–59` | 8개 helper+ART preload 제거 뒤 library/UID 후보. `daylight_controller.gd:94`의 settlement는 보존. 이 문서는 삭제 실행 아님 |
| P2 | test overlap 및 낡은 scaffold 설명 | `gate.sh:105–113`, `engine_tick_scaffold.rs:1–30,53,75,110` | 실제 세 tick test/역사 golden 유지, 설명 수정. 다른 oracle의 시험을 중복이라 제거하지 않음 |
| P3 | decision 비용·큰 모듈 | 아래 N1000 실측, 기존 아키텍처 지도 | 정렬·phase·keyed RNG 보존한 작은 분리만 후보; 장기 성능 개선 실측 전 개선 주장 금지 |

현재 atlas 참조 경로는 **HUD → ArtUI → ArtData → bindings.ui_icons → world_0.png**다. library는 wood_visuals의 무호출 helper cluster preload로만 남아 있다. `villager_ramp`, palette kit의 보존/ASK 결정은 별도이며 assets 삭제로 확대하지 않는다. `tools/v8/mux_score.py`는 `test_mux_score.py:4`에서 import되고 `test_mux_*`는 gate unittest discovery 대상이므로 dead code가 아니다. 역사 문서는 archive/index 후보이지 삭제 대상이 아니다.

## N=1,000 실측과 시험 건강도 해석

clean Stage A의 frozen release `worldsim_perf stress --seeds 15100..15101 --agents 1000 --side 64 --days 1 --feed fed --rules baseline`: 총1,440tick, warmup 후1,380tick, mean **0.88171087ms**, p99.9 **1.667ms**, max1.687ms, alive1,000. AgentDecisionSystem은 steady863.994297ms, 626.082824µs/call, tick time의 **71.007724%**였다. [B1 receipt](/Users/rexxa/work/mux-data/stage-b/review/b1-tool-receipt.json)의 profile SHA는 `af969d2f2eaacb3c500759588095bb5719535b3bcbdb3e6d36e61f59cd94f086`다. 단일 seed/1일/fed stress이며 colony48일·렌더 FPS·최종 Stage B 성능이 아니다. 역사 M2의80%를 현재 비율로 쓰지 않는다.

기존 slow-target 분석은 아래 전문의 gate1 시점 자료다. 정확한 per-test CPU와 현재 구현의 전체 suite timing은 미측정이다. baseline의 분열·출생·관계·save oracle, V7 worldgen golden 및 M5.5/M-ART2 migration은 현재 기능의 역사 계약을 검증한다. 삭제된 화면 이름을 포함한다는 이유로 이 시험을 지울 수 없다. 8seed×10일처럼 겹쳐 보이는 시험도 command/non-vacuity oracle가 다르다. gate의 재실행을 줄이려면 동일 coverage·실패 전파를 별도로 입증해야 한다.

Baseline은 격리 제안만 유지한다. `baseline_engine`, `baseline_scene`, frozen worldgen, stress constructor, legacy save DTO, `init_production_engine` fixture, V7 golden은 보존한다. `tools/v8/identity.sh:6–7`의 game_v26 alias와 `gate.sh:105–106`의 baseline identity/determinism이 실제 의존한다. world-model 연구 branch의 독립성은 확인되지 않았으며 baseline-only LOC를 혼합 파일 전체 LOC로 대신하지 않는다.

## 기존 전체 조사 보존본 — 아래 내용은 당시 시점의 기록

아래는 아키텍처/시스템/data flow, enum family별 판정과 match 사용처, hard-coded ID 검색, 우선순위별 위험, 시험 건강도 및 provenance를 생략하지 않기 위한 기존 통합본이다. 과거 현재형·PENDING은 당시 상태이며 이 문서 앞부분의 갱신 표가 이를 대체한다. 원본: [stage-b-review-consolidated.md](/Users/rexxa/work/mux-data/stage-b-review-consolidated.md).

# 코드 검토 2026-10 — Stage B1 통합 후보

**B1 IN_PROGRESS / 외부 검토 후보 / NOT_PUBLISHED / Stage B 구현 완료 아님.** 2026-10-25. 저장소 통합 예정 경로는 `docs/v8/review/CODE_REVIEW_2026-10.md`다. 현재 gate3는 clean `ca9f531ca`에서 root가 실행 중이며 이 문서는 결과를 선점하지 않는다. 이 작업에서는 외부 후보 파일만 작성했고 저장소 수정·빌드·시험·Godot·프로파일·Graft 상태 생성·하위 에이전트 실행을 하지 않았다.

## 1. 판정과 근거 범위

원래 `docs/v8/mux/MUX_GOAL.md:75` B1의 요구를 기준으로 기존 아키텍처·사용처·시험 건강도 조사를 통합했다. 새 실행 계획이나 compiler-certified census를 추가 완료 조건으로 요구하지 않는다. 정적 소스 판정은 그 방법과 한계를 공개하고 재사용한다. **남은 실제 B1 실행 증거는 dead_code, 가능한 udeps, N=1,000 시스템별 profile이며 Stage B 구현 후 변경 소스 재검토도 PENDING이다.**

- 원본 Rust/bridge/UI 두 hashed survey 283경로:275개 동일·8개 변경. 변경8개 본문과 기존 제외 steward8파일·2,153줄을 읽은 후속 조사 기준은 clean `6b0f572e4afa0a5d642793b8201386a81838239c`다. 동일275개를 이번에 다시 읽었다고 주장하지 않는다.
- third survey의 dirty `38289384214483e6165dcffea6db04ac05a02e09` 이후50경로는 현재 텍스트47개·13,712줄을 읽고 이미지2개는 해시만, 삭제1개는 부재를 확인했다. 원래 dirty bytes별 SHA가 없어 역사적 exact diff는 복원 불가다.
- production 시스템48개 등록·priority·interval·offset·동일 priority 순서를 정적으로 확인했다. 실측 trace 또는 per-system 성능 결과가 아니다.
- enum 후속 조사 HEAD는 `f38d488f43156145a44898671e77b26b3ee6c496`: 기존81에서 steward4개를 더한 동일범위85개, bridge6개와 production CLI Feed1개를 더해 **production92개**. raw95개 중 test-only3개는 별도 제외했다. 선언·사용처·match 분류를 아래 부록에 유지한다.
- Stage A 현재 커밋 문서는 gate2 `f38d488f43156145a44898671e77b26b3ee6c496`, 2026-10-25 09:07:08 KST exit0,35/35 및 debug/release 각각2060/0/1을 기록한다. gate3·게시 결과로 전용하지 않는다. A8 FAILED, 물리 트랙패드 미검증, ISS030 OPEN(반복73–99ms·시작 포함 약650/646ms), 정지 tick1 미니맵345µs 한계를 그대로 유지한다. UI14 545µs는 독립 최대합의 불확정 이력이며 같은 프레임 비용이나 전체 FPS 판정이 아니다.

### 남은 실행 증거와 갱신 표

| 항목 | 현재 상태 | 근거·root가 채울 결과 | 판정 한계 |
|---|---|---|---|
| workspace `cargo clippy -W dead_code` | **PENDING / root-run** | 명령·소스 SHA·exit·원문 diagnostics 및 각 후보 처리 | 기존 gate clippy PASS를 이 명시 검사 영수증으로 대체하지 않음 |
| `cargo +nightly udeps` | **PENDING / root-run** | 설치/가용성 확인 후 가능하면 실행; 불가하면 이유 기록 | 도구 불가를 unused dependency 없음으로 바꾸지 않음 |
| `worldsim_perf` N=1,000 per-system | **PENDING / root-run** | profile 설정·표본·시스템 비용·decision/system.rs 점유율 | 과거 M2 80%, baseline mean, 정지 UI 비용으로 대체 금지 |
| Stage B 구현 후 전체 변경 source-refresh | **PENDING** | 아래 manifest 대비 변경·추가·삭제 경로, enum/LOC/참조 변화 재검토 | 기존 정적 조사와 최종 구현의 동일성 미입증 |
| 개별 시험 정확 timing / 비용 귀속 | **미측정** | gate1 target wall·13개 slow notice는 있음; 필요 시 root 별도 계측 | binary wall을 함수 CPU나 개별 test duration으로 표현 금지 |
| baseline 연구 의존성·격리 결정 | **미확인 / owner decision 후보** | identity 경로는 보호; world-model 연구 의존성 별도 확인 | baseline 삭제 승인 없음 |

정적 enum census는 lexical+명시적 소스 추론이다. comments/strings/cfg(test) 제외, inherent Self와 직접 variant glob을 처리했고 primary15 family에는 명시 alias/glob이 없음을 확인했다. macro 확장·임의 reexport·완전한 rustc 타입 해석을 수행하지 않았다. 이 한계는 공개하되 **추가 compiler 인증을 선행 차단 조건으로 만들지 않는다**. 새 구현에서 관련 사용처가 바뀌면 해당 경로를 다시 검토한다.

### 입력과 provenance

[기존 종합 초안](/Users/rexxa/work/mux-data/CODE_REVIEW_2026-10-draft.md), [소스 갱신](/Users/rexxa/work/mux-data/stage-b-review-refresh.md), [enum census](/Users/rexxa/work/mux-data/stage-b-enum-use-census.md), [시험 건강도](/Users/rexxa/work/mux-data/stage-b-test-health.md)를 보존했다. 상세 per-file SHA·행·원문은 동명의 JSON 및 refresh-systems/third-survey JSON에 있다. Graft는 Rust caller 누락/Godot 미포함 때문에 방향 파악에만 사용된 원본 자료이며 이번 작업은 호출하지 않았다. 아래 SHA는 통합 시 읽은 입력 자체의 bytes를 고정한다.

| 입력 | SHA256 |
|---|---|
| [CODE_REVIEW_2026-10-draft.md](/Users/rexxa/work/mux-data/CODE_REVIEW_2026-10-draft.md) | `0c1c21e7ab30faeddce3c4cade00dbd13f02add07f38853e8f4b013a73ea132c` |
| [stage-b-review-refresh.md](/Users/rexxa/work/mux-data/stage-b-review-refresh.md) | `0da905cefc9da72a14badeccc066e0ebdc0a28e2b964686fe55dbd9134352c4a` |
| [stage-b-review-refresh.json](/Users/rexxa/work/mux-data/stage-b-review-refresh.json) | `f01aed8b18527a398edad11f219b474373082a448d944d31c73de518cff242b3` |
| [stage-b-review-refresh-systems.json](/Users/rexxa/work/mux-data/stage-b-review-refresh-systems.json) | `438d347ca8eac7cf30bac8fd5f15f599e4ad9309d2c299c9e1b596bf89e43bf9` |
| [stage-b-review-refresh-third-survey.json](/Users/rexxa/work/mux-data/stage-b-review-refresh-third-survey.json) | `8db837e4ce627624631c919f381906dd7506be8d992ff6b2779aead73fcdb154` |
| [stage-b-enum-use-census.md](/Users/rexxa/work/mux-data/stage-b-enum-use-census.md) | `f9164a8459c9253d4876797e5412de1fe72a2edd9e99cd3b6647b41dce1b7d8d` |
| [stage-b-enum-use-census.json](/Users/rexxa/work/mux-data/stage-b-enum-use-census.json) | `6a734044a3f9f927b50834c7e2c054690babd25804f24140fb244daec2756cfb` |
| [stage-b-test-health.md](/Users/rexxa/work/mux-data/stage-b-test-health.md) | `1e536cf937d4974bc189b58a24e33682e5de9ed87a06098bd7621728399b5a66` |
| [stage-b-test-health.json](/Users/rexxa/work/mux-data/stage-b-test-health.json) | `b03d5b3ea93017ff44c079b511f19663fea697a8ecb0df8c667c94bc66132a99` |

## 2. 아키텍처와 데이터 흐름

줄 수는 주석과 inline test를 포함한다. 다음은 원본 survey 수치다. 이번에 제외 steward를 포함한 Rust 범위는 core91파일·22,410줄, systems99파일·37,518줄이며 engine1파일·1,446줄은 동일하다. 이는 해당 crate 범위이며 제품 전체 LOC는 아니다.

| 영역 | 파일 | 물리 줄 수 | 역할 |
|---|---:|---:|---|
| sim-core | 88 | 22,122 | 직렬화 타입·결정적 기본형·규칙·콘텐츠 |
| sim-systems | 94 | 35,653 | worldgen·명령·시스템·save adapter |
| sim-engine | 1 | 1,446 | Resources/ECS·tick·시스템 정렬·계측 |
| sim-bridge/src | 27 | 10,990 | 읽기 collector·Godot marshalling·명령 진입점 |
| scripts/core | 3 | 620 | Locale·선택·SimReads |
| scripts/ui/hud | 3 | 1,359 | 상단 HUD·명령 막대·story banner |
| scripts/ui/panels | 14 | 4,757 | inspector·room·tech·teach·trade 등 |
| scripts/ui/mux | 15 | 2,404 | alerts·learning·goals·controls·settings |
| scripts/ui 직접 파일 | 8 | 1,464 | 표시 문자열·아이콘·읽기 cache |
| scripts/world/art | 7 | 2,189 | manifest·material/flora resolver·baker |
| scripts/world/colony | 11 | 5,114 | terrain/person/tree/animal/order/room layer |
| scripts/world 직접 파일 | 6 | 808 | camera·daylight·legacy sprite library |
| shaders / scenes | 5 / 1 | 341 / 88 | 표현·main scene wiring |
| sim-test | 152 | 89,058 | unit·integration·probe·perf·공통 fixture |

Assets는 import/uid 제외 1,250파일,28,608,450bytes를 inventory했다. 이것은 시각 검토나 각 asset 실행 증거가 아니다.

`core 정의/registry → systems worldgen·ordinary command → engine Resources/ECS → 정렬된 runtime → bridge collector → SimReads cache → UI/renderer`가 기본 방향이다. UI는 읽고 명령을 보낸다. art binding은 시뮬레이션의 두 번째 권위가 되어서는 안 된다.

| 패스 | 입력·상태·출력 흐름 |
|---|---|
| M1 | OrderTool→SimReads.mutate→WorldSimNode→ordinary commands; reservations/jobs/overlay/trees를 layer로 읽음 |
| M2 | bills/recipes/materials/items→workshop/economy snapshots→workshop panel·EconomyText |
| M3 | construction·room topology→get_space/get_rooms→room layer·material/style drawing |
| M4 | knowledge gates/discovery→get_tech/waits/teach orders→tech/teach panel |
| M5 | storyteller·animals·draft·mood·apparel→story/animal/person collectors→world/UI |
| M5.5 | neighbour waves/caravans/trade→neighbours collectors→migrant/trade dialogs |
| M-ART2 | simulation 코드/물성→manifest/bindings/material/flora resolver→draw; HUD 명칭 표는 아직 별도 |
| MUX | bulk ui_facts·steward read/command→alerts/lessons/goals/settings; 변경8개·steward8개 정적 재검토, 실행 증거 별도 |

`SimReads`는 getter/mutator allowlist, tick cache, mutation/reset 무효화를 맡는다. 여러 person 컬럼을 합칠 때는 AgentId 정렬과 배열 길이·동일 행 의미를 보존해야 한다. 모듈 크기 자체가 결함은 아니다: decision/system.rs3,494줄, settlement_system.rs1,571줄, snapshot.rs1,481줄, node.rs1,373줄은 분할 후보이지 측정된 병목 판정이 아니다.

### 2.1 현재 소스 추가 검토

변경8개는 SimReads의 동기 world replacement receipt/cache 무효화, reset 전용 새 게임 설정, HUD의 deferred signal 중복 처리와 steward sequence queue, 명령 막대의 player/background action 분리, locale tooltip 갱신, authoritative winter need 합산, 외부 시험 settings 경로다. [파일별 범위와 SHA](/Users/rexxa/work/mux-data/stage-b-review-refresh.json)에 고정했으며 정적 읽기를 회귀 PASS로 해석하지 않는다.

기존 제외 steward8개는 이제 읽었다. 기본 OFF 상태·scope/cooldown/veto·ordinary command/replay를 확인했고, 완공 role 처리의 raw player Auto/Nothing 이력과 모든 실제 건설 경계 소유권 검사를 읽었다. rules의 Action/Condition/Target/Scope는 mechanics 후보지만 내부 ItemKind/BlueprintKind/RecipeKind/AnimalKind/RoomRole 참조는 B2 데이터 이행 범위다. 후속 production 92개 선언·사용처 census와 함께 읽으며, 컴파일러 이름 해석의 완전성을 주장하지 않는다.

third tests/tools/docs survey에는 파일별 이전 SHA가 없다. 그 dirty HEAD부터 현재까지50경로 commit delta는 당시 읽은 bytes를 재구성하지 못한다. 후속으로 현재47텍스트13,712줄 전체를 읽었고 PNG2개는 해시, 삭제PNG1개는 부재 확인했다. 모든50개 상태/해시는 앞선 refresh와 동일하다. [세부 manifest](/Users/rexxa/work/mux-data/stage-b-review-refresh-third-survey.json). 현재 목록의 미독 공백만 해소했으며 역사적 dirty baseline의 정확성이나 전체 B1 완료를 주장하지 않는다.

## 3. 콘텐츠 enum 판정과 하드코딩

**content**는 새 종류를 데이터만 추가해 표현해야 하는 목록이다. **mechanics/event/protocol**은 실행 상태·명령·사건 schema이며 enum 유지가 가능하다. **hybrid**는 콘텐츠 ID와 닫힌 동작 category를 분리해 사용처를 추가 판정해야 한다. 아래 부록은 lexical 추출 결과를 보존하며 exhaustive compiler-resolved match 수라고 부르지 않는다.

필수 B2 우선순위는 ItemKind·BlueprintKind/건축 style·RecipeKind/workshop이다. AnimalKind·PressureKind·Top은 content, Era·Lane·Modifier·NodeKind도 리뷰에서 content 후보로 분류했다. Designation·SkillKind·WorkType·RoomRole·TerrainType·BuildingKind·ResourceKind는 hybrid이며 단순히 모든 enum을 제거하는 방식은 부적절하다. ChronicleKind/CausalEvent는 사건 schema, JobKind/JobStage/BillMode/RejectReason은 mechanics다.

기존 `.code()` 값과 Serde/bincode enum 표현은 별개의 계약이다. 새 ID 타입에 같은 숫자를 넣어도 기존 저장·명령 바이트가 자동 호환되지 않는다. 선택한 codec 계약에 따라 기존 enum discriminant와 유효한 raw-code domain의 실제 바이트 호환을 검증하고, 호환되지 않는 mixed Chronicle cause 등에는 명시적 legacy migration을 적용한다. 모든 필드에 무조건 mirror를 추가하는 것이 아니다. M5.5/M-ART2 저장 continuation과 command replay가 필요하다.

| 어휘 검색 | 범위·결과 | 한계 |
|---|---|---|
| Rust | 후보 ID757개,69일치/10파일 | 세 crate, tests 포함, 전체 줄 주석 제외; enum 분기와 파일 include catalogue는 별도 |
| bridge/UI | manifest/material/style/binding/flora ID855개,39일치/6파일 | 두 quote 형식; schema/category 문자열도 일치하므로39개 위반이 아님 |
| 추가 RON ID | explicit id35개,37 quoted matches | 완전 RON parser 아님; 연결 문자열 평가 안 함 |

확인된 중복 catalogue는 EconomyText의 ITEM_NAMES17·RECIPE_NAMES11·BUILD_KINDS12와11개 고정 메뉴 슬롯, KnowledgeText의24개 지식 이름, StoryText의 압력/동물/의복 이름이다. Rust registry→bridge의 stable code/id/name_key/category/capability/menu metadata를 UI가 읽어야 데이터 추가가 실제 조작까지 이어진다. 기존 grep은 scripts/world와 double quote에 치우쳤다. UI·shader·single quote·concatenation의 실패를 심은 negative control 없이 B3 완료로 판정하지 않는다.

## 4. 우선순위별 문제·수정 제안

아래 '보존'은 구현 의도다. 아직 어느 제안도 이 리뷰로 행동 보존이 입증된 것은 아니다.

| 우선 | 문제 | 근거(리뷰 당시 소스 위치) | 수정 방향 | 위험 | 보존/변경과 필요한 증거 |
|---|---|---|---|---|---|
| P1 | item/recipe/blueprint가 닫힌 목록; UI도 별도 목록 | core colony/workshop.rs237–270; orders.rs238; economy_text.gd26–73; order_bar.gd50,208,324 | versioned validated registry, stable ID, bridge metadata, category fallback | 저장·명령·순서·hotkey | 기존 콘텐츠는 보존; 각 family identity+9 fixture literal summary; 새 메뉴 overflow는 별도 UI 동작 검증 |
| P1 | code() 일치가 저장 호환을 보장하지 않음 | items.rs9; workshop.rs93; save/mod.rs6,99 | 실제 wire/domain 호환 증명; 비호환 mixed payload는 legacy decoder→명시 변환 | 과거 저장/command 오독 | 기존 continuation 보존; migration·replay·stable code 시험 |
| P1 | 완공 wall/floor의 style이 축약됨 | ffi/space.rs274,302; jobs/space.rs64 | 완료·save·bridge에 authoritative style/category 유지 | 철거 수율·저장 schema | 기존 보존; mud brick place→complete→save/load→render까지 시험, art resolver 단독으로 충분하지 않음 |
| P1 | permissive parser와 중복 ID/unknown field 검증 공백 | workshop.rs215,221,259–270; materials.rs66; art_data.gd25–49 | schema/version/ID/ref/bounds 엄격 검증; 정상 unknown content fallback과 malformed 입력 구분 | startup/error 행동 변경 | 정상 bundled data 보존과 오류 정책 변경을 구분해 기록 |
| P1 | B3 gate가 UI/shader/연결 문자열을 못 봄 | test_mart2_extensibility.py21–22,79 | scope/literal 검사 확장, category 허용 목록, negative controls | 오탐·누락 | tooling 변경; 기존 gate PASS를 새 범위 PASS로 재사용 금지 |
| P2 | knowledge ID가 리스트 위치이며32개 제한 | knowledge.rs24,385–404 | 기존 ID 고정·중복/overflow/ref/order 검증 | reorder가 저장 의미 변경 | append 제한 내 B3 시험; bitset 확장은 별도 migration |
| P2 | topbar가 old atlas에 실제 의존 | hud_topbar.gd76; art_ui.gd6,26; art_sprite_library.gd7,59 | semantic HUD icon binding→ArtData, 이후 reachability 재감사 | 아이콘·layout·fallback | 초기 raster/32px layout 보존,3 UI scale 실제 캡처 후 삭제 판단 |
| P2 | 동적 참조 검사가 불완전 | gd_reach.py17–39; manifest 상대 경로/RON | quote/dynamic/resolver fixture와 수동 entrypoint 확인 | 잘못된 삭제 | reference 없음 단독 삭제 근거 금지 |
| P2 | 일부 wood helper와 문서 참조가 낡음 | wood_visuals.gd30–58; inventory.py91; agents_snapshot.rs314 | cluster 단위 caller 재검토; stale comment 수정 | callback/보존 정책 누락 | wood_visuals.settlement은 live; 전체 파일 삭제 금지 |
| P2 | 거리·test helper 중복 | nav.rs47/spatial.rs37/jobs/items.rs14; relations_rule.rs48–60 vs relations_caps.rs44–56 | 동일 함수만 공유, 입력·tie tuple·constness 보존 | 선택 순서/fixture 결합 | 집중 회귀+전체 B 안전망 |
| P2 | gate가 일부 suite를 재실행 | gate.sh95–98,111,114,137,150 | 완전 run 결과를 이름별로 재사용하거나 의도적 중복 유지 | 실패 전파·verification coverage 변경 | coverage 동등성·실패 control 없이 시간절약 주장 금지 |
| P2 | test header/문서가 삭제된 harness를 가리킴 | engine_tick_scaffold.rs4–30; sim-test/src/lib.rs/scenes.rs | 실제3개 tick checks와 frozen baseline 설명으로 수정 | 역사 손실 | 설명 수정만; 시험 삭제 금지 |
| P3 | 큰 모듈·여러 person 순회·space 전체 scan | decision/system.rs; node.rs442; space.rs267; ui_facts/people.rs59 | helper 이동·정렬 projection·version cache를 측정 후 분리 | 행 정렬·stale cache·tick phase | N1000/실제 UI 계측 먼저; module 이동과 의미 변경 분리 |
| 결정 | baseline isolation·palette kit 보존 | ADR0011/0014; inventory.py81,100; V8_0_INVENTORY514 | 기존 interface 유지한 격리 제안 및 owner decision 기록 | identity·연구·수동 진입점 | baseline 삭제 금지; 미확인 연구 독립성 주장 금지 |

## 5. baseline·실사용 atlas·삭제 경계

`identity.sh`는 game_v26 baseline alias를 사용한다. `baseline_engine`, `baseline_scene`, frozen worldgen, stress constructor, `init_production_engine` lattice fixture, 옛 save DTO와 `sim-core/tests/data/v7_final_worldgen_seed*.txt`는 보호한다. `baseline_worldgen_matches_v7_final.rs`와 `identity_removed_keys.txt`의 제거 필드0검사도 보존한다. main scene이 colony를 쓴다는 사실은 이 경로의 불필요성을 뜻하지 않는다. world-model 연구 branch/tag의 의존성은 이 세 리뷰에서 확인하지 않았으므로 격리는 owner decision 후보이며 삭제 승인이 아니다. 혼합 파일 전체 LOC를 baseline-only LOC로 표기하지 않는다.

old atlas의 live 경로는 `hud_topbar→ArtUI→ArtSpriteLibrary→assets/art/atlas/world_atlas.json→world_0.png`다. library의 동적 variant 경로와 villager_ramp도 보수적 보존 범위다. topbar를 manifest로 이관하기 전 atlas 삭제는 금지한다.

`palette_swap.gdshader`는 정적 runtime closure 밖이지만 inventory의 ASK/보존 결정이 남아 있다. `palette_lut.png`·`agent_base.png`도 자동 삭제 대상이 아니다. wood_visuals의 frame helper cluster는 정적 무호출 후보이며 `daylight_controller→WOOD.settlement`은 live다. exported Rust #[func], Godot signal/string callback, manifest-loaded assets, .uid/.import는 직접 문자열 호출이 없다는 이유로 삭제할 수 없다.

현재 **삭제 승인된 파일은0개**다. tools의 mux_score/test_mux_*는 dotted import·unittest discovery 때문에 filename 검색에서 누락된 false candidate다. p3의 오래된 audit 문서는 archive/index 후보이며 역사 삭제 대상이 아니다.

## 6. 결정성과 성능

engine은 stable priority 정렬을 사용하므로 같은 priority의 등록 순서를 유지해야 한다. tick 증가·interval 검사·명령 처리 phase 이동은 단순 파일 정리가 아니다. HashMap/HashSet 존재 자체는 결정성 버그가 아니다. 선택·ID 부여·구조 변경·직렬화에 쓰이는 순회는 정렬과 전체 tie-break tuple을 검증해야 한다. snapshot/settlement/resource에서 명시 정렬이 관찰됐다.

material 초기화의 f64 파싱/정수화는 현재 존재하는 초기화 동작이며 tick 내 float 위반의 증거와 구별한다. shader float·animation time·Instant 계측도 sim-state nondeterminism의 자동 증거가 아니다. 이 검토는 전체 float/time 데이터 흐름 분석을 완료하지 않았다.

N=1,000 worldsim_perf per-system 결과가 없다. 목표에 적힌 M2 decision 80%는 역사적 수치이며 현재 점유율로 복사하지 않는다. paused UI 평균 FPS는 활성 simulation 비용을 입증하지 않는다. get_space 전체 지도 scan, ui_facts 중첩 collectors, household의 settlement×people 순회, lesson_highlight의0.1초 읽기는 **측정할 후보**다. alert evaluator 2Hz 제한과 bulk reader의 다른 호출 빈도는 서로 다르며 최신 learning 변경을 확인해야 한다.

## 7. 시험·문서 건강도

sim-test의1,408개 literal #[test]는 Cargo-discovered/실행/PASS 개수가 아니다. tools/v8의11 test_*.py와91 literal test method,19 *test*.gd도 자동 실행 보증이 아니다. workspace debug/release와 선택 suite의 중복은 의도적 검증일 수 있으므로 동등한 실패 전파를 증명하기 전 제거하지 않는다.

완료된 clean `6b0f572e4` gate1 로그를 후속 분석했다([시험 건강도](/Users/rexxa/work/mux-data/stage-b-test-health.md), [전체116×2 target/원문/SHA JSON](/Users/rexxa/work/mux-data/stage-b-test-health.json)). debug/release 각각2059 PASS·1 FAIL·1 ignored, phase wall2112/205초, Cargo build48.32/67초, target elapsed 합1996.15/103.69초다. debug11타깃이60초 초과이며 개별 시험13개에 >60초 알림이 있고 release에는 없다. 이는 binary 단위 wall duration이며 `--test-threads=8`의 정확한 per-test 시간·CPU profile은 여전히 미확보다.

상위 debug target은 colony_items283.26초, relations_caps272.58초, colony_world268.98초, save_load162.71초다. 느린11개 current source SHA는 gate1과 동일하다. V7 baseline의 분열·출생·관계·저장 호환 oracle는 현재 보존 계약이며 역사 이름만으로 dead code가 아니다. 8seed×10일 overlap도 quarry command/non-vacuity oracle가 다르므로 삭제나 동등성은 입증되지 않았다. ignored v6 migration은 외부 fixture가 필요하며 PASS가 아니다.

gate1 실패는 Chronicle discriminant count left46/right45이다. 후속 `f38d488f4` fixture 수정 및 coordinator의 targeted PASS 보고와 구분하며 이후 gate2는 같은 수정 소스에서 35/35 exit0, debug/release 각각2060/0/1로 완료됐다. gate1 실패·시간 수치는 역사적 관측으로 보존한다. 이 분석에서 새 실행은 없었다. 과거 선택 시험/임시179.10초 diagnostic을 현재 suite 최장 시험으로 발표하지 않는다. dead code/udeps·N=1000 profile 완료는 주장하지 않는다. enum 조사 수준은 아래 후속 census의 정적 검색·소스 판정 범위다.

확인된 문서 차이는 engine_tick_scaffold의 사라진 renderer/harness 설명, baseline을 live world라 부르는 설명, wood_visuals/inventory의 삭제된 renderer/layer 이름, palette_swap 삭제 주장이다. 현재50경로 본문 refresh에서 ARCHITECTURE의 오래된 API/schema 숫자·offset 없는 도식·Steward Writes 누락과 MUX 상단 ledger의 과거 상태를 확인했다(뒤 보충 표 참조). 이번 통합에서 현재 README·ROADMAP·STAGE_A_REPORT의 gate2/미게시 정정은 별도로 확인했다. 이 확인을 전체 트리 재조사로 확대하지 않는다. 역사 보고서의 실패 bar는 최신 결과로 덮어쓰지 않는다.

## 8. B2–B7 실행에 넘길 조건

1. clean6b0f572e4에서 두 hashed inventory 대조, steward 제외 해소, production48개 priority/interval/offset/order 확인을 완료한 정적 범위를 재사용한다. 최종 Stage A 공개 source와의 추가 차이 및 third survey의 dirty snapshot 공백은 별도로 재검토한다.
2. 각 behavior-preserving commit 전후에 baseline byte identity와 steward OFF의 soak/story/neighbours/knowledge F10/M5.5 seeds0–7×48일 **9 fixture literal summary**를 비교한다. 이 초안은 어느 refactor의 안전망 PASS도 대신하지 않는다.
3. 안정 ID·legacy decode·registry adapter부터 family별로 이행한다. item/recipe/blueprint/style은 필수이며 known content 순서·코드·행동을 유지한다.
4. B3 ext_pack은 item·recipe·mud-brick blueprint/style·pressure·animal·knowledge·alert/lesson이 실제 world/UI에서 쓰이는 경로를 시험한다. resolver 단독 등록만으로 완료하지 않는다.
5. B4 population acceptance cap/trader weighting/hearth meat fallback/F4·F9 arithmetic과 B5 tree species는 behavior-changing 작업이다. 사전 예상 효과·별도 commit·fixture diff를 남기며 보존 리팩터와 혼합하지 않는다.
6. root가 직렬 실행하는 clippy dead-code/가능한 udeps·N1000 profile 결과와 후속 소스 차이를 반영해 B1 상태를 갱신한다. 기존 정적 census를 재활용하며 별도의 compiler-certified census를 새 완료 조건으로 추가하지 않는다. 실패·미실행 결과를 완료로 바꾸지 않는다.
7. 최종 B gate·migration·data_only_ext와 source refresh 후 root가 저장소 보고서로 통합한다. 이 파일은 공개나 삭제를 승인하지 않는다.

## 부록 A. 현재 production 등록 시스템 — 정적 확인

clean HEAD `6b0f572e4afa0a5d642793b8201386a81838239c`의 **48개**다. 테스트 AtomicMock를 제외하고 Steward를 포함한다. `worldgen/mod.rs:202` → `sim-systems/src/lib.rs:302`의 helper 등록을 따라갔으며 `sim-engine/src/lib.rs:995`의 stable priority sort 때문에 같은 priority는 등록 순서를 보존한다. 실행 순서는 due인 시스템 사이의 순서이며 매 tick 모두 실행된다는 뜻이 아니다. 실제 runtime trace/프로파일은 수집하지 않았다.

기본 offset0은 engine trait 기본값이다. F = `(720 + 1440 - clock_offset_ticks % 1440) % 1440`. O = `(oral_minute % 1440 + 1440 - clock_offset_ticks % 1440) % 1440`이고 rule 부재 fallback minute는1230이다. engine tick loop1070행은 interval/offset으로 due를 결정한다. 모든 등록 interval은 양수다. [상수·메서드·SHA 증거](/Users/rexxa/work/mux-data/stage-b-review-refresh-systems.json).

| 실행 순서 | System | 등록 순서 | Priority | Interval(ticks) | Offset | 선언 |
|---:|---|---:|---:|---:|---|---|
| 1 | BuildingStampSystem | 1 | 90 | 1 | 0 | `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs:64` |
| 2 | InfluenceUpdateSystem | 2 | 100 | 1 | 0 | `rust/crates/sim-systems/src/runtime/influence/update.rs:225` |
| 3 | AgentInfluenceSampleSystem | 3 | 110 | 1 | 0 | `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs:54` |
| 4 | AgentMovementSystem | 5 | 120 | 1 | 0 | `rust/crates/sim-systems/src/runtime/agent/movement.rs:270` |
| 5 | DraftSystem | 41 | 121 | 1 | 0 | `rust/crates/sim-systems/src/runtime/wildlife/draft.rs:170` |
| 6 | BillSystem | 30 | 123 | 60 | 0 | `rust/crates/sim-systems/src/runtime/jobs/bills.rs:184` |
| 7 | FleeSystem | 42 | 123 | 3 | 0 | `rust/crates/sim-systems/src/runtime/wildlife/flee.rs:120` |
| 8 | JobAssignSystem | 31 | 124 | 1 | 0 | `rust/crates/sim-systems/src/runtime/jobs/assign.rs:962` |
| 9 | AgentDecisionSystem | 6 | 125 | 1 | 0 | `rust/crates/sim-systems/src/runtime/decision/system.rs:120` |
| 10 | StewardSystem | 32 | 125 | 360 | 0 | `rust/crates/sim-systems/src/runtime/steward/mod.rs:15` |
| 11 | StaleSeekTargetSystem | 17 | 126 | 1 | 0 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs:567` |
| 12 | JobExecuteSystem | 33 | 127 | 1 | 0 | `rust/crates/sim-systems/src/runtime/jobs/execute.rs:839` |
| 13 | HungerDecaySystem | 7 | 130 | 1 | 0 | `rust/crates/sim-systems/src/runtime/needs/hunger_decay.rs:48` |
| 14 | ThirstDecaySystem | 8 | 131 | 1 | 0 | `rust/crates/sim-systems/src/runtime/needs/thirst_decay.rs:49` |
| 15 | SleepDecaySystem | 9 | 132 | 1 | 0 | `rust/crates/sim-systems/src/runtime/needs/sleep_decay.rs:54` |
| 16 | ConstructionSystem | 11 | 133 | 1 | 0 | `rust/crates/sim-systems/src/runtime/construction/construction_system.rs:102` |
| 17 | SocialInteractionSystem | 12 | 134 | 1 | 0 | `rust/crates/sim-systems/src/runtime/social/social_interaction_system.rs:103` |
| 18 | SocialDecaySystem | 10 | 135 | 1 | 0 | `rust/crates/sim-systems/src/runtime/needs/social_decay.rs:56` |
| 19 | MemorySystem | 13 | 136 | 1 | 0 | `rust/crates/sim-systems/src/runtime/memory/memory_system.rs:74` |
| 20 | CombatSystem | 14 | 137 | 1 | 0 | `rust/crates/sim-systems/src/runtime/combat/combat_system.rs:66` |
| 21 | SettlementSystem | 15 | 138 | 1 | 0 | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs:1350` |
| 22 | StarvationSystem | 16 | 139 | 1 | 0 | `rust/crates/sim-systems/src/runtime/survival/mod.rs:205` |
| 23 | ResourceRegenSystem | 18 | 140 | 120 | 0 | `rust/crates/sim-systems/src/runtime/resource_regen/mod.rs:399` |
| 24 | StockpileDepositSystem | 19 | 141 | 10 | 0 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_deposit.rs:62` |
| 25 | StockpileConsumeSystem | 20 | 142 | 5 | 0 | `rust/crates/sim-systems/src/runtime/settlement/stockpile_consume.rs:82` |
| 26 | ShelterPlanningSystem | 21 | 143 | 1 | 0 | `rust/crates/sim-systems/src/runtime/settlement/shelter_planning.rs:23` |
| 27 | MaturationSystem | 22 | 144 | 60 | 0 | `rust/crates/sim-systems/src/runtime/agent/maturation.rs:17` |
| 28 | PairingSystem | 23 | 144 | 1 | 0 | `rust/crates/sim-systems/src/runtime/agent/pairing.rs:20` |
| 29 | BedAssignmentSystem | 24 | 145 | 1 | 0 | `rust/crates/sim-systems/src/runtime/agent/beds.rs:27` |
| 30 | AgingSystem | 25 | 146 | 60 | 0 | `rust/crates/sim-systems/src/runtime/agent/aging.rs:146` |
| 31 | HearthFeedSystem | 34 | 146 | 60 | 0 | `rust/crates/sim-systems/src/runtime/jobs/hearth.rs:177` |
| 32 | FireSystem | 27 | 147 | 60 | 0 | `rust/crates/sim-systems/src/runtime/settlement/fire.rs:99` |
| 33 | FissionSystem | 28 | 148 | 1 | 0 | `rust/crates/sim-systems/src/runtime/settlement/fission.rs:1102` |
| 34 | RelationsSystem | 29 | 149 | 1 | 0 | `rust/crates/sim-systems/src/runtime/settlement/relations.rs:181` |
| 35 | FertilitySystem | 26 | 150 | 1440 | F | `rust/crates/sim-systems/src/runtime/agent/fertility.rs:101` |
| 36 | RoomSystem | 35 | 152 | 1 | 0 | `rust/crates/sim-systems/src/runtime/rooms.rs:229` |
| 37 | OralTransmissionSystem | 36 | 160 | 1440 | O | `rust/crates/sim-systems/src/runtime/knowledge/oral.rs:50` |
| 38 | ObservationSystem | 37 | 161 | 30 | 0 | `rust/crates/sim-systems/src/runtime/knowledge/observe.rs:51` |
| 39 | KnowledgeDecaySystem | 38 | 162 | 60 | 0 | `rust/crates/sim-systems/src/runtime/knowledge/decay.rs:29` |
| 40 | DiscoverySystem | 39 | 163 | 240 | 0 | `rust/crates/sim-systems/src/runtime/knowledge/discover.rs:103` |
| 41 | TechAggregateSystem | 40 | 164 | 240 | 0 | `rust/crates/sim-systems/src/runtime/knowledge/aggregate.rs:147` |
| 42 | StorytellerSystem | 43 | 170 | 1440 | 0 | `rust/crates/sim-systems/src/runtime/story/mod.rs:717` |
| 43 | AnimalSystem | 44 | 171 | 3 | 0 | `rust/crates/sim-systems/src/runtime/wildlife/mod.rs:719` |
| 44 | MoodSystem | 45 | 172 | 60 | 7 | `rust/crates/sim-systems/src/runtime/mood/mod.rs:605` |
| 45 | QuarrelSystem | 46 | 173 | 6 | 3 | `rust/crates/sim-systems/src/runtime/mood/quarrel.rs:117` |
| 46 | NeighbourSystem | 47 | 174 | 1440 | 0 | `rust/crates/sim-systems/src/runtime/neighbours/mod.rs:442` |
| 47 | VisitorSystem | 48 | 175 | 3 | 1 | `rust/crates/sim-systems/src/runtime/neighbours/visitors.rs:103` |
| 48 | InfluenceVisualizationSystem | 4 | 1000 | 6 | 0 | `rust/crates/sim-systems/src/runtime/influence/visualization.rs:62` |

**확정 문서 드리프트:** `sim-systems/src/lib.rs:326`의 Quarrel 30틱 주석과 실제 `runtime/mood/quarrel.rs:44,126,131`의 interval6·offset3이 다르다. 소스 수정 없이 기록했다.


## 부록 B. production enum 판정·사용처 집계 갱신

이 부록은 원본의 좁은81개 lexical 표를 대체하는 후속 조사다. 원본81개 표는 원본 파일에서 보존한다. 아래 숫자는 production bridge/probe를 포함하고 tests를 제외한다. mention은 선언/import/type 위치도 포함하며 전부 행동 분기라는 뜻이 아니다. pattern/guard match는 arm의 family 참조, reference match는 RHS 생성/변환도 포함하는 가장 안쪽 match다. `matches!`와 if/while-let은 별도다. 따라서 서로 다른 열이나 이전 survey를 단순 빼서 제거량으로 발표하지 않는다.


|Enum|Class|Definition|Rationale|
|---|---|---|---|
|FfiFieldValue|mechanic/protocol|rust/crates/sim-bridge/src/ffi/causal.rs:579|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|RectCommand|mechanic/protocol|rust/crates/sim-bridge/src/ffi/colony.rs:18|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|EconomyCommand|mechanic/protocol|rust/crates/sim-bridge/src/ffi/economy.rs:35|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|NeighbourCommand|mechanic/protocol|rust/crates/sim-bridge/src/ffi/neighbours.rs:424|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|SpaceCommand|mechanic/protocol|rust/crates/sim-bridge/src/ffi/space.rs:31|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|StoryCommand|mechanic/protocol|rust/crates/sim-bridge/src/ffi/story.rs:744|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|DissolutionCause|event/protocol|rust/crates/sim-core/src/causal/event.rs:38|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|DeathReason|event/protocol|rust/crates/sim-core/src/causal/event.rs:63|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|DecisionReason|event/protocol|rust/crates/sim-core/src/causal/event.rs:119|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|MemoryRecallTrigger|mechanic/protocol|rust/crates/sim-core/src/causal/event.rs:179|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|CausalEvent|event/protocol|rust/crates/sim-core/src/causal/event.rs:227|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|ChronicleKind|event/protocol|rust/crates/sim-core/src/chronicle.rs:20|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|DepartCause|event/protocol|rust/crates/sim-core/src/chronicle.rs:270|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|CauseRef|event/protocol|rust/crates/sim-core/src/chronicle.rs:433|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|Command|mechanic/protocol|rust/crates/sim-core/src/colony/command.rs:11|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|RejectReason|mechanic/protocol|rust/crates/sim-core/src/colony/command.rs:381|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|ItemKind|content|rust/crates/sim-core/src/colony/items.rs:9|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|WorkType|hybrid/category|rust/crates/sim-core/src/colony/job.rs:12|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|JobKind|mechanic/protocol|rust/crates/sim-core/src/colony/job.rs:88|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|JobStage|mechanic/protocol|rust/crates/sim-core/src/colony/job.rs:317|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|CancelReason|mechanic/protocol|rust/crates/sim-core/src/colony/job.rs:404|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Lane|content|rust/crates/sim-core/src/colony/knowledge.rs:38|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|Era|content|rust/crates/sim-core/src/colony/knowledge.rs:96|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|Source|mechanic/protocol|rust/crates/sim-core/src/colony/knowledge.rs:124|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Cond|mechanic/category|rust/crates/sim-core/src/colony/knowledge.rs:198|물성·조건·gate 동작 category. 피연산자와 콘텐츠 참조는 데이터가 공급한다.|
|Channel|mechanic/category|rust/crates/sim-core/src/colony/knowledge.rs:211|물성·조건·gate 동작 category. 피연산자와 콘텐츠 참조는 데이터가 공급한다.|
|Lore|hybrid/category|rust/crates/sim-core/src/colony/knowledge.rs:220|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|Gate|mechanic/category|rust/crates/sim-core/src/colony/knowledge.rs:231|물성·조건·gate 동작 category. 피연산자와 콘텐츠 참조는 데이터가 공급한다.|
|Seeding|mechanic/protocol|rust/crates/sim-core/src/colony/knowledge.rs:248|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|CondRaw|mechanic/category|rust/crates/sim-core/src/colony/knowledge.rs:332|물성·조건·gate 동작 category. 피연산자와 콘텐츠 참조는 데이터가 공급한다.|
|GateRaw|mechanic/category|rust/crates/sim-core/src/colony/knowledge.rs:340|물성·조건·gate 동작 category. 피연산자와 콘텐츠 참조는 데이터가 공급한다.|
|SeedRaw|mechanic/protocol|rust/crates/sim-core/src/colony/knowledge.rs:350|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|TechStatus|mechanic/protocol|rust/crates/sim-core/src/colony/knowledge.rs:853|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Edge|mechanic/protocol|rust/crates/sim-core/src/colony/neighbours.rs:10|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|VisitState|mechanic/protocol|rust/crates/sim-core/src/colony/neighbours.rs:97|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|WaveAnswer|mechanic/protocol|rust/crates/sim-core/src/colony/neighbours.rs:125|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|NodeKind|content|rust/crates/sim-core/src/colony/orders.rs:17|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|Designation|hybrid/category|rust/crates/sim-core/src/colony/orders.rs:59|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|ZoneKind|hybrid/category|rust/crates/sim-core/src/colony/orders.rs:94|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|BlueprintKind|content|rust/crates/sim-core/src/colony/orders.rs:112|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|Occupant|mechanic/protocol|rust/crates/sim-core/src/colony/orders.rs:426|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|ReserveKey|mechanic/protocol|rust/crates/sim-core/src/colony/orders.rs:437|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Top|content|rust/crates/sim-core/src/colony/people.rs:13|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|DraftOrder|mechanic/protocol|rust/crates/sim-core/src/colony/people.rs:67|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Modifier|content|rust/crates/sim-core/src/colony/people.rs:103|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|Condition|mechanic/protocol|rust/crates/sim-core/src/colony/steward/rules.rs:10|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Action|mechanic/protocol|rust/crates/sim-core/src/colony/steward/rules.rs:27|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|StewardScope|mechanic/protocol|rust/crates/sim-core/src/colony/steward/state.rs:8|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Target|mechanic/protocol|rust/crates/sim-core/src/colony/steward/state.rs:43|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|PressureKind|content|rust/crates/sim-core/src/colony/story.rs:7|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|AnimalKind|content|rust/crates/sim-core/src/colony/wildlife.rs:13|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|AnimalState|mechanic/protocol|rust/crates/sim-core/src/colony/wildlife.rs:40|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Fighter|mechanic/protocol|rust/crates/sim-core/src/colony/wildlife.rs:105|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|SkillKind|hybrid/category|rust/crates/sim-core/src/colony/workshop.rs:26|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|RecipeKind|content|rust/crates/sim-core/src/colony/workshop.rs:94|이름 있는 콘텐츠 목록. 데이터 identity와 지원하는 mechanic category를 분리한다.|
|BillMode|mechanic/protocol|rust/crates/sim-core/src/colony/workshop.rs:275|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|ActivityKind|hybrid/category|rust/crates/sim-core/src/components/activity.rs:8|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|RoutineOutcome|mechanic/protocol|rust/crates/sim-core/src/components/activity.rs:71|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|TargetKind|mechanic/protocol|rust/crates/sim-core/src/components/agent_state.rs:29|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|AgentState|mechanic/protocol|rust/crates/sim-core/src/components/agent_state.rs:81|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|BuildingKind|hybrid/category|rust/crates/sim-core/src/components/building_kind.rs:6|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|FamilyEvent|event/protocol|rust/crates/sim-core/src/components/family_events.rs:18|사건·원인 payload schema이며 콘텐츠 목록이 아니다. 기존 discriminant 호환을 보존한다.|
|MemoryArm|mechanic/protocol|rust/crates/sim-core/src/components/memory.rs:51|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|PorterKind|mechanic/protocol|rust/crates/sim-core/src/components/porter.rs:18|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|PorterStage|mechanic/protocol|rust/crates/sim-core/src/components/porter.rs:43|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|ResourceKind|hybrid/category|rust/crates/sim-core/src/components/resource_kind.rs:28|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|InfluenceChannel|hybrid/category|rust/crates/sim-core/src/influence/channel.rs:26|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|AggKind|mechanic/protocol|rust/crates/sim-core/src/influence/channel.rs:127|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|UpdateTier|mechanic/protocol|rust/crates/sim-core/src/influence/channel.rs:138|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|DecayKind|mechanic/protocol|rust/crates/sim-core/src/influence/channel.rs:152|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|BlockingDerive|mechanic/protocol|rust/crates/sim-core/src/influence/channel.rs:168|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|LodTier|mechanic/protocol|rust/crates/sim-core/src/influence/propagate.rs:32|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|MaterialCategory|hybrid/category|rust/crates/sim-core/src/material/category.rs:9|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|DerivedStatKind|mechanic/category|rust/crates/sim-core/src/material/derivation.rs:71|물성·조건·gate 동작 category. 피연산자와 콘텐츠 참조는 데이터가 공급한다.|
|MaterialError|mechanic/protocol|rust/crates/sim-core/src/material/error.rs:9|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|PropertyKind|mechanic/category|rust/crates/sim-core/src/material/explanation.rs:9|물성·조건·gate 동작 category. 피연산자와 콘텐츠 참조는 데이터가 공급한다.|
|TerrainType|hybrid/category|rust/crates/sim-core/src/material/terrain.rs:10|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|RoomRole|hybrid/category|rust/crates/sim-core/src/rooms.rs:33|안정 category·저장·동작 계약. 연결된 콘텐츠를 분리하되 enum 자체를 일괄 제거하지 않는다.|
|BirthRule|mechanic/protocol|rust/crates/sim-core/src/rules/mod.rs:254|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|SettlementAnchor|mechanic/protocol|rust/crates/sim-core/src/rules/mod.rs:321|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|HousingCount|mechanic/protocol|rust/crates/sim-core/src/rules/mod.rs:329|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|ThresholdLaw|mechanic/protocol|rust/crates/sim-core/src/rules/mod.rs:609|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|CascadeArm|mechanic/protocol|rust/crates/sim-systems/src/runtime/decision/memory_bias.rs:19|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Change|mechanic/protocol|rust/crates/sim-systems/src/runtime/decision/routine/mod.rs:33|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Search|mechanic/protocol|rust/crates/sim-systems/src/runtime/jobs/assign.rs:127|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Built|content-adapter|rust/crates/sim-systems/src/runtime/jobs/space.rs:23|Five named blueprint forms mirrored in deconstruction adapter; earlier generic mechanics label missed content coupling. Migrate via authoritative completion/category/identity, not a second new catalogue.|
|Fact|mechanic/protocol|rust/crates/sim-systems/src/runtime/mood/mod.rs:62|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Expect|mechanic/protocol|rust/crates/sim-systems/src/runtime/mood/mod.rs:386|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Link|mechanic/protocol|rust/crates/sim-systems/src/runtime/mood/mod.rs:548|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|TradeRefusal|mechanic/protocol|rust/crates/sim-systems/src/runtime/neighbours/trade.rs:251|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|SaveError|mechanic/protocol|rust/crates/sim-systems/src/save/mod.rs:58|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Feed|mechanic/protocol|rust/crates/sim-test/src/perf.rs:41|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|
|Case|test-only|rust/crates/sim-test/src/probe/routine_death_tests.rs:12|실행 상태·명령·오류·동작 선택자이며 별도 근거 없이 제거하지 않는다.|


### 관련 family 사용처 수

| Family | 언급 파일 | 언급 수 | pattern/guard match | reference match | matches! | if/while-let |
|---|---:|---:|---:|---:|---:|---:|
|ActivityKind|9|32|1|1|0|0|
|AnimalKind|7|25|5|5|0|0|
|BlueprintKind|15|89|6|13|6|0|
|BuildingKind|17|51|3|4|1|0|
|Built|4|21|3|3|0|0|
|ChronicleKind|35|127|7|12|0|0|
|DerivedStatKind|5|14|3|3|0|0|
|Designation|13|77|5|11|1|0|
|Era|4|12|2|3|0|0|
|InfluenceChannel|16|115|4|4|0|0|
|ItemKind|35|268|11|19|4|0|
|JobKind|14|179|25|28|12|3|
|Lane|1|4|2|2|0|0|
|Lore|4|24|1|2|0|0|
|MaterialCategory|7|22|1|1|0|0|
|Modifier|5|71|8|8|1|0|
|NodeKind|6|19|2|3|0|0|
|PressureKind|13|93|6|12|1|0|
|PropertyKind|3|9|1|1|0|0|
|RecipeKind|15|41|3|7|0|0|
|ResourceKind|31|114|0|8|0|0|
|RoomRole|16|37|2|4|0|0|
|SkillKind|13|58|2|8|0|0|
|TerrainType|21|113|6|8|10|0|
|Top|6|17|4|6|0|0|
|WorkType|11|74|5|8|0|0|
|ZoneKind|4|6|0|0|0|0|


전체 match 위치는 [census의 family별 경로·행](/Users/rexxa/work/mux-data/stage-b-enum-use-census.md#every-discovered-match-site-grouped-by-family), 개별 arm·identifier·해시는 [JSON](/Users/rexxa/work/mux-data/stage-b-enum-use-census.json)을 따른다. Built는 `runtime/jobs/space.rs:23–45`의 콘텐츠별 다섯 형태 adapter로서 단순 mechanics 분류를 정정한다. `colony/skills.rs:149`의 KnapBlade recipe material_term도 registry 이행 경로이며 workshop-only 조사로 빠뜨리지 않는다. AnimalKind/PressureKind는 B2 문구의 시간 허용 여부와 별개로 B3 실제 새 콘텐츠 실행 시험에 반드시 필요하다. 상세 production/RNG/save/u32 경로는 [pressure/animal map](/Users/rexxa/work/mux-data/stage-b-pressure-animal-map.md)에 있다.

## 부록 C. 후속 본문 조사에서 발견한 독립 항목

다음은 재현 시험 PASS가 아니라 소스 근거다. 원래 기준이나 동결 held-out 결과를 수정할 권한은 부여하지 않는다.

| 우선순위 | 문제·근거 | 수정 방향 | 위험·보존/검증 |
|---|---|---|---|
| P2 / BR-TR01 | probe JSON 숫자의 묵시 축소: `rust/crates/sim-test/src/probe/colony.rs:192`의 u64→u32, `:754`의 count→u16, `:1331/:1348`의 level/knowledge→u8. 예: x4294967296→0, bill65536→0 | 경계에서 checked conversion과 명시 오류; 정상 콘텐츠 fallback과 잘못된 숫자를 구별 | 잘못된 스크립트의 의도와 실행이 달라짐. 허용 최대/최대+1/음수 fixture, 기존 정상 명령·9 fixture 동등성 유지. 현재 등록 스크립트가 범위를 넘었다는 주장은 아님 |
| P2 / BR-TR02 | 같은 파일 `:1287` chop tiles 및 `:1431–1432` trade의 filter_map이 잘못된 행을 지우며, `:1428`은 잘못된/없는 목록을 빈 목록으로 바꿈 | 행별 Result 수집, 필수 필드/tuple 크기 검증, 의도적인 빈 gift와 malformed 구별 | 부분 명령이나 의도치 않은 gift로 바뀔 수 있음. bad-row negative control과 valid command replay 필요 |
| P2 / BR-TR03 | `tools/v8/mux_ui_test.gd:457,464–495`는 roofed만 확인하며 “matches walled_beds_for_all”이라 기록. 실제 probe `rust/crates/sim-test/src/probe/steward.rs:128`은 roofed AND !dug AND doors nonempty | UI 설명을 실제 roofed 검사로 좁히거나 probe와 같은 독립 조건을 검사 | dug/doorless 지붕 침상도 UI 완료를 통과할 수 있음. 기존 실제 집 캡처를 무효로 단정하지 않음; dug/doorless negative control 필요. held-out 원래 기준 변경 금지 |
| P2 / BR-TR04 | `tools/v8/mart21_trees_test.gd:305–309`는 모든 종을 sats에 넣지만 n>0일 때만 낮은 채도를 실패 목록에 넣음 | 종별 유효 mask pixel 수>0를 별도 필수조건으로 검사 | summer 채도 assertion은 빈 mask도 통과 가능. 다른 가을/전체 suite가 모두 통과한다고 주장하지 않음; zero-mask/transparent summer negative control 필요 |
| P2 / BR-TR05 | `rust/crates/sim-test/src/probe/steward.rs:178` alive_at_day48은 tick>=48days일 때 최종 alive를 반환 | 정확한 day48 경계 표본을 저장하거나 도구를 정확히48일로 제한/명칭 수정 | 48일 이후 사망을 day48로 잘못 보고할 수 있음. 현재 정확히48일인 동결 보정·held-out 값에는 이 차이가 없음. 48일 경계 후 사망 fixture 필요 |
| P2 / BR-TR06 | `tools/v8/check_docs_helpers.py:109–126`가 fn 이름만 key로 setdefault하여 모듈별 동명이인을 구별 못 함. snapshot_schema.md는 literal key 합집합 검사를 명시함 | module-qualified call 해석 또는 충돌 명시 실패; nested path/type 검증은 별도 설계 | 현재 고유 명칭 수정은 안전하지만 향후 중첩 schema 누락 가능. 같은 이름/다른 keys fixture 필요. 현재 잘못된 key 추출을 재현했다는 주장은 아님 |


### 현재 문서 정정과 잔여 drift

README·ROADMAP·STAGE_A_REPORT는 gate2 기술 PASS/NOT_PUBLISHED와 gate3 필요를 현재 커밋에 반영했다. README 현재 gate 수는35이며 과거 M-ART2의34단계 기록은 역사적 사실로 유지한다. gate1 Chronicle 기대값45/실제46 실패와 inventory DESIGN.md 누락은 D35 최소 수정 후 gate2로 검증됐으므로 현재 미해결 실패로 재보고하지 않는다.

ARCHITECTURE의97 API items 설명(:225), Steward Writes `?`(:179), offset 생략 도식 등은 앞선 refresh의 잔여 drift다. 이 통합에서 API 설명과 Steward 행은 직접 재확인했다. 원본 조사 이후 전체 문서가 수정됐다고 추정하지 않으며 나머지 상세 범위는 refresh manifest를 따른다. historical report의 실패 bar와 연구/V7 참조는 현재 PASS로 지우지 않는다.

최종 판정: **수정 우선순위와 정적 근거를 통합한 B1 후보이며 구현·삭제·게시 완료가 아니다.** root의 미실행 표가 채워지고 Stage B 변경분을 재검토한 뒤 실제 결과에 맞춰 갱신한다. 기준선/V7 보호, 동적 asset 경로, 실패했던 원래 평가값은 그대로 보존한다.

## Root 실행 증거 갱신 — 2026-10-25 10:55 KST

위 후보 작성 시점의 PENDING 표는 역사적 상태다. 다음 세 검사를 실제 수행했다. Stage B 변경 소스 재검토와 전체 게이트는 여전히 PENDING이다.

- Stage A clean `ca9f531ca6ef700ca223d99dba14d834b94b45da`: gate3 35/35 exit0, 10:29:24 KST 종료; 10:30:05 KST wip/main fast-forward 게시 확인. `stage-a-publication.json` 및 `gate-stage-a-3/receipt.json`.
- 같은 소스에서 `cargo clippy --workspace --all-targets -- -W dead_code` exit0, 24.48초. 원문 `stage-b/review/clippy-dead-code.log`. 경고 없음은 미사용 데이터/GDScript 증명이 아니다.
- `rustup toolchain list`는 stable/1.93.1만, cargo 명령 목록에 udeps 없음. nightly/udeps는 NOT_RUN; 새 도구를 설치하지 않았다.
- frozen Stage A release `worldsim_perf stress --seeds 15100..15101 --agents 1000 --side 64 --days 1 --feed fed --rules baseline`: 1,440ticks(steady1,380), mean0.88171087ms, p99.9 1.667ms, max1.687ms, alive1,000. AgentDecisionSystem steady863.994297ms, 626.082824µs/call, tick time의 **71.007724%**. 단일 seed/1일 fed stress 표본이며 전체 게임/장기 프레임 성능 주장이 아니다. `stage-b/review/b1-tool-receipt.json`에 원문 해시와 설정/결과를 기록했다.

## B6 두 도메인 추출 현재 검증

ADR0039/D38에 따라 resource snapshot facade1481→1238줄과 child256줄, decision 본체3494→3487줄(새 회귀46줄 포함)과 child114줄을 분리했다. 기본 소비 세 동작만 옮기며 GatherFood/Idle/routine은 유지한다. 전체 LOC 감소나 실행속도 개선 주장은 없다. 수정 전 bridge51/decision21/통합128, 신규 야간 경계1, 수정 후 bridge51/decision22/통합128 및 strict clippy/fmt가 통과했다. 실제 렌더러 일치도 사람10명·외형10개로 통과했다. 최종 새 바이너리 identity/OFF/ON은 실행 중이다. 첫 clippy 실패와 LSP Transport closed는 ISS285/284로 구분한다([단위 검증](../mux/B6_VERIFICATION.md)).

### A source identifier correction (ISS303)
An external draft and performance runbook duplicated `b94` in the displayed A SHA. The actual A is `ca9f531ca6ef700ca223d99dba14d834b94b45da`. Both metrics manifests already used the correct identifier; a fresh437-blob OID/SHA/LOC/enum comparison found zero mismatches. External `b1-final-a-sha-correction.md/.json` preserves the typo locations and verification; prior draft hashes remain historical.
