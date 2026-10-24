# flora_resolver.gd — the flora catalogue's resolver in GDScript (M-ART2.1 §2.2, ADR 0033, ADR 0034 addendum).
# Purpose: a port of the art line's `art/flora/resolve_flora.py` (v8/art-trees f182769f): `resolve(species, size,
#   season, variant_hash, habit)` picks the frame, canopy mask, season tint (CIELAB shift, foliage or bare frame,
#   snow cap), anchor, canvas, map canopy and log material of a plant from `assets/v8art/flora/species.json` and
#   `habit_defaults.json`. An unknown species falls back to its habit's fallback species (the caller's habit, else
#   the document's `unknown_habit`) with the habit's own season records, never the fallback's; `requested_id` keeps the
#   asked-for id. The variant is `variant_hash` modulo the variant count (non-negative, as Python's %). No species or
#   season is named here: they come from the documents (the habit names of schema.py's rules are its schema
#   vocabulary). `load_docs` checks the documents as the art line's schema does (keys equal ids, four seasons, both
#   sizes, bare frames and a bare winter exactly for deciduous plants, a tree's log material = its id with a stump,
#   anchors inside canvases, fallbacks of the right habit) and lists every problem in `problems`; a broken document
#   resolves to {}. `resolver_parity.gd` checks
#   this port against resolve_flora.py itself (gate step `resolver_parity`).
# Bridge methods called: none.
extends RefCounted

const ArtData = preload("res://scripts/world/art/art_data.gd")
const SPECIES_PATH: String = "res://assets/v8art/flora/species.json"
const DEFAULTS_PATH: String = "res://assets/v8art/flora/habit_defaults.json"
## schema.py's habit vocabulary and its rules (schema names of the art line's Literal, not content): the habit whose
## plants must have bare frames and a bare winter, and the one exempt from the tree rule (log material = id, a stump).
const HABIT_BARE: String = "deciduous"
const HABIT_SHRUB: String = "shrub"
## schema.py `DefaultsDocument.unknown_habit` default.
const DEFAULT_UNKNOWN_HABIT: String = "deciduous"

## Loader problems (empty = the documents parsed as the schema requires).
var problems: Array[String] = []
var unknown_habit: String = ""
var _species: Dictionary = {}
var _habits: Dictionary = {}
var _canopies: Dictionary = {}
var _seasons: Array[String] = []
var _sizes: Array[String] = []


## Load the imported catalogue (assets/v8art/flora/).
func load_imported() -> void:
	load_docs(_read(SPECIES_PATH), _read(DEFAULTS_PATH))


func _read(path: String) -> Dictionary:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	if parsed is Dictionary:
		return parsed
	problems.append("cannot read %s" % path)
	return {}


## Load a species document and a habit-defaults document (tests pass their own).
func load_docs(species_doc: Dictionary, defaults_doc: Dictionary) -> void:
	_species = (species_doc.get("species", {}) as Dictionary).duplicate(true)
	_canopies = (species_doc.get("map_canopies", {}) as Dictionary).duplicate(true)
	_habits = (defaults_doc.get("habits", {}) as Dictionary).duplicate(true)
	unknown_habit = str(defaults_doc.get("unknown_habit", DEFAULT_UNKNOWN_HABIT))
	if int(species_doc.get("schema_version", 0)) != 1 or int(defaults_doc.get("schema_version", 0)) != 1:
		problems.append("schema_version is not 1")
	# The season and size vocabularies are the defaults' and the first species' keys (data, not code).
	_seasons.clear()
	_sizes.clear()
	if not _habits.is_empty():
		for s: Variant in ((_habits.values()[0] as Dictionary).get("seasons", {}) as Dictionary).keys():
			_seasons.append(str(s))
	if not _species.is_empty():
		for s: Variant in ((_species.values()[0] as Dictionary).get("sizes", {}) as Dictionary).keys():
			_sizes.append(str(s))
	for key: Variant in _species.keys():
		_check_species(str(key), _species[key])
	for habit: Variant in _habits.keys():
		var d: Dictionary = _habits[habit]
		if not _same_keys(d.get("seasons", {}), _seasons):
			problems.append("habit %s lacks a season" % habit)
		var fb: Dictionary = _species.get(str(d.get("fallback_species", "")), {})
		if fb.is_empty():
			problems.append("habit %s: fallback species %s is not in the catalogue" % [habit, d.get("fallback_species", "")])
		elif str(fb.get("habit", "")) != str(habit):
			problems.append("habit %s: fallback species has habit %s" % [habit, fb.get("habit", "")])
	if not _habits.has(unknown_habit):
		problems.append("unknown_habit %s is not a habit" % unknown_habit)


