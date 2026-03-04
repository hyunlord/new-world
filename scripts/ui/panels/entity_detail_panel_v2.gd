class_name EntityDetailPanelV2
extends Control

# ── Colors ──────────────────────────────────────────
const CLR_BG := Color(0.08, 0.08, 0.08, 0.95)
const CLR_TEXT := Color(0.80, 0.80, 0.80)
const CLR_HEADER := Color(1.0, 1.0, 1.0)
const CLR_NARRATIVE := Color(0.70, 0.70, 0.70)
const CLR_NAME := Color(0.95, 0.85, 0.45)
const CLR_GOOD := Color(0.50, 0.85, 0.50)
const CLR_WARN := Color(0.95, 0.85, 0.30)
const CLR_DANGER := Color(0.95, 0.35, 0.30)
const CLR_CRISIS := Color(1.0, 0.2, 0.2)
const CLR_DIM := Color(0.45, 0.45, 0.45)
const CLR_BAR_BG := Color(0.15, 0.15, 0.15)
const CLR_BAR_FILL := Color(0.30, 0.55, 0.80)
const CLR_BAR_DANGER := Color(0.80, 0.30, 0.25)
const CLR_VALUE_POS := Color(0.45, 0.75, 0.95)
const CLR_VALUE_NEG := Color(0.95, 0.60, 0.30)
const CLR_SECTION_LINE := Color(0.30, 0.30, 0.30)
const CLR_COLLAPSE_ARROW := Color(0.60, 0.60, 0.60)

# ── Sizes ────────────────────────────────────────────
const FONT_SIZE_HEADER := 14
const FONT_SIZE_BODY := 12
const FONT_SIZE_NARRATIVE := 11
const FONT_SIZE_SMALL := 10
const LINE_HEIGHT := 18.0
const LINE_HEIGHT_SMALL := 15.0
const SECTION_GAP := 12.0
const BAR_HEIGHT := 10.0
const BAR_LABEL_W := 110.0
const BAR_VALUE_W := 40.0
const MARGIN_LEFT := 12.0
const MARGIN_RIGHT := 20.0
const SCROLLBAR_WIDTH := 8.0

# ── State ────────────────────────────────────────────
var _entity_id: int = -1
var _l2_data: Dictionary = {}
var _l3_mind: Dictionary = {}
var _l3_body: Dictionary = {}
var _l3_skills: Dictionary = {}
var _l3_social: Dictionary = {}
var _l3_memory: Dictionary = {}
var _l3_misc: Dictionary = {}
var _l3_loaded: bool = false
var _scroll_offset: float = 0.0
var _content_height: float = 0.0
var _sim_bridge: Object = null
var _section_rects: Dictionary = {}

var _collapsed: Dictionary = {
	"needs": false,
	"personality": false,
	"personality_detail": true,
	"emotions": false,
	"stress": false,
	"stress_detail": true,
	"traits": false,
	"body": false,
	"body_detail": true,
	"intelligence": false,
	"intelligence_detail": true,
	"skills": false,
	"values": false,
	"values_detail": true,
	"relationships": false,
	"memories": true,
	"economy": true,
	"derived": true,
	"faith": true,
	"flavor": true,
	"action_log": true,
}


## Stores the SimBridge reference for entity data queries.
func init(sim_bridge: Object) -> void:
	_sim_bridge = sim_bridge


## Shows entity data whether alive or deceased (delegates to set_entity_id).
func show_entity_or_deceased(entity_id: int) -> void:
	set_entity_id(entity_id)


## Sets the entity to display and reloads L2 data.
func set_entity_id(entity_id: int) -> void:
	_entity_id = entity_id
	_l2_data = {}
	_l3_mind = {}
	_l3_body = {}
	_l3_skills = {}
	_l3_social = {}
	_l3_memory = {}
	_l3_misc = {}
	_l3_loaded = false
	_scroll_offset = 0.0
	_section_rects = {}

	if _sim_bridge != null and _sim_bridge.has_method("runtime_get_entity_detail"):
		_l2_data = _sim_bridge.runtime_get_entity_detail(entity_id)

	queue_redraw()


## Loads all 6 L3 tabs at once (~3KB total).
func _load_l3_data() -> void:
	if _sim_bridge == null or _entity_id < 0:
		return
	if not _sim_bridge.has_method("runtime_get_entity_tab"):
		return
	_l3_mind = _sim_bridge.runtime_get_entity_tab(_entity_id, "mind")
	_l3_body = _sim_bridge.runtime_get_entity_tab(_entity_id, "body")
	_l3_skills = _sim_bridge.runtime_get_entity_tab(_entity_id, "skills")
	_l3_social = _sim_bridge.runtime_get_entity_tab(_entity_id, "social")
	_l3_memory = _sim_bridge.runtime_get_entity_tab(_entity_id, "memory")
	_l3_misc = _sim_bridge.runtime_get_entity_tab(_entity_id, "misc")
	_l3_loaded = true


func _process(_delta: float) -> void:
	if visible and not _l2_data.is_empty():
		queue_redraw()


# ── Master Draw ──────────────────────────────────────

func _draw() -> void:
	if _l2_data.is_empty():
		return

	var font: Font = get_theme_default_font()

	draw_rect(Rect2(Vector2.ZERO, size), CLR_BG)

	var x: float = MARGIN_LEFT
	var y: float = -_scroll_offset + 8.0

	y = _draw_header(font, x, y)
	y = _draw_causal_summary(font, x, y)
	y = _draw_needs(font, x, y)
	y = _draw_personality(font, x, y)
	y = _draw_emotions(font, x, y)
	y = _draw_stress(font, x, y)
	y = _draw_traits(font, x, y)
	y = _draw_body(font, x, y)
	y = _draw_intelligence(font, x, y)
	y = _draw_skills(font, x, y)
	y = _draw_values(font, x, y)
	y = _draw_relationships(font, x, y)
	y = _draw_memories(font, x, y)
	y = _draw_economy(font, x, y)
	y = _draw_derived_stats(font, x, y)
	y = _draw_faith(font, x, y)
	y = _draw_flavor(font, x, y)
	y = _draw_action_log(font, x, y)

	_content_height = y + _scroll_offset + 20.0
	_draw_scrollbar()


# ── Section 1: Header ────────────────────────────────

