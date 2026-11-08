# C1 구현·게시 보고

## C1-a 게시 보고

### 구현 의도 / 구현 내용 / 구현 방법 / 기능 설명
기후·지형·고도에 따라 야생종의 기원을 제한하고, 야생 수확 또는 실제 교역에서 얻은 종자로만 작물을 시작한다. 신석기 지식 39개, 아이템 29개, 조리법 23개를 추가했다. 가루·빵·맥주·치즈·훈제는 기존 물리 제작·소비·산출 장부를 사용한다. 곡식 창고의 실제 지붕·곡물 보관, 울타리 우리, 화덕집·훈제집을 연결했다. 작업 자리 때문에 화덕집·훈제집은 4×4로 작성했다. 가축이 없는 C1-a에서 5장·첫 새끼 카드는 C1-b 연결 대기다. 지식 마스크는 기존 ID·하위 인코딩을 보존하면서 u128로 확장했고 저장 형식은 20이다.

### 게시 SHA와 게이트 결과
2026-11-09 02:43 KST경 `094fea092f467a7c82764165ad0d2f9e74b806e3`를 v8/wip → v8/main 순서로 FF 게시했다. 두 원격 SHA 일치는 `C1-a-publication.json`에 있다. A3는 2026-11-08 23:20~11-09 02:36, 전체 49/49 PASS, exit0이다. PLAY_ROOT/PLAY_DATA는 각각 외부 c1-data/play-root, play-data로 지정했다. 원본 로그: fullgate-a-attempt3/gate.txt; 3058개 소스 핀·설정·러너 불변 검증: postrun-binding.json.

step41 원시 엄격 프레임 보고는 실패지만 기존 면제로 단계 PASS다. zoom1.0 최대53.994ms, >50ms 1회; zoom0.5 최대49.275ms, >50ms 0회. 기존 상한을 바꾸지 않았고 별도 재실행은 없었다. D-C1-1의 A3 단독 step41 실패 조건은 발생하지 않았다. A2 draw-only 78.6ms hitch는 2시간 이내 조사에서 C1 최초 자원 사용으로 귀속하지 못했다. texture create/free/request가 주변에서 모두0이어서 추측성 warmup 변경은 하지 않았다. 원인은 UNKNOWN이며 원본 보존. READ_CHANGED_FIXTURES는 공백 분리·정확한10개 사전 단언으로 수정했고 A3 knowledge paired fixture가 통과했다. A1의 Steward 지연은 템플릿 불변 geometry 캐시로 수정했다. A1→A3 보존된 summary/churn 19쌍은 기존 timing/git_sha 제외 규칙 아래 공통 값이 모두 존재하고 차이·양측 단독 값0이다. 이는 해당 관측치 보존 증명이며 모든 파일 원시 동일성 주장과 구분한다.

A1은47/49 PASS(step41 Steward 지연, step46 EN Why), A2는43단계 중41PASS(knowledge 러너 오류, 별개의 draw-only hitch) 뒤 당시 §5 해석으로 중단했다. D-C1-1 재개를 적용했다. A3 step46은 전체 게이트 안에서 EN64/KO64 모두 PASS다. A1의 locale/패널 실패 원인은 UNKNOWN으로 남기며 추측성 수정은 하지 않았다. 실패·중단 원본을 보존했다.

debug3205 PASS/14ignored, release3204 PASS/14ignored, Python597 PASS, Clippy PASS. baseline identity 공통228748·차이0·단독0, chronicle64파일19683항목 및 정해진5개 zero-field/git 차이를 제외한 원시64파일 PASS. 결정성·명령 재생·저장 왕복·장부·예약·C3 상한 모두 PASS. N1000 tick 평균 중앙값 BASE1742.5→후보1753.5µs, 비율1.006은 보고 전용이다. 세 번의 BASE/후보 교차 측정은 각 진입 load1<8, 단일 측정 작업이었다. 실제 녹색 게이트에 연결한 외부 PERF_LEDGER 12행 및 입력 해시는 C1-a-ledger-append-receipt.json/C1-a-PERF_LEDGER.md에 있다.

