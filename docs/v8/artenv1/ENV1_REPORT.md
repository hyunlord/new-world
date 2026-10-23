# ART-ENV1 결과 보고서

## 판정

**PARTIAL / 전체 PASS 불가.** 인벤토리와 실제 장면 출처·기준선은 확보했다. 아이템은 신규 상태 이미지 13개 중 12개가 스타일 검사를 통과했지만 32px 종류 구별은 미충족이다. 창은 3회 모두 수치 탈락했다. 동물은 사슴의 독립 가독성만 통과했으며 늑대 자세와 동물 수치 보정의 음성 대조가 실패하여 두 종 모두 `CANDIDATE_NOT_ADOPTED`다. 제작·수치·의미 판독·장면 비교·성능·오너 승인은 별도 판정이다.

건축은 신규23원천 중13개만 locally accepted이며 벽 이음새246/256이 실패했다. 초상은34룩 후보 모두 보존했으나 보정 실패로 수락0/34다. 합성48장·보드2장과 바닥93,888검사는 동결·검증됐다. **장면 가독성은 허용된 한 번의 수정 후에도 FAIL**이다. ½× 인원 수와 시간 제한은 두 라운드 모두 통과했다. 추가 아이템134답변과 실제 창 FPS 최종 두 실행을 보존했다. 최종 성능은1×340.984428377771·½×263.182634332511 FPS로 PASS다. 전달 완료와 아트 수락은 다르다. 게임 통합·merge·오너 승인은 수행하거나 주장하지 않는다.

| 항목 | 현재 증거 기반 판정 | 권위 기록 |
|---|---|---|
| 인벤토리 | 129행 추적 완료: 신규18·재사용19·부분61·보류31; 수락 수가 아님 | [최종 인계](INVENTORY_CLOSURE.md), [원천](inventory_source.json) |
| 실제 장면 / 현재 렌더러 | 3개 실제 상태 및 6개 실제 창 기준선 확보 | [SCENE_SOURCE](SCENE_SOURCE.md), [BASELINE](BASELINE.md) |
| 아이템 | 신규 스타일 12/13; 정확 종류 19/44, 구별 요구 미충족 | [ITEM_ASSETS](ITEM_ASSETS.md), [ITEMS_READABILITY](ITEMS_READABILITY.md) |
| 동물 | 구조 20/20; 사슴 가독성 PASS, 늑대 FAIL; 두 종 미채택 | [ANIMALS](ANIMALS.md), [리뷰 색인](animal_reviews.json) |
| 건축·작업장·지형 / 이음새 | 13/23 locally accepted; 32/32 연결, 246/256 seam 실패 | [자산](built_assets.json), [수치](built_numeric.json), [접속](built_autotile.json) |
| 초상 | 수락0/34; 수치·모델 음성 대조 보정 FAIL | [PORTRAITS](PORTRAITS.md), [음성 대조](portrait_negative_controls.json) |
| 합성기·오너 보드 / 장면 가독성 / FPS | 48렌더·2보드 FROZEN / R0·R1 FAIL / 두 줌 PASS | [COMPOSITOR](COMPOSITOR.md), [R1 54셀](reviews/readability/round1/comparison.md), [FPS](FPS.md) |
| 전달 대상 / 원격 동일성 기록 | `v8/art-env1`; 게시 직후 exact SHA 대조 | 최종 회신의 커밋 링크 및 외부 `publication.json` |

아트 기준은 `159ef0f9a1cfb7ee9e6005e85d5388d677e07fa8`, 시뮬레이션 조사 기준은 `3a3b236c125c0c25e9b74261be88536473fc01e9`다. 작업 브랜치는 `v8/art-env1`, 허용 변경은 `art/**`, `docs/v8/artenv1/**`다. 원본·측정·로그는 `/Users/rexxa/work/artenv1-data/`에 보존한다. 제작 한도는 일반 자산/동물 시트당 3회, 초상당 2회, 전체 도구 호출 260회이며 **전체 호출은135/260 = 건축56 + 아이템27 + 동물9 + 초상43**이다. 실패한 호출도 제외하지 않는다. [통합 원장](GENERATION_BUDGET.md)은134개 성공 출력 쌍·398개 기록 참조 해시와 각 시도 상한을 검증했다. 52회 시간 미기록·실패 요청 본문 결손·동물 후보 attachment5회 결손 때문에 provenance는 PARTIAL이다.

## 인벤토리: 신규·재사용·보류

| 분모 | 수 | 해석 |
|---|---:|---|
| 시뮬레이션 의미 행 | 129 | 지형 15, 자연 12, 건축 18, 아이템 18, 지정 9, 방 5, 동물 2, 사람 4, 의복 3, 작업 16, 일과 7, 효과 20 |
| 시작 시 REUSE | 57 | 기준 자산 경로가 존재함; 이번 패스 재수락 수가 아님 |
| 시작 시 NEW | 70 | 새 생성·코드 overlay·명시 보류를 포함 |
| 시작 시 PARTIAL | 2 | 화덕의 추가 상태, packed floor의 추가 상태 |
| 명시 보류 | 13 | NEW 70의 부분집합이며 총합에 다시 더하지 않음 |
| 기존 05 환경 PNG / 채택 사람 | 35 / 340 | 중복 없는 파일 수; 사람은 34 looks × 10 frames |