func _draw_header(font: Font, x: float, y: float) -> float:
	var sex_str: String = _l2_data.get("sex", "")
	var sex_icon: String = "♂" if sex_str == "Male" else "♀"
	var entity_name: String = _l2_data.get("name", "???")
	var age: int = int(_l2_data.get("age_years", 0))
	var stage_raw: String = str(_l2_data.get("growth_stage", "Adult")).to_upper()
	var stage: String = Locale.ltr("STAGE_" + stage_raw)
	var occ_raw: String = str(_l2_data.get("occupation", "NONE")).to_upper()
	var job: String = Locale.ltr("OCCUPATION_" + occ_raw)

	var header_text: String = "%s %s    %d%s %s    %s" % [
		sex_icon, entity_name, age, Locale.ltr("UI_AGE_UNIT"), stage, job
	]
	draw_string(font, Vector2(x, y + 13.0), header_text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_HEADER, CLR_NAME)

	var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	draw_line(Vector2(x, y + LINE_HEIGHT + 2.0),
		Vector2(x + panel_w, y + LINE_HEIGHT + 2.0), CLR_SECTION_LINE)
	return y + LINE_HEIGHT + SECTION_GAP


# ── Section 2: Causal Summary ────────────────────────

func _draw_causal_summary(font: Font, x: float, y: float) -> float:
	var result: Array = _generate_causal_summary(_l2_data)
	draw_string(font, Vector2(x, y + 11.0), str(result[0]),
		HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_NARRATIVE, result[1])
	return y + LINE_HEIGHT_SMALL + 6.0


func _generate_causal_summary(l2: Dictionary) -> Array:
	var active_break: String = str(l2.get("active_break", ""))
	if active_break != "" and active_break != "none":
		var break_name: String = Locale.ltr("MENTAL_BREAK_TYPE_" + active_break.to_upper())
		return [Locale.ltr("UI_SUMMARY_MENTAL_BREAK").replace("{type}", break_name), CLR_CRISIS]

	var stress_state: String = str(l2.get("stress_state", "Calm"))
	if stress_state == "Exhaustion" or stress_state == "Collapse":
		return [Locale.ltr("UI_SUMMARY_STRESS_EXHAUSTION"), CLR_DANGER]

	var health: float = float(l2.get("health", 1.0))
	if health < 0.3:
		return [Locale.ltr("UI_SUMMARY_HEALTH_CRITICAL"), CLR_DANGER]

	var need_checks := [
		["need_hunger", "NEED_HUNGER"], ["need_thirst", "NEED_THIRST"],
		["need_warmth", "NEED_WARMTH"], ["need_safety", "NEED_SAFETY"],
		["energy", "NEED_ENERGY"],
	]
	for nc in need_checks:
		var val: float = float(l2.get(nc[0], 1.0))
		if val < 0.15:
			var need_name: String = Locale.ltr(nc[1])
			return [Locale.ltr("UI_SUMMARY_NEED_CRITICAL").replace("{need}", need_name), CLR_WARN]

	if stress_state == "Resistance":
		return [Locale.ltr("UI_SUMMARY_STRESS_RESISTANCE"), CLR_WARN]

	var dom_emo: String = str(l2.get("dominant_emotion", ""))
	if dom_emo != "":
		var emo_keys := {
			"joy": "emo_joy", "trust": "emo_trust", "fear": "emo_fear",
			"surprise": "emo_surprise", "sadness": "emo_sadness",
			"disgust": "emo_disgust", "anger": "emo_anger", "anticipation": "emo_anticipation"
		}
		var intensity_key: String = emo_keys.get(dom_emo, "")
		if intensity_key != "":
			var intensity: float = float(l2.get(intensity_key, 0.0))
			if intensity > 0.7:
				var emo_name: String = Locale.ltr("STRESS_EMO_" + dom_emo.to_upper())
				return [Locale.ltr("UI_SUMMARY_EMOTION_STRONG").replace("{emotion}", emo_name), CLR_WARN]

	var action: String = str(l2.get("current_action", "Idle"))
	var action_name: String = Locale.ltr("ACTION_" + action.to_upper())
	return [Locale.ltr("UI_SUMMARY_ACTION_DEFAULT").replace("{action}", action_name), CLR_TEXT]


# ── Section 3: Needs ─────────────────────────────────

