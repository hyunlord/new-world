# ART-TRIAL4 보고서

## 판정

**불채택. 생성·코드 기반 전신 인물 실험을 종료한다.** S1과 P1은 통과했으나 R1의 의상·머리·성별·연령 및 2× 활동 판독이 실패했다. TEMPLATE v2와 route_p는 외주 전달용 후보이며 게임 캐릭터 표준이나 M-ART2 승인 자산이 아니다. A/B 원본은 이동하거나 폐기하지 않았다.

또한 실행 과정에서 **S2 총 spec 수정 2회 제한을 위반했다.** 최초 동결 이후만 세는 초기 해석이 잘못됐다. 보존된 서로 다른 spec/ink 해시 네 개로 최소 3회 변경이 확인되며, 덮어쓴 초기 시도 때문에 정확한 총수는 알 수 없다. 이 절차 위반을 기술 검사 통과로 상쇄하지 않는다. 추가 spec 수정과 두 번째 판독 라운드는 실행하지 않았다.

| 지표 | 결과 | 근거/분모 |
|---|---|---|
| S1 | PASS | 레이어 636/636, 조합 48/48 |
| S2 | FAIL | 최소 3회 변경, 허용 총 2회 |
| R1 | FAIL | 두 리뷰어 모두 속성·인구학 및 2× 활동 기준 미달 |
| T1 | 보고 완료 | numeric 0/6 통과, 모델 평균 2.33/5; 채택 게이트 아님 |
| P1 | PASS | 200명·1×·60.003초·280.719 FPS |
| K1 | 보고 완료 | 636레이어, 재생성 22.652초, RGBA PNG 합계 1,741,698 bytes |
| D1 | PASS | 두 생성의 1,910파일 바이트 동일; 측정시간 JSON 제외 |

브랜치 `v8/art-trial4`, 시작점 `7c767aae5b4c5ef2a1b8eb575bffa251dea0263f`. 원본 목표는 `7429877d`, Part 0 기록은 `3475894e`에 보존했다. 원시 증거 루트는 `/Users/rexxa/work/arttrial4-data/`이며 아래 상대 증거 경로는 이 루트를 기준으로 한다.

## 구현 의도

손·다리·관절 대신 머리, 캡슐 몸통, 머리카락, 의상, 도구를 독립 레이어로 구성했다. 데이터 spec을 Pillow로 4× 래스터화하고 Lanczos로 1× 축소한다. 저장된 PNG를 검사기와 브라우저/Godot lab이 같은 합성 함수로 읽는다. 성별·연령·색·장비의 시뮬레이션 연결은 `art/character/PAWN_ATTRIBUTES.md`에 매핑만 기록했으며 게임 코드는 변경하지 않았다.

## 템플릿 v2 변경

64×96, 바닥 y88, lab 접촉 그림자 24×8, S/E/N과 머리가 왼쪽인 96×64 수면 레이아웃을 사용했다. BODY는 발 쐐기를 포함하고 손/다리 검사 대신 아이템 BODY 접촉 2px를 검사한다. v1 검사 경로와 기존 45개 테스트는 유지했다.

원문 머리 하단 y52와 몸통 상단 y54는 겹치지 않으므로 목 연결부를 명시했다. 시계 방향 회전과 머리 왼쪽 요구도 양립하지 않아 머리 왼쪽을 우선했다. 의상은 실제 BODY alpha의 상부 70%를 덮는다. 여성·노인·아동의 몸통과 의상은 동일한 등록 변환을 공유하며, 아동 머리카락도 머리 크기에 맞춰 변환한다. 자세별 bob/lean과 도구 회전은 저장 레이어의 공통 합성 단계에서 수행한다. 세부 결정은 `TRIAL4_DECISIONS.md` D2·D4·D8~D13에 남겼다.

안티앨리어싱 halo는 생성 단계의 공통 BODY support-envelope로 제한한다. 손으로 출력 픽셀을 고치지 않았고 alpha 기준 8을 높이지 않았다. `none` 레이어는 명시적으로 비어 있어야 한다.

## 생성기와 잉크 패스(파라미터 표)