보류 13행은 해안·사막·툰드라·늪·동굴(T05–T09), 산 정상/절벽(T10), 강 내부와 전체 접속형(T13), 자연 잔가지(N07), 부싯돌 노두(N09), 영아(P03), 인간 사망·도구 파손·싸움 효과(E09/E18/E19)다. 기존 강둑·눈·바위·통나무를 이들의 전용 커버리지로 세지 않는다. 노인은 채택 성인 외형으로 표시하는 명시된 대체이며 전용 노인 외형의 완성을 주장하지 않는다.

의미 행, ItemKind, 상태 이미지, 파생 타일, 생성 호출은 다른 분모다. 예를 들어 17개 ItemKind의 빈/찬 토기는 한 종류의 두 상태이고, 통나무 3크기와 legacy berry basket 때문에 아이템 manifest는 22개 이미지 행이다. [최종 closure](INVENTORY_CLOSURE.md)는 같은129행을 **DELIVERED_NEW18 / REUSED19 / PARTIAL61 / DEFERRED31**로 추적한다. 신규18에는 실패 후보도 포함되므로 수락18이 아니다. 처음 보류13행 외에 예정했던 designation·방 역할·roof/indoor·illness/draft/mood cue 등이 미구현으로 남아 최종 보류가31행이다. 지정/방 역할의 UI 어휘는 M-ART2에서 일관되게 정하도록 보류했고, roof/indoor는 live 가시성 정책, illness/draft/mood는 상태→cue 연결과 오너 UI 선택에 의존한다. 시간 부족을 이유로 한 보류가 아니며 [D14](ENV1_DECISIONS.md)에 선택 이유를 남겼다. 17/17 ItemKind canonical 및66 bucket 참조(60 unique PNG)는 존재하나 held/carried 내용물·quality·covered·shelf 같은 상태는 부분이다. Route I340+기존환경35=375PNG는 기준 Git blob과 바이트 동일하다. [inventory_closure.json](inventory_closure.json)에 행별 결손과 M-ART2 작업을 보존했다.

## 자산 생산과 판정

| 가족 | 원본 생성 / 점수표·실패 호출 | 최종 후보와 수락 | 남은 제한 |
|---|---|---|---|
| 아이템 | 원본 23 + 실제 모델 점수표 3 + 생성 전 거절 1 = 27호출 | 신규 13상태 중 스타일 12수락·창 1탈락; 기존 9파일 동일성 확인 | 32px 구별 미충족; 추가 수량 판독105/134 |
| 동물 | 시트 4 + 실제 모델 점수표 5 = 9호출 | wolf try3·deer try1, 현재 20프레임; 정식 채택 0종 | 늑대 자세 FAIL, 수치 보정 FAIL |
| 건축·작업장·바닥·눈 | 원본47 + 실제 모델 점수표9 = 56호출 | 23원천+32파생; 수치18/23·모델22/23·교집합18/23·local 수락13/23 | 벽·굴착면 seam, 문/바닥 수치, 눈 수치·모델·의미 실패 |
| 초상 | 원본36 + 실제 모델 점수표7 = 43호출 | 34선택본 중 개별4검사 교집합24; 최종 수락0/34 | 보정 음성 대조 수치·모델 모두 FAIL |
| 전체 | 135 / 260 | 분모가 다른 가족 결과를 단순 합산하지 않음 | 일반/동물≤3회·초상≤2회·눈1회 준수; 전역 최종 파일 검사는 별도 |

### 아이템

신규 시도 수: firewood 2, ground_axe 2, spear 3, cured_hide 1, hide_wrap 3, fur_cloak 2, wolf_hide 2, clay 1, raw_pot 1, clay_pot_empty 2, clay_pot_filled 1, meat 2, flint_nodule 1. 원본 23개 중 수치 통과 12개·실패 11개를 모두 보존했다. 최종 스타일 수락은 동결 judge v2, 실제 이미지 생성 모델의 medium/line/light/palette 각 4/5 이상, 제작자 한 줄 판독의 결합이다. 상세 점수와 문장은 [items_model_reviews.json](items_model_reviews.json)에 있으며 일반 vision 리뷰로 대체하지 않았다.

창의 `wash_flatness`는 0.0347 → 0.0249 → 0.0256으로 세 번 모두 동결 상한 **0.020179014811321002**를 초과했다. 모델 5/5/4/5와 블라인드 이름 인식이 좋아도 탈락은 유지한다. 다른 초기 실패는 주로 담채 분산, 빈 토기는 saturation_q25였다. 기준 수정·네 번째 시도·픽셀 수술은 하지 않았다.

신규 파일은 `art/style05/env1/items/`의 64×64 RGBA, footprint 1×1, anchor (32,32)다. 기존 `art/style05/env/items/`의 log_pile 3종, stone_pile, flint_pile, flint_blade, stone_axe, hide, berry_basket 9파일은 prior `working_sha256`과 같다. 각 manifest의 `count_buckets`가 1 / 2–8 / 9+ 경로의 권위다. 새 변형은 전체 sprite 1/3/9개 복제이며 기존 통나무는 3크기를 재사용한다. 그림 속 낱개의 수와 구간을 같다고 해석하지 않는다. 초기 선언만 있던 spoiled meat는 `item_variants.py`로 전체 RGB 배수 `[0.55,0.70,0.45]`를 실제 적용했고, alpha 바이트 보존64×64 파생 PNG/sidecar를 생성했다. 추가 생성 호출0, `meat_spoiled.png` SHA는 `136b78e58bd657a2f3ad231d691f856973b26e7fc27fe352c4289d2e33590350`이다. 기존22이미지/44답변 분모에 새 파생 상태를 섞지 않는다. 추가 블라인드 패킷은22×3구간66+부패고기1=67항목,61고유 source이며 반복된 통나무3그룹×3개를 모두 유지했다. 실제32px·100×88셀·67표시 crop/해시를 검증했고 정답 키를 외부에 분리했다. **두 새 리뷰어의134답변은 대분류119/134·정확 종류69/134·수량105/134**다. 수량은 기존66항목×2의103/132와 부패 고기2/2를 구분한다. 의미 있는 상태38답변의 확정 판독은9/38, 불확실한 후보 해석 포함12/38이다. 부패 고기의 녹색·갈색 변색은 둘 다 봤으나 확정 부패 명명은0/2, 가능성 언급은2/2다. [추가 결과](reviews/items_states/RESULTS.md)는 원문·중복 그림·ID 오기3건의 명시적 대응을 보존한다. 기존 의미 실패를 대체하지 않는다. 실제 sim은 부패 meat를 제거한다. filled pot도 현재 ItemStack의 별도 상태가 아닌 선언 변형이다.

