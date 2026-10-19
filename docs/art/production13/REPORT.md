## 구현 완료 보고

**WIP-only / PARTIAL. 전체 승인 미완료이며 green 배포가 아니다.** Part0 release 기준선과 declump 후보 승인은 실측했다. 이후 사용자의 GUI 금지 지시가 우선하여 새 창·숨김 창·최소화 창을 열지 않았고, CLI-Anything headless로 가능한 소스 이미지·텍스처·글리프 검증만 수행했다. 최종 after 성능·화면 판독·실제 마우스 입력은 미검증이다. 최종 게이트는118/120이며 GUI 차단2건 때문에 전체 미통과다. 요구사항 전체를 완료했다고 주장하지 않는다.

### 구현 의도

game_v26에서 실제 프레임 비용을 측정하고 유효한 이익이 있는 커널만 기본 활성화한다. 활동 그림과 이름 글자를 구별할 수 있도록 개선하되, CPU·소스 이미지 결과를 실제 화면 승인으로 바꾸지 않는다.

### 구현 내용

Pass21을 병합하여 init/reset을 game_v26으로 맞췄으며 관계·운반자 packed 배열과 chronicle0–21 계약을 유지했다. CPU 보정15회와 전후 probe 입장 규칙으로 release 기준선, 두 running 세계 구간 타이머 및 W200 node ablation을 확보했다. Declump는 두 조건 모두 수락하여 기본ON, 정확한 GDScript off-switch를 보존했다. 보수적 반복 쓰기 생략은 구현·강제쓰기 단위검사까지 완료했으나 후보 화면/속도 검증이 중단되어 기본OFF다.

일곱 활동에 어두운 둥근 배지·두 색 그림을 적용하고 마을 요약에서 같은 아트를2×로 쓴다. 동일 사람을0.4초 가리키면 이름·활동 툴팁을 표시하는 경로를 구현했다. 소스 그림 블라인드는7/7이지만 세계 crop Stage B는 NOT_RUN이다. 글리프24/36px 실패를 보존하고 실제 Life regular/bold48px에서10,442쌍 기계 검사를 통과했다. 사실과 이름을 분리하는 판독 기준은 유지했으나 신규 화면 판독은 실행하지 않았다. OWNER_FEEDBACK.md는 시작 시 없었다.

### 구현 방법

읽기 전용 Pass21 chronicles에서 최저 적격 seed19603을 선택하고 출발일10:00 tick90840을 재현했다. 별도 base/measure worktree에서 identity와 release를 분리하고 release dylib을 measure의 debug 경로에 복사한 SHA를 확인했다. 후보마다 여섯 화면·running600×3·pan240×2 및 교대150-frame3쌍을 사용했다. 모든 제외·실패·중단 근거를 보존했다.

GUI 금지 이후 설치된 cli-anything-godot의 headless 실행만 허용했다. wrapper exit0 외에 엔진 반환값·빈 stderr·완료 sentinel을 확인했다. Godot headless dummy driver의 시간을 화면 성능으로 사용하지 않았다. 글리프는 실제 TextServer CPU atlas, 동일 origin/baseline, alpha≥0.5 XOR로 계산했다. 그림 생성 도구 사용0/4회다.

### 기능 설명

커널ON으로 declump 결과를 유지하면서 연산 비용을 줄인다. P12_DECLUMP_KERNEL=0으로 기존 경로를 유지할 수 있고 P13_WRITE_SKIP은 기본OFF다. 배지는 #211C25 alpha0.85, 여백2 source pixels이며 Craft는 원래 U1_04 형태를 보존한다. 툴팁은 mouse-out·활동 없음·reset·포커스 상실·언어 변경 등에 숨겨진다. Life48px은12px 설계 격자의4배이며 좁은332px 영역의 줄바꿈 증가와 실제 배치는 아직 확인하지 못했다.

### 변경된 파일 목록