### 사슬 시험·데이터 전용 시험·origin 시험 결과, 캡처 경로
C1-a 기술·건설·조리법 실게임 UI ko/en94검사 PASS,110PNG: screens/2320_feature-run/. 대표 screens/2320_pen.png,2320_bread.png. 신석기 전체 노드 ko/en 및 재료가 있는 실제 제작 저장 장면을 사용했다. 02:14 실제 겨울 게임 화면은 screens/0214_gate39_winter_colony.png와 영수증에 있다. scene-a3/00-recipes.wsave 등 실제 제작 저장을 보존했다. origin/knowledge/recipe/debt 및 데이터 전용 시험은 전체 게이트 PASS; 새로운 crop wild-source 데이터 전용 경로, 실제 수입 종자5종 발견→파종을 검증했다. 실제19→20저장 +1440tick twin PASS. C1-b 염소 전체 사슬은 아직 구현 전이며 이 보고의 성공 주장에 포함하지 않는다.

### 크래시 감지 (BASE vs 후보 생존 배열, KnowledgeLost, Inert 항목)
시작 BASE d30dbfc5b의 중단된 최초 실행을 crash-base-r2로 다시 수행해 exit0을 확인했다. 양측 seeds0–7×48일, steward ON, 주문 없음, 원래 BASE 바이너리를 사용했다.
BASE 생존 [10,12,11,14,10,13,12,14], 후보 [10,13,11,14,10,12,12,12]; KnowledgeLost 양측 [0,0,0,0,0,0,0,0]. 정확한 장부·예약 PASS, BASE 생존 시 후보 멸종0. A3 core_shadow가 같은 배열을 재현했다. 최초 후보 seed5 멸종/KL25 뒤 허용된 딱 한 번의 Inert 전환 wild_boar를 적용했고 수치·origin은 그대로 뒀다. 이슈 https://github.com/hyunlord/new-world/issues/694 (ISS364). 추가 Inert 전환·held-out·calibration·결과를 향한 수치 튜닝은 없었다.

### 콘텐츠 수 / 부채 / 새 도구
지식82(Connected55/Inert27; 새39 중19/20), 아이템72, 조리법50, 건물 템플릿23(Connected10/Inert13), 작물7, 야생 식물11, 야생 동물6정의(Connected5/Inert1), 실제 가축0. 신규 조각15울타리·16화덕·17훈제대. CONTENT_DEBT68행. BALANCE_DEBT에 새 콘텐츠 시작 수치와 툴 효율·우유/양모 공급·범주 fallback을 기록했다.
새 도구390줄(기존 보고359 + 실제 쉼표 러너 실패 수정·사전검사31). 실제 실패·원래 줄 수 내역은 보존된 이전 보고와 실행 기록에 있다. 일반 관측/감사 프레임워크·새 의존성은 추가하지 않았다. 기능 통합 시험1269줄은 별도 집계했다.

### 확인된 제한사항·후속
C1-b에서 포획→성체 쌍→출생→젖→도축 및 실제 원인·저장·카드·렌더링을 연결한다. 젖 발견의 순환 조건은 B에서 실제 포획 Practice와 토기 지식으로 연결한다. 양모 가공은 C2, 도구 효율·바구니 운반 등은 명시된 후속 부채다. 멧돼지는 Inert 유지. A2 draw-only hitch 원인은 미확정이다. C1-a에서 새 enum·생산 unwrap·시뮬레이션 직접 UI 쓰기·수치 튜닝은 도입하지 않았다. 변경 파일 전체는 게시 커밋 diff로 확인할 수 있다.

## C1-b 구현·검증 보고 — 전체 게이트 전

**물리 사슬 run18 PASS. 네이티브 여섯 장면299검사/오류0 PASS. 전체 gate·B 크래시 감지·벤치·게시 PENDING.** seed5에서 실제 우리→포획→성체 암수→출생→젖→도축을 tick80749에 마쳤다(절대 기한86400). 중간 저장 왕복, 이후 명령을 함께 보낸 twin, 전체384명령 replay가 통과했다. 이는 사슬 시험의 성공이며 아직 C1-b 게시 성공은 아니다.

### 구현 의도 / 구현 내용 / 구현 방법 / 기능 설명

