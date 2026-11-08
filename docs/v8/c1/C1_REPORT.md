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

## C1-b
C1-a 게시 후 구현 시작. 미게시.