두 신규 리뷰어가 22개 이미지를 실제 32px로 보고 남긴 **44개 원문**을 보존했다. A는 대분류 20/22·정확 종류 10/22, B는 19/22·9/22, 합계는 **39/44·19/44**다. 대분류의 나무·가죽·도끼 인식은 서로 다른 ItemKind 구별이 아니다. 장작↔통나무, 두 도끼, hide↔cured/wolf hide, 의복↔가죽/바위, raw pot↔일반 그릇, flint↔돌 혼동이 남았다. 이 첫22항목 시험은 수량 버킷을 질문하지 않아 해당 시험의 수량은 `NOT_VERIFIED`이며, 위 추가67항목 시험에 별도 결과가 있다. 사전 없는 합격률 문턱을 만들지 않았으며 스타일 수락 12개로 의미 판독 실패를 지우지 않는다.

### 오토타일·건축·작업장

[BUILT_ASSETS](BUILT_ASSETS.md)의 최종 결과는23원천+32mask, 총55raster다. 원본47회+실제 모델 scorecard9회=56호출(기존55회+재사용3종 재검토1회), 모든 자산≤3회·눈1회다. 수치18/23, 모델22/23, 교집합18/23이며 wrap/semantic 검사를 결합한 local 수락은13/23이다. 재료별 문4상태·rack2상태·pottery/grinding·lit-pots/unlit 화덕·clay2상태·ford·바닥·언덕/굴착면·눈 파일은 모두 존재하나 후보 상태를 유지한다.

| 검사 | 최종 결과 |
|---|---|
| 두 재질16mask씩 topology | 32/32 PASS, N1/E2/S4/W8 |
| compatible edge pair | **246/256 flagged**, 재질별128쌍; 개별 pair 전부 [AUTOTILE](AUTOTILE.md)에 보존 |
| 대표 seam | log02→08 E-W RGB MAE38.128/alpha gap0; log03→10 E-W22.441/gap0; T/X 내부와 재료/그림자 변화도 남음 |
| 판정 기준 | visible premultiplied RGB MAE>18 또는 alpha-support mismatch>2px; 측정은 시각 검토 대체가 아님 |
| floor·ford·hill edge 반복 | 두 floor 반복 PASS지만 raw 스타일 FAIL; ford/hill edge 반복 PASS |
| dug face / snow | dug face 스타일 PASS·수평 seam FAIL; snow 수치·모델·반복·투명 overlay 의미 모두 FAIL |
| hut·knapping·wood bench 재사용 | 원본 수치3/3·새 모델3/3 PASS(hut5/5/5/5, 나머지4/4/4/4); 실제64px 수치1/3만 PASS, 축소 후 잉크/담채 실패2건 보존; [재검토](built_reuse_recheck.md) |

로그문2종은3회 뒤에도 wash 실패, packed floor는 stroke_q90/wash, dug floor는 wash 실패다. 네 벽 원천은 style 통과하나 seamless 수락이 아니며 dug face도 local 탈락이다. 눈은 투명 틈 대신 흙/식물이 그려져 의미도 실패했다. 원래 alpha에 전역0.6을 곱해 최대153으로 제한했으며 국소 수선은 없다. 작업 이미지의 바닥 상한 통과가 raw style·눈 의미·seam 실패를 철회하지 않는다.

packed/dug/ford/hill/dug-face export의 평균휘도 범위0.363041–0.395217, 표준편차0.053571–0.130520, 평균채도0.222185–0.289183으로 최종 기준(≤0.40/≤0.16719985/≤0.30)을 만족했다. claybank는 resource cutout이라 base-ground 분모에 넣지 않았다. bedroll의 그림1×2, pottery2×1, claybank2×2와 물리1×1 점유를 분리했다. strip/corner crop·회전·합성만 사용했고 mirror-blend·양자화·픽셀 수술은 없다. 자세한 시도별 실패·점수·footprint·원본 해시는 [built_assets.json](built_assets.json), [built_reviews.json](built_reviews.json), [built_verification.json](built_verification.json)에 있다.

### 동물 시트와 독립 리뷰

wolf/deer는 부위 조립 없는 전체 도형 시트, S/E 각 5자세다. 현재 20/20프레임 구조·원본 패널 수치 통과, 실제 모델 점수는 wolf 5/5/5/5, deer 4/4/4/5다. wolf try1은 S/dead 경계 접촉으로 탈락, try2는 round0 자세 판독 실패, try3는 마지막 재시도다. 재시도 후 PNG와 원본 해시를 동결했고 사슴 그림은 바꾸지 않았다.

최종 round1의 **각 행 분모는 10**, 기준은 종 ≥9/10와 자세 ≥8/10을 동시에 충족하는 것이다. 두 새 리뷰어 C/D는 이전 A/B와 독립이다.

