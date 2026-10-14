extends SceneTree
const News = preload("res://scripts/ui/panels/hud_status_panel.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const VILLAGE_ARGUMENTS: Dictionary = {"HUD_FEED_FISSION_DEPARTED": 1, "HUD_FEED_SETTLEMENT_FOUNDED": 0, "HUD_FEED_SETTLEMENT_ABANDONED": 0, "HUD_FEED_SETTLEMENT_FORMED": 0, "HUD_FEED_SETTLEMENT_DISSOLVED": 0, "HUD_FEED_GOD_CAST_FMT": 0, "HUD_FEED_DAY_SUMMARY_FMT": 0, "HUD_FEED_STOCKPILE_DEPOSIT": 0, "HUD_FEED_STOCKPILE_CONSUME": 0}

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	Selection.reset()
	Selection.observe({"settlement_ids": PackedInt32Array([7, 42]), "parent_ids": PackedInt32Array([-1, 7]), "member_counts": PackedInt32Array([30, 10])})
	assert(Selection.number_for(7) == 1 and Selection.number_for(42) == 2)
	Selection.observe({"settlement_ids": PackedInt32Array([42]), "parent_ids": PackedInt32Array([7]), "member_counts": PackedInt32Array([10])})
	assert(Selection.number_for(7) == 1 and Selection.number_for(42) == 2, "retired numbers stable")
	for language: String in ["en", "ko"]:
		var document: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://localization/compiled/%s.json" % language))
		var strings: Dictionary = document["strings"]
		for key: String in News.FEED_CLASSES:
			assert(strings.has(key), "every feed class key localized")
		for key: String in VILLAGE_ARGUMENTS:
			var args: Array = [Selection.number_for(42)]
			match key:
				"HUD_FEED_FISSION_DEPARTED": args = [13, Selection.number_for(42)]
				"HUD_FEED_GOD_CAST_FMT": args.append(strings["GOD_HAND_GIFT_FOOD"])
				"HUD_FEED_DAY_SUMMARY_FMT": args.append_array([12, 30, 18])
				"HUD_FEED_STOCKPILE_DEPOSIT", "HUD_FEED_STOCKPILE_CONSUME": args.append(4)
			var text: String = str(strings[key]) % args
			assert(args[int(VILLAGE_ARGUMENTS[key])] == 2)
			assert(text.contains("Village 2") if language == "en" else text.contains("2번 마을"), key)
			assert(not text.contains("42") and not text.to_lower().contains("famine") and not text.contains("기근"), key)
		assert(str(strings["HUD_FEED_MORE_FMT"]) % 5 != "HUD_FEED_MORE_FMT")
	print("FEED_WORDING PASS: all village slots, both locales, retired identities, no famine")
	quit()
