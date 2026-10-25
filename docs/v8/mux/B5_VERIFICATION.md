# B5 나무 종 — 구현 및 검증 기록

**집중 Rust·기존 회귀·headless renderer·통제된 실제 창54검사 PASS / 원본/OFF/ON 안전망 확인, 전체 B gate 대기.** 제품 커밋 `3c39dcbe0039c98479c6294c09b599ed009650b0`다. D44/ADR0044를 먼저 등록하고 H `bdbd4fb2ea9c5b446648778ea7b0b2da14635049` 다음 별도 동작 변경으로 진행한다. B4 네 빌드·평가기·기준은 외부에 고정했고, B5 코드가 그 평가에 들어가지 않는다. 원시 근거는 `/Users/rexxa/work/mux-data/stage-b/`에 둔다.

## 데이터와 저장 경계

기존 물리 Wood MaterialId를 종으로 사용한다. schema1 flora는 Forest 저지대0–159 oak600/pine250/birch150, 고지대160–255 200/650/150의 작성된 게임 분포다. 기존 나무 위치와 원목 수 draw를 유지하고 별도 keyed draw만 추가한다. 이름 순서·정수 누적 선택, strict 필드·완전한 고도 coverage·기본 데이터 선검증·명시적 전체 분포 교체를 사용한다.

format10 ContentState는 저장된 기본 재료와 다른 나무만 sparse map에 둔다. 새 데이터의 default로 옛 나무를 재해석하지 않는다. format9 body를 동결하고 실제 H writer의 tick4000 저장 SHA `e315e1c952b7e0504fd7cbe3b46c867b8c16d0735787e27e11ddab65f6894141`을 이행 근거로 사용한다. 6–9의 기존 나무는 codec에서 역사적 oak로 이행하고, 옛 재료 수치를 덮지 않은 채 빠진 검증된 Wood만 추가한다. 현재 format10은 정확한 catalog revision을 요구한다.

## 표현과 실제 경로

get_trees에 좌표와 정렬된 species 열을 추가한다. 일반 나무·제거된 나무의 그루터기·¼× canopy가 같은 저장된 종을 읽으며 독립 종 해시는 삭제한다. size/variant/jitter/season은 유지한다. 기존 저장에서 화면 종이 바뀌는 경우는 예전 물리 oak와 독립 visual hash의 불일치를 바로잡는 것이며, 옛 화면 동일성은 주장하지 않는다.

필수 실제 경로는 원래 생성 좌표의 소나무 벌목→실제 소나무 원목→일반 운반/건축 명령의 소나무 벽이며, 참나무 벽 대조와 벌목 전/운반 중 저장 재개를 포함한다. 생성된 나무를 별도 지도에 복사하는 제안은 필수 증거로 채택하지 않았다. 원래 세계에서 작업자 배치·욕구·이동 간격을 명시적으로 통제한 메커니즘 시험이며 자연 정착지 성능 시험은 아니다. 실제 창은 같은 저장을 읽고 일반 벌목을 이어서 그루터기를 관측한 뒤 완성 저장의 벽을 캡처한다. 실행과 통제된 픽셀 판정은 아래 후속 절에 기록한다.

데이터만 추가 시험은 두 범위를 구분한다. 물리 pine이 이미 있는 제한 fixture에서 flora 구성원 oak/birch→pine 추가를 동일 population helper로 시험한다. 별도 실제 host는 전체 기본 구성원을 유지하고100%pine 분포만 교체한다. 임의 새 물리 재료나 네 번째 종을 추가했다고 주장하지 않는다.

## 독립 소스 검토와 수정

- core 검토: 기본 flora 검증에 병합된 재료 선언을 사용하면 확장이 잘못된 기본 선언을 숨기는 결함을 찾았다(ISS297). 기본 선언과 병합 선언을 분리하고 회귀 시험을 추가했다. 소스 재검토 당시에는 시험 실행 전이었다. 아래 첫 실행에서 회귀 시험 통과를 확인했다. `b5-core-independent-review.md`와 명시적으로 재구성이라 표시한 최초 소스 발췌를 보존한다.
- codec/state/worldgen 검토: 실제 H의 format9 필드 순서·변하지 않은 resource/tree wire·명시적 revision·기본 재료 독립성·원목 소멸 전 검증을 확인했다. 작성 중 STREAMS 배열 크기와 잘못된 fixture 변수 참조를 소스에서 발견해 수정했다. 컴파일 실패 로그가 있었던 것은 아니다. 기존 infallible worldgen의 startup invariant 실패 경계는 D44에 명시했고, 런타임 벌목 실패가 나무를 지우지 않도록 별도 검사한다.
- renderer 검토: 차단할 구체적 소스 결함은 찾지 못했다. unknown-species 시험의 style slot 확인은 fallback 이미지의 픽셀 증거가 아니다. 실제 저장 export·창 실행·그루터기/벽/계절 캐시 결과는 별도 검증 대상이다.

