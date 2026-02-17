# T-2018-01: EmotionData Data Structure

## Objective
Create the EmotionData RefCounted class holding Plutchik 8 basic emotions with 3-layer temporal dynamics, Valence-Arousal derivation, 24 Dyad real-time calculation, and legacy 5-emotion compatibility.

## File to Create

### `scripts/core/emotion_data.gd` (NEW)

## Godot 4.6 Headless Compatibility (CRITICAL)
- **NO `class_name`** — this is a RefCounted script; class_name fails in Godot 4.6 headless mode
- Use `extends RefCounted` at top
- Other scripts reference this via `preload("res://scripts/core/emotion_data.gd")`
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)` (inferred type fails)

## Full Implementation

```gdscript
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
	var labels = INTENSITY_LABELS.get(emotion_id, ["", "", ""])
	if val <= 33.0:
		return labels[0]    # mild
	if val <= 66.0:
		return labels[1]    # base
	return labels[2]         # intense


func get_intensity_label_kr(emotion_id: String) -> String:
	var val: float = get_emotion(emotion_id)
	if val < 1.0:
		return ""
	var labels = INTENSITY_LABELS_KR.get(emotion_id, ["", "", ""])
	if val <= 33.0:
		return labels[0]
	if val <= 66.0:
		return labels[1]
	return labels[2]


const INTENSITY_LABELS: Dictionary = {
	"joy":          ["Serenity",     "Joy",          "Ecstasy"],
	"trust":        ["Acceptance",   "Trust",        "Admiration"],
	"fear":         ["Apprehension", "Fear",         "Terror"],
	"surprise":     ["Distraction",  "Surprise",     "Amazement"],
	"sadness":      ["Pensiveness",  "Sadness",      "Grief"],
	"disgust":      ["Boredom",      "Disgust",      "Loathing"],
	"anger":        ["Annoyance",    "Anger",        "Rage"],
	"anticipation": ["Interest",     "Anticipation", "Vigilance"]
}

const INTENSITY_LABELS_KR: Dictionary = {
	"joy":          ["평온",   "기쁨",   "황홀"],
	"trust":        ["수용",   "신뢰",   "경외"],
	"fear":         ["우려",   "공포",   "경악"],
	"surprise":     ["산만",   "놀람",   "경이"],
	"sadness":      ["수심",   "슬픔",   "비통"],
	"disgust":      ["지루함", "혐오",   "증오"],
	"anger":        ["짜증",   "분노",   "격노"],
	"anticipation": ["흥미",   "기대",   "경계"]
}

# Emotion display order (for UI)
const EMOTION_ORDER: Array = ["joy", "trust", "fear", "surprise", "sadness", "disgust", "anger", "anticipation"]

# English labels for UI
const EMOTION_LABELS_EN: Dictionary = {
	"joy": "Joy", "trust": "Trust", "fear": "Fear", "surprise": "Surprise",
	"sadness": "Sadness", "disgust": "Disgust", "anger": "Anger", "anticipation": "Anticipation"
}

# Korean labels for UI
const EMOTION_LABELS_KR: Dictionary = {
	"joy": "기쁨", "trust": "신뢰", "fear": "공포", "surprise": "놀람",
	"sadness": "슬픔", "disgust": "혐오", "anger": "분노", "anticipation": "기대"
}

# === Valence-Arousal calculation ===
func recalculate_va() -> void:
	var j: float = get_emotion("joy")
	var t: float = get_emotion("trust")
	var f: float = get_emotion("fear")
	var su: float = get_emotion("surprise")
	var sd: float = get_emotion("sadness")
	var d: float = get_emotion("disgust")
	var a: float = get_emotion("anger")
	var n: float = get_emotion("anticipation")

	# Valence: positive - negative (-100 ~ +100)
	valence = (j + t + 0.5 * n) - (sd + d + 0.5 * f)
	valence = clampf(valence, -100.0, 100.0)

	# Arousal: activation (0 ~ 100)
	arousal = (f + su + a + n + 0.3 * j) / 4.3
	arousal = clampf(arousal, 0.0, 100.0)


