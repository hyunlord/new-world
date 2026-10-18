class_name PersonNames
extends RefCounted

const Hash = preload("res://scripts/ui/art_hash.gd")
const EN: Array[String] = ["a", "i", "o", "e", "ka", "ki", "ku", "ko", "ta", "ti", "tu", "to", "na", "ni", "nu", "no", "ma", "mi", "mu", "mo", "ra", "ri", "ru", "ro", "sa", "si", "su", "so", "ha", "hi", "hu", "ho", "ya", "yu", "yo", "wa", "ba", "bi", "bo", "bu", "da", "do", "ga", "go", "de", "ge"]
const KO: Array[String] = ["아", "이", "오", "에", "카", "키", "쿠", "코", "타", "티", "투", "토", "나", "니", "누", "노", "마", "미", "무", "모", "라", "리", "루", "로", "사", "시", "수", "소", "하", "히", "후", "호", "야", "유", "요", "와", "바", "비", "보", "부", "다", "도", "가", "고", "데", "게"]
const BLOCK_EN: Array[String] = ["nig", "kak", "baka"]
const BLOCK_KO: Array[String] = ["시바", "바보", "미치", "시비"]

static func display_name(agent_id: int, lang: String) -> String:
	var h: int = Hash.mix(Hash.mix(agent_id) ^ 0x2545F491)
	var syllables: Array[int] = []
	for index: int in (2 if h % 4 == 0 else 3):
		syllables.append((h >> (5 * index + 2)) % 46)
	var attempts: int = 0
	while blocked(_join(syllables, EN), "en") or blocked(_join(syllables, KO), "ko"):
		var last: int = syllables.size() - 1
		syllables[last] = (syllables[last] + 1) % 46
		attempts += 1
		# A blocked prefix (e.g. #282 시바무) survives all 46 suffixes.
		# Only after exhausting those suffixes, carry into the preceding syllable.
		if attempts % 46 == 0:
			for index: int in range(last - 1, -1, -1):
				syllables[index] = (syllables[index] + 1) % 46
				if syllables[index] != 0:
					break
	var result: String = _join(syllables, KO if lang == "ko" else EN)
	return result if lang == "ko" else result.left(1).to_upper() + result.substr(1)

static func blocked(value: String, lang: String) -> bool:
	for fragment: String in (BLOCK_KO if lang == "ko" else BLOCK_EN):
		if value.to_lower().contains(fragment):
			return true
	return false

static func _join(indices: Array[int], table: Array[String]) -> String:
	var result: String = ""
	for index: int in indices:
		result += table[index]
	return result

static func plain(value: String) -> String:
	return value.replace("\u2060", "")

static func unbroken(value: String) -> String:
	return "\u2060".join(plain(value).split(""))

static func present(agent_id: int, lang: String) -> String:
	return unbroken(display_name(agent_id, lang))

static func prepare_rich_text(label: RichTextLabel) -> void:
	label.add_theme_font_override("bold_font", preload("res://assets/fonts/galmuri/Galmuri11-Bold.ttf"))
	label.selection_enabled = true
	if label.has_meta("p12_clean_copy"):
		return
	label.set_meta("p12_clean_copy", true)
	label.gui_input.connect(_copy_input.bind(label))
	label.get_menu().id_pressed.connect(_copy_menu)

static func _copy_input(event: InputEvent, label: RichTextLabel) -> void:
	if event.is_action_pressed("ui_copy") and not label.get_selected_text().is_empty():
		DisplayServer.clipboard_set(plain(label.get_selected_text()))
		label.accept_event()

static func _copy_menu(id: int) -> void:
	if id == RichTextLabel.MENU_COPY:
		_clean_clipboard.call_deferred()

static func _clean_clipboard() -> void:
	DisplayServer.clipboard_set(plain(DisplayServer.clipboard_get()))