세 검토는 소스 검토다. 빌드·시험·실제 과거9 이행·OFF 재캡처·시각 수용을 대신하지 않는다. root formatting 뒤 고정 B4 평가가 끝난 다음 아래 컴파일·집중 실행을 직렬로 수행했다.

## 첫 집중 실행 — 2026-10-25

`b5/focused-first-receipt.json` SHA `3ade08bdcd1cccf6209a6114cd4713b248de0ccdaaceb4b28238ad69bd7dd7bd`: 최초 제품2123파일 hash 전후 동일, 여섯 명령 모두 exit0. core 단위389·통합 parent38, 실제 제한 flora 구성원 추가→일반 벌목/벽2, 벌목 identity1, save 단위23, b5_flora parent6(별도 host child1 포함 실행), tree_snapshot3 통과. parent의 host child entry는 일반 실행에서 돌아오는 자리이므로 여섯 개 독립 실험이라고 확대하지 않는다.

실제 H9 fixture SHA를 다시 확인하고 환경변수로 지정해 옛 나무 oak 이행·벌목·format10 저장 계속 실행을 확인했다. 새 seed7의 원래 pine(0,48)에서 원목4개를 얻어 벽(0,45)/(5,47)을 완성했고, 이어 원래 oak(0,32)에서 대조 벽을 만들었다(완료 tick368). 생성 나무·원목·완성 벽을 삽입하거나 옮기지 않았다. 작업자 위치/욕구·M3·이동 간격 통제는 export receipt에 명시한다.

`pine-chain-1`의 before_chop SHA `59ce4ccfd71378071b6b3553b37276af5919c07b303b707114b7909bb79f24db`, mid_delivery `3cef4df68b1084f3f9f889b3d09ce9171f70461b20432114b4b73654ef96cfaa`, completed `de62954c4770b2081ee4671094e1a7d8136fa4b59927efc3067a89d88e9a30e9`를 보존한다. 일반 명령 경로와 두 저장 경계의 직렬화 byte 동일성 증거이며 CLI replay·픽셀 수용·자연 정착지 성공 증거는 아니다. ISS297–299의 소스 수정은 이 실행으로 확인했다. 이 첫 집중 실행 시점에는 기존 회귀·Godot·전체 gate가 대기였으며, 뒤의 절이 이후 회귀·창 실행 결과를 기록한다.

## 기존 회귀와 실제 창 후속
첫 기존 회귀에서 명시적 catalog boot가 없던 steward 시험이 실패했다(ISS301). 새 프로세스 단독 실패를 재현하고 시험 준비만 수정한 뒤 단독1통과, 전체 steward9통과/1ignored, data_only_ext2 및 child, save_load7, 실제 역사 저장4, workspace all-targets strict clippy와 bridge build가 통과했다. `/b5/regression-first/`와 `/b5/regression-second/`는 모두 보존한다.
Godot import·변경5파일 strict 및 FFI57 일치, headless renderer10, 기존 MART21 headless가 통과했다. MART21 rendered 부분은 SKIP이며 창 증거로 대체 해석하지 않는다.
첫 실제 창과 진단 재실행은 그루터기 표시 실패를 보존한다. frame facts로 초기 마을 중심 이동이 대상좌표를(-5696,-2020)으로 보낸 것을 확인했고, 해당 칸 자원 겹침은 없었다. 첫 focus 수정도 이전¼zoom으로 clamp한 뒤1zoom으로 바꾸는 순서 문제로 실패했다. 순서와 명시적 촬영 영역 수정 후 `visual-focused2`는 failures0이나, 독립 이미지 검토에서 주변 수관이 원래 그루터기와 첫 벽을 가리는 한계를 발견했다. D44에 추가 일반 벌목900tick 및 기존 두 번째 pine벽 촬영을 사전 기록했으며 픽셀 판정은 별도다. 제품 카메라·renderer 동작은 이 촬영 수정에서 변경하지 않았다.

실제 창 최종 `visual-haul-clearance`는 54검사0실패, 종료0, SCRIPT ERROR/Parse Error/ERROR 없음이다. 추가 일반 벌목만 한 `visual-clearance`는 원목 더미가 그루터기를 가렸다. 동쪽 일반 stockpile을 더한900tick 후속에서 실제 운반으로 원목이 이동했고 원래(0,48)그루터기(화면288,540)가 드러났다. 소나무 벽은 같은 원래 게임플레이가 완성한(5,47), 참나무 대조는(0,27)이다. 초기 수관 가림, 원목 가림, 카메라 실패 캡처는 삭제하지 않았다. 수관이 밀집한 원래 standing 장면을 단독 종 실루엣 판독 PASS로 주장하지 않으며 saved species와 실제 sprite 경로의 일치 증거를 구분한다. 원시 PNG6개·receipt·log 해시는 `b5/window-final-receipt.json`에 연결한다.

