# B4 O 관측·독립 산술 검증

**관측 단위의 최종 안전망 PASS. 홀드아웃 acceptance는 NOT_CLAIMED이며 전체 Stage B gate·게시 전이다.** 기준은 B6 `c9ed292d77902f30f5c8975f01d770bd43e52927`. 원문은 `/Users/rexxa/work/mux-data/stage-b/b4/`에 보존한다. ADR0040/D39와 원문 사전등록을 적용했고 P/T/H 동작은 아직 바꾸지 않았다.

## 변경

명시적 세 옵션으로만 켜지는 read-only probe 관측기, 실제 Cold/paid-craft 제어 fixture, stdlib-only 독립 scorer를 추가했다. pre-award craft 입력과 사망자 포함 persistent tally를 대조하고 명령·이주·상인·일별 상태 및 미해결 운반 관측을 남긴다. RuntimeSystem, 저장 필드, production rule 또는 의존성을 추가하지 않았다. 관측 OFF의 일반 출력은 기존과 같다.

## 실행 증거

| 검증 | 결과 | 원문 |
|---|---|---|
| 최종 fmt·strict clippy | PASS; clippy15.57초 | o-fmt-third.log, o-clippy-second.log |
| 관측기 Rust 시험 |7통과/0실패|o-probe-tests-final.log|
| 제어 fixture |2통과/0실패, 최종1.70초|o-controlled-final.log|
| Python scorer |35통과/0실패|o-scorer-unit-first.log|
| 최종 독립 calibration score |VALID, acceptance NOT_CLAIMED|o-final-calibration-index.json, o-final-calibration-score.json|
| 원본 identity |228748공유값 차이0,64Chronicle/19683항목 동일(기존 git_sha 제외), raw digest·결정성 일치|o-safety1-receipt.json|
| 원본 OFF9개 요약 |literal-byte 동일, 정확한 seed집합·불변식 위반0|o-safety1-compare.log|
| Stage A no-player-orders ON |8시드×48일 요약·명령8개 literal 동일|o-on2-receipt.json|
| 최종 관측 OFF/ON |8시드×48일 일반 요약 exact, 명령8개 literal 동일|o-final-calibration-equality.json|
| 실제 craft 후 저장 |tick4000, B6/최종OFF/최종ON bytes 동일; 해당 checkpoint 전 제작17건|o-final-save-receipt.json, post-craft-equivalence-receipt.json|
| 초기 저장 |tick100, B6/calibration OFF/ON bytes 동일|short-equivalence-receipt.json|
| 잘못된 CLI·metadata |옵션 누락·hash형식·미지 필드·다른 catalog 네 경우 exit1, summary 없음|negative-cli/receipt.json|

최종 probe SHA256 `865ec8e76d8c22e7646583878ca265d86ed7c311eddb484493ba76ee06ab466d`; 보관 제어 실행 파일은 `08c386c4ccd28cedc6b30aafba5cfdbb01a4c81232219ebdefa4f258184ed8e5`. 두 실행 파일은 시스템 dylib만 참조한다. 제품2102경로 해시 불변을 최종 실행 뒤 확인했다(`o-final-source.json`). Rust1.93.1/aarch64-apple-darwin, probe release/fixture debug-test이며 빌드는 O 커밋 전의 동결 소스로 수행했다. 카탈로그는 `catalog-v1-fnv1a64:10279805eb0560ff:44709`; 최종 metadata SHA는 `14ec78c35779a28039d4d97b57e2a363d51e93a9520cbfd4c549dfc3f08e46fb`. 이전15-source manifest와 확장한 최종22-source manifest를 각각 보존했다.

## 보정 결과와 해석

8시드×48일의384개 일별 행, primary craft1773건(`205,223,243,213,272,262,101,254`), 제어 Cold2304행과 paid-craft32건을 검증했다. 별도 검토자는 첫 calibration의 모든1773품질·32제어품질·2304keyed Cold와 입력해시·tally를 독립 재계산했다(`o-calibration-review.md`). 최종 scorer도 VALID다. Ledger/reservation 위반은0이다.

