# Part 2 verification

Six actual Godot scene tests passed: morning chill, cold presentation, idle facing, agent activity, meal pose, idle camp pose. The initial guessed filename `test_activity_frames.gd` does not exist; its failed discovery attempt is retained in `test_activity_frames.log` and was replaced by the actual `test_agent_activity.gd` test, not counted as a pass.

The merged game's clock is 1440 ticks/day, so one tick is one game minute. First observed clear tick starts the 90-minute window; last observed cold tick remains stored separately. Alpha stays 1 through minute60, fades linearly over minutes60–90 and is exactly0 at minute90 (07:30 for a06:00 clear), certainly absent at07:31. A repeated warm snapshot cannot extend the window. Capture chunks feed every8 ticks, well within the specified60-tick upper bound.

Memory is presentation-only and reset with the renderer's world-reset callback. Chill writes only `chill_*` keys and a `ChillBreath` child; existing cold metadata and `ColdBreath` remain independent. Original meal/work/walking/sleep poses win over crossed arms. The final natural captures and independent cold trace, not these unit tests, establish A22.