최종 창 재검토 `b5/b5-window-review-final.md/.json`은 PNG6개 SHA 일치를 재확인하고 노출된 원래 그루터기·pine/oak벽 대비에 bounded PASS를 기록했다. 담당 구현자의 재검토이므로 독립 소유자 승인으로 부르지 않는다. 현재 [화면 모음6장](../../images/mux_contact_sheet.png)은 장면별 가림·통제 조건을 표기하며 원본1920×1080은 외부에 유지한다.

## 최종 OFF 안전망 재캡처
릴리스 probe `e402d51b809680f52b377dffe59af5b15f811d8a2d3b7f2d71009d9beb723951`, 제품2123경로 freeze `e0bb069c051ef1164c37f4e15527cb7189ab2c5a8da296fe903110096c86e7d9`로 재캡처했다. `b5/final-safety1-evidence/receipt.json`은 CAPTURE_VALID_ORIGINAL_BASELINE_IDENTICAL, 실패0·소스 drift0이다. 원본 공유228748값 차이0, Chronicle64파일19683항목 동일(기존 git_sha 제외), 정규화 raw digest 및 결정성이 통과했다.
H 대비9 OFF 요약은 모두 달라졌다: soak1548, story4313, neighbours4800, knowledge_f10443, m55_pressure19939, m55_nodraft20331, m55_storyoff19135, m55_noneighbours18349, m55_quiet18268개 JSON 경로. 정확한9 seed집합과 불변식 위반0을 확인했고 모든 원문·차이 파일을 보존했다. 이 동작 변경을 colony byte identity라고 부르지 않는다. 독립 차이 해석과 동일 조건 ON 재캡처의 완료 기록은 다음 절을 따른다.
첫 빌드는 ignored Cargo.lock 사전 해시를 남기지 않았다. 이를 숨기지 않고 후속 `--locked` 릴리스 빌드에서 lock SHA `ddc99ac9fc161d9b610c0f5d8dbf5e279c8908c10717647ba50ab2fef70c36d1` 전후 일치·probe/perf 바이너리 모두 기존 freeze와 일치를 확인했다. 최초 기록 공백을 소급해 메운 것으로 표현하지 않는다. 추가 scene/shader/localization/gdextension 입력은 `supplemental-inputs-postbuild.json`의 현재 관측과 scope로 분리한다.

## 같은 입력의 ON 재캡처
`b5/final-on1-evidence/receipt.json`은 seeds0–7×48일, steward/story/neighbours ON, 입력 command script·B4 observer 없이 실행한 capture가 종료0·소스 drift0임을 확인한다. frozen A와 literal 동일했던 ecology ON 기준9파일의 해시를 시작 전에 검사했다. 요약20053경로, 시드0–7 명령 기록은 각각161/399/438/437/425/340/134/2경로가 달랐다. 이는 A 이후 P/T/H/B5의 누적 동작 변경 비교이며 B5만의 인과 효과나 A8 재수용 평가가 아니다. 모든 차이 원문을 보존한다.

## 독립 차이 검토와 최종 준비 검사
`final-off-independent-review.md/.json`은 OFF27파일 해시와 전체107126차이 행, ON18개 before/after 해시와 전체9차이 목록을 별도 재계산했다. MaterialId가 다른 ItemSpec은 별도 stack이 되고 제작 입력 재료·경도가 달라져 운반/제작/후속 정착지 결과로 이어지는 경로가 소스와 일치한다. 실제 예는 soak seed0 도끼 벌목100→101, story seed0 alive10→11·birth0→1, quiet seed0 제작200→180이다. 변경 경로 수는 배열 이동도 포함하므로 독립 사건 수나 심각도 점수가 아니다. ContentState 자체가 fingerprint에 들어가 초기부터 달라질 수 있으며 최초 벌목 시점이나 모든 결과의 정확한 인과를 증명하지 않는다. 이웃 OFF에서 달라진 neighbours 하위 값은 discovery뿐이고 caravan/wave는 유지됐다. 검토 범위에서 별도 정책 수정이나 원시 근거 불일치는 발견하지 못했다.
최종 준비 검사는 fmt·bridge API·schema(57getter/838key)·docs·inventory·GDScript strict70/FFI57·art manifest·확장성/마스크/import29시험 모두 종료0이다(`precommit-checks/receipt.json`). 이후2123 source freeze도 불변이다. 전체36단계 B gate와 clean HEAD 게시는 별도다.

## Full-stage gate follow-up

First B full gate at60f3371691b398e16a9d3675ea3e36b058559616 finished31/36, status1. Its five stale test/setup defects and focused corrections are recorded in [B_GATE_REPAIR_VERIFICATION.md](B_GATE_REPAIR_VERIFICATION.md). The original2123-path freeze stays unchanged:2118paths match and five test paths differ. No new product behavior or recapture is claimed by the test repair. New clean full-stage gate remains required.
