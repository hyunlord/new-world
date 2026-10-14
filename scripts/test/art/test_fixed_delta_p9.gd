extends SceneTree

var samples: int = 0
var invalid_samples: int = 0

static func accepts_delta(delta: float) -> bool:
	return is_finite(delta) and absf(delta - 1.0 / 60.0) <= 0.000000001

func _initialize() -> void:
	print("P9 FIXED DELTA args=", OS.get_cmdline_args())
	print("P9 FIXED DELTA user_args=", OS.get_cmdline_user_args())

func _process(delta: float) -> bool:
	samples += 1
	if not accepts_delta(delta):
		invalid_samples += 1
	print("P9 FIXED DELTA sample=%d delta=%.15f valid=%s" % [samples, delta, accepts_delta(delta)])
	if samples == 5:
		var expectation: String = OS.get_environment("P9_EXPECT_FIXED_DELTA")
		var expected_valid: bool = expectation == "1"
		var observed_valid: bool = invalid_samples == 0
		print("P9 FIXED DELTA observed_valid=%s expected_valid=%s" % [observed_valid, expected_valid])
		var predicate_valid: bool = accepts_delta(1.0 / 60.0) and not accepts_delta(1.0 / 30.0) and not accepts_delta(1.0 / 59.0) and not accepts_delta(NAN)
		quit(0 if predicate_valid and (expectation.is_empty() or observed_valid == expected_valid) else 1)
	return false