| 리뷰어 | 종 | 배율 | 종 /10 | 자세 /10 | 판정 |
|---|---|---|---:|---:|---|
| C | deer | 1× | 10 | 9 | PASS |
| C | deer | 2× | 10 | 10 | PASS |
| D | deer | 1× | 10 | 9 | PASS |
| D | deer | 2× | 10 | 9 | PASS |
| C | wolf | 1× | 10 | 8 | PASS |
| C | wolf | 2× | 10 | 8 | PASS |
| D | wolf | 1× | 10 | 7 | **FAIL** |
| D | wolf | 2× | 10 | 7 | **FAIL** |

늑대는 서기를 앉기로, 정면 걷기를 서기로 읽는 문제가 남았다. 공격은 읽히지만 사람처럼 선 인상도 지적됐다. `dead (lying)`는 옆으로 누운 자세를 인정하며 죽음과 잠·휴식의 의미 구별까지 입증하지 않는다. 사슴도 일부 dead↔도약, walk↔stand 혼동을 보존한다.

round0은 80개 응답 대응에서 사슴 4집단 PASS·늑대 4집단 FAIL이었다. A의 1× `wolf-like canid`를 엄격한 닫힌 어휘로 0/10 처리했지만 qualified 종 인식은 10/10이라는 민감도 결과를 함께 남겼다. round1은 같은 문턱을 일상적 의미로 채점했다. **두 라운드 점수 차이를 동일 채점법의 개선량으로 주장하지 않는다.** 원문·80개 ID 대응·시간·해시는 [round0](animal_reviews_round0.json), [round1](animal_reviews_round1.json)에 보존된다. 시간은 모델/도구 경과 시간이며 사람 반응 시간이 아니다.

동물 보정은 wolf try2와 deer try1 S/stand 두 패널을 동결했고 try3로 다시 맞추지 않았다. leave-one-out 2/2, 현재 패널 20/20은 통과했으나 **기존 음성 대조 3개 중 woodcut 1개만 수치 거절**했다. painterly/photo 수치 통과 때문에 보정 FAIL이다. 보충 이미지 모델은 painterly 2/2/3/3, photo 2/3/3/3으로 거절하여 수치 AND 모델은 3/3을 거절하지만, 원래 수치 보정 FAIL과 동물 전용 통제가 아니라는 한계는 유지한다. 두 종 모두 정식 채택하지 않는다.

### 초상과 보정

**34룩 후보 동결, 수락0/34 (`CANDIDATE_CALIBRATION_FAIL`).** 원본36회(최초34+재시도2), 실제 모델 scorecard7회, 실패호출0으로 총43회다. `adult_F_fur_cloak_long_tied`, `adult_F_loincloth_bun`만2회이고 나머지32룩은1회다. [portrait_manifest.json](portrait_manifest.json)에 모든 시도와 선택본을 보존했다.

| 분모 | 수치 | 모델 | 구조 | 머리·상의 identity | 개별4검사 교집합 / 최종 수락 |
|---|---:|---:|---:|---:|---|
| 전체36시도 | 34/36 | 31/36 | 선택본 검사 | 선택본 검사 | 해당 없음 |
| 선택34장 | 33/34 | 30/34 | 30/34 | 33/34 | **24/34 / 0/34** |

보정의 같은 원본3crop leave-one-out은3/3이지만 독립작품 검증이 아니다. **수치와 실제 모델 모두 음성대조3개 중 woodcut1개만 거절**하고 painterly/photo를 통과시켰다. photo에도4/4/4/4를 준 오판을 보존했으며 개별24장 검사 통과를 수락으로 승격하지 않는다.

선택 수치 실패는 adult_M_fur_cloak_short의 wash다. 모델 실패는 child_F_hide_wrap_bun·child_F_loincloth_long_tied의 medium3, child_M_hide_wrap_bun·child_M_hide_wrap_short의 medium/line3이다. 얼굴은 고정 타원(중심64,61·반지름44,52) 안이며 눈에 보이는 잘림은 없으나 머리2px여유에서 adult_F_fur_cloak_long_tied try2·adult_F_hide_wrap_bun·child_M_fur_cloak_bun·child_M_loincloth_bun이 실패했다. adult_F_fur_cloak_bun의 추가 뼈 머리핀은 identity 실패다. 수동 bbox는 정밀 segmentation이 아니며 고유 얼굴 동일성은 모두 `NOT_ESTABLISHED_FROM_LOW_RES_SPRITE`다.

정본36개는 `art/portrait/env1/<look>_try<n>.png`,128×128이며 [portraits_34.png](../../../art/boards/portraits_34.png)는34선택본의128/64px를 함께 보여 준다. 보드는 무손실 재압축 후1,483,844B, 디코딩 픽셀 동일성을 확인했다. 원본·참조·prompt·scorecard 해시와 세 Python 파일 Ruff/basedpyright 오류·경고0을 검증했다. 재생성·기준 완화 없이 동결했으며 자세한 기록은 [PORTRAITS](PORTRAITS.md)에 있다.

## 실제 장면 출처와 기준선

native probe는 동결 sim SHA의 별도 worktree에서 빌드했다. `--seeds 0..1`은 반열린 구간의 **seed 0 하나**다. 명령·바이너리·script·원본 해시는 [SCENE_SOURCE](SCENE_SOURCE.md)와 각 scene provenance에 있다. 192×192 native ASCII 3/3과 사람 위치 다중집합 3/3을 읽기 전용 추출기로 대조했다. 손상 ASCII/사람 좌표의 음성 시험은 결과 파일 없이 실패했다.

