extends RefCounted

## Plutchik 8 basic emotions with 3-layer temporal dynamics.
## Fast (episodic) + Slow (mood/baseline, OU process) + Memory trace (long-term scars).
## Valence-Arousal derived each tick. 24 Dyads computed on-the-fly.
## References:
##   Plutchik (1980, 2001) — 8 emotions, intensity levels, Dyad system
##   Russell (1980) — Circumplex Model of Affect (Valence-Arousal)

# === 8 basic emotions (each 0.0 ~ 100.0) ===
# Final emotion = fast + slow + memory (clamped 0~100)

# Fast layer (episodic, rapid decay)
var fast: Dictionary = {
	"joy": 0.0, "trust": 0.0, "fear": 0.0, "surprise": 0.0,
	"sadness": 0.0, "disgust": 0.0, "anger": 0.0, "anticipation": 0.0
}

# Slow layer (mood/baseline, slow OU process)
var slow: Dictionary = {
	"joy": 5.0, "trust": 5.0, "fear": 0.0, "surprise": 0.0,
	"sadness": 0.0, "disgust": 0.0, "anger": 0.0, "anticipation": 5.0
}

# Memory trace layer (long-term memory, event scars)
# key: emotion_id, value: Array of {source: String, intensity: float, decay_rate: float}
var memory_traces: Dictionary = {
	"joy": [], "trust": [], "fear": [], "surprise": [],
	"sadness": [], "disgust": [], "anger": [], "anticipation": []
}

# === Derived values (recalculated each tick) ===
var valence: float = 0.0      # -100 ~ +100 (positive/negative)
var arousal: float = 0.0      # 0 ~ 100 (activation)
var stress: float = 0.0       # 0 ~ 1000+ (Mental Break accumulator)

# === Habituation counters ===
# key: event_category, value: {count: int, last_tick: int}
var habituation: Dictionary = {}

# === Mental break state ===
var mental_break_type: String = ""      # "" = none, "panic"/"rage"/"shutdown"/"purge"/"outrage_violence"
var mental_break_remaining: float = 0.0  # hours remaining

var _intensity_labels: Dictionary = {}
var _intensity_labels_kr: Dictionary = {}
var _emotion_order: Array = []
var _emotion_labels_en: Dictionary = {}
var _emotion_labels_kr: Dictionary = {}
var _dyads: Dictionary = {}           # dyad_id -> Array of 2 emotion ids
var _dyad_labels_kr: Dictionary = {}
var _valence_positive: Dictionary = {}
var _valence_negative: Dictionary = {}
var _arousal_weights: Dictionary = {}


func _init() -> void:
	_load_emotion_definitions()
	_load_dyad_definitions()


func _load_emotion_definitions() -> void:
	var ed = SpeciesManager.emotion_definition
	_intensity_labels = ed.get("intensity_labels", {
		"joy": ["Serenity", "Joy", "Ecstasy"],
		"trust": ["Acceptance", "Trust", "Admiration"],
		"fear": ["Apprehension", "Fear", "Terror"],
		"surprise": ["Distraction", "Surprise", "Amazement"],
		"sadness": ["Pensiveness", "Sadness", "Grief"],
		"disgust": ["Boredom", "Disgust", "Loathing"],
		"anger": ["Annoyance", "Anger", "Rage"],
		"anticipation": ["Interest", "Anticipation", "Vigilance"]
	})
	_intensity_labels_kr = ed.get("intensity_labels_kr", {
		"joy": ["평온", "기쁨", "황홀"],
		"trust": ["수용", "신뢰", "경외"],
		"fear": ["우려", "공포", "경악"],
		"surprise": ["산만", "놀람", "경이"],
		"sadness": ["수심", "슬픔", "비통"],
		"disgust": ["지루함", "혐오", "증오"],
		"anger": ["짜증", "분노", "격노"],
		"anticipation": ["흥미", "기대", "경계"]
	})
	_emotion_order = ed.get("emotion_order", ["joy", "trust", "fear", "surprise", "sadness", "disgust", "anger", "anticipation"])
	_emotion_labels_en = ed.get("labels_en", {
		"joy": "Joy", "trust": "Trust", "fear": "Fear", "surprise": "Surprise",
		"sadness": "Sadness", "disgust": "Disgust", "anger": "Anger", "anticipation": "Anticipation"
	})
	_emotion_labels_kr = ed.get("labels_kr", {
		"joy": "기쁨", "trust": "신뢰", "fear": "공포", "surprise": "놀람",
		"sadness": "슬픔", "disgust": "혐오", "anger": "분노", "anticipation": "기대"
	})
	var vw = ed.get("valence_weights", {})
	_valence_positive = vw.get("positive", {"joy": 1.0, "trust": 1.0, "anticipation": 0.5})
	_valence_negative = vw.get("negative", {"sadness": 1.0, "disgust": 1.0, "fear": 0.5})
	_arousal_weights = ed.get("arousal_weights", {"fear": 1.0, "surprise": 1.0, "anger": 1.0, "anticipation": 1.0, "joy": 0.3})


