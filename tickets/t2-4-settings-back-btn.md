# T2-4: pause_menu.gd 설정 화면 뒤로 버튼 텍스트 수정

## Objective
`pause_menu.gd`에서 설정 화면(STATE_SETTINGS)의 "뒤로" 버튼이 텍스트 없이 빈 버튼으로 표시되는 버그를 수정한다.

## Non-goals
- 설정 메뉴 레이아웃 변경 불가
- 기존 기능 변경 불가
- 다른 파일 수정 불가 (JSON 제외)

## Scope
Files to touch:
- `scripts/ui/pause_menu.gd` — settings back 버튼 텍스트 수정

## 버그 원인

`_build_ui()`에서 settings container의 뒤로 버튼을 로컬 변수로 생성하고 있어, `_refresh_texts()`에서 접근 불가:

```gdscript
# ❌ 현재 (line ~_build_ui):
var btn_settings_back := _create_button("", Callable(self, "_on_settings_back"), 16)
_settings_container.add_child(btn_settings_back)
# → btn_settings_back이 로컬 변수라서 _refresh_texts()에서 text를 설정할 수 없음!
```

## 수정 방법

멤버 변수 `_btn_settings_back: Button`을 추가하고 `_refresh_texts()`에서 텍스트 설정:

### 단계 1: 멤버 변수 선언 추가
파일 상단의 변수 선언부에 추가:
```gdscript
var _btn_settings_back: Button  # 설정 화면 뒤로 버튼
```

### 단계 2: _build_ui()에서 멤버 변수로 할당
```gdscript
# ❌ 현재:
var btn_settings_back := _create_button("", Callable(self, "_on_settings_back"), 16)
_settings_container.add_child(btn_settings_back)

# ✅ 수정:
_btn_settings_back = _create_button("", Callable(self, "_on_settings_back"), 16)
_settings_container.add_child(_btn_settings_back)
```

### 단계 3: _refresh_texts()에 추가
기존 `_btn_back.text = Locale.ltr("UI_BACK")` 아래에 추가:
```gdscript
if _btn_settings_back != null:
    _btn_settings_back.text = Locale.ltr("UI_BACK")
```

## 확인 방법
1. ESC → "설정" 클릭 → 설정 화면에 "뒤로" / "Back" 버튼 텍스트 표시됨
2. 언어 전환 시 버튼 텍스트도 즉시 변경됨
3. `UI_BACK` 키는 ko: `"<- 뒤로"`, en: `"<- Back"` 이미 존재함

## Acceptance Criteria
- [ ] 설정 화면에서 뒤로 버튼에 텍스트 표시됨 (`"<- 뒤로"` 또는 `"<- Back"`)
- [ ] 언어 전환 시 버튼 텍스트 즉시 변경
- [ ] GDScript 문법 오류 없음
- [ ] 기타 기능 변경 없음