## Add one species record (the data-only fixture `test_alder`); returns false (and records why) when it is invalid.
func add_species(record: Dictionary) -> bool:
	var before: int = problems.size()
	_check_species(str(record.get("id", "")), record)
	if problems.size() != before:
		return false
	_species[str(record["id"])] = record.duplicate(true)
	return true


func _same_keys(d: Variant, keys: Array[String]) -> bool:
	if not d is Dictionary or (d as Dictionary).size() != keys.size():
		return false
	for k: String in keys:
		if not (d as Dictionary).has(k):
			return false
	return true


func _check_species(key: String, sp: Dictionary) -> void:
	if str(sp.get("id", "")) != key:
		problems.append("species key %s differs from its id %s" % [key, sp.get("id", "")])
	if not _same_keys(sp.get("seasons", {}), _seasons):
		problems.append("species %s lacks a season" % key)
	if not _same_keys(sp.get("sizes", {}), _sizes):
		problems.append("species %s lacks a size" % key)
	if not _canopies.has(str(sp.get("map_canopy", ""))):
		problems.append("species %s map canopy %s does not resolve" % [key, sp.get("map_canopy", "")])
	var wants_bare: bool = false
	for season: Variant in (sp.get("seasons", {}) as Dictionary).values():
		wants_bare = wants_bare or str((season as Dictionary).get("frame", "foliage")) == "bare"
	var habit: String = str(sp.get("habit", ""))
	var last: String = _seasons[_seasons.size() - 1] if not _seasons.is_empty() else ""
	if habit == HABIT_BARE and str(((sp.get("seasons", {}) as Dictionary).get(last, {}) as Dictionary).get("frame", "foliage")) != "bare":
		problems.append("species %s: a %s plant's %s must select bare frames" % [key, habit, last])
	if habit != HABIT_BARE and wants_bare:
		problems.append("species %s: only %s plants select bare frames" % [key, HABIT_BARE])
	if habit != HABIT_SHRUB and (str(sp.get("log_material", "")) != key or not sp.get("stump", null) is Dictionary):
		problems.append("species %s: a tree's log_material equals its id and it has a stump" % key)
	if sp.get("stump", null) is Dictionary:
		_check_frame(key, sp["stump"])
	for size: Variant in (sp.get("sizes", {}) as Dictionary).values():
		var frames: Dictionary = size
		if (frames.get("variants", []) as Array).is_empty():
			problems.append("species %s has a size without variants" % key)
		var bare: Array = frames.get("bare", [])
		if (habit == HABIT_BARE) == bare.is_empty():
			problems.append("species %s: bare frames exactly for %s plants" % [key, HABIT_BARE])
		for f: Variant in (frames.get("variants", []) as Array) + bare:
			_check_frame(key, f)


## schema.py `Frame`: a positive canvas with the anchor inside it, and the two paths.
func _check_frame(key: String, f: Variant) -> void:
	var d: Dictionary = f if f is Dictionary else {}
	var c: Array = d.get("canvas", [])
	var a: Array = d.get("anchor", [])
	if c.size() != 2 or a.size() != 2 or int(c[0]) <= 0 or int(c[1]) <= 0 or int(a[0]) < 0 or int(a[1]) < 0 or int(a[0]) >= int(c[0]) or int(a[1]) >= int(c[1]) or str(d.get("path", "")) == "" or str(d.get("canopy_mask", "")) == "":
		problems.append("species %s: a frame needs a positive canvas with its anchor inside and both paths" % key)


## The species ids of the catalogue, in document order.
func species_ids() -> Array[String]:
	var out: Array[String] = []
	for k: Variant in _species.keys():
		out.append(str(k))
	return out