현재 확인한 first-parent 커밋 전체를 아래에 기록한다. 링크마다 해당 커밋의 전체 변경 파일과 diff를 제공한다. 기존 로컬 .gitignore/.ignore 수정은 별도 보존하며 본 작업에 staging하지 않는다. 핵심 제품 변경은 agent_renderer의 커널 기본값/opt-in 쓰기 생략, activity_icons/agent_activity/village summary, activity tooltip와 main 연결, 다섯 localization 파일, agent inspector Life 크기다. 나머지는 p13 계측·검사 도구와 증거다.

- Part 0: [bc62745ee57e2fc24e8c140fdb845dd04c974005](https://github.com/hyunlord/new-world/commit/bc62745ee57e2fc24e8c140fdb845dd04c974005) — art-p13(p0): bring in simulation pass 21; 변경 157개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 0: [c9c0c35577dc8f0871c0bd47b8f95f0eebbb330b](https://github.com/hyunlord/new-world/commit/c9c0c35577dc8f0871c0bd47b8f95f0eebbb330b) — art-p13(p0): preserve the execution contract and measurement deadlines; 변경 3개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 0: [c1b1358e5d443f2eff11733b0ca8136df32a7d00](https://github.com/hyunlord/new-world/commit/c1b1358e5d443f2eff11733b0ca8136df32a7d00) — art-p13(p0): make interference measurable before judging frame costs; 변경 69개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 0: [6d493fa8ade6d368eca8e7efa7ab3f668eea1827](https://github.com/hyunlord/new-world/commit/6d493fa8ade6d368eca8e7efa7ab3f668eea1827) — art-p13(p0): prevent premature admission exhaustion and incomplete diagnosis; 변경 10개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 0: [91508a77b1b8f628ad417ab4fa0638064c09772a](https://github.com/hyunlord/new-world/commit/91508a77b1b8f628ad417ab4fa0638064c09772a) — art-p13(p0): preserve exact references and refused admission evidence; 변경 155개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 0: [716e102cb52d500d00352c85e096c2da23b2bb34](https://github.com/hyunlord/new-world/commit/716e102cb52d500d00352c85e096c2da23b2bb34) — art-p13(p0): prevent reference controls from qualifying as candidate evidence; 변경 10개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 0: [93a344c75574139f69c7d130f147de4b4d7494d6](https://github.com/hyunlord/new-world/commit/93a344c75574139f69c7d130f147de4b4d7494d6) — art-p13(p0): establish an admitted release baseline before optimization; 변경 36개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 1: [6e01bb2997bb00b4441277ebd92d79e2ad2ce76a](https://github.com/hyunlord/new-world/commit/6e01bb2997bb00b4441277ebd92d79e2ad2ce76a) — art-p13(p1): enable the kernel after both measured acceptance bars pass; 변경 17개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 1: [f8659c848fda8a3e85e7903674844ba14e3406ee](https://github.com/hyunlord/new-world/commit/f8659c848fda8a3e85e7903674844ba14e3406ee) — art-p13(p1): isolate unchanged property writes for measured acceptance; 변경 29개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 1: [215bae332d647f38323a29ef00411d24dad3266f](https://github.com/hyunlord/new-world/commit/215bae332d647f38323a29ef00411d24dad3266f) — art-p13(p1): preserve interrupted evidence and enforce windowless continuation; 변경 14개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 2: [e099b9fcecbdc5e6412af8446c77268124d538fc](https://github.com/hyunlord/new-world/commit/e099b9fcecbdc5e6412af8446c77268124d538fc) — art-p13(p2): distinguish activities while respecting windowless validation; 변경 49개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 3: [9050fa0b348dcc9b76eb50a5de138e9c3eaf0d42](https://github.com/hyunlord/new-world/commit/9050fa0b348dcc9b76eb50a5de138e9c3eaf0d42) — art-p13(p3): make Life glyphs meet the measured distinction bar; 변경 7개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.
- Part 5: [d3c18a1cffd1c57c85238f7a97bf8bfeb96ba7f1](https://github.com/hyunlord/new-world/commit/d3c18a1cffd1c57c85238f7a97bf8bfeb96ba7f1) — art-p13(p5): keep standalone glyph validation reproducible; 변경 3개. 커밋 링크의 Files changed가 해당 커밋의 전체 파일 원장이다.

- Part 5: [1e0ff6b1bde5097510a23f525c3d420aae5675cb](https://github.com/hyunlord/new-world/commit/1e0ff6b1bde5097510a23f525c3d420aae5675cb) — 스크롤 갱신 순서 수리, 정적 장면 자원 검사 보강, 실패 게이트 보존; 변경132개.
- Part 5: [8be1a9dbdf7587ecc9622671be317f0133f8fb0b](https://github.com/hyunlord/new-world/commit/8be1a9dbdf7587ecc9622671be317f0133f8fb0b) — 모든 수리 시도·절차 초과·리뷰2·수정 동결 기록; 변경14개.

- Part 5: [25ddf8fc43e7c1c68397ea88dab38c9c1873a03c](https://github.com/hyunlord/new-world/commit/25ddf8fc43e7c1c68397ea88dab38c9c1873a03c) — art-p13(p5): keep candidate dispatch statically auditable; 변경 2개.
- Part 4: [f34b4a0bfa8b3929e7173cdab78566fca64fe000](https://github.com/hyunlord/new-world/commit/f34b4a0bfa8b3929e7173cdab78566fca64fe000) — art-p13(p4): preserve the frozen build and unavailable display measurements; 변경 27개.
- Part 5: [a7efe9f47dca3c805f5d11623351cdf03eea7345](https://github.com/hyunlord/new-world/commit/a7efe9f47dca3c805f5d11623351cdf03eea7345) — art-p13(p5): retain tooltip checks at the real input boundary; 변경 2개.
- Part 5: [0fac7632648053816696941c031027be79ae047a](https://github.com/hyunlord/new-world/commit/0fac7632648053816696941c031027be79ae047a) — art-p13(p5): publish the partial verdict without promoting unavailable checks; 변경 133개.

[커밋별 전체 파일 원장](commit-files.json). 병합 원장은 첫 부모(기존 art WIP)와의 실제157파일 차이로 정정했다. 이전 원장에 섞였던 두 번째 부모 기준의 기존 art 파일 목록은 이번 병합 변경이 아니며, 해당 과거 원본은 f34b4a0b 커밋에 보존된다. 후속 보고서 커밋은 최종 원격 확인에 추가한다.

### 측정 결과

아래 기준선과 후보 교대 실험은 서로 다른 측정이다. 최종 동결 빌드 after는 전부 NOT_MEASURED이며 전후 배수도 계산하지 않는다.

| H | 기준선 / 현재 증거 | 최종 판정 범위 |
|---|---|---|
| H1 | W0 frozen median2.8235ms≤6.0 | 기준선 bar 충족; 최종 after NOT_MEASURED |
| H2 | W1 running median12.5550,p9913.9210ms≤16.67 | 기준선 bar 충족; 최종 after NOT_MEASURED |
| H3 | W200 running median26.1450ms>16.67 | 기준선 FAIL; 최종 after NOT_MEASURED |
| H4 | W200 frozen median16.2575ms>10.0 | 기준선 FAIL; 최종 after NOT_MEASURED |
| H5 | W500 frozen median28.4035ms>16.67 | 기준선 FAIL; 최종 after NOT_MEASURED |
| H6 | W-multi running16.3505ms;p9918.2170. W500 running3블록 인구감소로 제외 | 보고 전용, pass/fail 없음; after NOT_MEASURED |
| H7 | 수락한 커널6화면 RGBA차이0, running/pan2,280프레임 byte-identical | 커널 후보 PASS; 최종 UI 동일성 주장은 아님 |
| H8 | 소스 시트7/7; 실제 Image21쌍 XOR/색상 bar 통과; Stage B NOT_RUN | NOT_VERIFIED |
| H9 | actual48px10,442쌍 PASS; 사실/이름 신규 판독 및 실제 입력 미실행 | 전체 NOT_VERIFIED |
| H10 | init/reset v26·계약 확인; porter/관계색/화해 문장의 최종 화면 증거 미확인 | NOT_VERIFIED |

Release 기준선: process+render CPU, setup 제외, 조건별3×300 counted frames, 각 블록 fresh reset. p90/p99는 nearest rank. running은2×이며 아래 ticks/frame은 중앙값이다.

| 세계 | 조건 | median ms | p90 ms | p99 ms | max ms | ticks/frame | 인구 |
|---|---|---:|---:|---:|---:|---:|---|
| W0 | frozen | 2.8235 | 3.2300 | 3.8020 | 4.5000 | 0 | 29→29 |
| W0 | running | 5.3035 | 5.7640 | 6.0620 | 6.2550 | 1 | 29→29 |
| W500 | frozen | 28.4035 | 29.7040 | 30.9490 | 31.5260 | 0 | 500→500 |
| W500 | running | 제외 | 제외 | 제외 | 제외 | — | 500→392/392/404 |
| W1 | frozen | 6.2970 | 7.1710 | 10.9150 | 13.0250 | 0 | 71→71 |
| W1 | running | 12.5550 | 13.2380 | 13.9210 | 14.4940 | 1 | 71→71 |
| W200 | frozen | 16.2575 | 17.0700 | 17.7780 | 18.4360 | 0 | 200→200 |
| W200 | running | 26.1450 | 27.8510 | 30.0760 | 62.1090 | 2 | 200→200 |
| W-multi | frozen | 7.3015 | 8.0800 | 14.4980 | 15.5280 | 0 | 88→88 |
| W-multi | running | 16.3505 | 17.4210 | 18.2170 | 19.0050 | 1 | 88→87 |

세계 seed/tick/시작 인구/마을: W0=20261010/40560/29/1; W1=20261013/126994/71/3; W200=15100/1440/200/1; W500=15100/1440/500/1; W-multi=19603/90840/88/3. Live는v26, stress는v19다. W500 running 블록 중앙값80.6540/79.6050/80.3400ms는 원본에 남기지만 인구가392/392/404로 줄어 유효500명 결과에서 제외한다. [전체 기준선](perf/baseline.md), [원시 JSON](perf/baseline.json), [세계](perf/worlds.md).

CPU 기준 R=**2.0089961669873446s**, 1.05R=2.109445975336712s.15회 원본·각 ps/load는 [calibration.json](perf/calibration.json), 표는 [calibration.md](perf/calibration.md). 모든 측정 전후 probe와 거절 시도는 [holds.md](perf/holds.md), [baseline-admission.md](perf/baseline-admission.md), [attempts.json](perf/attempts.json)에 연결한다. 실패 첫 시도도 삭제하지 않았으며 각 유효 baseline은 attempt2다.

| 후보 교대 측정 | control median | candidate median | candidate/control | 각 쌍 비율 | 판정 |
|---|---:|---:|---:|---|---|
| 커널 W500 frozen |27.8490|22.1760|0.796294|0.782092,0.796257,0.798697|ACCEPTED |
| 커널 W200 running |25.7140|24.7935|0.964202|0.958226,0.965759,0.972939|ACCEPTED |
| 쓰기 생략 |NOT_MEASURED|NOT_MEASURED|—|완료한 쌍 없음|NOT_VERIFIED, 기본OFF |

커널은 두 조건 모두 pooled≤0.97,3/3쌍 faster다. W200 세 번째0.972939는 개별0.97 bar가 아니라 faster bar를 충족한다. 각 mode450프레임. 수락 커널 재캡처도6화면·2,280프레임 일치한다. [후보 원본/유효성/검증](perf/declump-kernel/verification.md), [판정](perf/declump-kernel/verdict.json), [쓰기 생략 중단 근거](perf/write-skip/verdict.json).

실행 진단의1ms 이상 exclusive 비용(서로 다른 세계 측정): W200 walker_offsets9.8755, W-multi seats5.6970, W200 seats4.4075, W-multi walker_offsets1.3535, W200 pose1.1100ms; 전부 AgentRenderer 안이다. 전체 AgentRenderer는 W20022.327,W-multi11.509ms. Frozen W500 declump6.423ms. W200 ablation은 AgentRenderer21.8505ms, WorldSim9.7855ms 차이이나 후자는 시뮬레이션을 끄므로 제거 가능한 화면비용으로 해석하지 않는다. 외부 presentation 최고 ActivityTrailRenderer0.8615/BuildingRenderer0.8605ms로 후보(c) 조건1ms에 미달했다. 개별 getter1ms 이상 없음; aggregate nested median W2000.108/W-multi0.180ms. 구간·getter·ablation은 겹치는 별도 실험이므로 더하지 않는다. 진단 after NOT_MEASURED. [실행 진단](perf/running-diagnosis.md), [모든 타이머/getter](perf/section-timers.md), [27노드 원장](perf/baseline-ablation-W200.md).

아이콘 소스 크기20×20(Craft22×22), headless Z4 transform80×80(Craft88×88), summary40×40/44×44다.21쌍 최소 XOR/union0.613636≥0.25, 최소 hue49.606318°≥40. 실제 framebuffer 대비는 NOT_MEASURED. 이전 P1에서 명시한 grass/dirt/roof patch와 옛 main-fill 대비는 [baseline-headless.md](activity/baseline-headless.md)의 대표 샘플이며 새 배지 화면 대비로 재사용하지 않는다. [전체 색·쌍 지표](activity/source-icons/metrics.json), [아이콘 시트](activity/icons.md).

Stage A 소스시트 최초7/7, 반복 없음. 각 종류1/1; Craft low confidence·소거법 의존과 Repair/Craft 혼동 가능성 유지. 실제 표시 크기의 framebuffer 블라인드와 동일하다고 주장하지 않는다. Stage B(범례)·범례 없음·마을6쌍 모두 NOT_RUN, eligible 실제 pair 수 NOT_MEASURED. [동결 답변](activity/blind-stage-a.md).

글리프 실제 regular/bold48px, 라틴92표기·한글46표기, 양 weight 합10,442쌍 전부≥12px. 한글 regular 최소16px(마/미 등), bold32px(아/이 등); 라틴 regular48px(nu/hu 등), bold96px(no/ho 등).24px 한글 regular11쌍/bold9쌍 실패,36px regular9쌍 실패는 보존한다. [최근접10쌍 및 실패 이력](readability/glyph-distinctness.md), [actual48 원본](readability/glyph-actual48.json). 사실 판독·이름 OCR count/confusion은 NOT_RUN이며 글리프 PASS로 대체하지 않는다. 툴팁 headless boundary_failures=0, 실제 입력 NOT_VERIFIED.

### 환경

Mac, Godot4.6-stable official89cea143, Rust jobs6, dev/test opt-level2 및 별도 release 계측. Release bridge SHA256 `b4d29cf526676adc3fa77f21454d2bee79c7865a08b32f398c975e7d8bf6a9f6`. 증거 `/Users/rexxa/work/art-production13-evidence`; Pass21 자료는 읽기 전용 p21-data. DGX 사용 없음. GPU 시간이 unavailable인0 필드는 GPU비용0이라는 뜻이 아니다. GUI 금지 이후 headless dummy 실행만 했고 새 release display 측정은 하지 않았다.

### 독립 리뷰

독립 fresh reviewer1회차는9050fa0b에서 새로운 blocking correctness/evidence-overclaim 문제를 발견하지 않았다. standalone gate의 glyph 출력 경로 누락은 nonblocking 지적으로, tooling-only d3c18a1c에서 해결했다. [1회차 범위](review/round1.md). 마지막2회차는 커널 증거159파일 해시, 원시450샘플/모드/세계와6비율, 보정15회, 병합 부모·packed 계약 보존, 최신 스크롤 수리와 정적 장면 검사를 독립 확인했다. 새 차단 결함은 없으나 최종 gate는 리뷰 당시 진행 전이며 보고서 갱신과 시도 초과 공개가 필요하다고 명시했다. [2회차 범위와 한계](review/round2.md). 두 리뷰 모두 최종 gate 완료나 화면 승인을 대신하지 않는다.

### 자율 결정 목록

[DECISIONS.md](DECISIONS.md)에 상세 기록한다. 커널은 실측 수락, 쓰기 생략은 opt-in OFF 유지; 외부1ms presentation 후보 없음; W-multi ablation은 첫 droppable로 생략. 새 GUI 금지는 과거 화면 요구보다 우선하며 headless 결과 범위를 제한한다. 폰트 이름11이 아닌 파일 격자12px를 기준으로24→36→48을 시험했다. 근거 없는 사실 판독과 최종 성능값을 만들지 않았다.

### 실행 기록

시작2026-10-19T01:46:00Z(최초 읽기 보수적 기준). 아래 경계는 커밋 타임스탬프이며 동시 작업은 중복 합산하지 않는다.

| Part | 구간 UTC | 경과 |
|---|---|---|
| 0 |01:46:00→03:19:40|1시간33분40초 |
| 1 |03:19:40→03:48:55|29분15초 |
| 2 |03:48:55→03:58:57|10분02초 |
| 3 |03:58:57→04:02:25|3분28초 |
| 4 |04:02:25→04:11:11 증거 기록;04:22:06 수정 동결|8분46초;수정은Part5에 포함 |
| 5 |04:02:25→04:38:21 최종 gate/PR 확인|35분56초;Part4와 일부 겹침 |

명시적 admission 대기 union1243.6406910419464초(20분43.641초)는 위 경과 안에 포함된다. 개별 job 대기나 겹친 프로세스 시간을 추가 합산하지 않는다. GUI 중단 이후 검증 축소는 조기 성능 성공이 아니라 사용자 지시 준수다.

보고서 작성 직전 git ls-remote 확인: WIP8be1a9dbdf7587ecc9622671be317f0133f8fb0b, publish branch57ae7ac40abde7d24369606c9250d3e06e0f18d6. 제품 소스는 수정 동결1e0ff6b1bde5097510a23f525c3d420aae5675cb와 diff0이며9050fa0b는 역사 기록이다. 이 보고서·최종 로그를 묶는 문서 커밋의 hash와 push 후 양 branch 확인은 최종 응답의 게시 영수증에 덧붙인다. [PR511](https://github.com/hyunlord/new-world/pull/511)에 ArtPass13 WIP-only/PARTIAL 절을 추가하고 기존 본문38,476자를 그대로 보존했음을 재조회로 확인했다. PR은OPEN/draft, head art/base revitalize 그대로다. 소유한 art-p13-base/art-p13-measure 두 worktree는 제거 후 git worktree list에서 부재를 확인했고 별도 audit는 건드리지 않았다. 전체 gate가 미통과이므로 publish branch를 이동하지 않았다.

### 확인된 제한사항

절차 위반: 스크롤 수리에서 실패 대상당3회 제한을 초과했다. 최초 gate 실패 이후 P13은8회(검사실패6,파싱중단1,통과1),P12는2회(파싱중단1,통과1) 실행했다. Debug 표기 실행도 실제 검사이며 제외하지 않았다. 파싱 오류 때 exit0이었지만 완료 sentinel이 없어 PASS로 세지 않았다. 책임은 root의 시도 제한 관리 누락이며, 최종 통과가 이를 소급 승인하지 않는다. [모든 시도와 원본 로그](readability/layout-repair/ATTEMPTS.md).

최종 after matrix/두 running timers, Stage B 세계 crop 판독, 실제 마우스13단계+툴팁, Life48px 실제 화면 레이아웃/스크롤(headless 수리·재검증은 통과), 운반자·복수 관계색·kind21의 최종 화면 확인이 미완료다. 새 아이콘·툴팁·48px 글자가 최종 프레임에 미친 영향도 미실측이다. 기준선 running W500은 인구 손실로500명 대표성이 없고 W-multi88→87 변화를 유지했다. 일부 과거 section metadata의 frozen 설명은 실제 running 수치와 불일치하여 원본에 경고를 남겼다. 이전 Pass12 판정을 다시 쓰지 않았다.

### 게이트 결과

Part0 full gate116/116 통과 기록. 최종 attempt1은 opt-level2 환경 누락을 확인하여 의도적으로 중단(exit130)했다. 최종 attempt2는 terminal exit1이며 [results.tsv](gates/final-attempt2/results.tsv)120행 중115행 exit0, 나머지5행은 다음과 같다.

| 검사 | exit | 확인 내용 |
|---|---:|---|
| cargo_test |101|툴팁 추가 후 static scene resource 수 기대값이 오래됨 |
| test_life_layout_p12 |1|실제48px Life 레이아웃/스크롤 실패 |
| test_life_layout_p13 |1|같은 실제48px 레이아웃 실패 |
| test_real_input_p12 |75|GUI 금지로 실행 불가, 기능 PASS 아님 |
| test_real_input_p13 |75|GUI 금지로 실행 불가, 기능 PASS 아님 |

실패 로그는 [final-attempt2](gates/final-attempt2/results.tsv)에 보존한다. Static scene 검사4개는 ext_resource22→23, load_steps23→24로 실제 추가 자원을 반영하고 정확한 tooltip script/node binding assertion을 추가하여 수리했다. 전체 optimized cargo session97862는 exit0,2,368개 통과·실패0·ignored0으로 완료됐다. [전체 원본](gates/resource-repair/cargo_test.log). Inspector는 viewport582→568 변경 전에 옛 scrollbar page로 복원하여14px 잘리던 순서 문제를 수정했다. Resize와 sort 완료 뒤 위치를 복원하며48px과 기존 테스트를 유지한다. 변경 없는 P12/P13 headless 레이아웃 재검사는 둘 다 exit0/failures=0, 오류 없음이다. [P12](readability/layout-repair/p12.log), [P13](readability/layout-repair/p13.log). 실제 framebuffer 확인을 대신하지 않는다.

실제 입력 harness는 a7efe9f4에서 기존13단계를 유지하고 tooltip4단계를 추가했다. 현재 parse만 확인했으며17단계 실제 입력은 NOT_VERIFIED다. 수정 후 runtime 동결은1e0ff6b1, 독립 리뷰2회차는 완료했다. 최종 gate3 session24830은 exit1로 종료했다:120개 중118개 exit0, 실제 입력P12/P13만 GUI 실행 차단으로 exit75. [최종 결과](gates/final/results.tsv). Rust2,368개/Python480개, Clippy, strict GDScript, bridge,900-frame headless smoke 및 두 스크롤 검사가 통과했다. 전체 PASS/green을 보고하지 않으며 기술 게이트가 통과해도 미실측 H와 화면 판독은 별도다.

### 정직한 판단

검증된 release 기준선에서71명 W1은2× p9913.921ms로16.67ms bar 안이었다.88명 W-multi는median16.3505지만p9918.217로 같은 부드러움을 보장하지 않는다.200명은median26.145로 bar를 넘고500명 running은 인구감소로 제외된다. 이 숫자는 최종 배지·48px UI의 성능 보증이 아니다.

커널은 W500 frozen약20.37%, W200 running약3.58% 실측 감소했지만 쓰기 생략은 빨라졌다고 말할 근거가 없다. 활동은 소스 그림만7/7 인식했으며 범례가 있는 실제 세계와 없는 실제 세계에서는 아직 답할 수 없다. 이름 글자는 actual48px CPU glyph에서 기계적으로 구별되고 headless 스크롤 검사도 통과했지만 실제 화면 배치와 사람 판독은 남았다. 다음 아트 과제는 새 배지/48px Life의 실제 화면·입력 검증과 최종 release 재측정이며, 실행 권한이 없는 GUI를 우회하지 않는다.