자연 야생종에 원인을 둔 사육 경로를 연결했다. tameable 데이터가 포획 지식·도구·나이·산출을 선택하고, 코드가 공통 물리 작업과 생활주기를 실행한다. 실제 완성 울타리·문, hard capacity6, 내부9칸의 풀 공급을 검사한다. 포획 성공 때 실제 끈1개를 소비하며, 젖·고기·가죽은 실제 산출 공간과 장부를 함께 반영한다. 두 성체의 출생과 먹이·나이·굶주림·죽음은 단일 LivestockSystem에서 처리한다. 출생·포획·생산·도축·죽음은 실제 Chronicle 인덱스에 묶인 불변 typed witness로 검증한다.

집사는 기존 일반 명령으로 알려진 livestock_capacity 템플릿을 요청하고 포획을 지정한다. colony mind는 기존 do_job 생명주기의 작업 소유를 설명한다. 젖 지식은 실제 Capture Practice와 pot_firing400에서 발견하므로 우유를 먼저 요구하는 순환이 없다. 저장 format21은 이전 본문을 유지하며 가축 tail을 추가하고, Skills 완료23칸을 보존한 채26칸으로 늘렸다. 실제 게시 A20 저장3개와 원래 BASE19를 검증했다. 과거 mind 등록 집합을 정확히 복원하여 기존 다섯 food fact를 유지한다.

주요 변경 파일은 `sim-core/data/colony/{animals,knowledge,mind,steward,buildings,items}.ron`, `sim-core/src/colony/{livestock,livestock_demographics,job,orders,command}.rs`, `sim-systems/src/runtime/{livestock,jobs/livestock,steward/livestock,mind}/`, `sim-systems/src/save/{legacy_livestock_v20,livestock_tests,world,mod}.rs`다. 화면 연결은 `sim-bridge/src/ffi/livestock.rs`, `scripts/world/colony/livestock_layer.gd`, 실제 chapter/card/chronicle/diagnosis와 ko/en locale이며, 사슬은 `sim-test/tests/c1_chain_goat.rs`, 데이터 전용 확장은 `sim-test/data/ext_pack/animals.ron`과 기존 확장 시험에 있다. 경로의 crate 접두사는 `rust/crates/`다. 별도 job scheduler·observer framework·의존성은 추가하지 않았다.

### 게시 SHA와 게이트 결과 (단계 수, 면제·재실행, PLAY_ROOT 외부 지정)

B 게시 SHA와 full gate 결과는 **PENDING**이다. 최종 bridge 재빌드·네이티브 검증을 거쳐 gate를 수행해야 한다. `PLAY_ROOT=/Users/rexxa/work/c1-data/play-root`, `PLAY_DATA=/Users/rexxa/work/c1-data/play-data` 외부 지정을 유지한다. B 단계 수·step41 면제·재실행 여부는 실제 gate 뒤 기록한다.

직전 게시 A는 `094fea092f467a7c82764165ad0d2f9e74b806e3`이다. A3 전체49/49 PASS, 기존 step41 면제, isolated 재실행 없이2026-11-09 02:41:59 KST에 v8/wip→v8/main FF와 원격 일치를 확인했다(`C1-a-publication.json`). A 결과를 B 결과로 재사용하지 않는다.

### 사슬 시험·데이터 전용 시험·origin 시험 결과, 캡처 경로

`b-chain-run-18.log`: release `c1_chain_goat` 1 PASS/0 FAIL,82.41초. 실제 증거는 `scene/chain-67989-1794167617256211000/`의 여섯 `.wsave`/`.json`, `mid-second-capture.*`, `complete.json`이다. `complete.json`은 seed5, tick80749, six checkpoints, first_kid3, commands384, mid-save/twin/full replay=true를 기록한다.

| 실제 단계 | tick / 표시 날짜 | 실제 사실 |
| --- | --- | --- |
| 00-pen-placed | 24001 /17일 | site5, anchor117·117; 배치 뒤 실제 건설 완료를 별도 대기 |
| 01-capture | 27904 /19일 | 자연 source17 암컷→가축1, 실제 끈1 소비, witness354 |
| 02-two-adults | 48961 /34일 | source26 수컷→가축2, 같은 우리; 나이43/30, 끈 누적2 소비 |
| 03-kid | 55681 /39일 | 실제 부모1·2에서 새끼3 출생, witness743 |
| 04-milk | 79948 /55일 | 출생 후 새 젖1 생산, dairy500 작업자, witness1062 |
| 05-slaughter | 80749 /56일 | 일반 주문의 새끼3 도축, 고기+3·염소가죽+1, witness1079; 실제 가축2 잔존 |