# === 24 Dyad definitions (computed on-the-fly, never stored) ===
const DYADS: Dictionary = {
	# Primary (adjacent on Plutchik wheel)
	"love":           ["joy", "trust"],
	"submission":     ["trust", "fear"],
	"awe":            ["fear", "surprise"],
	"disappointment": ["surprise", "sadness"],
	"remorse":        ["sadness", "disgust"],
	"contempt":       ["disgust", "anger"],
	"aggressiveness": ["anger", "anticipation"],
	"optimism":       ["anticipation", "joy"],
	# Secondary (one apart)
	"hope":           ["anticipation", "trust"],
	"guilt":          ["joy", "fear"],
	"curiosity":      ["trust", "surprise"],
	"despair":        ["fear", "sadness"],
	"unbelief":       ["surprise", "disgust"],
	"envy":           ["sadness", "anger"],
	"cynicism":       ["disgust", "anticipation"],
	"pride":          ["anger", "joy"],
	# Tertiary (two apart)
	"delight":        ["joy", "surprise"],
	"sentimentality": ["trust", "sadness"],
	"shame":          ["fear", "disgust"],
	"outrage":        ["surprise", "anger"],
	"pessimism":      ["sadness", "anticipation"],
	"morbidness":     ["disgust", "joy"],
	"dominance":      ["anger", "trust"],
	"anxiety":        ["anticipation", "fear"]
}

# Korean Dyad labels
const DYAD_LABELS_KR: Dictionary = {
	"love": "사랑", "submission": "복종", "awe": "경외", "disappointment": "실망",
	"remorse": "후회", "contempt": "경멸", "aggressiveness": "공격성", "optimism": "낙관",
	"hope": "희망", "guilt": "죄책감", "curiosity": "호기심", "despair": "절망",
	"unbelief": "불신", "envy": "시기", "cynicism": "냉소", "pride": "자부심",
	"delight": "환희", "sentimentality": "감상", "shame": "수치", "outrage": "격분",
	"pessimism": "비관", "morbidness": "잔혹", "dominance": "지배", "anxiety": "불안"
}


func get_dyad(dyad_id: String) -> float:
	var pair = DYADS.get(dyad_id, [])
	if pair.size() < 2:
		return 0.0
	var e1: float = get_emotion(pair[0])
	var e2: float = get_emotion(pair[1])
	return sqrt(e1 * e2)  # Geometric mean: strong only when both are high


## Get all dyads with value >= threshold (for UI display)
func get_active_dyads(threshold: float = 30.0) -> Array:
	var result: Array = []
	for dyad_id in DYADS:
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
	# Love → joy+trust fast layers
	var old_love: float = float(old_emotions.get("love", 0.0))
	ed.fast["joy"] += old_love * 20.0
	ed.fast["trust"] += old_love * 20.0
	ed.recalculate_va()
	return ed
```

## Non-goals
- Do NOT create EmotionSystem (that's T-2018-02)
- Do NOT modify entity_data.gd, save_manager.gd, or any other file
- Do NOT add any Node dependencies — this is pure RefCounted data
- Do NOT use `class_name` anywhere

## Acceptance Criteria
- [ ] `scripts/core/emotion_data.gd` exists with `extends RefCounted` and NO `class_name`
- [ ] 8 basic emotions in fast, slow, memory_traces layers
- [ ] `get_emotion()` returns clamped sum of 3 layers
- [ ] `recalculate_va()` computes valence (-100~+100) and arousal (0~100)
- [ ] 24 Dyads defined in DYADS const, computed by `get_dyad()` using geometric mean
- [ ] `get_active_dyads(threshold)` returns sorted array of active dyads
- [ ] Intensity labels (EN + KR) with 3 levels per emotion
- [ ] Legacy compat: `get_legacy_happy/love/grief/stress/lonely()` return 0.0-1.0 range
- [ ] `to_legacy_dict()` returns old-format Dictionary
- [ ] `to_dict()` / `from_dict()` serialization roundtrip
- [ ] `from_legacy()` migration from old 5-emotion Dictionary
- [ ] Mental break state fields (type + remaining)
- [ ] No GDScript parse errors