func _load_dyad_definitions() -> void:
	var dd = SpeciesManager.dyad_definition
	var dyads_raw = dd.get("dyads", {})
	_dyads = {}
	_dyad_labels_kr = {}
	for dyad_id in dyads_raw:
		var entry = dyads_raw[dyad_id]
		_dyads[dyad_id] = entry.get("components", [])
		_dyad_labels_kr[dyad_id] = str(entry.get("name_kr", dyad_id))
	# Fallback if empty
	if _dyads.is_empty():
		_dyads = {
			"love": ["joy", "trust"], "submission": ["trust", "fear"],
			"awe": ["fear", "surprise"], "disappointment": ["surprise", "sadness"],
			"remorse": ["sadness", "disgust"], "contempt": ["disgust", "anger"],
			"aggressiveness": ["anger", "anticipation"], "optimism": ["anticipation", "joy"],
			"hope": ["anticipation", "trust"], "guilt": ["joy", "fear"],
			"curiosity": ["trust", "surprise"], "despair": ["fear", "sadness"],
			"unbelief": ["surprise", "disgust"], "envy": ["sadness", "anger"],
			"cynicism": ["disgust", "anticipation"], "pride": ["anger", "joy"],
			"delight": ["joy", "surprise"], "sentimentality": ["trust", "sadness"],
			"shame": ["fear", "disgust"], "outrage": ["surprise", "anger"],
			"pessimism": ["sadness", "anticipation"], "morbidness": ["disgust", "joy"],
			"dominance": ["anger", "trust"], "anxiety": ["anticipation", "fear"]
		}


# === Final emotion value (fast + slow + memory, clamped 0~100) ===
func get_emotion(emotion_id: String) -> float:
	var f = fast.get(emotion_id, 0.0)
	var s = slow.get(emotion_id, 0.0)
	var m = _get_memory_total(emotion_id)
	return clampf(f + s + m, 0.0, 100.0)


func _get_memory_total(emotion_id: String) -> float:
	var total: float = 0.0
	var traces = memory_traces.get(emotion_id, [])
	for i in range(traces.size()):
		total += traces[i].intensity
	return total


# === Intensity labels (UI display) ===
func get_intensity_label(emotion_id: String) -> String:
	var val: float = get_emotion(emotion_id)
	if val < 1.0:
		return ""
	var labels = _intensity_labels.get(emotion_id, ["", "", ""])
	if val <= 33.0:
		return labels[0]    # mild
	if val <= 66.0:
		return labels[1]    # base
	return labels[2]         # intense


func get_intensity_label_kr(emotion_id: String) -> String:
	var val: float = get_emotion(emotion_id)
	if val < 1.0:
		return ""
	var labels = _intensity_labels_kr.get(emotion_id, ["", "", ""])
	if val <= 33.0:
		return labels[0]
	if val <= 66.0:
		return labels[1]
	return labels[2]

# === Valence-Arousal calculation ===
func recalculate_va() -> void:
	var pos: float = 0.0
	for emo in _valence_positive:
		pos += float(_valence_positive[emo]) * get_emotion(emo)
	var neg: float = 0.0
	for emo in _valence_negative:
		neg += float(_valence_negative[emo]) * get_emotion(emo)
	valence = clampf(pos - neg, -100.0, 100.0)

	var arousal_sum: float = 0.0
	var arousal_divisor: float = 0.0
	for emo in _arousal_weights:
		arousal_sum += float(_arousal_weights[emo]) * get_emotion(emo)
		arousal_divisor += float(_arousal_weights[emo])
	if arousal_divisor > 0.0:
		arousal = clampf(arousal_sum / arousal_divisor, 0.0, 100.0)
	else:
		arousal = 0.0


func get_dyad(dyad_id: String) -> float:
	var pair = _dyads.get(dyad_id, [])
	if pair.size() < 2:
		return 0.0
	var e1: float = get_emotion(pair[0])
	var e2: float = get_emotion(pair[1])
	return sqrt(e1 * e2)  # Geometric mean: strong only when both are high


