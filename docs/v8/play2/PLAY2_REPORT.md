# V8-PLAY2 실행 검증 보고

상태: 구현·검증 진행 중. 전체 gate와 원격 게시가 끝나기 전에는 완료로 보지 않는다.

## 조사 결과

`/Users/rexxa/work/new-world-play/`에는 이 클론의 깨끗한 등록 작업 트리 main(3.5G), m1(1.4G), v7(4.3G)이 있었다. main은 d796fcac에서 M-ART2.1 f0b759fe로 갱신했다. 최초 debug 빌드 3초, import 4초, 헤드리스 300프레임 exit 0이었다. Godot 4.6은 Downloads의 앱을 사용한다. 자세한 로그·범위는 [조사 기록](PLAY2_FINDINGS.md), 키별 소스 근거는 [KEYS.md](KEYS.md).

## 고친 것

실행기가 수정된 작업 트리·다른 클론·갈라진 이력을 거절하고, 컴파일에 포함되는 데이터 변경도 Cargo의 증분 빌드로 확인한다. 오류 출력과 종료 코드를 함께 검사하며 `--update-only`로 창 없이 준비할 수 있다. README와 [PLAY.md](PLAY.md)는 최신 조작과 열 단계 첫 플레이, F12 설정을 설명한다. 게임 코드와 아트는 변경하지 않았다.

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

OS 화면 캡처와 System Events 접근이 실패했다. 창의 실제 viewport와 프로그램 입력은 별도로 확인하지만, 물리 키보드·fn·한글 입력기·Finder 더블클릭 성공을 주장하지 않는다. 기존 Desktop 링크의 대상에서 실행하지 말라는 지시 때문에 해당 링크는 열지 않았다. 새 실행기를 담은 별도 체크아웃은 오너 명령을 유지하기 위해 남긴다.

## Gate 결과

대기. gate 명령은 깨끗한 커밋에서 `bash tools/v8/gate.sh /Users/rexxa/work/play-data/gate-play2 --base-data /Users/rexxa/work/m55-data/base/identity`다. 기준 데이터 경로가 존재한다. status 0 전에는 원격을 갱신하지 않는다.

## 원격 확인

조사 시 `git ls-remote origin v8/main v8/wip` 두 브랜치는 모두 f0b759fe29d1f8c16f4ae6a994807036933b50df였다. 최종 결과는 게시 후 기록한다.