모든 단계의 장부·예약 invariant가 passed다. 사슬의 실제 Chronicle에는 Died가 없었다. 자연 점토36, 생토기13, 구운 토기1과 젖1의 생산 장부도 남았다. dairy는 tick78720에 실제 발견됐다. 화면 캡처가 아닌 저장·manifest만으로 그리기 성공을 주장하지 않는다. 여섯 저장을 실제 게임 창1920×1080에서 읽은 native2는 ko/en299검사 PASS/실패0/engine0/script0/exit0이며32PNG를 보존했다. 실제 Keep·만료·Slaughter와 제거된 대상의 typed Why가 통과했다. 경로는 같은 scene 디렉터리의 native-1920-run2/receipt.json이다. native1의 한글 U+2060 버튼 조회4실패와 HUD 가림은 D25의 검사/캡처 수정 후 통과했으며 원본을 보존했다. 앞선 기술/건설/조리법 네이티브 검증은 `b-progress-native-3/receipt.json`의413 PASS, engine/script 오류0이며 대표 화면은 `screens/0330_livestock_tech_connected.png`다. 최종 B native와 구별한다.

앞서 완료한 집중 검증은 core lib541 PASS, systems lib813 PASS/ignore3, bridge lib67 PASS, 가축26 PASS/ignore1, 생활주기7 PASS, genuine A20 별도 시험1 PASS(최근 `b-genuine20-tests-5.log`), genuine BASE19→21/60tick PASS다. genuine20은 실제 세 입력의 기존 카운터·상위 mask·물리 상태·등록 slot·byte-stable21 재저장·60tick twin을 검사했다. 원본은 `b-core-tests-3.log`, `b-systems-lib-tests-2.log`, `b-bridge-tests-1.log`, `b-livestock-tests-2.log`, `b-lifecycle-tests-1.log`, `b-genuine19-base-1.log`다. 겹치는 시험 수를 합산하지 않는다.

데이터 전용 확장은 `b-data-only-ext-1.log`의 상위 Rust3 PASS와 seven-family simulation/selected-scene UI PASS다. 이름을 바꾼 tameable 종과 wild-source 작물을 데이터로 노출한다. origin/knowledge/recipe 시험은 A3에서 PASS했으며 B 최종 gate 재검증은 남았다. 집중 Clippy·docs·GDScript87파일/FFI81호출은 통과했고, 당시 대상 생산 Rust unwrap audit 발견0이다. 이 기록은 이후 변경을 포함한 동결 최종 gate를 대신하지 않는다.

실패한 run1–17의 로그·산출 디렉터리·만료 저장·임시 진단은 보존한다. no-op 명령, 배치 Craft 완료 수 오인, 초기 kid 만료, 자연 사망 뒤 참조, OFF 장면의 실제 전투/탈수 붕괴, manifest tuple-key 직렬화, 실제 출력 공간·명령/작업자/필요·수면 문제를 각각 구분했다. run17 trace는 staged actor1이 Harvest/Haul을 하고 실제 Capture는0이 맡아 수면으로 취소된 사실을 입증했다. run18은 staged actor의 일반 Haul 우선순위0 지정·성공 뒤2 복원으로 경쟁 작업을 막았다. 임시 trace는 제품 트리에서 제거하고 원본은 외부에 보존했다.

seed는 첫 사전 조건 통과 seed5(고지403칸·clay·자연 암수 kid)에서 바꾸지 않았다. 초기 ID10/11 고정은 GOAL에 없던 fixture 제한이어서 D15에서 실행 전에 정정했다. 이후 실제 성체만 일반 사냥하고 기존8일 habitat/cap 유입을 기다려 현재 적격 kid를 순서대로 고정했다. 실제 포획 source는17/26이다. D16 Housing steward ON과 이동·필요 회복·작업 우선순위는 일반 플레이 명령이며 전체 replay에 포함했다. 나이·성별·RNG·성공률·사육 수치·기한·위험·아이템을 바꾸거나 주입하지 않았고, 지식 grant는 선언된 두 성인 founder의 animal_taming뿐이다.