func _draw_needs(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_NEEDS"), "needs")
	if _is_collapsed("needs"):
		return y

	var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	var need_map: Array = [
		["need_hunger", "NEED_HUNGER"],
		["need_thirst", "NEED_THIRST"],
		["need_sleep", "NEED_SLEEP"],
		["need_warmth", "NEED_WARMTH"],
		["energy", "NEED_ENERGY"],
		["need_safety", "NEED_SAFETY"],
		["need_belonging", "NEED_BELONGING"],
		["need_intimacy", "NEED_INTIMACY"],
		["need_recognition", "NEED_RECOGNITION"],
		["need_autonomy", "NEED_AUTONOMY"],
		["need_competence", "NEED_COMPETENCE"],
		["need_self_actualization", "NEED_SELF_ACTUALIZATION"],
		["need_meaning", "NEED_MEANING"],
		["need_transcendence", "NEED_TRANSCENDENCE"],
	]

	for entry in need_map:
		var val: float = float(_l2_data.get(entry[0], 1.0))
		var label: String = Locale.ltr(entry[1])
		var bar_color: Color = CLR_BAR_FILL
		var suffix: String = ""
		if val < 0.15:
			bar_color = CLR_DANGER
			suffix = Locale.ltr("UI_NEED_CRITICAL")
		elif val < 0.30:
			bar_color = CLR_BAR_DANGER
			suffix = Locale.ltr("UI_NEED_LOW")
		y = _draw_bar(font, x, y, panel_w, label, val, bar_color, suffix)

	return y + SECTION_GAP


# ── Section 4: Personality ───────────────────────────

func _draw_personality(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_PERSONALITY"), "personality")
	if _is_collapsed("personality"):
		return y

	var narrative: String = _generate_personality_narrative(_l2_data)
	y = _draw_wrapped_text(font, x, y, narrative, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
	y += 4.0

	# Detail toggle (facet bars)
	y = _draw_section_header(font, x + 8.0, y, Locale.ltr("UI_SUBSECTION_FACETS"), "personality_detail")
	if not _is_collapsed("personality_detail"):
		if not _l3_loaded:
			_load_l3_data()
		var facets: PackedFloat32Array = _l3_mind.get("facets", PackedFloat32Array())
		var facet_keys: Array = [
			"FACET_H_SINCERITY", "FACET_H_FAIRNESS", "FACET_H_GREED_AVOIDANCE", "FACET_H_MODESTY",
			"FACET_E_FEARFULNESS", "FACET_E_ANXIETY", "FACET_E_DEPENDENCE", "FACET_E_SENTIMENTALITY",
			"FACET_X_SOCIAL_SELF_ESTEEM", "FACET_X_SOCIAL_BOLDNESS", "FACET_X_SOCIABILITY", "FACET_X_LIVELINESS",
			"FACET_A_FORGIVENESS", "FACET_A_GENTLENESS", "FACET_A_FLEXIBILITY", "FACET_A_PATIENCE",
			"FACET_C_ORGANIZATION", "FACET_C_DILIGENCE", "FACET_C_PERFECTIONISM", "FACET_C_PRUDENCE",
			"FACET_O_AESTHETIC_APPRECIATION", "FACET_O_INQUISITIVENESS", "FACET_O_CREATIVITY", "FACET_O_UNCONVENTIONALITY",
		]
		var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
		for i in range(mini(facets.size(), facet_keys.size())):
			var label: String = Locale.ltr(facet_keys[i])
			y = _draw_bar(font, x + 8.0, y, panel_w - 8.0, label, facets[i], CLR_BAR_FILL)

	return y + SECTION_GAP


func _generate_personality_narrative(l2: Dictionary) -> String:
	var sex: String = str(l2.get("sex", "Male"))
	var pronoun: String = Locale.ltr("UI_PRONOUN_HE") if sex == "Male" else Locale.ltr("UI_PRONOUN_SHE")

	var parts: Array = []
	var axes: Array = [
		{"key": "hex_h", "high": "UI_HEXACO_H_HIGH", "low": "UI_HEXACO_H_LOW"},
		{"key": "hex_e", "high": "UI_HEXACO_E_HIGH", "low": "UI_HEXACO_E_LOW"},
		{"key": "hex_x", "high": "UI_HEXACO_X_HIGH", "low": "UI_HEXACO_X_LOW"},
		{"key": "hex_a", "high": "UI_HEXACO_A_HIGH", "low": "UI_HEXACO_A_LOW"},
		{"key": "hex_c", "high": "UI_HEXACO_C_HIGH", "low": "UI_HEXACO_C_LOW"},
		{"key": "hex_o", "high": "UI_HEXACO_O_HIGH", "low": "UI_HEXACO_O_LOW"},
	]

	for axis in axes:
		var val: float = float(l2.get(axis.key, 0.5))
		var intensity_key: String = _get_intensity_key(val)
		if intensity_key == "":
			continue
		var desc_key: String = axis.high if val > 0.55 else axis.low
		parts.append(Locale.ltr(intensity_key) + " " + Locale.ltr(desc_key))

	if parts.is_empty():
		return pronoun + " " + Locale.ltr("UI_PERSONALITY_AVERAGE")

	var sentences: Array = []
	var current: String = pronoun + " "
	for i in range(parts.size()):
		if i == 0:
			current += parts[i]
		elif i % 2 == 0:
			sentences.append(current + ".")
			current = Locale.ltr("UI_ALSO") + " " + parts[i]
		else:
			current += ", " + parts[i]
	sentences.append(current + ".")
	return " ".join(sentences)


# ── Section 5: Emotions ──────────────────────────────

func _draw_emotions(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_EMOTIONS"), "emotions")
	if _is_collapsed("emotions"):
		return y

	var narrative: String = _generate_emotion_narrative(_l2_data)
	y = _draw_wrapped_text(font, x, y, narrative, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
	y += 4.0

	var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	var emo_map: Array = [
		["emo_joy", "STRESS_EMO_JOY"],
		["emo_trust", "STRESS_EMO_TRUST"],
		["emo_fear", "STRESS_EMO_FEAR"],
		["emo_surprise", "STRESS_EMO_SURPRISE"],
		["emo_sadness", "STRESS_EMO_SADNESS"],
		["emo_disgust", "STRESS_EMO_DISGUST"],
		["emo_anger", "STRESS_EMO_ANGER"],
		["emo_anticipation", "STRESS_EMO_ANTICIPATION"],
	]
	for entry in emo_map:
		var val: float = float(_l2_data.get(entry[0], 0.0))
		var label: String = Locale.ltr(entry[1])
		var bar_color: Color = CLR_BAR_FILL
		if entry[0] in ["emo_fear", "emo_anger", "emo_disgust"]:
			bar_color = CLR_BAR_DANGER if val > 0.6 else CLR_BAR_FILL
		y = _draw_bar(font, x, y, panel_w, label, val, bar_color)

	return y + SECTION_GAP


func _generate_emotion_narrative(l2: Dictionary) -> String:
	var dom_emo: String = str(l2.get("dominant_emotion", ""))
	if dom_emo == "":
		return Locale.ltr("UI_EMO_DESC_CALM")

	var emo_key: String = "emo_" + dom_emo
	var intensity: float = float(l2.get(emo_key, 0.0))
	var emo_name: String = Locale.ltr("STRESS_EMO_" + dom_emo.to_upper())

	if intensity > 0.65:
		return Locale.ltr("UI_EMO_DESC_STRONG").replace("{emotion}", emo_name)
	elif intensity > 0.35:
		return Locale.ltr("UI_EMO_DESC_MODERATE").replace("{emotion}", emo_name)
	return Locale.ltr("UI_EMO_DESC_CALM")


# ── Section 6: Stress ────────────────────────────────

func _draw_stress(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_STRESS"), "stress")
	if _is_collapsed("stress"):
		return y

	var narrative: String = _generate_stress_narrative(_l2_data)
	y = _draw_wrapped_text(font, x, y, narrative, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
	y += 4.0

	var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	var stress_level: float = float(_l2_data.get("stress_level", 0.0))
	var stress_color: Color = CLR_BAR_FILL
	var stress_state: String = str(_l2_data.get("stress_state", "Calm"))
	if stress_state in ["Exhaustion", "Collapse"]:
		stress_color = CLR_DANGER
	elif stress_state == "Resistance":
		stress_color = CLR_BAR_DANGER
	y = _draw_bar(font, x, y, panel_w, Locale.ltr("UI_SECTION_STRESS"), stress_level, stress_color)
	y = _draw_bar(font, x, y, panel_w, Locale.ltr("UI_STRESS_RESERVE"), float(_l2_data.get("stress_reserve", 1.0)), CLR_GOOD)
	y = _draw_bar(font, x, y, panel_w, Locale.ltr("UI_STRESS_RESILIENCE"), float(_l2_data.get("resilience", 0.5)), CLR_BAR_FILL)

	# Stressor list (collapsed by default)
	y = _draw_section_header(font, x + 8.0, y, Locale.ltr("UI_SUBSECTION_STRESSORS"), "stress_detail")
	if not _is_collapsed("stress_detail"):
		if not _l3_loaded:
			_load_l3_data()
		var stressors: Array = _l3_mind.get("stressors", [])
		if stressors.is_empty():
			draw_string(font, Vector2(x + 8.0, y + 11.0), "-",
				HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, CLR_DIM)
			y += LINE_HEIGHT_SMALL
		else:
			for s in stressors:
				var src: String = str(s.get("source_id", "?"))
				var per_tick: float = float(s.get("per_tick", 0.0))
				var line: String = "%s  +%.3f/tick" % [src, per_tick]
				draw_string(font, Vector2(x + 8.0, y + 11.0), line,
					HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, CLR_DANGER)
				y += LINE_HEIGHT_SMALL

	return y + SECTION_GAP


func _generate_stress_narrative(l2: Dictionary) -> String:
	var state: String = str(l2.get("stress_state", "Calm"))
	match state:
		"Calm": return Locale.ltr("UI_STRESS_DESC_CALM")
		"Alert": return Locale.ltr("UI_STRESS_DESC_ALERT")
		"Resistance": return Locale.ltr("UI_STRESS_DESC_RESISTANCE")
		"Exhaustion": return Locale.ltr("UI_STRESS_DESC_EXHAUSTION")
		"Collapse": return Locale.ltr("UI_STRESS_DESC_COLLAPSE")
	return Locale.ltr("UI_STRESS_DESC_CALM")


# ── Section 7: Traits ────────────────────────────────

func _draw_traits(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_TRAITS"), "traits")
	if _is_collapsed("traits"):
		return y

	var traits: PackedStringArray = _l2_data.get("active_traits", PackedStringArray())
	if traits.is_empty():
		draw_string(font, Vector2(x, y + 11.0), "-",
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_NARRATIVE, CLR_DIM)
		return y + LINE_HEIGHT_SMALL + SECTION_GAP

	# DF-style badge row: "근면 · 친절 · 불안"
	var _panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	var parts: PackedStringArray = PackedStringArray()
	for t in traits:
		var tkey: String = "TRAIT_" + str(t).to_upper()
		var tname: String = Locale.ltr(tkey) if tkey != t.to_upper() else str(t)
		parts.append(tname)
	var badge_line: String = " · ".join(parts)
	y = _draw_wrapped_text(font, x, y, badge_line, CLR_TEXT, FONT_SIZE_BODY)
	return y + SECTION_GAP


# ── Section 8: Body ──────────────────────────────────

func _draw_body(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_BODY"), "body")
	if _is_collapsed("body"):
		return y

	var narrative: String = _generate_body_narrative(_l2_data)
	y = _draw_wrapped_text(font, x, y, narrative, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
	y += 4.0

	var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	var health: float = float(_l2_data.get("health", 1.0))
	var health_color: Color = CLR_GOOD if health > 0.6 else (CLR_WARN if health > 0.3 else CLR_DANGER)
	y = _draw_bar(font, x, y, panel_w, Locale.ltr("UI_BODY_HEALTH"), health, health_color)

	y = _draw_section_header(font, x + 8.0, y, Locale.ltr("UI_SUBSECTION_ATTRIBUTES"), "body_detail")
	if not _is_collapsed("body_detail"):
		var axes_l2: Array = [
			["body_str", "UI_BODY_STR"],
			["body_agi", "UI_BODY_AGI"],
			["body_end", "UI_BODY_END"],
			["body_tou", "UI_BODY_TOU"],
			["body_rec", "UI_BODY_REC"],
			["body_dr", "UI_BODY_DR"],
		]
		for entry in axes_l2:
			var val: float = float(_l2_data.get(entry[0], 0.5))
			y = _draw_bar(font, x + 8.0, y, panel_w - 8.0, Locale.ltr(entry[1]), val, CLR_BAR_FILL)

	return y + SECTION_GAP


func _generate_body_narrative(l2: Dictionary) -> String:
	var health: float = float(l2.get("health", 1.0))
	if health < 0.2:
		return Locale.ltr("UI_BODY_DESC_HEALTH_CRITICAL")

	var parts: Array = []
	var str_val: float = float(l2.get("body_str", 0.5))
	var end_val: float = float(l2.get("body_end", 0.5))
	var agi_val: float = float(l2.get("body_agi", 0.5))
	var tou_val: float = float(l2.get("body_tou", 0.5))
	var attr: float = float(l2.get("attractiveness", 0.5))
	var height: float = float(l2.get("height", 170.0))

	if str_val > 0.65 and end_val > 0.65:
		parts.append(Locale.ltr("UI_BODY_DESC_STRONG"))
	elif str_val < 0.35 and end_val < 0.35:
		parts.append(Locale.ltr("UI_BODY_DESC_FRAIL"))
	if agi_val > 0.65:
		parts.append(Locale.ltr("UI_BODY_DESC_AGILE"))
	if tou_val > 0.65:
		parts.append(Locale.ltr("UI_BODY_DESC_TOUGH"))
	if attr > 0.70:
		parts.append(Locale.ltr("UI_BODY_DESC_ATTRACTIVE"))
	if height > 185.0:
		parts.append(Locale.ltr("UI_BODY_DESC_TALL"))
	elif height < 158.0:
		parts.append(Locale.ltr("UI_BODY_DESC_SHORT"))

	if parts.is_empty():
		return "%.0fcm" % height
	return " ".join(parts)


# ── Section 9: Intelligence ──────────────────────────

func _draw_intelligence(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_INTELLIGENCE"), "intelligence")
	if _is_collapsed("intelligence"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var narrative: String = _generate_intelligence_narrative(_l3_body)
	y = _draw_wrapped_text(font, x, y, narrative, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
	y += 4.0

	y = _draw_section_header(font, x + 8.0, y, Locale.ltr("UI_SUBSECTION_DETAIL"), "intelligence_detail")
	if not _is_collapsed("intelligence_detail"):
		var intel_arr: PackedFloat32Array = _l3_body.get("intelligence", PackedFloat32Array())
		var g_factor: float = float(_l3_body.get("g_factor", 0.5))
		var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
		y = _draw_bar(font, x + 8.0, y, panel_w - 8.0, Locale.ltr("UI_INTEL_GENERAL"), g_factor, CLR_BAR_FILL)
		var intel_names: Array = [
			"UI_INTEL_LINGUISTIC", "UI_INTEL_LOGICAL", "UI_INTEL_SPATIAL",
			"UI_INTEL_KINESTHETIC", "UI_INTEL_MUSICAL", "UI_INTEL_INTERPERSONAL",
			"UI_INTEL_INTRAPERSONAL", "UI_INTEL_NATURALISTIC",
		]
		for i in range(mini(intel_arr.size(), intel_names.size())):
			y = _draw_bar(font, x + 8.0, y, panel_w - 8.0,
				Locale.ltr(intel_names[i]), intel_arr[i], CLR_BAR_FILL)

	return y + SECTION_GAP


func _generate_intelligence_narrative(l3_body: Dictionary) -> String:
	var g_factor: float = float(l3_body.get("g_factor", 0.5))
	if g_factor > 0.70:
		return Locale.ltr("UI_INTEL_DESC_BRIGHT")
	var intel_arr: PackedFloat32Array = l3_body.get("intelligence", PackedFloat32Array())
	var intel_names: Array = [
		"UI_INTEL_LINGUISTIC", "UI_INTEL_LOGICAL", "UI_INTEL_SPATIAL",
		"UI_INTEL_KINESTHETIC", "UI_INTEL_MUSICAL", "UI_INTEL_INTERPERSONAL",
		"UI_INTEL_INTRAPERSONAL", "UI_INTEL_NATURALISTIC",
	]
	var best_i: int = 0
	var best_val: float = 0.0
	var worst_i: int = 0
	var worst_val: float = 1.0
	for i in range(mini(intel_arr.size(), intel_names.size())):
		if intel_arr[i] > best_val:
			best_val = intel_arr[i]
			best_i = i
		if intel_arr[i] < worst_val:
			worst_val = intel_arr[i]
			worst_i = i

	if best_val > 0.70 and best_i < intel_names.size():
		var intel_name: String = Locale.ltr(intel_names[best_i])
		return Locale.ltr("UI_INTEL_DESC_GIFTED").replace("{intel}", intel_name)
	if worst_val < 0.30 and worst_i < intel_names.size():
		var intel_name: String = Locale.ltr(intel_names[worst_i])
		return Locale.ltr("UI_INTEL_DESC_WEAK").replace("{intel}", intel_name)
	return Locale.ltr("UI_INTEL_DESC_BRIGHT")


# ── Section 10: Skills ───────────────────────────────

func _draw_skills(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_SKILLS"), "skills")
	if _is_collapsed("skills"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var skills: Array = _l3_skills.get("skills", [])
	if skills.is_empty():
		draw_string(font, Vector2(x, y + 11.0), Locale.ltr("UI_SKILLS_NONE"),
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_NARRATIVE, CLR_DIM)
		return y + LINE_HEIGHT_SMALL + SECTION_GAP

	var visible_skills: Array = []
	for s in skills:
		if int(s.get("level", 0)) >= 3:
			visible_skills.append(s)
	visible_skills.sort_custom(func(a, b): return int(a.get("level", 0)) > int(b.get("level", 0)))

	var skill_parts: Array = []
	for s in visible_skills:
		var level: int = int(s.get("level", 0))
		var rank: String = Locale.ltr(_skill_rank_key(level))
		var skill_id: String = str(s.get("id", ""))
		var skill_name: String = Locale.ltr("UI_SKILL_" + skill_id.to_upper())
		skill_parts.append("%s %s(%d)" % [rank, skill_name, level])

	var line: String = ", ".join(skill_parts) if not skill_parts.is_empty() else Locale.ltr("UI_SKILLS_UNSKILLED")
	y = _draw_wrapped_text(font, x, y, line, CLR_TEXT, FONT_SIZE_BODY)
	return y + SECTION_GAP


static func _skill_rank_key(level: int) -> String:
	if level >= 30: return "UI_SKILL_RANK_LEGENDARY"
	if level >= 20: return "UI_SKILL_RANK_MASTER"
	if level >= 15: return "UI_SKILL_RANK_EXPERT"
	if level >= 10: return "UI_SKILL_RANK_SKILLED"
	if level >= 7:  return "UI_SKILL_RANK_COMPETENT"
	if level >= 4:  return "UI_SKILL_RANK_ADEQUATE"
	if level >= 1:  return "UI_SKILL_RANK_NOVICE"
	return "UI_SKILL_RANK_DABBLING"


# ── Section 11: Values ───────────────────────────────

func _draw_values(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_VALUES"), "values")
	if _is_collapsed("values"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var narrative: String = _generate_values_narrative(_l3_mind)
	y = _draw_wrapped_text(font, x, y, narrative, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
	y += 4.0

	y = _draw_section_header(font, x + 8.0, y, Locale.ltr("UI_SUBSECTION_ALL_VALUES"), "values_detail")
	if not _is_collapsed("values_detail"):
		var values_all: PackedFloat32Array = _l3_mind.get("values_all", PackedFloat32Array())
		var value_names: Array = [
			"VALUE_LAW", "VALUE_LOYALTY", "VALUE_FAMILY", "VALUE_FRIENDSHIP", "VALUE_POWER",
			"VALUE_TRUTH", "VALUE_CUNNING", "VALUE_ELOQUENCE", "VALUE_FAIRNESS", "VALUE_DECORUM",
			"VALUE_TRADITION", "VALUE_ARTWORK", "VALUE_COOPERATION", "VALUE_INDEPENDENCE",
			"VALUE_STOICISM", "VALUE_INTROSPECTION", "VALUE_SELF_CONTROL", "VALUE_TRANQUILITY",
			"VALUE_HARMONY", "VALUE_MERRIMENT", "VALUE_CRAFTSMANSHIP", "VALUE_MARTIAL_PROWESS",
			"VALUE_SKILL", "VALUE_HARD_WORK", "VALUE_SACRIFICE", "VALUE_COMPETITION",
			"VALUE_PERSEVERANCE", "VALUE_LEISURE", "VALUE_COMMERCE", "VALUE_ROMANCE",
			"VALUE_KNOWLEDGE", "VALUE_NATURE", "VALUE_PEACE",
		]
		var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
		for i in range(mini(values_all.size(), value_names.size())):
			var label: String = Locale.ltr(value_names[i])
			y = _draw_bipolar_bar(font, x + 8.0, y, panel_w - 8.0, label,
				values_all[i], CLR_VALUE_POS, CLR_VALUE_NEG)

	return y + SECTION_GAP


func _generate_values_narrative(l3_mind: Dictionary) -> String:
	var values_all: PackedFloat32Array = l3_mind.get("values_all", PackedFloat32Array())
	if values_all.is_empty():
		return Locale.ltr("UI_VALUES_UNKNOWN")

	var value_names: Array = [
		"LAW", "LOYALTY", "FAMILY", "FRIENDSHIP", "POWER", "TRUTH", "CUNNING",
		"ELOQUENCE", "FAIRNESS", "DECORUM", "TRADITION", "ARTWORK", "COOPERATION",
		"INDEPENDENCE", "STOICISM", "INTROSPECTION", "SELF_CONTROL", "TRANQUILITY",
		"HARMONY", "MERRIMENT", "CRAFTSMANSHIP", "MARTIAL_PROWESS", "SKILL",
		"HARD_WORK", "SACRIFICE", "COMPETITION", "PERSEVERANCE", "LEISURE",
		"COMMERCE", "ROMANCE", "KNOWLEDGE", "NATURE", "PEACE",
	]
	var pairs: Array = []
	for i in range(mini(values_all.size(), 33)):
		pairs.append({"name": value_names[i], "val": float(values_all[i])})
	pairs.sort_custom(func(a, b): return absf(a.val) > absf(b.val))

	var sex: String = str(_l2_data.get("sex", "Male"))
	var pronoun: String = Locale.ltr("UI_PRONOUN_HE") if sex == "Male" else Locale.ltr("UI_PRONOUN_SHE")

	var loved: Array = []
	var despised: Array = []
	for p in pairs:
		if float(p.val) > 0.3 and loved.size() < 3:
			loved.append(Locale.ltr("VALUE_" + p.name))
		elif float(p.val) < -0.3 and despised.size() < 3:
			despised.append(Locale.ltr("VALUE_" + p.name))

	var result: String = ""
	if not loved.is_empty():
		result += pronoun + " " + Locale.ltr("UI_VALUES_CHERISH").replace("{values}", Locale.ltr("UI_AND").join(loved))
	if not despised.is_empty():
		if result != "":
			result += " "
		result += Locale.ltr("UI_VALUES_DESPISE").replace("{values}", Locale.ltr("UI_AND").join(despised))
	if result == "":
		result = pronoun + " " + Locale.ltr("UI_VALUES_BALANCED")
	return result


# ── Section 12: Relationships ────────────────────────

func _draw_relationships(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_RELATIONSHIPS"), "relationships")
	if _is_collapsed("relationships"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var rels: Array = _l3_social.get("relationships", [])
	if rels.is_empty():
		draw_string(font, Vector2(x, y + 11.0), Locale.ltr("UI_RELATIONSHIP_NONE"),
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_NARRATIVE, CLR_DIM)
		y += LINE_HEIGHT_SMALL
	else:
		for rel in rels:
			var target_id: int = int(rel.get("target_id", -1))
			var affinity: float = float(rel.get("affinity", 0.0))
			var rel_type: String = str(rel.get("relation_type", ""))
			var rel_name: String = Locale.ltr("UI_RELATIONSHIP_" + rel_type.to_upper())
			var line: String = "%s #%d  aff:%.2f" % [rel_name, target_id, affinity]
			var rel_color: Color = CLR_GOOD if affinity > 0.3 else (CLR_DANGER if affinity < -0.3 else CLR_TEXT)
			draw_string(font, Vector2(x, y + 11.0), line,
				HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, rel_color)
			y += LINE_HEIGHT_SMALL

	return y + SECTION_GAP


# ── Section 13: Memories ─────────────────────────────

func _draw_memories(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_MEMORIES"), "memories")
	if _is_collapsed("memories"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var recent: Array = _l3_memory.get("recent_memories", [])
	if recent.is_empty():
		draw_string(font, Vector2(x, y + 11.0), Locale.ltr("UI_MEMORY_NONE"),
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_NARRATIVE, CLR_DIM)
		y += LINE_HEIGHT_SMALL
	else:
		for mem in recent.slice(0, 10):
			var evt: String = str(mem.get("event_type", "?"))
			var intensity: float = float(mem.get("intensity", 0.0))
			var is_perm: bool = bool(mem.get("is_permanent", false))
			var marker: String = "★" if is_perm else "·"
			var line: String = "%s %s  i:%.2f" % [marker, evt, intensity]
			var mem_color: Color = CLR_WARN if intensity > 0.6 else CLR_TEXT
			draw_string(font, Vector2(x, y + 11.0), line,
				HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, mem_color)
			y += LINE_HEIGHT_SMALL

	return y + SECTION_GAP


# ── Section 14: Economy ──────────────────────────────

func _draw_economy(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_ECONOMY"), "economy")
	if _is_collapsed("economy"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var narrative: String = _generate_economy_narrative(_l3_social)
	y = _draw_wrapped_text(font, x, y, narrative, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
	y += 4.0

	var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	var wealth: float = float(_l3_social.get("wealth", 0.0))
	draw_string(font, Vector2(x, y + 11.0), Locale.ltr("UI_ECON_WEALTH") + ": %.1f" % wealth,
		HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_BODY, CLR_TEXT)
	y += LINE_HEIGHT
	y = _draw_bar(font, x, y, panel_w, Locale.ltr("UI_ECON_SAVING"), float(_l3_social.get("saving_tendency", 0.5)), CLR_BAR_FILL)
	y = _draw_bar(font, x, y, panel_w, Locale.ltr("UI_ECON_GENEROSITY"), float(_l3_social.get("generosity", 0.5)), CLR_BAR_FILL)

	return y + SECTION_GAP


func _generate_economy_narrative(l3_social: Dictionary) -> String:
	var saving: float = float(l3_social.get("saving_tendency", 0.5))
	var generosity: float = float(l3_social.get("generosity", 0.5))
	var materialism: float = float(l3_social.get("materialism", 0.5))
	var risk: float = float(l3_social.get("risk_appetite", 0.5))

	var traits_econ: Array = []
	if saving > 0.65:
		traits_econ.append(Locale.ltr("UI_ECON_DESC_FRUGAL"))
	if generosity > 0.65:
		traits_econ.append(Locale.ltr("UI_ECON_DESC_GENEROUS"))
	if materialism > 0.65:
		traits_econ.append(Locale.ltr("UI_ECON_DESC_MATERIALISTIC"))
	if risk < 0.35:
		traits_econ.append(Locale.ltr("UI_ECON_DESC_RISK_AVERSE"))
	elif risk > 0.65:
		traits_econ.append(Locale.ltr("UI_ECON_DESC_RISK_SEEKING"))

	if traits_econ.is_empty():
		return Locale.ltr("UI_ECON_BALANCED")
	return ", ".join(traits_econ) + "."


# ── Section 15: Derived Stats ────────────────────────

func _draw_derived_stats(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_DERIVED"), "derived")
	if _is_collapsed("derived"):
		return y

	# Placeholder — DerivedStatsSystem not yet implemented (Phase D3)
	draw_string(font, Vector2(x, y + 11.0), "(derived stats: Phase D3)",
		HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, CLR_DIM)
	y += LINE_HEIGHT_SMALL
	return y + SECTION_GAP


# ── Section 16: Faith ────────────────────────────────

func _draw_faith(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_FAITH"), "faith")
	if _is_collapsed("faith"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var tradition: String = str(_l3_misc.get("faith_tradition", ""))
	var strength: float = float(_l3_misc.get("faith_strength", 0.0))
	var ritual_count: int = int(_l3_misc.get("ritual_count", 0))

	if tradition == "" or strength < 0.05:
		draw_string(font, Vector2(x, y + 11.0), "-",
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_NARRATIVE, CLR_DIM)
		y += LINE_HEIGHT_SMALL
	else:
		var strength_key: String
		if strength > 0.65:
			strength_key = "UI_FAITH_STRENGTH_STRONG"
		elif strength > 0.35:
			strength_key = "UI_FAITH_STRENGTH_MODERATE"
		else:
			strength_key = "UI_FAITH_STRENGTH_WEAK"
		var faith_text: String = Locale.ltr("UI_FAITH_DESC") \
			.replace("{tradition}", tradition) \
			.replace("{strength}", Locale.ltr(strength_key))
		y = _draw_wrapped_text(font, x, y, faith_text, CLR_NARRATIVE, FONT_SIZE_NARRATIVE)
		if ritual_count > 0:
			var ritual_text: String = Locale.ltr("UI_FAITH_RITUAL_COUNT") \
				.replace("{count}", str(ritual_count))
			draw_string(font, Vector2(x, y + 11.0), ritual_text,
				HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, CLR_DIM)
			y += LINE_HEIGHT_SMALL

	return y + SECTION_GAP


# ── Section 17: Flavor ───────────────────────────────

func _draw_flavor(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_FLAVOR"), "flavor")
	if _is_collapsed("flavor"):
		return y

	if not _l3_loaded:
		_load_l3_data()

	var zodiac: String = str(_l2_data.get("zodiac", ""))
	var blood_type: String = str(_l2_data.get("blood_type", ""))
	var speech_tone: String = str(_l2_data.get("speech_tone", ""))

	if zodiac != "":
		var zkey: String = "ZODIAC_" + zodiac.to_upper()
		draw_string(font, Vector2(x, y + 11.0),
			Locale.ltr(zkey) + "  " + blood_type,
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, CLR_DIM)
		y += LINE_HEIGHT_SMALL

	if speech_tone != "":
		var tone_key: String = "SPEECH_" + speech_tone.to_upper()
		draw_string(font, Vector2(x, y + 11.0), Locale.ltr(tone_key),
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, CLR_DIM)
		y += LINE_HEIGHT_SMALL

	return y + SECTION_GAP


# ── Section 18: Action Log ───────────────────────────

func _draw_action_log(font: Font, x: float, y: float) -> float:
	y = _draw_section_header(font, x, y, Locale.ltr("UI_SECTION_ACTION_LOG"), "action_log")
	if _is_collapsed("action_log"):
		return y

	var action: String = str(_l2_data.get("current_action", "Idle"))
	var progress: float = float(_l2_data.get("action_progress", 0.0))
	var panel_w: float = size.x - MARGIN_LEFT - MARGIN_RIGHT
	var action_name: String = Locale.ltr("ACTION_" + action.to_upper())
	y = _draw_bar(font, x, y, panel_w, action_name, progress, CLR_BAR_FILL)

	# ExplainLog (stub — Phase D4)
	var explains: PackedStringArray = _l2_data.get("recent_explains", PackedStringArray())
	for ex in explains:
		draw_string(font, Vector2(x, y + 11.0), str(ex),
			HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_SMALL, CLR_DIM)
		y += LINE_HEIGHT_SMALL

	return y + SECTION_GAP


# ── Scrollbar ────────────────────────────────────────

func _draw_scrollbar() -> void:
	if _content_height <= size.y:
		return
	var track_h: float = size.y - 10.0
	var thumb_h: float = maxf(20.0, track_h * (size.y / _content_height))
	var scroll_ratio: float = _scroll_offset / (_content_height - size.y)
	var thumb_y: float = 5.0 + scroll_ratio * (track_h - thumb_h)
	var sx: float = size.x - SCROLLBAR_WIDTH - 2.0

	draw_rect(Rect2(sx, 5.0, SCROLLBAR_WIDTH, track_h), Color(0.2, 0.2, 0.2, 0.6))
	draw_rect(Rect2(sx, thumb_y, SCROLLBAR_WIDTH, thumb_h), Color(0.5, 0.5, 0.5, 0.8))


# ── Input ────────────────────────────────────────────

func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb: InputEventMouseButton = event
		if mb.pressed:
			if mb.button_index == MOUSE_BUTTON_WHEEL_UP:
				_scroll_offset = maxf(0.0, _scroll_offset - 40.0)
				queue_redraw()
				get_viewport().set_input_as_handled()
			elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				var max_scroll: float = maxf(0.0, _content_height - size.y)
				_scroll_offset = minf(max_scroll, _scroll_offset + 40.0)
				queue_redraw()
				get_viewport().set_input_as_handled()
			elif mb.button_index == MOUSE_BUTTON_LEFT:
				_check_section_click(mb.position)

	elif event is InputEventKey:
		var ke: InputEventKey = event
		if ke.pressed:
			if ke.keycode == KEY_PAGEDOWN:
				var max_scroll: float = maxf(0.0, _content_height - size.y)
				_scroll_offset = minf(max_scroll, _scroll_offset + size.y * 0.8)
				queue_redraw()
			elif ke.keycode == KEY_PAGEUP:
				_scroll_offset = maxf(0.0, _scroll_offset - size.y * 0.8)
				queue_redraw()
			elif ke.keycode == KEY_HOME:
				_scroll_offset = 0.0
				queue_redraw()
			elif ke.keycode == KEY_END:
				_scroll_offset = maxf(0.0, _content_height - size.y)
				queue_redraw()


func _check_section_click(click_pos: Vector2) -> void:
	var _adjusted_y: float = click_pos.y + _scroll_offset
	for section_id in _section_rects:
		var rect: Rect2 = _section_rects[section_id]
		var adj_rect: Rect2 = Rect2(rect.position.x, rect.position.y + _scroll_offset,
			rect.size.x, rect.size.y)
		if adj_rect.has_point(click_pos):
			_collapsed[section_id] = not _collapsed.get(section_id, false)
			if not _l3_loaded and not _collapsed.get(section_id, false):
				if section_id in ["skills", "values", "values_detail", "body_detail",
					"intelligence_detail", "personality_detail", "relationships",
					"memories", "economy", "faith", "flavor", "stress_detail"]:
					_load_l3_data()
			queue_redraw()
			return


# ── Drawing Helpers ──────────────────────────────────

func _draw_section_header(font: Font, x: float, y: float, title: String, section_id: String) -> float:
	var arrow: String = "▶" if _is_collapsed(section_id) else "▼"
	var header_text: String = "%s %s" % [arrow, title]
	draw_string(font, Vector2(x, y + 12.0), header_text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE_HEADER, CLR_HEADER)

	var panel_w: float = size.x - x - MARGIN_RIGHT
	_section_rects[section_id] = Rect2(x, y - _scroll_offset, panel_w, LINE_HEIGHT)

	return y + LINE_HEIGHT + 2.0


func _draw_bar(font: Font, x: float, y: float, w: float,
		label: String, value: float, color: Color, suffix: String = "") -> float:
	draw_string(font, Vector2(x, y + 11.0), label,
		HORIZONTAL_ALIGNMENT_LEFT, int(BAR_LABEL_W), FONT_SIZE_BODY, CLR_TEXT)

	var bar_x: float = x + BAR_LABEL_W + 4.0
	var bar_w: float = w - BAR_LABEL_W - BAR_VALUE_W - 8.0
	if bar_w < 10.0:
		bar_w = 10.0
	draw_rect(Rect2(bar_x, y + 2.0, bar_w, BAR_HEIGHT), CLR_BAR_BG)

	var fill_w: float = bar_w * clampf(value, 0.0, 1.0)
	if fill_w > 0.0:
		draw_rect(Rect2(bar_x, y + 2.0, fill_w, BAR_HEIGHT), color)

	var val_text: String = "%.2f" % value
	if suffix != "":
		val_text += " " + suffix
	var val_color: Color = CLR_TEXT
	if value < 0.15:
		val_color = CLR_DANGER
	elif value < 0.30:
		val_color = CLR_WARN
	draw_string(font, Vector2(bar_x + bar_w + 4.0, y + 11.0), val_text,
		HORIZONTAL_ALIGNMENT_LEFT, int(BAR_VALUE_W + 60), FONT_SIZE_SMALL, val_color)

	return y + LINE_HEIGHT


func _draw_bipolar_bar(font: Font, x: float, y: float, w: float,
		label: String, value: float, pos_color: Color, neg_color: Color) -> float:
	draw_string(font, Vector2(x, y + 11.0), label,
		HORIZONTAL_ALIGNMENT_LEFT, int(BAR_LABEL_W), FONT_SIZE_BODY, CLR_TEXT)

	var bar_x: float = x + BAR_LABEL_W + 4.0
	var bar_w: float = w - BAR_LABEL_W - BAR_VALUE_W - 8.0
	if bar_w < 10.0:
		bar_w = 10.0
	var center: float = bar_x + bar_w * 0.5
	draw_rect(Rect2(bar_x, y + 2.0, bar_w, BAR_HEIGHT), CLR_BAR_BG)
	draw_line(Vector2(center, y + 1.0), Vector2(center, y + BAR_HEIGHT + 3.0), CLR_SECTION_LINE)

	var clamped: float = clampf(value, -1.0, 1.0)
	var fill_w: float = bar_w * 0.5 * absf(clamped)
	if clamped > 0.0:
		draw_rect(Rect2(center, y + 2.0, fill_w, BAR_HEIGHT), pos_color)
	elif clamped < 0.0:
		draw_rect(Rect2(center - fill_w, y + 2.0, fill_w, BAR_HEIGHT), neg_color)

	var val_color: Color = pos_color if clamped >= 0.0 else neg_color
	draw_string(font, Vector2(bar_x + bar_w + 4.0, y + 11.0),
		"%+.2f" % value, HORIZONTAL_ALIGNMENT_LEFT, int(BAR_VALUE_W + 30),
		FONT_SIZE_SMALL, val_color)

	return y + LINE_HEIGHT


func _draw_wrapped_text(font: Font, x: float, y: float, text: String,
		color: Color, fsize: int = FONT_SIZE_NARRATIVE) -> float:
	var max_w: float = size.x - x - MARGIN_RIGHT
	if max_w < 20.0:
		max_w = 20.0
	var words: PackedStringArray = text.split(" ")
	var current_line: String = ""

	for word in words:
		var test_line: String = (current_line + " " + word).strip_edges() if current_line != "" else word
		var test_w: float = font.get_string_size(test_line, HORIZONTAL_ALIGNMENT_LEFT, -1, fsize).x
		if test_w > max_w and current_line != "":
			draw_string(font, Vector2(x, y + 11.0), current_line,
				HORIZONTAL_ALIGNMENT_LEFT, -1, fsize, color)
			y += LINE_HEIGHT_SMALL
			current_line = word
		else:
			current_line = test_line

	if current_line != "":
		draw_string(font, Vector2(x, y + 11.0), current_line,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fsize, color)
		y += LINE_HEIGHT_SMALL

	return y


func _is_collapsed(section_id: String) -> bool:
	return _collapsed.get(section_id, false)


static func _get_intensity_key(val: float) -> String:
	if val >= 0.86: return "UI_INTENSITY_EXTREME"
	if val >= 0.71: return "UI_INTENSITY_VERY"
	if val >= 0.56: return "UI_INTENSITY_SLIGHTLY"
	if val <= 0.14: return "UI_INTENSITY_EXTREME"
	if val <= 0.29: return "UI_INTENSITY_VERY"
	if val <= 0.44: return "UI_INTENSITY_SLIGHTLY"
	return ""