## The season names in document order (index = the calendar's season code).
func seasons() -> Array[String]:
	return _seasons


## A season record with the art line's defaults filled in (SeasonTint: delta 0, foliage, no snow).
static func _tint(src: Variant) -> Dictionary:
	var t: Dictionary = src if src is Dictionary else {}
	var delta: Array = t.get("delta_lab", [0, 0, 0])
	var snow: Dictionary = t.get("snow_cap", {})
	return {"delta_lab": [float(delta[0]), float(delta[1]), float(delta[2])], "frame": str(t.get("frame", "foliage")),
		"snow_cap": {"depth_px": int(snow.get("depth_px", 0)), "alpha": float(snow.get("alpha", 0.0))}}


## resolve_flora.py `resolve_flora`: the frame of `species_id` at `size` in `season` (see the header); {} when the
## catalogue cannot answer (a broken document, an unknown size or season).
func resolve(species_id: String, size: String, season: String, variant_hash: int, habit: String = "") -> Dictionary:
	var known: Dictionary = _species.get(species_id, {})
	var selected_habit: String = str(known.get("habit", "")) if not known.is_empty() else (habit if habit != "" else unknown_habit)
	var default: Dictionary = _habits.get(selected_habit, {})
	var selected: Dictionary = known if not known.is_empty() else _species.get(str(default.get("fallback_species", "")), {})
	if selected.is_empty() or default.is_empty():
		return {}
	var record: Variant = (selected.get("seasons", {}) as Dictionary).get(season, null) if not known.is_empty() else (default.get("seasons", {}) as Dictionary).get(season, null)
	if record == null:
		return {}
	var tint: Dictionary = _tint(record)
	var frames: Dictionary = (selected.get("sizes", {}) as Dictionary).get(size, {})
	var variants: Array = frames.get("variants" if tint["frame"] == "foliage" else "bare", [])
	if variants.is_empty():
		return {}
	var frame: Dictionary = variants[posmod(variant_hash, variants.size())]
	var canopy: String = str(selected.get("map_canopy", ""))
	return {"requested_id": species_id, "species_id": str(selected.get("id", "")), "habit": selected_habit,
		"fallback_used": known.is_empty(), "path": str(frame.get("path", "")),
		"canopy_mask": str(frame.get("canopy_mask", "")), "tint": tint,
		"anchor": [int(frame["anchor"][0]), int(frame["anchor"][1])],
		"canvas": [int(frame["canvas"][0]), int(frame["canvas"][1])], "map_canopy": canopy,
		"map_canopy_path": str(_canopies.get(canopy, "")), "log_material": selected.get("log_material", null)}


## The stump frame of `species_id` (its habit fallback's when unknown or stumpless): {path, canopy_mask, anchor,
## canvas} or {} (not part of resolve_flora.py; the renderer's felled-tree sprite).
func stump(species_id: String, habit: String = "") -> Dictionary:
	var known: Dictionary = _species.get(species_id, {})
	var s: Variant = known.get("stump", null)
	if s is Dictionary:
		return s
	var h: String = str(known.get("habit", "")) if not known.is_empty() else (habit if habit != "" else unknown_habit)
	var fb: Dictionary = _species.get(str((_habits.get(h, {}) as Dictionary).get("fallback_species", "")), {})
	var t: Variant = fb.get("stump", null)
	return t if t is Dictionary else {}


## The canopy mask path the catalogue gives the frame at art path `frame_path` ("" when no frame has that path).
func mask_of(frame_path: String) -> String:
	for sp: Variant in _species.values():
		for size: Variant in ((sp as Dictionary).get("sizes", {}) as Dictionary).values():
			for list: String in ["variants", "bare"]:
				for f: Variant in (size as Dictionary).get(list, []):
					if str((f as Dictionary).get("path", "")) == frame_path:
						return str((f as Dictionary).get("canopy_mask", ""))
	return ""


## The res:// path of an art-line path the catalogue names ("" when it was not imported).
static func res_path(art_path: String) -> String:
	return ArtData.path_of_source(art_path)
