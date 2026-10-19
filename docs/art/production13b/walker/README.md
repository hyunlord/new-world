# Walker candidate, default OFF

Pure Rust replay:57 sim-bridge tests PASS, including560 actual Godot numerical cases and2000 renderer calls replayed in both spatial/brute modes (4000records). Fixture distributions:700 W200running,700 W-multirunning,600 W500frozen. Oracle captured before the new bridge was installed; exact input and source hashes are in manifests.

Cargo attempt1 failed compilation before any test because godot-rust supports at most14arguments. The new wrapper groups world dimensions in Vector2i; attempt2 passes. Clippy passes. Reference CampSeats remains unchanged.

Actual release bridge SHA30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042. Actual bridge attempt1:24bit-exact comparisons but22nonempty, FAIL coverage. Attempt2 adds10running warmupframes before attaching observer:12calls,24nonempty ordered-ID/f32 exact comparisons, unchanged inputs/source/HEAD, PASS. Assertions unchanged.

Identity and speed are still NOT_VERIFIED; do not enable by default from fixture or bridge success alone.