| 상태 | tick | 실제 날짜·시각 | 전체 사람 / 잠든 사람 / 실제 운반 |
|---|---:|---|---|
| day6 noon | 7440 | 봄 6일 12:00 | 10 / 0 / 0 |
| day6 night | 6840 | 봄 6일 02:00 | 10 / 10 / 0 |
| first winter noon | 26160 | 절대 19일·겨울 1일 12:00 | 10 / 0 / 0 |

하루 1440tick·시작 offset 480·계절 6일이다. day30은 겨울이 아니므로 목표가 허용한 첫 겨울 정오를 썼다. 사람은 실제 AgentId·성별·생애 단계로 결합했다. native `people-json` 자체는 좌표만 주므로 ID·성별·활동을 제공한다고 꾸미지 않았다. ASCII `l/L`은 모든 ground item의 축약이어서 종류를 추측할 수 없고, 이후 동일 save의 읽기 전용 bridge 자료로 정확한 종류·수량을 보충한다. `ascii::engine_at`은 script 항목을 한 번 실행하므로 일반 soak의 반복·조건 재시도 결과와 같다고 주장하지 않는다.

현재 렌더러 기준선은 동일 save 3개 × 2줌 = **6장 실제 Godot 창**이다. Godot 4.6, Compatibility/OpenGL 4.1 Metal, Apple M4 Max, 1600×900이며 HUD 상단112px를 원본 그대로 사용했다. 캠프 `(91,101)`은 화면 `(800,506)`, 타일은 64/32px다. 전체 인구10과 맵 중심점 기준 노출8/10/8, 이후 픽셀 가림을 적용한 가시 인구는 서로 다른 값이므로 섞지 않는다.

세 save에는 완성 Storehouse 방·늑대·운반자가 모두 0이다. 실제 저장 과제 대상은 zone2의 36칸 적치장과 미완성 벽 둘레다. 전체 지도에는 사슴9마리가 있고 ½× 밤에 한 마리만 들어온다. 낮의 일부 내부 routine은 제작·보수 등이지만 표시 프레임은 서기다. 정답은 숨은 routine을 외형에서 읽으라고 요구하지 않는다. M5 기존 contact sheet는 다른 tick·줌·사건이므로 직접 동일 조건 점수 근거로 바꾸지 않는다.

## 합성기와 선언 오버레이

**Round1 최종48렌더·2보드 동결·검증 완료 (2026-10-23T07:45:54.312898+00:00).** 3실제장면×4조건×(1×figure/½×LOD/½×figure/¼×LOD)=48장이다. 이는 후보를 수락하거나 가독성/성능을 인증하는 판정이 아니다. 권위는 [COMPOSITOR](COMPOSITOR.md), [render_manifest.json](render_manifest.json), [검증](renders/verification.json)이다.

선언 overlay는 7×5 통나무집·문·바닥·침구, 5×4 돌 저장고·혼합 아이템, 굴착 포켓·굴착면·언덕 가장자리, 도기 작업대·갈돌·가죽 건조대·소성 화덕·점토 둑·여울, 늑대2·사슴3, 실제 사람7명의 외형/위치/활동 대체(가죽옷4·운반2·벌목1)다. 원래 인물 ID와 전체 인구는 유지하며 실제 좌표와 선언 좌표를 별도 보존한다. 이 배치가 그 tick의 실제 건물·동물·행동이었다고 주장하지 않는다. 모든 출력에 선언 범례가 있어야 한다.

| 규칙 | 계약 | 최종 증거 |
|---|---|---|
| 사람 | Route I 원본; upright 80×112 anchor(40,104), sleep 112×80 anchor(56,72); 접촉 그림자 별도 | 검증 완료 |
| ½× / ¼× LOD | 실제 상의색 원 10/6px, 1px 잉크 테두리; ½×만 axe/log/zz | 검증 완료 |
| 동물 LOD | ½× 이하 늑대 빨강·사슴 황갈색 삼각형 | 검증 완료 |
| 바닥 등록 상한 | 평균 휘도≤0.40, 표준편차≤person anchor의60%, 평균 채도≤0.30 | 93,888/93,888 통과 |
| 사전 설정 목표 | 평균 휘도≤0.32, 표준편차≤person의25%, 채도≤0.22, 반올림 여유10% | 동일93,888/93,888 통과 |
| 밤 | 전역 값 −45%, 바닥만 청색 이동, 화덕5타일 안 값 복원; 사람·물체 hue 보존 | 최종 측정 검증 완료 |
| 겨울 / 가뭄 | 바닥 탈채도+한 번 생성한 눈 alpha≤60%; 가뭄은 바닥 황토색 방향 | 최종 검수 검증 완료 |
| site/blueprint | 완성 sprite의 옅은 잉크 외곽+25% wash, 코드 처리 | 검증 완료 |
| 출력 | 1600×900, 조건/장면에 맞는 실제112px HUD, 1×·½×LOD·½×figure 비교; ¼× 진단 | 48장, 최대1,246,505B |

휘도는 encoded sRGB의 Rec.709 가중치이며 linear-light 측정 주장이 아니다. 최종 바닥93,888개 crop은 계절·눈·상한·화덕빛 이후 foreground/HUD 이전에 측정했고 등록 기준과 더 엄격한 초기 목표를 모두 통과했다. 관측 최대는 평균휘도0.2943612·휘도표준편차0.0413080·평균채도0.2132244, person 기준표준편차0.1706134다. 평균만으로 개별 실패 타일을 가리지 않는다. 원래 조건과 다른 진단 렌더의 HUD 시각·계절은 그대로 남으므로 matched 낮/밤/겨울 쌍만 비교에 사용한다.