자연 F4a/F9a 조건은 보정8/8이지만24/32 홀드아웃 기준으로 환산하지 않는다. 자연 Cold는 clothed0/8460, unclothed0/7332여서 모든 비율이 NOT_MEASURED다. 의복 보호 또는 생존 효과 PASS가 아니다. 제어 각 그룹768표본에서 none768/hide529/fur381, keyed mismatch0; hide E2688/5·V4032/25, fur E384·V192이며 보정 산술 검사가 통과했다. 평가의 garment당3072표본·craft128회/build는 아직 수행하지 않았다.

자연 haul의 unresolved starts는0, unresolved terminal은80이다. job 소실만으로 완료·취소를 구별하지 않으며 종료 시 진행 중인 haul도 성공으로 세지 않는다. 정확한 H 효과는 별도 제어 검증으로 증명해야 한다. 선택적 전체 M55 controls는 사전등록대로 NOT_RUN이다.

## 보존한 실패

- 최초 fixture는 AgentId가1에서 시작한다고 가정해 cold288호출·craft32건이 무효였고 시험1통과/1실패했다. 공개 allocator로0을 예약하여 사전등록1–8을 유지한 뒤 재실행했다. 최초 무효 자료와 ISS286 보존.
- 최초 fmt 모듈 순서, clippy nonminimal_bool 및 BPT/BPTH 내부 enum 표기3진단을 수정했다. 동치 is_none_or와 명시적 serde rename으로 외부 wire명을 유지했다. 최초 실패 및 ISS287 보존.
- 최초 ON 비교는 기준과 달리 M55-U 플레이어 명령을 추가해30882경로 차이가 났다. 과거 네 기준의 timing.script=null과 첫 tick360 명령 차이를 독립 확인했다. 같은 원래 입력으로 ON2를 실행해 literal 동일성을 확인했다. 소스·golden을 바꾸지 않았으며 최초 결과와 ISS288 보존.
- 최초 release 빌드는 잘못된 저장소 루트에서 Cargo.toml 오류로 실패했다. rust/에서 재실행했고 오류 원문·ISS289를 보존한다.

## 증거의 경계

일곱 번째 관측 시험은 실제 paid craft를 engine.tick으로 완료한 뒤 실제 decay/despawn을 시험이 수동 호출한다. pre-input/tally 유지·XP crossing·continuing bill·취소를 증명하며 자연 scheduler의 craft/death 우연 일치를 주장하지 않는다. 같은 tick 동시 craft reconciliation은 합성 시험이다. 제어 F9는 ordinary payment 후 고정 tick27360의 실제 finish_craft 경계이며 ticks_left=1 또는 자연 작업시간·skill speed 증거가 아니다. F4는 새 세계마다 damage만 끈 실제 FireSystem 호출이며48일 생존 실험이 아니다.

자연 Cold 분모에는 따뜻한 집 안 성인도 있고 의복 availability에는 아이도 포함된다. 계절 sampling과 FireSystem evening-season을 동일시하지 않는다. Caravan 수집은 감사한 현재174/175 및 후속 writer 일정에 한정된다. 명령 purchase tick과 engine 완료 tick을 구별하며 이주 인구는 tick-boundary 관측이라고 표시한다. 신규 system으로 일정이 바뀌면 다시 검토한다(`o-observer-review.md`).

Rust는 전달된 SHA의 형식·metadata를 검사하고 runner/scorer가 실제 bytes를 확인한다. provenance 내부 설명은 root 책임이며 hash검증만으로 모든 환경 설명이 독립 감사되었다고 주장하지 않는다. LSP는 ISS284대로 불가; compiler/clippy가 LSP clean을 뜻하지 않는다. P/T/H 및128개 홀드아웃 세계, 전체 B gate·게시가 남아 있다.