| 항목 | 값 |
|---|---|
| 렌더러 | Python 3.14, Pillow 12.2.0, numpy 2.4.3; cairo/skia 미사용 |
| 작성/출력 | 4× → Lanczos 1× |
| 외곽선 | 4×에서 8px, #141210 |
| 내부선 | 4×에서 4px; 머리 최대 4줄, 의상 최대 3줄 |
| 명암 | base, 우하단 면적 35% −18%, 좌상단 15% +10% |
| 종이결 | spec/ink 해시 시드, 곱셈 잡음 ±8% |
| 색 | 기존 palette_05 값, 새 의존성/팔레트 추가 없음 |
| 이동 | walk bob ±1px, item sway ±4°, work lean 2px 및 −30°/+40° |
| carry/sleep | 어깨 y56 통나무, 머리 왼쪽 회전 |

최종 spec+ink SHA256: `d9133b411b0cd935ac253d3a2bddd1a2dadfe2befc8a186069c1f60780dc4b50`.
BODY 504개(성인 168 + 아동 168 + 노인 168), HAIR 72, TOP 24, ITEM 36으로 총 636개다. 성인 필수 세트는 300개다. PNG당 104~3,715 bytes이며 합계 1,741,698 bytes는 RGBA 레이어만 센 값이다. 마스크·sidecar·보드 용량은 포함하지 않았다. PNG 평균은 약 2,738.5 bytes다. 전체 재생성 실측 22.652258초는 한 번의 관측이며 성능 분포가 아니다.

`candidate-final/`과 `candidate-final-repeat/`의 RGBA 636 + mask 636 + sidecar 636 + inventory/manifest 2 = 1,910파일이 바이트 동일하다. `GENERATION.json`은 측정 시간 때문에 제외했다(`D1.json`).

S2의 보존된 해시 순서: `9aa69a05…` → `3f5d3681…` → `db35e71a…` → `d9133b41…`. `implementation-attempts.json`과 각 디렉터리의 `GENERATION.json`으로 확인된다. 초기 실패 후보마다 judge를 실행하지 않은 점과 일부 초기 출력 덮어쓰기도 절차 누락이다. 최종 동결 후 수정 0회라는 사실로 총 횟수를 재정의하지 않는다.

## 구조 검사 결과

최종 전체 636/636 레이어와 48/48 조합 통과. 독립 감사에서도 같은 결과이며 검사기 조합 48개의 픽셀은 실제 consumer `compose_layers`와 동일하고 도구 clipping이 없다. 48조합은 모든 속성·방향·상태 범주를 포함하지만 전체 조합 공간의 전수 검사는 아니다.

증거: `candidate-final-checks/CHECKS.json`, `part0/independent-final-checks/`, `installed-checks/CHECKS.json`. 저장소 설치본도 다시 검사했다. 기존 45개를 포함한 pipeline3 67개와 pipeline4 3개, 총 70개 테스트 통과. Python 컴파일과 Ruff E9/F 검사 통과. 전체 게임 gate/identity는 실행하지 않았다.

Part 0 문서 검사: UID 보정 전 종료 코드 **1**, 보정 후 **0** (`part0/docs-before.log`, `docs-after.log`). CharacterLab/wash_tint UID를 추적하고 camp/character_lab 씬 헤더 UID를 보완했다. 신규 PawnLab도 UID를 포함한다.

## 리뷰어 결과(라운드별 원문 요약)

**라운드 0만 실행.** 새 리뷰어 A/B에게 정답 키나 의미 있는 파일명을 주지 않고 번호·배율만 제공했다. 각 리뷰어는 72개 실제 재생 contact 이미지와 24개 응답을 기록했다. unknown은 오답이다. 원문 전체는 `reviewer-round0-a.json`, `reviewer-round0-b.json`, 채점은 `readability-round0.json`, 독립 재산정은 `final-score-audit.md`에 있다.

| 리뷰어 | 배율 | 활동 /12 | 도구 /12 | 의상 /12 | 머리 /12 | 세 속성 /36 | 성별 /12 | 연령 /12 | 인구학 /24 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| A | 1× | 8 | 12 | 2 | 3 | 17 | 0 | 0 | 0 |
| A | 2× | 8 | 12 | 2 | 3 | 17 | 0 | 0 | 0 |
| B | 1× | 8 | 12 | 6 | 5 | 23 | 0 | 8 | 8 |
| B | 2× | 8 | 12 | 6 | 5 | 23 | 0 | 8 | 8 |