오너 보드 경로는 `art/boards/env1_owner_board.png`, `art/boards/env1_winter_overlay_board.png`다. 첫 보드의 현재 화면·05 1×·05 ½× LOD·05 밤 및 두 번째 겨울/overlay를 원본 픽셀로 직접 확인했다. 보드는3200×520, 각각1,061,593B·948,370B이며 panel source hash가 맞다. 48개 image/raw-evidence 해시와48개 실제 HUD crop 픽셀이 일치했다. 선택6장 정답은 `renders/selected_readability_truth.json`의 FROZEN 값이며 외부 사본과 동일하고 리뷰 패킷에서 제외한다. 낮/겨울은8명, 밤10명이 실제 픽셀로 보이고 시험 줌마다 선언 운반2명이다. 14 pytest·Ruff0·13모듈 basedpyright 오류/경고0을 확인했다. 겨울의 반복 질감·밤 수면glyph 밀집은 관찰 한계로 유지한다. 보드는 오너 검토 자료이며 승인 증거가 아니다.

## 장면 가독성 비교

**Round0·Round1 모두 원시 비열등성 FAIL.** [고정 규칙](READABILITY_PROTOCOL.md)에 따라 조건마다 서로 독립된 새 리뷰어2명, 3조건 × 2줌 × 2렌더러 × 2리뷰어 = 라운드당 **24개 이미지 평가**다. 각 이미지의 6과제와 A/B/C 활동 각각, 원문·opaque ID·정답 대응·60초 제한의 관측 시간·늦은 답을 보존한다. 이미지 로딩/기록을 포함한 모델·도구 시간이며 사람 반응 시간과 같지 않다.

아래 각 칸은 **두 리뷰어 합계 before → after**로 채운다. 활동은 총합뿐 아니라 A/B/C 각 점수를 함께 적는다. 모든 비교 셀에서 after≥before여야 하며, ½× LOD 사람 수는 각 리뷰어가 ±10% 이내여야 한다. 전체 평균으로 개별 실패를 상쇄하지 않는다.

| 라운드 | 조건·줌 | 사람 수 | 저장 위치 | 운반 수 | 늑대 존재·위치 | 낮/밤 | 활동 A/B/C | 시간·판정 |
|---|---|---|---|---|---|---|---|---|
| 0 | 낮 1× | 2→2 | 2→2 | 2→2 | 2→2 | 2→2 | 2→1/2→2/2→2 (합계 6→5) | 시간 PASS / FAIL |
| 0 | 낮 ½× LOD | 2→2 | 2→2 | 2→1 | 2→2 | 2→2 | 1→0/1→2/2→2 (합계 4→4) | 시간 PASS / FAIL |
| 0 | 밤 1× | 2→2 | 1→2 | 0→2 | 2→2 | 2→2 | 2→2/0→2/0→0 (합계 2→4) | 시간 PASS / 원시 셀 PASS |
| 0 | 밤 ½× LOD | 2→2 | 2→2 | 0→2 | 2→2 | 2→2 | 2→0/0→2/0→2 (합계 2→4) | 시간 PASS / FAIL |
| 0 | 겨울 1× | 2→2 | 1→2 | 0→2 | 2→2 | 2→2 | 2→2/1→2/1→1 (합계 4→5) | 시간 PASS / 원시 셀 PASS |
| 0 | 겨울 ½× LOD | 2→2 | 2→2 | 1→1 | 2→2 | 2→2 | 2→1/2→2/1→2 (합계 5→5) | 시간 PASS / FAIL |
| 1 | 낮 1× | 2→2 | 1→2 | 2→2 | 2→2 | 2→2 | 2→2/2→2/2→0 (합계 6→4) | 시간 PASS / FAIL |
| 1 | 낮 ½× LOD | 2→2 | 1→2 | 2→2 | 2→2 | 2→2 | 2→2/2→2/2→2 (합계 6→6) | 시간 PASS / 원시 셀 PASS |
| 1 | 밤 1× | 2→1 | 1→2 | 0→2 | 2→2 | 2→2 | 2→2/0→2/0→0 (합계 2→4) | 시간 PASS / FAIL |
| 1 | 밤 ½× LOD | 2→2 | 2→2 | 0→2 | 2→2 | 2→2 | 2→2/0→2/0→2 (합계 2→6) | 시간 PASS / 원시 셀 PASS |
| 1 | 겨울 1× | 2→2 | 2→2 | 1→2 | 2→2 | 2→2 | 2→2/2→2/2→0 (합계 6→4) | 시간 PASS / FAIL |
| 1 | 겨울 ½× LOD | 2→2 | 2→2 | 1→2 | 2→2 | 2→2 | 2→2/2→2/2→2 (합계 6→6) | 시간 PASS / 원시 셀 PASS |

Round0은36과제 셀 중2개(낮1× 활동6→5, 낮½× 운반2→1), 별도18활동 셀 중4개(A: 낮1×2→1·낮½×1→0·밤½×2→0·겨울½×2→1)가 감소했다. ½× 인원 수6/6과24/24시간(14–33초)은 통과했다. [원문·54셀](reviews/readability/round0/comparison.md)을 보존했다. 작은 도끼가 짐·일반 도구로 읽힌 원인에 대해 허용된 한 번의 수정으로14×14 도끼/14×8 통나무 glyph와1× 기존 stump 문맥을 적용했다. 사람 원본·위치·정답·판정 문턱은 바꾸지 않았다. [수정 전후 기록](round1repair/DECISION.md)에 R0 보존과 R1 선언을 남겼다.