### 크래시 감지 (BASE vs 후보 생존 배열, KnowledgeLost, Inert 항목)

B paired 크래시 감지는 **PENDING**이다. BASE는 최초 `d30dbfc5b8c6d68973e1c13fdd291741db892998`와 보존된 원래 바이너리이며 게시 A로 바꾸지 않는다. 양측 seeds0–7×48일, steward ON·주문 없음으로 측정해야 한다. B 생존·KnowledgeLost 배열은 아직 없다.

참고인 A3의 BASE 생존은 `[10,12,11,14,10,13,12,14]`, A 후보는 `[10,13,11,14,10,12,12,12]`, 양측 KnowledgeLost는 `[0,0,0,0,0,0,0,0]`이었다. 이는 B 수치가 아니다. 허용된 한 번의 wild_boar Inert 전환은 A에서 이미 사용했고 유지한다(ISS364, https://github.com/hyunlord/new-world/issues/694). 추가 전환·held-out·calibration·결과 지향 수치 튜닝은 없다. N1000 tick before/after 벤치와 B 성능 장부도 **PENDING**이다.

### 콘텐츠 수: 기술 Connected/Inert, 아이템, 조리법, 건물, 작물, 야생종, 가축

현재 기본 정의는 기술82(Connected56/Inert26), 아이템72, 조리법50, 건물 템플릿23(Connected10/Inert13), 작물7, 야생 식물11, 야생 동물6(Connected5/Inert1)이다. A 대비 herding이 연결됐다. tameable 정책은 염소·양·순록3종이며 별도 종을 추가하지 않았다. 실제 run18은 가축2→출생3→도축 뒤2마리이며, 정상 세계에 가축을 미리 생성하지 않는다.

### CONTENT_DEBT 항목 수, BALANCE_DEBT 추가 행

현재 `CONTENT_DEBT.md`는 머리/구분선을 제외한63행이다. C1-b 연결과 남은 wool/C2 부채를 구분한다. `BALANCE_DEBT.md` C1-b starting policy는11행(source, capture, pen, feeding/death, breeding, milk, slaughter, dairy, herding, steward, mind)이며 모든 수치는 설계 시작값이다. 최종 gate 전에 생성 문서와 현재 데이터를 다시 일치시켜야 한다.

### 새 도구 줄 수와 그 이유

현재 전달된 보수적 누적은 **1992줄/상한2000줄(잠정, 최종 재집계 필요)**이다. 게시 A 누적390줄, 실제 외부 runner 산출 환경1줄, 기존 spine_tech의 old-save load/capture, 기존 spine_build의 실제 가축 UI, selected Rust/native 데이터 전용 경로와 새 chain harness를 포함한다. 실제 필요한 동기는 genuine 이전 저장 검증, 실제 화면 연결, 필수 사슬·저장/twin/replay 및 발견된 실패의 재현이다. 새 chain은 도구 집계에 포함하고, 임시 진단 제거 전 최고치와 실패 원본을 보존한다. 게시 A에서 따로 공개한 기존 기능 통합 시험1269줄은 구별한다. 최종 파일별 줄 수·합계는 최종 소스에서 확정하며 추정 세부 합계를 쓰지 않는다.

### 확인된 제한사항·후속 (C2로 넘기는 것)

물리 사슬은 통과했지만 실제 여섯 화면과 ko/en 원인·5장·첫 새끼 카드의 네이티브 검증, 최종 identity/결정성/저장/장부/예약·C3를 포함한 전체 gate, B paired 크래시 감지, N1000 벤치·장부·게시가 남았다. 아직 최종 완료/게시를 주장하지 않는다. 양모 수거·방적·옷은 C2, 저장 건초 보충·도구 효율·바구니 운반·사육 산출 균형은 기록된 후속 부채다. wild_boar는 Inert 유지다. A2 draw-only hitch의 원인은 UNKNOWN이며 A1/A2와 run1–17 실패를 run18 통과로 지우지 않는다.
