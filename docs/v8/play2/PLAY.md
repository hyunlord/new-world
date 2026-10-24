# WorldSim 플레이 안내 — M-ART2.1 / V8-PLAY2

## 한 줄로 시작

```bash
/Users/rexxa/work/new-world-play2-dev/tools/play.sh main
```

Godot 4.6과 Rust `cargo`가 필요하다. 실행기는 원격 `v8/main`을 확인하고 `/Users/rexxa/work/new-world-play/main`에 게임을 준비한다. 필요한 debug 브리지 빌드·에셋 가져오기 뒤 창을 연다. 준비만 하려면 같은 명령 끝에 `--update-only`, 창 없는 확인은 `--headless-check`를 붙인다. 실행기 체크아웃 `new-world-play2-dev`는 지우지 않는다. **플레이 루트 자체에 `git worktree add`를 하지 않는다.**

더블클릭 바로가기는 오너가 한 번만 다음 명령으로 연결한다. 기존 바로가기는 `new-world-pass4`를 가리키므로 이 작업에서는 실행하거나 바꾸지 않는다.

```bash
ln -sf /Users/rexxa/work/new-world-play2-dev/tools/play/WorldSim.command ~/Desktop/WorldSim.command
```

그 뒤 바탕화면 `WorldSim.command`를 더블클릭한다. 실제 실행·검증 상태는 [PLAY2_REPORT.md](PLAY2_REPORT.md)를 확인한다.

## 첫 플레이

기본 1× 확대의 캠프에서 시작한다. 64px 타일, Route I 사람·초상, 재료별 벽, 초록 나무와 둥근 물가가 현재 화면이다.

1. **Space**로 멈추고 **H**로 캠프를 찾는다. 사람을 눌러 초상·상태를 본다.
2. **F2**로 빈 땅을 드래그해 비축 구역을 만든다.
3. **F1**로 벌목 구역을 지정하고 Space로 재개해 운반을 본다.
4. **F3→5** 벽을 놓되 문 자리를 남긴다. **→6** 문, **→7** 바닥, **→8** 침상. Shift+벽 드래그는 닫힌 테두리이므로 문 자리의 청사진은 F6으로 취소한다.
5. **R**로 방 보기를 켜고 방 내부를 눌러 침실·지붕을 확인한다. 다른 방은 창고 역할과 내부 비축 구역을 정한다.
6. **P**에서 작업 우선순위를 정하고 작업장을 클릭해 제작 주문을 넣는다.
7. **K**에서 기술·스승·제자를 골라 가르치기를 명령한다.
8. **T**에서 기술 보유자와 전승 위험을 확인한다.
9. 늑대에 대응할 사람을 클릭하고 **D**로 징집, 오른쪽 클릭으로 이동·공격, 다시 D로 해제한다.
10. **N**으로 이웃을 확인하고 이주민·교역 배너에서 수락·거래를 고른다. **F5**로 저장한다.

**조작:** 1–4 속도(메뉴에서는 항목 선택), 휠/`-`/`=` 확대 ¼×·½×·1×·2×, WASD/방향키/가운데 버튼 드래그 이동, C 연대기, Q 타일 기록, M 기분, L 언어, F9 마지막 저장 불러오기. 사람 선택 중 D는 징집이므로 오른쪽 이동은 →를 쓴다. 명령 종료는 Esc/오른쪽 클릭. 기능 키는 macOS 설정에 따라 fn을 함께 누른다. **전체 키와 코드 근거:** [KEYS.md](KEYS.md), [README 조작표](../../../README.md#전체-조작표).

**F12 화면 조정:** 실행 중인 `new-world-play/main/assets/v8art/render_params.json`을 작업 트리 밖에 백업한 뒤 편집한다. `walls.top_value`는 벽 밝기, `ground.tone.*`는 지면 톤, `water.*`는 물결·물가·대비, `people.night_halo_alpha`는 야간 인물 테두리, `season.snow_alpha`는 눈, `lod.people_below_zoom`·`glyph_zoom`은 마커·활동 기호 임계값이다. 실행기의 debug 빌드에서 F12를 누르면 “렌더 설정 다시 읽음” 토스트가 뜬다. 실험 뒤 파일을 복원하고 F12를 다시 누른다. 로컬 수정이 남으면 실행기가 거절한다.

## 옛 버전 실행하기

```bash
/Users/rexxa/work/new-world-play2-dev/tools/play.sh v7
/Users/rexxa/work/new-world-play2-dev/tools/play.sh f0b759fe29d1f8c16f4ae6a994807036933b50df
```

`m1`, `v8-0`, 태그·커밋·브랜치도 가능하다. 버전마다 별도 빌드 공간을 쓴다. 조사 시작 시 `m1` 1.4G, `main` 3.5G, `v7` 4.3G였다. 과거 버전의 조작·저장 호환성은 현재 버전과 같다고 가정하지 않는다.

## 오래된 플레이 작업 트리 정리하기

게임을 정상 종료한 뒤 등록 목록과 대상의 수정 여부부터 확인한다. 다음은 **v7을 더 쓰지 않을 때의 예시**다.

```bash
git -C /Users/rexxa/work/new-world-play2-dev worktree list --porcelain
git -C /Users/rexxa/work/new-world-play/v7 status --short --untracked-files=all
du -sh /Users/rexxa/work/new-world-play/v7
```

목록에 이 클론의 정확한 경로가 등록되어 있고 수정이 없으며 보관할 파일이 없는지 확인한 뒤에만 다음을 실행한다. 거절되면 이유와 파일을 확인하고 보존한다. `--force`나 `rm -rf`로 우회하지 않는다. 목록에 없는 디렉터리는 이 절차로 지우지 않는다.

```bash
git -C /Users/rexxa/work/new-world-play2-dev worktree remove /Users/rexxa/work/new-world-play/v7
```

**실패했을 때:** `/Users/rexxa/work/play-data/logs/`의 출력된 로그를 확인한다. fetch 실패는 마지막으로 받은 버전 사용 경고이며, 받지 않은 Rust 의존성까지 오프라인 준비를 보장하지 않는다. 로컬 수정은 직접 보관·복원하고 다시 실행한다. 저장 파일은 플레이 작업 트리가 아닌 Godot `user://saves/`에 있다.
