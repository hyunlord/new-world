# CLAUDE.md

## 프로젝트 개요
Godot 4.3, Windows 환경, 2D 액션 RPG

## 기술 스택
- GDScript (정적 타이핑)
- Godot 4.3 stable (Windows 64bit)

## 코드 컨벤션
- 클래스명: PascalCase
- 변수/함수: snake_case
- 시그널: past_tense (health_changed)
- 타입 힌트 필수: var speed: float = 200.0

## 구조
- scenes/ — .tscn 씬 파일
- scripts/ — .gd 스크립트
- resources/ — .tres 리소스
- assets/ — 이미지, 사운드
- autoload/ — 글로벌 싱글톤
```

## 7. 바이브코딩 실전 워크플로우
```
┌─────────────┐    LSP     ┌──────────────┐
│ Godot Editor │◄─────────►│    VSCode     │
│ (씬 편집/F5) │           │ (코드 편집)    │
└─────────────┘           └──────┬───────┘
                                 │ 터미널
                          ┌──────▼───────┐
                          │  Claude Code  │
                          │ (자연어→코드)  │
                          └──────┬───────┘
                                 │ git push
                          ┌──────▼───────┐
                          │    GitHub     │
                          └──────────────┘
```

### 실전 루프

1. **Godot Editor는 항상 켜둠** (LSP 서버 역할)
2. **VSCode 터미널에서 `claude` 실행**
3. 자연어로 지시:
```
> 플레이어 CharacterBody2D 만들어줘. 
  WASD 이동, 점프, 중력 적용. scripts/player.gd로 저장해줘.

> 방금 만든 플레이어에 대시 기능 추가해줘. 
  Shift 키로 이동 방향으로 빠르게 대시, 쿨다운 0.5초.

> 에러 나는데: "Invalid call. Nonexistent function 'get_input_vector'"
  이거 고쳐줘.
