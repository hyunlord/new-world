# V8-PLAY2 실행 검증 보고

전체 gate **34단계 status 0**. 검증된 구현 커밋 `72d2db1ffff59490f6212e55a6eec7010e7b5bd5`를 `v8/wip`·`v8/main`에 fast-forward 게시했고, 오너용 실행기로 내려받아 헤드리스·창 실행을 재확인했다. 이 보고서는 이후 문서 전용 커밋이며, 전체 gate가 실행된 SHA와 구별한다.

## 조사 결과

`/Users/rexxa/work/new-world-play/`에는 이 클론의 깨끗한 등록 작업 트리 main(3.5G), m1(1.4G), v7(4.3G)만 있었다. 삭제한 항목은 없다. main은 d796fcac에서 M-ART2.1 f0b759fe를 거쳐 게시된 72d2db1f로 갱신했다.

| 실행 | fetch / 작업 트리 | debug 빌드 / import | 결과 |
|---|---|---|---|
| 최초 실행 | 기존 실행기에 타이머 없어 미측정 | 3초 / 4초 | 300프레임 exit 0 |
| `.command --update-only` | fetch 1초 | 0초 / 4초 | exit 0 |
| 게시 후 헤드리스 | 0초 / 0초 | 0초 / 최신이라 생략 | exit 0, ERROR 0줄 |
| 게시 후 창 실행 | 1초 / 0초 | 0초 / 최신이라 생략 | 실제 Metal 창, 자연 종료 exit 0 |

Godot은 `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot` 4.6이며, `rust/target/debug/libsim_bridge.dylib`를 로드한다. 게시 후 로그는 `/Users/rexxa/work/play-data/logs/main-20261025-022952-22528.log`와 `main-20261025-023003-24417.log`다. 실행기 파일은 개발 체크아웃과 play/main에서 바이트가 같다.

창 입력 검증 **79개 성공, 실패 0**: 기본 1×·새 아트, Space, 1–4 속도, 휠·-/=의 ¼×/½×/1×/2×, 저배율 인물 마커, 클릭·초상, C/T/N/P/R, D 징집·해제, F5/F9 저장·복원, F12를 확인했다. 실제 벽을 건설한 뒤 `walls.top_value` 0.80→0.15의 토스트와 벽 윗면 변화까지 확인하고 원본 바이트를 복원했다. 10초·30초 캡처 및 JSON은 `/Users/rexxa/work/play-data/shots/play2/gui2/`에 있다. 이는 실제 창에 합성 입력을 넣은 검증이다.

자세한 로그·실패 이력은 [조사 기록](PLAY2_FINDINGS.md), 모든 키의 코드 줄 근거는 [KEYS.md](KEYS.md).

## 고친 것

`tools/play.sh`는 수정된 작업 트리·다른 클론·갈라진 이력을 거절한다. 불완전한 파일 확장자/mtime 검사를 없애고 Cargo 증분 빌드로 내장 데이터 변경·삭제까지 확인한다. 오프라인이면 경고 후 마지막 fetch 상태를 사용한다. import·헤드리스·창 실행의 오류 출력과 종료 코드를 함께 검사하며, 긴 출력의 SIGPIPE 누락도 고쳤다. `--update-only`로 창 없이 준비할 수 있다.

격리 회귀 **44개 성공, 실패 0**. `tools/play/test_launcher.sh`와 창 확인용 `play2_gui_check.gd`를 추가했다. README와 [PLAY.md](PLAY.md)는 최신 조작·열 단계 첫 플레이·F12·옛 버전·안전한 정리를 설명한다. 변경은 허용된 실행기/안내 경로와 CHANGELOG/HISTORY/ROADMAP 각 한 줄뿐이다. 게임 코드와 아트는 변경하지 않았다.

## 오너가 할 일

실행 한 줄:

```bash
/Users/rexxa/work/new-world-play2-dev/tools/play.sh main
```

바탕화면 바로가기는 존재하지만 이전 pass4를 가리키므로 다음 한 줄이 필요하다. 이 작업에서는 Desktop을 변경하지 않았다.

```bash
ln -sf /Users/rexxa/work/new-world-play2-dev/tools/play/WorldSim.command ~/Desktop/WorldSim.command
```

## 남은 문제

OS 화면 캡처와 System Events 접근이 실패해 물리 키보드·fn·한글 입력기·Finder 더블클릭은 미검증이다. `.command`의 셸 실행은 성공했다. 기존 Desktop 링크는 실행 금지된 pass4를 가리켜 열지 않았다. Desktop은 변경하지 않았다. 새 실행기를 담은 별도 체크아웃은 오너 명령을 유지하도록 깨끗한 상태로 남긴다. shellcheck는 설치되지 않아 실행하지 못했고, `bash -n`과 문서 검사는 통과했다.

## Gate 결과

2026-10-25 01:18:11–02:29:34 KST, **71분 23초**, clean HEAD `72d2db1ffff59490f6212e55a6eec7010e7b5bd5`, dirty 0.

```bash
bash tools/v8/gate.sh /Users/rexxa/work/play-data/gate-play2-final --base-data /Users/rexxa/work/m55-data/base/identity
```

- 34단계 모두 exit 0, 최종 status 0.
- debug/release 각각 2,025개 성공, 실패 0. 실행기 gate 6/6.
- identity: 공유 값 228,748개 차이 0, 연대기 64파일·원시 샘플 64파일 일치. 결정성·리플레이·저장 왕복 통과.
- perf: 평균 +10% 초과 없음. 정착지 bench의 시간·메모리 한계 모두 통과. nav 133.8µs/tick ≤600µs.
- soak·지식·이야기·이웃 통과. resolver 39 material +352 flora 차이 0, 아트 렌더 38개 통과.

전체 증거: `/Users/rexxa/work/play-data/gate-play2-final/gate.txt`. 앞서 결함 발견으로 중단한 `/Users/rexxa/work/play-data/gate-play2/`는 성공으로 세지 않았다. 후속 보고서 커밋은 실행기·게임 내용이 동일함을 확인하고 문서 검사를 별도로 수행한다.

## 원격 확인

게이트 성공 후 `git push origin v8/play2:v8/wip`, 이어 `git push origin v8/play2:v8/main` 모두 fast-forward 성공. 구현 게시 직후 `git ls-remote origin v8/main v8/wip`:

```text
72d2db1ffff59490f6212e55a6eec7010e7b5bd5 refs/heads/v8/main
72d2db1ffff59490f6212e55a6eec7010e7b5bd5 refs/heads/v8/wip
```

보고서 전용 커밋 게시 후의 최종 HEAD·원격 일치·clean 확인은 `/Users/rexxa/work/play-data/play2/delivery-final.txt`에 기록한다. 이 문서에 자신의 커밋 SHA를 적어 검증 SHA와 혼동하지 않는다.