기준은 활동 1× 8/12, 2× 10/12, 세 속성 30/36, 인구학 20/24이다. 두 리뷰어 모두 도구는 읽었지만 걷기를 운반 또는 정지로, bun/long_tied를 short로, 직물/털 의상을 hide_wrap 또는 unknown으로 읽었다. 성별은 전부 unknown이었다. 의상과 머리의 형태 차이, 체형·연령 단서, bob의 활동 전달력이 부족하다.

A의 01 원문: “Still upright figure, short dark cap of hair, olive enclosed garment; garment type and demographic cues not distinct.” B의 12 원문: “Bound logs held across chest; olive garment has straight vertical neckline; receding hair and lined face suggest elder.” 원문 응답을 사후 정답으로 고치거나 리뷰어를 재사용하지 않았다.

24 GIF는 각 1초/8프레임이며 S/E × meadow/forest 패널을 포함한다. Chrome 155.0.8059.85 실제 GIF 재생을 목표 8Hz로 3초씩 캡처해 총 576장을 기록했고 JS 오류는 0이었다. 실제 시각은 `round0/lab/browser/playback.json`에 있다. 정지 상태의 GIF가 정적으로 보이는 것은 예상 동작이다. 동영상 프레임을 별도로 seek하거나 합성 이미지로 대체하지 않았다.

기준 미달 후 수정·새 리뷰어 라운드를 해야 했으나, 총 수정 한도를 이미 초과했으므로 **라운드 1은 미실행**이다. 이는 통과나 면제 처리되지 않는다.

## 스타일 점수

person family judge v2는 12개 기준 이미지 leave-one-out 보정 12/12 후 실행했다. 숫자 검사는 6개 모두 실패했다. 숫자 결과의 model_status=MISSING은 별도 모델 평가 전 CLI 산출물이며, 모델 점수는 `style/model-round0.json`에 독립 보존했다. 양쪽 결과를 합쳐 통과로 표시하지 않는다. 이 지표는 보고용이다.

| still | edge density | saturation q25/q75 | wash flatness | medium/line/light/palette | 작성자의 직접 관찰 |
|---|---:|---:|---:|---|---|
| 01 | .06056 | .28732/.56314 | .00601 | 2/3/2/2 | 짙은 짧은 머리와 올리브 몸통은 분명하지만 의상 종류 단서가 약하다. |
| 02 | .06466 | .27085/.46815 | .05367 | 2/3/2/3 | 옆모습의 갈색 의상과 바구니는 구분되지만 긴 머리 형태가 뭉친다. |
| 03 | .06744 | .25425/.54972 | .02628 | 2/3/2/2 | 민머리와 도끼는 읽히지만 의상은 평평한 올리브 덩어리로 보인다. |
| 04 | .05935 | .28531/.54448 | .00894 | 2/3/1/3 | 통나무 묶음이 강하게 읽히며 머리 묶음과 의상 디테일은 작다. |
| 05 | .06988 | .32618/.56240 | .00476 | 2/3/2/2 | 왼쪽 머리의 수평 수면 자세는 읽히나 작은 체형만으로 연령은 확정하기 어렵다. |
| 06 | .07492 | .30035/.48456 | .02579 | 2/3/2/3 | 좁은 체형·바구니·갈색 옷은 분명하지만 땋은 머리와 성별 단서는 약하다. |

stroke q25/q50/q75/q90은 여섯 후보 모두 0. 모델 평균은 medium 2.00, line 3.00, light 1.83, palette 2.50, 전체 2.33/5다. 모델은 어두운 윤곽의 일부 유사성은 인정했지만 기준 인물의 따뜻한 회화적 질감과 조명에 비해 평면 색면이라고 평가했다. 여섯 still과 두 person 기준 이미지를 직접 비교했다.

보드: `art/character/boards/pawn_sheet_1x.png`, `pawn_sheet_2x.png`(72룩), `pawn_states_1x.png`, `pawn_states_2x.png`(21상태), `compare_ABP_2x.png`(기존 A/B 동일 6사례와 P). 모두 REJECTED_READABILITY를 표시했다. 배경은 기존 calm 자산 `art/style05/ground_v2/{meadow,forest_floor}.png`이며 상세한 구형 env ground 대신 사용한 경로 차이는 D7에 기록했다.

