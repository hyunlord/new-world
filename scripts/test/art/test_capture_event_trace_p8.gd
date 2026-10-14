extends SceneTree

const Trace = preload("res://scripts/test/art/capture_event_trace_p8.gd")

func _initialize() -> void:
	var trace = Trace.new()
	trace.ingest({"tick": 0}, {"ids": [7]}, {"settlement_ids": [7], "parent_ids": [-1], "founded_ticks": [0]}, {"ids": [80]}, {"births": 2}, {}, {"ids": [1, 2]})
	assert(trace.rows().is_empty(), "Initial state establishes baseline without invented news")
	var society: Dictionary = {"settlement_ids": [7, 42], "parent_ids": [-1, 7], "founded_ticks": [0, 0], "fission_ids": [9], "fission_parents": [7], "fission_depart_ticks": [0], "fission_sizes": [13], "fission_founded_ticks": [0], "fission_daughters": [42], "fission_cancelled_ticks": [-1]}
	var casts: Dictionary = {"log_ids": [0, 1], "log_settlement_ids": [7, 7], "log_ticks": [1, 1], "log_kinds": ["gift_food", "gift_food"], "log_applied": [60, 60]}
	trace.ingest({"tick": 1}, {"ids": [7, 42]}, society, {"ids": [81]}, {"births": 4}, casts, {"ids": [1, 2, 3, 4]})
	var rows: Array[Dictionary] = trace.rows()
	assert(rows.size() == 8, "Departure/founding, two construction changes, two births, two equal casts")
	assert(rows[0]["event_id"] == "depart:9" and rows[0]["args_raw"] == [13, 7])
	assert(rows[1]["event_id"] == "founded:42", "Snapshot and log daughter are exactly one event")
	assert(rows[4]["event_id"] == "birth:3" and rows[5]["event_id"] == "birth:4" and rows[4]["count"] == 1 and rows[4]["args_raw"] == [4], "Birth count and current population remain distinct")
	assert(rows[6]["event_id"] == "cast:0" and rows[7]["event_id"] == "cast:1", "Same-tick identical casts preserve log identity")
	trace.ingest({"tick": 1}, {"ids": [7, 42]}, society, {"ids": [81]}, {"births": 4}, casts, {"ids": [1, 2, 3, 4]})
	assert(trace.rows().size() == 8, "Paused repeated snapshots do not duplicate events")
	society["fission_cancelled_ticks"] = [1]
	trace.ingest({"tick": 2}, {"ids": [42]}, society, {"ids": [81]}, {"births": 4}, casts)
	assert(trace.rows().size() == 10)
	assert(trace.rows()[8]["event_id"] == "cancel:9" and trace.rows()[9]["event_id"] == "abandoned:7")
	assert(trace.metadata()["coverage_complete"])
	trace.ingest({"tick": 3}, {"ids": [42]}, society, {"ids": [81]}, {"births": 5}, casts)
	assert(not trace.metadata()["coverage_complete"], "Missing agent snapshot must not fabricate birth population")
	trace.reset()
	trace.ingest({"tick": 0}, {}, {}, {}, {}, {})
	trace.ingest({"tick": 1}, {"ids": [90]}, {"settlement_ids": [90], "parent_ids": [-1], "founded_ticks": [0]}, {}, {}, {})
	assert(trace.rows()[0]["key"] == "HUD_FEED_SETTLEMENT_FORMED" and trace.rows()[0]["raw_settlement_id"] == 90)
	trace.ingest({"tick": 3}, {}, {}, {}, {}, {})
	assert(not trace.metadata()["coverage_complete"], "A skipped tick cannot claim complete census")
	trace.ingest({"tick": 0}, {}, {}, {}, {}, {})
	assert(trace.rows().is_empty() and trace.metadata()["coverage_complete"], "Clock rewind resets trace")
	print("CAPTURE EVENT TRACE P8 PASS: baseline, births, sites, society, same-tick casts, coverage, reset")
	quit(0)