Round1은36과제 셀 중3개(낮1× 활동6→4·밤1× 사람2→1·겨울1× 활동6→4), 별도18활동 셀 중2개(낮/겨울1× C2→0)가 감소했다. A의 도끼/벌목은 after12/12로 읽혔지만, C의 정지 외형을 걷기로 읽는 혼동과 밤 인원 누락이 남았다. 밤1× B는10명 중7명으로 답했다. ½× 인원 수6/6, 시간24/24(12–41초)는 통과했다. [R1 원문·54셀·판정 근거](reviews/readability/round1/comparison.md)를 보존했고 두 번째 수정은 하지 않는다.

비교 가능성의 제한은 결과 전 고정했다. before는 미완성 적치장, 늑대0, 운반0이며 after는 완성 저장고·늑대2·선언 운반/작업을 포함한다. 저장·운반·늑대·활동은 통제된 스타일 비교로 `NOT_COMPARABLE`, 사람은 선언 위치 차이가 있고 시각 분류는 비교 가능하다. 이 셀들도 원시 점수 분모에서 빼지 않는다. **원시 표시 장면의 수치 기준을 모두 만족하더라도 순수 아트 교체의 통제된 비열등성은 `NOT_VERIFIED`**다. 미관측 정답은 추측 PASS로 채우지 않는다.

## Crowd FPS

**PASS: 최종 순차 실행의 실제 창 완료 렌더 프레임 측정.** Route I 사람200 + 환경 sprite400 + ground, 각60초, 1×와 ½×LOD 모두 ≥60FPS가 기준이다. `_process` 빈도·과거 패스 FPS·headless 결과는 대체 증거가 아니다.

| 배율 | 사람 / 환경 / ground | 관측 구간 | 완료 렌더 FPS | process FPS | 화면·설정·자산 SHA | 판정 |
|---|---|---|---|---|---|---|
| 1× | 200 / 400 / 있음 | 60.002740s / 20,460프레임 | 340.984428377771 | NOT_MEASURED | [전체 설정·SHA](fps/results.json), [화면](fps/1x_final.png) | PASS |
| ½× LOD | 200 / 400 / 있음 | 60.003959s / 15,792프레임 | 263.182634332511 | NOT_MEASURED | [전체 설정·SHA](fps/results.json), [화면](fps/half_final.png) | PASS |

Godot4.6/Compatibility/OpenGL4.1 Metal/Apple M4 Max,1600×900, nearest, vsync off, cap0, warm-up2초다. 두 실행 모두 invalid0·입력 해시 유지·Engine 완료 카운터 대조·200/400 전체 viewport 경계 검사 PASS다. [모든 시도](FPS.md)에는 초기 점수표 파일 사전 로딩 실패, 이전 표식 결과, 창 warm-up 경계가 겹친 불확실 실행을 남겼다. 최종 쌍은 이전 프로세스 종료 후 다음을 실행했다. OS on-screen 창은 확인했으나 창 공유 제한으로 OS screenshot은 UNAVAILABLE이며 실제 창의 Godot viewport를 캡처했다. 물리 scanout·완전한 가림 없음·실제 게임 시뮬 성능을 인증하지 않는다.

## 변경 파일과 검증

| 경로 묶음 | 내용 |
|---|---|
| `art/style05/env1/built/`, `docs/v8/artenv1/built_*.json` | 건축·작업장·바닥·눈 및32mask;13/23 local 수락·실패 후보 보존 |
| `art/style05/env1/items/`, `art/pipeline_env1/items*.py` | 아이템·count 버킷·보드·점수표, 보존/판정/타입 파싱 도구 |
| `art/animal/`, `art/pipeline_env1/animal*.py` | 두 종 전체 프레임·A1·양 라운드·실패 후보·보정 |
| `art/portrait/env1/`, `art/boards/portraits_34.png`, `portrait*.json` | 36시도·34선택·수락0과 보정 실패 보존 |
| `art/compositor/` | 읽기 전용 추출·실제 장면·기준선 캡처·선언 overlay·톤/LOD 합성 |
| `art/proto/`, `art/boards/` | 부하 하네스(두 줌60초 FPS PASS)·동결 오너 보드2장 |
| `docs/v8/artenv1/` | 목표·인벤토리·판정·원문 리뷰·장면/기준선·렌더·최종 보고서 |

아이템 도구는 최종 보조 모듈 포함4파일의 `uv basedpyright` 오류0·경고0, Ruff 및 diff 검사 통과를 확인했다. 22자산·23생성 기록·13모델 점수·44리뷰의 타입 파싱, 잘못된 필드 거부, JSON 무손실 왕복을 검사했다. 이 코드 수정 전후 동결 자산/문서98파일 해시는 같았다. 이는 아이템 판정 변경이나 전체 pass 검증을 뜻하지 않는다. **최종 기술 검증 PASS:** Python35파일 basedpyright 오류0·경고0, Ruff0,44테스트 PASS; Godot parse·실제 두 실행 종료0이다. 610개 사전 요약 대상 파일의 경로·크기, PNG335개 디코딩·JSON195개 파싱을 확인했고 최대1,483,844B다. 생산원본375개와 추가 역사PNG2개를 base와 바이트 대조했다. 생성135회의 성공 출력134쌍·기록 참조398개·원천manifest8개 해시도 재검증했다. [최종 검증](FINAL_VERIFICATION.json)에 범위와 초기 검사 오류의 수정 내역을 남겼다. 건축 도구는 기존 typed JSON 경계를 재사용해 Any 접근을 제거했고, 테스트5곳의 의도적 반환값 폐기만 명시했다. 동결 자산201파일·검사 의미를 유지했다. 게시 직전 staged byte/scope 검사와 exact remote SHA 확인은 최종 회신·외부 게시 receipt에 귀속된다.