## 군중 FPS

실제 표시된 Godot 4.6 창, Apple M4 Max, Compatibility/OpenGL 4.1 Metal 91.7, 1600×1080에서 200명 1×를 60.003113초 렌더링했다. 16,844 rendered frames / 실제 경과 시간 = **280.718769 FPS**. 61회 표본 모두 drawable/focused, 숨김 처리 프레임 0, bounds 내 배치, 측정 중 소스 해시 불변이었다. `round0/lab/crowd/runtime.json`과 캡처 두 장을 보존했다.

초기 두 번은 85초 timeout으로 실패했다. macOS에서 가려진 창의 frame_post_draw가 멈출 수 있어 기존 CharacterLab처럼 전면/always-on-top과 별도 monotonic 종료를 사용했고, 그 후 성공한 별도 실행을 측정했다. 실패 로그를 제거하지 않았다. 이 수치는 standalone lab 렌더 처리량이며 디스플레이 주사율이나 게임 시뮬레이션 포함 FPS를 뜻하지 않는다.

## 변경된 파일 목록

- `art/character/TEMPLATE.md`, `LAYER_CONTRACT.md`, `PAWN_ATTRIBUTES.md`: 후보 규격·계약·속성 매핑.
- `art/pipeline4/{pawn_spec.py,pawn_gen.py,ink.py,ink_params.json}` 및 테스트/README: 결정적 레이어 생성.
- `art/pipeline3/{check_layer.py,check_set.py,layer_schema.py,pawn_checks.py,pawn_sets.py,test_pawn_contract.py}`: v1 유지, v2 검사와 회귀 테스트.
- `art/character/route_p/**`: 636 레이어·마스크·sidecar, manifest/inventory/checks, 불채택 안내.
- `art/pawn_lab/**`, `art/proto/PawnLab.gd`, UID, `scenes/pawn_lab.tscn`: 실제 저장 레이어 소비, GIF/브라우저/군중 측정.
- `art/character/boards/pawn_*.png`, `compare_ABP_2x.png`: 최종 검토 보드 5개.
- `art/proto/CharacterLab.gd.uid`, `wash_tint.gdshader.uid`, `scenes/{camp,character_lab}.tscn`: Part 0 UID 정리.
- `docs/v8/arttrial4/**`: 원본 목표, addendum, 계획·결정·최종 보고.

## 제한사항

S2 위반과 라운드 1 미실행으로 계약 전체를 충족했다고 주장할 수 없다. 초기 후보 일부는 덮어써졌고 judge도 누락됐다. 최종 구조·결정성·실시간 증거는 이 한계를 제거하지 않는다. 모델 두 명의 판독은 사용자나 실제 플레이어 승인과 다르다. 48조합 결과는 전수 조합 보증이 아니다. 모든 원시 캡처·로그·응답은 저장소 밖에 있어 다른 머신에는 별도 전달해야 한다. CHECKS의 절대 경로는 다른 checkout에서 재생성해야 한다.

기존 `.gitignore` 변경, 미추적 `.ignore`, `docs/art/`, `.omo/`는 보존했다. 공통 CHANGELOG/ROADMAP/HISTORY/skills는 허용 범위 밖이라 변경하지 않고 이 보고서와 결정 기록으로 pass를 남겼다. 이미지 생성 서비스·3D 도구·로그인·결제를 사용하지 않았으며 game bridge와 M-ART2는 검증하거나 변경하지 않았다.

## 오너 결정 필요

**외주 인물 패키지 권고를 유지한다.** 후속 발주 범위·비용·작가 선정은 별도 결정이며 이번 실행에서 승인하거나 발주하지 않았다. 전달 패키지는 TEMPLATE v2, 레이어 계약, 속성 표, pawn_spec/ink_params, 실패 판독 표와 A/B/P 비교 보드다. 작가의 첫 표본에서 의상·헤어·연령 구분과 1×/2× 활동 판독을 먼저 확인해야 한다. 이 보고서를 이유로 자동 실험을 재개하거나 M-ART2에 설치하지 않는다.