## Get all dyads with value >= threshold (for UI display)
func get_active_dyads(threshold: float = 30.0) -> Array:
	var result: Array = []
	for dyad_id in _dyads:
		var val: float = get_dyad(dyad_id)
		if val >= threshold:
			result.append({"id": dyad_id, "value": val})
	# Sort by value descending
	result.sort_custom(func(a, b): return a.value > b.value)
	return result


# === Legacy 5-emotion compatibility ===
# Existing systems reference these values. Derived from Plutchik emotions.

func get_legacy_happy() -> float:
	return clampf(get_emotion("joy") - get_emotion("sadness"), 0.0, 100.0) / 100.0


func get_legacy_love() -> float:
	return get_dyad("love") / 100.0


func get_legacy_grief() -> float:
	return get_emotion("sadness") / 100.0


func get_legacy_stress() -> float:
	return clampf((get_emotion("fear") + get_emotion("anger") + get_emotion("anticipation")) / 3.0, 0.0, 100.0) / 100.0


func get_legacy_lonely() -> float:
	return clampf(50.0 - get_emotion("trust") + get_emotion("sadness") * 0.5, 0.0, 100.0) / 100.0


## Build legacy emotions Dictionary (0.0-1.0 range) for backward compat
func to_legacy_dict() -> Dictionary:
	return {
		"happiness": get_legacy_happy(),
		"loneliness": get_legacy_lonely(),
		"stress": get_legacy_stress(),
		"grief": get_legacy_grief(),
		"love": get_legacy_love(),
	}


# === Serialization ===

func to_dict() -> Dictionary:
	var mt: Dictionary = {}
	for emo in memory_traces:
		var arr: Array = []
		var traces = memory_traces[emo]
		for i in range(traces.size()):
			arr.append({
				"source": traces[i].source,
				"intensity": traces[i].intensity,
				"decay_rate": traces[i].decay_rate,
			})
		mt[emo] = arr

	return {
		"fast": fast.duplicate(),
		"slow": slow.duplicate(),
		"memory_traces": mt,
		"valence": valence,
		"arousal": arousal,
		"stress": stress,
		"habituation": habituation.duplicate(true),
		"mental_break_type": mental_break_type,
		"mental_break_remaining": mental_break_remaining,
	}


static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/emotion_data.gd")
	var ed = script.new()
	var f_data = data.get("fast", {})
	for emo in ed.fast:
		ed.fast[emo] = float(f_data.get(emo, 0.0))
	var s_data = data.get("slow", {})
	for emo in ed.slow:
		ed.slow[emo] = float(s_data.get(emo, ed.slow[emo]))
	var mt_data = data.get("memory_traces", {})
	for emo in ed.memory_traces:
		ed.memory_traces[emo] = []
		var traces = mt_data.get(emo, [])
		for i in range(traces.size()):
			var t = traces[i]
			ed.memory_traces[emo].append({
				"source": str(t.get("source", "unknown")),
				"intensity": float(t.get("intensity", 0.0)),
				"decay_rate": float(t.get("decay_rate", 0.0)),
			})
	ed.stress = float(data.get("stress", 0.0))
	var hab = data.get("habituation", {})
	ed.habituation = {}
	for cat in hab:
		ed.habituation[cat] = {
			"count": int(hab[cat].get("count", 0)),
			"last_tick": int(hab[cat].get("last_tick", 0)),
		}
	ed.mental_break_type = str(data.get("mental_break_type", ""))
	ed.mental_break_remaining = float(data.get("mental_break_remaining", 0.0))
	ed.recalculate_va()
	return ed


## Migrate from legacy 5-emotion Dictionary (0.0-1.0 range)
static func from_legacy(old_emotions: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/emotion_data.gd")
	var ed = script.new()
	# Map legacy values (0-1 range) to Plutchik (0-100 range)
	ed.fast["joy"] = float(old_emotions.get("happiness", 0.5)) * 50.0
	ed.slow["joy"] = 5.0
	ed.slow["sadness"] = float(old_emotions.get("grief", 0.0)) * 30.0
	ed.fast["trust"] = (1.0 - float(old_emotions.get("loneliness", 0.0))) * 30.0
	ed.stress = float(old_emotions.get("stress", 0.0)) * 300.0
	# Love -> joy+trust fast layers
	var old_love: float = float(old_emotions.get("love", 0.0))
	ed.fast["joy"] += old_love * 20.0
	ed.fast["trust"] += old_love * 20.0
	ed.recalculate_va()
	return ed