게시 대상은 `origin/v8/art-env1`이다. 자기 자신을 포함한 커밋의 SHA는 본문에 순환 기재하지 않는다. 최종 회신에 `git rev-parse HEAD`와 `git ls-remote origin refs/heads/v8/art-env1`의 동일 SHA와 고정 링크를 보고하고, 원시 확인은 `/Users/rexxa/work/artenv1-data/publication.json`에 보존한다. 게시 성공은 자산·가독성·오너 수락을 의미하지 않는다.

## M-ART2에 넘길 것

| 대상 | 경로·계약 | 설치 전 제한 |
|---|---|---|
| 채택 사람 | `art/character/route_i/MANIFEST.json`, `docs/v8/artprod3/HANDOVER.md` | 34룩340프레임 원본 유지, 기존 두 adult fallback 계약 유지 |
| 기존 환경 | `art/style05/env/**` | 35개 기준 자산; 이번 신규 의미 행 전체 커버리지로 세지 않음 |
| 신규 건축·작업장 | `art/style05/env1/built/`, `built_assets.json` | 23원천+32mask; local 수락13과 seam/style/semantic 실패를 구분 |
| 아이템 | `items_manifest.json`, `art/style05/env1/items/`, 기존 `env/items/` | 17kind 매핑·명시 count 경로·상태 변환 전달; 창 탈락, 32px 혼동 유지 |
| 동물 | `art/animal/{wolf,deer}/{S,E}/*.png`, 종별 MANIFEST·TEMPLATE_A1 | 두 종 후보 전용; upright96×80 anchor(48,72), dead96×64 anchor(48,56); W는 E 전체반전 |
| 초상 | `art/portrait/env1/`, `portrait_manifest.json`, `art/boards/portraits_34.png` | 수락0/34; 수치·모델 보정 FAIL 및 개별 구조/identity 상태 보존 |
| 장면·표시 규칙 | `art/compositor/scenes/`, `COMPOSITOR.md`, 원본/overlay/기준선 manifest | offline 증거와 live bridge 구독·애니메이션을 구분 |

M-ART2는 ASCII에 없는 **벽·문·바닥·침구·굴착, 실제 작업장 종류/활동·bill, resource node의 종류/잔량, 정확한 아이템 종류/수량/품질, 방 역할·적치장·지정, 동물 종류/상태, 실제 의복·운반/작업·routine**을 live bridge 레이어에서 읽어야 한다. 이번 동일-save bridge 보충은 일부 실제 객체를 복구한 offline 증거이며 전체 live 연결을 완료한 것이 아니다. 현재 sim에 없는 문 open/material, filled pot, persistent carcass, spoiled meat 등의 표현은 별도 데이터 계약 또는 선언 상태로 남긴다.

½× 사람10px 원·동물 삼각형, ¼×6px, 상의색·활동 glyph·anchor·접촉 그림자·y정렬·바닥 상한·밤45%/화덕5타일·겨울눈60% 규칙을 함께 인계한다. 표시 후보를 렌더에 썼다는 이유로 미수락 자산을 정식 설치하지 않는다. 임의 fallback으로 최종 보류31행·부분61행이나 의미 판독 결손을 완료로 바꾸지 않는다.

## 제한사항과 오너 결정 필요

확인된 제한은 최종31행 보류·61행 부분, 창 수치 탈락, 아이템 종류/수량 판독 결손, 늑대 자세 실패, 동물·초상 보정 음성 대조 실패, 건축246/256 seam·문/바닥 수치·눈 의미 실패, 같은 원본 crop 보정의 비독립성, 선언 overlay로 인한 비교 내용 차이, 모델 시간의 해석 제한이다. 합성 수치와 FPS 통과는 장면 가독성 R1 FAIL 및 이 제한 목록을 덮어쓰지 않는다. 원본·실패 시도·이전 리뷰를 보존하고 예산 소진을 수락으로 바꾸지 않는다.

오너 질문 없이 실행 선택은 [ENV1_DECISIONS](ENV1_DECISIONS.md)에 기록했다. 후속 채택을 위해 필요한 결정은 다음과 같으며 **현재 승인 상태는 모두 PENDING**이다.

| 결정 | 검토 자료 | 남겨야 할 판단 |
|---|---|---|
| 05 적용 강도 | 현재/05 1×/½×LOD/밤, 겨울/overlay 보드 | 세부 묘사를 물체·사람에 집중하고 바닥을 낮춘 정도가 시뮬 읽기에 적합한가 |
| LOD 문턱 | ½× figure와10px marker 비교, 정량 가독성 결과 | 32px부터 marker로 전환할지, glyph·크기·종별 표식을 어떻게 유지할지 |
| 밤 밝기 | −45% 값과5타일 화덕 복원 장면·실패 과제 | 분위기와 대상 구별 사이의 실제 표시값 채택 여부 |
| 미수락/결손 처리 | 창·아이템 혼동·늑대·보정 실패·최종 보류31행·부분61행 | M-ART2 이전 별도 수정 패스 또는 명시 제한을 선택할지; 자동 채택 없음 |

본 보고서와 게시 보드는 오너의 눈으로 판단할 자료이며 **오너 승인을 주장하지 않는다**.
